Type: grilling
Blocked by: 01
Status: resolved

## Question

Sollen CLI/Pre-Commit-Hook und MCP-Server dasselbe Binary mit Subcommands teilen (z. B. `norma serve` vs. `norma validate --file ...`), oder getrennte Binaries/Crates sein?

Kontext: Pre-Commit-Hook/CLI gehören laut Zweck-Entscheidung zur Destination dieser Spec (nicht Out of scope). Die Antwort hängt am gewählten MCP-Framework aus [mcpkit vs. Alternative](01-mcpkit-vs-alternative.md), weil das Transport-/Entry-Point-Setup (stdio-MCP-Server vs. CLI-Invocation) je nach Framework unterschiedlich verdrahtet wird. Zu klären:

- Ein Binary mit Subcommands (norma serve / norma validate / norma check-patterns) oder separate Binaries/Crates in einem Workspace?
- Teilen sich CLI und MCP-Server denselben Validierungs-Kern (pattern_engine/pattern_store) 1:1, oder braucht die CLI-Variante eigene Anpassungen (z. B. synchron statt async, andere Output-Formate für Pre-Commit)?
- Wie sieht die `.pre-commit-config.yaml`-Integration konkret aus?

## Answer

**Ein Binary mit `clap`-Subcommands** (`norma serve` für den MCP-Server via `rmcp`/stdio, `norma validate --file ... --lang ...` für CLI/Pre-Commit, ggf. `norma list-patterns`) statt eines Cargo-Workspace mit getrennten Crates — einfacher zu bauen, passt zur Standing Preference "Dependencies schlank halten" und ist als spätere Lehr-Vorlage leichter durchschaubar. Festgehalten als [ADR 0001](../../../docs/adr/0001-single-binary-shared-validation-core.md).

**Geteilter async Core:** `norma validate` ruft dieselben async-Funktionen aus `pattern_engine`/`pattern_store` wie der MCP-Server auf (über `#[tokio::main]`, kein zweiter synchroner Pfad) — keine Duplikation der Validierungslogik.

**Pre-Commit-Hook-Typ:** `.pre-commit-config.yaml` referenziert das bereits gebaute `norma`-Binary auf dem `PATH` (`language: system`), setzt also voraus, dass Studierende `norma` einmal installieren (z. B. `cargo install --path .`) statt es bei jedem Pre-Commit-Lauf neu zu bauen.

**CLI-Output/Exit-Code (v1):** `norma validate` gibt standardmässig menschenlesbaren Text aus, mit optionalem `--json`-Flag für CI/Tooling; Exit-Code ≠ 0, sobald mindestens eine Violation gefunden wurde. Feinere Steuerung (z. B. Exit-Code nur ab bestimmtem Severity-Level) ist bewusst auf später verschoben, keine v1-Entscheidung.
