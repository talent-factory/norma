Type: task
Status: open
Blocked by: (keine, 04 resolved)

## Question

Lass die neuen Packs einmalig gegen norma's eigenes `src/` laufen
(`norma validate --language rust …`) und halte fest: Verstösse je Regel,
offensichtliche False Positives, Regeln ohne einen einzigen Treffer und ohne
Positiv-Test (verdächtig). Zweck ist Plausibilität der Regeln, nicht das
Beheben von Verstössen. Regeln mit False-Positive-Problem werden im Ticket 04
nachgebessert oder auf defer gesetzt; die Anpassung dokumentiert dieses Ticket.

Hinweis aus Ticket 04: Rauschverdacht bei `style-use-before-other-items-rust`, `style-imports-before-mod-declarations-rust`, `style-comment-space-after-slashes-rust` (auf fremdem Code ungetestet); `google-non-exhaustive-pub-struct-rust` hatte in `src/`+`tests/` 18 Treffer; `google-prefer-expect-over-unwrap-rust` überspringt `tests/`-Dateien nicht.
