# norma — Architecture Spec (wayfinder map)

## Destination

Eine Architektur-Spezifikation für norma (Rust MCP Server zur Design-Pattern-Validierung, FFHS-Lehre): verbindliche Entscheidungen zu MCP-Framework, Pattern-Matching-Ansatz, MVP-Sprachen/Pattern-Scope und CLI/Pre-Commit-Architektur. Kein Code — die eigentliche Umsetzung folgt danach in einer separaten Session/einem separaten Effort.

## Notes

- **Zweck:** norma soll zuerst als funktionierendes Tool für Daniels eigene FFHS-Lehrtätigkeit dienen (Pattern-Validierung von Studierendencode), danach als Ausbau-Vorlage für Studierende selbst.
- **MVP-Sprachen:** Java, Python, Rust, TypeScript.
- **Standing preferences:** Code-Kommentare pädagogisch klar halten (Englisch, da Rust-Ökosystem-Konvention und spätere GitHub-Veröffentlichung unter `talent-factory/norma`); Dependencies bewusst schlank halten; Chat-Kommunikation auf Deutsch.
- **Skills:** `/grilling` und `/domain-modeling` für jede Ticket-Session nutzen.
- **Ausgangslage:** Ein per Claude Desktop erzeugter Scaffold (`norma-scaffold.zip`) wurde bereits ins Repo entpackt (`src/`, `Cargo.toml`, `README.md`, `DEVELOPMENT.md`, `QUICKSTART.md`, `LICENSE-MIT`, `rust-toolchain.toml`). `cargo build` schlägt aktuell mit 4 Fehlern fehl — `mcpkit` ist ein echtes, publiziertes Crate (nicht halluziniert; `mcpkit-core`/`-macros`/`-server`/`-transport`/`-client` stehen im `Cargo.lock`), aber `Cargo.toml` deklariert nur `mcpkit` direkt, und `mcp_server.rs` referenziert `mcpkit_server`/`mcpkit_core` als Crate-Root sowie ein `#[serde(...)]`-Attribut ohne nötigen Derive-Import. Der Scaffold wurde nie gebaut, bevor er übergeben wurde.

## Decisions so far

- [mcpkit vs. Alternative (research)](issues/01-mcpkit-vs-alternative.md) — Wechsel zu `rmcp` (offizielles MCP-Rust-SDK); `mcpkit` bleibt wegen geringer Reife/Aktivität und struktureller Facade-Crate-Probleme (Build-Fehler nicht trivial fixbar) nicht das Fundament.

## Not yet specified

- Pattern-Datenmodell/Schema — abhängig vom Ergebnis von [Pattern-Matching-Ansatz](issues/02-pattern-matching-approach.md)
- SQLite-Pattern-Registry-Schema-Anpassungen — abhängig vom obigen
- Konkretes Pattern-Set pro Sprache (Java/Python/Rust/TypeScript) fürs MVP — abhängig vom Pattern-Matching-Ansatz
- Teststrategie (Unit/Integration/E2E) für die Spec-Phase

## Out of scope

- Web-UI für Pattern-Management
- Pattern-Marketplace
- crates.io-Publishing
- Teaching-Template-Repackaging für Studierende (separater, späterer Effort — erst nachdem das persönliche MVP funktioniert)
