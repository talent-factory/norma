Type: grilling
Status: resolved

## Question

Welches GoF-Pattern-Set (v2) soll norma zusätzlich zu den vier MVP-`no-debug-print`-Patterns ausliefern, und mit welcher konkreten, gegen echtes `ast-grep-core` verifizierten `rule`-YAML pro Sprache?

Kontext: Ticket 05 (MVP-Pattern-Set-Scope) hat GoF-Patterns bewusst auf v2 verschoben, weil sie "mehrteilige AST-Bedingungen" brauchen. `DEVELOPMENT.md`s "Next Steps" nennt Singleton/Factory/Observer/Strategy als Kandidaten. Zu klären:

- Welche vier Patterns, in welcher Sprachabdeckung (4×4-Matrix oder selektiv)?
- Was genau erkennt jedes Pattern -- fehlerhafte Implementierung, Anti-Pattern/Overuse, oder reine Präsenz?
- Sind die 16 resultierenden `rule`-YAMLs tatsächlich matchbar (Positiv- und Negativfall) gegen `ast-grep-core` 0.45.3?

## Answer

**Umfang:** genau die vier in `DEVELOPMENT.md` genannten Patterns -- Singleton, Factory, Observer, Strategy -- **volle 4×4-Matrix** (16 Regeln total), keine Sprache ausgelassen.

**Kategorien** (GoF-korrekt statt `code-quality`): Singleton/Factory -> `creational`, Observer/Strategy -> `behavioral`.

**Erkennungsrichtung, gemischt pro Pattern:**

| Pattern | Richtung | Severity | ID-Schema |
|---|---|---|---|
| Singleton | (a) fehlerhafte Implementierung: sieht aus wie ein Singleton (`private static instance`-Feld bzw. äquivalent), aber der Konstruktor ist **nicht** privat -- jeder kann die Kapselung umgehen | `warning` | `singleton-quality-<lang>` |
| Factory | (b) Anti-Pattern/Overuse: `if`/`elif`-Kette, die je Zweig einen **anderen Typ direkt konstruiert** (`new $TYPE(...)`) -- Symptom für "hier fehlt eine Factory" | `warning` | `factory-overuse-<lang>` |
| Observer | (c) reine Präsenz, kein Werturteil: Klasse/Struct hat sowohl eine Listener-/Observer-Sammlung als auch eine `notify`-artige Methode | `info` | `observer-presence-<lang>` |
| Strategy | (b) Anti-Pattern/Overuse: `if`/`elif`-Kette, die je Zweig eine **andere Methode** aufruft (Verhalten statt Konstruktion) -- Symptom für "hier fehlt eine Strategy" | `warning` | `strategy-overuse-<lang>` |

Geteilter `name` je Pattern über die 4 Sprachvarianten (ADR 0002): "Singleton Implementation Quality", "Factory Overuse (Type Switch)", "Observer Presence", "Strategy Overuse (Type Switch)".

**Engine-Voraussetzung (siehe auch die zugehörige Design-Spec):** `observer-presence-*` nutzt `severity: info`. Damit ein informativer Fund nicht fälschlich den `score` senkt, muss `pattern_engine::validate` künftig nur `warning`/`error`-Treffer als Score-mindernden Verstoss zählen -- das ist Teil dieser Spec, nicht mehr rein additiv wie das MVP.

**Verifiziert (Q3):** alle 16 Regeln wurden gegen ein isoliertes Scratch-Cargo-Projekt (`ast-grep-core`/`-config`/`-language` `=0.45.3`, exakt wie im Hauptprojekt gepinnt) mit je einem Positiv- und Negativfall geprüft -- 32/32 Probes grün. Zwei Stolpersteine dabei, für künftige Regel-Autor:innen festgehalten:

1. **`has` ist standardmässig nur "direktes Kind"**, nicht "irgendein Nachfahre" -- eine Bedingung wie "Klasse hat irgendwo ein Feld X" (das tatsächlich zwei Ebenen tiefer sitzt, z. B. Java: `class_declaration` -> `class_body` -> `field_declaration`) braucht **`stopBy: end`** auf der `has`-Regel, sonst matcht sie nie.
2. **`pattern` auf einem Fragment, das nicht für sich allein gültiger Code ist** (z. B. `private static $TYPE instance;` als Java-Feld, `private static instance: $TYPE;` als TS-Feld) **schlägt ohne Kontext fehl** -- es braucht die Langform `pattern: { context: '<gültiger umgebender Code>', selector: <kind> }`.

Die 16 verifizierten Regeln:

```yaml
# Singleton -- Java
id: singleton-quality-java
message: Class looks like a Singleton (private static instance field) but its constructor is not private
severity: warning
language: Java
rule:
  kind: class_declaration
  all:
    - has:
        stopBy: end
        kind: field_declaration
        pattern:
          context: 'class C { private static $TYPE instance; }'
          selector: field_declaration
    - has:
        stopBy: end
        kind: constructor_declaration
        not:
          has:
            kind: modifiers
            regex: private
---
# Singleton -- Python
id: singleton-quality-python
message: Class has a Singleton-style `_instance` attribute but no `__new__` guard enforcing a single instance
severity: warning
language: Python
rule:
  kind: class_definition
  all:
    - has:
        stopBy: end
        kind: assignment
        pattern: _instance = None
    - not:
        has:
          stopBy: end
          kind: function_definition
          has:
            field: name
            regex: '^__new__$'
---
# Singleton -- Rust
id: singleton-quality-rust
message: A public `new()` next to a module-level `static INSTANCE` defeats the Singleton -- callers can construct extra instances directly
severity: warning
language: Rust
rule:
  kind: source_file
  all:
    - has:
        stopBy: end
        kind: static_item
        pattern: static INSTANCE $$$REST
    - has:
        stopBy: end
        kind: function_item
        pattern: pub fn new($$$PARAMS) -> $$$RET { $$$BODY }
---
# Singleton -- TypeScript
id: singleton-quality-typescript
message: Class looks like a Singleton (private static instance field) but its constructor is not private
severity: warning
language: TypeScript
rule:
  kind: class_declaration
  all:
    - has:
        stopBy: end
        kind: public_field_definition
        pattern:
          context: 'class C { private static instance: $TYPE; }'
          selector: public_field_definition
    - has:
        stopBy: end
        kind: method_definition
        pattern:
          context: 'class C { constructor() {} }'
          selector: method_definition
        not:
          has:
            regex: private
---
# Factory -- Java
id: factory-overuse-java
message: Type-switch construction (if/else-if each calling `new`) suggests a Factory would be a better fit
severity: warning
language: Java
rule:
  pattern: |
    if ($COND1) {
      new $TYPE1($$$ARGS1);
    } else if ($COND2) {
      new $TYPE2($$$ARGS2);
    }
---
# Factory -- Python
id: factory-overuse-python
message: Type-switch construction (if/elif each instantiating a different class) suggests a Factory would be a better fit
severity: warning
language: Python
rule:
  pattern: |
    if $COND1:
        $TYPE1($$$ARGS1)
    elif $COND2:
        $TYPE2($$$ARGS2)
---
# Factory -- Rust
id: factory-overuse-rust
message: Type-switch construction (if/else-if each building a different struct literal) suggests a Factory function would be a better fit
severity: warning
language: Rust
rule:
  pattern: |
    if $COND1 {
        $TYPE1 { $$$FIELDS1 }
    } else if $COND2 {
        $TYPE2 { $$$FIELDS2 }
    }
---
# Factory -- TypeScript
id: factory-overuse-typescript
message: Type-switch construction (if/else-if each calling `new`) suggests a Factory would be a better fit
severity: warning
language: TypeScript
rule:
  pattern: |
    if ($COND1) {
      new $TYPE1($$$ARGS1);
    } else if ($COND2) {
      new $TYPE2($$$ARGS2);
    }
---
# Observer -- Java
id: observer-presence-java
message: Class has a Listener collection and a notify-style method -- an Observer-shaped construct
severity: info
language: Java
rule:
  kind: class_declaration
  all:
    - has:
        stopBy: end
        kind: field_declaration
        regex: Listener
    - has:
        stopBy: end
        kind: method_declaration
        regex: notify
---
# Observer -- Python
id: observer-presence-python
message: Class has an observers collection and a notify-style method -- an Observer-shaped construct
severity: info
language: Python
rule:
  kind: class_definition
  all:
    - has:
        stopBy: end
        kind: assignment
        regex: observers
    - has:
        stopBy: end
        kind: function_definition
        regex: notify
---
# Observer -- Rust
id: observer-presence-rust
message: Struct has an observers-style field -- an Observer-shaped construct
severity: info
language: Rust
rule:
  kind: struct_item
  has:
    stopBy: end
    kind: field_declaration
    regex: observers
---
# Observer -- TypeScript
id: observer-presence-typescript
message: Class has an observers collection and a notify-style method -- an Observer-shaped construct
severity: info
language: TypeScript
rule:
  kind: class_declaration
  all:
    - has:
        stopBy: end
        kind: public_field_definition
        regex: observers
    - has:
        stopBy: end
        kind: method_definition
        regex: notify
---
# Strategy -- Java
id: strategy-overuse-java
message: Type-switch behavior selection (if/else-if each calling a different method) suggests a Strategy would be a better fit
severity: warning
language: Java
rule:
  pattern: |
    if ($COND1) {
      $METHOD1($$$ARGS1);
    } else if ($COND2) {
      $METHOD2($$$ARGS2);
    }
---
# Strategy -- Python
id: strategy-overuse-python
message: Type-switch behavior selection (if/elif each calling a different function) suggests a Strategy would be a better fit
severity: warning
language: Python
rule:
  pattern: |
    if $COND1:
        $METHOD1($$$ARGS1)
    elif $COND2:
        $METHOD2($$$ARGS2)
---
# Strategy -- Rust
id: strategy-overuse-rust
message: Type-switch behavior selection (if/else-if each calling a different function) suggests a Strategy would be a better fit
severity: warning
language: Rust
rule:
  pattern: |
    if $COND1 {
        $METHOD1($$$ARGS1);
    } else if $COND2 {
        $METHOD2($$$ARGS2);
    }
---
# Strategy -- TypeScript
id: strategy-overuse-typescript
message: Type-switch behavior selection (if/else-if each calling a different method) suggests a Strategy would be a better fit
severity: warning
language: TypeScript
rule:
  pattern: |
    if ($COND1) {
      $METHOD1($$$ARGS1);
    } else if ($COND2) {
      $METHOD2($$$ARGS2);
    }
```

Das tatsächliche Einpflegen dieser 16 Regeln als `DefaultPattern`-Einträge sowie die `pattern_engine::validate`-Anpassung (Score/`passed` ignorieren `off`/`hint`/`info`) sind Umsetzung und bleiben ausserhalb dieser Karte -- siehe die zugehörige Implementierungs-Spec/-Plan.
