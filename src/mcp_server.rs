use crate::models::*;
use crate::pattern_engine::PatternEngine;
use crate::pattern_store::PatternStore;
use mcpkit::prelude::*;
use serde_json::json;
use std::sync::Arc;
use tracing::{info, error};

pub struct NormaMcpServer {
    store: Arc<PatternStore>,
    engine: PatternEngine,
}

impl NormaMcpServer {
    pub fn new(store: Arc<PatternStore>) -> Self {
        Self {
            store,
            engine: PatternEngine::new(),
        }
    }

    /// Validate code against patterns
    pub async fn validate_pattern_compliance(
        &self,
        code: String,
        language: String,
        file_path: Option<String>,
    ) -> ToolOutput {
        let start = std::time::Instant::now();
        
        info!("Validating {} code ({} bytes)", language, code.len());

        // Get all enabled patterns for this language
        match self.store.get_patterns_for_language(&language).await {
            Ok(patterns) => {
                if patterns.is_empty() {
                    return ToolOutput::text(format!(
                        "No patterns defined for language: {}",
                        language
                    ));
                }

                // Run validation
                match self.engine.validate_code(
                    &code,
                    &language,
                    patterns.clone(),
                ) {
                    Ok(violations) => {
                        let duration = start.elapsed();
                        let passed = violations.is_empty();
                        let score = if passed { 1.0 } else { 0.5 };

                        let result = ValidationResult {
                            violations,
                            passed,
                            score,
                            duration_ms: duration.as_millis(),
                        };

                        ToolOutput::json(result)
                    }
                    Err(e) => {
                        error!("Validation error: {}", e);
                        ToolOutput::text(format!("Validation error: {}", e))
                    }
                }
            }
            Err(e) => {
                error!("Failed to load patterns: {}", e);
                ToolOutput::text(format!("Error loading patterns: {}", e))
            }
        }
    }

    /// Get applicable patterns for a language
    pub async fn get_pattern_checklist(&self, language: String) -> ToolOutput {
        match self.store.get_patterns_for_language(&language).await {
            Ok(patterns) => {
                let checklist: Vec<_> = patterns
                    .iter()
                    .map(|p| {
                        json!({
                            "id": p.id,
                            "name": p.name,
                            "description": p.description,
                            "severity": format!("{:?}", p.severity).to_lowercase(),
                            "enabled": p.enabled,
                        })
                    })
                    .collect();

                ToolOutput::json(checklist)
            }
            Err(e) => {
                error!("Failed to load pattern checklist: {}", e);
                ToolOutput::text(format!("Error: {}", e))
            }
        }
    }

    /// Register a new pattern
    pub async fn register_pattern(
        &self,
        name: String,
        description: String,
        rule: String,
        languages: Vec<String>,
        severity: Option<String>,
    ) -> ToolOutput {
        let sev = match severity.as_deref() {
            Some("error") => Severity::Error,
            Some("warning") => Severity::Warning,
            Some("critical") => Severity::Critical,
            _ => Severity::Warning,
        };

        let mut pattern = Pattern::new(name.clone(), description, rule, languages);
        pattern.severity = sev;

        match self.store.save_pattern(pattern).await {
            Ok(saved_pattern) => {
                info!("Pattern registered: {}", name);
                ToolOutput::json(json!({
                    "success": true,
                    "pattern_id": saved_pattern.id,
                    "name": saved_pattern.name,
                }))
            }
            Err(e) => {
                error!("Failed to register pattern: {}", e);
                ToolOutput::text(format!("Error: {}", e))
            }
        }
    }

    /// List all registered patterns
    pub async fn list_patterns(&self) -> ToolOutput {
        match self.store.list_all_patterns().await {
            Ok(patterns) => ToolOutput::json(patterns),
            Err(e) => {
                error!("Failed to list patterns: {}", e);
                ToolOutput::text(format!("Error: {}", e))
            }
        }
    }
}

#[mcp_server(name = "norma", version = "0.1.0")]
impl NormaMcpServer {
    #[tool(description = "Validate code against design patterns")]
    async fn validate_pattern_compliance(
        &self,
        code: String,
        language: String,
        #[serde(default)]
        file_path: Option<String>,
    ) -> ToolOutput {
        self.validate_pattern_compliance(code, language, file_path)
            .await
    }

    #[tool(description = "Get applicable patterns for a programming language")]
    async fn get_pattern_checklist(&self, language: String) -> ToolOutput {
        self.get_pattern_checklist(language).await
    }

    #[tool(description = "Register a new pattern rule")]
    async fn register_pattern(
        &self,
        name: String,
        description: String,
        rule: String,
        languages: Vec<String>,
        #[serde(default)]
        severity: Option<String>,
    ) -> ToolOutput {
        self.register_pattern(name, description, rule, languages, severity)
            .await
    }

    #[tool(description = "List all registered patterns")]
    async fn list_patterns(&self) -> ToolOutput {
        self.list_patterns().await
    }
}
