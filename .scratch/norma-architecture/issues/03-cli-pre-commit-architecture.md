Type: grilling
Blocked by: 01

## Question

Sollen CLI/Pre-Commit-Hook und MCP-Server dasselbe Binary mit Subcommands teilen (z. B. `norma serve` vs. `norma validate --file ...`), oder getrennte Binaries/Crates sein?

Kontext: Pre-Commit-Hook/CLI gehören laut Zweck-Entscheidung zur Destination dieser Spec (nicht Out of scope). Die Antwort hängt am gewählten MCP-Framework aus [mcpkit vs. Alternative](01-mcpkit-vs-alternative.md), weil das Transport-/Entry-Point-Setup (stdio-MCP-Server vs. CLI-Invocation) je nach Framework unterschiedlich verdrahtet wird. Zu klären:

- Ein Binary mit Subcommands (norma serve / norma validate / norma check-patterns) oder separate Binaries/Crates in einem Workspace?
- Teilen sich CLI und MCP-Server denselben Validierungs-Kern (pattern_engine/pattern_store) 1:1, oder braucht die CLI-Variante eigene Anpassungen (z. B. synchron statt async, andere Output-Formate für Pre-Commit)?
- Wie sieht die `.pre-commit-config.yaml`-Integration konkret aus?
