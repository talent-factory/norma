Type: research
Status: resolved

## Question

Lies die [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/about.html)
vollständig (Checkliste und Detailseiten, alle Kapitel: Naming, Interoperability,
Macros, Documentation, Predictability, Flexibility, Type safety, Dependability,
Debuggability, Future proofing, Necessities). Lizenz prüfen.

Pro Guideline (C-…) entscheiden: adopt/defer/reject gegen die Linter-Grenze in
`../map.md` (Notes). Besonders sauber trennen: *rein syntaktisch prüfbar*
(z. B. Naming-Konventionen) vs. *braucht Typinformation oder Semantik* (z. B.
"implementiert Debug") vs. *Clippy deckt ab* (Lint-Name nachschlagen).
Ergebnis: Tabelle (C-ID, Entscheid, Einzeiler-Begründung). **Keine YAML**; das
macht Ticket 04.

## Answer

Findings: Branch `research/rust-api-guidelines` (Commit `4e1c84b`, nicht gepusht), Datei `.scratch/rust-style-guide-rule-packs/research/03-rust-api-guidelines.md`.

54 Guidelines, 60 Zeilen (sechs mit Teilforderungen): 11 adopt (davon 4 Teilzeilen), 9 defer, 40 reject. Nach Haupteintrag: 7 adopt, 7 defer, 40 reject.

- **Lizenz:** `MIT OR Apache-2.0`, Paraphrase mit Link reicht.
- **adopt:** C-FEATURE (nur Quelltext-Teil), C-GETTER, C-DEREF, C-CUSTOM-TYPE, C-DTOR-FAIL, C-STRUCT-PRIVATE, C-STRUCT-BOUNDS, C-CASE (Akronyme), C-CONV (`_mut`-Suffix), C-CONV-TRAITS (`TryInto`), C-GOOD-ERR (`description()`). Zehn davon traf ast-grep im Scratch-Test sauber; C-CUSTOM-TYPE nur mit festem Arity-Muster.
- Für `TryInto`-impl, `description()`, Panic in `Drop`, Struct-Bounds und `Deref`-Missbrauch gibt es weder in rustc noch in Clippy einen Lint: die sauberste Pack-Kandidaten.
- Alle fünf Macro-Guidelines sind rejected (Macro-Bodies sind für tree-sitter-rust flache Token-Bäume).
- **Offen für Ticket 04:** Viele passende Clippy-Lints sind nur opt-in (pedantic/restriction/nursery/cargo), die Grenze sagt aber "Default-Lints". Die Linie muss einheitlich gezogen werden (betrifft C-CUSTOM-TYPE, C-FAILURE).
- `wrong_self_convention` greift bei exportierter API per Default nicht.
- ast-grep-Fallen: `fn $N(&self)` matcht `pub fn` nicht; zwei `$$$`-Lücken um zwei `bool`-Parameter matchten nicht.
- Die Guidelines richten sich an Library-Crates, norma ist eine Anwendung: das Pack so beschreiben. Einzige Streuung in `src/`: 57 Treffer bei `pub`-Feldern (Rauschen für Ticket 05).
