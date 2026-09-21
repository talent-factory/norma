use crate::models::{Pattern, Severity};
use anyhow::Result;
use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::Row;
use std::path::Path;

/// Why `PatternStore::register_pattern` failed. Distinguishes a
/// client-caused problem (bad YAML, empty `id`, unsupported language --
/// see `Pattern::from_rule`) from a storage failure (SQLite I/O), so a
/// caller like the MCP server's `register_pattern` tool can map each to
/// the right kind of error instead of reporting every failure as "your
/// input was wrong" -- which is exactly wrong for a disk-full or
/// corrupted-database condition, and would send an MCP client (often an
/// LLM agent) off retrying with cosmetic input tweaks that could never fix
/// the real problem.
#[derive(Debug)]
pub enum RegisterPatternError {
    InvalidRule(anyhow::Error),
    Storage(anyhow::Error),
}

impl std::fmt::Display for RegisterPatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterPatternError::InvalidRule(e) => write!(f, "{e}"),
            RegisterPatternError::Storage(e) => write!(f, "storage error: {e}"),
        }
    }
}

impl std::error::Error for RegisterPatternError {}

/// SQLite-backed registry of `Pattern`s. See docs/adr/0002.md for the
/// schema rationale: one row per language, `rule` holds the full
/// ast-grep RuleConfig YAML, and `language`/`severity` are always
/// derived from parsing it rather than accepted separately.
pub struct PatternStore {
    pool: SqlitePool,
}

impl PatternStore {
    /// Opens (creating if missing) a SQLite database at `db_path` and
    /// ensures the `patterns` table exists.
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        // Build the connection options programmatically rather than
        // interpolating the path into a `sqlite:...?mode=rwc` URL: a path
        // containing `?` or `#` would otherwise be parsed as a query string
        // or fragment and silently point at the wrong file.
        let options = SqliteConnectOptions::new()
            .filename(db_path.as_ref())
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS patterns (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                description TEXT NOT NULL,
                category    TEXT,
                language    TEXT NOT NULL,
                severity    TEXT NOT NULL,
                rule        TEXT NOT NULL,
                enabled     INTEGER NOT NULL DEFAULT 1,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_patterns_language ON patterns(language)")
            .execute(&pool)
            .await?;
        Ok(Self { pool })
    }

    /// Registers a pattern. Fails fast if `rule_yaml` is not a valid
    /// ast-grep RuleConfig -- nothing is written to SQLite in that case.
    /// `id`, `language` and `severity` are derived from the parsed YAML by
    /// `Pattern::from_rule`, never accepted as separate inputs, so they
    /// cannot drift from it. A rule for a language norma does not support
    /// is rejected the same way -- storing it would produce a row no
    /// lookup could ever reach.
    pub async fn register_pattern(
        &self,
        name: String,
        description: String,
        category: Option<String>,
        rule_yaml: String,
    ) -> std::result::Result<Pattern, RegisterPatternError> {
        let now = Utc::now();
        let pattern = Pattern::from_rule(name, description, category, rule_yaml, true, now, now)
            .map_err(RegisterPatternError::InvalidRule)?;
        self.save_pattern(pattern)
            .await
            .map_err(RegisterPatternError::Storage)
    }

