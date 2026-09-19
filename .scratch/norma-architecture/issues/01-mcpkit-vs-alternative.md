Type: research
Status: resolved

## Question

Ist `mcpkit` das richtige MCP-Framework-Fundament für norma, oder ist eine Alternative (z. B. das offizielle Rust-MCP-SDK `rmcp`) besser geeignet?

Kontext: `mcpkit` ist ein echtes, publiziertes Crate (`mcpkit-core`, `-macros`, `-server`, `-transport`, `-client` — siehe `Cargo.lock`), aber der vorhandene Scaffold kompiliert nicht (4 Fehler: falsche Crate-Root-Referenzen `mcpkit_server`/`mcpkit_core`, fehlendes `#[serde(...)]`-Derive). Zu klären:

- Wie sieht die tatsächliche, korrekte API von `mcpkit` aus (welche Crates müssen direkt als Dependency deklariert werden, wie wird ein MCP-Tool-Server korrekt aufgesetzt)?
- Wie reif/aktiv gepflegt ist `mcpkit` (Version, Doku-Qualität, letzte Releases, Community)?
- Wie sieht die Alternative `rmcp` (offizielles Rust-MCP-SDK) im Vergleich aus (API-Ergonomie, Reife, Doku)?
- Empfehlung: bei `mcpkit` bleiben (und Scaffold korrigieren) oder wechseln?

## Answer

Wechsel zu `rmcp` (offizielles Rust-SDK der Model-Context-Protocol-Org, https://github.com/modelcontextprotocol/rust-sdk). `mcpkit` ist zwar ein echtes Crate, aber ein Ein-Personen-Kleinprojekt (6 GitHub-Stars, letzter Push ca. 7 Wochen vor Recherchedatum, 4'128 Downloads insgesamt), dessen Facade-Crate-Architektur den reproduzierten Build-Fehler strukturell verursacht: das `#[mcp_server(...)]`-Makro generiert Code, der `mcpkit_core`/`mcpkit_server` als direkte Crate-Namen referenziert — die aber nur über `mcpkit` transitiv, nicht direkt in `Cargo.toml` deklariert sind (Rusts Extern-Prelude sieht nur direkte Deps). Zusätzlich nutzt der Scaffold mit `#[serde(default)]` auf Funktionsparametern eine Syntax, die in keinem offiziellen `mcpkit`-Beispiel vorkommt — vermutlich vom scaffold-generierenden Modell erfunden, ein Symptom der geringen Verbreitung/Dokumentationsdichte von `mcpkit`. `rmcp` dagegen ist offiziell von der MCP-Org gepflegt, sehr aktiv (Push gestern, 3'938 Stars, 27.6 Mio. Downloads, Version 3.4.0 vor 4 Tagen released), unterstützt stdio-Tool-Server nativ und gut belegt (`#[tool_router]`/`#[tool]`-Makros, `ServiceExt::serve(stdio())`), und ist für ein später öffentlich als Lehrvorlage veröffentlichtes Projekt (`talent-factory/norma`) die langfristig robustere Wahl. Volle Quellenlage inkl. Zitaten: [01-mcpkit-vs-alternative-research.md](01-mcpkit-vs-alternative-research.md).
