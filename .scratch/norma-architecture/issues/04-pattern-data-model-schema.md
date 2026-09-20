Type: grilling
Status: resolved

## Question

Welche Felder braucht der `Pattern`-Datentyp, und wie sieht die SQLite-Tabelle (`pattern_store.rs`) dafür konkret aus?

Kontext: [Pattern-Matching-Ansatz](02-pattern-matching-approach.md) hat entschieden: `ast-grep-core`, `rule`-Feld einheitlich im YAML-Rule-Format gespeichert (auch einfache Patterns als `rule: { pattern: "$A" }`). Zu klären:

- Welche Felder braucht `Pattern` über `rule` hinaus (id, name, description, severity, languages, category/pattern-type wie "Singleton"/"Factory", created_at/updated_at, aktiv/inaktiv)?
- Wie sieht die konkrete SQLite-Tabellendefinition aus (Spalten, Typen, Indizes, z. B. Index auf `languages` für `get_pattern_checklist`)?
- Wie wird `languages` gespeichert (mehrere Sprachen pro Pattern möglich, siehe Scaffold) — separate Join-Tabelle oder JSON-Array-Spalte?
- Validierung: reicht "wird beim Laden geparst" (ast-grep meldet ungültiges YAML), oder soll `register_pattern` das `rule`-YAML schon beim Registrieren gegen ast-grep validieren, bevor es in SQLite landet?
- Sollte diese Klärung eine kleine Probe enthalten (ein `rule`-String tatsächlich mit `ast-grep-core` parsen und gegen einen Beispiel-Snippet matchen), um die Annahmen aus Ticket 02 konkret zu verifizieren, bevor das Schema festgezurrt wird?

## Answer

**Domain-Scope (Q1):** norma deckt breiter jede strukturell prüfbare Code-Konvention ab, nicht nur klassische GoF-Design-Patterns — passend zu den bestehenden Scaffold-Defaults.

**`category`-Feld (Q2):** `Pattern` bekommt ein freies `category: Option<String>`-Feld (kein festes Enum).

**Feasibility-Probe (Q5):** Durchgeführt und erfolgreich — `ast-grep-core` 0.45.3 + `ast-grep-config` + `ast-grep-language` parsen ein YAML-`RuleConfig` und matchen es korrekt gegen echten Code, verifiziert für alle vier MVP-Sprachen (Java, Python, Rust, TypeScript) plus ein Negativ-Fall (kein Match). Code: [`04-ast-grep-probe.rs`](04-ast-grep-probe.rs) (Referenz, nicht Teil des norma-Codes).

**Wichtiger Fund aus der Probe (revidiert die ursprüngliche Join-Table-Antwort zu Q3):** `ast-grep-config`s `RuleConfig<L>` hat genau **eine** Sprache pro Rule-Config (`language: L`, kein `Vec`) — ein `rule`-Text kann nicht über mehrere Sprachen hinweg gelten, da die Pattern-Syntax sprachspezifisch geparst wird. Damit entschieden:

- **`Pattern` ist einsprachig** (`language: String`, keine Liste, kein Join-Table, kein JSON-Array). Ein fachliches Konzept über mehrere Sprachen (z. B. "kein Debug-Print") wird zu mehreren Pattern-Zeilen, eine pro Sprache, locker verbunden über gemeinsamen `name`. Der ursprüngliche `LIKE '%java%'`/`"javascript"`-Bug verschwindet dadurch von selbst.
- **`rule` speichert die vollständige ast-grep-`RuleConfig`-YAML** (`id`, `message`, `severity`, `language`, `rule:`, optional `fix:`), nicht nur die innere Match-Klausel. `severity` und `language` werden beim Registrieren **aus dieser YAML abgeleitet** (via `ast_grep_config::from_yaml_string`, das schon Q4s Fail-Fast-Validierung erledigt) und in eigene, indizierte Spalten gespiegelt — nie separat vom Aufrufer entgegengenommen, damit YAML und Spalten nicht auseinanderlaufen. `id` = die ast-grep-eigene `id` aus der YAML (menschenlesbarer Slug, z. B. `"ts-no-console-log"`), direkt als SQLite-Primärschlüssel — keine separate UUID mehr nötig.
- Das separate `rewrite`-Feld aus dem Scaffold entfällt — ast-grep unterstützt `fix:` bereits innerhalb derselben YAML.
- `description` bleibt ein eigenes norma-Feld (länger/pädagogischer als ast-greps knapp gehaltenes `message`).

Festgehalten als [ADR 0002](../../../docs/adr/0002-pattern-single-language-full-rule-config.md).

**Resultierendes SQLite-Schema:**

```sql
CREATE TABLE patterns (
    id          TEXT PRIMARY KEY,   -- aus der YAML extrahiert, z. B. "ts-no-console-log"
    name        TEXT NOT NULL,      -- norma-Katalogname, kann sich über Sprachvarianten wiederholen
    description TEXT NOT NULL,      -- norma-eigene, längere Erklärung
    category    TEXT,               -- frei, z. B. "code-quality", "creational"
    language    TEXT NOT NULL,      -- SupportLang-Alias: "java" | "python" | "rust" | "typescript"
    severity    TEXT NOT NULL,      -- aus der YAML abgeleitet, gespiegelt für Queries
    rule        TEXT NOT NULL,      -- vollständige ast-grep-RuleConfig-YAML
    enabled     BOOLEAN NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
CREATE INDEX idx_patterns_language ON patterns(language);
```

`register_pattern` validiert Fail-Fast: parst die übergebene `rule`-YAML via `ast_grep_config::from_yaml_string::<SupportLang>`, lehnt bei Parse-Fehler ab, extrahiert `id`/`severity`/`language` daraus, nimmt `name`/`description`/`category` als separate Eingaben entgegen.
