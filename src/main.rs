use clap::Parser;
use norma::cli::{Cli, Command};
use norma::models::ValidationResult;
use norma::pattern_store::PatternStore;
use norma::{mcp_server, pattern_engine};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

/// One file's validation outcome, as emitted by `norma validate --json`.
/// `validate` accepts many files, so the JSON output is always an array of
/// these -- even for a single file -- so consumers never have to branch on
/// the argument count.
#[derive(Serialize)]
struct FileReport<'a> {
    file: &'a Path,
    result: ValidationResult,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logs MUST go to stderr, never stdout: `norma serve` speaks JSON-RPC
    // over stdio, so anything this process (or `rmcp`'s own instrumentation,
    // which uses the same global subscriber) writes to stdout is interleaved
    // into the protocol stream and corrupts it for real MCP clients.
    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let db_path = resolve_db_path(cli.db);
    if let Some(parent) = db_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let store = Arc::new(PatternStore::new(&db_path).await?);
    store.seed_defaults().await?;

    match cli.command {
        Command::Serve => {
            info!(db = %db_path.display(), "starting norma MCP server on stdio");
            mcp_server::serve(store).await?;
        }
        Command::Validate {
            files,
            language,
            json,
        } => {
            // Resolve the language once, before any file is read: an
            // unsupported or misspelled `--language` must be a loud error,
            // not a run that matches zero patterns and reports "no violations".
            let language = pattern_engine::resolve_language(&language)?;
            let patterns = store.get_patterns_for_language(language).await?;

            let mut reports = Vec::with_capacity(files.len());
            for file in &files {
                let code = std::fs::read_to_string(file)
                    .map_err(|e| anyhow::anyhow!("{}: {e}", file.display()))?;
                reports.push(FileReport {
                    file,
                    result: pattern_engine::validate(&code, language, &patterns)?,
                });
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&reports)?);
            } else {
                for report in &reports {
                    print_human_readable(report.file, &report.result);
                }
            }
            // Any file with violations fails the whole run -- that is what
            // makes this usable as a pre-commit hook over a batch of files.
            if reports.iter().any(|r| !r.result.passed) {
                std::process::exit(1);
            }
        }
        Command::ListPatterns => {
            for p in store.list_all_patterns().await? {
                println!(
                    "{:<28} {:<10} [{}] {}",
                    p.id,
                    p.language,
                    p.severity.as_str(),
                    p.name
                );
            }
        }
    }
    Ok(())
}

/// Decides where norma's SQLite registry lives, in precedence order:
/// `--db`, then `$NORMA_DB`, then a fixed per-user path under `$HOME`.
/// It deliberately never defaults to a bare relative filename when better
/// information exists: a relative path would give the pre-commit hook a
/// stray `norma.db` in every consumer repo, and would hand an MCP client
/// a different (empty) registry for every working directory it happens to
/// launch `norma serve` from. The bare `norma.db` remains only as a
/// last-resort fallback for the (unusual) case of an unset `$HOME`.
fn resolve_db_path(flag: Option<PathBuf>) -> PathBuf {
    if let Some(path) = flag {
        return path;
    }
    if let Some(env) = std::env::var_os("NORMA_DB").filter(|v| !v.is_empty()) {
        return PathBuf::from(env);
    }
    if let Some(home) = std::env::var_os("HOME").filter(|v| !v.is_empty()) {
        return PathBuf::from(home).join(".local/share/norma/norma.db");
    }
    PathBuf::from("norma.db")
}

/// Prints a `ValidationResult` as `file:line:column: [severity] name -- text`
/// lines, one per violation, plus a one-line summary -- the format
/// `norma validate` uses without `--json` (see the CLI/pre-commit ticket).
fn print_human_readable(file: &Path, result: &ValidationResult) {
    if result.violations.is_empty() {
        println!(
            "{}: no violations ({} ms)",
            file.display(),
            result.duration_ms
        );
        return;
    }
    for v in &result.violations {
        println!(
            "{}:{}:{}: [{}] {} -- {}",
            file.display(),
            v.location.line + 1,
            v.location.column + 1,
            v.severity.as_str(),
            v.pattern_name,
            v.matched_text
        );
    }
    println!(
        "{} violation(s), score {:.2} ({} ms)",
        result.violations.len(),
        result.score,
        result.duration_ms
    );
}
