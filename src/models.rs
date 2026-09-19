use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Severity level for pattern violations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// A code pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule: String,               // AST-grep pattern
    pub rewrite: Option<String>,    // Optional rewrite rule
    pub severity: Severity,
    pub languages: Vec<String>,     // e.g., ["java", "typescript"]
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Pattern {
    /// Create a new pattern with defaults
    pub fn new(
        name: String,
        description: String,
        rule: String,
        languages: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            rule,
            rewrite: None,
            severity: Severity::Warning,
            languages,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// A violation found in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternViolation {
    pub pattern_id: String,
    pub pattern_name: String,
    pub severity: Severity,
    pub location: CodeLocation,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Location in code where violation was found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Result of pattern validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub violations: Vec<PatternViolation>,
    pub passed: bool,
    pub score: f64,  // 0.0 - 1.0
    pub duration_ms: u128,
}

/// Request to validate code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub code: String,
    pub language: String,
    pub file_path: Option<String>,
    pub patterns: Option<Vec<String>>, // Pattern IDs to check, None = all
}

/// Pattern registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPatternRequest {
    pub name: String,
    pub description: String,
    pub rule: String,
    pub languages: Vec<String>,
    pub severity: Option<Severity>,
}
