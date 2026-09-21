Type: grilling

## Question

Soll norma ein Such-Tool bekommen, das ein Pattern *ephemer* (ohne vorherige Registrierung) gegen Code matcht — analog zu `ast-grep-mcp`s `find_code`/`find_code_by_rule`?

**Prämisse (Map-Notes):** norma soll komplementär zu `ast-grep-mcp` bleiben, nicht dessen Kerngeschäft duplizieren (README-Abschnitt "norma vs. ast-grep's own ast-grep-mcp" positioniert norma explizit als *Compliance-Gate*, nicht als *Such-Tool*). Diese Frage prüft, ob das bei diesem einen Feature trotzdem Sinn ergibt oder ob "reject, da redundant zu ast-grep-mcp" die richtige Antwort ist.

Zu klären:
- Gibt es einen norma-eigenen Anwendungsfall für Ad-hoc-Suche, der sich von `ast-grep-mcp`s Rolle unterscheidet (z. B. Suche *innerhalb der bereits registrierten Pattern-Sprache-Auswahl*, oder als Vorstufe zu Ticket "Regel-Testing")?
- Falls reject: explizit als "Out of scope" auf der Map vermerken statt nur implizit offen zu lassen.
