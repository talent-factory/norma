# GoF Pattern Set v2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend norma's default pattern set with four classic Gang-of-Four patterns (Singleton, Factory, Observer, Strategy), each expressed once per MVP language (Java, Python, Rust, TypeScript) -- 16 new default patterns total -- and make `pattern_engine::validate`'s scoring severity-aware so an `info`-level match (Observer) can't fail a validation run.

**Architecture:** Purely additive on top of the existing MVP structure (ADR 0001/0002): all 16 new patterns are `DefaultPattern` entries in `src/default_patterns.rs`, registered through the unchanged `PatternStore::seed_defaults`/`Pattern::from_rule` path. The one non-additive change is in `pattern_engine::validate`: `off`/`hint`/`info`-severity matches are still reported in `ValidationResult.violations` but no longer count toward `passed`/`score`.

**Tech Stack:** Rust, `ast-grep-core`/`-config`/`-language` `=0.45.3` (unchanged, exact pin), existing `PatternStore`/`pattern_engine` modules. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-20-gof-pattern-set-v2-design.md` and `.scratch/norma-architecture/issues/06-gof-pattern-set-v2-scope.md` (holds the 16 verified `rule` YAML documents this plan copies verbatim). This plan implements that design; it does not re-open its decisions.

## Global Constraints

- `ast-grep-core`, `ast-grep-config`, `ast-grep-language` stay pinned at `=0.45.3` -- no version bump in this plan.
- Every new `rule` YAML below was already verified against real `ast-grep-core` while writing Ticket 06 (positive and negative case each); copy it verbatim, do not "improve" the YAML while implementing.
- `Pattern`'s `id`/`language`/`severity` are always derived from `rule` via `Pattern::from_rule` -- never construct a `Pattern` by hand outside test fixtures that already exist (`Pattern::from_trusted_row` stays test/`PatternStore`-only, unchanged by this plan).
- Code comments: English, written for a reader who doesn't yet know the toolset (per the MVP plan's Global Constraints -- norma is an FFHS teaching artifact).
- No `PatternStore`, MCP tool, or CLI changes -- this plan only touches `src/pattern_engine.rs`, `src/default_patterns.rs`, `tests/gof_patterns.rs` (new), `DEVELOPMENT.md`, `README.md`.

---

### Task 1: Severity-aware scoring in `pattern_engine::validate`

**Files:**
- Modify: `src/pattern_engine.rs` (the `validate` function and its test module)

**Interfaces:**
- Consumes: `models::{Severity, ValidationResult}` (unchanged shapes).
- Produces: `validate`'s behavior change that every later task (and the MCP server / CLI, unchanged callers) relies on: an `off`/`hint`/`info`-severity match appears in `ValidationResult.violations` but does not affect `passed` or `score`.

- [ ] **Step 1: Write the failing test**

Add this to `src/pattern_engine.rs`'s `#[cfg(test)] mod tests` block, near the other `validate_*` tests:

```rust
const RUST_OBSERVER_PRESENCE_INFO: &str = r#"
id: observer-presence-rust
message: Struct has an observers-style field -- an Observer-shaped construct
severity: info
language: Rust
rule:
  kind: struct_item
  has:
    stopBy: end
    kind: field_declaration
    regex: observers
"#;

#[test]
fn validate_info_severity_match_is_visible_but_does_not_fail_the_run() {
    let pattern = test_pattern(RUST_OBSERVER_PRESENCE_INFO);
    let source = "struct Publisher { observers: Vec<Box<dyn Observer>> }";
    let result = validate(source, "rust", &[pattern]).unwrap();
    assert_eq!(result.violations.len(), 1, "the match must still be visible");
    assert_eq!(result.violations[0].severity, Severity::Info);
    assert!(result.passed, "an info-severity match must not fail the run");
    assert_eq!(result.score, 1.0, "an info-severity match must not lower the score");
}
```

- [ ] **Step 2: Run it and confirm it fails**

Run: `cargo test --lib -- pattern_engine::tests::validate_info_severity_match_is_visible_but_does_not_fail_the_run --nocapture`
Expected: FAIL at `assert!(result.passed, ...)` -- today every match (regardless of severity) makes `violations` non-empty, and `passed = violations.is_empty()`, so `passed` is currently `false`.

- [ ] **Step 3: Make severity-aware scoring the implementation**

