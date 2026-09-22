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
        /// Canonical language key, or any ast-grep alias (e.g. "rs" for
        /// Rust) -- see `pattern_engine::resolve_language`. Accepts any of
        /// the 28 languages ast-grep-language supports; norma currently
        /// ships default patterns for only Java, Python, Rust, and
        /// TypeScript (see docs/adr/0002.md), so the other 24 are
        /// registrable but have no patterns to check out of the box.
        #[arg(long)]
        language: String,
        /// Print machine-readable JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
        /// Rewrite each file in-place with every non-conflicting pattern
        /// fix applied (like `eslint --fix`/`biome --fix`), then report
        /// what remains. No dry-run in v1 -- keeping a clean git working
        /// tree beforehand is the caller's responsibility, documented but
        /// not enforced. See `pattern_engine::apply_fixes` and TF-890.
        #[arg(long)]
        fix: bool,
        /// Files to validate. Positional (and variadic) so `pre-commit`
        /// can append every staged file to the hook's `entry` line.
        #[arg(required = true, num_args = 1.., value_name = "FILE")]
        files: Vec<PathBuf>,
    },
    /// List every registered pattern.
    ListPatterns,
    /// Bulk-import every `.yml`/`.yaml` rule file found under a directory
    /// (TF-894) -- e.g. a cloned `sgconfig.yaml` rule directory, or a local
    /// checkout of ast-grep's own rule catalog. Wraps
    /// `PatternStore::import_rules`; see its doc comment for the
    /// per-document `name`/`description` derivation and skip-with-reason
    /// semantics.
    Import {
        /// Directory to scan recursively for `.yml`/`.yaml` rule files.
        dir: PathBuf,
        /// Applied to every pattern imported by this call -- see
        /// `PatternStore::import_rules`.
        #[arg(long)]
        category: Option<String>,
        /// Print machine-readable JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
    },
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

/// One file's outcome for `norma validate --fix`, as emitted by its
/// `--json` output.
#[derive(Debug, Serialize, PartialEq)]
pub struct FixReport {
    pub file: PathBuf,
    pub applied: usize,
    pub conflicts: Vec<crate::models::FixConflict>,
    /// `(pattern_id, parse error)` for every enabled pattern whose stored
    /// `rule` no longer parses -- forwarded from `FixResult::skipped_rules`.
    pub skipped_rules: Vec<(String, String)>,
    /// The remaining violations after applying every non-conflicting fix
    /// -- includes anything left over: unfixable patterns, conflicted
    /// matches, and any real defect the fixes didn't touch.
    pub result: crate::models::ValidationResult,
}

