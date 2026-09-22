Type: grilling
Status: resolved

## Question

Soll norma das bereits geparste `fix`/`fixer` (und ggf. `rewriters`) aus einer registrierten Pattern-RuleConfig-YAML tatsächlich nutzen, statt es wie heute stillschweigend zu verwerfen?

**Befund:** `ast_grep_config::SerializableRuleConfig` (Crate v0.45.3, `src/rule_config.rs`) hat bereits `pub fix: Option<SerializableFixer>` und `pub rewriters: Option<Vec<SerializableRewriter>>` — ADR 0002 erwähnt `fix:` explizit als unterstützt. Aber `pattern_engine::find_violations` (`src/pattern_engine.rs:129`) nutzt `config.fixer` nirgends, und `PatternViolation` (`src/models.rs:144`) hat kein Fix-Feld. Ein registriertes `fix:` existiert also nur in der gespeicherten YAML, nie im Validierungsergebnis.

Zu klären:
- Nur *anzeigen* (Suggested-Fix-Text in `PatternViolation`, z. B. `suggested_fix: Option<String>`) oder auch *anwenden* (ein `--fix`-Flag für `norma validate` bzw. ein eigenes MCP-Tool, das die Datei überschreibt)?
- Falls Anwenden: wie mit Konflikten zwischen mehreren matchenden Patterns auf demselben Codebereich umgehen?
- Lohnt sich `rewriters` (mehrstufige Rewrite-Pipelines) für norma's Use-Case, oder reicht einfaches `fix:`?
- Prämisse aus der Map: bleibt norma damit komplementär zu `ast-grep-mcp`, oder überschneidet sich Autofix mit dessen Rolle?

## Answer

**Verdict: adopt.** norma nutzt `fix`/`fixer` (und lokal eingebettete `rewriters`) künftig aktiv, statt sie zu verwerfen.

1. **Anzeigen UND Anwenden** (revidiert gegenüber der ursprünglichen Empfehlung "nur anzeigen"): `PatternViolation` bekommt ein `suggested_fix: Option<String>`-Feld (befüllt via `Fixer::generate_replacement(&NodeMatch)`, das auf dem bereits compilierten `config.fixer: Vec<Fixer>` steht — rein additiv, kein Parsing-Mehraufwand). Zusätzlich wird der Fix tatsächlich anwendbar gemacht.
2. **Mechanik pro Surface** (aus der Dateipfad-Asymmetrie hergeleitet, da `validate_pattern_compliance` nur `code: String` ohne Pfad bekommt, `norma validate` aber echte Pfade hat):
   - **CLI**: neues `--fix`-Flag auf `norma validate`, schreibt die Datei **direkt in-place** (wie `eslint --fix`/`biome --fix`) — kein separater Dry-Run-Modus in v1; Verantwortung für einen sauberen Git-Working-Tree liegt bei der aufrufenden Person, nur dokumentiert, nicht technisch erzwungen.
   - **MCP**: bleibt rein lesend nach aussen. Neues, eigenständiges Tool `apply_pattern_fix(code, language)` (Name grob, Feinschliff bei Umsetzung) gibt den umgeschriebenen Code als String zurück, schreibt selbst nie ins Dateisystem — der Caller entscheidet, was er mit dem String macht. Bewusst kein `apply_fixes: bool`-Parameter auf `validate_pattern_compliance`, um lesende/schreibende Tools klar getrennt zu halten (Konvention, die zu norma's bisherigen vier Tools passt).
3. **Konfliktauflösung**: überlappen die Fix-Ranges zweier matchender Patterns auf demselben Codebereich, wird **keiner** der beiden angewendet, stattdessen eine Warnung ausgegeben (fail-safe statt kaskadierender Auto-Rewrites — bewusst konservativ im FFHS-Lehrkontext).
4. **`rewriters`**: lokal in derselben Pattern-YAML eingebettete Rewriter laufen strukturell vermutlich automatisch mit, sobald `config.fixer`/`generate_replacement` überhaupt genutzt wird (`parse_rule` parst die volle YAML inkl. lokaler `rewriters:`-Liste) — wird bei der Umsetzung verifiziert, kein eigenes Recherche-Ticket nötig. *Global* über mehrere Pattern-Zeilen geteilte Rewriter bleiben **out of scope** (passen nicht ins "ein Pattern = eine Zeile"-Modell aus ADR 0002).
5. **Komplementär-Prämisse**: unproblematisch — weder Anzeigen noch Anwenden überschneidet sich mit `ast-grep-mcp`s Rolle (Suche/Test/Debug); das Tool bietet keinen Fix-Mechanismus an.

→ Graduiert zu [TF-890](https://linear.app/talent-factory/issue/TF-890) im `norma`-Projekt.
