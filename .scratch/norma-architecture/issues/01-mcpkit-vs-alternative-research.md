# Research: mcpkit vs. rmcp als MCP-Framework-Fundament für norma

Datum: 2026-09-19
Research für: [01-mcpkit-vs-alternative.md](01-mcpkit-vs-alternative.md)

## Zusammenfassung / Empfehlung

**Wechsel von `mcpkit` zu `rmcp`** (offizielles Rust-SDK der Model Context Protocol Org, https://github.com/modelcontextprotocol/rust-sdk).

Begründung in Kürze:

1. Der Build-Fehler im Scaffold ist zwar mit `mcpkit` technisch reparierbar (siehe Abschnitt "Falls doch mcpkit" unten), aber er entsteht aus einer echten strukturellen Schwäche der Facade-Crate-Architektur von `mcpkit` 0.1.0, nicht aus einem trivialen Tippfehler: die von `#[mcp_server(...)]` generierten Makro-Pfade referenzieren `mcpkit_core`/`mcpkit_server` als **direkte** Crate-Namen im Extern-Prelude des aufrufenden Crates — das funktioniert nur, wenn `mcpkit-core` und `mcpkit-server` zusätzlich zu `mcpkit` selbst **direkt** in `Cargo.toml` deklariert werden (Rusts Extern-Prelude enthält nur direkte Dependencies, keine transitiven). Das ist mit lokal reproduziertem `cargo build`-Output bestätigt (siehe unten).
2. Reifegrad/Aktivität: `mcpkit` ist ein Ein-Personen-/Kleinprojekt (6 GitHub-Stars, 2 Forks, Repo erstellt 2025-12-11, letzter Push 2026-07-27 — zum Recherchezeitpunkt 19.09.2026 also ca. 7 Wochen ohne Aktivität), 4'128 Downloads insgesamt auf crates.io. `rmcp` ist das offizielle SDK der `modelcontextprotocol`-GitHub-Org, 3'938 Stars, 645 Forks, letzter Push **gestern** (2026-09-18), 27.6 Mio. Downloads auf crates.io, aktuellste Version 3.4.0 (veröffentlicht 2026-09-15, vor 4 Tagen).
3. Die Scaffold-Autorin (Claude Desktop) hat bei `mcpkit` bereits eine Parameter-Attribut-Syntax (`#[serde(default)]` direkt auf Funktionsparametern) verwendet, die in keinem der offiziellen `mcpkit`-Beispiele vorkommt — ein Hinweis, dass selbst das generierende Modell die reale API von `mcpkit` nicht zuverlässig kennt (naheliegend bei einem Nischen-Crate mit wenig Trainingsdaten/Dokumentation im Vergleich zum viel prominenteren offiziellen SDK).
4. Pädagogischer Kontext (norma wird später auf GitHub unter `talent-factory/norma` veröffentlicht und als Lehr-Vorlage für Studierende dienen, siehe `map.md`): Abhängigkeit vom offiziell von der MCP-Org gepflegten SDK ist für ein Lehrprojekt die sicherere, langfristig stabilere Wahl als ein wenig verbreitetes Solo-Crate.
5. `rmcp` unterstützt stdio-Transport nativ und explizit für Tool-Server (`rmcp::transport::stdio`, `ServiceExt::serve(stdio())`), genau das, was norma als MCP-Tool-Server braucht.

Gewichtiger Gegenpunkt (für Vollständigkeit): `mcpkit`s eigene Doku wirbt mit "66% weniger Boilerplate" und einem einzigen vereinheitlichten `#[mcp_server]`-Makro gegenüber `rmcp`s mehreren, voneinander abhängigen Makros — das stimmt ergonomisch für den Erfolgsfall, aber die geringe Reife/Aktivität und der bereits erlebte Build-Fehler wiegen für ein Projekt, das stabil und für Dritte (Studierende) nachvollziehbar sein soll, schwerer als der Boilerplate-Vorteil.

## Details pro Quelle

### 1. `mcpkit` auf crates.io

- Existiert real (nicht halluziniert). Crate-Seite: https://crates.io/crates/mcpkit
- 12 veröffentlichte Versionen: 0.1.0, 0.2.0–0.2.5, 0.3.0, 0.4.0, 0.5.0, 0.6.0, 0.7.0 (aktuellste).
- Beschreibung (neueste Version): "Rust SDK for the Model Context Protocol (MCP) - the official facade crate providing unified access to all mcpkit functionality". Frühere Versionen (0.1.0–0.2.3): "Rust SDK for the Model Context Protocol (MCP)".
- Autor/Owner: GitHub-User `jkindrix` (Justin Kindrix); Repository unter der GitHub-Org `praxiomlabs` (frühe 0.1.0-Metadaten zeigten noch einen Tippfehler `praxislabs`).
- Insgesamt 4'128 Downloads (alle Versionen).
- Lizenz: MIT OR Apache-2.0.
- Quelle: WebFetch von https://crates.io/api/v1/crates/mcpkit, 2026-09-19.

### 2. `mcpkit` 0.1.0 auf docs.rs (die im Cargo.lock des Scaffolds gepinnte Version)

Quelle: https://docs.rs/mcpkit/0.1.0/mcpkit/ (Rohtext lokal via `curl` extrahiert, 2026-09-19).

Zitat (Crate-Doku-Kopf):

> "A production-grade Rust SDK for the Model Context Protocol that dramatically reduces boilerplate compared to rmcp through a unified #[mcp_server] macro."

Publikationsdatum laut docs.rs-Metadaten: **11 December 2025**.

Deklarierte Dependencies dieser Version (direkt von der docs.rs-Metadatenseite):

```
mcpkit-client    ^0.1.0  normal
mcpkit-core      ^0.1.0  normal
mcpkit-macros    ^0.1.0  normal
mcpkit-server    ^0.1.0  normal
mcpkit-transport ^0.1.0  normal
```

Quick-Start-Beispiel aus der Doku (Zitat):

```rust
use mcpkit::prelude::*;
struct Calculator;
#[mcp_server(name = "calculator", version = "1.0.0")]
impl Calculator {
    #[tool(description = "Add two numbers")]
    async fn add(&self, a: f64, b: f64) -> ToolOutput {
        ToolOutput::text((a + b).to_string())
    }
    #[tool(description = "Multiply two numbers")]
    async fn multiply(&self, a: f64, b: f64) -> ToolOutput {
        ToolOutput::text((a * b).to_string())
    }
}
#[tokio::main]
async fn main() -> Result<(), McpError> {
    Calculator.serve_stdio().await
}
```

Wichtig: **In keinem offiziellen Beispiel wird ein `#[serde(...)]`-Attribut direkt auf einem Funktionsparameter verwendet** — optionale Parameter werden in der 0.1.0-Doku nicht demonstriert. Das im norma-Scaffold verwendete Muster (`#[serde(default)] file_path: Option<String>` als Parameterattribut) ist damit vermutlich erfundene/nicht belegte Syntax und keine reale `mcpkit`-API — das erklärt die beiden `serde`-Attribut-Fehler unabhängig vom Crate-Root-Problem.

Öffentlich dokumentierte Modul-Struktur der Facade (0.1.0): `auth`, `capability`, `client`, `error`, `prelude`, `protocol`, `schema`, `server`, `state`, `transport`, `types` — die Facade re-exportiert Funktionalität **unter eigenen Modulnamen** (`client`, `server`, `transport`), **nicht** unter den ursprünglichen Sub-Crate-Namen (`mcpkit_client`, `mcpkit_server`, `mcpkit_transport`). D.h. Code, der `mcpkit_server::...` oder `mcpkit_core::...` referenziert, funktioniert so oder so nicht direkt über die Facade — genau dieses Muster hat aber das `#[mcp_server(...)]`-Makro selbst intern generiert (siehe Abschnitt "Reproduzierter Build-Fehler" unten), nicht der von Menschen geschriebene Code.

### 3. `mcpkit` auf GitHub (praxiomlabs/mcpkit)

Quelle: `curl https://api.github.com/repos/praxiomlabs/mcpkit`, 2026-09-19 (GitHub REST API, primäre Quelle):

```
full_name:          praxiomlabs/mcpkit
stargazers_count:   6
forks_count:        2
open_issues_count:  0
created_at:         2025-12-11T16:50:11Z
pushed_at:           2026-07-27T17:59:02Z   (letzter Push, ~7 Wochen vor Recherchedatum)
updated_at:          2026-09-02T05:39:43Z
owner.type:          Organization  (praxiomlabs — nach Vereinsregister/Impressum nicht verifizierbar, wirkt wie persönliches/kleines Projekt eines einzelnen Autors)
description: "A Rust SDK for the Model Context Protocol (MCP) that reduces boilerplate through a unified `#[mcp_server]` macro."
```

Ergänzend (via WebFetch der Repo-Startseite, 2026-09-19, sekundär zur API bestätigend): 295 Commits, README wirbt mit Vergleichstabelle "rmcp vs. mcpkit" (u. a. "4 interdependent Macros" vs. "1 unified #[mcp_server]", "3 nested error layers" vs. "1 unified McpError").

### 4. `rmcp` auf crates.io

Quelle: WebFetch https://crates.io/api/v1/crates/rmcp, 2026-09-19.

- Existiert, 65 veröffentlichte Versionen.
- Repository: https://github.com/modelcontextprotocol/rust-sdk/
- Lizenz: MIT OR Apache-2.0 (docs.rs-Seite) bzw. laut GitHub-API-Feld `license.spdx_id: NOASSERTION` — GitHub konnte die Lizenzdatei nicht eindeutig automatisch klassifizieren, das MIT/Apache-2.0-Dual-Licensing steht aber explizit auf der Crate-Doku-Seite.
- Aktuellste Version: **3.4.0**, veröffentlicht 2026-09-15 (4 Tage vor Recherchedatum).
- Downloads insgesamt: 27'625'137.

### 5. `rmcp` auf GitHub (modelcontextprotocol/rust-sdk)

Quelle: `curl https://api.github.com/repos/modelcontextprotocol/rust-sdk`, 2026-09-19 (primäre Quelle, GitHub REST API):

```
full_name:          modelcontextprotocol/rust-sdk
stargazers_count:   3942
forks_count:        645
open_issues_count:  54
created_at:         2025-02-18T10:55:26Z
pushed_at:           2026-09-18T18:02:49Z   (letzter Push: gestern)
updated_at:          2026-09-19T15:49:21Z
description:         "The official Rust SDK for the Model Context Protocol"
```

Offiziell unter der GitHub-Org `modelcontextprotocol` — derselben Org, die das MCP-Protokoll selbst spezifiziert (https://github.com/modelcontextprotocol).

### 6. `rmcp` API-Ergonomie / stdio-Tool-Server-Beispiel

Quelle: docs.rs https://docs.rs/rmcp (Version 3.4.0) und GitHub-Beispieldateien im offiziellen Repo:
- https://github.com/modelcontextprotocol/rust-sdk/blob/main/examples/servers/src/common/calculator.rs
- https://github.com/modelcontextprotocol/rust-sdk/blob/main/examples/servers/src/calculator_stdio.rs

Dokumentations-Coverage laut docs.rs-Metadaten (2026-09-19): 49.04 % (1100 von 2243 Items dokumentiert, 31 von 932 Items mit Beispielen) — niedriger als `mcpkit`s beworbene "100 % dokumentiert", aber `rmcp` deckt auch deutlich mehr Fläche ab (Client, Server, Auth/OAuth, mehrere Transports, Makro-Crate) und die Kern-Patterns (Tool-Server, stdio) sind in den offiziellen `examples/` durchgängig vorhanden und aktuell gehalten.

Belegtes Server-Pattern (paraphrasiert, Original einsehbar unter obigen URLs, da Volltext-Zitation über Zeichenlimit hinausging):
- Parameter-Structs pro Tool (`SumRequest { a: i32, b: i32 }`) mit `#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]` und `#[schemars(description = "...")]` pro Feld.
- `impl`-Block mit `#[tool_router(server_handler)]`, einzelne Methoden mit `#[tool(description = "...")]`, Parameter über `Parameters<SumRequest>` typisiert.
- `main.rs`: `Calculator.serve(stdio()).await` gefolgt von `service.waiting().await?` — stdio-Transport ist first-class und exakt das Muster, das norma als lokaler MCP-Tool-Server braucht (`rmcp::transport::stdio`, `rmcp::ServiceExt`).
- Installation: `cargo add rmcp --features server` (Feature-Flag-gesteuert, kein Facade-Crate-Problem wie bei `mcpkit`, da alles in einem Crate `rmcp` + optionalem `rmcp-macros` liegt und über Cargo-Features statt über mehrere Sub-Crate-Namen gesteuert wird).

### 7. Reproduzierter Build-Fehler im norma-Scaffold (primäre Quelle: lokaler `cargo build`)

Ausgeführt am 2026-09-19 im Repo-Root (`cd /Users/daniel/GitRepository/norma && cargo build`), 4 Fehler:

```
error: cannot find attribute `serde` in this scope
   --> src/mcp_server.rs:156:11  (#[serde(default)] auf Parameter `file_path`)
error: cannot find attribute `serde` in this scope
   --> src/mcp_server.rs:175:11  (#[serde(default)] auf Parameter `severity`)
error[E0433]: cannot find `mcpkit_server` in the crate root
   --> src/mcp_server.rs:149:1  (im expandierten #[mcp_server(...)]-Makro)
error[E0433]: cannot find `mcpkit_core` in the crate root
   --> src/mcp_server.rs:149:1  (im expandierten #[mcp_server(...)]-Makro)
```

Interpretation:
- Die beiden `serde`-Fehler bestätigen den Verdacht aus Abschnitt 2: `#[serde(default)]` ist keine unterstützte Parameter-Attribut-Syntax von `mcpkit`s `#[tool(...)]`-Makro und muss so oder so entfernt/anders gelöst werden (z. B. `Option<T>`-Parameter ohne das Attribut, falls `mcpkit`s Makro `Option<T>` automatisch als optional erkennt — das ist aus der 0.1.0-Doku nicht belegbar, da kein Beispiel mit optionalen Parametern existiert).
- Die beiden `E0433`-Fehler entstehen **nicht** aus explizitem Code in `mcp_server.rs` (dort steht nur `use mcpkit::prelude::*;`), sondern aus dem von `#[mcp_server(name = "norma", version = "0.1.0")]` erzeugten Makro-Output, der intern `mcpkit_server::`/`mcpkit_core::`-Pfade referenziert. Das ist ein bekanntes Rust-Muster bei Facade-Crates: Von Proc-Macros generierter Code liegt im *aufrufenden* Crate und wird dort nach dessen Extern-Prelude aufgelöst — transitive Dependencies (hier: `mcpkit-core`/`mcpkit-server` als Deps von `mcpkit`) zählen dafür nicht, nur direkte Dependencies von `norma`s eigenem `Cargo.toml`.

## Falls doch `mcpkit` (Fix-Anleitung für den Fall eines Verbleibs)

Nicht empfohlen (siehe Zusammenfassung), aber der Vollständigkeit halber dokumentiert:

1. In `Cargo.toml` zusätzlich zu `mcpkit = "0.1"` die von der Facade genutzten Sub-Crates **direkt** deklarieren, exakt in den Namen, die das Makro erwartet:
   ```toml
   mcpkit = "0.1"
   mcpkit-core = "0.1"
   mcpkit-server = "0.1"
   mcpkit-macros = "0.1"
   ```
   (`mcpkit-transport`/`mcpkit-client` nur, falls das Makro auch dorthin referenziert — im reproduzierten Fehler wurden nur `mcpkit_core`/`mcpkit_server` vermisst.)
2. Die beiden `#[serde(default)]`-Attribute auf den Funktionsparametern `file_path` und `severity` entfernen — diese Syntax ist durch kein offizielles `mcpkit`-Beispiel belegt. Stattdessen prüfen, ob `Option<T>` von `mcpkit`s `#[tool(...)]`-Makro implizit als optionaler Parameter erkannt wird (nicht in der 0.1.0-Doku belegt — müsste experimentell verifiziert werden).
3. Nach diesen Änderungen erneut `cargo build` laufen lassen und verbleibende Fehler iterativ beheben, da die genannten Fixes nur die vier bekannten Fehler adressieren, nicht notwendigerweise alle Folgefehler.

## Offene Punkte / Grenzen dieser Recherche

- Die crates.io-REST-API (`crates.io/api/v1/...`) war per direktem `curl` aus der Sandbox nicht erreichbar (leere Antwort); alle crates.io-Daten stammen daher aus WebFetch-Zusammenfassungen (Zweitquelle: von einem kleineren Modell zusammengefasste Seiteninhalte) statt aus roh geparstem JSON. Die docs.rs- und GitHub-API-Daten wurden dagegen direkt per `curl` roh abgerufen und lokal geparst (höhere Verlässlichkeit).
- Es wurde nicht verifiziert, ob `Option<T>`-Parameter in `mcpkit` 0.1.0 ohne jedes Attribut automatisch optional sind — dafür wäre ein lokaler Kompilierversuch mit angepasster `Cargo.toml` nötig, der im Rahmen dieser Recherche nicht durchgeführt wurde, da die Empfehlung ohnehin auf einen Wechsel zu `rmcp` lautet.
