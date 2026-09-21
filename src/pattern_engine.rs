use crate::models::{
    CodeLocation, FixConflict, FixResult, Pattern, PatternViolation, Severity, ValidationResult,
};
use ast_grep_config::{from_yaml_string, GlobalRules, RuleConfig, Severity as AstGrepSeverity};
use ast_grep_core::replacer::Replacer;
use ast_grep_language::{LanguageExt, SupportLang};
use chrono::{DateTime, Utc};
use std::str::FromStr;
use std::sync::LazyLock;
use std::time::Instant;

/// Pattern id used for the synthetic warning `validate` adds when zero
/// enabled patterns are registered for the requested language. Never a
/// real registered pattern's id -- see `Pattern::from_rule`'s doc comment
/// on why a real `Pattern` can't be constructed with an arbitrary id.
pub const NO_COVERAGE_PATTERN_ID: &str = "__norma_no_coverage__";

/// `(SupportLang variant, norma's canonical key)` for every `SupportLang`
/// `ast-grep-language` ships, generically derived from
/// `SupportLang::all_langs()` rather than hardcoded -- see `language_key`'s
/// doc comment for why. Built once, on first use; `String` rather than a
/// leaked `&'static str`, since a `static`'s own storage already lives for
/// the program's lifetime -- `.as_str()` on an entry borrows that for free.
///
/// The canonical key is the variant's `Display` output (its bare name,
/// e.g. `"Rust"`, `"CSharp"`) lowercased. That's not itself one of
/// ast-grep-language's declared aliases, and those alias lists are
/// inconsistent about which alias is "canonical" (Rust's first alias is
/// `"rs"`, TypeScript's is `"ts"`) -- but the lowercased variant name does
/// land on *some* alias of that same language for every one of the 28
/// current variants, which is what the round-trip assertion below verifies
/// rather than assumes. This is a build-time-deterministic property (no
/// user input involved, and the `ast-grep-language` version is exactly
/// pinned in Cargo.toml) -- a future version bump that broke it for some
/// variant must fail loudly and immediately (here, at first access, before
/// norma serves a single request) rather than silently registering that
/// language under a key nothing could ever look up again, which would
/// otherwise surface only as a confusing "no coverage" report much later.
static LANGUAGE_KEYS: LazyLock<Vec<(SupportLang, String)>> = LazyLock::new(|| {
    SupportLang::all_langs()
        .iter()
        .map(|&lang| {
            let key = lang.to_string().to_lowercase();
            assert_eq!(
                SupportLang::from_str(&key).ok(),
                Some(lang),
                "ast-grep-language variant {lang:?} lowercases to {key:?}, \
                 which doesn't parse back to it -- LANGUAGE_KEYS's \
                 derivation assumption broke, most likely from an \
                 ast-grep-language version bump changing an alias list"
            );
            (lang, key)
        })
        .collect()
});

/// Every language key norma supports, in `SupportLang::all_langs()`'s
/// order -- the order shown in error messages. Derived from
/// `LANGUAGE_KEYS`, so it can't drift out of sync with `language_key`.
pub static SUPPORTED_LANGUAGES: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| LANGUAGE_KEYS.iter().map(|(_, key)| key.as_str()).collect());

/// Parses a full, possibly multi-document (`---`-separated) ast-grep
/// RuleConfig YAML string and returns every document it contains, in
/// document order -- the same convention a cloned `sgconfig.yaml` rule
/// directory or ast-grep's own rule catalog uses. `parse_rule` (below) is
/// this with only the first document kept; this is the basis for
/// `PatternStore::import_rules`'s bulk import (TF-894), which needs every
/// document, not just the first.
pub fn parse_rules(rule_yaml: &str) -> anyhow::Result<Vec<RuleConfig<SupportLang>>> {
    let globals = GlobalRules::default();
    let configs = from_yaml_string::<SupportLang>(rule_yaml, &globals)?;
    if configs.is_empty() {
        anyhow::bail!("rule YAML did not contain a RuleConfig document");
    }
    Ok(configs)
}

/// Parses a full ast-grep RuleConfig YAML document (see docs/adr/0002.md)
/// and returns the compiled rule, or an error if the YAML is malformed or
/// the rule has no matchable AST kinds. Used both to run a pattern
/// (`validate`, below) and to validate one before it's persisted
/// (`PatternStore::register_pattern` in `pattern_store.rs`). Discards every
/// document past the first -- see `parse_rules` for the multi-document
/// variant.
pub fn parse_rule(rule_yaml: &str) -> anyhow::Result<RuleConfig<SupportLang>> {
    Ok(parse_rules(rule_yaml)?.remove(0))
}

