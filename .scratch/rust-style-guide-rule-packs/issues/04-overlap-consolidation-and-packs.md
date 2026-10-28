Type: task
Status: open
Blocked by: (keine, 01–03 resolved)

## Question

Führe die Kandidatenlisten aus 01–03 zusammen. Überschneidende Regeln werden
nur einmal gebaut, im Guide mit der präzisesten Formulierung; die anderen
Guides bekommen einen "siehe …"-Verweis. Falls nach der Linter-Grenze für einen
Guide nichts übrig bleibt: das ist das Ergebnis ("kein Pack, weil …").

Für jedes verbleibende "adopt" die vollständige `RuleConfig`-YAML entwerfen
nach `rule-packs/<guide>/…` (ADR 0002; Rule-ID trägt Guide-Referenz), mit
`norma test_pattern`/`norma validate` gegen positive und negative Beispiele
prüfen und pro Guide-Verzeichnis `norma import rule-packs/<guide> --category
<guide>` laufen lassen (0 skipped, keine ID-Kollision mit den 20 Defaults und
den 39 bestehenden Pack-Regeln).

Resolution: finale Entscheidungsliste pro Guide plus Verifikationsergebnis.

## Mitgebrachte Punkte aus Tickets 01 und 03

- Einheitliche Linie für opt-in-Clippy-Gruppen (pedantic/restriction/nursery) festlegen; die Notes sagen nur "Default-Lints".
- Quellenhinweis (CC BY 3.0) für Pack(s) aus dem Google-Guide.
- Grenzfälle G-09 und G-21 bestätigen.
- Google-Guide widerspricht teils Clippy-Default-Lints: solche Regeln nicht bauen.
- Pack-Beschreibung für die API Guidelines: richtet sich an Library-Crates.
- Aus Ticket 02: rustc-Default-Lints gehören zur Linter-Grenze (jetzt in den Map-Notes).
