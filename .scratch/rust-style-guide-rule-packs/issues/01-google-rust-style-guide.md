Type: research
Status: resolved

## Question

Lies den [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html)
vollständig. Prüfe zuerst die Lizenz des Guides (Paraphrase mit Link muss
zulässig sein; sonst festhalten). Erfasse jede Einzelregel als Kandidat und
entscheide pro Kandidat adopt/defer/reject gegen die Linter-Grenze in
`../map.md` (Notes): syntaktisch mit ast-grep zuverlässig erkennbar **und**
nicht schon von rustfmt oder Clippy-Default-Lints abgedeckt. Bei Clippy den
konkreten Lint-Namen nennen (nachschlagen, nicht raten).

Wo der Guide den offiziellen Style Guide oder die API-Guidelines nur
referenziert, den Kandidaten als "Verweis auf …" markieren statt doppelt zu
bewerten. Ergebnis: Tabelle (Regel, Abschnitt/Anker, Entscheid, Einzeiler-
Begründung). **Keine YAML** entwerfen; das macht Ticket 04.

## Answer

Findings: Branch `research/google-rust-style-guide` (Commit `eeb38d6`, nicht gepusht), Datei `.scratch/rust-style-guide-rule-packs/research/01-google-rust-style-guide.md`.

89 Kandidaten: 20 adopt, 6 defer, 57 reject, 6 Verweise (auf offiziellen Style Guide bzw. API Guidelines).

- **Lizenz:** CC BY 3.0 (Code-Samples Apache 2.0), nur im Repo `google/styleguide` dokumentiert. Paraphrase mit Link ist zulässig, verlangt aber Namensnennung samt Bearbeitungshinweis, also Quellenhinweis in jedem Pack.
- Der Guide kollidiert mit Clippy: sein Gut-Beispiel `Shape::Triangle | _ => {}` löst den Default-Lint `wildcard_in_or_patterns` aus. Zwei Guide-Regeln laufen gegen Clippy-Lints der Gruppe nursery.
- Der Guide ist intern uneinheitlich (`new_with_foo` vs. `new_from_foo`).
- Clippy-Lints stammen aus `clippy-driver -W help` (Clippy 0.1.99), weil die Web-Lintliste ohne JavaScript nicht lesbar war; andere Clippy-Versionen können abweichen.
- Zu bestätigen in Ticket 04: G-09 (`use` aus einem Modul, rustfmt-Option nur Nightly) und G-21 (`&Option<T>`, Clippy-Lint nur pedantic) stehen als adopt, liegen aber an der Linter-Grenze.

Die Findings-Datei liegt zusätzlich auf develop unter `research/01-google-rust-style-guide.md`; der Research-Branch ist damit entbehrlich.