/// norma's own canonical language key for a `SupportLang`, matching the
/// strings used in the `patterns.language` SQLite column and the MCP tool
/// parameters -- e.g. `"java"`, `"python"`, `"rust"`, `"typescript"`, and,
/// since TF-893, every other `SupportLang` variant too (`"go"`, `"css"`,
/// `"markdown"`, ...; see `LANGUAGE_KEYS`). *Registrable* is not the same
/// as *shipped with default patterns*: `default_patterns.rs` still only
/// covers Java/Python/Rust/TypeScript -- the other 24 languages are
/// registrable but pattern-less until someone writes patterns for them.
///
/// In practice this can never return `None` for a real `SupportLang`
/// value: `LANGUAGE_KEYS` covers every variant `SupportLang::all_langs()`
/// lists (i.e. all of them -- `SupportLang` is a closed enum), and its
/// construction panics rather than skipping an entry if the round-trip
/// assumption ever breaks (see `LANGUAGE_KEYS`'s doc comment). The
/// `Option` return type is kept anyway as the honest contract for
/// callers: MUST treat `None` as an error rather than substituting a
/// placeholder -- a pattern stored under a placeholder key would be
/// permanently unreachable, and a validation run against one would
/// silently check nothing and report `passed: true`.
pub fn language_key(lang: SupportLang) -> Option<&'static str> {
    LANGUAGE_KEYS
        .iter()
        .find(|(candidate, _)| *candidate == lang)
        .map(|(_, key)| key.as_str())
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

/// The two structural requirements a parsed rule must meet before it can
/// become a `Pattern` (see `Pattern::from_rule`, below) or be safely run by
/// `test_rule`: a non-empty top-level `id` and a language norma supports.
/// Returns the resolved canonical language key on success.
///
/// Shared by both callers so `test_rule`'s dry run rejects exactly what
/// `register_pattern` would reject later -- an id/language problem
/// discovered only after registering a rule would defeat the point of
/// testing it first (TF-891).
fn check_registerable(config: &RuleConfig<SupportLang>) -> anyhow::Result<&'static str> {
    if config.id.is_empty() {
        anyhow::bail!("rule YAML must set a non-empty top-level `id`");
    }
    language_key(config.language).ok_or_else(|| {
        anyhow::anyhow!(
            "unsupported language: {:?} (norma supports: {})",
            config.language,
            SUPPORTED_LANGUAGES.join(", ")
        )
    })
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
        let language = check_registerable(&config)?;
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
///
/// Takes `pattern_id`/`pattern_name`/`severity`/`message` as plain values
/// rather than a `&Pattern` so this also serves `test_rule`, which has no
/// registered `Pattern` to read them from (TF-891) -- `validate`, below,
/// passes a real pattern's catalog fields (`pattern.id()`, `pattern.name`,
/// `pattern.description`); `test_rule` has no catalog name or description
/// to pass, so it passes the rule's own `id` for *both* `pattern_id` and
/// `pattern_name`, and the rule's own `message` in place of a description.
fn find_violations(
    pattern_id: &str,
    pattern_name: &str,
    severity: Severity,
    message: &str,
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
                pattern_id: pattern_id.to_string(),
                pattern_name: pattern_name.to_string(),
                severity,
                location: CodeLocation {
                    file: None,
                    line: pos.line(),
                    column: pos.column(&node_match),
                },
                matched_text: node_match.text().to_string(),
                message: message.to_string(),
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

/// The real hit rate: 1.0 minus the fraction of `checked_patterns` that
/// produced at least a warning/error-severity match, clamped to never go
/// negative (a single pattern matching more than once would otherwise push
/// the naive ratio below zero). Only warning/error-severity matches count
/// against it -- an info/hint/off match (e.g. the purely informational
/// `observer-presence-*` pattern) is still visible in `violations`, but
/// finding one isn't a defect, so it must not make the score worse.
///
/// Shared by `validate` and `test_rule` so the formula can't drift between
/// them (TF-891) -- `validate` calls this on the matches alone, before any
/// synthetic coverage warning is appended (see its call site's comment for
/// why), while `test_rule` has no coverage warnings to worry about.
fn score_from(violations: &[PatternViolation], checked_patterns: usize) -> f64 {
    if checked_patterns == 0 {
        return 0.0;
    }
    let scoring_matches = violations
        .iter()
        .filter(|v| matches!(v.severity, Severity::Warning | Severity::Error))
        .count();
    (1.0 - scoring_matches as f64 / checked_patterns as f64).max(0.0)
}

/// Whether `violations` contains at least one warning/error-severity entry
/// -- mirrors `score_from`'s severity filter (an info/hint/off-only result
/// still passes). Shared with `test_rule` for the same reason `score_from`
/// is.
fn passed_from(violations: &[PatternViolation]) -> bool {
    !violations
        .iter()
        .any(|v| matches!(v.severity, Severity::Warning | Severity::Error))
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
                violations.extend(find_violations(
                    pattern.id(),
                    &pattern.name,
                    pattern.severity(),
                    &pattern.description,
                    &config,
                    source,
                ));
            }
            Err(err) => {
                tracing::warn!(pattern_id = %pattern.id(), error = %err, "skipping pattern with unparsable rule");
                skipped.push((pattern.id().to_string(), err));
            }
        }
    }

    // `score_from` runs on the matches alone, before any synthetic coverage
    // warning is appended below, so it never divides by patterns that never
    // actually ran.
    let score = score_from(&violations, checked_patterns);

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

    // `passed_from` mirrors `score_from`'s severity filter, checked after
    // the synthetic coverage warnings above (also Warning-severity) are
    // folded into `violations` -- degraded coverage must still fail a run
    // exactly as before.
    let passed = passed_from(&violations);
    Ok(ValidationResult {
        violations,
        passed,
        score,
        checked_patterns,
        duration_ms: start.elapsed().as_millis(),
    })
}

