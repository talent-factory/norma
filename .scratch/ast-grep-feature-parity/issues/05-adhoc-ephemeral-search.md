Type: grilling
Status: resolved

## Question

Soll norma ein Such-Tool bekommen, das ein Pattern *ephemer* (ohne vorherige Registrierung) gegen Code matcht — analog zu `ast-grep-mcp`s `find_code`/`find_code_by_rule`?

**Prämisse (Map-Notes):** norma soll komplementär zu `ast-grep-mcp` bleiben, nicht dessen Kerngeschäft duplizieren (README-Abschnitt "norma vs. ast-grep's own ast-grep-mcp" positioniert norma explizit als *Compliance-Gate*, nicht als *Such-Tool*). Diese Frage prüft, ob das bei diesem einen Feature trotzdem Sinn ergibt oder ob "reject, da redundant zu ast-grep-mcp" die richtige Antwort ist.

Zu klären:
- Gibt es einen norma-eigenen Anwendungsfall für Ad-hoc-Suche, der sich von `ast-grep-mcp`s Rolle unterscheidet (z. B. Suche *innerhalb der bereits registrierten Pattern-Sprache-Auswahl*, oder als Vorstufe zu Ticket "Regel-Testing")?
- Falls reject: explizit als "Out of scope" auf der Map vermerken statt nur implizit offen zu lassen.

## Answer

**Verdict: reject.** Kein Ad-hoc/ephemeres Such-Tool für norma.

Zwei potenzielle Anknüpfungspunkte sind bereits anderweitig abgedeckt: "Kandidat-Rule gegen ein Snippet testen" durch `test_pattern` (Ticket "Regel-Testing/AST-Debug-Tooling", adopted); "alle Dateien gegen *registrierte* Patterns validieren" durch `norma validate` mit Shell-Globs/Pre-Commit-Hook (betrifft aber registrierte, keine Ad-hoc-Patterns). Für echtes Ad-hoc-Multi-File-Suchen (unregistriertes Pattern gegen einen ganzen Codebase-Baum) gibt es keinen norma-eigenen Anwendungsfall, der sich von `ast-grep-mcp`s Rolle (`find_code`/`find_code_by_rule`) unterscheidet — reine Redundanz, verstösst gegen die Komplementär-Prämisse der Map.

Zusätzlich als "Out of scope" auf der Map vermerkt (siehe dort) — kein Linear-Ticket, da nichts zu adoptieren ist.
