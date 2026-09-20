use clap::Parser;
use norma::cli::{self, Cli, Command};
use norma::models::ValidationResult;
use norma::pattern_store::PatternStore;
use norma::{mcp_server, pattern_engine};
use std::path::Path;
use std::sync::Arc;
use tracing::info;

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
    let db_path = cli::resolve_db_path(
        cli.db,
        std::env::var("NORMA_DB").ok(),
        std::env::var("HOME").ok(),
    );
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
            let reports = cli::validate_files(&files, language, &patterns)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&reports)?);
            } else {
                for report in &reports {
                    print_human_readable(&report.file, &report.result);
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
                    p.id(),
                    p.language(),
                    p.severity().as_str(),
                    p.name
                );
            }
        }
    }
    Ok(())
}

/// Prints a `ValidationResult` as `file:line:column: [severity] name -- text`
/// lines, one per violation, plus a one-line summary -- the format
/// `norma validate` uses without `--json` (this format is this function's
/// own choice; the CLI/pre-commit ticket only decided that text-by-default
/// vs. `--json` split, not the exact string). A synthetic coverage
/// warning (see `pattern_engine::validate`) has no `matched_text`, so its
/// `message` is shown instead.
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
        let detail = if v.matched_text.is_empty() {
            &v.message
        } else {
            &v.matched_text
        };
        println!(
            "{}:{}:{}: [{}] {} -- {}",
            file.display(),
            v.location.line + 1,
            v.location.column + 1,
            v.severity.as_str(),
            v.pattern_name,
            detail
        );
    }
    println!(
        "{} violation(s), score {:.2}, {} pattern(s) checked ({} ms)",
        result.violations.len(),
        result.score,
        result.checked_patterns,
        result.duration_ms
    );
}
