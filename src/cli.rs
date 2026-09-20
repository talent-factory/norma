use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
}
