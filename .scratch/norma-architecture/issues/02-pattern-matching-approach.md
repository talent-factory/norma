Type: grilling
Status: resolved

## Question

Welcher Pattern-Matching-Ansatz für norma: `ast-grep-core` vs. rohes `tree-sitter` (bereits Dependency) vs. "Regex bleibt für v1, AST erst v2"?

Kontext: DEVELOPMENT.md priorisiert eine AST-Grep-Integration als "High priority", der aktuelle Scaffold nutzt aber noch Regex (`src/pattern_engine.rs`, 113 Zeilen). `tree-sitter` ist bereits als direkte Dependency in `Cargo.toml` eingetragen; `ast-grep-core` baut selbst auf tree-sitter-Parsern auf. Zu klären:

- Reicht `ast-grep-core`s eigenes Pattern-/Rule-Format (inkl. Multi-Language-Support für Java/Python/Rust/TypeScript), oder ist rohes tree-sitter-Query-Matching flexibler/nötig?
- Wie hoch ist der Integrationsaufwand für jeden Ansatz, realistisch für ein MVP?
- Bleibt Regex als Übergangslösung für v1 vertretbar, oder ist der Sprung direkt zu AST-Matching nötig, weil Regex für Java/Python/Rust/TypeScript-Patterns zu unzuverlässig ist?
- Ergebnis prägt direkt das Pattern-Datenmodell/-Schema (siehe „Not yet specified" auf der Map).

## Answer

Entscheidung: **`ast-grep-core`** (nicht rohes `tree-sitter`, kein Regex-Zwischenschritt).

Begründung: `ast-grep-core` ist mit v0.45.3 (Release vor 19 Tagen, 182 Versionen, 2,5 Mio. Downloads, MIT) aktiv und reif gepflegt — ein völlig anderes Profil als `mcpkit`. Alle vier MVP-Sprachen (Java, Python, Rust, TypeScript) sind über `ast-grep-language` mit gebündelten, aktuellen tree-sitter-Grammatiken abgedeckt. String-Patterns mit Metavariablen (`$A`, `$$$A`) decken die meisten "shape"-Matches ab, wie sie Design-Pattern-Checks brauchen; für positionale/relationale Bedingungen steht das YAML-Rule-Format (`kind`/`field`/`has`/`inside`) bereit — beides bleibt als Text speicherbar. Rohes tree-sitter würde denselben Matching-Komfort (Metavariablen, Pattern-Syntax) selbst nachbauen, ohne erkennbaren Vorteil für norma.

Einzige Einschränkung: die Rust-API von `ast-grep-core` ist laut offizieller Doku explizit "not stable yet" — deshalb Standing Preference (siehe Map-Notes): Version gepinnt halten, kein automatisches Bumpen auf neue Minor-Versionen ohne bewussten Test.

Für das Pattern-Datenmodell: das `rule`-Feld eines Patterns wird **einheitlich im YAML-Rule-Format** gespeichert (auch einfache Patterns als `rule: { pattern: "$A" }`-Kurzform) — ein einziger Parser-Pfad, erweiterbar auf relationale Regeln ohne Schema-Migration.

Aktuell in `Cargo.toml` gepinnte Versionen (`ast-grep-core = "0.26"`, `tree-sitter = "0.20"`) sind veraltet und müssen beim Implementieren auf aktuelle, zueinander passende Versionen angehoben werden (`ast-grep-core` und die Sprachgrammatiken über `ast-grep-language` bezogen statt `tree-sitter` als direkte Dependency).

Quellen/Details: siehe Recherche-Zitate oben in der Diskussion (keine separate Research-Datei — direkt per WebSearch/WebFetch während der Grilling-Session ermittelt: [ast-grep-core auf crates.io](https://crates.io/crates/ast-grep-core), [Rust-API-Referenz](https://ast-grep.github.io/reference/api.html), [Core Concepts](https://ast-grep.github.io/advanced/core-concepts.html), [ast-grep-language Cargo.toml](https://github.com/ast-grep/ast-grep/blob/main/crates/language/Cargo.toml)).
