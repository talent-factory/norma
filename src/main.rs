use clap::Parser;
use norma::cli::{self, Cli, Command};
use norma::pattern_store::PatternStore;
use norma::{mcp_server, pattern_engine};
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
        std::env::var("XDG_DATA_HOME").ok(),
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
            fix,
        } => {
            // Resolve the language once, before any file is read: an
            // unsupported or misspelled `--language` must be a loud error,
            // not a run that matches zero patterns and reports "no violations".
            let language = pattern_engine::resolve_language(&language)?;
            let patterns = store.get_patterns_for_language(language).await?;

            // `--fix` rewrites each file in-place (see `cli::fix_files`)
            // before reporting; otherwise this is a read-only check. Both
            // paths share the same pass/fail exit-status rule below, so
            // `--fix` still exits 1 for whatever the fixes couldn't clear.
            let passed = if fix {
                let reports = cli::fix_files(&files, language, &patterns)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&reports)?);
                } else {
                    for report in &reports {
                        println!("{}", cli::render_fix_report(&report.file, report));
                    }
                }
                reports.iter().all(|r| r.result.passed)
            } else {
                let reports = cli::validate_files(&files, language, &patterns)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&reports)?);
                } else {
                    for report in &reports {
                        println!(
                            "{}",
                            cli::render_human_readable(&report.file, &report.result)
                        );
                    }
                }
                reports.iter().all(|r| r.result.passed)
            };
            // Any file with violations fails the whole run -- that is what
            // makes this usable as a pre-commit hook over a batch of files.
            if !passed {
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
