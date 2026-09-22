Type: grilling
Status: resolved

## Question

Welche der von `ast-grep-language` 0.45.3 bereits mitgelieferten Sprachen (über die MVP-4 hinaus) sollte norma freischalten, und nach welchem Mechanismus?

**Befund:** `SupportLang` (`ast-grep-language` 0.45.3, `src/lib.rs:259`) hat 28 Varianten: Bash, C, Cpp, CSharp, Css, Dart, Go, Elixir, Haskell, Hcl, Html, Java, JavaScript, Json, Kotlin, Lua, Markdown, Nix, Php, Python, Ruby, Rust, Scala, Solidity, Swift, Tsx, TypeScript, Yaml. `default = ["builtin-parser"]` in dessen `Cargo.toml` kompiliert **alle** davon ein; norma's eigenes `Cargo.toml` (`ast-grep-language = "=0.45.3"`) deaktiviert keine Features — alle 28 Tree-Sitter-Grammatiken sind schon im norma-Binary vorhanden, unabhängig von dieser Entscheidung. `pattern_engine::language_key` (`src/pattern_engine.rs:44`) hat aber ein hartes `_ => None` und lässt nur die MVP-4 (Java/Python/Rust/TypeScript) durch — reine Anwendungsschicht-Beschränkung, kein technisches Hindernis, kein zusätzlicher Dependency- oder Binary-Size-Preis.

Zu klären:
- Alle 28 freischalten, eine kuratierte Zusatzliste (welche?), oder MVP-4 bewusst so belassen (Issue 05 der alten Map war eine bewusste Scope-Entscheidung — gilt die Begründung noch)?
- Falls Erweiterung: braucht jede neue Sprache eigene Default-Patterns (`default_patterns.rs`), oder bleibt sie zunächst "nur registrierbar" ohne mitgelieferte Patterns?
- SQLite-Schema/CLI-Validierung (`SUPPORTED_LANGUAGES`-Konstante) mitziehen.

## Answer

**Verdict: adopt.** Alle 28 `SupportLang`-Sprachen werden generisch für die Registrierung freigeschaltet — keine kuratierte Zusatzliste, MVP-4 wird nicht künstlich beibehalten.

1. **Umfang**: alle 28 statt einer Teilmenge — technisch kostenlos (schon einkompiliert über das Default-Feature `builtin-parser`), vermeidet beliebiges Bikeshedding, welche Sprachen "es wert sind".
2. **Default-Patterns**: die 24 neu freigeschalteten Sprachen bleiben **nur registrierbar**, ohne eingebaute Default-Patterns. Pattern-Autoring pro Sprache ist eigene, nicht-triviale Arbeit (vgl. GoF-Pattern-Set) und damit expliziter, separater Folge-Effort — nicht Teil dieses mechanischen Tickets. Die pädagogische Fokussierung der alten Map bleibt so über die *mitgelieferten* Patterns gewahrt, nicht über eine künstliche Registrierungssperre.
3. **Implementierung**: `language_key`/`SUPPORTED_LANGUAGES` werden **generisch aus `SupportLang::all_langs()` abgeleitet** statt hart um 28 Strings erweitert — vermeidet Drift bei künftigen `ast-grep-language`-Versionsbumps. `resolve_language`s Sicherheitseigenschaft (unbekannte/falsch geschriebene Sprache wird laut abgelehnt statt still 0 Patterns zu matchen) bleibt vollständig erhalten, da `SupportLang::from_str` weiterhin alles ausserhalb der 28 bekannten Sprachen ablehnt.

→ Graduiert zu [TF-893](https://linear.app/talent-factory/issue/TF-893) im `norma`-Projekt.