In `src/pattern_engine.rs`, replace the `score`/`passed` computation inside `validate` (currently the block starting with the comment `// Score is the real hit rate...` through `let passed = violations.is_empty();`) with:

```rust
    // Score is the real hit rate: computed before any synthetic coverage
    // warning is appended below, so it never divides by patterns that
    // never actually ran. Only warning/error-severity matches count
    // against the score -- an info/hint/off match (e.g. the purely
    // informational `observer-presence-*` pattern) is still visible in
    // `violations` below, but finding one isn't a defect, so it must not
    // make the score worse.
    let scoring_matches = violations
        .iter()
        .filter(|v| matches!(v.severity, Severity::Warning | Severity::Error))
        .count();
    let score = if checked_patterns == 0 {
        0.0
    } else {
        (1.0 - scoring_matches as f64 / checked_patterns as f64).max(0.0)
    };

    for (pattern_id, err) in &skipped {
        violations.push(coverage_warning(
            pattern_id,
            "Unparsable Pattern",
            format!("this pattern's stored rule could not be parsed and was skipped: {err}"),
        ));
    }
    if checked_patterns == 0 && skipped.is_empty() {
        violations.push(coverage_warning(
            NO_COVERAGE_PATTERN_ID,
            "No Coverage",
            format!(
                "no enabled patterns are registered for language {language:?} -- nothing was validated"
            ),
        ));
    }

    // `passed` mirrors `score`'s severity filter, checked after the
    // synthetic coverage warnings above (also Warning-severity) are folded
    // into `violations` -- degraded coverage must still fail a run exactly
    // as before.
    let passed = !violations
        .iter()
        .any(|v| matches!(v.severity, Severity::Warning | Severity::Error));
```

