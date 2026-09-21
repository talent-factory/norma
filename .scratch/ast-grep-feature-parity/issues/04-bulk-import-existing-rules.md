Type: grilling

## Question

Soll norma einen Weg bekommen, ein ganzes Verzeichnis bestehender ast-grep-Regel-YAML-Dateien (z. B. ein geklontes `sgconfig.yaml`-Regelverzeichnis oder ast-greps eigener Katalog) in einem Rutsch zu importieren, statt jede Regel einzeln per `register_pattern` zu übergeben?

**Kontext:** Direkt anschliessend an README "Adopting an existing ast-grep rule" (dokumentiert bereits den 1:1-Copy-Paste-Weg für eine einzelne Regel) — diese Frage ist die Fortsetzung für *mehrere* Regeln auf einmal.

Zu klären:
- MCP-Tool (`import_rules_from_directory`) und/oder CLI-Subcommand (`norma import <dir>`)?
- `register_pattern` verlangt `name`/`description`/`category` zusätzlich zur YAML — ast-grep-Regel-YAML hat dafür keine Entsprechung (nur `id`/`message`). Woher kommen diese beim Bulk-Import (aus `id`/`message` ableiten? Verzeichnisname als `category`? Interaktiv nachfragen, unrealistisch bei Bulk)?
- Umgang mit Regeln, deren `language:` ausserhalb des zu dem Zeitpunkt unterstützten Sprachsets liegt (siehe Ticket "Sprachabdeckung erweitern") — überspringen mit Warnung, oder hart fehlschlagen?
- Duplikat-/Update-Verhalten, falls dieselbe `id` erneut importiert wird (vgl. `seed_defaults`' Per-Id-Semantik, `DEVELOPMENT.md`).