/// Compiles a candidate rule and runs it against `source`, without
/// registering anything -- the MCP `test_pattern` tool's core (TF-891, see
/// .scratch/ast-grep-feature-parity/issues/02-rule-testing-ast-debug-tooling.md).
/// Answers "does this rule match what I intend?" *before* `register_pattern`,
/// closing the gap the README's "Adopting an existing ast-grep rule" section
/// used to describe as a workaround (register first, then validate).
///
/// Applies the exact same structural checks `Pattern::from_rule` does
/// (non-empty `id`, a language norma supports) via `check_registerable`,
/// so a rule this accepts is guaranteed to also be accepted by
/// `register_pattern`, and one it rejects would be rejected there too --
/// before ever reaching storage in either case (`register_pattern` runs
/// those same checks up front and never writes to SQLite on failure, see
/// `PatternStore::register_pattern`'s doc comment). What this *doesn't*
/// cover: if `rule_yaml`'s `id` collides with an already-registered
/// pattern, `register_pattern` will silently overwrite it -- `test_rule`
/// never looks at the store, so it can't warn about that.
///
/// Reuses `find_violations`, so a `fix:` in `rule_yaml` populates
/// `suggested_fix` exactly like `validate_pattern_compliance` does -- same
/// result shape, just for one ad-hoc rule instead of a whole language's
/// registered catalog. There is no catalog `Pattern` yet to read a name or
/// description from, so both `pattern_id` and `pattern_name` on each
/// violation come from the rule's own `id`, and `message` comes from the
/// rule's own `message` field rather than a catalog `description`.
pub fn test_rule(rule_yaml: &str, source: &str) -> anyhow::Result<ValidationResult> {
    let start = Instant::now();
    let config = parse_rule(rule_yaml)?;
    check_registerable(&config)?;
    let severity = severity_from_ast_grep(&config.severity);
    let violations = find_violations(
        &config.id,
        &config.id,
        severity,
        &config.message,
        &config,
        source,
    );

    // Exactly one rule is ever checked here -- unlike `validate`, above,
    // `checked_patterns` can never legitimately be 0: `check_registerable`
    // already returned early if the rule itself couldn't be run. No
    // coverage warnings apply either (those are `validate`'s "zero
    // patterns registered"/"pattern failed to parse" concerns, neither of
    // which exists for one already-parsed, already-checked rule), so
    // `score_from`/`passed_from` run on `violations` as-is, unlike
    // `validate`'s two-phase before/after-coverage-warnings split.
    let checked_patterns = 1;
    let score = score_from(&violations, checked_patterns);
    let passed = passed_from(&violations);

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

    // Go is one of the 24 languages TF-893 unlocked -- previously rejected
    // by `check_registerable`, now registrable (just without shipped
    // default patterns; see `default_patterns.rs`).
    const GO_RULE: &str = r#"
id: no-debug-print-go
message: Avoid fmt.Println in production code
severity: warning
language: Go
rule:
  pattern: fmt.Println($$$ARGS)
"#;

    // A plain `pattern: <string>` rule (like GO_RULE above) is accepted
    // as structurally valid by `parse_rule`, but that doesn't mean it
    // actually matches real Go source: unlike Rust/Java/Python/
    // TypeScript, a bare Go fragment like `fmt.Println($$$ARGS)` doesn't
    // parse into the same `call_expression` node a full Go file's real
    // call expressions do, so GO_RULE never matches anything in practice
    // (see validate_finds_a_real_violation_in_a_previously_unsupported_language,
    // which needs this rule instead). ast-grep's fix for exactly this is
    // the `context`/`selector` object form of `pattern:` -- give it a
    // parseable enclosing snippet (`context`) and name which node inside
    // that snippet is the actual matcher (`selector`). This is the
    // "pattern-authoring per language is its own follow-up effort" the
    // TF-893 ticket deliberately scoped out -- captured here only because
    // this test needs *a* working rule, not as a general solution.
    const GO_RULE_WITH_WORKING_PATTERN: &str = r#"
id: no-debug-print-go-working
message: Avoid fmt.Println in production code
severity: warning
language: Go
rule:
  pattern:
    context: "func _() { fmt.Println($$$ARGS) }"
    selector: call_expression
"#;

    // Cobol isn't one of the 28 `SupportLang` variants ast-grep-language
    // 0.45.3 ships -- no version of norma has ever supported it, TF-893
    // included. Used by the tests that need a language genuinely outside
    // ast-grep's own reach, as opposed to one merely outside norma's old
    // 4-language MVP set.
    const COBOL_RULE: &str = r#"
id: no-debug-print-cobol
message: Avoid DISPLAY in production code
severity: warning
language: Cobol
rule:
  pattern: DISPLAY $$$ARGS
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
    fn from_rule_accepts_a_previously_unsupported_language() {
        // TF-893: a Go rule is no longer rejected at the `language_key`
        // gate -- it's now built like any other, just under the "go" key
        // rather than one of the original four.
        let pattern = test_pattern(GO_RULE);
        assert_eq!(pattern.language(), "go");
    }

    #[test]
    fn from_rule_rejects_a_language_ast_grep_does_not_support() {
        let now = Utc::now();
        let err = Pattern::from_rule(
            "No Debug Print".to_string(),
            "d".to_string(),
            None,
            COBOL_RULE.to_string(),
            true,
            now,
            now,
        )
        .expect_err("a language ast-grep-language itself doesn't know must still be rejected");
        // Unlike an unsupported-but-real `SupportLang` (impossible since
        // TF-893 -- see `from_rule_accepts_a_previously_unsupported_language`),
        // this fails inside `parse_rule`'s YAML deserialization, before
        // `check_registerable` (and its "unsupported language" message)
        // ever runs. `err.to_string()` (Display, top-level only) is just
        // "Fail to parse yaml as RuleConfig" -- the useful detail is one
        // level down in the anyhow chain, so assert on `{err:?}` instead.
        let chain = format!("{err:?}");
        assert!(
            chain.contains("Cobol"),
            "expected the error chain to name the rejected language, got: {chain}"
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
    fn validate_finds_a_real_violation_in_a_previously_unsupported_language() {
        // TF-893: `from_rule_accepts_a_previously_unsupported_language` and
        // `test_rule_accepts_a_previously_unsupported_language` only prove
        // a Go rule is no longer rejected at the `language_key` gate --
        // neither runs it against source that actually contains a match,
        // so neither could tell "Go matching works" apart from "Go
        // matching silently finds nothing". This does, against real Go
        // source, the same way `validate_finds_a_real_violation` does for
        // Rust -- proving the tree-sitter-go grammar (already compiled in,
        // per `Cargo.toml`'s default `ast-grep-language` features) is
        // genuinely reachable end to end, not just that the gate got out
        // of the way. Deliberately *not* GO_RULE -- see
        // GO_RULE_WITH_WORKING_PATTERN's doc comment for why the plain
        // `pattern: fmt.Println($$$ARGS)` string every other Go-rule test
        // in this file uses never actually matches real Go source.
        let pattern = test_pattern(GO_RULE_WITH_WORKING_PATTERN);
        let source =
            "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"debug\")\n}\n";
        let result = validate(source, "go", &[pattern]).unwrap();
        assert_eq!(result.violations.len(), 1);
        assert!(!result.passed);
        assert_eq!(result.violations[0].matched_text, "fmt.Println(\"debug\")");
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
    fn parse_rules_returns_every_document_in_a_multi_document_yaml() {
        let two_rules = format!("{RUST_NO_DEBUG_PRINT}---\n{RUST_NO_UNWRAP}");
        let configs = parse_rules(&two_rules).unwrap();
        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].id, "no-debug-print-rust");
        assert_eq!(configs[1].id, "no-unwrap-rust");
    }

    #[test]
    fn parse_rule_keeps_only_the_first_document_of_a_multi_document_yaml() {
        // `parse_rule` (single-document) must still behave exactly as
        // before `parse_rules` was introduced -- discarding every document
        // past the first, not erroring on one.
        let two_rules = format!("{RUST_NO_DEBUG_PRINT}---\n{RUST_NO_UNWRAP}");
        let config = parse_rule(&two_rules).unwrap();
        assert_eq!(config.id, "no-debug-print-rust");
    }

    #[test]
    fn language_key_covers_every_ast_grep_language() {
        // TF-893: `language_key` is no longer a curated 4-language
        // allowlist -- Go and Css (both previously `None`) now resolve
        // like any other `SupportLang` variant.
        assert_eq!(language_key(SupportLang::Rust), Some("rust"));
        assert_eq!(language_key(SupportLang::Go), Some("go"));
        assert_eq!(language_key(SupportLang::Css), Some("css"));
        // The derivation (see `LANGUAGE_KEYS`) must be total across every
        // variant `ast-grep-language` currently ships, not just these
        // three -- a `None` here would make that language's patterns
        // permanently unreachable.
        for &lang in SupportLang::all_langs() {
            assert!(
                language_key(lang).is_some(),
                "{lang:?} has no canonical key"
            );
        }
    }

    #[test]
    fn supported_languages_covers_every_ast_grep_language() {
        assert_eq!(SUPPORTED_LANGUAGES.len(), SupportLang::all_langs().len());
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
        // A real language, but not one of the 28 `SupportLang` variants
        // ast-grep-language ships (since TF-893, "go" no longer belongs
        // here -- see resolve_language_accepts_every_ast_grep_language)...
        assert!(resolve_language("cobol").is_err());
        // ...and something that isn't a language at all.
        assert!(resolve_language("rustt").is_err());
    }

    #[test]
    fn resolve_language_accepts_every_ast_grep_language() {
        // TF-893: Go -- previously rejected -- now resolves like any
        // other `SupportLang`, even though norma ships no default
        // patterns for it (see `default_patterns.rs`).
        assert_eq!(resolve_language("go").unwrap(), "go");
        assert_eq!(resolve_language("golang").unwrap(), "go");
        // Case-insensitivity (already proven for an original MVP language
        // by resolve_language_accepts_canonical_keys_and_ast_grep_aliases)
        // holds for a newly-unlocked one too -- it's implemented once,
        // generically, in ast-grep-language's shared `FromStr`.
        assert_eq!(resolve_language("Go").unwrap(), "go");
        assert_eq!(resolve_language("GOLANG").unwrap(), "go");
    }

    #[test]
    fn validate_errors_instead_of_falsely_passing_an_unsupported_language() {
        let pattern = test_pattern(RUST_NO_DEBUG_PRINT);
        let err = validate("package main", "cobol", &[pattern]).unwrap_err();
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

    // --- test_rule (TF-891) ------------------------------------------------

    #[test]
    fn test_rule_finds_a_real_violation() {
        let result = test_rule(RUST_NO_DEBUG_PRINT, "fn main() { println!(\"debug\"); }").unwrap();
        assert_eq!(result.violations.len(), 1);
        assert!(!result.passed);
        assert_eq!(result.violations[0].pattern_id, "no-debug-print-rust");
        assert_eq!(result.violations[0].matched_text, "println!(\"debug\")");
        assert_eq!(result.checked_patterns, 1);
    }

    #[test]
    fn test_rule_reports_no_violations_for_clean_code() {
        let result = test_rule(RUST_NO_DEBUG_PRINT, "fn main() {}").unwrap();
        assert!(result.violations.is_empty());
        assert!(result.passed);
        assert_eq!(result.score, 1.0);
        assert_eq!(result.checked_patterns, 1);
    }

    /// Mirrors `validate_info_severity_match_is_visible_but_does_not_fail_the_run`
    /// -- `test_rule` shares `score_from`/`passed_from` with `validate`, but
    /// that sharing is only worth anything if both sides are actually
    /// exercised.
    #[test]
    fn test_rule_info_severity_match_is_visible_but_does_not_fail_the_run() {
        let source = "struct Publisher { observers: Vec<Box<dyn Observer>> }";
        let result = test_rule(RUST_OBSERVER_PRESENCE_INFO, source).unwrap();
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

    /// Mirrors `validate_score_never_goes_negative_when_one_pattern_matches_many_times`.
    /// `test_rule` reaches the same clamp far more easily than `validate`
    /// does, since `checked_patterns` is always 1 -- two matches already
    /// drive the naive ratio negative, not several checked patterns' worth.
    #[test]
    fn test_rule_score_never_goes_negative_when_the_rule_matches_many_times() {
        let source = "fn main() { println!(\"a\"); println!(\"b\"); println!(\"c\"); }";
        let result = test_rule(RUST_NO_DEBUG_PRINT, source).unwrap();
        assert_eq!(result.checked_patterns, 1);
        assert_eq!(result.violations.len(), 3);
        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn test_rule_populates_suggested_fix_when_the_rule_has_a_fix() {
        let result = test_rule(RUST_UNWRAP_WITH_FIX, "fn main() { value.unwrap(); }").unwrap();
        assert_eq!(result.violations.len(), 1);
        assert_eq!(
            result.violations[0].suggested_fix.as_deref(),
            Some(r#"value.expect("TODO")"#)
        );
    }

    #[test]
    fn test_rule_rejects_malformed_yaml() {
        let err = test_rule("not: valid: yaml: at: all: -", "fn main() {}").unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_rule_rejects_a_rule_with_no_id() {
        let err = test_rule(RULE_WITHOUT_ID, "fn main() {}").unwrap_err();
        assert!(
            err.to_string().contains("non-empty"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_rule_accepts_a_previously_unsupported_language() {
        // TF-893: mirrors from_rule_accepts_a_previously_unsupported_language.
        let result = test_rule(GO_RULE, "package main").unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_rule_rejects_a_language_ast_grep_does_not_support() {
        let err = test_rule(COBOL_RULE, "PROGRAM-ID. MAIN.").unwrap_err();
        // See from_rule_rejects_a_language_ast_grep_does_not_support for
        // why this asserts on `{err:?}` rather than norma's own
        // "unsupported language" text.
        let chain = format!("{err:?}");
        assert!(
            chain.contains("Cobol"),
            "expected the error chain to name the rejected language, got: {chain}"
        );
    }

    /// Confirms `test_rule` and `Pattern::from_rule` (`register_pattern`'s
    /// own validation) agree on accept/reject for every rule shape the
    /// other tests in this module already probe individually -- the "dry
    /// run" property TF-891 is built on. A single hardcoded rule can only
    /// confirm one of the two ever agree on "accept"; this drives both
    /// through a valid rule and each of the known-bad/now-accepted shapes
    /// so a future divergence (e.g. `check_registerable` and
    /// `Pattern::from_rule` drifting apart again) would actually be
    /// caught here.
    #[test]
    fn test_rule_and_from_rule_agree_on_every_rule_shape() {
        fn from_rule_accepts(rule: &str) -> bool {
            Pattern::from_rule(
                "n".to_string(),
                "d".to_string(),
                None,
                rule.to_string(),
                true,
                Utc::now(),
                Utc::now(),
            )
            .is_ok()
        }

        for (rule, source) in [
            (RUST_NO_DEBUG_PRINT, "fn main() {}"),
            (RULE_WITHOUT_ID, "fn main() {}"),
            // Accepted since TF-893 (was rejected before).
            (GO_RULE, "package main"),
            // Still rejected -- ast-grep itself has no Cobol grammar.
            (COBOL_RULE, "PROGRAM-ID. MAIN."),
        ] {
            assert_eq!(
                test_rule(rule, source).is_ok(),
                from_rule_accepts(rule),
                "test_rule and Pattern::from_rule disagree on: {rule}"
            );
        }
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
        let err = apply_fixes("package main", "cobol", &[pattern]).unwrap_err();
        assert!(
            err.to_string().contains("unsupported language"),
            "unexpected error: {err}"
        );
    }
}
