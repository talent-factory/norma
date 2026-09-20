Type: grilling
Status: claimed

## Question

Welche Felder braucht der `Pattern`-Datentyp, und wie sieht die SQLite-Tabelle (`pattern_store.rs`) dafür konkret aus?

Kontext: [Pattern-Matching-Ansatz](02-pattern-matching-approach.md) hat entschieden: `ast-grep-core`, `rule`-Feld einheitlich im YAML-Rule-Format gespeichert (auch einfache Patterns als `rule: { pattern: "$A" }`). Zu klären:

- Welche Felder braucht `Pattern` über `rule` hinaus (id, name, description, severity, languages, category/pattern-type wie "Singleton"/"Factory", created_at/updated_at, aktiv/inaktiv)?
- Wie sieht die konkrete SQLite-Tabellendefinition aus (Spalten, Typen, Indizes, z. B. Index auf `languages` für `get_pattern_checklist`)?
- Wie wird `languages` gespeichert (mehrere Sprachen pro Pattern möglich, siehe Scaffold) — separate Join-Tabelle oder JSON-Array-Spalte?
- Validierung: reicht "wird beim Laden geparst" (ast-grep meldet ungültiges YAML), oder soll `register_pattern` das `rule`-YAML schon beim Registrieren gegen ast-grep validieren, bevor es in SQLite landet?
- Sollte diese Klärung eine kleine Probe enthalten (ein `rule`-String tatsächlich mit `ast-grep-core` parsen und gegen einen Beispiel-Snippet matchen), um die Annahmen aus Ticket 02 konkret zu verifizieren, bevor das Schema festgezurrt wird?