/// Runs `pattern_engine::apply_fixes` against every file in `files`,
/// writes the rewritten source back in-place when it actually changed
/// anything, and re-validates the (now-fixed) file so `FixReport.result`
/// reflects what's left rather than what applying fixes just erased. See
/// `pattern_engine::apply_fixes`'s doc comment for the fail-safe conflict
/// policy this reports via `FixReport.conflicts`.
///
/// A file is only written when at least one fix was applied -- an
/// unmodified file keeps its original mtime, so this is safe to run
/// against a directory `norma validate` (without `--fix`) already found
/// clean.
///
/// Every file is read up front, before any file is written: `--fix`
/// mutates files as a side effect, so failing an unreadable file *after*
/// already rewriting earlier ones in the batch (as `validate_files`'s
/// read-as-you-go loop would, harmlessly, since it never writes) would
/// leave those rewrites unreported -- the caller's only signal would be an
/// error naming just the one file that failed to read. Reading everything
/// first means a read failure here still means nothing was written.
pub fn fix_files(
    files: &[PathBuf],
    language: &str,
    patterns: &[Pattern],
) -> anyhow::Result<Vec<FixReport>> {
    let mut sources = Vec::with_capacity(files.len());
    for file in files {
        let code = std::fs::read_to_string(file)
            .map_err(|e| anyhow::anyhow!("{}: {e}", file.display()))?;
        sources.push((file, code));
    }

    let mut reports = Vec::with_capacity(files.len());
    for (file, code) in sources {
        let fix = pattern_engine::apply_fixes(&code, language, patterns)?;
        if fix.applied_count > 0 {
            std::fs::write(file, &fix.fixed_source)
                .map_err(|e| anyhow::anyhow!("{}: {e}", file.display()))?;
        }
        let result = pattern_engine::validate(&fix.fixed_source, language, patterns)?;
        reports.push(FixReport {
            file: file.clone(),
            applied: fix.applied_count,
            conflicts: fix.conflicts,
            skipped_rules: fix.skipped_rules,
            result,
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

/// Renders a `FixReport` as an "applied N fix(es)" line (if any were),
/// one `[warning] fix conflict` line per unapplied overlapping cluster,
/// and then the remaining violations via `render_human_readable` -- the
/// human-readable format `norma validate --fix` uses without `--json`.
pub fn render_fix_report(file: &Path, report: &FixReport) -> String {
    let mut out = String::new();
    if report.applied > 0 {
        out.push_str(&format!(
            "{}: applied {} fix(es)\n",
            file.display(),
            report.applied
        ));
    }
    for conflict in &report.conflicts {
        out.push_str(&format!(
            "{}:{}:{}: [warning] fix conflict -- overlapping fixes from {} were not applied\n",
            file.display(),
            conflict.location.line + 1,
            conflict.location.column + 1,
            conflict.pattern_ids.join(", ")
        ));
    }
    for (pattern_id, error) in &report.skipped_rules {
        out.push_str(&format!(
            "{}: [warning] pattern {pattern_id:?} has an unparsable rule, skipped: {error}\n",
            file.display()
        ));
    }
    out.push_str(&render_human_readable(file, &report.result));
    out
}

/// Recursively collects every `.yml`/`.yaml` file under `dir`, sorted for
/// deterministic output. The walk is recursive because that's how a cloned
/// `sgconfig.yaml` rule directory or a checkout of ast-grep's own rule
/// catalog nests rule files, in per-language subdirectories (TF-894,
/// `norma import`'s core file discovery).
///
/// Skips dot-directories (`.git`, `.github`, ...) -- never rule
/// directories, and `.git` in particular can be large. Uses
/// `DirEntry::file_type` (not `Path::is_dir`, which follows symlinks) to
/// decide whether to recurse, so a symlink cycle in an externally-sourced
/// rule directory can't send this into unbounded recursion.
fn collect_yaml_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| anyhow::anyhow!("{}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| anyhow::anyhow!("{}: {e}", dir.display()))?;
        let file_type = entry
            .file_type()
            .map_err(|e| anyhow::anyhow!("{}: {e}", entry.path().display()))?;
        let path = entry.path();
        if file_type.is_dir() {
            let is_dot_dir = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with('.'));
            if !is_dot_dir {
                files.extend(collect_yaml_files(&path)?);
            }
        } else if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("yml" | "yaml")
        ) {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// Reads every `.yml`/`.yaml` file under `dir` (see `collect_yaml_files`)
/// and bulk-imports every RuleConfig document they contain -- the `norma
/// import <dir>` subcommand's core (TF-894).
///
/// One `PatternStore::import_rules` call *per file*, not one call over
/// every file's content concatenated together: an earlier version
/// concatenated raw file bytes with a synthetic `---` separator, which
/// broke on the extremely common case of a rule file that already starts
/// with its own `---` document marker -- two adjacent `---` lines parse as
/// a single, valid, but *empty* YAML document, which used to show up as an
/// unidentifiable rejected rule even though every real rule in the
/// directory was perfectly valid (TF-894 PR #8 review). Per-file calls
/// also mean a genuine YAML syntax error in one file no longer aborts
/// every other file's import -- see the per-file `match` below.
///
/// Every file is read up front, before any is imported (like `fix_files`):
/// a file that can't be read fails the whole command with nothing written,
/// rather than leaving an arbitrary already-committed prefix behind.
/// `import_rules`'s upsert semantics would make re-running safe regardless,
/// but there's no reason to accept a partial result when reading every
/// file first is still cheap and has no side effects of its own.
///
/// Fails the whole command if `dir` contains no `.yml`/`.yaml` files at
/// all, or if `imported`/`skipped` both end up empty despite files being
/// found (e.g. every file was empty, comment-only, or otherwise contained
/// zero RuleConfig documents) -- a scan that found real files but
/// extracted nothing from any of them must not look like a clean,
/// successful no-op import.
pub async fn import_dir(
    store: &crate::pattern_store::PatternStore,
    dir: &Path,
    category: Option<String>,
) -> anyhow::Result<crate::pattern_store::ImportResult> {
    use crate::pattern_store::SkippedRule;

    let files = collect_yaml_files(dir)?;
    if files.is_empty() {
        anyhow::bail!("{}: no .yml/.yaml files found", dir.display());
    }

    let mut sources = Vec::with_capacity(files.len());
    for file in &files {
        let content = std::fs::read_to_string(file)
            .map_err(|e| anyhow::anyhow!("{}: {e}", file.display()))?;
        sources.push((file, content));
    }

    let mut imported = Vec::new();
    let mut skipped = Vec::new();
    for (file, content) in sources {
        match store.import_rules(&content, category.clone()).await {
            Ok(result) => {
                imported.extend(result.imported);
                // Prefix every skip reason with the file it came from --
                // `import_rules` alone has no file/path concept (its `yaml`
                // is just a string), so without this a skip from a
                // many-file directory import would be unattributable to
                // any particular file.
                skipped.extend(result.skipped.into_iter().map(|skip| SkippedRule {
                    id: skip.id,
                    reason: format!("{}: {}", file.display(), skip.reason),
                }));
            }
            // The whole file failed to even split into documents (a real
            // YAML syntax error) -- skip just this file, named and with
            // its reason, rather than aborting every other file's import
            // along with it.
            Err(crate::pattern_store::RegisterPatternError::InvalidRule(err)) => {
                skipped.push(SkippedRule {
                    id: None,
                    reason: format!("{}: {err}", file.display()),
                });
            }
            // A genuine storage failure is not a per-file problem -- it
            // aborts the whole command, same as `import_rules` documents.
            Err(err @ crate::pattern_store::RegisterPatternError::Storage(_)) => {
                return Err(err.into());
            }
        }
    }

    if imported.is_empty() && skipped.is_empty() {
        anyhow::bail!(
            "{}: found {} .yml/.yaml file(s), but none contained a RuleConfig document",
            dir.display(),
            files.len()
        );
    }

    Ok(crate::pattern_store::ImportResult { imported, skipped })
}

/// Renders an `ImportResult` as one line per imported pattern, one line per
/// skipped document (with its reason), and a one-line summary -- the
/// human-readable format `norma import` uses without `--json`.
pub fn render_import_result(result: &crate::pattern_store::ImportResult) -> String {
    let mut out = String::new();
    for pattern in &result.imported {
        out.push_str(&format!(
            "imported {} [{}]\n",
            pattern.id(),
            pattern.language()
        ));
    }
    for skip in &result.skipped {
        let name = skip.id.as_deref().unwrap_or("<no id>");
        out.push_str(&format!("[warning] skipped {name}: {}\n", skip.reason));
    }
    out.push_str(&format!(
        "{} imported, {} skipped",
        result.imported.len(),
        result.skipped.len()
    ));
    out
}

/// Whether `norma import`'s process should exit non-zero for `result` --
/// true whenever anything was skipped (`!result.fully_succeeded()`), so a
/// scripted/CI bulk-adoption run can't mistake a partially-rejected import
/// for a fully clean one. Broken out of `main.rs`'s `Command::Import` arm
/// so `cargo test --lib` can pin this decision directly, rather than only
/// through the compiled binary's exit code.
pub fn import_should_fail(result: &crate::pattern_store::ImportResult) -> bool {
    !result.fully_succeeded()
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
                fix: false,
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
                fix: false,
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
    fn parses_import_with_category_and_json() {
        let cli = Cli::parse_from([
            "norma",
            "import",
            "--category",
            "bulk-imported",
            "--json",
            "rules/",
        ]);
        assert_eq!(
            cli.command,
            Command::Import {
                dir: PathBuf::from("rules/"),
                category: Some("bulk-imported".to_string()),
                json: true,
            }
        );
    }

    #[test]
    fn parses_import_without_category() {
        let cli = Cli::parse_from(["norma", "import", "rules/"]);
        assert_eq!(
            cli.command,
            Command::Import {
                dir: PathBuf::from("rules/"),
                category: None,
                json: false,
            }
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
                    suggested_fix: None,
                },
                PatternViolation {
                    pattern_id: "observer-presence-rust".to_string(),
                    pattern_name: "Observer Presence".to_string(),
                    severity: Severity::Info,
                    location,
                    matched_text: "struct Publisher { observers: Vec<Box<dyn Observer>> }"
                        .to_string(),
                    message: "d".to_string(),
                    suggested_fix: None,
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

    // --- fix_files / render_fix_report (TF-890) ----------------------------

    const RUST_UNWRAP_WITH_FIX: &str = r#"
id: no-unwrap-rust
message: Avoid unwrap() in production code
severity: warning
language: Rust
rule:
  pattern: $EXPR.unwrap()
fix: $EXPR.expect("TODO")
"#;

    fn rust_unwrap_pattern() -> Pattern {
        let now = chrono::Utc::now();
        Pattern::from_rule(
            "No Unwrap".to_string(),
            "d".to_string(),
            None,
            RUST_UNWRAP_WITH_FIX.to_string(),
            true,
            now,
            now,
        )
        .unwrap()
    }

    #[test]
    fn parses_validate_with_the_fix_flag() {
        let cli = Cli::parse_from([
            "norma",
            "validate",
            "--language",
            "rust",
            "--fix",
            "src/main.rs",
        ]);
        assert_eq!(
            cli.command,
            Command::Validate {
                language: "rust".to_string(),
                json: false,
                fix: true,
                files: vec![PathBuf::from("src/main.rs")],
            }
        );
    }

    #[test]
    fn fix_files_rewrites_the_file_in_place_and_reports_what_remains() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("has_unwrap.rs");
        std::fs::write(&file, "fn main() { value.unwrap(); }").unwrap();

        let reports = fix_files(
            std::slice::from_ref(&file),
            "rust",
            &[rust_unwrap_pattern()],
        )
        .unwrap();

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].applied, 1);
        assert!(reports[0].conflicts.is_empty());
        assert!(
            reports[0].result.passed,
            "the just-applied fix must leave nothing outstanding"
        );
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            r#"fn main() { value.expect("TODO"); }"#
        );
    }

    #[test]
    fn fix_files_does_not_touch_a_file_with_nothing_to_fix() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("clean.rs");
        let original = "fn main() {}";
        std::fs::write(&file, original).unwrap();

        let reports = fix_files(
            std::slice::from_ref(&file),
            "rust",
            &[rust_unwrap_pattern()],
        )
        .unwrap();

        assert_eq!(reports[0].applied, 0);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), original);
    }

    #[test]
    fn fix_files_reports_a_pattern_whose_stored_rule_no_longer_parses() {
        use crate::models::Severity;

        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("has_unwrap.rs");
        std::fs::write(&file, "fn main() { value.unwrap(); }").unwrap();

        let now = chrono::Utc::now();
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

        let reports = fix_files(
            std::slice::from_ref(&file),
            "rust",
            &[rust_unwrap_pattern(), broken],
        )
        .unwrap();

        // The still-valid pattern is applied normally...
        assert_eq!(reports[0].applied, 1);
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            r#"fn main() { value.expect("TODO"); }"#
        );
        // ...and the broken one is reported, not silently dropped.
        assert_eq!(reports[0].skipped_rules.len(), 1);
        assert_eq!(reports[0].skipped_rules[0].0, "was-valid-once");

        let rendered = render_fix_report(&file, &reports[0]);
        assert!(
            rendered.contains("\"was-valid-once\" has an unparsable rule, skipped:"),
            "got: {rendered}"
        );
    }

    #[test]
    fn render_fix_report_shows_applied_count_then_remaining_violations() {
        let report = FixReport {
            file: PathBuf::from("src/lib.rs"),
            applied: 2,
            conflicts: vec![],
            skipped_rules: vec![],
            result: ValidationResult {
                violations: vec![],
                passed: true,
                score: 1.0,
                checked_patterns: 1,
                duration_ms: 1,
            },
        };
        let rendered = render_fix_report(Path::new("src/lib.rs"), &report);
        assert!(rendered.contains("applied 2 fix(es)"), "got: {rendered}");
        assert!(rendered.contains("no violations"), "got: {rendered}");
    }

    #[test]
    fn render_fix_report_surfaces_a_conflict_as_a_warning_line() {
        use crate::models::{CodeLocation, FixConflict};

        let report = FixReport {
            file: PathBuf::from("src/lib.rs"),
            applied: 0,
            conflicts: vec![FixConflict {
                pattern_ids: vec!["a".to_string(), "b".to_string()],
                location: CodeLocation {
                    file: None,
                    line: 4,
                    column: 2,
                },
            }],
            skipped_rules: vec![],
            result: ValidationResult {
                violations: vec![],
                passed: true,
                score: 1.0,
                checked_patterns: 2,
                duration_ms: 1,
            },
        };
        let rendered = render_fix_report(Path::new("src/lib.rs"), &report);
        assert!(
            rendered.contains("src/lib.rs:5:3: [warning] fix conflict -- overlapping fixes from a, b were not applied"),
            "got: {rendered}"
        );
    }

    // --- import (TF-894) ------------------------------------------------

    use crate::pattern_store::PatternStore;

    async fn empty_store() -> PatternStore {
        let dir = tempfile::tempdir().unwrap();
        let store = PatternStore::new(dir.path().join("norma-test.db"))
            .await
            .unwrap();
        // Keep the tempdir alive for the test's duration.
        std::mem::forget(dir);
        store
    }

    const RUST_RULE: &str = r#"
