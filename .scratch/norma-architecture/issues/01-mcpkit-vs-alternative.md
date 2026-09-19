Type: research

## Question

Ist `mcpkit` das richtige MCP-Framework-Fundament für norma, oder ist eine Alternative (z. B. das offizielle Rust-MCP-SDK `rmcp`) besser geeignet?

Kontext: `mcpkit` ist ein echtes, publiziertes Crate (`mcpkit-core`, `-macros`, `-server`, `-transport`, `-client` — siehe `Cargo.lock`), aber der vorhandene Scaffold kompiliert nicht (4 Fehler: falsche Crate-Root-Referenzen `mcpkit_server`/`mcpkit_core`, fehlendes `#[serde(...)]`-Derive). Zu klären:

- Wie sieht die tatsächliche, korrekte API von `mcpkit` aus (welche Crates müssen direkt als Dependency deklariert werden, wie wird ein MCP-Tool-Server korrekt aufgesetzt)?
- Wie reif/aktiv gepflegt ist `mcpkit` (Version, Doku-Qualität, letzte Releases, Community)?
- Wie sieht die Alternative `rmcp` (offizielles Rust-MCP-SDK) im Vergleich aus (API-Ergonomie, Reife, Doku)?
- Empfehlung: bei `mcpkit` bleiben (und Scaffold korrigieren) oder wechseln?
