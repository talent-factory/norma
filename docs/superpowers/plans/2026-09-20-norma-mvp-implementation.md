# norma MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the non-compiling `mcpkit` + regex scaffold with a working norma MVP: an `rmcp`-based MCP tool server and a `clap`-based CLI, sharing one `ast-grep-core`-based validation core and a SQLite pattern registry, seeded with the four MVP default patterns.

**Architecture:** One binary crate (`norma`) with a thin `[lib]` target so `tests/` can import it. `pattern_engine` (pure, sync) parses ast-grep RuleConfig YAML and matches it against source; `pattern_store` (async, SQLite via `sqlx`) persists `Pattern` rows and derives `language`/`severity` from the YAML at register time; `mcp_server` (`rmcp`, stdio) and `cli` (`clap`) are both thin callers of the same two modules — see ADR 0001.

**Tech Stack:** Rust, `rmcp` 3.x (official MCP SDK), `ast-grep-core`/`ast-grep-config`/`ast-grep-language` 0.45.3 (pinned exactly), `sqlx` 0.9 (SQLite), `clap` 4, `tokio`.

**Spec:** `.scratch/norma-architecture/map.md` and its resolved tickets (01–05), plus `docs/adr/0001-single-binary-shared-validation-core.md` and `docs/adr/0002-pattern-single-language-full-rule-config.md`. This plan implements those decisions; it does not re-open them.

## Global Constraints