(The `for`/`if` block in the middle is unchanged from the current code -- it's shown here only so the surrounding context is unambiguous about where the new `score`/`passed` lines go.)

- [ ] **Step 4: Run the full `pattern_engine` test suite and confirm everything passes**

Run: `cargo test --lib -- pattern_engine:: --nocapture`
Expected: `15 passed; 0 failed` (the 14 tests that existed before this task, unaffected because they all match on `Warning`-severity rules or the `Warning`-severity `coverage_warning`, plus the new one).

- [ ] **Step 5: Commit**

```bash
git add src/pattern_engine.rs
git commit -m "norma: make validate's score/passed severity-aware (info matches don't fail a run)"
```

---

### Task 2: Generalize the `default_patterns` coverage test

**Files:**
- Modify: `src/default_patterns.rs` (test module only -- no new patterns yet)

**Interfaces:**
- None new. This is a pure test refactor that removes a hardcoded "exactly 4 entries" assumption before Tasks 3-6 add 16 more entries to `ALL` -- without it, adding even one new pattern would break the existing `every_default_rule_parses_and_covers_one_mvp_language_each` test's `assert_eq!(languages, ["java", "python", "rust", "typescript"])`, which assumes exactly one entry per language.

This task has no red step -- it's a refactor that keeps the same guarantees (every rule parses, every rule targets a supported language, all four languages are covered), just expressed in a way that doesn't break as `ALL` grows.

- [ ] **Step 1: Replace the existing test**

In `src/default_patterns.rs`, replace the whole `#[cfg(test)] mod tests { ... }` block with:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern_engine::{language_key, parse_rule};

    #[test]
    fn every_default_rule_parses_and_targets_a_supported_language() {
        for def in ALL {
            let config = parse_rule(def.rule).expect("default rule must parse");
            language_key(config.language)
                .expect("every default rule must target a language norma supports");
        }
    }

    #[test]
    fn every_mvp_language_has_at_least_one_default_pattern() {
        let mut languages: Vec<&str> = ALL
            .iter()
            .map(|def| {
                let config = parse_rule(def.rule).expect("default rule must parse");
                language_key(config.language).unwrap()
            })
            .collect();
        languages.sort();
        languages.dedup();
        assert_eq!(languages, ["java", "python", "rust", "typescript"]);
    }
}
```

- [ ] **Step 2: Run the tests and confirm they pass**

Run: `cargo test --lib -- default_patterns:: --nocapture`
Expected: `2 passed; 0 failed`.

- [ ] **Step 3: Commit**

```bash
git add src/default_patterns.rs
git commit -m "norma: generalize default_patterns coverage test ahead of the GoF v2 additions"
```

---

### Task 3: Singleton default patterns (Java, Python, Rust, TypeScript)

**Files:**
- Modify: `src/default_patterns.rs` (append 4 entries to `ALL`)
- Create: `tests/gof_patterns.rs`

**Interfaces:**
- Consumes: `default_patterns::ALL` (Task 2's generalized test already tolerates growth), `pattern_engine::{parse_rule, validate}`, `models::Pattern::from_rule`.
- Produces: `tests/gof_patterns.rs`'s `default_pattern`/`assert_matches`/`assert_does_not_match` helpers, reused by Tasks 4-6.

- [ ] **Step 1: Append the four Singleton entries to `ALL`**

In `src/default_patterns.rs`, add these four entries at the end of the `ALL` slice (after the existing four MVP entries, before the closing `];`):

```rust
    DefaultPattern {
        name: "Singleton Implementation Quality",
        description: "Class looks like a Singleton (private static instance field) but its constructor is not private, so callers can bypass the single-instance guarantee.",
        category: "creational",
        rule: r#"
id: singleton-quality-java
message: Class looks like a Singleton (private static instance field) but its constructor is not private
severity: warning
language: Java
rule:
  kind: class_declaration
  all:
    - has:
        stopBy: end
        kind: field_declaration
        pattern:
          context: 'class C { private static $TYPE instance; }'
          selector: field_declaration
    - has:
        stopBy: end
        kind: constructor_declaration
        not:
          has:
            kind: modifiers
            regex: private
"#,
    },
    DefaultPattern {
        name: "Singleton Implementation Quality",
        description: "Class has a Singleton-style `_instance` attribute but no `__new__` guard enforcing a single instance.",
        category: "creational",
        rule: r#"
id: singleton-quality-python
message: Class has a Singleton-style `_instance` attribute but no `__new__` guard enforcing a single instance
severity: warning
language: Python
rule:
  kind: class_definition
  all:
    - has:
        stopBy: end
        kind: assignment
        pattern: _instance = None
    - not:
        has:
          stopBy: end
          kind: function_definition
          has:
            field: name
            regex: '^__new__$'
"#,
    },
    DefaultPattern {
        name: "Singleton Implementation Quality",
        description: "A public `new()` next to a module-level `static INSTANCE` defeats the Singleton -- callers can construct extra instances directly.",
        category: "creational",
        rule: r#"
id: singleton-quality-rust
message: A public `new()` next to a module-level `static INSTANCE` defeats the Singleton -- callers can construct extra instances directly
severity: warning
language: Rust
rule:
  kind: source_file
  all:
    - has:
        stopBy: end
        kind: static_item
        pattern: static INSTANCE $$$REST
    - has:
        stopBy: end
        kind: function_item
        pattern: pub fn new($$$PARAMS) -> $$$RET { $$$BODY }
"#,
    },
    DefaultPattern {
        name: "Singleton Implementation Quality",
        description: "Class looks like a Singleton (private static instance field) but its constructor is not private, so callers can bypass the single-instance guarantee.",
        category: "creational",
        rule: r#"
id: singleton-quality-typescript
message: Class looks like a Singleton (private static instance field) but its constructor is not private
severity: warning
language: TypeScript
rule:
  kind: class_declaration
  all:
    - has:
        stopBy: end
        kind: public_field_definition
        pattern:
          context: 'class C { private static instance: $TYPE; }'
          selector: public_field_definition
    - has:
        stopBy: end
        kind: method_definition
        pattern:
          context: 'class C { constructor() {} }'
          selector: method_definition
        not:
          has:
            regex: private
"#,
    },
