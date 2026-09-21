Type: grilling

## Question

Soll norma ein MCP-Tool (und/oder CLI-Subcommand) hinzufügen, mit dem ein *Kandidaten*-Rule gegen ein Beispiel-Snippet getestet wird, bevor es per `register_pattern` dauerhaft registriert wird — analog zu `ast-grep-mcp`s `test_match_code_rule`/`dump_syntax_tree`?

**Befund:** `register_pattern` (`src/mcp_server.rs`) prüft heute ausschliesslich, ob die YAML *parst* (`RegisterPatternError::InvalidRule`) — nicht, ob die Regel das tut, was der Aufrufer beabsichtigt. Der einzige heutige Weg, das zu prüfen, ist: registrieren, dann `validate_pattern_compliance` gegen ein Beispiel aufrufen (README "Adopting an existing ast-grep rule" beschreibt das als Workaround) — ein fehlerhaftes Pattern landet also potenziell erst nach dem Registrieren im Store.

Zu klären:
- Neues MCP-Tool `test_pattern`/`dry_run_pattern` (Rule-YAML + Beispielcode rein, Treffer/Nicht-Treffer + Diagnose raus, nichts wird gespeichert)?
- Auch ein `dump_syntax_tree`-Äquivalent (AST eines Snippets anzeigen), oder ist das zu nah am ureigenen Kerngeschäft von `ast-grep-mcp` (Prämisse: komplementär bleiben)?
- Falls beide MCP-Server (norma + `ast-grep-mcp`) parallel im selben Claude-Code-Setup laufen: ist ein Duplikat hier überhaupt ein Problem, oder schadet die Redundanz nicht?