- `rmcp` version `"3"`, features `["server", "transport-io", "schemars"]` — official Rust MCP SDK (ticket 01).
- `ast-grep-core`, `ast-grep-config`, `ast-grep-language` version `"=0.45.3"` each, exact pin — Rust API is documented "not stable yet"; bump deliberately, never automatically (ticket 02, map Notes).
- `sqlx` version `"0.9"`, features `["sqlite", "runtime-tokio", "chrono"]` (bumped from the scaffold's stale `"0.7"` pin — verified against the current crate).
- `clap` version `"4"`, features `["derive"]`.
- Single binary, one `Cargo.toml`, no Cargo workspace (ADR 0001).
- `Pattern` rows are single-language; `rule` stores the *complete* ast-grep RuleConfig YAML (`id`/`message`/`severity`/`language`/`rule`/optional `fix`); `severity` and `language` columns are always derived from parsing that YAML at register time, never accepted as separate inputs (ADR 0002).
- Code comments: English, written for a reader who doesn't yet know the toolset (norma becomes an FFHS teaching artifact — map Notes).
- Every fact below (exact crate versions, macro syntax, function signatures) was verified against the real, locally-downloaded crate source or a compiling scratch probe during planning — not taken from documentation alone. Where this plan's code differs from what ticket 01/02/04's probes tried, this plan's version is the one that was actually compiled and run.

---

### Task 1: Dependencies and library target

**Files:**
- Modify: `Cargo.toml`
- Create: `src/lib.rs` (replacing its current placeholder content)

**Interfaces:**
- Produces: a `norma` library target (so later tasks' `tests/*.rs` files can `use norma::...`) alongside the existing `norma` binary target.

This task has no test cycle of its own (it's package configuration) — its deliverable is verified in Task 2, once `src/lib.rs` has a real module to declare.

- [ ] **Step 1: Rewrite `Cargo.toml`**

```toml
[package]
name = "norma"
version = "0.1.0"
edition = "2021"
authors = ["Talent Factory GmbH <daniel@talentfactory.ch>"]
description = "Developer-grade code pattern enforcement everywhere you code"
repository = "https://github.com/talent-factory/norma"
license = "MIT OR Apache-2.0"

[lib]
name = "norma"
path = "src/lib.rs"

[[bin]]
name = "norma"
path = "src/main.rs"

[dependencies]
# MCP server framework: official Rust SDK (see docs/adr/0001.md and ticket 01).
rmcp = { version = "3", features = ["server", "transport-io", "schemars"] }

# AST-based pattern matching (see docs/adr/0002.md and tickets 02/04).
# Pinned *exactly*: ast-grep's own docs say the Rust API is "not stable
# yet". Bump these three together, deliberately, after testing -- never
# let a caret range silently pull in a breaking update.
ast-grep-core = "=0.45.3"
ast-grep-config = "=0.45.3"
ast-grep-language = "=0.45.3"

# Async runtime.
tokio = { version = "1", features = ["full"] }

# CLI (see docs/adr/0001.md: one binary, clap subcommands).
clap = { version = "4", features = ["derive"] }

# Data serialization.
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Database.
sqlx = { version = "0.9", features = ["sqlite", "runtime-tokio", "chrono"] }

# Error handling & logging.
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"

# Time.
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tempfile = "3"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

- [ ] **Step 2: Replace `src/lib.rs` with an empty module root for now**

```rust
//! norma: a design-pattern and code-quality validator built on ast-grep,
//! exposed both as an MCP tool server (`norma serve`) and a CLI
//! (`norma validate`, `norma list-patterns`). See docs/adr/0001.md and
//! docs/adr/0002.md for the architecture decisions this crate follows.
//!
//! Modules are added here one at a time by the tasks in
//! docs/superpowers/plans/2026-09-20-norma-mvp-implementation.md; each
//! addition should keep `cargo test --lib` green.
```

- [ ] **Step 3: Confirm the package still parses**

Run: `cargo metadata --no-deps --format-version 1 > /dev/null`
Expected: exits 0 (confirms `Cargo.toml` is well-formed; the crate won't fully build until later tasks add real modules and fix `src/main.rs`, which is expected at this point).

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src/lib.rs
git commit -m "norma: switch dependencies to rmcp + ast-grep, add lib target"
```

---

### Task 2: `models.rs` — data types

**Files:**
- Create: `src/models.rs` (replacing its current content)
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: `Severity` (`Off`/`Hint`/`Info`/`Warning`/`Error`, with `parse(&str) -> Option<Severity>` and `as_str(&self) -> &'static str`), `Pattern`, `PatternViolation`, `CodeLocation`, `ValidationResult` — all used by every later task.

- [ ] **Step 1: Write the failing test inside the new module**

Create `src/models.rs` with the types and this test module at the bottom:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Severity level for a pattern violation.
///
/// Mirrors `ast_grep_config::rule_config::Severity` (see docs/adr/0002.md
/// and `pattern_engine::parse_rule`): norma derives this from the parsed
/// rule YAML at register time rather than accepting it as a separate
/// input, so a `Pattern`'s severity can never drift from the YAML that
/// actually produced it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Off,
    Hint,
    Info,
    Warning,
    Error,
}

impl Severity {
    /// Parses the lowercase string form used in the `patterns.severity`
    /// SQLite column (see `as_str`). Returns `None` for anything else.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "off" => Some(Severity::Off),
            "hint" => Some(Severity::Hint),
            "info" => Some(Severity::Info),
            "warning" => Some(Severity::Warning),
            "error" => Some(Severity::Error),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Off => "off",
            Severity::Hint => "hint",
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

/// A registered code pattern.
///
/// Per docs/adr/0002.md, one row is always single-language: an idea that
/// should hold across several languages (e.g. "no debug prints") becomes
/// several `Pattern` rows, one per language, related only by a shared
/// `name` -- there is no data-level link between them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pattern {
    /// The ast-grep RuleConfig `id` from the YAML in `rule`, e.g. `"no-debug-print-rust"`.
    pub id: String,
    /// norma's catalog name. Repeats across the language variants of one idea.
    pub name: String,
    /// norma's own, longer explanation (ast-grep's own `message` field
    /// inside `rule` is documented as "should be single line and concise").
    pub description: String,
    /// Free-text grouping, e.g. `"code-quality"`, `"creational"`. Not a fixed enum.
    pub category: Option<String>,
    /// norma's canonical language key: `"java"` | `"python"` | `"rust"` | `"typescript"`.
    pub language: String,
    /// Derived from the parsed `rule` YAML when the pattern was registered.
    pub severity: Severity,
    /// The full ast-grep RuleConfig YAML document (id/message/severity/language/rule/fix).
    pub rule: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single match of a `Pattern` against a piece of code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatternViolation {
    pub pattern_id: String,
    pub pattern_name: String,
    pub severity: Severity,
    pub location: CodeLocation,
    pub matched_text: String,
    pub message: String,
}

/// A position ast-grep matched at. `line` and `column` are zero-based, as
/// ast-grep-core's own `Position` type documents them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeLocation {
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
}

/// The result of validating one piece of code against a set of patterns.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationResult {
    pub violations: Vec<PatternViolation>,
    pub passed: bool,
    /// 1.0 = no violations relative to the number of patterns checked, 0.0 = worst case.
    pub score: f64,
    pub duration_ms: u128,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_round_trips_through_its_string_form() {
        for severity in [
            Severity::Off,
            Severity::Hint,
            Severity::Info,
            Severity::Warning,
            Severity::Error,
        ] {
            assert_eq!(Severity::parse(severity.as_str()), Some(severity));
        }
    }

    #[test]
    fn severity_parse_rejects_unknown_strings() {
        assert_eq!(Severity::parse("critical"), None);
        assert_eq!(Severity::parse(""), None);
    }
}
```

- [ ] **Step 2: Add the module to `src/lib.rs`**

```rust
pub mod models;
```

(Append this line; keep the doc comment block from Task 1 above it.)

- [ ] **Step 3: Run the tests and confirm they pass**

Run: `cargo test --lib -- models:: --nocapture`
Expected: `test models::tests::severity_round_trips_through_its_string_form ... ok` and `test models::tests::severity_parse_rejects_unknown_strings ... ok`, `2 passed; 0 failed`.

(If they fail first because the module didn't exist yet, that's the expected red step; this task writes the implementation directly since there's no separate empty-stub step worth its own commit for a data-only module. If you're following strict red-green-refactor, comment out the `impl Severity` block, confirm the test fails to compile, then restore it before Step 3.)

- [ ] **Step 4: Commit**

```bash
git add src/models.rs src/lib.rs
git commit -m "norma: add Pattern/Severity/ValidationResult data model (ADR 0002)"
```

---

### Task 3: `pattern_engine.rs` — ast-grep matching core

**Files:**
- Create: `src/pattern_engine.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `models::{Pattern, Severity, PatternViolation, CodeLocation, ValidationResult}` (Task 2).
- Produces: `pattern_engine::parse_rule(&str) -> anyhow::Result<RuleConfig<SupportLang>>`, `pattern_engine::language_key(SupportLang) -> &'static str`, `pattern_engine::validate(&str, &str, &[Pattern]) -> ValidationResult`. Task 4 (`pattern_store`) calls `parse_rule` and `language_key`; Task 6 (`mcp_server`) and Task 7 (`cli`) call `validate`.

- [ ] **Step 1: Write the failing tests**

Create `src/pattern_engine.rs`:

```rust
use crate::models::{CodeLocation, Pattern, PatternViolation, Severity, ValidationResult};
use ast_grep_config::{GlobalRules, RuleConfig, from_yaml_string};
use ast_grep_language::{LanguageExt, SupportLang};
use std::time::Instant;

/// Parses a full ast-grep RuleConfig YAML document (see docs/adr/0002.md)
/// and returns the compiled rule, or an error if the YAML is malformed or
/// the rule has no matchable AST kinds. Used both to run a pattern
/// (`validate`, below) and to validate one before it's persisted
/// (`PatternStore::register_pattern` in `pattern_store.rs`).
pub fn parse_rule(rule_yaml: &str) -> anyhow::Result<RuleConfig<SupportLang>> {
    let globals = GlobalRules::default();
    let mut configs = from_yaml_string::<SupportLang>(rule_yaml, &globals)?;
    if configs.is_empty() {
        anyhow::bail!("rule YAML did not contain a RuleConfig document");
    }
    Ok(configs.remove(0))
}

/// norma's own canonical language key for a `SupportLang`, matching the
/// strings used in the `patterns.language` SQLite column and the MCP tool
/// parameters (`"java"`, `"python"`, `"rust"`, `"typescript"`).
/// ast-grep-language's own alias lists are inconsistent for this purpose
/// (Rust's first alias is `"rs"`, TypeScript's is `"ts"`), so norma keeps
/// its own explicit mapping for the four MVP languages.
pub fn language_key(lang: SupportLang) -> &'static str {
    match lang {
        SupportLang::Java => "java",
        SupportLang::Python => "python",
        SupportLang::Rust => "rust",
        SupportLang::TypeScript => "typescript",
        _ => "unsupported",
    }
}

/// Runs one already-parsed rule against `source` and returns every match
/// as a `PatternViolation`, tagged with `pattern`'s catalog metadata.
fn find_violations(
    pattern: &Pattern,
    config: &RuleConfig<SupportLang>,
    source: &str,
) -> Vec<PatternViolation> {
    let grep = config.language.ast_grep(source);
    grep.root()
        .find_all(&config.matcher)
        .map(|node_match| {
            let pos = node_match.start_pos();
            PatternViolation {
                pattern_id: pattern.id.clone(),
                pattern_name: pattern.name.clone(),
                severity: pattern.severity,
                location: CodeLocation {
                    file: None,
                    line: pos.line(),
                    column: pos.column(&node_match),
                },
                matched_text: node_match.text().to_string(),
                message: pattern.description.clone(),
            }
        })
        .collect()
}

/// Runs every enabled pattern whose `language` matches `language` against
/// `source`, and aggregates the result. Shared by the MCP
/// `validate_pattern_compliance` tool (`mcp_server.rs`) and the `norma
/// validate` CLI subcommand (`cli.rs`) -- see docs/adr/0001.md: one core,
/// no separate sync/async implementation. A pattern whose stored `rule`
/// no longer parses is skipped with a warning rather than failing the
/// whole run.
pub fn validate(source: &str, language: &str, patterns: &[Pattern]) -> ValidationResult {
    let start = Instant::now();
    let mut violations = Vec::new();
    let mut checked = 0usize;
    for pattern in patterns.iter().filter(|p| p.enabled && p.language == language) {
        checked += 1;
        match parse_rule(&pattern.rule) {
            Ok(config) => violations.extend(find_violations(pattern, &config, source)),
            Err(err) => {
                tracing::warn!(pattern_id = %pattern.id, error = %err, "skipping pattern with unparsable rule");
            }
        }
    }
    let passed = violations.is_empty();
    let score = if checked == 0 {
        1.0
    } else {
        (1.0 - violations.len() as f64 / checked as f64).max(0.0)
    };
    ValidationResult {
        violations,
        passed,
        score,
        duration_ms: start.elapsed().as_millis(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn test_pattern(id: &str, language: &str, rule: &str) -> Pattern {
        let now = Utc::now();
        Pattern {
            id: id.to_string(),
            name: "Test Pattern".to_string(),
            description: "test".to_string(),
            category: None,
            language: language.to_string(),
            severity: Severity::Warning,
            rule: rule.to_string(),
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    const RUST_NO_DEBUG_PRINT: &str = r#"
id: no-debug-print-rust
message: Avoid println! in production code
severity: warning
language: Rust
rule:
  pattern: println!($$$ARGS)
"#;

    #[test]
    fn validate_finds_a_real_violation() {
        let pattern = test_pattern("no-debug-print-rust", "rust", RUST_NO_DEBUG_PRINT);
        let source = "fn main() { println!(\"debug\"); }";
        let result = validate(source, "rust", &[pattern]);
        assert_eq!(result.violations.len(), 1);
        assert!(!result.passed);
        assert_eq!(result.violations[0].matched_text, "println!(\"debug\")");
    }

    #[test]
    fn validate_reports_no_violations_for_clean_code() {
        let pattern = test_pattern("no-debug-print-rust", "rust", RUST_NO_DEBUG_PRINT);
        let source = "fn main() { tracing::info!(\"structured, fine\"); }";
        let result = validate(source, "rust", &[pattern]);
        assert!(result.violations.is_empty());
        assert!(result.passed);
        assert_eq!(result.score, 1.0);
    }

    #[test]
    fn validate_ignores_patterns_for_other_languages() {
        let pattern = test_pattern("no-debug-print-rust", "rust", RUST_NO_DEBUG_PRINT);
        // Would match if the language filter didn't work -- this source is
        // deliberately Rust syntax passed under a "python" validation request.
        let source = "println!(\"debug\")";
        let result = validate(source, "python", &[pattern]);
        assert!(result.violations.is_empty());
        assert_eq!(result.score, 1.0); // no patterns were checked for "python"
    }

    #[test]
    fn parse_rule_rejects_malformed_yaml() {
        assert!(parse_rule("not: valid: yaml: at: all: -").is_err());
    }
}
```

- [ ] **Step 2: Add the module to `src/lib.rs`**

```rust
pub mod pattern_engine;
```

- [ ] **Step 3: Run the tests and confirm they pass**

Run: `cargo test --lib -- pattern_engine:: --nocapture`
Expected: 4 tests pass (`validate_finds_a_real_violation`, `validate_reports_no_violations_for_clean_code`, `validate_ignores_patterns_for_other_languages`, `parse_rule_rejects_malformed_yaml`).

- [ ] **Step 4: Commit**

```bash
git add src/pattern_engine.rs src/lib.rs
git commit -m "norma: add ast-grep-based pattern_engine::validate"
```

---

### Task 4: `pattern_store.rs` — SQLite registry

**Files:**
- Create: `src/pattern_store.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `models::{Pattern, Severity}` (Task 2), `pattern_engine::{parse_rule, language_key}` (Task 3).
- Produces: `PatternStore::new`, `PatternStore::register_pattern`, `PatternStore::get_patterns_for_language`, `PatternStore::list_all_patterns`. Task 5 adds `PatternStore::seed_defaults` (needs `default_patterns`, created in that task). Tasks 6 and 7 depend on all of the above.

- [ ] **Step 1: Write the failing tests**

Create `src/pattern_store.rs`:

```rust
use crate::models::{Pattern, Severity};
use crate::pattern_engine::{language_key, parse_rule};
use anyhow::{Result, bail};
use ast_grep_config::Severity as AstGrepSeverity;
use chrono::Utc;
use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteRow};
use std::path::Path;

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
        let database_url = format!("sqlite:{}?mode=rwc", db_path.as_ref().display());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
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
    /// `id`, `language` and `severity` are derived from the parsed YAML,
    /// never accepted as separate inputs, so they cannot drift from it.
    pub async fn register_pattern(
        &self,
        name: String,
        description: String,
        category: Option<String>,
        rule_yaml: String,
    ) -> Result<Pattern> {
        let config = parse_rule(&rule_yaml)?;
        if config.id.is_empty() {
            bail!("rule YAML must set a non-empty top-level `id`");
        }
        let now = Utc::now();
        let pattern = Pattern {
            id: config.id.clone(),
            name,
            description,
            category,
            language: language_key(config.language).to_string(),
            severity: severity_from_ast_grep(&config.severity),
            rule: rule_yaml,
            enabled: true,
            created_at: now,
            updated_at: now,
        };
        self.save_pattern(pattern).await
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
        .bind(&pattern.id)
        .bind(&pattern.name)
        .bind(&pattern.description)
        .bind(&pattern.category)
        .bind(&pattern.language)
        .bind(pattern.severity.as_str())
        .bind(&pattern.rule)
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
}

fn row_to_pattern(row: &SqliteRow) -> Result<Pattern> {
    let severity_str: String = row.get("severity");
    let created_at_str: String = row.get("created_at");
    let updated_at_str: String = row.get("updated_at");
    Ok(Pattern {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        category: row.get("category"),
        language: row.get("language"),
        severity: Severity::parse(&severity_str)
            .ok_or_else(|| anyhow::anyhow!("unknown severity in database: {severity_str}"))?,
        rule: row.get("rule"),
        enabled: row.get("enabled"),
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)?.with_timezone(&Utc),
    })
}

fn severity_from_ast_grep(severity: &AstGrepSeverity) -> Severity {
    match severity {
        AstGrepSeverity::Off => Severity::Off,
        AstGrepSeverity::Hint => Severity::Hint,
        AstGrepSeverity::Info => Severity::Info,
        AstGrepSeverity::Warning => Severity::Warning,
        AstGrepSeverity::Error => Severity::Error,
    }
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
        assert_eq!(pattern.id, "no-debug-print-rust");
        assert_eq!(pattern.language, "rust");
        assert_eq!(pattern.severity, Severity::Warning);
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
        assert_eq!(store.get_patterns_for_language("rust").await.unwrap().len(), 1);
        assert_eq!(store.get_patterns_for_language("python").await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn register_pattern_upserts_on_matching_id() {
        let store = test_store().await;
        store
            .register_pattern("v1".to_string(), "d".to_string(), None, RUST_RULE.to_string())
            .await
            .unwrap();
        store
            .register_pattern("v2".to_string(), "d".to_string(), None, RUST_RULE.to_string())
            .await
            .unwrap();
        let all = store.list_all_patterns().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "v2");
    }
}
```

- [ ] **Step 2: Add the module to `src/lib.rs`**

```rust
pub mod pattern_store;
```

- [ ] **Step 3: Run the tests and confirm they pass**

Run: `cargo test --lib -- pattern_store:: --nocapture`
Expected: 4 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/pattern_store.rs src/lib.rs
git commit -m "norma: add SQLite PatternStore with fail-fast registration"
```

---

### Task 5: `default_patterns.rs` — the MVP pattern set

**Files:**
- Create: `src/default_patterns.rs`
- Modify: `src/lib.rs`, `src/pattern_store.rs`

**Interfaces:**
- Consumes: `pattern_engine::parse_rule` (Task 3), `pattern_store::PatternStore` (Task 4).
- Produces: `default_patterns::ALL: &[DefaultPattern]`, `PatternStore::seed_defaults(&self) -> Result<()>`. Tasks 6, 7 and 8 call `seed_defaults` at startup.

- [ ] **Step 1: Write the failing test**

Create `src/default_patterns.rs` with the four rules decided on the "MVP-Pattern-Set-Scope" ticket (`.scratch/norma-architecture/issues/05-mvp-pattern-set-scope.md`), already verified against real `ast-grep-core` there:

```rust
/// norma's default pattern set (see the "MVP-Pattern-Set-Scope" ticket on
/// the wayfinder map, `.scratch/norma-architecture/issues/05-mvp-pattern-set-scope.md`):
/// one pattern per MVP language, all expressing the same idea -- "no
/// debug prints in production code" -- so the same concept is visibly
/// expressed differently per language, per docs/adr/0002.md. Every rule
/// below was verified against real `ast-grep-core` while writing that
/// ticket.
pub struct DefaultPattern {
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub rule: &'static str,
}

pub const ALL: &[DefaultPattern] = &[
    DefaultPattern {
        name: "No Debug Print",
        description: "System.out.println left in production code should go through a proper logger instead.",
        category: "code-quality",
        rule: r#"
id: no-debug-print-java
message: Avoid System.out.println in production code
severity: warning
language: Java
rule:
  pattern: System.out.println($$$ARGS)
"#,
    },
    DefaultPattern {
        name: "No Debug Print",
        description: "print() left in production code should go through a proper logger instead.",
        category: "code-quality",
        rule: r#"
id: no-debug-print-python
message: Avoid print() in production code
severity: warning
language: Python
rule:
  pattern: print($$$ARGS)
"#,
    },
    DefaultPattern {
        name: "No Debug Print",
        description: "println! left in production code should go through the `tracing` crate instead.",
        category: "code-quality",
        rule: r#"
id: no-debug-print-rust
message: Avoid println! in production code
severity: warning
language: Rust
rule:
  pattern: println!($$$ARGS)
"#,
    },
    DefaultPattern {
        name: "No Debug Print",
        description: "console.log left in production code should go through a proper logger instead.",
        category: "code-quality",
        rule: r#"
id: no-debug-print-typescript
message: Avoid console.log in production code
severity: warning
language: TypeScript
rule:
  pattern: console.log($$$ARGS)
"#,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern_engine::parse_rule;

    #[test]
    fn every_default_rule_parses_and_covers_one_mvp_language_each() {
        let mut languages: Vec<&str> = ALL
            .iter()
            .map(|def| {
                let config = parse_rule(def.rule).expect("default rule must parse");
                crate::pattern_engine::language_key(config.language)
            })
            .collect();
        languages.sort();
        assert_eq!(languages, ["java", "python", "rust", "typescript"]);
    }
}
```

- [ ] **Step 2: Add the module to `src/lib.rs`**

```rust
pub mod default_patterns;
```

- [ ] **Step 3: Add `seed_defaults` to `PatternStore`**

In `src/pattern_store.rs`, add this method to the `impl PatternStore` block (after `list_all_patterns`):

```rust
    /// Registers norma's default pattern set the first time the store is
    /// empty. Safe to call on every startup.
    pub async fn seed_defaults(&self) -> Result<()> {
        if !self.list_all_patterns().await?.is_empty() {
            return Ok(());
        }
        for def in crate::default_patterns::ALL {
            self.register_pattern(
                def.name.to_string(),
                def.description.to_string(),
                Some(def.category.to_string()),
                def.rule.to_string(),
            )
            .await?;
        }
        Ok(())
    }
```

- [ ] **Step 4: Add a test for `seed_defaults` in `src/pattern_store.rs`'s test module**

```rust
    #[tokio::test]
    async fn seed_defaults_loads_exactly_the_four_mvp_patterns_once() {
        let store = test_store().await;
        store.seed_defaults().await.unwrap();
        assert_eq!(store.list_all_patterns().await.unwrap().len(), 4);
        // Calling it again must not duplicate or error.
        store.seed_defaults().await.unwrap();
        assert_eq!(store.list_all_patterns().await.unwrap().len(), 4);
    }
```

- [ ] **Step 5: Run the tests and confirm they pass**

Run: `cargo test --lib -- default_patterns:: pattern_store:: --nocapture`
Expected: `default_patterns::tests::every_default_rule_parses_and_covers_one_mvp_language_each` and `pattern_store::tests::seed_defaults_loads_exactly_the_four_mvp_patterns_once` both pass, plus the four pre-existing `pattern_store` tests.

- [ ] **Step 6: Commit**

```bash
git add src/default_patterns.rs src/pattern_store.rs src/lib.rs
git commit -m "norma: seed the four MVP default patterns (no-debug-print, ticket 05)"
```

---

### Task 6: `mcp_server.rs` — the rmcp tool server

**Files:**
- Create: `src/mcp_server.rs`
- Modify: `src/lib.rs`, `Cargo.toml` (add `schemars` as a direct dependency)

**Interfaces:**
- Consumes: `models::ValidationResult` (Task 2), `pattern_engine::validate` (Task 3), `pattern_store::PatternStore` (Tasks 4–5).
- Produces: `NormaServer::new(Arc<PatternStore>) -> NormaServer`, `mcp_server::serve(Arc<PatternStore>) -> anyhow::Result<()>`. Task 8 (`main.rs`) calls `serve`.

`schemars` is used directly here (the `JsonSchema` derive on the tool parameter structs), so it needs to be a direct dependency, not just pulled in transitively through `rmcp`'s `schemars` feature.

- [ ] **Step 1: Add `schemars` to `Cargo.toml`**

Add this line under `[dependencies]`, next to `rmcp`:

```toml
schemars = "1"
```

- [ ] **Step 2: Write the failing tests**

Create `src/mcp_server.rs`:

```rust
use crate::models::ValidationResult;
use crate::pattern_engine;
use crate::pattern_store::PatternStore;
use rmcp::{ErrorData, ServiceExt, handler::server::wrapper::Parameters, tool, tool_router, transport::stdio};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateParams {
    /// Source code to validate.
    pub code: String,
    /// norma's canonical language key: "java" | "python" | "rust" | "typescript".
    pub language: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LanguageParams {
    /// norma's canonical language key: "java" | "python" | "rust" | "typescript".
    pub language: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RegisterPatternParams {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    /// A full ast-grep RuleConfig YAML document (id/message/severity/language/rule).
    pub rule: String,
}

fn to_tool_error(err: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(err.to_string(), None)
}

fn to_json(value: &impl serde::Serialize) -> Result<String, ErrorData> {
    serde_json::to_string(value).map_err(to_tool_error)
}

/// The norma MCP tool server. Wraps a `PatternStore` and exposes the four
/// tools from the wayfinder map: validating code, listing the patterns
/// for a language, registering a new pattern, and listing every pattern.
/// This is the same core the `norma validate` CLI subcommand calls
/// (`cli.rs`), just reached over stdio instead -- see docs/adr/0001.md.
#[derive(Clone)]
pub struct NormaServer {
    store: Arc<PatternStore>,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<NormaServer>,
}

#[tool_router(server_handler)]
impl NormaServer {
    pub fn new(store: Arc<PatternStore>) -> Self {
        Self {
            store,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Validate source code against every enabled pattern registered for its language")]
    pub async fn validate_pattern_compliance(
        &self,
        Parameters(params): Parameters<ValidateParams>,
    ) -> Result<String, ErrorData> {
        let patterns = self
            .store
            .get_patterns_for_language(&params.language)
            .await
            .map_err(to_tool_error)?;
        let result: ValidationResult = pattern_engine::validate(&params.code, &params.language, &patterns);
        to_json(&result)
    }

    #[tool(description = "List the patterns that apply to a given language")]
    pub async fn get_pattern_checklist(
        &self,
        Parameters(params): Parameters<LanguageParams>,
    ) -> Result<String, ErrorData> {
        let patterns = self
            .store
            .get_patterns_for_language(&params.language)
            .await
            .map_err(to_tool_error)?;
        to_json(&patterns)
    }

    #[tool(description = "Register a new pattern from a full ast-grep RuleConfig YAML")]
    pub async fn register_pattern(
        &self,
        Parameters(params): Parameters<RegisterPatternParams>,
    ) -> Result<String, ErrorData> {
        let pattern = self
            .store
            .register_pattern(params.name, params.description, params.category, params.rule)
            .await
            .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
        to_json(&pattern)
    }

    #[tool(description = "List every registered pattern")]
    pub async fn list_patterns(&self) -> Result<String, ErrorData> {
        let patterns = self.store.list_all_patterns().await.map_err(to_tool_error)?;
        to_json(&patterns)
    }
}

/// Runs the norma MCP server on stdio until the client disconnects.
pub async fn serve(store: Arc<PatternStore>) -> anyhow::Result<()> {
    let service = NormaServer::new(store).serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Pattern;

    /// Builds a `NormaServer` over a temporary, seeded store. Tool methods
    /// are plain async functions, so tests call them directly -- no stdio
    /// transport or MCP client needed.
    async fn test_server() -> NormaServer {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("norma-test.db");
        let store = PatternStore::new(&db_path).await.unwrap();
        store.seed_defaults().await.unwrap();
        std::mem::forget(dir);
        NormaServer::new(Arc::new(store))
    }

    #[tokio::test]
    async fn validate_pattern_compliance_reports_a_violation() {
        let server = test_server().await;
        let params = Parameters(ValidateParams {
            code: "fn main() { println!(\"debug\"); }".to_string(),
            language: "rust".to_string(),
        });
        let json = server.validate_pattern_compliance(params).await.unwrap();
        let result: ValidationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.violations.len(), 1);
    }

    #[tokio::test]
    async fn list_patterns_returns_the_seeded_defaults() {
        let server = test_server().await;
        let json = server.list_patterns().await.unwrap();
        let patterns: Vec<Pattern> = serde_json::from_str(&json).unwrap();
        assert_eq!(patterns.len(), 4);
    }

    #[tokio::test]
    async fn register_pattern_rejects_malformed_yaml_as_invalid_params() {
        let server = test_server().await;
        let params = Parameters(RegisterPatternParams {
            name: "Broken".to_string(),
            description: "d".to_string(),
            category: None,
            rule: "not: valid: yaml: at: all: -".to_string(),
        });
        let result = server.register_pattern(params).await;
        assert!(result.is_err());
    }
}
```

- [ ] **Step 3: Add the module to `src/lib.rs`**

```rust
pub mod mcp_server;
```

- [ ] **Step 4: Run the tests and confirm they pass**

Run: `cargo test --lib -- mcp_server:: --nocapture`
Expected: 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/mcp_server.rs src/lib.rs
git commit -m "norma: add rmcp tool server with the 4 MCP tools"
```

---

### Task 7: `cli.rs` — clap subcommands

**Files:**
- Create: `src/cli.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: `Cli` (clap `Parser`), `Command` (clap `Subcommand`: `Serve`, `Validate { file, language, json }`, `ListPatterns`). Task 8 (`main.rs`) parses `Cli` and matches on `Command`.

Argument parsing is declarative and low-risk; this task's test asserts the parsed shape rather than exercising any behavior (behavior lives in `pattern_engine`/`pattern_store`/`mcp_server`, already tested).

- [ ] **Step 1: Write the failing test**

Create `src/cli.rs`:

```rust
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
```

- [ ] **Step 2: Add the module to `src/lib.rs`**

```rust
pub mod cli;
```

- [ ] **Step 3: Run the tests and confirm they pass**

Run: `cargo test --lib -- cli:: --nocapture`
Expected: 2 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/cli.rs src/lib.rs
git commit -m "norma: add clap CLI surface (serve/validate/list-patterns)"
```

---

### Task 8: `main.rs` — wire it together

**Files:**
- Modify: `src/main.rs` (full rewrite)

**Interfaces:**
- Consumes: everything produced by Tasks 2–7 (`norma::cli::{Cli, Command}`, `norma::mcp_server`, `norma::pattern_store::PatternStore`, `norma::pattern_engine`, `norma::models::ValidationResult`).
- Produces: the `norma` binary. This is the first task where `cargo build` (the full binary, not just `--lib`) is expected to succeed.

- [ ] **Step 1: Rewrite `src/main.rs`**

```rust
use clap::Parser;
use norma::cli::{Cli, Command};
use norma::models::ValidationResult;
use norma::pattern_store::PatternStore;
use norma::{mcp_server, pattern_engine};
use std::path::Path;
use std::sync::Arc;
use tracing::info;

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
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                print_human_readable(&file, &result);
            }
            if !result.passed {
                std::process::exit(1);
            }
        }
        Command::ListPatterns => {
            for p in store.list_all_patterns().await? {
                println!("{:<28} {:<10} [{}] {}", p.id, p.language, p.severity.as_str(), p.name);
            }
        }
    }
    Ok(())
}

/// Prints a `ValidationResult` as `file:line:column: [severity] name -- text`
/// lines, one per violation, plus a one-line summary -- the format
/// `norma validate` uses without `--json` (see the CLI/pre-commit ticket).
fn print_human_readable(file: &Path, result: &ValidationResult) {
    if result.violations.is_empty() {
        println!("{}: no violations ({} ms)", file.display(), result.duration_ms);
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
```

- [ ] **Step 2: Build the whole crate and confirm it succeeds**

Run: `cargo build`
Expected: `Finished` with no errors (warnings are fine; fix any that appear from unused imports left over from earlier tasks).

- [ ] **Step 3: Run the full test suite**

Run: `cargo test`
Expected: every test from Tasks 2–7 passes (18 tests: 2 models + 4 pattern_engine + 4 pattern_store + 1 default_patterns + 3 mcp_server + 2 cli, plus `seed_defaults_loads_exactly_the_four_mvp_patterns_once` = 17 total -- count what actually ran and make sure none were skipped).

- [ ] **Step 4: Smoke-test the CLI by hand**

Run: `rm -f norma.db && cargo run -- list-patterns`
Expected: 4 lines, one per default pattern, e.g. `no-debug-print-java   java   [warning] No Debug Print`.

Run: `echo 'fn main() { println!("hi"); }' > /tmp/norma-smoke.rs && cargo run -- validate --file /tmp/norma-smoke.rs --language rust; echo "exit: $?"`
Expected: one violation line for the `println!` call, and `exit: 1`.

- [ ] **Step 5: Commit**

```bash
git add src/main.rs
git commit -m "norma: wire CLI + MCP server together in main.rs"
```

---

### Task 9: Dogfooding integration test

**Files:**
- Create: `tests/dogfooding.rs`

**Interfaces:**
- Consumes: `norma::pattern_engine::validate`, `norma::default_patterns` (via the library target added in Task 1 -- this is why it exists).

This is the integration test the "MVP-Pattern-Set-Scope" ticket promised: norma's own Rust default pattern, run against norma's own source, should currently find nothing (verified manually with `grep -rn "println!" src/` while writing that ticket -- this test makes that fact permanent and enforced).

- [ ] **Step 1: Write the test**

Create `tests/dogfooding.rs`:

```rust
//! norma validates itself: the "no debug print" pattern for Rust, run
//! against norma's own `src/` directory, must find zero violations.
//! This is the dogfooding demo decided in the "MVP-Pattern-Set-Scope"
//! ticket (`.scratch/norma-architecture/issues/05-mvp-pattern-set-scope.md`).

use norma::default_patterns;
use norma::pattern_engine::{parse_rule, validate};
use std::fs;
use std::path::Path;

#[test]
fn norma_has_no_debug_prints_in_its_own_source() {
    let rust_default = default_patterns::ALL
        .iter()
        .find(|def| parse_rule(def.rule).unwrap().id == "no-debug-print-rust")
        .expect("the MVP pattern set must include a Rust default pattern");

    let pattern = norma::models::Pattern {
        id: "no-debug-print-rust".to_string(),
        name: rust_default.name.to_string(),
        description: rust_default.description.to_string(),
        category: Some(rust_default.category.to_string()),
        language: "rust".to_string(),
        severity: norma::models::Severity::Warning,
        rule: rust_default.rule.to_string(),
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut total_violations = 0;
    for entry in fs::read_dir(&src_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let code = fs::read_to_string(&path).unwrap();
        let result = validate(&code, "rust", std::slice::from_ref(&pattern));
        assert!(
            result.violations.is_empty(),
            "{} contains a debug println!: {:?}",
            path.display(),
            result.violations
        );
        total_violations += result.violations.len();
    }
    assert_eq!(total_violations, 0);
}
```

- [ ] **Step 2: Run it and confirm it passes**

Run: `cargo test --test dogfooding -- --nocapture`
Expected: `test norma_has_no_debug_prints_in_its_own_source ... ok`.

If it fails, that means a `println!` really did creep into `src/` during this plan's implementation -- replace it with a `tracing` call rather than weakening the test.

- [ ] **Step 3: Commit**

```bash
git add tests/dogfooding.rs
git commit -m "norma: add dogfooding test (no debug prints in norma's own src/)"
```

---

### Task 10: Pre-commit integration and docs

**Files:**
- Create: `.pre-commit-config.yaml`
- Modify: `README.md`, `DEVELOPMENT.md`

**Interfaces:**
- None (documentation and a config template; no code depends on this task).

- [ ] **Step 1: Create `.pre-commit-config.yaml`**

Per the "CLI/Pre-Commit-Architektur" ticket (`language: system` -- references the already-built binary rather than having pre-commit compile Rust on every run):

```yaml
repos:
  - repo: local
    hooks:
      - id: norma-validate
        name: norma pattern validation
        description: >
          Validates staged files against norma's registered patterns.
          Requires `norma` to be installed and on PATH first: run
          `cargo install --path .` once from the norma repo.
        entry: norma validate --language rust --file
        language: system
        files: \.rs$
```

- [ ] **Step 2: Update `README.md`**

Replace the "Quick Start" / usage section (or add one if missing) with:

```markdown
## Usage

Build and install once:

```bash
cargo build --release
cargo install --path .
```

Run the MCP tool server (for Claude Code / MCP Inspector):

```bash
norma serve
```

Validate a file from the command line:

```bash
norma validate --file src/main.rs --language rust
norma validate --file src/main.rs --language rust --json
```

List every registered pattern:

```bash
norma list-patterns
```

Enable the pre-commit hook (see `.pre-commit-config.yaml` -- requires
`norma` already installed via `cargo install --path .`):

```bash
pre-commit install
```
```

- [ ] **Step 3: Update `DEVELOPMENT.md`**

Replace the "Current Status" section's checklist to reflect what this plan actually built (`rmcp`, `ast-grep-core`, single binary + CLI, SQLite with the ADR 0002 schema, the four `no-debug-print` defaults, the dogfooding test) instead of the old mcpkit/regex scaffold description. Point future work at the map's remaining fog (`.scratch/norma-architecture/map.md` -- "Not yet specified: Teststrategie") and at a v2 pattern set including GoF patterns (deferred per ticket 05).

- [ ] **Step 4: Commit**

```bash
git add .pre-commit-config.yaml README.md DEVELOPMENT.md
git commit -m "norma: add pre-commit hook template, update README/DEVELOPMENT docs"
```

---

## Self-Review

**Spec coverage:**
- mcpkit → rmcp: Tasks 6, 8. ✅
- ast-grep-core pattern matching: Task 3. ✅
- Single binary, clap subcommands, shared core (ADR 0001): Tasks 7, 8. ✅
- Pattern single-language, full RuleConfig YAML, derived severity/language, fail-fast register (ADR 0002): Tasks 2, 4. ✅
- MVP pattern set, dogfooding (ticket 05): Tasks 5, 9. ✅
- Pre-commit `language: system` (CLI/pre-commit ticket): Task 10. ✅
- Standing preferences (pinned ast-grep versions, lean dependencies, English pedagogical comments): Global Constraints + applied throughout. ✅
- "Not yet specified: Teststrategie" fog: deliberately not resolved by this plan (it was left open on the map because it doesn't block implementation) -- each task supplies its own tests as it goes, which is a reasonable de facto answer; noted in Task 10's `DEVELOPMENT.md` update rather than invented as a separate up-front decision.

**Placeholder scan:** none found -- every step has real code or a real shell command.

**Type consistency:** `Pattern`, `Severity`, `PatternViolation`, `CodeLocation`, `ValidationResult` (Task 2) are used identically in Tasks 3, 4, 5, 6, 9. `PatternStore::{new, register_pattern, get_patterns_for_language, list_all_patterns, seed_defaults}` (Tasks 4–5) are called with matching signatures in Tasks 6 and 8. `pattern_engine::{parse_rule, language_key, validate}` (Task 3) match their call sites in Tasks 4, 6, 8, 9. `Cli`/`Command` (Task 7) match `main.rs`'s `match cli.command` (Task 8).