```

- [ ] **Step 2: Run the generalized coverage tests and confirm they still pass**

Run: `cargo test --lib -- default_patterns:: --nocapture`
Expected: `2 passed; 0 failed` (Task 2's tests tolerate the new entries by construction).

- [ ] **Step 3: Write the failing behavioral tests**

Create `tests/gof_patterns.rs`:

```rust
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
```

- [ ] **Step 4: Run it and confirm it passes**

Run: `cargo test --test gof_patterns -- --nocapture`
Expected: `4 passed; 0 failed` (`singleton_quality_java`, `singleton_quality_python`, `singleton_quality_rust`, `singleton_quality_typescript`).

(If any fail, do not "adjust" the rule YAML -- it was already verified in Ticket 06. Check first whether the source snippet was copied exactly; a mismatch there is far more likely than the verified rule being wrong.)

- [ ] **Step 5: Commit**

```bash
git add src/default_patterns.rs tests/gof_patterns.rs
git commit -m "norma: add Singleton Implementation Quality default patterns (GoF v2)"
```

---

### Task 4: Factory default patterns (Java, Python, Rust, TypeScript)

**Files:**
- Modify: `src/default_patterns.rs` (append 4 entries), `tests/gof_patterns.rs` (append 4 tests)

**Interfaces:**
- Consumes: `default_pattern`/`assert_matches`/`assert_does_not_match` from Task 3.

- [ ] **Step 1: Append the four Factory entries to `ALL`**

```rust
    DefaultPattern {
        name: "Factory Overuse (Type Switch)",
        description: "Type-switch construction (if/else-if each calling `new`) suggests a Factory would be a better fit.",
        category: "creational",
        rule: r#"
id: factory-overuse-java
message: Type-switch construction (if/else-if each calling `new`) suggests a Factory would be a better fit
severity: warning
language: Java
rule:
  pattern: |
    if ($COND1) {
      new $TYPE1($$$ARGS1);
    } else if ($COND2) {
      new $TYPE2($$$ARGS2);
    }
"#,
    },
    DefaultPattern {
        name: "Factory Overuse (Type Switch)",
        description: "Type-switch construction (if/elif each instantiating a different class) suggests a Factory would be a better fit.",
        category: "creational",
        rule: r#"
id: factory-overuse-python
message: Type-switch construction (if/elif each instantiating a different class) suggests a Factory would be a better fit
severity: warning
language: Python
rule:
  pattern: |
    if $COND1:
        $TYPE1($$$ARGS1)
    elif $COND2:
        $TYPE2($$$ARGS2)
"#,
    },
    DefaultPattern {
        name: "Factory Overuse (Type Switch)",
        description: "Type-switch construction (if/else-if each building a different struct literal) suggests a Factory function would be a better fit.",
        category: "creational",
        rule: r#"
id: factory-overuse-rust
message: Type-switch construction (if/else-if each building a different struct literal) suggests a Factory function would be a better fit
severity: warning
language: Rust
rule:
  pattern: |
    if $COND1 {
        $TYPE1 { $$$FIELDS1 }
    } else if $COND2 {
        $TYPE2 { $$$FIELDS2 }
    }
"#,
    },
    DefaultPattern {
        name: "Factory Overuse (Type Switch)",
        description: "Type-switch construction (if/else-if each calling `new`) suggests a Factory would be a better fit.",
        category: "creational",
        rule: r#"
id: factory-overuse-typescript
message: Type-switch construction (if/else-if each calling `new`) suggests a Factory would be a better fit
severity: warning
language: TypeScript
rule:
  pattern: |
    if ($COND1) {
      new $TYPE1($$$ARGS1);
    } else if ($COND2) {
      new $TYPE2($$$ARGS2);
    }
"#,
    },
```

- [ ] **Step 2: Write the failing behavioral tests**

Append to `tests/gof_patterns.rs`:

```rust
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
```

- [ ] **Step 3: Run it and confirm it passes**

Run: `cargo test --test gof_patterns -- --nocapture`
Expected: `8 passed; 0 failed` (Task 3's 4 plus these 4).

- [ ] **Step 4: Commit**

```bash
git add src/default_patterns.rs tests/gof_patterns.rs
git commit -m "norma: add Factory Overuse (Type Switch) default patterns (GoF v2)"
```

---

### Task 5: Observer default patterns (Java, Python, Rust, TypeScript)

**Files:**
- Modify: `src/default_patterns.rs` (append 4 entries), `tests/gof_patterns.rs` (append 4 tests)

**Interfaces:**
- Consumes: same helpers as Task 3/4. Also the first real behavioral proof (beyond Task 1's unit test) that an `info`-severity default pattern round-trips through `Pattern::from_rule` and `validate` correctly.

- [ ] **Step 1: Append the four Observer entries to `ALL`**

```rust
    DefaultPattern {
        name: "Observer Presence",
        description: "Class has a Listener collection and a notify-style method -- an Observer-shaped construct.",
        category: "behavioral",
        rule: r#"
id: observer-presence-java
message: Class has a Listener collection and a notify-style method -- an Observer-shaped construct
severity: info
language: Java
rule:
  kind: class_declaration
  all:
    - has:
        stopBy: end
        kind: field_declaration
        regex: Listener
    - has:
        stopBy: end
        kind: method_declaration
        regex: notify
"#,
    },
    DefaultPattern {
        name: "Observer Presence",
        description: "Class has an observers collection and a notify-style method -- an Observer-shaped construct.",
        category: "behavioral",
        rule: r#"
id: observer-presence-python
message: Class has an observers collection and a notify-style method -- an Observer-shaped construct
severity: info
language: Python
rule:
  kind: class_definition
  all:
    - has:
        stopBy: end
        kind: assignment
        regex: observers
    - has:
        stopBy: end
        kind: function_definition
        regex: notify
"#,
    },
    DefaultPattern {
        name: "Observer Presence",
        description: "Struct has an observers-style field -- an Observer-shaped construct.",
        category: "behavioral",
        rule: r#"
id: observer-presence-rust
message: Struct has an observers-style field -- an Observer-shaped construct
severity: info
language: Rust
rule:
  kind: struct_item
  has:
    stopBy: end
    kind: field_declaration
    regex: observers
"#,
    },
    DefaultPattern {
        name: "Observer Presence",
        description: "Class has an observers collection and a notify-style method -- an Observer-shaped construct.",
        category: "behavioral",
        rule: r#"
id: observer-presence-typescript
message: Class has an observers collection and a notify-style method -- an Observer-shaped construct
severity: info
language: TypeScript
rule:
  kind: class_declaration
  all:
    - has:
        stopBy: end
        kind: public_field_definition
        regex: observers
    - has:
        stopBy: end
        kind: method_definition
        regex: notify
"#,
    },