    async fn save_pattern(&self, pattern: Pattern) -> Result<Pattern> {
        sqlx::query(
            r#"
            INSERT INTO patterns
                (id, name, description, category, language, severity, rule, enabled, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                category = excluded.category,
                language = excluded.language,
                severity = excluded.severity,
                rule = excluded.rule,
                enabled = excluded.enabled,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(pattern.id())
        .bind(&pattern.name)
        .bind(&pattern.description)
        .bind(&pattern.category)
        .bind(pattern.language())
        .bind(pattern.severity().as_str())
        .bind(pattern.rule())
        .bind(pattern.enabled)
        .bind(pattern.created_at.to_rfc3339())
        .bind(pattern.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(pattern)
    }

    /// Returns every enabled pattern registered for `language` (norma's
    /// canonical key, e.g. `"rust"` -- see `pattern_engine::language_key`).
    pub async fn get_patterns_for_language(&self, language: &str) -> Result<Vec<Pattern>> {
        let rows = sqlx::query(
            "SELECT id, name, description, category, language, severity, rule, enabled, created_at, updated_at
             FROM patterns WHERE enabled = 1 AND language = ?",
        )
        .bind(language)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(row_to_pattern).collect()
    }

    /// Returns every pattern, enabled or not, across all languages.
    pub async fn list_all_patterns(&self) -> Result<Vec<Pattern>> {
        let rows = sqlx::query(
            "SELECT id, name, description, category, language, severity, rule, enabled, created_at, updated_at
             FROM patterns ORDER BY language, id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(row_to_pattern).collect()
    }

    /// Registers every default pattern whose id isn't already a row in the
    /// store. Safe to call on every startup -- this is how an existing
    /// installation picks up default patterns added by a newer norma
    /// version (e.g. the GoF v2 set added after the MVP shipped), without
    /// ever touching a row that already exists. That's a deliberate,
    /// narrow guarantee: a pre-existing row is left completely alone even
    /// if the shipped `rule` for that id has since changed, so a user's
    /// own edits (or just an MVP-era row) can never be silently reset.
    pub async fn seed_defaults(&self) -> Result<()> {
        let existing_ids: std::collections::HashSet<String> = self
            .list_all_patterns()
            .await?
            .into_iter()
            .map(|p| p.id().to_string())
            .collect();
        for def in crate::default_patterns::ALL {
            let now = Utc::now();
            let pattern = Pattern::from_rule(
                def.name.to_string(),
                def.description.to_string(),
                Some(def.category.to_string()),
                def.rule.to_string(),
                true,
                now,
                now,
            )?;
            if existing_ids.contains(pattern.id()) {
                continue;
            }
            self.save_pattern(pattern).await?;
        }
        Ok(())
    }
}

/// Reconstructs a `Pattern` from a stored row without re-parsing `rule` --
/// see `Pattern::from_trusted_row`. Trusts that `id`/`language`/`severity`
/// still agree with `rule`, which holds as long as every row was written
/// by `save_pattern` (i.e. every row ever went through `Pattern::from_rule`
/// first).
fn row_to_pattern(row: &SqliteRow) -> Result<Pattern> {
    let severity_str: String = row.get("severity");
    let created_at_str: String = row.get("created_at");
    let updated_at_str: String = row.get("updated_at");
    Ok(Pattern::from_trusted_row(
        row.get("id"),
        row.get("name"),
        row.get("description"),
        row.get("category"),
        row.get("language"),
        Severity::parse(&severity_str)
            .ok_or_else(|| anyhow::anyhow!("unknown severity in database: {severity_str}"))?,
        row.get("rule"),
        row.get("enabled"),
        chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc),
        chrono::DateTime::parse_from_rfc3339(&updated_at_str)?.with_timezone(&Utc),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_store() -> PatternStore {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("norma-test.db");
        let store = PatternStore::new(&db_path).await.unwrap();
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

    #[tokio::test]
    async fn register_pattern_derives_id_language_and_severity_from_the_yaml() {
        let store = test_store().await;
        let pattern = store
            .register_pattern(
                "No Debug Print".to_string(),
                "println! left in production code".to_string(),
                Some("code-quality".to_string()),
                RUST_RULE.to_string(),
            )
            .await
            .unwrap();
        assert_eq!(pattern.id(), "no-debug-print-rust");
        assert_eq!(pattern.language(), "rust");
        assert_eq!(pattern.severity(), Severity::Warning);
    }

    #[tokio::test]
    async fn register_pattern_fails_fast_on_malformed_yaml() {
        let store = test_store().await;
        let result = store
            .register_pattern(
                "Broken".to_string(),
                "broken rule".to_string(),
                None,
                "not: valid: yaml: at: all: -".to_string(),
            )
            .await;
        assert!(result.is_err());
        // Nothing should have been written.
        assert!(store.list_all_patterns().await.unwrap().is_empty());
    }

    // Go is one of the 24 languages TF-893 unlocked -- previously
    // rejected by `register_pattern`, now stored like any other language
    // (just without shipped default patterns; see `default_patterns.rs`).
    const GO_RULE: &str = r#"
id: no-debug-print-go
message: Avoid fmt.Println in production code
severity: warning
language: Go
rule:
  pattern: fmt.Println($$$ARGS)
"#;

    // Cobol isn't one of the 28 `SupportLang` variants ast-grep-language
    // ships -- no version of norma has ever supported it.
    const COBOL_RULE: &str = r#"
id: no-debug-print-cobol
message: Avoid DISPLAY in production code
severity: warning
language: Cobol
rule:
  pattern: DISPLAY $$$ARGS
"#;

    #[tokio::test]
    async fn register_pattern_accepts_a_previously_unsupported_language() {
        let store = test_store().await;
        let pattern = store
            .register_pattern(
                "No Debug Print".to_string(),
                "fmt.Println left in production code".to_string(),
                None,
                GO_RULE.to_string(),
            )
            .await
            .expect("a Go pattern must now be stored");
        assert_eq!(pattern.language(), "go");
        assert_eq!(store.list_all_patterns().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn register_pattern_rejects_a_language_ast_grep_does_not_support() {
        let store = test_store().await;
        let result = store
            .register_pattern(
                "No Debug Print".to_string(),
                "DISPLAY left in production code".to_string(),
                None,
                COBOL_RULE.to_string(),
            )
            .await;
        // See pattern_engine::from_rule_rejects_a_language_ast_grep_does_not_support
        // for why this asserts on `{err:?}` (RegisterPatternError's
        // derived Debug, which renders the wrapped anyhow::Error's own
        // chain) rather than norma's own "unsupported language" text: the
        // failure happens inside YAML deserialization, before that check
        // ever runs.
        let err = result.expect_err("a Cobol pattern must be rejected, not stored");
        let chain = format!("{err:?}");
        assert!(
            chain.contains("Cobol"),
            "expected the error chain to name the rejected language, got: {chain}"
        );
        // Nothing should have been written -- in particular no row tagged
        // with a placeholder language that no lookup could ever reach.
        assert!(store.list_all_patterns().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn get_patterns_for_language_only_returns_matching_language() {
        let store = test_store().await;
        store
            .register_pattern(
                "No Debug Print".to_string(),
                "d".to_string(),
                None,
                RUST_RULE.to_string(),
            )
            .await
            .unwrap();
        assert_eq!(
            store.get_patterns_for_language("rust").await.unwrap().len(),
            1
        );
        assert_eq!(
            store
                .get_patterns_for_language("python")
                .await
                .unwrap()
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn register_pattern_upserts_on_matching_id() {
        let store = test_store().await;
        store
            .register_pattern(
                "v1".to_string(),
                "d".to_string(),
                None,
                RUST_RULE.to_string(),
            )
            .await
            .unwrap();
        store
            .register_pattern(
                "v2".to_string(),
                "d".to_string(),
                None,
                RUST_RULE.to_string(),
            )
            .await
            .unwrap();
        let all = store.list_all_patterns().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "v2");
    }

    #[tokio::test]
    async fn seed_defaults_loads_all_twenty_default_patterns_once() {
        let store = test_store().await;
        store.seed_defaults().await.unwrap();
        assert_eq!(store.list_all_patterns().await.unwrap().len(), 20);
        // Calling it again must not duplicate or error.
        store.seed_defaults().await.unwrap();
        assert_eq!(store.list_all_patterns().await.unwrap().len(), 20);
    }

    #[tokio::test]
    async fn seed_defaults_adds_missing_patterns_without_touching_an_existing_one() {
        let store = test_store().await;
        // Simulate a pre-existing (e.g. MVP-era, or user-customized) row
        // under one of the shipped default patterns' ids.
        store
            .register_pattern(
                "My Custom Name".to_string(),
                "custom description".to_string(),
                Some("custom-category".to_string()),
                RUST_RULE.to_string(), // id: no-debug-print-rust
            )
            .await
            .unwrap();

        store.seed_defaults().await.unwrap();

        let all = store.list_all_patterns().await.unwrap();
        assert_eq!(
            all.len(),
            20,
            "the other 19 defaults must still be added on top of the pre-existing row"
        );
        let customized = all
            .iter()
            .find(|p| p.id() == "no-debug-print-rust")
            .expect("the pre-existing row must still be there");
        assert_eq!(
            customized.name, "My Custom Name",
            "seed_defaults must never overwrite a row that already exists"
        );
    }
}
