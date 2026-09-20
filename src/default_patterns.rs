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
