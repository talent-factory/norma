Type: task
Status: open
Blocked by: 04-overlap-consolidation-and-packs.md

## Question

Lass die neuen Packs einmalig gegen norma's eigenes `src/` laufen
(`norma validate --language rust …`) und halte fest: Verstösse je Regel,
offensichtliche False Positives, Regeln ohne einen einzigen Treffer und ohne
Positiv-Test (verdächtig). Zweck ist Plausibilität der Regeln, nicht das
Beheben von Verstössen. Regeln mit False-Positive-Problem werden im Ticket 04
nachgebessert oder auf defer gesetzt; die Anpassung dokumentiert dieses Ticket.
