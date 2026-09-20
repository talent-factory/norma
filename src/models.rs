use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Severity level for a pattern violation.
///
/// Mirrors `ast_grep_config::rule_config::Severity` (see docs/adr/0002.md
/// and `pattern_engine::parse_rule`): norma derives this from the parsed
/// rule YAML at register time rather than accepting it as a separate
/// input, so a `Pattern`'s severity can never drift from the YAML that
/// actually produced it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Off,
    Hint,
    Info,
    Warning,
    Error,
}

impl Severity {
    /// Parses the lowercase string form used in the `patterns.severity`
    /// SQLite column (see `as_str`). Returns `None` for anything else.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "off" => Some(Severity::Off),
            "hint" => Some(Severity::Hint),
            "info" => Some(Severity::Info),
            "warning" => Some(Severity::Warning),
            "error" => Some(Severity::Error),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Off => "off",
            Severity::Hint => "hint",
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

/// A registered code pattern.
///
/// Per docs/adr/0002.md, one row is always single-language: an idea that
/// should hold across several languages (e.g. "no debug prints") becomes
/// several `Pattern` rows, one per language, related only by a shared
/// `name` -- there is no data-level link between them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pattern {
    /// The ast-grep RuleConfig `id` from the YAML in `rule`, e.g. `"no-debug-print-rust"`.
    pub id: String,
    /// norma's catalog name. Repeats across the language variants of one idea.
    pub name: String,
    /// norma's own, longer explanation (ast-grep's own `message` field
    /// inside `rule` is documented as "should be single line and concise").
    pub description: String,
    /// Free-text grouping, e.g. `"code-quality"`, `"creational"`. Not a fixed enum.
    pub category: Option<String>,
    /// norma's canonical language key: `"java"` | `"python"` | `"rust"` | `"typescript"`.
    pub language: String,
    /// Derived from the parsed `rule` YAML when the pattern was registered.
    pub severity: Severity,
    /// The full ast-grep RuleConfig YAML document (id/message/severity/language/rule/fix).
    pub rule: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single match of a `Pattern` against a piece of code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatternViolation {
    pub pattern_id: String,
    pub pattern_name: String,
    pub severity: Severity,
    pub location: CodeLocation,
    pub matched_text: String,
    pub message: String,
}

/// A position ast-grep matched at. `line` and `column` are zero-based, as
/// ast-grep-core's own `Position` type documents them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeLocation {
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
}

/// The result of validating one piece of code against a set of patterns.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationResult {
    pub violations: Vec<PatternViolation>,
    pub passed: bool,
    /// 1.0 = no violations relative to the number of patterns checked, 0.0 = worst case.
    pub score: f64,
    pub duration_ms: u128,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_round_trips_through_its_string_form() {
        for severity in [
            Severity::Off,
            Severity::Hint,
            Severity::Info,
            Severity::Warning,
            Severity::Error,
        ] {
            assert_eq!(Severity::parse(severity.as_str()), Some(severity));
        }
    }

    #[test]
    fn severity_parse_rejects_unknown_strings() {
        assert_eq!(Severity::parse("critical"), None);
        assert_eq!(Severity::parse(""), None);
    }
}
