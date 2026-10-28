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

## Nachtrag 2026-10-28: Lauf auf fremdem Code

631 Dateien / ca. 181k Zeilen aus serde, clap, tokio, anyhow, regex, itertools, rand und sqlx-core (lokale Cargo-Registry), alle 40 Regeln plus Defaults: 36 Regeln trafen. Die lautesten:

| Regel | Treffer | Einschätzung |
|---|---|---|
| `google-unsafe-needs-safety-comment-rust` | 718 (tokio 645, anyhow 67) | Mindestens ein grosser Teil ist Schreibweise: die Regex akzeptiert nur `// SAFETY:`, tokio schreibt 188x `// Safety:` und 66x `// safety:`. Heuristisch tragen 204 der 718 Treffer ein "safety"-Wort in den 4 Zeilen darüber. Der Guide verlangt `SAFETY:`, die Regel folgt ihm wörtlich. |
| `google-lifetime-name-descriptive-rust` | 499 | erwartetes Hint-Rauschen (`'a`) |
| `style-imports-before-mod-declarations-rust` | 333 | zu laut: trifft das gängige Muster "erst `mod`, dann `use`" (serde `lib.rs`). Severity `info`. |
| `google-prefer-expect-over-unwrap-rust` | 164 | teils `tests/`-Lücke, teils echt |
| `api-c-feature-placeholder-words-rust` | 117 | trifft auch `use_std` (Unterstrich); inhaltlich vertretbar |
| `style-use-before-other-items-rust` | 97 | nahe an `imports-before-mod`, gleiche Prüfung |

Offene Entscheidung: `SAFETY:` case-insensitive akzeptieren (weniger Rauschen, weicht vom Wortlaut des Guides ab) oder wörtlich lassen. Nicht entschieden, Regel unverändert.
