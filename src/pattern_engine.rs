use crate::models::{CodeLocation, Pattern, PatternViolation, ValidationResult};
use ast_grep_config::{from_yaml_string, GlobalRules, RuleConfig};
use ast_grep_language::{LanguageExt, SupportLang};
use std::str::FromStr;
use std::time::Instant;

/// Every language key norma supports, in the order shown in error
/// messages. Kept in sync with `language_key` below.
pub const SUPPORTED_LANGUAGES: [&str; 4] = ["java", "python", "rust", "typescript"];

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
///
/// Returns `None` for every other `SupportLang`. Callers MUST treat that
/// as an error rather than substituting a placeholder: a pattern stored
/// under a placeholder key would be permanently unreachable, and a
/// validation run against one would silently check nothing and report
/// `passed: true`.
pub fn language_key(lang: SupportLang) -> Option<&'static str> {
    match lang {
        SupportLang::Java => Some("java"),
        SupportLang::Python => Some("python"),
        SupportLang::Rust => Some("rust"),
        SupportLang::TypeScript => Some("typescript"),
        _ => None,
    }
}

/// Resolves a user-supplied language string (`--language`, or the MCP
/// tools' `language` parameter) to norma's canonical key, accepting any
/// ast-grep alias for a supported language (`"rs"` -> `"rust"`) and
/// rejecting everything else. Unsupported *and* misspelled languages must
/// fail loudly -- silently matching zero patterns would turn a typo into a
/// green "0 violations" result.
pub fn resolve_language(language: &str) -> anyhow::Result<&'static str> {
    SupportLang::from_str(language)
        .ok()
        .and_then(language_key)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "unsupported language: {language:?} (norma supports: {})",
                SUPPORTED_LANGUAGES.join(", ")
            )
        })
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
///
/// Fails if `language` is not one of norma's supported languages -- see
/// `resolve_language`. `language` may be any ast-grep alias for a
/// supported language; it is normalised to norma's canonical key before
/// patterns are filtered.
pub fn validate(
    source: &str,
    language: &str,
    patterns: &[Pattern],
) -> anyhow::Result<ValidationResult> {
    let language = resolve_language(language)?;
    let start = Instant::now();
    let mut violations = Vec::new();
    let mut checked = 0usize;
    for pattern in patterns
        .iter()
        .filter(|p| p.enabled && p.language == language)
    {
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
    Ok(ValidationResult {
        violations,
        passed,
        score,
        duration_ms: start.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Severity;
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
        let result = validate(source, "rust", &[pattern]).unwrap();
        assert_eq!(result.violations.len(), 1);
        assert!(!result.passed);
        assert_eq!(result.violations[0].matched_text, "println!(\"debug\")");
    }

    #[test]
    fn validate_reports_no_violations_for_clean_code() {
        let pattern = test_pattern("no-debug-print-rust", "rust", RUST_NO_DEBUG_PRINT);
        let source = "fn main() { tracing::info!(\"structured, fine\"); }";
        let result = validate(source, "rust", &[pattern]).unwrap();
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
        let result = validate(source, "python", &[pattern]).unwrap();
        assert!(result.violations.is_empty());
        assert_eq!(result.score, 1.0); // no patterns were checked for "python"
    }

    #[test]
    fn parse_rule_rejects_malformed_yaml() {
        assert!(parse_rule("not: valid: yaml: at: all: -").is_err());
    }

    #[test]
    fn language_key_is_none_for_languages_outside_the_mvp_set() {
        assert_eq!(language_key(SupportLang::Rust), Some("rust"));
        assert_eq!(language_key(SupportLang::Go), None);
        assert_eq!(language_key(SupportLang::Css), None);
    }

    #[test]
    fn resolve_language_accepts_canonical_keys_and_ast_grep_aliases() {
        assert_eq!(resolve_language("rust").unwrap(), "rust");
        assert_eq!(resolve_language("rs").unwrap(), "rust");
        assert_eq!(resolve_language("ts").unwrap(), "typescript");
        assert_eq!(resolve_language("Java").unwrap(), "java");
    }

    #[test]
    fn resolve_language_rejects_unsupported_and_misspelled_languages() {
        // A real ast-grep language norma has no patterns for...
        assert!(resolve_language("go").is_err());
        // ...and something that isn't a language at all.
        assert!(resolve_language("rustt").is_err());
    }

    #[test]
    fn validate_errors_instead_of_falsely_passing_an_unsupported_language() {
        let pattern = test_pattern("no-debug-print-rust", "rust", RUST_NO_DEBUG_PRINT);
        let err = validate("package main", "go", &[pattern]).unwrap_err();
        assert!(
            err.to_string().contains("unsupported language"),
            "unexpected error: {err}"
        );
    }
}
