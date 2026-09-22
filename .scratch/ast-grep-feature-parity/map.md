# norma — ast-grep Feature Parity (wayfinder map)

## Destination

Eine Spec: eine geprüfte, entschiedene Liste (adopt/defer/reject) von ast-grep-Funktionalitäten, die norma heute nicht abdeckt, obwohl `ast-grep-core`/`-config`/`-language` bereits eingebunden sind. Jede mit "adopt" entschiedene Position wird danach als eigenes Ticket im Linear-Projekt [norma](https://linear.app/talent-factory/project/norma-b050418c4d0d) angelegt — diese Map plant, sie exekutiert nicht.

## Notes

- **Prämisse (Grilling Q4):** norma bleibt komplementär zu `ast-grep-mcp`, nie ein Ersatz dafür (siehe README "norma vs. ast-grep's own ast-grep-mcp") — jede Ticket-Entscheidung wird daran gemessen.
- **Tracker-Entscheidung:** diese Map lebt lokal unter `.scratch/` (Option A) statt in Linear, weil wayfinder keinen First-Class-Linear-Support hat. Adoptierte Entscheidungen graduieren manuell zu Linear-Issues im `norma`-Projekt.
- **Verwandte, abgeschlossene Map:** [norma — Architecture Spec](../norma-architecture/map.md) (MVP-Architektur, Destination bereits erreicht) — diese Map ist ein eigenständiger Folge-Effort, keine Fortsetzung davon.
- **Skills:** `/grilling` und `/domain-modeling` für jede Ticket-Session nutzen (wie in der alten Map).
- **Fakten-Fundament** (bereits recherchiert beim Chartern, siehe Ticket-Bodies für Details je Bereich): norma parst Registrierungen bereits über den echten `ast-grep-config`-0.45.3-Crate (`pattern_engine::parse_rule`), nicht über eine eigene Teilmenge — Ausdrucksstärke der Regeln (constraints/utils/transform/composite) ist grösstenteils schon "for free" vorhanden. Die Lücken liegen in der *Anwendungsschicht* (was norma mit dem geparsten `RuleConfig` tut), nicht im Matching selbst.
- **Chat-Kommunikation:** Deutsch (Standing preference, wie alte Map).

## Decisions so far

- [Autofix/Rewrite-Unterstützung](issues/01-autofix-rewrite-support.md) — Adopt: `suggested_fix` in `PatternViolation` (Anzeigen) + `--fix` (CLI, in-place) + eigenständiges MCP-Tool `apply_pattern_fix` (Anwenden, rein lesend/rückgabebasiert). Fail-safe bei überlappenden Fixes (keiner wird angewendet). Lokale `rewriters` laufen vermutlich automatisch mit, globale sind out of scope. → [TF-890](https://linear.app/talent-factory/issue/TF-890).
- [Regel-Testing/AST-Debug-Tooling](issues/02-rule-testing-ast-debug-tooling.md) — Adopt (eingeschränkt): neues Tool `test_pattern(rule, code)`, nichts gespeichert, liefert auch `suggested_fix`. Kein `dump_syntax_tree`-Äquivalent (reiner Klon von `ast-grep-mcp` ohne norma-Mehrwert). → [TF-891](https://linear.app/talent-factory/issue/TF-891).
- [Sprachabdeckung erweitern](issues/03-language-coverage-expansion.md) — Adopt: alle 28 `SupportLang`-Sprachen generisch für die Registrierung freischalten (aus `SupportLang::all_langs()` abgeleitet, nicht hartcodiert). Neue Sprachen bleiben ohne eingebaute Default-Patterns. → [TF-893](https://linear.app/talent-factory/issue/TF-893).
- [Bulk-Import bestehender Regeln](issues/04-bulk-import-existing-rules.md) — Adopt: MCP-Tool `import_rules(yaml, category?)` (Multi-Dokument-YAML-String) + CLI `norma import <dir>`. `name`/`description` aus `id`/`message`, erbt bestehendes Upsert-Verhalten, überspringt kaputte Dateien mit Warnung statt abzubrechen. → [TF-894](https://linear.app/talent-factory/issue/TF-894).
- [Ad-hoc/ephemeres Pattern-Suchen](issues/05-adhoc-ephemeral-search.md) — Reject: reine Redundanz zu `ast-grep-mcp`s `find_code`/`find_code_by_rule`, kein norma-eigener Anwendungsfall. Kein Linear-Ticket.

## Not yet specified

(keine offene Fog aus der Chartering-Runde; neue Fog entsteht ggf. beim Resolven der Tickets unten)

## Out of scope

- `ast-grep lsp` (Editor-Language-Server) — anderes Einsatzmodell als norma's MCP/CLI-Ansatz
- `ast-grep new` (Projekt-Scaffolding) — kein Analogon in norma's Domäne
- `sgconfig.yaml`-Custom-Language-dylib-Loading — Nischenfall, kein erkennbarer Bedarf
- Ad-hoc/ephemeres Pattern-Suchen (siehe [Ticket](issues/05-adhoc-ephemeral-search.md)) — reine Redundanz zu `ast-grep-mcp`s `find_code`/`find_code_by_rule`; für diesen Bedarf auf `ast-grep-mcp` verweisen statt selbst nachzubauen

## Status

**Alle 5 Tickets resolved, Frontier leer — Destination erreicht.** 4 von 5 Kandidaten adopted und als Linear-Issues graduiert (TF-890 bis TF-894, siehe "Decisions so far"); einer (Ad-hoc-Suche) rejected. Diese Map ist abgeschlossen.
