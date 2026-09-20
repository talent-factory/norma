use clap::Parser;
use norma::cli::{Cli, Command};
use norma::models::ValidationResult;
use norma::pattern_store::PatternStore;
use norma::{mcp_server, pattern_engine};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let cli = Cli::parse();
    let store = Arc::new(PatternStore::new("norma.db").await?);
    store.seed_defaults().await?;

    match cli.command {
        Command::Serve => {
            info!("starting norma MCP server on stdio");
            mcp_server::serve(store).await?;
        }
        Command::Validate { file, language, json } => {
            let code = std::fs::read_to_string(&file)?;
            let patterns = store.get_patterns_for_language(&language).await?;
            let result = pattern_engine::validate(&code, &language, &patterns);
            if json {
                let json_output = serde_json::to_string_pretty(&result)?;
                debug!("{}", json_output);
            } else {
                print_human_readable(&file, &result);
            }
            if !result.passed {
                std::process::exit(1);
            }
        }
        Command::ListPatterns => {
            for p in store.list_all_patterns().await? {
                debug!("{:<28} {:<10} [{}] {}", p.id, p.language, p.severity.as_str(), p.name);
            }
        }
    }
    Ok(())
}

/// Logs a `ValidationResult` as `file:line:column: [severity] name -- text`
/// lines, one per violation, plus a one-line summary -- the format
/// `norma validate` uses without `--json` (see the CLI/pre-commit ticket).
fn print_human_readable(file: &Path, result: &ValidationResult) {
    if result.violations.is_empty() {
        debug!("{}: no violations ({} ms)", file.display(), result.duration_ms);
        return;
    }
    for v in &result.violations {
        debug!(
            "{}:{}:{}: [{}] {} -- {}",
            file.display(),
            v.location.line + 1,
            v.location.column + 1,
            v.severity.as_str(),
            v.pattern_name,
            v.matched_text
        );
    }
    debug!(
        "{} violation(s), score {:.2} ({} ms)",
        result.violations.len(),
        result.score,
        result.duration_ms
    );
}
