use crate::models::{Pattern, Severity};
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
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

    /// Bulk-imports every RuleConfig document in `yaml` (a `---`-separated
    /// multi-document YAML string -- e.g. one file from a cloned
    /// `sgconfig.yaml` rule directory, or a chunk of ast-grep's own rule
    /// catalog; TF-894, see
    /// .scratch/ast-grep-feature-parity/issues/04-bulk-import-existing-rules.md).
    /// `category`, if given, is applied to every pattern imported by this
    /// one call -- ast-grep's own rule YAML has no per-document category.
    ///
    /// Mirrors `register_pattern`'s derivation and upsert semantics for
    /// each document: `name` <- the document's own raw `id:` field
    /// (unhumanized), `description` <- its raw `message:` field, and
    /// importing an `id` that already exists -- including twice within one
    /// `yaml` string -- overwrites, exactly like `register_pattern` does
    /// (`ON CONFLICT(id) DO UPDATE`, see `save_pattern`); there is no
    /// separate "never overwrite" mode. `imported` reflects that: if two
    /// documents share an `id`, only the *last* one's `Pattern` appears in
    /// it (deduplicated in place, see below), since that's the one row
    /// that's actually left in the store afterward -- `imported.len()`
    /// always matches the number of distinct rows this call actually wrote.
    ///
    /// A document that fails to register -- an unsupported `language:`, a
    /// missing/empty `id`, or any other reason `register_pattern` would
    /// reject it for -- is skipped, with the reason recorded in the
    /// result's `skipped` list, rather than aborting the whole import: one
    /// bad or not-yet-supported document in an otherwise-good batch must
    /// not block every other rule in it. A document that instead has
    /// neither an `id:` nor a `rule:` key at all (see `split_yaml_documents`)
    /// is silently excluded from both lists -- it doesn't look like a rule
    /// that was *attempted*, so counting it as "skipped" would misreport a
    /// non-rule YAML document (an `sgconfig.yaml`, a CI workflow file, ...)
    /// as a rejected one.
    ///
    /// Two things abort the whole call early, returned as `Err`, rather
    /// than being folded into `skipped`: `yaml` failing to even split into
    /// documents (a genuine YAML syntax error -- there's no per-document
    /// reason to attach when documents were never successfully separated
    /// in the first place), and a genuine storage failure (SQLite I/O).
    /// The latter is logged (`tracing::error!`, with how far the batch got)
    /// before returning, since anything already registered before the
    /// failure is already durably committed -- `import_rules`'s upsert
    /// semantics make a full re-run safe, but only if that's visible
    /// somewhere, not silently implied.
    pub async fn import_rules(
        &self,
        yaml: &str,
        category: Option<String>,
    ) -> std::result::Result<ImportResult, RegisterPatternError> {
        let documents =
            split_yaml_documents(yaml).map_err(|e| RegisterPatternError::InvalidRule(e.into()))?;

        let mut imported: Vec<Pattern> = Vec::with_capacity(documents.len());
        let mut skipped = Vec::new();
        for doc in documents {
            let name = doc.id.clone().unwrap_or_default();
            let description = doc.message.unwrap_or_default();
            match self
                .register_pattern(name, description, category.clone(), doc.text)
                .await
            {
                Ok(pattern) => {
                    // Upsert-on-repeated-id within this one call: replace
                    // the earlier entry in place rather than appending a
                    // second one, so `imported` never claims two rows for
                    // an `id` this call only ever left one row under.
                    match imported.iter_mut().find(|p| p.id() == pattern.id()) {
                        Some(existing) => *existing = pattern,
                        None => imported.push(pattern),
                    }
                }
                Err(RegisterPatternError::InvalidRule(err)) => {
                    // The flattened error chain (not `{err:?}`'s multi-line
                    // Debug form) -- `render_import_result`/`norma import`
                    // render one line per skip, and a raw `{err:?}` (e.g.
                    // "Fail to parse yaml as RuleConfig\n\nCaused by:\n
                    // ...") would break that. Still every level of detail
                    // (e.g. "language: Cobol is not supported!"), just
                    // joined onto one line instead of kept as a Debug tree
                    // -- see `mcp_server::to_tool_error`'s doc comment for
                    // why the useful detail lives one level down the chain.
                    let reason = err
                        .chain()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(": ");
                    skipped.push(SkippedRule { id: doc.id, reason });
                }
                Err(err @ RegisterPatternError::Storage(_)) => {
                    tracing::error!(
                        imported = imported.len(),
                        skipped = skipped.len(),
                        error = ?err,
                        "import_rules aborted by a storage failure -- documents already \
                         imported before this point remain committed; re-running the same \
                         `yaml` is safe (import_rules upserts)"
                    );
                    return Err(err);
                }
            }
        }
        Ok(ImportResult { imported, skipped })
    }
}