```

- [ ] **Step 2: Write the failing behavioral tests**

Append to `tests/gof_patterns.rs`:

```rust
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
```

- [ ] **Step 3: Run it and confirm it passes**

Run: `cargo test --test gof_patterns -- --nocapture`
Expected: `13 passed; 0 failed` (Tasks 3+4's 8, plus these 5).

- [ ] **Step 4: Commit**

```bash
git add src/default_patterns.rs tests/gof_patterns.rs
git commit -m "norma: add Observer Presence default patterns (GoF v2, info severity)"
```

---

### Task 6: Strategy default patterns (Java, Python, Rust, TypeScript)

**Files:**
- Modify: `src/default_patterns.rs` (append 4 entries), `tests/gof_patterns.rs` (append 4 tests)

**Interfaces:**
- Consumes: same helpers as Tasks 3-5. This is the last of the 16 new `DefaultPattern` entries -- Task 7 asserts the final total.

- [ ] **Step 1: Append the four Strategy entries to `ALL`**

```rust
    DefaultPattern {
        name: "Strategy Overuse (Type Switch)",
        description: "Type-switch behavior selection (if/else-if each calling a different method) suggests a Strategy would be a better fit.",
        category: "behavioral",
        rule: r#"
id: strategy-overuse-java
message: Type-switch behavior selection (if/else-if each calling a different method) suggests a Strategy would be a better fit
severity: warning
language: Java
rule:
  pattern: |
    if ($COND1) {
      $METHOD1($$$ARGS1);
    } else if ($COND2) {
      $METHOD2($$$ARGS2);
    }
"#,
    },
    DefaultPattern {
        name: "Strategy Overuse (Type Switch)",
        description: "Type-switch behavior selection (if/elif each calling a different function) suggests a Strategy would be a better fit.",
        category: "behavioral",
        rule: r#"
id: strategy-overuse-python
message: Type-switch behavior selection (if/elif each calling a different function) suggests a Strategy would be a better fit
severity: warning
language: Python
rule:
  pattern: |
    if $COND1:
        $METHOD1($$$ARGS1)
    elif $COND2:
        $METHOD2($$$ARGS2)
"#,
    },
    DefaultPattern {
        name: "Strategy Overuse (Type Switch)",
        description: "Type-switch behavior selection (if/else-if each calling a different function) suggests a Strategy would be a better fit.",
        category: "behavioral",
        rule: r#"
id: strategy-overuse-rust
message: Type-switch behavior selection (if/else-if each calling a different function) suggests a Strategy would be a better fit
severity: warning
language: Rust
rule:
  pattern: |
    if $COND1 {
        $METHOD1($$$ARGS1);
    } else if $COND2 {
        $METHOD2($$$ARGS2);
    }
"#,
    },
    DefaultPattern {
        name: "Strategy Overuse (Type Switch)",
        description: "Type-switch behavior selection (if/else-if each calling a different method) suggests a Strategy would be a better fit.",
        category: "behavioral",
        rule: r#"
id: strategy-overuse-typescript
message: Type-switch behavior selection (if/else-if each calling a different method) suggests a Strategy would be a better fit
severity: warning
language: TypeScript
rule:
  pattern: |
    if ($COND1) {
      $METHOD1($$$ARGS1);
    } else if ($COND2) {
      $METHOD2($$$ARGS2);
    }
"#,
    },
