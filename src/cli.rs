use crate::models::{Pattern, Severity, ValidationResult};
use crate::pattern_engine;
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// norma: design-pattern and code-quality validation via ast-grep.
/// See docs/adr/0001.md -- one binary, subcommands share one core.
#[derive(Debug, Parser)]
#[command(name = "norma", version, about)]
pub struct Cli {
    /// Path to norma's SQLite pattern database. Overrides `$NORMA_DB`;
    /// without either, norma uses a fixed per-user location so the
    /// registry does not depend on the working directory.
    #[arg(long, global = true, value_name = "PATH")]
    pub db: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum Command {
    /// Run the MCP tool server on stdio.
    Serve,
    /// Validate one or more files against the patterns registered for
    /// their language.
    Validate {
        #[arg(long)]
        language: String,
        /// Print machine-readable JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
        /// Files to validate. Positional (and variadic) so `pre-commit`
        /// can append every staged file to the hook's `entry` line.
        #[arg(required = true, num_args = 1.., value_name = "FILE")]
        files: Vec<PathBuf>,
    },
    /// List every registered pattern.
    ListPatterns,
}

/// One file's validation outcome, as emitted by `norma validate --json`.
/// `validate` accepts many files, so the JSON output is always an array of
/// these -- even for a single file -- so consumers never have to branch on
/// the argument count.
#[derive(Debug, Serialize, PartialEq)]
pub struct FileReport {
    pub file: PathBuf,
    pub result: crate::models::ValidationResult,
}

/// Runs `pattern_engine::validate` against every file in `files` and
/// aggregates the results. Extracted out of `main.rs` (rather than living
/// inline in the `Command::Validate` match arm) so it can be covered by
/// `cargo test --lib` instead of only by hand -- this loop combines
/// several of the CLI's riskiest behaviors (multi-file aggregation, exit
/// status, JSON array shape) in one place.
///
/// A file that can't be read fails the whole call with an error naming
/// that file, rather than silently skipping it.
pub fn validate_files(
    files: &[PathBuf],
    language: &str,
    patterns: &[Pattern],
) -> anyhow::Result<Vec<FileReport>> {
    let mut reports = Vec::with_capacity(files.len());
    for file in files {
        let code = std::fs::read_to_string(file)
            .map_err(|e| anyhow::anyhow!("{}: {e}", file.display()))?;
        reports.push(FileReport {
            file: file.clone(),
            result: pattern_engine::validate(&code, language, patterns)?,
        });
    }
    Ok(reports)
}

/// Renders a `ValidationResult` as `file:line:column: [severity] name -- text`
/// lines, one per violation, plus a one-line summary -- the format
/// `norma validate` uses without `--json`. A synthetic coverage warning
/// (see `pattern_engine::validate`) has no `matched_text`, so its
/// `message` is shown instead.
///
/// Returns a `String` (rather than printing directly) so the
/// blocking/informational split below is covered by `cargo test --lib`
/// instead of only by hand -- `main.rs` does the actual `println!`.
///
/// The summary line separates blocking violations (`warning`/`error`
/// severity, the ones `pattern_engine::validate`'s severity-aware scoring
/// counts against `score`/`passed`) from informational ones (`off`/`hint`/
/// `info`, like the `observer-presence-*` patterns) -- printing a single
/// combined count next to `score` would misleadingly read as a
/// contradiction (e.g. "1 violation(s), score 1.00") when every violation
/// found was purely informational.
pub fn render_human_readable(file: &Path, result: &ValidationResult) -> String {
    if result.violations.is_empty() {
        return format!(
            "{}: no violations ({} ms)",
            file.display(),
            result.duration_ms
        );
    }
    let mut out = String::new();
    for v in &result.violations {
        let detail = if v.matched_text.is_empty() {
            &v.message
        } else {
            &v.matched_text
        };
        out.push_str(&format!(
            "{}:{}:{}: [{}] {} -- {}\n",
            file.display(),
            v.location.line + 1,
            v.location.column + 1,
            v.severity.as_str(),
            v.pattern_name,
            detail
        ));
    }
    let blocking = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, Severity::Warning | Severity::Error))
        .count();
    let informational = result.violations.len() - blocking;
    out.push_str(&format!(
        "{} violation(s) ({} informational), score {:.2}, {} pattern(s) checked ({} ms)",
        blocking, informational, result.score, result.checked_patterns, result.duration_ms
    ));
    out
}

