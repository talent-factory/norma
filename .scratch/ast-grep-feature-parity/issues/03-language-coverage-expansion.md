Type: grilling

## Question

Welche der von `ast-grep-language` 0.45.3 bereits mitgelieferten Sprachen (über die MVP-4 hinaus) sollte norma freischalten, und nach welchem Mechanismus?

**Befund:** `SupportLang` (`ast-grep-language` 0.45.3, `src/lib.rs:259`) hat 28 Varianten: Bash, C, Cpp, CSharp, Css, Dart, Go, Elixir, Haskell, Hcl, Html, Java, JavaScript, Json, Kotlin, Lua, Markdown, Nix, Php, Python, Ruby, Rust, Scala, Solidity, Swift, Tsx, TypeScript, Yaml. `default = ["builtin-parser"]` in dessen `Cargo.toml` kompiliert **alle** davon ein; norma's eigenes `Cargo.toml` (`ast-grep-language = "=0.45.3"`) deaktiviert keine Features — alle 28 Tree-Sitter-Grammatiken sind schon im norma-Binary vorhanden, unabhängig von dieser Entscheidung. `pattern_engine::language_key` (`src/pattern_engine.rs:44`) hat aber ein hartes `_ => None` und lässt nur die MVP-4 (Java/Python/Rust/TypeScript) durch — reine Anwendungsschicht-Beschränkung, kein technisches Hindernis, kein zusätzlicher Dependency- oder Binary-Size-Preis.

Zu klären:
- Alle 28 freischalten, eine kuratierte Zusatzliste (welche?), oder MVP-4 bewusst so belassen (Issue 05 der alten Map war eine bewusste Scope-Entscheidung — gilt die Begründung noch)?
- Falls Erweiterung: braucht jede neue Sprache eigene Default-Patterns (`default_patterns.rs`), oder bleibt sie zunächst "nur registrierbar" ohne mitgelieferte Patterns?
- SQLite-Schema/CLI-Validierung (`SUPPORTED_LANGUAGES`-Konstante) mitziehen.
