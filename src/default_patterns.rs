/// norma's default pattern set: 4 MVP `no-debug-print` patterns (see the
/// "MVP-Pattern-Set-Scope" ticket on the wayfinder map,
/// `.scratch/norma-architecture/issues/05-mvp-pattern-set-scope.md`), one
/// per MVP language, all expressing the same idea -- "no debug prints in
/// production code" -- so the same concept is visibly expressed
/// differently per language, per docs/adr/0002.md; plus 16 GoF patterns --
/// Singleton, Factory, Observer, Strategy, one per MVP language each (see
/// Ticket 06, `.scratch/norma-architecture/issues/06-gof-pattern-set-v2-scope.md`,
/// and `docs/superpowers/specs/2026-09-20-gof-pattern-set-v2-design.md`) --
/// across three categories (`code-quality`, `creational`, `behavioral`).
/// Every rule below was verified against real `ast-grep-core` while
/// writing its respective ticket.
pub struct DefaultPattern {
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub rule: &'static str,
}

pub const ALL: &[DefaultPattern] = &[
    DefaultPattern {
        name: "No Debug Print",
        description:
            "System.out.println left in production code should go through a proper logger instead.",
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
        description:
            "println! left in production code should go through the `tracing` crate instead.",
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
        description:
            "console.log left in production code should go through a proper logger instead.",
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
    DefaultPattern {
        name: "Singleton Implementation Quality",
        description: "Class looks like a Singleton (private static instance field) but its constructor is not private (or is left implicit), so callers can bypass the single-instance guarantee.",
        category: "creational",
        rule: r#"
id: singleton-quality-java
message: Class looks like a Singleton (private static instance field) but its constructor is not private (or is left implicit)
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
    - any:
        - not:
            has:
              stopBy: end
              kind: constructor_declaration
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
        description: "A public `new()` next to a module-level `static INSTANCE` of the same type defeats the Singleton -- callers can construct extra instances directly.",
        category: "creational",
        rule: r#"
id: singleton-quality-rust
message: A public `new()` next to a module-level `static INSTANCE` of the same type defeats the Singleton -- callers can construct extra instances directly
severity: warning
language: Rust
rule:
  kind: impl_item
  all:
    - has:
        field: type
        pattern: $TYPE
    - has:
        stopBy: end
        kind: function_item
        pattern: pub fn new($$$PARAMS) -> $$$RET { $$$BODY }
    - inside:
        stopBy: end
        kind: source_file
        has:
          stopBy: end
          kind: static_item
          any:
            - pattern: 'static INSTANCE: $WRAPPER<$TYPE> = $$$INIT;'
            - pattern: 'static INSTANCE: $TYPE = $$$INIT;'
            - pattern: 'static mut INSTANCE: $WRAPPER<$TYPE> = $$$INIT;'
            - pattern: 'static mut INSTANCE: $TYPE = $$$INIT;'
"#,
    },
    DefaultPattern {
        name: "Singleton Implementation Quality",
        description: "Class looks like a Singleton (private static instance field) but its constructor is not private (or is left implicit), so callers can bypass the single-instance guarantee.",
        category: "creational",
        rule: r#"
id: singleton-quality-typescript
message: Class looks like a Singleton (private static instance field) but its constructor is not private (or is left implicit)
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
    - any:
        - not:
            has:
              stopBy: end
              kind: method_definition
              has:
                kind: property_identifier
                regex: '^constructor$'
        - has:
            stopBy: end
            kind: method_definition
            has:
              kind: property_identifier
              regex: '^constructor$'
            not:
              has:
                kind: accessibility_modifier
                regex: '^private$'
"#,
    },
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
];

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
}
