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
        // main.rs is the CLI entry point whose println! calls are its intended,
        // spec-mandated stdout output (see ADR 0001 and the CLI/pre-commit
        // ticket), not a debug print the "no debug print" pattern is meant to
        // catch -- every other file in src/ is library code and must stay clean.
        if path.file_name().and_then(|n| n.to_str()) == Some("main.rs") {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let code = fs::read_to_string(&path).unwrap();
        let result = validate(&code, "rust", std::slice::from_ref(&pattern)).unwrap();
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
