# norma — Architecture Spec (wayfinder map)

## Destination

Eine Architektur-Spezifikation für norma (Rust MCP Server zur Design-Pattern-Validierung, FFHS-Lehre): verbindliche Entscheidungen zu MCP-Framework, Pattern-Matching-Ansatz, MVP-Sprachen/Pattern-Scope und CLI/Pre-Commit-Architektur. Kein Code — die eigentliche Umsetzung folgt danach in einer separaten Session/einem separaten Effort.

## Notes

- **Zweck:** norma soll zuerst als funktionierendes Tool für Daniels eigene FFHS-Lehrtätigkeit dienen (Pattern-Validierung von Studierendencode), danach als Ausbau-Vorlage für Studierende selbst.
- **MVP-Sprachen:** Java, Python, Rust, TypeScript.
- **Standing preferences:** Code-Kommentare pädagogisch klar halten (Englisch, da Rust-Ökosystem-Konvention und spätere GitHub-Veröffentlichung unter `talent-factory/norma`); Dependencies bewusst schlank halten; Chat-Kommunikation auf Deutsch; `ast-grep-core` versionsgepinnt halten (kein automatisches Bumpen auf neue Minor-Versionen ohne bewussten Test — Rust-API ist offiziell "not stable yet").
- **Skills:** `/grilling` und `/domain-modeling` für jede Ticket-Session nutzen.
- **Ausgangslage:** Ein per Claude Desktop erzeugter Scaffold (`norma-scaffold.zip`) wurde bereits ins Repo entpackt (`src/`, `Cargo.toml`, `README.md`, `DEVELOPMENT.md`, `QUICKSTART.md`, `LICENSE-MIT`, `rust-toolchain.toml`). `cargo build` schlägt aktuell mit 4 Fehlern fehl — `mcpkit` ist ein echtes, publiziertes Crate (nicht halluziniert; `mcpkit-core`/`-macros`/`-server`/`-transport`/`-client` stehen im `Cargo.lock`), aber `Cargo.toml` deklariert nur `mcpkit` direkt, und `mcp_server.rs` referenziert `mcpkit_server`/`mcpkit_core` als Crate-Root sowie ein `#[serde(...)]`-Attribut ohne nötigen Derive-Import. Der Scaffold wurde nie gebaut, bevor er übergeben wurde.

## Decisions so far

- [mcpkit vs. Alternative (research)](issues/01-mcpkit-vs-alternative.md) — Wechsel zu `rmcp` (offizielles MCP-Rust-SDK); `mcpkit` bleibt wegen geringer Reife/Aktivität und struktureller Facade-Crate-Probleme (Build-Fehler nicht trivial fixbar) nicht das Fundament.
- [Pattern-Matching-Ansatz (grilling)](issues/02-pattern-matching-approach.md) — `ast-grep-core` (aktiv gepflegt, alle 4 MVP-Sprachen abgedeckt); Pattern-`rule`-Feld einheitlich im YAML-Rule-Format gespeichert.
- [CLI/Pre-Commit-Architektur (grilling)](issues/03-cli-pre-commit-architecture.md) — ein Binary mit `clap`-Subcommands, geteilter async Core mit dem MCP-Server, Pre-Commit via `language: system`; siehe [ADR 0001](../../docs/adr/0001-single-binary-shared-validation-core.md).
- [Pattern-Datenmodell & SQLite-Schema (grilling)](issues/04-pattern-data-model-schema.md) — breiter Scope (nicht nur GoF-Patterns), `Pattern` ist einsprachig (kein Join-Table), `rule` speichert die volle ast-grep-RuleConfig-YAML, Fail-Fast-Validierung beim Registrieren. Verifiziert per Probe gegen echtes `ast-grep-core`; siehe [ADR 0002](../../docs/adr/0002-pattern-single-language-full-rule-config.md).
- [MVP-Pattern-Set-Scope (grilling)](issues/05-mvp-pattern-set-scope.md) — 1 Pattern pro Sprache (4 total), gemeinsames Konzept "kein Debug-Print" über Java/Python/Rust/TypeScript, kein GoF-Pattern in v1, Rust-Variante dient als Dogfooding-Demo gegen norma's eigenen `src/`-Ordner (aktuell 0 Treffer). Alle 4 Rules verifiziert.

## Not yet specified

- Teststrategie (Unit/Integration/E2E) für die Spec-Phase

## Out of scope

- Web-UI für Pattern-Management
- Pattern-Marketplace
- crates.io-Publishing
- Teaching-Template-Repackaging für Studierende (separater, späterer Effort — erst nachdem das persönliche MVP funktioniert)