/// Decides where norma's SQLite registry lives, in precedence order:
/// `--db`, then `$NORMA_DB`, then the XDG Base Directory data location
/// (`$XDG_DATA_HOME/norma/norma.db`, honoring an override the same way
/// `$XDG_DATA_HOME` itself is meant to), then a fixed per-user path under
/// `$HOME` (`$HOME/.local/share/norma/norma.db` -- `$XDG_DATA_HOME`'s own
/// documented default, used when the variable isn't set). Takes the
/// environment values as plain arguments (rather than reading them
/// itself) so this decision is testable without mutating process-global
/// environment state.
///
/// This is `~/.local/share`, not `~/.config`: per the XDG spec,
/// `$XDG_CONFIG_HOME` is for settings a user might edit or version
/// (norma has none yet), `$XDG_DATA_HOME` is for persistent application
/// *data* -- which is exactly what a pattern registry is.
///
/// Deliberately never defaults to a bare relative filename when better
/// information exists: a relative path would give the pre-commit hook a
/// stray `norma.db` in every consumer repo, and would hand an MCP client a
/// different (empty) registry for every working directory it happens to
/// launch `norma serve` from. The bare `norma.db` remains only as a
/// last-resort fallback for the (unusual) case of an unset `$HOME`.
pub fn resolve_db_path(
    flag: Option<PathBuf>,
    norma_db_env: Option<String>,
    xdg_data_home_env: Option<String>,
    home_env: Option<String>,
) -> PathBuf {
    if let Some(path) = flag {
        return path;
    }
    if let Some(env) = norma_db_env.filter(|v| !v.is_empty()) {
        return PathBuf::from(env);
    }
    if let Some(xdg_data_home) = xdg_data_home_env.filter(|v| !v.is_empty()) {
        return Path::new(&xdg_data_home).join("norma/norma.db");
    }
    if let Some(home) = home_env.filter(|v| !v.is_empty()) {
        return Path::new(&home).join(".local/share/norma/norma.db");
    }
    PathBuf::from("norma.db")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_validate_with_all_flags() {
        let cli = Cli::parse_from([
            "norma",
            "validate",
            "--language",
            "rust",
            "--json",
            "src/main.rs",
        ]);
        assert_eq!(
            cli.command,
            Command::Validate {
                language: "rust".to_string(),
                json: true,
                files: vec![PathBuf::from("src/main.rs")],
            }
        );
    }

    /// This is exactly the shape `pre-commit` produces: the hook's `entry`
    /// line followed by every staged file appended as its own argument.
    #[test]
    fn parses_validate_with_many_trailing_files() {
        let cli = Cli::parse_from([
            "norma",
            "validate",
            "--language",
            "rust",
            "--json",
            "src/a.rs",
            "src/b.rs",
            "src/c.rs",
        ]);
        assert_eq!(
            cli.command,
            Command::Validate {
                language: "rust".to_string(),
                json: true,
                files: vec![
                    PathBuf::from("src/a.rs"),
                    PathBuf::from("src/b.rs"),
                    PathBuf::from("src/c.rs"),
                ],
            }
        );
    }

    #[test]
    fn validate_requires_at_least_one_file() {
        assert!(Cli::try_parse_from(["norma", "validate", "--language", "rust"]).is_err());
    }

    #[test]
    fn parses_serve_and_list_patterns() {
        assert_eq!(Cli::parse_from(["norma", "serve"]).command, Command::Serve);
        assert_eq!(
            Cli::parse_from(["norma", "list-patterns"]).command,
            Command::ListPatterns
        );
    }

    #[test]
    fn db_is_global_and_accepted_before_or_after_the_subcommand() {
        let before = Cli::parse_from(["norma", "--db", "/tmp/a.db", "list-patterns"]);
        let after = Cli::parse_from(["norma", "list-patterns", "--db", "/tmp/a.db"]);
        assert_eq!(before.db, Some(PathBuf::from("/tmp/a.db")));
        assert_eq!(after.db, Some(PathBuf::from("/tmp/a.db")));
        assert_eq!(Cli::parse_from(["norma", "list-patterns"]).db, None);
    }

    // --- resolve_db_path -----------------------------------------------

    #[test]
    fn resolve_db_path_prefers_the_flag_over_everything_else() {
        let path = resolve_db_path(
            Some(PathBuf::from("/explicit.db")),
            Some("/from-env.db".to_string()),
            Some("/xdg-data".to_string()),
            Some("/home/daniel".to_string()),
        );
        assert_eq!(path, PathBuf::from("/explicit.db"));
    }

    #[test]
    fn resolve_db_path_prefers_norma_db_env_over_xdg_data_home() {
        let path = resolve_db_path(
            None,
            Some("/from-env.db".to_string()),
            Some("/xdg-data".to_string()),
            Some("/home/daniel".to_string()),
        );
        assert_eq!(path, PathBuf::from("/from-env.db"));
    }

    #[test]
    fn resolve_db_path_prefers_xdg_data_home_over_the_home_default() {
        let path = resolve_db_path(
            None,
            None,
            Some("/xdg-data".to_string()),
            Some("/home/daniel".to_string()),
        );
        assert_eq!(path, PathBuf::from("/xdg-data/norma/norma.db"));
    }

    #[test]
    fn resolve_db_path_ignores_an_empty_xdg_data_home_value() {
        // An env var set to the empty string (e.g. `XDG_DATA_HOME=`) must
        // not win over the $HOME-based default the way an unset one wouldn't.
        let path = resolve_db_path(
            None,
            None,
            Some(String::new()),
            Some("/home/daniel".to_string()),
        );
        assert_eq!(
            path,
            PathBuf::from("/home/daniel/.local/share/norma/norma.db")
        );
    }

    #[test]
    fn resolve_db_path_falls_back_to_a_fixed_path_under_home() {
        let path = resolve_db_path(None, None, None, Some("/home/daniel".to_string()));
        assert_eq!(
            path,
            PathBuf::from("/home/daniel/.local/share/norma/norma.db")
        );
    }

    #[test]
    fn resolve_db_path_ignores_an_empty_norma_db_value() {
        // An env var set to the empty string (e.g. `NORMA_DB=`) must not
        // win over the $HOME-based default the way an unset one wouldn't.
        let path = resolve_db_path(
            None,
            Some(String::new()),
            None,
            Some("/home/daniel".to_string()),
        );
        assert_eq!(
            path,
            PathBuf::from("/home/daniel/.local/share/norma/norma.db")
        );
    }

    #[test]
    fn resolve_db_path_falls_back_to_a_bare_relative_name_when_home_is_unset() {
        let path = resolve_db_path(None, None, None, None);
        assert_eq!(path, PathBuf::from("norma.db"));
    }

    // --- validate_files --------------------------------------------------

    const RUST_NO_DEBUG_PRINT: &str = r#"
id: no-debug-print-rust
message: Avoid println! in production code
severity: warning
language: Rust
rule:
  pattern: println!($$$ARGS)
"#;

    fn rust_pattern() -> Pattern {
        let now = chrono::Utc::now();
        Pattern::from_rule(
            "No Debug Print".to_string(),
            "d".to_string(),
            None,
            RUST_NO_DEBUG_PRINT.to_string(),
            true,
            now,
            now,
        )
        .unwrap()
    }

    #[test]
    fn validate_files_aggregates_a_clean_file_and_a_violating_one() {
        let dir = tempfile::tempdir().unwrap();
        let dirty = dir.path().join("dirty.rs");
        let clean = dir.path().join("clean.rs");
        std::fs::write(&dirty, "fn main() { println!(\"debug\"); }").unwrap();
        std::fs::write(&clean, "fn main() { tracing::info!(\"fine\"); }").unwrap();

        let reports =
            validate_files(&[dirty.clone(), clean.clone()], "rust", &[rust_pattern()]).unwrap();

        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0].file, dirty);
        assert!(!reports[0].result.passed);
        assert_eq!(reports[1].file, clean);
        assert!(reports[1].result.passed);
    }

    #[test]
    fn validate_files_still_returns_a_one_element_array_for_a_single_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("only.rs");
        std::fs::write(&file, "fn main() {}").unwrap();

        let reports =
            validate_files(std::slice::from_ref(&file), "rust", &[rust_pattern()]).unwrap();

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].file, file);
    }

    #[test]
    fn validate_files_names_the_file_when_it_cannot_be_read() {
        let missing = PathBuf::from("/definitely/does/not/exist.rs");
        let err =
            validate_files(std::slice::from_ref(&missing), "rust", &[rust_pattern()]).unwrap_err();
        assert!(
            err.to_string().contains("does/not/exist.rs"),
            "unexpected error: {err}"
        );
    }

    // --- render_human_readable --------------------------------------------

    /// A warning-severity match mixed with an info-severity one must not
    /// render as the self-contradictory "N violation(s), score 1.00" --
    /// see `render_human_readable`'s doc comment.
    #[test]
    fn render_human_readable_splits_blocking_from_informational_counts() {
        use crate::models::{CodeLocation, PatternViolation};

        let location = CodeLocation {
            file: None,
            line: 0,
            column: 0,
        };
        let result = ValidationResult {
            violations: vec![
                PatternViolation {
                    pattern_id: "singleton-quality-rust".to_string(),
                    pattern_name: "Singleton Implementation Quality".to_string(),
                    severity: Severity::Warning,
                    location: location.clone(),
                    matched_text: "impl Config { pub fn new() -> Config { Config } }".to_string(),
                    message: "d".to_string(),
                },
                PatternViolation {
                    pattern_id: "observer-presence-rust".to_string(),
                    pattern_name: "Observer Presence".to_string(),
                    severity: Severity::Info,
                    location,
                    matched_text: "struct Publisher { observers: Vec<Box<dyn Observer>> }"
                        .to_string(),
                    message: "d".to_string(),
                },
            ],
            passed: false,
            score: 0.5,
            checked_patterns: 2,
            duration_ms: 1,
        };

        let rendered = render_human_readable(Path::new("src/lib.rs"), &result);
        assert!(
            rendered.contains("1 violation(s) (1 informational), score 0.50"),
            "unexpected summary line, got: {rendered}"
        );
    }
}