```

- [ ] **Step 2: Write the failing behavioral tests**

Append to `tests/gof_patterns.rs`:

```rust
#[test]
fn strategy_overuse_java() {
    assert_matches(
        "strategy-overuse-java",
        "java",
        "class Payment { void pay(String kind) { if (kind.equals(\"card\")) { payByCard(); } else if (kind.equals(\"cash\")) { payByCash(); } } }",
    );
    assert_does_not_match(
        "strategy-overuse-java",
        "java",
        "class Payment { void pay(String kind) { if (kind.equals(\"card\")) { payByCard(); } } }",
    );
}

#[test]
fn strategy_overuse_python() {
    assert_matches(
        "strategy-overuse-python",
        "python",
        "def pay(kind):\n    if kind == 'card':\n        pay_by_card()\n    elif kind == 'cash':\n        pay_by_cash()\n",
    );
    assert_does_not_match(
        "strategy-overuse-python",
        "python",
        "def pay(kind):\n    if kind == 'card':\n        pay_by_card()\n",
    );
}

#[test]
fn strategy_overuse_rust() {
    assert_matches(
        "strategy-overuse-rust",
        "rust",
        "fn pay(kind: &str) { if kind == \"card\" { pay_by_card(); } else if kind == \"cash\" { pay_by_cash(); } }",
    );
    assert_does_not_match(
        "strategy-overuse-rust",
        "rust",
        "fn pay(kind: &str) { if kind == \"card\" { pay_by_card(); } }",
    );
}

#[test]
fn strategy_overuse_typescript() {
    assert_matches(
        "strategy-overuse-typescript",
        "typescript",
        "function pay(kind: string) { if (kind === 'card') { payByCard(); } else if (kind === 'cash') { payByCash(); } }",
    );
    assert_does_not_match(
        "strategy-overuse-typescript",
        "typescript",
        "function pay(kind: string) { if (kind === 'card') { payByCard(); } }",
    );
}
```

- [ ] **Step 3: Run it and confirm it passes**

Run: `cargo test --test gof_patterns -- --nocapture`
Expected: `17 passed; 0 failed` (Tasks 3-5's 13, plus these 4).

- [ ] **Step 4: Commit**

```bash
git add src/default_patterns.rs tests/gof_patterns.rs
git commit -m "norma: add Strategy Overuse (Type Switch) default patterns (GoF v2)"
```

---

### Task 7: Final shape assertions -- 20 patterns, category-per-id-prefix

**Files:**
- Modify: `src/default_patterns.rs` (test module)

**Interfaces:**
- Consumes: the complete `ALL` (20 entries: 4 MVP + 16 GoF) produced by Tasks 3-6.

- [ ] **Step 1: Write the failing tests**

Append to `src/default_patterns.rs`'s `mod tests` block (the one Task 2 generalized):

```rust
    #[test]
    fn the_default_pattern_set_has_exactly_20_entries_after_the_gof_v2_addition() {
        assert_eq!(ALL.len(), 20);
    }

    #[test]
    fn every_default_pattern_id_prefix_matches_its_expected_category() {
        let expected_category = |id: &str| -> &'static str {
            if id.starts_with("no-debug-print-") {
                "code-quality"
            } else if id.starts_with("singleton-") || id.starts_with("factory-") {
                "creational"
            } else if id.starts_with("observer-") || id.starts_with("strategy-") {
                "behavioral"
            } else {
                panic!("no expected category mapping for pattern id {id:?} -- update this test's mapping when adding a new pattern family");
            }
        };
        for def in ALL {
            let config = parse_rule(def.rule).expect("default rule must parse");
            assert_eq!(
                def.category,
                expected_category(&config.id),
                "category mismatch for {}",
                config.id
            );
        }
    }
