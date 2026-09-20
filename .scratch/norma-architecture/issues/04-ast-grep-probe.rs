use ast_grep_config::{from_yaml_string, GlobalRules};
use ast_grep_language::{LanguageExt, SupportLang};

fn try_rule(label: &str, yaml: &str, source: &str) {
    let globals = GlobalRules::default();
    let configs = from_yaml_string::<SupportLang>(yaml, &globals).expect("rule should parse");
    let config = &configs[0];
    let grep = config.language.ast_grep(source);
    match grep.root().find(&config.matcher) {
        Some(m) => println!("[{label}] MATCH: {:?}", m.text()),
        None => println!("[{label}] no match"),
    }
}

fn main() {
    debug_print_set();
    // TypeScript: forbid console.log(...)
    try_rule(
        "ts-console-log",
        r#"
id: ts-console-log
message: Remove console.log
severity: warning
language: TypeScript
rule:
  pattern: console.log($$$ARGS)
"#,
        "function greet() { console.log('hi'); }",
    );

    // Java: same *concept*, different language + different rule text
    try_rule(
        "java-static-import",
        r#"
id: java-static-import
message: Avoid wildcard static imports
severity: warning
language: Java
rule:
  pattern: import static $PKG.*;
"#,
        "import static java.lang.Math.*;\nclass Foo {}",
    );

    // Rust: dogfooding check, e.g. forbid unwrap()
    try_rule(
        "rust-no-unwrap",
        r#"
id: rust-no-unwrap
message: Avoid unwrap()
severity: warning
language: Rust
rule:
  pattern: $EXPR.unwrap()
"#,
        "fn main() { let x = Some(1).unwrap(); }",
    );

    // Python
    try_rule(
        "python-print",
        r#"
id: python-print
message: Avoid print() in library code
severity: info
language: Python
rule:
  pattern: print($$$ARGS)
"#,
        "def f():\n    print('debug')\n",
    );

    // Negative case: verify a non-matching source correctly reports no match
    try_rule(
        "ts-console-log-negative",
        r#"
id: ts-console-log
message: Remove console.log
severity: warning
language: TypeScript
rule:
  pattern: console.log($$$ARGS)
"#,
        "function greet() { logger.info('hi'); }",
    );
}

fn debug_print_set() {
    try_rule(
        "java-debug-print",
        r#"
id: no-debug-print-java
message: Avoid System.out.println in production code
severity: warning
language: Java
rule:
  pattern: System.out.println($$$ARGS)
"#,
        "class Foo { void bar() { System.out.println(\"debug\"); } }",
    );
    try_rule(
        "rust-debug-print",
        r#"
id: no-debug-print-rust
message: Avoid println! in production code
severity: warning
language: Rust
rule:
  pattern: println!($$$ARGS)
"#,
        "fn main() { println!(\"debug\"); }",
    );
    try_rule(
        "rust-debug-print-negative-tracing",
        r#"
id: no-debug-print-rust
message: Avoid println! in production code
severity: warning
language: Rust
rule:
  pattern: println!($$$ARGS)
"#,
        "fn main() { tracing::info!(\"structured, fine\"); }",
    );
}
