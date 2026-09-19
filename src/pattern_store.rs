use crate::models::*;
use anyhow::{Result, anyhow};
use chrono::Utc;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::path::Path;
use tracing::{info, debug};

pub struct PatternStore {
    pool: SqlitePool,
}

impl PatternStore {
    /// Create a new pattern store, connecting to SQLite database
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let database_url = format!("sqlite:{}", db_path.as_ref().display());
        
        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        // Run migrations
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS patterns (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL,
                rule TEXT NOT NULL,
                rewrite TEXT,
                severity TEXT NOT NULL,
                languages TEXT NOT NULL,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        info!("Pattern store initialized with SQLite");

        Ok(Self { pool })
    }

    /// Initialize with default patterns
    pub async fn initialize_defaults(&self) -> Result<()> {
        // Check if defaults already exist
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM patterns")
            .fetch_one(&self.pool)
            .await?;

        if count.0 > 0 {
            debug!("Patterns already exist, skipping defaults");
            return Ok(());
        }

        // Add some starter patterns
        let defaults = vec![
            Pattern {
                id: "java-static-import".to_string(),
                name: "Avoid Static Imports".to_string(),
                description: "Discourage wildcard static imports in Java".to_string(),
                rule: r"import\s+static\s+\S+\.\*".to_string(),
                rewrite: None,
                severity: Severity::Warning,
                languages: vec!["java".to_string()],
                enabled: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Pattern {
                id: "ts-console-log".to_string(),
                name: "Remove Debug Logs".to_string(),
                description: "Remove console.log from production code".to_string(),
                rule: r"console\.log\(".to_string(),
                rewrite: Some("// Removed console.log".to_string()),
                severity: Severity::Error,
                languages: vec!["typescript".to_string(), "javascript".to_string()],
                enabled: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Pattern {
                id: "java-nested-ternary".to_string(),
                name: "Avoid Nested Ternary".to_string(),
                description: "Nested ternary operators reduce readability".to_string(),
                rule: r"\?\s*.*\s*:\s*\(.*\?".to_string(),
                rewrite: None,
                severity: Severity::Warning,
                languages: vec!["java".to_string(), "typescript".to_string()],
                enabled: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        for pattern in defaults {
            self.save_pattern(pattern).await?;
        }

        info!("Default patterns loaded");
        Ok(())
    }

    /// Save or update a pattern
    pub async fn save_pattern(&self, pattern: Pattern) -> Result<Pattern> {
        let languages_str = serde_json::to_string(&pattern.languages)?;
        
        sqlx::query(
            r#"
            INSERT INTO patterns 
            (id, name, description, rule, rewrite, severity, languages, enabled, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                rule = excluded.rule,
                rewrite = excluded.rewrite,
                severity = excluded.severity,
                languages = excluded.languages,
                enabled = excluded.enabled,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&pattern.id)
        .bind(&pattern.name)
        .bind(&pattern.description)
        .bind(&pattern.rule)
        .bind(&pattern.rewrite)
        .bind(format!("{:?}", pattern.severity).to_lowercase())
        .bind(languages_str)
        .bind(pattern.enabled)
        .bind(pattern.created_at.to_rfc3339())
        .bind(pattern.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        info!("Pattern saved: {} (id: {})", pattern.name, pattern.id);
        Ok(pattern)
    }

    /// Get patterns for a specific language
    pub async fn get_patterns_for_language(&self, language: &str) -> Result<Vec<Pattern>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, rule, rewrite, severity, languages, enabled, created_at, updated_at
            FROM patterns
            WHERE enabled = 1 AND languages LIKE ?
            "#,
        )
        .bind(format!("%{}%", language))
        .fetch_all(&self.pool)
        .await?;

        let patterns = rows
            .into_iter()
            .filter_map(|row| self.parse_pattern_row(&row).ok())
            .collect();

        Ok(patterns)
    }

    /// List all patterns
    pub async fn list_all_patterns(&self) -> Result<Vec<Pattern>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, rule, rewrite, severity, languages, enabled, created_at, updated_at
            FROM patterns
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let patterns = rows
            .into_iter()
            .filter_map(|row| self.parse_pattern_row(&row).ok())
            .collect();

        Ok(patterns)
    }

    /// Get a pattern by ID
    pub async fn get_pattern(&self, id: &str) -> Result<Pattern> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, rule, rewrite, severity, languages, enabled, created_at, updated_at
            FROM patterns
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        self.parse_pattern_row(&row)
    }

    /// Delete a pattern by ID
    pub async fn delete_pattern(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM patterns WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        info!("Pattern deleted: {}", id);
        Ok(())
    }

    // Helper function to parse a pattern from a database row
    fn parse_pattern_row(&self, row: &sqlx::sqlite::SqliteRow) -> Result<Pattern> {
        let id: String = row.get("id");
        let name: String = row.get("name");
        let description: String = row.get("description");
        let rule: String = row.get("rule");
        let rewrite: Option<String> = row.get("rewrite");
        let severity_str: String = row.get("severity");
        let languages_str: String = row.get("languages");
        let enabled: bool = row.get("enabled");
        let created_at_str: String = row.get("created_at");
        let updated_at_str: String = row.get("updated_at");

        let severity = match severity_str.as_str() {
            "error" => Severity::Error,
            "critical" => Severity::Critical,
            "info" => Severity::Info,
            _ => Severity::Warning,
        };

        let languages: Vec<String> = serde_json::from_str(&languages_str)?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)?
            .with_timezone(&Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)?
            .with_timezone(&Utc);

        Ok(Pattern {
            id,
            name,
            description,
            rule,
            rewrite,
            severity,
            languages,
            enabled,
            created_at,
            updated_at,
        })
    }
}