/// One YAML document out of `import_rules`' multi-document input, split via
/// `split_yaml_documents`. `id`/`message` are read straight from the
/// document's own deserialized `serde_yaml::Value` -- not from a validated
/// `RuleConfig` -- so they're the "raw" values `import_rules`'s doc comment
/// promises for `name`/`description`, and are still available to name a
/// document in a `SkippedRule` even when it goes on to fail registration
/// (e.g. an unsupported `language:`).
#[derive(Debug)]
struct YamlDocument {
    /// Re-serialized (via a `serde_yaml::Value` round-trip), not a
    /// byte-identical copy of the original document -- semantically
    /// equivalent, but formatting/key order/comments are not preserved the
    /// way `register_pattern`'s `rule` is stored verbatim.
    text: String,
    id: Option<String>,
    message: Option<String>,
}

/// Splits a multi-document YAML string into one [`YamlDocument`] per
/// document, re-serializing each back to its own single-document YAML text
/// (via a `serde_yaml::Value` round-trip) so it can be handed straight to
/// `register_pattern`, exactly like a single already-adopted rule would be.
/// Reuses `serde_yaml::Deserializer`'s own document-boundary detection
/// (the same one `ast_grep_config::from_yaml_string` uses internally)
/// rather than re-implementing `---`-splitting by hand, which would risk
/// disagreeing with it on an edge case (e.g. `---` appearing inside an
/// indented block scalar).
///
/// Two kinds of document are dropped here rather than turned into a
/// `YamlDocument` (and, downstream, a `SkippedRule`):
///
/// - An *empty* document (`Value::Null`) -- valid YAML, but carries no
///   import intent. This is what a stray `---\n---\n` produces: e.g. two
///   already-`---`-prefixed rule files concatenated with a synthetic
///   `---` separator between them (see `cli::import_dir`, TF-894 PR #8
///   review) doubles up the marker into exactly this. Without this, such
///   an artifact would reach `register_pattern`, fail (no `id`, no
///   `language`), and show up as an unidentifiable `SkippedRule { id:
///   None, .. }` -- indistinguishable from a real, broken rule, even
///   though no real document was ever malformed.
/// - A document with *neither* a top-level `id:` nor a `rule:` key. Every
///   real ast-grep RuleConfig has at least one of the two; a document
///   with neither is overwhelmingly more likely to be some other kind of
///   YAML entirely that a directory scan swept up alongside real rules
///   (an `sgconfig.yaml`, a CI workflow file, a snapshot fixture, ...)
///   than an attempted-but-broken rule. Reporting it as "skipped" would
///   misattribute a stranger file's presence as a rejected rule.
fn split_yaml_documents(yaml: &str) -> std::result::Result<Vec<YamlDocument>, serde_yaml::Error> {
    let mut documents = Vec::new();
    for doc in serde_yaml::Deserializer::from_str(yaml) {
        let value = serde_yaml::Value::deserialize(doc)?;
        if value.is_null() {
            continue;
        }
        let has_id = value.get("id").is_some();
        let has_rule = value.get("rule").is_some();
        if !has_id && !has_rule {
            continue;
        }
        let id = value
            .get("id")
            .and_then(serde_yaml::Value::as_str)
            .map(str::to_string);
        let message = value
            .get("message")
            .and_then(serde_yaml::Value::as_str)
            .map(str::to_string);
        let text = serde_yaml::to_string(&value)?;
        documents.push(YamlDocument { text, id, message });
    }
    Ok(documents)
}

/// One document's outcome inside `PatternStore::import_rules`'s bulk
/// import that could not be registered -- `id` is best-effort (see
/// `YamlDocument`'s doc comment: read from the raw document, not a
/// validated `RuleConfig`), so it may be `None` for a document with no
/// `id:` field at all, or whose `id:` value isn't a plain YAML string
/// (e.g. `id: 42`).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(test, derive(Deserialize))]
pub struct SkippedRule {
    pub id: Option<String>,
    pub reason: String,
}

/// `PatternStore::import_rules`'s result: every document that was
/// successfully registered, and every one that was skipped and why. Never
/// partially silent -- a bulk import of N rule-shaped documents where M
/// didn't register still accounts for all N, not just the ones that
/// succeeded (a document that wasn't rule-shaped at all -- see
/// `split_yaml_documents` -- was never counted as an N to begin with).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(test, derive(Deserialize))]
pub struct ImportResult {
    pub imported: Vec<Pattern>,
    pub skipped: Vec<SkippedRule>,
}

