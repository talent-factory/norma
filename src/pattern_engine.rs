use crate::models::{
    CodeLocation, FixConflict, FixResult, Pattern, PatternViolation, Severity, ValidationResult,
};
use ast_grep_config::{from_yaml_string, GlobalRules, RuleConfig, Severity as AstGrepSeverity};
use ast_grep_core::replacer::Replacer;
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
///
/// `config.fixer` holds the rule's compiled `fix:` (see docs/adr/0002.md
/// and TF-890) -- at most a single-element `Vec` for the plain `fix: <str>`
/// form norma's patterns use (the multi-entry "named fix list" ast-grep
/// also supports is a code-action-picker UI feature with no norma
/// consumer, so only the first entry is ever used here). When present,
/// each match's `suggested_fix` is filled in from it via
/// `Fixer::generate_replacement`, regardless of whether `apply_fixes`
/// would actually apply it -- see `PatternViolation::suggested_fix`'s doc
/// comment on why "shown" and "applied" are independent.
fn find_violations(
    pattern: &Pattern,
    config: &RuleConfig<SupportLang>,
    source: &str,
) -> Vec<PatternViolation> {
    let fixer = config.fixer.first();
    let grep = config.language.ast_grep(source);
    grep.root()
        .find_all(&config.matcher)
        .map(|node_match| {
            let pos = node_match.start_pos();
            let suggested_fix = fixer.map(|fixer| {
                String::from_utf8_lossy(&fixer.generate_replacement(&node_match)).into_owned()
            });
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
                suggested_fix,
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
        suggested_fix: None,
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

/// One match's fix, ready to splice into `source` -- the byte-range
/// equivalent of `ast_grep_core::source::Edit`, plus the `pattern_id` and
/// position `apply_fixes` needs to report a conflict.
struct FixEdit {
    pattern_id: String,
    start: usize,
    end: usize,
    inserted_text: Vec<u8>,
    line: usize,
    column: usize,
}

/// Applies every enabled pattern's `fix`/`fixer` for `language` against
/// `source`, and returns the rewritten text. Shared by `cli::fix_files`
/// (`norma validate --fix`, writes the result back to the file) and the
/// MCP `apply_pattern_fix` tool (returns it, never touches the
/// filesystem) -- see TF-890's per-surface split, mirroring `validate`
/// above (docs/adr/0001.md: one core).
///
/// A pattern without a `fix:` is silently skipped here -- `validate`'s
/// `suggested_fix: None` already surfaces that, and it isn't a defect.
/// A pattern whose stored `rule` no longer parses is also skipped, but
/// *not* silently: it's logged and returned in `FixResult.skipped_rules`,
/// exactly like `validate`'s `coverage_warning` handling for the same
/// failure -- a caller (in particular the MCP `apply_pattern_fix` tool,
/// which has no second `validate` call surfacing this the way
/// `cli::fix_files` does) must be able to tell "nothing needed fixing"
/// from "some of this was never even attempted".
///
/// A single pattern that matches at more than one nesting level of the
/// same construct (e.g. `$EXPR.unwrap()` against `a.unwrap().unwrap()`,
/// matching both the outer call and, inside it, the inner one) is *not*
/// treated as two patterns disagreeing: only the outermost match per
/// nesting chain is kept, exactly as ast-grep's own rewrite tooling
/// resolves this. The dropped inner match still shows up as its own
/// `PatternViolation` via `find_violations`/`validate`, so a second
/// `--fix` pass converges on it once the outer rewrite has landed.
///
/// When two matches' fix ranges overlap (including one nested inside the
/// other) *and* they come from different patterns, *neither* is applied:
/// the whole cluster is dropped and reported as a single `FixConflict`
/// instead. Fail-safe over guessing a winner -- see decision point 3 in
/// `.scratch/ast-grep-feature-parity/issues/01-autofix-rewrite-support.md`
/// (TF-890). A cluster is detected by a standard sweep-line interval merge
/// (sort by start, track the running max end), so a chain of three or
/// more mutually-touching ranges is one cluster, not several overlapping
/// pairs.
pub fn apply_fixes(
    source: &str,
    language: &str,
    patterns: &[Pattern],
) -> anyhow::Result<FixResult> {
    let language = resolve_language(language)?;
    let mut edits: Vec<FixEdit> = Vec::new();
    let mut skipped_rules: Vec<(String, String)> = Vec::new();

    for pattern in patterns
        .iter()
        .filter(|p| p.enabled && p.language() == language)
    {
        let config = match parse_rule(pattern.rule()) {
            Ok(config) => config,
            Err(err) => {
                tracing::warn!(pattern_id = %pattern.id(), error = %err, "skipping pattern with unparsable rule");
                skipped_rules.push((pattern.id().to_string(), err.to_string()));
                continue;
            }
        };
        let Some(fixer) = config.fixer.first() else {
            continue;
        };
        let grep = config.language.ast_grep(source);
        // Matches come back in pre-order (a node before its descendants,
        // see `SgNode::dfs`), so for one nesting chain the outermost match
        // is always seen before any match nested inside it -- keeping a
        // running list of already-kept ranges and skipping anything fully
        // contained in one is enough to prune self-nested matches down to
        // just the outermost, per the doc comment above.
        let mut kept_ranges: Vec<std::ops::Range<usize>> = Vec::new();
        for node_match in grep.root().find_all(&config.matcher) {
            let range = node_match.range();
            if kept_ranges
                .iter()
                .any(|r| r.start <= range.start && range.end <= r.end)
            {
                continue;
            }
            kept_ranges.push(range);
            let pos = node_match.start_pos();
            let edit = node_match.make_edit(&config.matcher, fixer);
            edits.push(FixEdit {
                pattern_id: pattern.id().to_string(),
                start: edit.position,
                end: edit.position + edit.deleted_length,
                inserted_text: edit.inserted_text,
                line: pos.line(),
                column: pos.column(&node_match),
            });
        }
    }

    edits.sort_by_key(|e| e.start);

    let mut accepted: Vec<&FixEdit> = Vec::new();
    let mut conflicts: Vec<FixConflict> = Vec::new();
    let mut cluster: Vec<&FixEdit> = Vec::new();
    let mut cluster_end = 0usize;
    for edit in &edits {
        if !cluster.is_empty() && edit.start < cluster_end {
            cluster.push(edit);
            cluster_end = cluster_end.max(edit.end);
        } else {
            flush_cluster(&mut cluster, &mut accepted, &mut conflicts);
            cluster.push(edit);
            cluster_end = edit.end;
        }
    }
    flush_cluster(&mut cluster, &mut accepted, &mut conflicts);

    let bytes = source.as_bytes();
    let mut fixed = Vec::with_capacity(bytes.len());
    let mut cursor = 0usize;
    for edit in &accepted {
        fixed.extend_from_slice(&bytes[cursor..edit.start]);
        fixed.extend_from_slice(&edit.inserted_text);
        cursor = edit.end;
    }
    fixed.extend_from_slice(&bytes[cursor..]);

    // Splicing bytes at tree-sitter-reported node boundaries and inserting
    // `Fixer::generate_replacement`'s own output can, in principle, never
    // produce invalid UTF-8 from a valid `&str` source -- but that
    // guarantee lives in ast-grep, not in norma's type system, and this
    // text is about to be written straight to the caller's file
    // (`cli::fix_files`). A silent `_lossy` substitution here would mean
    // silently corrupting that file instead of failing loudly, which is
    // exactly the failure mode `validate`'s own doc comment above warns
    // against -- so this fails hard instead.
    let fixed_source = String::from_utf8(fixed)
        .map_err(|_| anyhow::anyhow!("apply_fixes produced non-UTF-8 output; refusing it"))?;

    Ok(FixResult {
        fixed_source,
        applied_count: accepted.len(),
        conflicts,
        skipped_rules,
    })
}

/// Closes out the in-progress cluster built by `apply_fixes`'s sweep: a
/// lone edit is accepted, a cluster of two or more conflicts and none of
/// them are. No-op if `cluster` is already empty (the state right after a
/// previous flush).
///
/// `pattern_ids` is deduplicated (while preserving first-seen order)
/// before being reported: after the same-pattern nesting prune in
/// `apply_fixes`, a cluster spanning the same pattern twice shouldn't
/// normally happen, but a pattern matching two distinct, merely
/// overlapping-not-nested spans is possible for an unusual rule, and a
/// `FixConflict` listing one pattern id twice would be a confusing report.
fn flush_cluster<'a>(
    cluster: &mut Vec<&'a FixEdit>,
    accepted: &mut Vec<&'a FixEdit>,
    conflicts: &mut Vec<FixConflict>,
) {
    match cluster.len() {
        0 => {}
        1 => accepted.push(cluster[0]),
        _ => {
            let first = &cluster[0];
            let mut pattern_ids: Vec<String> = Vec::new();
            for edit in cluster.iter() {
                if !pattern_ids.contains(&edit.pattern_id) {
                    pattern_ids.push(edit.pattern_id.clone());
                }
            }
            conflicts.push(FixConflict {
                pattern_ids,
                location: CodeLocation {
                    file: None,
                    line: first.line,
                    column: first.column,
                },
            });
        }
    }
    cluster.clear();
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

    // --- suggested_fix / apply_fixes (TF-890) ------------------------------

    const RUST_UNWRAP_WITH_FIX: &str = r#"
id: no-unwrap-rust
message: Avoid unwrap() in production code
severity: warning
language: Rust
rule:
  pattern: $EXPR.unwrap()
fix: $EXPR.expect("TODO")
"#;

    #[test]
    fn validate_populates_suggested_fix_when_the_rule_has_a_fix() {
        let pattern = test_pattern(RUST_UNWRAP_WITH_FIX);
        let source = "fn main() { value.unwrap(); }";
        let result = validate(source, "rust", &[pattern]).unwrap();
        assert_eq!(result.violations.len(), 1);
        assert_eq!(
            result.violations[0].suggested_fix.as_deref(),
            Some(r#"value.expect("TODO")"#)
        );
    }

    #[test]
    fn validate_leaves_suggested_fix_none_when_the_rule_has_no_fix() {
        let pattern = test_pattern(RUST_NO_DEBUG_PRINT);
        let source = "fn main() { println!(\"debug\"); }";
        let result = validate(source, "rust", &[pattern]).unwrap();
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].suggested_fix, None);
    }

    #[test]
    fn apply_fixes_rewrites_a_single_match_in_place() {
        let pattern = test_pattern(RUST_UNWRAP_WITH_FIX);
        let source = "fn main() { value.unwrap(); }";
        let fix = apply_fixes(source, "rust", &[pattern]).unwrap();
        assert_eq!(fix.fixed_source, r#"fn main() { value.expect("TODO"); }"#);
        assert_eq!(fix.applied_count, 1);
        assert!(fix.conflicts.is_empty());
    }

    #[test]
    fn apply_fixes_rewrites_several_non_overlapping_matches() {
        let pattern = test_pattern(RUST_UNWRAP_WITH_FIX);
        let source = "fn main() { a.unwrap(); b.unwrap(); }";
        let fix = apply_fixes(source, "rust", &[pattern]).unwrap();
        assert_eq!(
            fix.fixed_source,
            r#"fn main() { a.expect("TODO"); b.expect("TODO"); }"#
        );
        assert_eq!(fix.applied_count, 2);
        assert!(fix.conflicts.is_empty());
    }

    #[test]
    fn apply_fixes_skips_patterns_and_matches_without_a_fix() {
        // Registered alongside a fixable pattern: `apply_fixes` must only
        // touch the source where a `fix:` actually exists.
        let unwrap_pattern = test_pattern(RUST_UNWRAP_WITH_FIX);
        let debug_print_pattern = test_pattern(RUST_NO_DEBUG_PRINT);
        let source = "fn main() { println!(\"debug\"); value.unwrap(); }";
        let fix = apply_fixes(source, "rust", &[unwrap_pattern, debug_print_pattern]).unwrap();
        assert_eq!(
            fix.fixed_source,
            "fn main() { println!(\"debug\"); value.expect(\"TODO\"); }"
        );
        assert_eq!(fix.applied_count, 1);
    }

    #[test]
    fn apply_fixes_leaves_unfixable_source_unchanged() {
        let pattern = test_pattern(RUST_NO_DEBUG_PRINT);
        let source = "fn main() { println!(\"debug\"); }";
        let fix = apply_fixes(source, "rust", &[pattern]).unwrap();
        assert_eq!(fix.fixed_source, source);
        assert_eq!(fix.applied_count, 0);
        assert!(fix.conflicts.is_empty());
    }

    #[test]
    fn apply_fixes_applies_neither_side_of_an_overlapping_conflict() {
        // Two distinct patterns whose rule matches the exact same span
        // (`$EXPR.unwrap()`) but disagree on the replacement -- the
        // fail-safe case from TF-890's decision doc: neither wins, both
        // are reported as one conflict, and the source is untouched.
        let now = Utc::now();
        let pattern_a = Pattern::from_rule(
            "Unwrap Fix A".to_string(),
            "d".to_string(),
            None,
            r#"
id: unwrap-fix-a
message: a
severity: warning
language: Rust
rule:
  pattern: $EXPR.unwrap()
fix: $EXPR.expect("a")
"#
            .to_string(),
            true,
            now,
            now,
        )
        .unwrap();
        let pattern_b = Pattern::from_rule(
            "Unwrap Fix B".to_string(),
            "d".to_string(),
            None,
            r#"
id: unwrap-fix-b
message: b
severity: warning
language: Rust
rule:
  pattern: $EXPR.unwrap()
fix: $EXPR.expect("b")
"#
            .to_string(),
            true,
            now,
            now,
        )
        .unwrap();
        let source = "fn main() { value.unwrap(); }";

        let fix = apply_fixes(source, "rust", &[pattern_a, pattern_b]).unwrap();

        assert_eq!(fix.fixed_source, source, "conflicting fixes must not apply");
        assert_eq!(fix.applied_count, 0);
        assert_eq!(fix.conflicts.len(), 1);
        let mut ids = fix.conflicts[0].pattern_ids.clone();
        ids.sort();
        assert_eq!(
            ids,
            vec!["unwrap-fix-a".to_string(), "unwrap-fix-b".to_string()]
        );
    }

    #[test]
    fn apply_fixes_applies_a_chain_of_three_mutually_overlapping_patterns_as_one_cluster() {
        // Three distinct patterns whose fix ranges all touch the same
        // code -- the full call `outer(value, extra)`, and two more
        // patterns each separately matching just the `value` identifier
        // nested inside it -- must collapse into a *single* conflict via
        // the sweep, not two or three separate pairwise conflicts. See
        // `apply_fixes`'s doc comment.
        let now = Utc::now();
        let pattern_a = Pattern::from_rule(
            "Call Fix".to_string(),
            "d".to_string(),
            None,
            r#"
id: call-fix
message: a
severity: warning
language: Rust
rule:
  pattern: outer($ARG, extra)
fix: renamed($ARG, extra)
"#
            .to_string(),
            true,
            now,
            now,
        )
        .unwrap();
        let pattern_b = Pattern::from_rule(
            "Arg Fix".to_string(),
            "d".to_string(),
            None,
            r#"
id: arg-fix
message: b
severity: warning
language: Rust
rule:
  pattern: value
fix: renamed_value
"#
            .to_string(),
            true,
            now,
            now,
        )
        .unwrap();
        let pattern_c = Pattern::from_rule(
            "Ident Fix".to_string(),
            "d".to_string(),
            None,
            r#"
id: ident-fix
message: c
severity: warning
language: Rust
rule:
  kind: identifier
  regex: "^value$"
fix: renamed_value_c
"#
            .to_string(),
            true,
            now,
            now,
        )
        .unwrap();
        let source = "fn main() { outer(value, extra); }";

        let fix = apply_fixes(source, "rust", &[pattern_a, pattern_b, pattern_c]).unwrap();

        assert_eq!(
            fix.fixed_source, source,
            "a chained cluster must apply nothing"
        );
        assert_eq!(fix.applied_count, 0);
        assert_eq!(
            fix.conflicts.len(),
            1,
            "three mutually-touching ranges must be one cluster, not several pairwise conflicts"
        );
        let mut ids = fix.conflicts[0].pattern_ids.clone();
        ids.sort();
        assert_eq!(
            ids,
            vec![
                "arg-fix".to_string(),
                "call-fix".to_string(),
                "ident-fix".to_string(),
            ]
        );
    }

    #[test]
    fn apply_fixes_applies_only_the_outermost_match_of_a_self_nested_pattern() {
        // A pattern that matches at more than one nesting level of the
        // same construct (`$EXPR.unwrap()` against `a.unwrap().unwrap()`)
        // is not two patterns disagreeing -- it's one pattern seeing
        // itself twice. Before this was pruned, this used to be
        // misreported as a self-conflict (`pattern_ids: ["no-unwrap-rust",
        // "no-unwrap-rust"]`) that applied nothing at all.
        let pattern = test_pattern(RUST_UNWRAP_WITH_FIX);
        let source = "fn main() { a.unwrap().unwrap(); }";

        let fix = apply_fixes(source, "rust", &[pattern]).unwrap();

        assert!(
            fix.conflicts.is_empty(),
            "a pattern matching its own nested output is not a conflict: {:?}",
            fix.conflicts
        );
        assert_eq!(fix.applied_count, 1, "only the outermost match is applied");
        assert_eq!(
            fix.fixed_source, r#"fn main() { a.unwrap().expect("TODO"); }"#,
            "the still-nested inner unwrap() is left for a follow-up --fix pass"
        );
    }

    #[test]
    fn apply_fixes_reports_a_pattern_whose_stored_rule_no_longer_parses() {
        // Mirrors `validate_reports_a_skipped_pattern_instead_of_silently_dropping_it`:
        // a `Pattern` built via `from_trusted_row` can carry a `rule` that
        // no longer parses (simulating bit rot after an ast-grep upgrade,
        // or a corrupted row). `apply_fixes` must report this in
        // `skipped_rules` rather than just vanishing it from the output --
        // the MCP `apply_pattern_fix` tool has no second `validate` call
        // to catch this the way `cli::fix_files` does.
        let now = Utc::now();
        let good = test_pattern(RUST_UNWRAP_WITH_FIX);
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

        let fix = apply_fixes("fn main() { value.unwrap(); }", "rust", &[good, broken]).unwrap();

        assert_eq!(
            fix.applied_count, 1,
            "the still-valid pattern must still be applied"
        );
        assert_eq!(fix.skipped_rules.len(), 1);
        assert_eq!(fix.skipped_rules[0].0, "was-valid-once");
    }

    // Verifies TF-890 decision point 4: a `rewriters:` list embedded
    // locally in the same pattern YAML (as opposed to one shared *across*
    // pattern rows, which ADR 0002's "one pattern = one row" model has no
    // place for and stays out of scope) needs no norma-side support code
    // of its own -- `parse_rule` already parses the full YAML including
    // `rewriters:`, and once a rule's `fix:` references a `transform:`
    // that invokes one (`{ rewrite: { rewriters: [id], source: $VAR } }`),
    // `Fixer::generate_replacement` resolves it as part of the same
    // template expansion `find_violations`/`apply_fixes` already trigger.
    // A plain function call, not a macro invocation: a `$SINGLE` capture
    // (as opposed to `$$$ARGS`, used everywhere else in this file) doesn't
    // line up with a macro's raw token-tree body in tree-sitter-rust, so
    // `println!($INNER)` matches nothing here -- unrelated to `rewriters:`
    // itself, which is what this test is actually pinning down.
    const RUST_FIX_WITH_LOCAL_REWRITER: &str = r#"
id: wrap-debug-rust
message: Wrap wrap()'s inner call, renaming it via a local rewriter
severity: warning
language: Rust
rule:
  pattern: wrap($INNER)
transform:
  REWRITTEN:
    rewrite:
      rewriters: [rename-foo-to-bar]
      source: $INNER
fix: wrap($REWRITTEN)
rewriters:
  - id: rename-foo-to-bar
    rule:
      pattern: foo($$$ARGS)
    fix: bar($$$ARGS)
"#;

    #[test]
    fn apply_fixes_resolves_a_fix_that_uses_a_locally_embedded_rewriter() {
        let pattern = test_pattern(RUST_FIX_WITH_LOCAL_REWRITER);
        let source = "fn main() { wrap(foo(1, 2)); }";
        let fix = apply_fixes(source, "rust", &[pattern]).unwrap();
        assert_eq!(fix.fixed_source, "fn main() { wrap(bar(1, 2)); }");
        assert_eq!(fix.applied_count, 1);
    }

    #[test]
    fn apply_fixes_errors_instead_of_falsely_no_opping_an_unsupported_language() {
        let pattern = test_pattern(RUST_UNWRAP_WITH_FIX);
        let err = apply_fixes("package main", "go", &[pattern]).unwrap_err();
        assert!(
            err.to_string().contains("unsupported language"),
            "unexpected error: {err}"
        );
    }
}
