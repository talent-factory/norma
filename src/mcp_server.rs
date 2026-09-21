use crate::models::ValidationResult;
use crate::pattern_engine;
use crate::pattern_store::{PatternStore, RegisterPatternError};
use rmcp::{
    handler::server::wrapper::Parameters, tool, tool_handler, tool_router, transport::stdio,
    ErrorData, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateParams {
    /// Source code to validate.
    pub code: String,
    /// norma's canonical language key: "java" | "python" | "rust" | "typescript".
    pub language: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LanguageParams {
    /// norma's canonical language key: "java" | "python" | "rust" | "typescript".
    pub language: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TestPatternParams {
    /// A full ast-grep RuleConfig YAML document (id/message/severity/language/rule,
    /// optionally fix) -- the exact same shape `register_pattern`'s `rule`
    /// expects. Nothing is persisted; this only tests the rule.
    pub rule: String,
    /// Example source code to test the rule against.
    pub code: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RegisterPatternParams {
    pub name: String,
    pub description: String,
    // `serde(default)` is redundant for deserializing a missing field --
    // serde already defaults a missing `Option<T>` field to `None` -- but
    // schemars' derive only omits a field from the schema's `required` list
    // via *either* that attribute *or* recognizing the field's type as
    // `Option<T>`, and `schema_with` below swaps in a wrapper type that no
    // longer looks like `Option<T>` to it. Without this, `category` would
    // wrongly show up as required in the generated schema.
    #[serde(default)]
    #[schemars(schema_with = "optional_string_schema")]
    pub category: Option<String>,
    /// A full ast-grep RuleConfig YAML document
    /// (id/message/severity/language/rule, optionally fix). A `fix:` key
    /// makes the pattern usable by `apply_pattern_fix` and populates
    /// `validate_pattern_compliance`'s `suggested_fix` on a match; without
    /// one, the pattern can still be validated against, just not fixed.
    pub rule: String,
}

/// Renders `Option<String>` as `{"anyOf": [{"type": "string"}, {"type":
/// "null"}]}` instead of schemars' default `{"type": ["string", "null"]}`.
/// Both are valid JSON Schema, but several MCP clients read `type` as a
/// single string and either reject the tool or silently drop the null
/// branch -- this is the fix for the MCP Inspector's schema-portability
/// warning on `register_pattern`'s `category` field.
fn optional_string_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "anyOf": [
            { "type": "string" },
            { "type": "null" }
        ]
    })
}

/// Maps a store/engine failure to an MCP `internal_error`, logging it
/// first. `norma serve` is a long-running stdio process whose only
/// observability channel is stderr (see `main.rs`'s stdout/stderr split);
/// without this, every failure here -- a SQLite I/O error, a corrupted row
/// -- was visible only inside the one JSON-RPC error response sent back to
/// the client, with no local trail to diagnose a recurring problem from.
fn to_tool_error(err: impl std::fmt::Display) -> ErrorData {
    tracing::error!(error = %err, "MCP tool call failed");
    ErrorData::internal_error(err.to_string(), None)
}

/// Maps a client-caused failure (bad input) to an MCP `invalid_params`,
/// logging it at `warn` rather than `error` -- this is an expected
/// response to bad input, not a server fault.
fn to_invalid_params(err: impl std::fmt::Display) -> ErrorData {
    tracing::warn!(error = %err, "MCP tool call rejected invalid input");
    ErrorData::invalid_params(err.to_string(), None)
}

fn to_json(value: &impl serde::Serialize) -> Result<String, ErrorData> {
    serde_json::to_string(value).map_err(to_tool_error)
}

/// The norma MCP tool server. Wraps a `PatternStore` and exposes six
/// tools: validating code, applying registered patterns' fixes to code,
/// listing the patterns for a language, dry-running a candidate rule
/// before registering it, registering a new pattern, and listing every
/// pattern. This is the same core the `norma validate` CLI subcommand
/// calls (`cli.rs`), just reached over stdio instead -- see
/// docs/adr/0001.md.
#[derive(Clone)]
pub struct NormaServer {
    store: Arc<PatternStore>,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<NormaServer>,
}

#[tool_router]
impl NormaServer {
    pub fn new(store: Arc<PatternStore>) -> Self {
        Self {
            store,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Validate source code against every enabled pattern registered for its language"
    )]
    pub async fn validate_pattern_compliance(
        &self,
        Parameters(params): Parameters<ValidateParams>,
    ) -> Result<String, ErrorData> {
        // Reject an unsupported/misspelled language up front, as
        // `invalid_params` rather than a falsely-green result: matching zero
        // patterns would otherwise report `passed: true` and silently
        // disable the check.
        let language =
            pattern_engine::resolve_language(&params.language).map_err(to_invalid_params)?;
        let patterns = self
            .store
            .get_patterns_for_language(language)
            .await
            .map_err(to_tool_error)?;
        let result: ValidationResult = pattern_engine::validate(&params.code, language, &patterns)
            .map_err(to_invalid_params)?;
        to_json(&result)
    }

    /// Applies every enabled pattern's `fix`/`fixer` to `params.code` and
    /// returns the rewritten source, never writing to disk itself -- the
    /// caller decides what to do with the returned string (see TF-890:
    /// this stays deliberately separate from `validate_pattern_compliance`,
    /// with no `apply_fixes` flag added to it, to keep read-only and
    /// write-shaped tools apart). Reuses `ValidateParams`: same `code` +
    /// `language` shape as `validate_pattern_compliance`.
    #[tool(
        description = "Rewrite source code by applying every enabled pattern's fix, and return the rewritten code (never written to disk)"
    )]
    pub async fn apply_pattern_fix(
        &self,
        Parameters(params): Parameters<ValidateParams>,
    ) -> Result<String, ErrorData> {
        let language =
            pattern_engine::resolve_language(&params.language).map_err(to_invalid_params)?;
        let patterns = self
            .store
            .get_patterns_for_language(language)
            .await
            .map_err(to_tool_error)?;
        let result = pattern_engine::apply_fixes(&params.code, language, &patterns)
            .map_err(to_invalid_params)?;
        to_json(&result)
    }

    #[tool(description = "List the patterns that apply to a given language")]
    pub async fn get_pattern_checklist(
        &self,
        Parameters(params): Parameters<LanguageParams>,
    ) -> Result<String, ErrorData> {
        let language =
            pattern_engine::resolve_language(&params.language).map_err(to_invalid_params)?;
        let patterns = self
            .store
            .get_patterns_for_language(language)
            .await
            .map_err(to_tool_error)?;
        to_json(&patterns)
    }

    /// Dry-runs a candidate rule against example code without registering
    /// it (TF-891) -- the gap the README's "Adopting an existing ast-grep
    /// rule" section used to describe as a workaround (register first via
    /// `register_pattern`, then check it worked via
    /// `validate_pattern_compliance`). Applies the exact same structural
    /// checks `register_pattern` would (non-empty `id`, a language norma
    /// supports -- see `pattern_engine::check_registerable`), so a rule
    /// this accepts is guaranteed to also be accepted by `register_pattern`.
    ///
    /// Deliberately has no `dump_syntax_tree` equivalent: unlike this tool
    /// (which runs the rule through norma's own validation pipeline,
    /// including its language allowlist), a raw AST dump would be a
    /// value-free clone of `ast-grep-mcp`'s own tool -- see this crate's
    /// "norma vs. ast-grep-mcp" README section and
    /// .scratch/ast-grep-feature-parity/issues/02-rule-testing-ast-debug-tooling.md.
    #[tool(
        description = "Dry-run a candidate ast-grep RuleConfig YAML against example code without registering it -- returns matches (with suggested_fix, if the rule has a fix). For raw AST inspection instead of testing a rule, use ast-grep-mcp's dump_syntax_tree."
    )]
    pub async fn test_pattern(
        &self,
        Parameters(params): Parameters<TestPatternParams>,
    ) -> Result<String, ErrorData> {
        let result =
            pattern_engine::test_rule(&params.rule, &params.code).map_err(to_invalid_params)?;
        to_json(&result)
    }

    #[tool(description = "Register a new pattern from a full ast-grep RuleConfig YAML")]
    pub async fn register_pattern(
        &self,
        Parameters(params): Parameters<RegisterPatternParams>,
    ) -> Result<String, ErrorData> {
        // `register_pattern` distinguishes "your rule was invalid" from "the
        // database write failed" -- see `RegisterPatternError`'s doc comment.
        // Conflating them (as a single `anyhow::Error` would) risks an MCP
        // client interpreting a storage fault as its own mistake to retry.
        let pattern = self
            .store
            .register_pattern(
                params.name,
                params.description,
                params.category,
                params.rule,
            )
            .await
            .map_err(|err| match err {
                RegisterPatternError::InvalidRule(e) => to_invalid_params(e),
                RegisterPatternError::Storage(e) => to_tool_error(e),
            })?;
        to_json(&pattern)
    }

    #[tool(description = "List every registered pattern")]
    pub async fn list_patterns(&self) -> Result<String, ErrorData> {
        let patterns = self
            .store
            .list_all_patterns()
            .await
            .map_err(to_tool_error)?;
        to_json(&patterns)
    }
}

