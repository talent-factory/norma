//! Behavioral coverage for the v2 GoF pattern set (Singleton, Factory,
//! Observer, Strategy) -- see
//! docs/superpowers/specs/2026-09-20-gof-pattern-set-v2-design.md and
//! .scratch/norma-architecture/issues/06-gof-pattern-set-v2-scope.md.
//! Each of the 16 default rules (4 patterns x 4 languages) gets one
//! positive and one negative case, using the exact source snippets
//! already verified against ast-grep-core while writing that ticket.

use norma::default_patterns;
use norma::models::Pattern;
use norma::pattern_engine::{parse_rule, validate};

/// Builds a `Pattern` from the real, shipped `DefaultPattern` whose rule's
/// `id` is `pattern_id` -- so these tests exercise the exact rule norma
/// ships, not a copy that could silently drift from it.
fn default_pattern(pattern_id: &str) -> Pattern {
    let now = chrono::Utc::now();
    let def = default_patterns::ALL
        .iter()
        .find(|def| parse_rule(def.rule).unwrap().id == pattern_id)
        .unwrap_or_else(|| panic!("no default pattern with id {pattern_id:?}"));
    Pattern::from_rule(
        def.name.to_string(),
        def.description.to_string(),
        Some(def.category.to_string()),
        def.rule.to_string(),
        true,
        now,
        now,
    )
    .expect("default pattern's rule must be valid")
}

fn assert_matches(pattern_id: &str, language: &str, source: &str) {
    let pattern = default_pattern(pattern_id);
    let result = validate(source, language, std::slice::from_ref(&pattern)).unwrap();
    assert_eq!(
        result.violations.len(),
        1,
        "{pattern_id} should match {source:?}, got {:?}",
        result.violations
    );
}

fn assert_does_not_match(pattern_id: &str, language: &str, source: &str) {
    let pattern = default_pattern(pattern_id);
    let result = validate(source, language, std::slice::from_ref(&pattern)).unwrap();
    assert!(
        result.violations.is_empty(),
        "{pattern_id} should not match {source:?}, got {:?}",
        result.violations
    );
}

#[test]
fn singleton_quality_java() {
    assert_matches(
        "singleton-quality-java",
        "java",
        "class Config { private static Config instance; public Config() {} }",
    );
    assert_does_not_match(
        "singleton-quality-java",
        "java",
        "class Config { private static Config instance; private Config() {} }",
    );
}

#[test]
fn singleton_quality_python() {
    assert_matches(
        "singleton-quality-python",
        "python",
        "class Config:\n    _instance = None\n    def __init__(self):\n        pass\n",
    );
    assert_does_not_match(
        "singleton-quality-python",
        "python",
        "class Config:\n    _instance = None\n    def __new__(cls):\n        if cls._instance is None:\n            cls._instance = super().__new__(cls)\n        return cls._instance\n",
    );
}

#[test]
fn singleton_quality_rust() {
    assert_matches(
        "singleton-quality-rust",
        "rust",
        "static INSTANCE: OnceLock<Config> = OnceLock::new();\nstruct Config;\nimpl Config { pub fn new() -> Config { Config } }",
    );
    assert_does_not_match(
        "singleton-quality-rust",
        "rust",
        "static INSTANCE: OnceLock<Config> = OnceLock::new();\nstruct Config;\nimpl Config { fn new() -> Config { Config } }",
    );
}

#[test]
fn singleton_quality_typescript() {
    assert_matches(
        "singleton-quality-typescript",
        "typescript",
        "class Config { private static instance: Config; constructor() {} }",
    );
    assert_does_not_match(
        "singleton-quality-typescript",
        "typescript",
        "class Config { private static instance: Config; private constructor() {} }",
    );
}