id: no-debug-print-rust
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
    fn collect_yaml_files_finds_yml_and_yaml_recursively_and_ignores_other_extensions() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.yaml"), "").unwrap();
        std::fs::write(dir.path().join("b.yml"), "").unwrap();
        std::fs::write(dir.path().join("README.md"), "").unwrap();
        let nested = dir.path().join("nested");
        std::fs::create_dir(&nested).unwrap();
        std::fs::write(nested.join("c.yaml"), "").unwrap();

        let files = collect_yaml_files(dir.path()).unwrap();
        let names: Vec<_> = files
            .iter()
            .map(|f| f.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["a.yaml", "b.yml", "c.yaml"]);
    }

    #[test]
    fn collect_yaml_files_skips_dot_directories() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("real.yaml"), "").unwrap();
        let dot_dir = dir.path().join(".git");
        std::fs::create_dir(&dot_dir).unwrap();
        std::fs::write(dot_dir.join("config.yaml"), "").unwrap();

        let files = collect_yaml_files(dir.path()).unwrap();
        let names: Vec<_> = files
            .iter()
            .map(|f| f.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["real.yaml"]);
    }

    #[tokio::test]
    async fn import_dir_imports_every_yaml_file_under_the_directory() {
        let store = empty_store().await;
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("rust.yaml"), RUST_RULE).unwrap();
        std::fs::write(dir.path().join("go.yaml"), GO_RULE).unwrap();

        let result = import_dir(&store, dir.path(), Some("bulk".to_string()))
            .await
            .unwrap();

        assert_eq!(result.imported.len(), 2);
        assert!(result.skipped.is_empty());
        assert!(result
            .imported
            .iter()
            .all(|p| p.category.as_deref() == Some("bulk")));
    }

    #[tokio::test]
    async fn import_dir_fails_when_the_directory_has_no_yaml_files() {
        let store = empty_store().await;
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("README.md"), "not a rule").unwrap();

        let err = import_dir(&store, dir.path(), None).await.unwrap_err();
        assert!(
            err.to_string().contains("no .yml/.yaml files found"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn import_dir_imports_files_that_each_start_with_their_own_document_marker() {
        // Regression test for TF-894 PR #8's review finding: two rule
        // files, each independently a valid, self-contained YAML document
        // starting with `---` -- the extremely common convention for a
        // cloned `sgconfig.yaml` rule directory or ast-grep's own catalog.
        // The old concatenation-based `import_dir` turned the boundary
        // between two such files into a `---\n---\n` sequence, which
        // parsed as a spurious empty document and showed up as an
        // unidentifiable skip.
        let store = empty_store().await;
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("rust.yaml"), format!("---\n{RUST_RULE}")).unwrap();
        std::fs::write(dir.path().join("go.yaml"), format!("---\n{GO_RULE}")).unwrap();

        let result = import_dir(&store, dir.path(), None).await.unwrap();

        assert_eq!(result.imported.len(), 2, "got: {result:?}");
        assert!(
            result.skipped.is_empty(),
            "no phantom skip must appear, got: {:?}",
            result.skipped
        );
    }

    #[tokio::test]
    async fn import_dir_attributes_a_skip_to_the_file_it_came_from() {
        let store = empty_store().await;
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("rust.yaml"), RUST_RULE).unwrap();
        std::fs::write(
            dir.path().join("cobol.yaml"),
            r#"
id: no-display-cobol
message: Avoid DISPLAY in production code
severity: warning
language: Cobol
rule:
  pattern: DISPLAY $$$ARGS
"#,
        )
        .unwrap();

        let result = import_dir(&store, dir.path(), None).await.unwrap();

        assert_eq!(result.imported.len(), 1);
        assert_eq!(result.skipped.len(), 1);
        assert!(
            result.skipped[0].reason.contains("cobol.yaml: "),
            "expected the skip reason to name its source file, got: {}",
            result.skipped[0].reason
        );
    }

    #[tokio::test]
    async fn import_dir_isolates_a_yaml_syntax_error_in_one_file_from_the_rest() {
        let store = empty_store().await;
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("rust.yaml"), RUST_RULE).unwrap();
        std::fs::write(
            dir.path().join("broken.yaml"),
            "not: valid: yaml: at: all: -",
        )
        .unwrap();

        let result = import_dir(&store, dir.path(), None).await.unwrap();

        assert_eq!(
            result.imported.len(),
            1,
            "the well-formed file must still import despite the other file's syntax error"
        );
        assert_eq!(result.skipped.len(), 1);
        assert!(result.skipped[0].reason.contains("broken.yaml: "));
    }

    #[tokio::test]
    async fn import_dir_fails_when_every_file_contains_no_ruleconfig_document() {
        let store = empty_store().await;
        let dir = tempfile::tempdir().unwrap();
        // Present, real .yaml files -- just none of them are (or contain) a
        // rule: comment-only, blank, and a non-rule document all count.
        std::fs::write(dir.path().join("empty.yaml"), "# just a comment\n").unwrap();
        std::fs::write(dir.path().join("blank.yaml"), "\n\n").unwrap();

        let err = import_dir(&store, dir.path(), None).await.unwrap_err();
        assert!(
            err.to_string()
                .contains("none contained a RuleConfig document"),
            "unexpected error: {err}"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn import_dir_fails_and_writes_nothing_when_one_file_is_unreadable() {
        use std::os::unix::fs::PermissionsExt;

        let store = empty_store().await;
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("rust.yaml"), RUST_RULE).unwrap();
        let unreadable = dir.path().join("unreadable.yaml");
        std::fs::write(&unreadable, GO_RULE).unwrap();
        std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o000)).unwrap();

        let result = import_dir(&store, dir.path(), None).await;

        std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o644)).unwrap();

        assert!(
            result.is_err(),
            "an unreadable file must fail the whole import"
        );
        assert_eq!(
            store.list_all_patterns().await.unwrap().len(),
            0,
            "nothing must be written when reading fails before any import is attempted"
        );
    }

    #[test]
    fn import_should_fail_is_true_only_when_something_was_skipped() {
        use crate::pattern_store::{ImportResult, SkippedRule};

        assert!(!import_should_fail(&ImportResult {
            imported: vec![],
            skipped: vec![],
        }));
        assert!(import_should_fail(&ImportResult {
            imported: vec![],
            skipped: vec![SkippedRule {
                id: None,
                reason: "d".to_string(),
            }],
        }));
    }

    #[test]
    fn render_import_result_lists_imported_and_skipped_with_a_summary() {
        use crate::pattern_store::{ImportResult, SkippedRule};

        let now = chrono::Utc::now();
        let pattern = Pattern::from_rule(
            "no-debug-print-rust".to_string(),
            "d".to_string(),
            None,
            RUST_RULE.to_string(),
            true,
            now,
            now,
        )
        .unwrap();
        let result = ImportResult {
            imported: vec![pattern],
            skipped: vec![SkippedRule {
                id: Some("display-in-cobol".to_string()),
                reason: "unsupported language: Cobol".to_string(),
            }],
        };
        let rendered = render_import_result(&result);
        assert!(rendered.contains("imported no-debug-print-rust [rust]"));
        assert!(
            rendered.contains("[warning] skipped display-in-cobol: unsupported language: Cobol")
        );
        assert!(rendered.contains("1 imported, 1 skipped"));
    }
}