impl ImportResult {
    /// Whether every document this call attempted to register actually
    /// registered -- i.e. `skipped` is empty. A convenience for callers
    /// that only care about pass/fail, mirroring `ValidationResult.passed`;
    /// `norma import`'s exit status (`cli::import_should_fail`) is built on
    /// this.
    pub fn fully_succeeded(&self) -> bool {
        self.skipped.is_empty()
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

    // A rule that parses as YAML but has no top-level `id` (it defaults to
    // `""`), so `check_registerable` must reject it -- see
    // `import_rules_rejects_a_document_with_no_id_and_reports_none_as_its_skip_id`.
    const RULE_WITHOUT_ID: &str = r#"
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

    #[tokio::test]
    async fn import_rules_registers_every_document_deriving_name_and_description_from_id_and_message(
    ) {
        let store = test_store().await;
        let yaml = format!("{RUST_RULE}---\n{GO_RULE}");
        let result = store.import_rules(&yaml, None).await.unwrap();

        assert_eq!(result.imported.len(), 2);
        assert!(result.skipped.is_empty());

        let rust = result
            .imported
            .iter()
            .find(|p| p.id() == "no-debug-print-rust")
            .unwrap();
        // TF-894: name <- the rule's own raw `id`, description <- its
        // `message`, neither humanized nor otherwise reshaped.
        assert_eq!(rust.name, "no-debug-print-rust");
        assert_eq!(rust.description, "Avoid println! in production code");

        let go = result
            .imported
            .iter()
            .find(|p| p.id() == "no-debug-print-go")
            .unwrap();
        assert_eq!(go.name, "no-debug-print-go");
        assert_eq!(go.description, "Avoid fmt.Println in production code");
    }

    #[tokio::test]
    async fn import_rules_applies_the_given_category_to_every_document() {
        let store = test_store().await;
        let yaml = format!("{RUST_RULE}---\n{GO_RULE}");
        let result = store
            .import_rules(&yaml, Some("bulk-imported".to_string()))
            .await
            .unwrap();

        assert!(result
            .imported
            .iter()
            .all(|p| p.category.as_deref() == Some("bulk-imported")));
    }

    #[tokio::test]
    async fn import_rules_skips_an_unsupported_language_with_a_reason_but_imports_the_rest() {
        let store = test_store().await;
        let yaml = format!("{RUST_RULE}---\n{COBOL_RULE}");
        let result = store.import_rules(&yaml, None).await.unwrap();

        assert_eq!(result.imported.len(), 1);
        assert_eq!(result.imported[0].id(), "no-debug-print-rust");

        assert_eq!(result.skipped.len(), 1);
        let skip = &result.skipped[0];
        assert_eq!(skip.id.as_deref(), Some("no-debug-print-cobol"));
        assert!(
            skip.reason.contains("Cobol"),
            "expected the skip reason to name the rejected language, got: {}",
            skip.reason
        );

        // The one rejected document must never have been written.
        assert_eq!(store.list_all_patterns().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn import_rules_upserts_on_a_repeated_id_like_register_pattern_does() {
        let store = test_store().await;
        store.import_rules(RUST_RULE, None).await.unwrap();
        // Re-importing the same id (e.g. a re-run against an updated
        // external rule file) must overwrite, not duplicate.
        let result = store.import_rules(RUST_RULE, None).await.unwrap();

        assert_eq!(result.imported.len(), 1);
        assert_eq!(store.list_all_patterns().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn import_rules_rejects_a_document_with_no_id_and_reports_none_as_its_skip_id() {
        let store = test_store().await;
        let result = store.import_rules(RULE_WITHOUT_ID, None).await.unwrap();

        assert!(result.imported.is_empty());
        assert_eq!(result.skipped.len(), 1);
        assert_eq!(result.skipped[0].id, None);
    }

    #[tokio::test]
    async fn import_rules_ignores_an_empty_document_produced_by_a_stray_double_separator() {
        // Exactly what `cli::import_dir` used to produce (TF-894 PR #8
        // review) when two already-`---`-prefixed files were concatenated
        // with a synthetic `---` separator: a `---\n---\n` sequence,
        // which parses as a valid but empty (`Value::Null`) document.
        let store = test_store().await;
        let yaml = format!("{RUST_RULE}---\n---\n{GO_RULE}");
        let result = store.import_rules(&yaml, None).await.unwrap();

        assert_eq!(
            result.imported.len(),
            2,
            "both real rules must still import"
        );
        assert!(
            result.skipped.is_empty(),
            "the empty document must not show up as an unidentifiable skip, got: {:?}",
            result.skipped
        );
    }

    // Not a rule document at all: no `id:`, no `rule:` -- e.g. the kind of
    // thing `sgconfig.yaml` itself looks like, which a directory scan of a
    // real rule repo will often sweep up alongside genuine rule files.
    const NOT_A_RULE_DOCUMENT: &str = r#"
ruleDirs:
  - rules
"#;

    #[tokio::test]
    async fn import_rules_silently_drops_a_document_that_looks_like_no_rule_at_all() {
        let store = test_store().await;
        let yaml = format!("{RUST_RULE}---\n{NOT_A_RULE_DOCUMENT}");
        let result = store.import_rules(&yaml, None).await.unwrap();

        assert_eq!(result.imported.len(), 1);
        assert!(
            result.skipped.is_empty(),
            "a non-rule document must not be reported as a rejected rule, got: {:?}",
            result.skipped
        );
    }

    #[tokio::test]
    async fn import_rules_deduplicates_a_repeated_id_within_one_call() {
        // Two documents sharing an id in the *same* `yaml` string (as
        // opposed to `import_rules_upserts_on_a_repeated_id...` above,
        // which re-imports across two separate calls) must still only
        // count once in `imported` -- only one row is ever actually left
        // in the store under that id.
        let updated_rust_rule = r#"
id: no-debug-print-rust
message: Avoid println! in production code (v2)
severity: error
language: Rust
rule:
  pattern: println!($$$ARGS)
"#;
        let store = test_store().await;
        let yaml = format!("{RUST_RULE}---\n{updated_rust_rule}");
        let result = store.import_rules(&yaml, None).await.unwrap();

        assert_eq!(
            result.imported.len(),
            1,
            "a repeated id within one call must not be double-counted"
        );
        assert_eq!(
            result.imported[0].description,
            "Avoid println! in production code (v2)"
        );
        assert_eq!(store.list_all_patterns().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn import_rules_rejects_yaml_that_does_not_even_split_into_documents_as_invalid_rule() {
        let store = test_store().await;
        let result = store
            .import_rules("not: valid: yaml: at: all: -", None)
            .await;
        assert!(
            matches!(result, Err(RegisterPatternError::InvalidRule(_))),
            "a YAML string that can't even be split into documents must be InvalidRule, \
             not treated as a storage problem"
        );
    }

    #[test]
    fn split_yaml_documents_surfaces_a_genuine_yaml_syntax_error() {
        let err = split_yaml_documents("not: valid: yaml: at: all: -").unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn split_yaml_documents_does_not_mistake_a_literal_dashes_line_inside_a_block_scalar_for_a_document_boundary(
    ) {
        // The exact edge case this fn's own doc comment names as the
        // reason to reuse `serde_yaml::Deserializer` rather than
        // hand-rolled `---`-splitting: a `---` line that's part of an
        // indented block scalar's *content*, not a real document marker.
        let rule_with_dashes_in_a_block_scalar = r#"
id: has-a-dashes-line-in-its-fix
message: contains a literal --- inside an indented block scalar
severity: warning
language: Rust
rule:
  pattern: println!($$$ARGS)
fix: |
  // ---
  // not a document separator
"#;
        let docs = split_yaml_documents(rule_with_dashes_in_a_block_scalar).unwrap();
        assert_eq!(
            docs.len(),
            1,
            "the indented --- must stay inside the one document"
        );
        assert_eq!(docs[0].id.as_deref(), Some("has-a-dashes-line-in-its-fix"));
    }

    // Storage-failure test relies on Unix permission semantics (a
    // directory with its write bit revoked can't have a new file created
    // in it -- SQLite's rollback-journal mode needs exactly that on every
    // write, even for an already-open connection, unlike chmod'ing the
    // database file itself, which an already-open file descriptor ignores).
    #[cfg(unix)]
    #[tokio::test]
    async fn import_rules_logs_and_aborts_on_a_genuine_storage_failure() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("norma-test.db");
        let store = PatternStore::new(&db_path).await.unwrap();

        let mut perms = std::fs::metadata(dir.path()).unwrap().permissions();
        perms.set_mode(0o555);
        std::fs::set_permissions(dir.path(), perms).unwrap();

        let result = store.import_rules(RUST_RULE, None).await;

        // Restore write access before the tempdir's (never-called, since
        // `dir` is forgotten below like every other test store) drop path
        // could otherwise fail to clean up -- and so the assertion above
        // isn't masked by a permission-related panic in test teardown.
        let mut perms = std::fs::metadata(dir.path()).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(dir.path(), perms).unwrap();

        assert!(
            matches!(result, Err(RegisterPatternError::Storage(_))),
            "a real SQLite I/O failure must surface as Storage, not be swallowed: {result:?}"
        );
        std::mem::forget(dir);
    }

    #[test]
    fn import_result_fully_succeeded_is_false_only_when_something_was_skipped() {
        let ok = ImportResult {
            imported: vec![],
            skipped: vec![],
        };
        assert!(ok.fully_succeeded());

        let partial = ImportResult {
            imported: vec![],
            skipped: vec![SkippedRule {
                id: None,
                reason: "d".to_string(),
            }],
        };
        assert!(!partial.fully_succeeded());
    }
}
