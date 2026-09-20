use crate::models::ValidationResult;
use crate::pattern_engine;
use crate::pattern_store::PatternStore;
use rmcp::{ErrorData, ServiceExt, handler::server::wrapper::Parameters, tool, tool_router, transport::stdio};
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

fn to_tool_error(err: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(err.to_string(), None)
}

fn to_json(value: &impl serde::Serialize) -> Result<String, ErrorData> {
    serde_json::to_string(value).map_err(to_tool_error)
}

/// The norma MCP tool server. Wraps a `PatternStore` and exposes the four
/// tools from the wayfinder map: validating code, listing the patterns
/// for a language, registering a new pattern, and listing every pattern.
/// This is the same core the `norma validate` CLI subcommand calls
/// (`cli.rs`), just reached over stdio instead -- see docs/adr/0001.md.
#[derive(Clone)]
pub struct NormaServer {
    store: Arc<PatternStore>,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<NormaServer>,
}

#[tool_router(server_handler)]
impl NormaServer {
    pub fn new(store: Arc<PatternStore>) -> Self {
        Self {
            store,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Validate source code against every enabled pattern registered for its language")]
    pub async fn validate_pattern_compliance(
        &self,
        Parameters(params): Parameters<ValidateParams>,
    ) -> Result<String, ErrorData> {
        let patterns = self
            .store
            .get_patterns_for_language(&params.language)
            .await
            .map_err(to_tool_error)?;
        let result: ValidationResult = pattern_engine::validate(&params.code, &params.language, &patterns);
        to_json(&result)
    }

    #[tool(description = "List the patterns that apply to a given language")]
    pub async fn get_pattern_checklist(
        &self,
        Parameters(params): Parameters<LanguageParams>,
    ) -> Result<String, ErrorData> {
        let patterns = self
            .store
            .get_patterns_for_language(&params.language)
            .await
            .map_err(to_tool_error)?;
        to_json(&patterns)
    }

    #[tool(description = "Register a new pattern from a full ast-grep RuleConfig YAML")]
    pub async fn register_pattern(
        &self,
        Parameters(params): Parameters<RegisterPatternParams>,
    ) -> Result<String, ErrorData> {
        let pattern = self
            .store
            .register_pattern(params.name, params.description, params.category, params.rule)
            .await
            .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
        to_json(&pattern)
    }

    #[tool(description = "List every registered pattern")]
    pub async fn list_patterns(&self) -> Result<String, ErrorData> {
        let patterns = self.store.list_all_patterns().await.map_err(to_tool_error)?;
        to_json(&patterns)
    }
}

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
