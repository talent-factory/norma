use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// norma: design-pattern and code-quality validation via ast-grep.
/// See docs/adr/0001.md -- one binary, subcommands share one core.
#[derive(Debug, Parser)]
#[command(name = "norma", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum Command {
    /// Run the MCP tool server on stdio.
    Serve,
    /// Validate one file against the patterns registered for its language.
    Validate {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        language: String,
        /// Print machine-readable JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
    },
    /// List every registered pattern.
    ListPatterns,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_validate_with_all_flags() {
        let cli = Cli::parse_from([
            "norma",
            "validate",
            "--file",
            "src/main.rs",
            "--language",
            "rust",
            "--json",
        ]);
        assert_eq!(
            cli.command,
            Command::Validate {
                file: PathBuf::from("src/main.rs"),
                language: "rust".to_string(),
                json: true,
            }
        );
    }

    #[test]
    fn parses_serve_and_list_patterns() {
        assert_eq!(Cli::parse_from(["norma", "serve"]).command, Command::Serve);
        assert_eq!(
            Cli::parse_from(["norma", "list-patterns"]).command,
            Command::ListPatterns
        );
    }
}
