Type: task
Status: resolved
Blocked by: (keine, 04 resolved)

## Question

Lass die neuen Packs einmalig gegen norma's eigenes `src/` laufen
(`norma validate --language rust …`) und halte fest: Verstösse je Regel,
offensichtliche False Positives, Regeln ohne einen einzigen Treffer und ohne
Positiv-Test (verdächtig). Zweck ist Plausibilität der Regeln, nicht das
Beheben von Verstössen. Regeln mit False-Positive-Problem werden im Ticket 04
nachgebessert oder auf defer gesetzt; die Anpassung dokumentiert dieses Ticket.

Hinweis aus Ticket 04: Rauschverdacht bei `style-use-before-other-items-rust`, `style-imports-before-mod-declarations-rust`, `style-comment-space-after-slashes-rust` (auf fremdem Code ungetestet); `google-non-exhaustive-pub-struct-rust` hatte in `src/`+`tests/` 18 Treffer; `google-prefer-expect-over-unwrap-rust` überspringt `tests/`-Dateien nicht.

## Answer

Lauf: alle 40 neuen Regeln (frische DB, nur die drei Packs) gegen `git ls-files 'src/*.rs' 'tests/*.rs'` mit `norma validate --language rust --json`. Ergebnis: 36 Treffer in 8 Regeln (7 der 40 Pack-Regeln plus der Default `no-debug-print-rust`, 7 Treffer in `src/main.rs`: gewollte CLI-Ausgabe, kein Befund dieser Map).

| Regel | Treffer | Urteil |
|---|---|---|
| `google-non-exhaustive-pub-struct-rust` | 18 (5 Dateien) | Rauschen für eine Anwendung: sinnvoll nur für Library-Crates. Severity `hint` ist richtig, ins Pack-README gehört der Hinweis. |
| `google-non-exhaustive-pub-enum-rust` | 3 | wie oben |
| `google-prefer-expect-over-unwrap-rust` | 3 (`tests/gof_patterns.rs`) | False Positive: `tests/`-Dateien werden nicht übersprungen (bekannte Lücke aus Ticket 04). Nachbessern oder in der `note` benennen. |
| `google-no-combinator-chain-in-for-head-rust` | 2 | True Positive (`src/pattern_engine.rs:336`). |
| `google-use-one-module-per-use-rust` | 1 | True Positive (`src/mcp_server.rs:3`, verschachteltes `use`). |
| `google-new-returning-result-rust` | 1 | True Positive (`PatternStore::new -> Result<Self>`). |
| `style-use-before-other-items-rust` | 1 | True Positive (`src/cli.rs:964`, `use` mitten im Test-Modul nach Items). |

- **Kein Treffer bei 33 Regeln.** Verdächtig wäre das nur ohne Positiv-Test: Alle 40 Regeln wurden mit den Fixtures aus Ticket 04 erneut gegen dieselbe DB geprüft und feuern alle (40 von 40). Null Treffer in `src/` heisst also "sauber", nicht "kaputt".
- Die drei im Ticket 04 genannten Rauschverdächtigen: `style-imports-before-mod-declarations-rust` und `style-comment-space-after-slashes-rust` haben in `src/` null Treffer (kein Rauschen hier, auf fremdem Code weiter ungetestet); `style-use-before-other-items-rust` hat einen echten Treffer.
- Die Zahl 57 aus der API-Guidelines-Recherche (`pub`-Felder) taucht nicht mehr auf: C-STRUCT-PRIVATE ist in G-75 aufgegangen.
- **Empfohlene Nacharbeit (nicht in dieser Map ausgeführt):** `tests/` in `google-prefer-expect-over-unwrap-rust` ausnehmen oder in der `note` dokumentieren; im Doku-Nachzug (README) den Library-Crate-Hinweis zu den `non_exhaustive`-Regeln nennen.
