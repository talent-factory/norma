Type: research
Status: resolved

## Question

Lies den [offiziellen Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
vollständig (alle Unterseiten). Erwartung: fast alles ist Formatierung und damit
rustfmt-Territorium. Belege das pro Abschnitt, statt es anzunehmen, und suche
gezielt nach Regeln, die über Formatierung hinausgehen (Naming, Ordnung von
Items/Imports, Attribute, Kommentar-/Doc-Konventionen) und von rustfmt nicht
erzwungen werden (rustfmt-Optionen nachschlagen, inkl. Nightly-only).

Lizenz prüfen. Pro Kandidat adopt/defer/reject gegen die Linter-Grenze in
`../map.md` (Notes). Ergebnis: Tabelle (Regel, Abschnitt/Anker, Entscheid,
Einzeiler-Begründung, bei reject der konkrete Grund). **Keine YAML**; das macht
Ticket 04.

## Answer

Findings: Branch `research/official-rust-style-guide` (Commit `3de3cc6`, nicht gepusht), Datei `.scratch/rust-style-guide-rule-packs/research/02-official-rust-style-guide.md`.

53 Zeilen: 10 adopt, 5 defer, 38 reject (26 Einzelregeln plus 12 Formatierungsabschnitte, alle von rustfmt abgedeckt).

- **Lizenz:** keine eigene Lizenzdatei, `REUSE.toml` ohne Sondereintrag, also Repo-Standard MIT OR Apache-2.0. Paraphrase mit Link ist unproblematisch.
- **adopt:** `macro_rules!`-Namen in snake_case, Unit-Struct statt leerem Struct, Doc Comments vor Attributen, `///` statt `/** */`, `#[path]` vermeiden, Item-Ordnung (`extern crate` zuerst, `use` und `mod foo;` vor anderen Items, Imports vor Modulen), `macro_rules!` mit `{}`, Space nach `//`. Jede Regel wurde mit Standardkonfiguration gegen rustfmt, Clippy und rustc getestet: keiner meldet sie.
- **defer:** `//` statt `/* */`, einbuchstabige Generics, Kommentarzeilen über 80 Zeichen, Klammern bei zusammengesetzten Range-Grenzen, Kommentare in der Funktionssignatur.
- Die Lücke zu rustfmt ist grösser als erwartet: rustfmt ordnet Items nicht über Gruppen hinweg, Kommentarregeln greifen nur mit Nightly-Optionen.
- Drei Naming-Regeln sind rejected, weil rustc sie schon prüft. `Cargo.toml`-Regeln scheiden aus, weil `ast-grep-language 0.45.3` kein TOML kennt.
- Der Guide empfiehlt `r#crate`, rustc lehnt das ab; der Range-Abschnitt widerspricht sich selbst.
- **Map-Präzisierung:** Die Linter-Grenze umfasst auch rustc-Default-Lints (`non_snake_case`, `unused_parens`, `missing_abi`).