```

- [ ] **Step 2: Run it and confirm it passes**

Run: `cargo test --lib -- default_patterns:: --nocapture`
Expected: `4 passed; 0 failed` (Task 2's 2 plus these 2).

- [ ] **Step 3: Run the entire test suite as a final sanity check**

Run: `cargo test`
Expected: every test across `src/` and `tests/` passes -- by this point that's 15 `pattern_engine` tests (Task 1), 4 `default_patterns` tests (Task 2 + this task), the unchanged `models`/`pattern_store`/`mcp_server`/`cli` tests from the MVP, `tests/dogfooding.rs`'s one test, and `tests/gof_patterns.rs`'s 17 tests (Tasks 3-6). Count what actually ran; investigate anything unexpected before moving on.

- [ ] **Step 4: Commit**

```bash
git add src/default_patterns.rs
git commit -m "norma: assert the final GoF v2 pattern set shape (20 patterns, category-per-id-prefix)"
```

---

### Task 8: Documentation

**Files:**
- Modify: `DEVELOPMENT.md`, `README.md`

**Interfaces:**
- None -- documentation only, no code depends on this task.

- [ ] **Step 1: Update `DEVELOPMENT.md`'s "Current Status" section**

Move the "v2 pattern set, including GoF patterns" item out of "🔄 Next Steps" and into "✅ Completed", worded like the existing entries, e.g.:

```markdown
- [x] Four GoF default patterns (Singleton, Factory, Observer, Strategy), one per MVP language (16 patterns total, 20 with the MVP set) -- `singleton-quality-*`/`factory-overuse-*` under category `creational`, `observer-presence-*`/`strategy-overuse-*` under category `behavioral`; `observer-presence-*` is `info`-severity and (per `pattern_engine::validate`'s severity-aware scoring) visible without failing a run -- `src/default_patterns.rs`, `docs/superpowers/specs/2026-09-20-gof-pattern-set-v2-design.md`
```

Remove the corresponding numbered item from "🔄 Next Steps" and renumber the remaining ones. Update the Java Singleton YAML example already shown further down in the file (the one with the `# ... the actual field/method shape is still to be worked out` comment) to instead show the real, shipped `singleton-quality-java` rule from `src/default_patterns.rs`, and remove the "still to be worked out" comment since it's no longer true.

- [ ] **Step 2: Update `README.md`'s pattern examples**

In the "🎓 For FFHS Students" section's "Example: catching `System.out.println` in Java" subsection, add one sentence noting that `src/default_patterns.rs` now ships 20 patterns total, including four GoF patterns per language, and point to `DEVELOPMENT.md`'s "Pattern Definition Examples" section for the Singleton example.

- [ ] **Step 3: Commit**

```bash
git add DEVELOPMENT.md README.md
git commit -m "norma: document the GoF pattern set v2 in DEVELOPMENT.md and README.md"
```

---

## Self-Review

**Spec coverage:**
- 16 GoF `DefaultPattern` entries, exact verified YAML, 4x4 matrix: Tasks 3-6. ✅
- `creational`/`behavioral` categories, ID scheme, shared `name` per pattern: Tasks 3-6 (entries), Task 7 (asserted). ✅
- Severity-aware `validate` (info doesn't fail a run): Task 1 (engine), Task 5 (real default pattern proof). ✅
- Testing strategy from the spec (engine-level severity tests, extended coverage test, per-pattern positive/negative tests): Task 1, Task 2/7, Tasks 3-6. ✅
- "No `PatternStore`/MCP/CLI changes" constraint: honored throughout -- no task touches those files. ✅
- Known limitations (Factory/Strategy can't count branches, Rust/TypeScript have no classes) are pre-existing, accepted trade-offs from the spec, not something a task needs to "fix". ✅
- Docs: Task 8. ✅

**Placeholder scan:** none found -- every step has real, complete code or a real shell command; no "TBD"/"similar to Task N" shortcuts.

**Type consistency:** `Pattern::from_rule(name, description, category, rule, enabled, created_at, updated_at) -> anyhow::Result<Self>` (Tasks 3-6's `tests/gof_patterns.rs` helper) matches the real current signature in `src/pattern_engine.rs`. `validate(source, language, patterns) -> anyhow::Result<ValidationResult>` and `ValidationResult { violations, passed, score, checked_patterns, duration_ms }` (Task 1) match every later task's usage. `default_patterns::ALL: &[DefaultPattern]` and `DefaultPattern { name, description, category, rule }` (Tasks 3-6) match `src/default_patterns.rs`'s existing struct, unchanged. `pattern_engine::{parse_rule, language_key}` (Task 2, Task 7) match their existing signatures (`language_key` returns `Option<&'static str>`).
