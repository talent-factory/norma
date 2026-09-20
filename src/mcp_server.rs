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
pub struct RegisterPatternParams {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    /// A full ast-grep RuleConfig YAML document (id/message/severity/language/rule).
    pub rule: String,
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

/// The norma MCP tool server. Wraps a `PatternStore` and exposes four
/// tools: validating code, listing the patterns for a language,
/// registering a new pattern, and listing every pattern. This is the same
/// core the `norma validate` CLI subcommand calls (`cli.rs`), just reached
/// over stdio instead -- see docs/adr/0001.md.
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
        assert_eq!(patterns.len(), 4);
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
}
