use crate::models::{CodeLocation, Pattern, PatternViolation, Severity, ValidationResult};
use ast_grep_config::{from_yaml_string, GlobalRules, RuleConfig, Severity as AstGrepSeverity};
use ast_grep_language::{LanguageExt, SupportLang};
use chrono::{DateTime, Utc};
use std::str::FromStr;
use std::time::Instant;

/// Pattern id used for the synthetic warning `validate` adds when zero
/// enabled patterns are registered for the requested language. Never a
/// real registered pattern's id -- see `Pattern::from_rule`'s doc comment
/// on why a real `Pattern` can't be constructed with an arbitrary id.
pub const NO_COVERAGE_PATTERN_ID: &str = "__norma_no_coverage__";

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

fn severity_from_ast_grep(severity: &AstGrepSeverity) -> Severity {
    match severity {
        AstGrepSeverity::Off => Severity::Off,
        AstGrepSeverity::Hint => Severity::Hint,
        AstGrepSeverity::Info => Severity::Info,
        AstGrepSeverity::Warning => Severity::Warning,
        AstGrepSeverity::Error => Severity::Error,
    }
}

impl Pattern {
    /// The only way to construct a `Pattern`: parses `rule` (a full
    /// ast-grep RuleConfig YAML, see docs/adr/0002.md) and derives `id`,
    /// `language`, and `severity` from it. They can never be supplied
    /// independently, so they can never drift from the YAML that actually
    /// produced them. Fails if `rule` doesn't parse, has an empty `id`, or
    /// targets a language norma doesn't support -- a rule for an
    /// unsupported language is rejected here rather than stored, because a
    /// stored one would sit under a language key no `PatternStore` lookup
    /// could ever reach.
    pub fn from_rule(
        name: String,
        description: String,
        category: Option<String>,
        rule: String,
        enabled: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let config = parse_rule(&rule)?;
        if config.id.is_empty() {
            anyhow::bail!("rule YAML must set a non-empty top-level `id`");
        }
        let Some(language) = language_key(config.language) else {
            anyhow::bail!(
                "unsupported language: {:?} (norma supports: {})",
                config.language,
                SUPPORTED_LANGUAGES.join(", ")
            );
        };
        Ok(Pattern::from_trusted_row(
            config.id.clone(),
            name,
            description,
            category,
            language.to_string(),
            severity_from_ast_grep(&config.severity),
            rule,
            enabled,
            created_at,
            updated_at,
        ))
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
                pattern_id: pattern.id().to_string(),
                pattern_name: pattern.name.clone(),
                severity: pattern.severity(),
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

/// A synthetic, zero-location, `Warning`-severity `PatternViolation`
/// signalling something about *coverage* rather than a real match --
/// added to `ValidationResult.violations` so degraded coverage is visible
/// in ordinary human-readable/JSON output, not just in `checked_patterns`.
fn coverage_warning(pattern_id: &str, pattern_name: &str, message: String) -> PatternViolation {
    PatternViolation {
        pattern_id: pattern_id.to_string(),
        pattern_name: pattern_name.to_string(),
        severity: Severity::Warning,
        location: CodeLocation {
            file: None,
            line: 0,
            column: 0,
        },
        matched_text: String::new(),
        message,
    }
}

/// Runs every enabled pattern whose `language` matches `language` against
/// `source`, and aggregates the result. Shared by the MCP
/// `validate_pattern_compliance` tool (`mcp_server.rs`) and the `norma
/// validate` CLI subcommand (`cli.rs`) -- see docs/adr/0001.md: one core,
/// no separate sync/async implementation.
///
/// A pattern whose stored `rule` no longer parses is skipped rather than
/// failing the whole run, but the skip is never silent: it's logged and,
/// like the "zero patterns registered for this language" case, surfaced
/// as a `Warning`-severity entry in the result (see `coverage_warning`) --
/// a validator that can't tell "checked and clean" apart from "checked
/// nothing" is worse than one that errors.
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
    let mut checked_patterns = 0usize;
    let mut skipped: Vec<(String, anyhow::Error)> = Vec::new();

    for pattern in patterns
        .iter()
        .filter(|p| p.enabled && p.language() == language)
    {
        match parse_rule(pattern.rule()) {
            Ok(config) => {
                checked_patterns += 1;
                violations.extend(find_violations(pattern, &config, source));
            }
            Err(err) => {
                tracing::warn!(pattern_id = %pattern.id(), error = %err, "skipping pattern with unparsable rule");
                skipped.push((pattern.id().to_string(), err));
            }
        }
    }

    // Score is the real hit rate: computed before any synthetic coverage
    // warning is appended below, so it never divides by patterns that
    // never actually ran. Only warning/error-severity matches count
    // against the score -- an info/hint/off match (e.g. the purely
    // informational `observer-presence-*` pattern) is still visible in
    // `violations` below, but finding one isn't a defect, so it must not
    // make the score worse.
    let scoring_matches = violations
        .iter()
        .filter(|v| matches!(v.severity, Severity::Warning | Severity::Error))
        .count();
    let score = if checked_patterns == 0 {
        0.0
    } else {
        (1.0 - scoring_matches as f64 / checked_patterns as f64).max(0.0)
    };

    for (pattern_id, err) in &skipped {
        violations.push(coverage_warning(
            pattern_id,
            "Unparsable Pattern",
            format!("this pattern's stored rule could not be parsed and was skipped: {err}"),
        ));
    }
    if checked_patterns == 0 && skipped.is_empty() {
        violations.push(coverage_warning(
            NO_COVERAGE_PATTERN_ID,
            "No Coverage",
            format!(
                "no enabled patterns are registered for language {language:?} -- nothing was validated"
            ),
        ));
    }

    // `passed` mirrors `score`'s severity filter, checked after the
    // synthetic coverage warnings above (also Warning-severity) are folded
    // into `violations` -- degraded coverage must still fail a run exactly
    // as before.
    let passed = !violations
        .iter()
        .any(|v| matches!(v.severity, Severity::Warning | Severity::Error));
    Ok(ValidationResult {
        violations,
        passed,
        score,
        checked_patterns,
        duration_ms: start.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn test_pattern(rule: &str) -> Pattern {
        let now = Utc::now();
        Pattern::from_rule(
            "Test Pattern".to_string(),
            "test".to_string(),
            None,
            rule.to_string(),
            true,
            now,
            now,
        )
        .expect("test fixture rule must be valid")
    }

    const RUST_NO_DEBUG_PRINT: &str = r#"
id: no-debug-print-rust
message: Avoid println! in production code
severity: warning
language: Rust
rule:
  pattern: println!($$$ARGS)
"#;

    // A rule that parses as YAML but has no top-level `id`, so
    // `Pattern::from_rule` must reject it before anything derives from it.
    const RULE_WITHOUT_ID: &str = r#"
message: Avoid println! in production code
severity: warning
language: Rust
rule:
  pattern: println!($$$ARGS)
"#;

    const GO_RULE: &str = r#"
id: no-debug-print-go
message: Avoid fmt.Println in production code
severity: warning
language: Go
rule:
  pattern: fmt.Println($$$ARGS)
"#;

    #[test]
    fn from_rule_derives_id_language_and_severity() {
        let pattern = test_pattern(RUST_NO_DEBUG_PRINT);
        assert_eq!(pattern.id(), "no-debug-print-rust");
        assert_eq!(pattern.language(), "rust");
        assert_eq!(pattern.severity(), Severity::Warning);
    }

    #[test]
    fn from_rule_rejects_a_rule_with_no_id() {
        let now = Utc::now();
        let err = Pattern::from_rule(
            "No Id".to_string(),
            "d".to_string(),
            None,
            RULE_WITHOUT_ID.to_string(),
            true,
            now,
            now,
        )
        .expect_err("a rule with no top-level id must be rejected");
        assert!(
            err.to_string().contains("non-empty"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_rule_rejects_a_language_norma_does_not_support() {
        let now = Utc::now();
        let err = Pattern::from_rule(
            "No Debug Print".to_string(),
            "d".to_string(),
            None,
            GO_RULE.to_string(),
            true,
            now,
            now,
        )
        .expect_err("a Go rule must be rejected, not built");
        assert!(
            err.to_string().contains("unsupported language"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_finds_a_real_violation() {
        let pattern = test_pattern(RUST_NO_DEBUG_PRINT);
        let source = "fn main() { println!(\"debug\"); }";
        let result = validate(source, "rust", &[pattern]).unwrap();
        assert_eq!(result.violations.len(), 1);
        assert!(!result.passed);
        assert_eq!(result.violations[0].matched_text, "println!(\"debug\")");
        assert_eq!(result.checked_patterns, 1);
    }

    #[test]
    fn validate_reports_no_violations_for_clean_code() {
        let pattern = test_pattern(RUST_NO_DEBUG_PRINT);
        let source = "fn main() { tracing::info!(\"structured, fine\"); }";
        let result = validate(source, "rust", &[pattern]).unwrap();
        assert!(result.violations.is_empty());
        assert!(result.passed);
        assert_eq!(result.score, 1.0);
        assert_eq!(result.checked_patterns, 1);
    }

    #[test]
    fn validate_reports_degraded_coverage_instead_of_a_silent_pass_when_nothing_is_registered() {
        // Zero patterns for "python" at all -- would match if the language
        // filter didn't work, since this source is deliberately Rust syntax
        // passed under a "python" validation request.
        let pattern = test_pattern(RUST_NO_DEBUG_PRINT);
        let source = "println!(\"debug\")";
        let result = validate(source, "python", &[pattern]).unwrap();
        assert_eq!(result.checked_patterns, 0);
        // The whole point: this must NOT look identical to a clean pass.
        assert!(!result.passed);
        assert_eq!(result.score, 0.0);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].pattern_id, NO_COVERAGE_PATTERN_ID);
        assert_eq!(result.violations[0].severity, Severity::Warning);
        assert!(result.violations[0].message.contains("python"));
    }

    #[test]
    fn validate_reports_a_skipped_pattern_instead_of_silently_dropping_it() {
        // A pattern that parses fine at rule-construction time can still
        // become unparsable later (e.g. after an ast-grep upgrade changes
        // what's valid). Simulate that by handing `validate` a `Pattern`
        // whose `rule` field no longer parses -- `from_trusted_row` is the
        // one place that can build a `Pattern` without going through
        // `from_rule`'s validation, exactly like `PatternStore` reading a
        // row back from SQLite.
        let now = Utc::now();
        let broken = Pattern::from_trusted_row(
            "was-valid-once".to_string(),
            "Was Valid Once".to_string(),
            "d".to_string(),
            None,
            "rust".to_string(),
            Severity::Warning,
            "not: valid: yaml: at: all: -".to_string(),
            true,
            now,
            now,
        );
        let result = validate("fn main() {}", "rust", &[broken]).unwrap();
        assert_eq!(result.checked_patterns, 0);
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].pattern_id, "was-valid-once");
        assert!(result.violations[0].message.contains("could not be parsed"));
    }

    const RUST_NO_UNWRAP: &str = r#"
id: no-unwrap-rust
message: Avoid unwrap() in production code
severity: warning
language: Rust
rule:
  pattern: $EXPR.unwrap()
"#;

    #[test]
    fn validate_score_reflects_a_partial_hit_rate() {
        let debug_print = test_pattern(RUST_NO_DEBUG_PRINT);
        let no_unwrap = test_pattern(RUST_NO_UNWRAP);
        // Matches `debug_print` once; `no_unwrap` finds nothing.
        let source = "fn main() { println!(\"debug\"); }";
        let result = validate(source, "rust", &[debug_print, no_unwrap]).unwrap();
        assert_eq!(result.checked_patterns, 2);
        assert_eq!(
            result
                .violations
                .iter()
                .filter(|v| v.pattern_id == "no-debug-print-rust")
                .count(),
            1
        );
        assert_eq!(result.score, 0.5); // 1 real violation / 2 checked patterns
    }

    #[test]
    fn validate_score_never_goes_negative_when_one_pattern_matches_many_times() {
        let pattern = test_pattern(RUST_NO_DEBUG_PRINT);
        let source = "fn main() { println!(\"a\"); println!(\"b\"); println!(\"c\"); }";
        let result = validate(source, "rust", &[pattern]).unwrap();
        assert_eq!(result.checked_patterns, 1);
        assert_eq!(result.violations.len(), 3);
        // Naive `1.0 - 3.0/1.0` would be negative -- the `.max(0.0)` clamp must catch it.
        assert_eq!(result.score, 0.0);
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
        let pattern = test_pattern(RUST_NO_DEBUG_PRINT);
        let err = validate("package main", "go", &[pattern]).unwrap_err();
        assert!(
            err.to_string().contains("unsupported language"),
            "unexpected error: {err}"
        );
    }

    const RUST_OBSERVER_PRESENCE_INFO: &str = r#"
id: observer-presence-rust
message: Struct has an observers-style field -- an Observer-shaped construct
severity: info
language: Rust
rule:
  kind: struct_item
  has:
    stopBy: end
    kind: field_declaration
    regex: observers
"#;

    #[test]
    fn validate_info_severity_match_is_visible_but_does_not_fail_the_run() {
        let pattern = test_pattern(RUST_OBSERVER_PRESENCE_INFO);
        let source = "struct Publisher { observers: Vec<Box<dyn Observer>> }";
        let result = validate(source, "rust", &[pattern]).unwrap();
        assert_eq!(
            result.violations.len(),
            1,
            "the match must still be visible"
        );
        assert_eq!(result.violations[0].severity, Severity::Info);
        assert!(
            result.passed,
            "an info-severity match must not fail the run"
        );
        assert_eq!(
            result.score, 1.0,
            "an info-severity match must not lower the score"
        );
    }
}
