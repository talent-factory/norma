Type: grilling
Status: claimed

## Question

Welcher Pattern-Matching-Ansatz für norma: `ast-grep-core` vs. rohes `tree-sitter` (bereits Dependency) vs. "Regex bleibt für v1, AST erst v2"?

Kontext: DEVELOPMENT.md priorisiert eine AST-Grep-Integration als "High priority", der aktuelle Scaffold nutzt aber noch Regex (`src/pattern_engine.rs`, 113 Zeilen). `tree-sitter` ist bereits als direkte Dependency in `Cargo.toml` eingetragen; `ast-grep-core` baut selbst auf tree-sitter-Parsern auf. Zu klären:

- Reicht `ast-grep-core`s eigenes Pattern-/Rule-Format (inkl. Multi-Language-Support für Java/Python/Rust/TypeScript), oder ist rohes tree-sitter-Query-Matching flexibler/nötig?
- Wie hoch ist der Integrationsaufwand für jeden Ansatz, realistisch für ein MVP?
- Bleibt Regex als Übergangslösung für v1 vertretbar, oder ist der Sprung direkt zu AST-Matching nötig, weil Regex für Java/Python/Rust/TypeScript-Patterns zu unzuverlässig ist?
- Ergebnis prägt direkt das Pattern-Datenmodell/-Schema (siehe „Not yet specified" auf der Map).
