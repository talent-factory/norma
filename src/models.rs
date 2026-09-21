use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Severity level for a pattern violation.
///
/// Mirrors `ast_grep_config::Severity` (see docs/adr/0002.md and
/// `pattern_engine::parse_rule`): norma derives this from the parsed rule
/// YAML at register time rather than accepting it as a separate input, so
/// a `Pattern`'s severity can never drift from the YAML that actually
/// produced it.
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
///
/// `id`, `language`, and `severity` are `pub(crate)` rather than `pub`,
/// and there is no public struct-literal constructor: the only way to
/// build one is [`Pattern::from_rule`], which derives all three from
/// `rule` and cannot be called with mismatched values. Without this, the
/// ADR 0002 invariant ("these three always match what parsing `rule`
/// produces") was only convention enforced by one caller
/// (`PatternStore::register_pattern`) -- nothing stopped a different call
/// site from building an inconsistent `Pattern` by hand. See
/// [`Pattern::from_trusted_row`] for the one sanctioned way to skip
/// re-deriving them (reading a row `PatternStore` already validated at
/// write time).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(test, derive(Deserialize))]
pub struct Pattern {
    /// The ast-grep RuleConfig `id` from the YAML in `rule`, e.g. `"no-debug-print-rust"`.
    id: String,
    /// norma's catalog name. Repeats across the language variants of one idea.
    pub name: String,
    /// norma's own, longer explanation (ast-grep's own `message` field
    /// inside `rule` is documented as "should be single line and concise").
    pub description: String,
    /// Free-text grouping, e.g. `"code-quality"`, `"creational"`. Not a fixed enum.
    pub category: Option<String>,
    /// norma's canonical language key: `"java"` | `"python"` | `"rust"` | `"typescript"`.
    language: String,
    /// Derived from the parsed `rule` YAML when the pattern was registered.
    severity: Severity,
    /// The full ast-grep RuleConfig YAML document (id/message/severity/language/rule/fix).
    rule: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Pattern {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn rule(&self) -> &str {
        &self.rule
    }

    /// Reconstructs a `Pattern` from a SQLite row `PatternStore` already
    /// wrote via [`Pattern::from_rule`] (see `pattern_store::row_to_pattern`).
    /// Skips re-parsing `rule` for performance, trusting that `id`,
    /// `language`, and `severity` still agree with it -- named distinctly
    /// from `from_rule` so it's visually obvious, at the one call site
    /// that uses it, that this path does not re-derive anything.
    ///
    /// One argument per `patterns` column by design (this exists to
    /// reconstruct exactly that row); a params struct would only move the
    /// verbosity to its one construction site instead of removing it.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_trusted_row(
        id: String,
        name: String,
        description: String,
        category: Option<String>,
        language: String,
        severity: Severity,
        rule: String,
        enabled: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            category,
            language,
            severity,
            rule,
            enabled,
            created_at,
            updated_at,
        }
    }
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
    /// The replacement text ast-grep's `Fixer::generate_replacement` would
    /// produce for this match, if the pattern's rule YAML has a `fix:`.
    /// Always populated when a fixer exists, independent of whether
    /// `pattern_engine::apply_fixes` would actually apply it -- see TF-890:
    /// "Anzeigen" (this field) and "Anwenden" (`apply_fixes`) are separate
    /// concerns, so a fix that would conflict with another match still
    /// shows up here.
    pub suggested_fix: Option<String>,
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
///
/// `checked_patterns` exists so "clean" and "nothing was checked" are never
/// indistinguishable: without it, a request for a language with zero
/// enabled patterns (or one whose only patterns all failed to parse) would
/// produce the exact same `passed: true, violations: []` as a genuinely
/// clean result. `pattern_engine::validate` also pushes a synthetic,
/// `Warning`-severity entry into `violations` in that case, so the
/// degraded coverage is visible in ordinary output too, not just in a
/// field a caller has to know to check.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationResult {
    pub violations: Vec<PatternViolation>,
    pub passed: bool,
    /// 1.0 = no violations relative to the number of patterns checked, 0.0 = worst case.
    /// Always 0.0 when `checked_patterns == 0` -- there is nothing to be
    /// confidently clean about.
    pub score: f64,
    /// How many patterns were actually parsed and run. Excludes patterns
    /// that were skipped because their stored `rule` no longer parses.
    pub checked_patterns: usize,
    pub duration_ms: u128,
}

/// A cluster of two or more matching patterns' fix ranges that overlap on
/// the same code -- see `pattern_engine::apply_fixes`. `location.file` is
/// always `None` here: `apply_fixes` only ever sees `code: String`, never a
/// path (the same asymmetry `CodeLocation` itself documents); a caller with
/// a real path (`cli::fix_files`) attaches it when rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FixConflict {
    pub pattern_ids: Vec<String>,
    pub location: CodeLocation,
}

/// The result of applying every enabled pattern's `fix`/`fixer` for one
/// language against one piece of code -- see `pattern_engine::apply_fixes`.
/// Shared by the CLI's `norma validate --fix` (writes `fixed_source` back
/// to the file) and the MCP `apply_pattern_fix` tool (returns it, never
/// touches the filesystem), per TF-890's per-surface split.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FixResult {
    pub fixed_source: String,
    /// How many matches' fixes were actually applied. Excludes both
    /// patterns without a `fix:` and matches dropped due to a conflict.
    pub applied_count: usize,
    /// Fail-safe: neither side of a conflicting pair (or larger cluster) of
    /// overlapping fix ranges is applied, rather than guessing a winner --
    /// see `pattern_engine::apply_fixes`'s doc comment.
    pub conflicts: Vec<FixConflict>,
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