// Explicit `router = self.tool_router` rather than the `#[tool_router(server_handler)]`
// shorthand: that shorthand's generated `ServerHandler` impl calls
// `Self::tool_router()` fresh on every request, rebuilding the whole tool
// table each time instead of reusing the one built once in `new()` and
// stored above -- functionally harmless, but wasted work on every call.
#[tool_handler(router = self.tool_router)]
impl ServerHandler for NormaServer {}

/// Runs the norma MCP server on stdio until the client disconnects.
pub async fn serve(store: Arc<PatternStore>) -> anyhow::Result<()> {
    let service = NormaServer::new(store).serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Pattern;

    /// Builds a `NormaServer` over a temporary, seeded store. Tool methods
    /// are plain async functions, so tests call them directly -- no stdio
    /// transport or MCP client needed.
    async fn test_server() -> NormaServer {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("norma-test.db");
        let store = PatternStore::new(&db_path).await.unwrap();
        store.seed_defaults().await.unwrap();
        std::mem::forget(dir);
        NormaServer::new(Arc::new(store))
    }

    #[tokio::test]
    async fn validate_pattern_compliance_reports_a_violation() {
        let server = test_server().await;
        let params = Parameters(ValidateParams {
            code: "fn main() { println!(\"debug\"); }".to_string(),
            language: "rust".to_string(),
        });
        let json = server.validate_pattern_compliance(params).await.unwrap();
        let result: ValidationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.violations.len(), 1);
    }

    #[tokio::test]
    async fn list_patterns_returns_the_seeded_defaults() {
        let server = test_server().await;
        let json = server.list_patterns().await.unwrap();
        let patterns: Vec<Pattern> = serde_json::from_str(&json).unwrap();
        assert_eq!(patterns.len(), 20);
    }

    #[tokio::test]
    async fn validate_pattern_compliance_rejects_an_unsupported_language() {
        let server = test_server().await;
        let params = Parameters(ValidateParams {
            code: "package main".to_string(),
            language: "go".to_string(),
        });
        let err = server
            .validate_pattern_compliance(params)
            .await
            .expect_err("an unsupported language must not report a passing result");
        assert!(
            err.message.contains("unsupported language"),
            "unexpected error: {err:?}"
        );
    }

    // --- test_pattern (TF-891) -----------------------------------------

    #[tokio::test]
    async fn test_pattern_reports_a_match_without_registering_anything() {
        let server = test_server().await;
        let params = Parameters(TestPatternParams {
            rule: r#"
id: no-debug-print-rust
message: Avoid println! in production code
severity: warning
language: Rust
rule:
  pattern: println!($$$ARGS)
"#
            .to_string(),
            code: "fn main() { println!(\"debug\"); }".to_string(),
        });
        let json = server.test_pattern(params).await.unwrap();
        let result: ValidationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.violations.len(), 1);
        assert!(!result.passed);

        // Nothing was persisted -- the seeded defaults are still the only
        // registered patterns.
        let patterns_json = server.list_patterns().await.unwrap();
        let patterns: Vec<Pattern> = serde_json::from_str(&patterns_json).unwrap();
        assert_eq!(patterns.len(), 20);
    }

    #[tokio::test]
    async fn test_pattern_reports_suggested_fix_from_the_rules_own_fix() {
        let server = test_server().await;
        let params = Parameters(TestPatternParams {
            rule: r#"
id: no-unwrap-rust
message: Avoid unwrap() in production code
severity: warning
language: Rust
rule:
  pattern: $EXPR.unwrap()
fix: $EXPR.expect("TODO")
"#
            .to_string(),
            code: "fn main() { value.unwrap(); }".to_string(),
        });
        let json = server.test_pattern(params).await.unwrap();
        let result: ValidationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(
            result.violations[0].suggested_fix.as_deref(),
            Some(r#"value.expect("TODO")"#)
        );
    }

    #[tokio::test]
    async fn test_pattern_rejects_malformed_yaml_as_invalid_params() {
        let server = test_server().await;
        let params = Parameters(TestPatternParams {
            rule: "not: valid: yaml: at: all: -".to_string(),
            code: "fn main() {}".to_string(),
        });
        let err = server
            .test_pattern(params)
            .await
            .expect_err("malformed rule YAML must be rejected");
        assert!(!err.message.is_empty());
    }

    #[tokio::test]
    async fn test_pattern_rejects_a_rule_with_no_id_the_same_way_register_pattern_would() {
        let server = test_server().await;
        let params = Parameters(TestPatternParams {
            rule: r#"
message: Avoid println! in production code
severity: warning
language: Rust
rule:
  pattern: println!($$$ARGS)
"#
            .to_string(),
            code: "fn main() {}".to_string(),
        });
        let err = server
            .test_pattern(params)
            .await
            .expect_err("a rule with no id must be rejected, mirroring register_pattern");
        assert!(
            err.message.contains("non-empty"),
            "unexpected error: {err:?}"
        );
    }

    #[tokio::test]
    async fn register_pattern_rejects_malformed_yaml_as_invalid_params() {
        let server = test_server().await;
        let params = Parameters(RegisterPatternParams {
            name: "Broken".to_string(),
            description: "d".to_string(),
            category: None,
            rule: "not: valid: yaml: at: all: -".to_string(),
        });
        let result = server.register_pattern(params).await;
        assert!(result.is_err());
    }

    /// Pins the schema shape `optional_string_schema` exists to produce, so
    /// a future schemars upgrade or a "this attribute looks redundant"
    /// cleanup of `#[serde(default)]` fails loudly instead of silently
    /// reintroducing the MCP Inspector portability warning this schema was
    /// fixed for (see the doc comments on `RegisterPatternParams::category`
    /// and `optional_string_schema`).
    #[test]
    fn register_pattern_schema_keeps_category_optional_and_portable() {
        let schema = serde_json::to_value(schemars::schema_for!(RegisterPatternParams)).unwrap();

        let category = &schema["properties"]["category"];
        assert!(
            category["anyOf"].is_array() && category["type"].is_null(),
            "category must render as anyOf, not a `type` array: {category}"
        );

        let required = schema["required"].as_array().unwrap();
        assert!(
            !required.iter().any(|field| field == "category"),
            "category must stay out of `required`: {required:?}"
        );
    }

    // --- apply_pattern_fix (TF-890) -----------------------------------

    async fn register_unwrap_fixer(server: &NormaServer) {
        let params = Parameters(RegisterPatternParams {
            name: "No Unwrap".to_string(),
            description: "d".to_string(),
            category: None,
            rule: r#"
id: no-unwrap-rust
message: Avoid unwrap() in production code
severity: warning
language: Rust
rule:
  pattern: $EXPR.unwrap()
fix: $EXPR.expect("TODO")
"#
            .to_string(),
        });
        server.register_pattern(params).await.unwrap();
    }

    #[tokio::test]
    async fn apply_pattern_fix_rewrites_the_code_and_never_touches_disk() {
        use crate::models::FixResult;

        let server = test_server().await;
        register_unwrap_fixer(&server).await;

        let params = Parameters(ValidateParams {
            code: "fn main() { value.unwrap(); }".to_string(),
            language: "rust".to_string(),
        });
        let json = server.apply_pattern_fix(params).await.unwrap();
        let result: FixResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.applied_count, 1);
        assert!(result.conflicts.is_empty());
        assert_eq!(
            result.fixed_source,
            r#"fn main() { value.expect("TODO"); }"#
        );
    }

    #[tokio::test]
    async fn apply_pattern_fix_leaves_code_without_a_fixable_match_unchanged() {
        use crate::models::FixResult;

        let server = test_server().await;
        register_unwrap_fixer(&server).await;

        let params = Parameters(ValidateParams {
            code: "fn main() {}".to_string(),
            language: "rust".to_string(),
        });
        let json = server.apply_pattern_fix(params).await.unwrap();
        let result: FixResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.applied_count, 0);
        assert_eq!(result.fixed_source, "fn main() {}");
    }

    #[tokio::test]
    async fn apply_pattern_fix_rejects_an_unsupported_language() {
        let server = test_server().await;
        let params = Parameters(ValidateParams {
            code: "package main".to_string(),
            language: "go".to_string(),
        });
        let err = server
            .apply_pattern_fix(params)
            .await
            .expect_err("an unsupported language must be rejected, not silently no-op'd");
        assert!(
            err.message.contains("unsupported language"),
            "unexpected error: {err:?}"
        );
    }
}
