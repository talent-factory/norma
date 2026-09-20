use crate::models::{CodeLocation, Pattern, PatternViolation, Severity, ValidationResult};
use ast_grep_config::{GlobalRules, RuleConfig, from_yaml_string};
use ast_grep_language::{LanguageExt, SupportLang};
use std::time::Instant;

/// Parses a full ast-grep RuleConfig YAML document (see docs/adr/0002.md)
/// and returns the compiled rule, or an error if the YAML is malformed or
/// the rule has no matchable AST kinds. Used both to run a pattern
/// (`validate`, below) and to validate one before it's persisted
/// (`PatternStore::register_pattern` in `pattern_store.rs`).
pub fn parse_rule(rule_yaml: &str) -> anyhow::Result<RuleConfig<SupportLang>> {
    let globals = GlobalRules::default();
    let mut configs = from_yaml_string::<SupportLang>(rule_yaml, &globals)?;
    if configs.is_empty() {
        anyhow::bail!("rule YAML did not contain a RuleConfig document");
    }
    Ok(configs.remove(0))
}

/// norma's own canonical language key for a `SupportLang`, matching the
/// strings used in the `patterns.language` SQLite column and the MCP tool
/// parameters (`"java"`, `"python"`, `"rust"`, `"typescript"`).
/// ast-grep-language's own alias lists are inconsistent for this purpose
/// (Rust's first alias is `"rs"`, TypeScript's is `"ts"`), so norma keeps
/// its own explicit mapping for the four MVP languages.
pub fn language_key(lang: SupportLang) -> &'static str {
    match lang {
        SupportLang::Java => "java",
        SupportLang::Python => "python",
        SupportLang::Rust => "rust",
        SupportLang::TypeScript => "typescript",
        _ => "unsupported",
    }
}

/// Runs one already-parsed rule against `source` and returns every match
/// as a `PatternViolation`, tagged with `pattern`'s catalog metadata.
fn find_violations(
    pattern: &Pattern,
    config: &RuleConfig<SupportLang>,
    source: &str,
) -> Vec<PatternViolation> {
    let grep = config.language.ast_grep(source);
    grep.root()
        .find_all(&config.matcher)
        .map(|node_match| {
            let pos = node_match.start_pos();
            PatternViolation {
                pattern_id: pattern.id.clone(),
                pattern_name: pattern.name.clone(),
                severity: pattern.severity,
                location: CodeLocation {
                    file: None,
                    line: pos.line(),
                    column: pos.column(&node_match),
                },
                matched_text: node_match.text().to_string(),
                message: pattern.description.clone(),
            }
        })
        .collect()
}

/// Runs every enabled pattern whose `language` matches `language` against
/// `source`, and aggregates the result. Shared by the MCP
/// `validate_pattern_compliance` tool (`mcp_server.rs`) and the `norma
/// validate` CLI subcommand (`cli.rs`) -- see docs/adr/0001.md: one core,
/// no separate sync/async implementation. A pattern whose stored `rule`
/// no longer parses is skipped with a warning rather than failing the
/// whole run.
pub fn validate(source: &str, language: &str, patterns: &[Pattern]) -> ValidationResult {
    let start = Instant::now();
    let mut violations = Vec::new();
    let mut checked = 0usize;
    for pattern in patterns.iter().filter(|p| p.enabled && p.language == language) {
        checked += 1;
        match parse_rule(&pattern.rule) {
            Ok(config) => violations.extend(find_violations(pattern, &config, source)),
            Err(err) => {
                tracing::warn!(pattern_id = %pattern.id, error = %err, "skipping pattern with unparsable rule");
            }
        }
    }
    let passed = violations.is_empty();
    let score = if checked == 0 {
        1.0
    } else {
        (1.0 - violations.len() as f64 / checked as f64).max(0.0)
    };
    ValidationResult {
        violations,
        passed,
        score,
        duration_ms: start.elapsed().as_millis(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn test_pattern(id: &str, language: &str, rule: &str) -> Pattern {
        let now = Utc::now();
        Pattern {
            id: id.to_string(),
            name: "Test Pattern".to_string(),
            description: "test".to_string(),
            category: None,
            language: language.to_string(),
            severity: Severity::Warning,
            rule: rule.to_string(),
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    const RUST_NO_DEBUG_PRINT: &str = r#"
id: no-debug-print-rust
message: Avoid println! in production code
severity: warning
language: Rust
rule:
  pattern: println!($$$ARGS)
"#;

    #[test]
    fn validate_finds_a_real_violation() {
        let pattern = test_pattern("no-debug-print-rust", "rust", RUST_NO_DEBUG_PRINT);
        let source = "fn main() { println!(\"debug\"); }";
        let result = validate(source, "rust", &[pattern]);
        assert_eq!(result.violations.len(), 1);
        assert!(!result.passed);
        assert_eq!(result.violations[0].matched_text, "println!(\"debug\")");
    }

    #[test]
    fn validate_reports_no_violations_for_clean_code() {
        let pattern = test_pattern("no-debug-print-rust", "rust", RUST_NO_DEBUG_PRINT);
        let source = "fn main() { tracing::info!(\"structured, fine\"); }";
        let result = validate(source, "rust", &[pattern]);
        assert!(result.violations.is_empty());
        assert!(result.passed);
        assert_eq!(result.score, 1.0);
    }

    #[test]
    fn validate_ignores_patterns_for_other_languages() {
        let pattern = test_pattern("no-debug-print-rust", "rust", RUST_NO_DEBUG_PRINT);
        // Would match if the language filter didn't work -- this source is
        // deliberately Rust syntax passed under a "python" validation request.
        let source = "println!(\"debug\")";
        let result = validate(source, "python", &[pattern]);
        assert!(result.violations.is_empty());
        assert_eq!(result.score, 1.0); // no patterns were checked for "python"
    }

    #[test]
    fn parse_rule_rejects_malformed_yaml() {
        assert!(parse_rule("not: valid: yaml: at: all: -").is_err());
    }
}
