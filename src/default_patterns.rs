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
}
