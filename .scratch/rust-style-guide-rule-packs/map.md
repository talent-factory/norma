# norma — Rust-Style-Guide-Rule-Packs (wayfinder map)

## Destination

Eine belegte Entscheidung pro Guide, ob sich ein ast-grep-Rule-Pack lohnt, und
für das, was die Linter-Grenze übersteht, importierbare Pack-YAMLs. Die Guides:
[Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html),
[offizieller Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/),
[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/about.html).
Pro Guide eine Entscheidungsliste (adopt/defer/reject, je mit Begründung) plus
vollständige `RuleConfig`-YAML für jedes "adopt". "Lohnt sich nicht" ist ein
gültiges Ergebnis.

## Notes

- **Execution carried into this map** (Override von "plan, don't do", wie in
  [Rule-Pack-Sourcing](../ast-grep-rule-pack-sourcing/map.md)): Das Ticket
  "Overlap-Konsolidierung" erzeugt direkt die Pack-YAML-Dateien. Die
  Research-Tickets liefern nur Kandidatenlisten, keine YAML.
- **Linter-Grenze:** aufnehmen nur, was ast-grep rein syntaktisch zuverlässig
  erkennt **und** was weder rustfmt, Clippy (Default-Lints) noch rustc (Default-Lints, z. B. `non_snake_case`, `unused_parens`) schon prüfen.
  Alles andere wird rejected, mit Grund (z. B. "rustfmt", "braucht Typinfo",
  "Clippy `lint_name`"). Der offizielle Style Guide ist grösstenteils
  Formatierung (rustfmt); die API-Guidelines brauchen oft Typinformation
  (z. B. C-COMMON-TRAITS).
- **Struktur (bewusste Abweichung vom Vorgänger):** ein Verzeichnis pro Guide,
  `rule-packs/<guide>/…`, die Kategorie ist der Guide
  (`norma import rule-packs/<guide> --category <guide>`). Rule-IDs tragen die
  Guide-Referenz (z. B. `C-CASE` oder ein Abschnitts-Anker). Der Vorgänger
  sortierte nach Kategorie, weil `norma import` pro Aufruf eine `--category`
  setzt; hier steht Nachvollziehbarkeit gegen einen benannten Guide im Vordergrund.
- **Überschneidungen:** jede Regel wird nur einmal gebaut, im Guide mit der
  präzisesten Formulierung. Die anderen Guides bekommen in der
  Entscheidungsliste einen Verweis ("siehe …"), kein Duplikat.
- **Nutzer:** opt-in-Packs im Repo, auch für Dritte importierbar. Ein
  Plausibilitätscheck gegen norma's eigene `src/` ist Teil der Map; Verstösse
  beheben und ein Dauer-Dogfooding-Test sind out of scope.
- **Lizenz:** Regeltexte der Guides werden nicht kopiert, nur paraphrasiert und
  mit Link referenziert; die jeweilige Lizenz der Guides prüft das
  zugehörige Research-Ticket.
- **Tracker:** lokal unter `.scratch/` (wie die Vorgänger), kein Linear.
- **Skills:** `/grilling` und `/domain-modeling` für HITL-Sessions.
- **Chat-Kommunikation:** Deutsch.

## Decisions so far

- [Google Rust Style Guide](issues/01-google-rust-style-guide.md) — 89 Kandidaten: 20 adopt, 6 defer, 57 reject, 6 Verweise. Lizenz CC BY 3.0 (Quellenhinweis nötig). Guide widerspricht teils Clippy-Default-Lints.
- [Rust API Guidelines](issues/03-rust-api-guidelines.md) — 54 Guidelines: 7 adopt, 7 defer, 40 reject (nach Haupteintrag). MIT/Apache-2.0. Sauberste Kandidaten sind die, für die es keinen Lint gibt (`TryInto`, `description()`, Drop-Panic, Struct-Bounds, Deref).
- [Offizieller Rust Style Guide](issues/02-official-rust-style-guide.md) — 53 Zeilen: 10 adopt, 5 defer, 38 reject (rustfmt deckt fast alles). MIT/Apache-2.0. Echte Lücken: Item-Ordnung, `macro_rules!`-Namen, Doc-Comment-Stil. Grenze um rustc-Lints erweitert.

## Not yet specified

- Doku-Nachzug: README-Abschnitt "Ready-made rule packs", ggf. CHANGELOG/
  DEVELOPMENT.md. Wird scharf, sobald feststeht, welche Packs entstehen.

## Out of scope

- Engine-/Feature-Änderungen an norma (neue MCP-Tools, Sprachplumbing,
  `default_patterns.rs`) — ein Befund, der Code braucht, wird ein eigener Effort.
- Verstösse in norma's `src/` beheben oder Dauer-Dogfooding-Test einbauen.
- Formatierungsregeln, die rustfmt erzwingt; Regeln, die Typinformation brauchen.

## Status

Gechartert 2026-10-28. Tickets 01–03 resolved. Frontier: Ticket 04 (Overlap-Konsolidierung und Packs).
