Type: grilling
Status: resolved

## Question

Soll norma ein MCP-Tool (und/oder CLI-Subcommand) hinzufügen, mit dem ein *Kandidaten*-Rule gegen ein Beispiel-Snippet getestet wird, bevor es per `register_pattern` dauerhaft registriert wird — analog zu `ast-grep-mcp`s `test_match_code_rule`/`dump_syntax_tree`?

**Befund:** `register_pattern` (`src/mcp_server.rs`) prüft heute ausschliesslich, ob die YAML *parst* (`RegisterPatternError::InvalidRule`) — nicht, ob die Regel das tut, was der Aufrufer beabsichtigt. Der einzige heutige Weg, das zu prüfen, ist: registrieren, dann `validate_pattern_compliance` gegen ein Beispiel aufrufen (README "Adopting an existing ast-grep rule" beschreibt das als Workaround) — ein fehlerhaftes Pattern landet also potenziell erst nach dem Registrieren im Store.

Zu klären:
- Neues MCP-Tool `test_pattern`/`dry_run_pattern` (Rule-YAML + Beispielcode rein, Treffer/Nicht-Treffer + Diagnose raus, nichts wird gespeichert)?
- Auch ein `dump_syntax_tree`-Äquivalent (AST eines Snippets anzeigen), oder ist das zu nah am ureigenen Kerngeschäft von `ast-grep-mcp` (Prämisse: komplementär bleiben)?
- Falls beide MCP-Server (norma + `ast-grep-mcp`) parallel im selben Claude-Code-Setup laufen: ist ein Duplikat hier überhaupt ein Problem, oder schadet die Redundanz nicht?

## Answer

**Verdict: adopt (eingeschränkt).** Neues, eigenständiges MCP-Tool `test_pattern(rule, code)` — kein `dump_syntax_tree`-Äquivalent.

1. **`test_pattern`**: nimmt eine Rule-YAML plus ein Beispiel-Snippet, gibt Treffer/Nicht-Treffer + Diagnose zurück, speichert nichts. Lässt sich praktisch komplett aus bereits vorhandenem Code zusammensetzen (`pattern_engine::parse_rule` + `find_violations`, ohne `PatternStore`-Zugriff) — kein Neubau, nur eine neue Kombination bestehender Bausteine.
2. **Kein AST-Dump**: anders als `test_pattern` (das gegen norma's *eigene* Validierungspipeline testet, inkl. Sprachallowlist) wäre ein `dump_syntax_tree`-Klon ein reiner, wertfreier Nachbau von `ast-grep-mcp`s eigenem Tool ohne norma-spezifischen Mehrwert — verstösst gegen die Komplementär-Prämisse. Für AST-Debugging wird stattdessen in der Tool-Beschreibung von `test_pattern` auf `ast-grep-mcp` verwiesen.
3. **Tool-Form**: eigenständiges Tool, kein `dry_run`-Parameter auf `register_pattern` — passt ohnehin nicht ins Parameter-Set (`test_pattern` braucht kein `name`/`description`/`category`), und hält lesend/schreibend getrennt (Konsistenz mit Ticket 1, "Autofix/Rewrite-Unterstützung").
4. **Konsistenz mit Ticket 1**: `test_pattern` liefert auch das dort beschlossene `suggested_fix`-Feld — gleiches Ergebnis-Shape wie `validate_pattern_compliance`.

→ Graduiert zu [TF-891](https://linear.app/talent-factory/issue/TF-891) im `norma`-Projekt.
