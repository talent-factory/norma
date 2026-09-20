Type: grilling
Status: claimed

## Question

Welche konkreten Design-Patterns, wie viele pro Sprache, sollen für das MVP (v1) ausgewählt werden — für Java, Python, Rust, TypeScript?

Kontext: Der bisherige Scaffold nennt nur grobe Kandidaten (Singleton/Factory für Java, Factory für TypeScript, DEVELOPMENT.md nennt zusätzlich Observer/Strategy als "weitere Default-Patterns"-Idee). [Pattern-Matching-Ansatz](02-pattern-matching-approach.md) legt fest, dass Patterns als `ast-grep`-YAML-Rules gespeichert werden. Zu klären:

- Wie viele Patterns pro Sprache sind für v1 realistisch (z. B. 1-2 pro Sprache statt eines vollen Katalogs)?
- Welche konkreten Patterns eignen sich am besten als Erstes: leicht AST-matchbar, pädagogisch aussagekräftig für FFHS-Studierende, und in allen vier Sprachen sinnvoll übertragbar (dieselben Patterns über alle Sprachen, oder pro Sprache unterschiedliche)?
- Rust ist sowohl Ziel-Sprache für Validierung als auch norma's eigene Implementierungssprache — soll norma sich selbst validieren (dogfooding) als Teil des MVP?
- Gehört das Schreiben der konkreten `ast-grep`-Rules selbst noch zu dieser Spec-Phase (z. B. 1 Beispiel-Rule je Sprache als Nachweis), oder ist das schon Umsetzung und damit ausserhalb dieser Karte?
