Type: grilling
Status: resolved

## Question

Welche konkreten Design-Patterns, wie viele pro Sprache, sollen für das MVP (v1) ausgewählt werden — für Java, Python, Rust, TypeScript?

Kontext: Der bisherige Scaffold nennt nur grobe Kandidaten (Singleton/Factory für Java, Factory für TypeScript, DEVELOPMENT.md nennt zusätzlich Observer/Strategy als "weitere Default-Patterns"-Idee). [Pattern-Matching-Ansatz](02-pattern-matching-approach.md) legt fest, dass Patterns als `ast-grep`-YAML-Rules gespeichert werden. Zu klären:

- Wie viele Patterns pro Sprache sind für v1 realistisch (z. B. 1-2 pro Sprache statt eines vollen Katalogs)?
- Welche konkreten Patterns eignen sich am besten als Erstes: leicht AST-matchbar, pädagogisch aussagekräftig für FFHS-Studierende, und in allen vier Sprachen sinnvoll übertragbar (dieselben Patterns über alle Sprachen, oder pro Sprache unterschiedliche)?
- Rust ist sowohl Ziel-Sprache für Validierung als auch norma's eigene Implementierungssprache — soll norma sich selbst validieren (dogfooding) als Teil des MVP?
- Gehört das Schreiben der konkreten `ast-grep`-Rules selbst noch zu dieser Spec-Phase (z. B. 1 Beispiel-Rule je Sprache als Nachweis), oder ist das schon Umsetzung und damit ausserhalb dieser Karte?

## Answer

**Umfang (Q1):** genau **1 Pattern pro Sprache** für v1 (4 total) — Breite (alle 4 Sprachen funktionieren) vor Tiefe.

**Gemeinsames Konzept (Q2):** Alle vier Patterns setzen dieselbe Idee um — **"kein Debug-Print/-Log in Produktionscode"** — mit je sprachspezifischem `rule`-Text (gemäss [ADR 0002](../../../docs/adr/0002-pattern-single-language-full-rule-config.md): 4 einsprachige Pattern-Zeilen, ein gemeinsamer `name`).

**Kein GoF-Pattern in v1 (Q3):** bewusst verschoben — Singleton/Factory & Co. brauchen mehrteilige AST-Bedingungen, das ist eher "MVP-Pattern-Set v2", nachdem das Fundament (Ticket 4) implementiert ist.

**Rust-Dogfooding (Q4):** Das Rust-Pattern (`println!`-Verbot) wird als Demo gegen norma's eigenen `src/`-Ordner laufen gelassen. Bereits verifiziert: `grep -rn "println!" src/` findet **keine Treffer** (norma nutzt `tracing`-Makros) — die Dogfooding-Demo würde also sauber "bestanden" zeigen.

**Verifiziert (Q5), alle 4 Rules matchen tatsächlich** (erweiterte Probe, [`04-ast-grep-probe.rs`](04-ast-grep-probe.rs)), inkl. Negativ-Fall (Rust: `tracing::info!(...)` löst korrekt **keinen** Treffer aus):

```yaml
# Java
id: no-debug-print-java
message: Avoid System.out.println in production code
severity: warning
language: Java
rule:
  pattern: System.out.println($$$ARGS)
---
# Python
id: no-debug-print-python
message: Avoid print() in production code
severity: warning
language: Python
rule:
  pattern: print($$$ARGS)
---
# Rust
id: no-debug-print-rust
message: Avoid println! in production code
severity: warning
language: Rust
rule:
  pattern: println!($$$ARGS)
---
# TypeScript
id: no-debug-print-typescript
message: Avoid console.log in production code
severity: warning
language: TypeScript
rule:
  pattern: console.log($$$ARGS)
```

Gemeinsamer `name`: "No Debug Print", `category: "code-quality"`. Das tatsächliche Einpflegen dieser vier Rules als Default-Patterns in `pattern_store.rs`/Migrationen ist Umsetzung und bleibt ausserhalb dieser Map.
