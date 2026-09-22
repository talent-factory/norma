Type: grilling
Status: resolved

## Question

Soll norma einen Weg bekommen, ein ganzes Verzeichnis bestehender ast-grep-Regel-YAML-Dateien (z. B. ein geklontes `sgconfig.yaml`-Regelverzeichnis oder ast-greps eigener Katalog) in einem Rutsch zu importieren, statt jede Regel einzeln per `register_pattern` zu übergeben?

**Kontext:** Direkt anschliessend an README "Adopting an existing ast-grep rule" (dokumentiert bereits den 1:1-Copy-Paste-Weg für eine einzelne Regel) — diese Frage ist die Fortsetzung für *mehrere* Regeln auf einmal.

Zu klären:
- MCP-Tool (`import_rules_from_directory`) und/oder CLI-Subcommand (`norma import <dir>`)?
- `register_pattern` verlangt `name`/`description`/`category` zusätzlich zur YAML — ast-grep-Regel-YAML hat dafür keine Entsprechung (nur `id`/`message`). Woher kommen diese beim Bulk-Import (aus `id`/`message` ableiten? Verzeichnisname als `category`? Interaktiv nachfragen, unrealistisch bei Bulk)?
- Umgang mit Regeln, deren `language:` ausserhalb des zu dem Zeitpunkt unterstützten Sprachsets liegt (siehe Ticket "Sprachabdeckung erweitern") — überspringen mit Warnung, oder hart fehlschlagen?
- Duplikat-/Update-Verhalten, falls dieselbe `id` erneut importiert wird (vgl. `seed_defaults`' Per-Id-Semantik, `DEVELOPMENT.md`).

## Answer

**Verdict: adopt.** Neues MCP-Tool `import_rules(yaml, category?)` plus CLI-Subcommand `norma import <dir>`.

1. **`name`/`description`-Herkunft**: `name` ← Rule-`id` roh (keine Humanisierung), `description` ← Rule-`message`.
2. **`category`**: optionaler Batch-weiter Parameter, gilt für alle importierten Regeln des Aufrufs; ohne Angabe `None`.
3. **Nicht unterstützte Sprachen**: seit Ticket "Sprachabdeckung erweitern" (adopt, alle 28) nur noch für echte Tippfehler/Fremdsprachen relevant — einzelne kaputte Datei wird übersprungen + Warnung, Import bricht nicht komplett ab. Ergebnis listet importiert + übersprungen (mit Grund).
4. **Duplikat-/Update-Verhalten**: erbt 1:1 das bestehende `save_pattern`-Upsert (`ON CONFLICT(id) DO UPDATE`) — kein separates "nie überschreiben"-Flag. Re-Import einer aktualisierten externen Regel wirkt.
5. **Mechanik**: `import_rules(yaml: String, category: Option<String>)` (MCP) nimmt einen Multi-Dokument-YAML-String (`---`-getrennt) entgegen, passt zu norma's bestehender Daten-rein/raus-Konvention (kein Pfad-Parameter). Voraussetzung: `pattern_engine::parse_rule` (heute `configs.remove(0)`, nimmt nur das erste Dokument) muss um eine Multi-Dokument-Variante ergänzt werden — `from_yaml_string` liefert bereits alle Dokumente. `norma import <dir>` (CLI) liest Dateien direkt von der Platte (wie `validate_files`) und ruft dieselbe zugrundeliegende Import-Funktion.

→ Graduiert zu [TF-894](https://linear.app/talent-factory/issue/TF-894) im `norma`-Projekt.
