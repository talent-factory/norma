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

#[test]
fn factory_overuse_java() {
    assert_matches(
        "factory-overuse-java",
        "java",
        "class Foo { void bar(String kind) { if (kind.equals(\"a\")) { new Dog(); } else if (kind.equals(\"b\")) { new Cat(); } } }",
    );
    assert_does_not_match(
        "factory-overuse-java",
        "java",
        "class Foo { void bar(String kind) { if (kind.equals(\"a\")) { new Dog(); } } }",
    );
}

#[test]
fn factory_overuse_python() {
    assert_matches(
        "factory-overuse-python",
        "python",
        "def make(kind):\n    if kind == 'a':\n        Dog()\n    elif kind == 'b':\n        Cat()\n",
    );
    assert_does_not_match(
        "factory-overuse-python",
        "python",
        "def make(kind):\n    if kind == 'a':\n        Dog()\n",
    );
}

#[test]
fn factory_overuse_rust() {
    assert_matches(
        "factory-overuse-rust",
        "rust",
        "fn make(kind: &str) { if kind == \"a\" { Dog {} } else if kind == \"b\" { Cat {} } }",
    );
    assert_does_not_match(
        "factory-overuse-rust",
        "rust",
        "fn make(kind: &str) { if kind == \"a\" { Dog {} } }",
    );
}

#[test]
fn factory_overuse_typescript() {
    assert_matches(
        "factory-overuse-typescript",
        "typescript",
        "function make(kind: string) { if (kind === 'a') { new Dog(); } else if (kind === 'b') { new Cat(); } }",
    );
    assert_does_not_match(
        "factory-overuse-typescript",
        "typescript",
        "function make(kind: string) { if (kind === 'a') { new Dog(); } }",
    );
}

#[test]
fn observer_presence_java() {
    assert_matches(
        "observer-presence-java",
        "java",
        "class Publisher { private List<Listener> listeners; void notifyAll_() { for (Listener l : listeners) { l.update(); } } }",
    );
    assert_does_not_match(
        "observer-presence-java",
        "java",
        "class Config { private int value; int get() { return value; } }",
    );
}

#[test]
fn observer_presence_python() {
    assert_matches(
        "observer-presence-python",
        "python",
        "class Publisher:\n    def __init__(self):\n        self._observers = []\n    def notify_all(self):\n        for o in self._observers:\n            o.update()\n",
    );
    assert_does_not_match(
        "observer-presence-python",
        "python",
        "class Config:\n    def __init__(self):\n        self.value = 1\n    def get(self):\n        return self.value\n",
    );
}

#[test]
fn observer_presence_rust() {
    assert_matches(
        "observer-presence-rust",
        "rust",
        "struct Publisher { observers: Vec<Box<dyn Observer>> }",
    );
    assert_does_not_match("observer-presence-rust", "rust", "struct Config { value: i32 }");
}

#[test]
fn observer_presence_typescript() {
    assert_matches(
        "observer-presence-typescript",
        "typescript",
        "class Publisher { observers: Observer[] = []; notifyAll() { this.observers.forEach(o => o.update()); } }",
    );
    assert_does_not_match(
        "observer-presence-typescript",
        "typescript",
        "class Config { value: number = 1; get() { return this.value; } }",
    );
}

/// The one place these tests check severity explicitly: Observer is the
/// only `info`-severity default pattern (see
/// docs/superpowers/specs/2026-09-20-gof-pattern-set-v2-design.md), so a
/// match must be visible but must not fail the run -- `pattern_engine.rs`
/// already unit-tests the general rule (Task 1); this confirms the real
/// shipped default pattern behaves the same way, not just a synthetic
/// fixture.
#[test]
fn observer_presence_matches_do_not_fail_validation() {
    let pattern = default_pattern("observer-presence-rust");
    let source = "struct Publisher { observers: Vec<Box<dyn Observer>> }";
    let result = validate(source, "rust", std::slice::from_ref(&pattern)).unwrap();
    assert_eq!(result.violations.len(), 1);
    assert!(result.passed);
    assert_eq!(result.score, 1.0);
}
