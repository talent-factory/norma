# Research 02: Offizieller Rust Style Guide

Ticket: `issues/02-official-rust-style-guide.md` (Map: `map.md`). Stand: 2026-10-28.
Ergebnis ist eine Kandidatenliste mit adopt/defer/reject. **Keine YAML.**

## Kurzfazit

- Der Guide besteht fast vollständig aus Formatierung. Für jeden Formatierungsabschnitt
  ist belegt, dass `rustfmt` das mit Standardoptionen erledigt (Tabelle F, 12 Zeilen, alle reject).
- Über Formatierung hinaus bleiben 41 einzelne Aussagen. Davon wird nur eine Minderheit
  weder von `rustfmt` noch von Clippy-/rustc-Default-Lints geprüft:
  **10 adopt, 5 defer, 26 reject** (Tabellen B, D, R).
- Lohnt sich ein Pack? Ja, aber klein: etwa 10 Regeln, mit hoher Überlappung zu keinem
  Linter. Sie sind überwiegend Item-Ordnung, Doc-Comment-/Attribut-Konventionen und ein
  Namensloch (`macro_rules!`-Namen). Der Rest ist durch Werkzeuge abgedeckt.

## Methode und Quellen

Primärquellen, alle am 2026-10-28 gelesen:

- Guide-Quelltext: `rust-lang/rust`, Branch `master`, `src/doc/style-guide/src/*.md`
  (`README`, `items`, `statements`, `expressions`, `types`, `advice`, `cargo`,
  `principles`, `editions`, `nightly`). Gegengeprüft gegen die ausgelieferte Seite
  <https://doc.rust-lang.org/nightly/style-guide/> (`print.html` enthält zum Beispiel den
  "Rust next style edition"-Eintrag zu nullary calls; die Seiten stimmen inhaltlich überein).
  Die Unterseiten `items`, `statements`, `expressions`, `types`, `advice`, `cargo`,
  `principles`, `editions`, `nightly` und die Startseite (`index.html`) wurden vollständig gelesen.
- rustfmt-Optionen: `rust-lang/rustfmt`, `Configurations.md` (master). Pro Option wurden
  Default und "Stable: Yes/No" maschinell aus dem Dokument gezogen (87 Optionen), nicht aus
  dem Gedächtnis.
- Gegenprobe lokal mit `rustc 1.99.1`, `clippy 0.1.99`, `rustfmt 1.10.0-stable`, Standardkonfiguration:
  - `cargo clippy -- -W help` für Gruppe und Default-Level jedes Lints.
  - rustc-Lintlisten: <https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html>
    (analog `deny-by-default`, `allowed-by-default`).
  - Kleine Testdateien, um zu prüfen, was `rustfmt`/Clippy/rustc **tatsächlich** melden
    (in der Spalte "Beleg" als *getestet* markiert; sonst *Doku*).
- ast-grep-Machbarkeit: `tree-sitter-rust 0.24.2` (`node-types.json`) und
  `ast-grep-language 0.45.3`, die Versionen, die norma verwendet.

Bedeutung der Entscheide (Linter-Grenze aus `map.md`):

- **adopt**: syntaktisch zuverlässig erkennbar **und** weder `rustfmt` noch ein Default-Lint
  (Clippy `style`/`complexity`/`suspicious`/`perf`/`correctness`, rustc warn/deny) prüft es.
- **defer**: syntaktisch machbar und ungeprüft, aber advisory, rauschanfällig oder
  mehrdeutig. Entscheid fällt in Ticket 04/05 mit Messdaten.
- **reject**: mit konkretem Grund (Werkzeug, Typinfo, nicht entscheidbar, TOML).

"Default-Lint" heisst: ohne Konfiguration aktiv. Clippy-Lints der Gruppen `pedantic`,
`restriction`, `nursery` sind per Default aus und zählen **nicht** als Abdeckung (sie sind
unten trotzdem genannt, weil ein Nutzer sie einschalten könnte).

## Lizenz-Befund

- Der Guide liegt im Repo `rust-lang/rust` unter `src/doc/style-guide/`. Dort gibt es
  keine eigene Lizenzdatei, und `REUSE.toml` des Repos enthält **keinen** Sondereintrag für
  `style-guide` (0 Treffer). Damit gilt die Repo-Standardlizenz:
  **MIT OR Apache-2.0** (`COPYRIGHT`: "Except as otherwise noted, Rust is licensed under the
  Apache License, Version 2.0 or the MIT license, at your option";
  Urheber: "The Rust Project Contributors"). Zum Vergleich: `src/doc/rust-by-example/**`
  hat in `REUSE.toml` einen eigenen Eintrag mit zusätzlich CC-BY-SA-4.0, der Style Guide nicht.
- `rustfmt` (`Configurations.md`) steht unter MIT OR Apache-2.0 (README, Abschnitt License).
- Folge: Beide Lizenzen sind permissiv. Paraphrase mit Link, wie in `map.md` vorgesehen,
  ist unproblematisch. Selbst wörtliches Zitieren wäre mit Lizenzhinweis erlaubt. Für Packs
  genügt es, den Guide-Anker zu verlinken. Die Rust-Marken-Richtlinie (README von
  `rust-lang/rust`, Abschnitt Trademark) betrifft Logos und Namen, nicht Textparaphrasen.
- Der Guide betont selbst, dass er den *Default* beschreibt und Abweichungen nicht verbietet
  (Startseite, "The default Rust style"). Ein Pack, das ihn als Pflicht durchsetzt, ist also
  strenger als der Guide selbst. Das gehört als Hinweis in die Pack-Beschreibung.

## Tabelle F: Formatierungsabschnitte (alle reject: rustfmt)

Fast jede Zeile des Guides ist Layout. Die Quelle des Guides sagt selbst, dass Abweichungen
zwischen Guide und rustfmt als Bug gelten ("If the style guide differs from rustfmt, that
may represent a bug", Startseite, Abschnitt Bugs). Jeder Abschnitt ist deshalb
rustfmt-Territorium. Sub-Optionen mit "Stable: No" sind unter Nightly-only abschaltbar,
ihr *Default-Verhalten* (der Guide-Stil) gilt trotzdem immer.

| # | Abschnitt (Anker) | Entscheid | rustfmt-Beleg | Beleg |
|---|---|---|---|---|
| F1 | Indentation, Zeilenbreite, Block indent (`index#indentation-and-line-width`) | reject | `tab_spaces=4`, `hard_tabs=false`, `max_width=100` (alle Stable: Yes), `indent_style=Block` | Doku |
| F2 | Trailing commas (`index#trailing-commas`) | reject | `trailing_comma="Vertical"` (Option unstable, Default-Verhalten aktiv) | Doku |
| F3 | Blank lines 0 oder 1 (`index#blank-lines`) | reject | `blank_lines_upper_bound=1`, `blank_lines_lower_bound=0` (unstable, Default aktiv) | Doku |
| F4 | Trailing whitespace (`index#trailing-whitespace`) | reject | rustfmt entfernt es; zusätzlich Zeilenende-Kommentar `fn a() {}  //x` wird auf einen Space normalisiert | getestet |
| F5 | Sorting / Version-Sort (`index#sorting`) | reject | `reorder_imports=true`, `reorder_modules=true` (Stable: Yes); Version-Sort ab `style_edition=2024` (`editions`) | getestet (Imports sortiert) |
| F6 | Attribute-Layout, Argumentlisten, `=`-Spacing (`index#attributes`) | reject | rustfmt setzt jedes Attribut auf eigene Zeile (`#[allow(..)] struct S;` wurde getrennt) | getestet |
| F7 | Items: Funktionen, Tuples, Enums, Structs, Traits, Impls, Module, Generics, `where`, Type Aliases, Assoc Types (`items#...`) | reject | `fn_params_layout`, `brace_style`, `where_single_line`, `struct_lit_width`, `empty_item_single_line` u. a.; Type-Alias-`where`-Umbruch getestet | getestet / Doku |
| F8 | Imports: Layout, Gruppen, Normalisierung (`items#imports-use-statements`) | reject | `use a::{b}` wird `use a::b`, Gruppen werden sortiert (getestet); Merge/Unmerge nur per Option `imports_granularity` (unstable, Default `Preserve`, Guide: "Tools must not merge ... by default") | getestet |
| F9 | Statements: `let`, let-else, Macros in Statement-Position, Semikolon-Layout (`statements`) | reject | `single_line_let_else_max_width=50` (stable), `trailing_semicolon` | Doku |
| F10 | Expressions: Blöcke, Closures, Literale, Indexing, Operatoren, Casts, Calls, Chains, `if`/`match`, Combinable, Ranges, Patterns (`expressions`) | reject | `fn_call_width`, `chain_width`, `array_width`, `single_line_if_else_max_width`, `match_arm_blocks`, `match_arm_leading_pipes="Never"` (stable), `match_block_trailing_comma`, `remove_nested_parens` (stable) | Doku; `|`-Pipes getestet |
| F11 | Types und Bounds (`types`) | reject | allgemeine Layoutlogik von rustfmt | Doku |
| F12 | Editions, Nightly-Frontmatter, Principles (`editions`, `nightly`, `principles`) | reject | `style_edition` (Stable: Yes) steuert, welche Stilversion gilt. Frontmatter ist Nightly-Syntax. `principles` enthält keine Regel. | Doku |

Hinweis zu F8: Die Aussage "Prefer to use multiple imports rather than a multi-line import"
ist in der Guide-Quelle kombiniert mit "tools should not split imports by default". Das ist
eine Layoutpräferenz, die von der Breite abhängt, siehe R24.

## Tabelle B: adopt (10)

Alle zehn Befunde wurden mit Standardkonfiguration gegen `rustfmt`, `clippy` und `rustc`
**getestet** (Testdateien im Scratchpad, nicht im Repo). Keiner löste eine Meldung oder einen
Diff aus. Nodes: `tree-sitter-rust 0.24.2`.

| # | Regel (Paraphrase) | Abschnitt/Anker | Entscheid | Begründung | Machbarkeit (für Ticket 04) |
|---|---|---|---|---|---|
| B1 | `macro_rules!`-Namen in `snake_case` | `advice#names` | adopt | rustc `non_snake_case` nennt Variablen, Methoden, Funktionen, Lifetimes, Felder, Module, **nicht Macros**. Test: `macro_rules! BadMacroName` ergab keinerlei Meldung (Clippy/rustc), rustfmt lässt ihn unverändert. | `macro_definition` hat Feld `name`; Regex auf Namen. |
| B2 | Unit-Struct statt leerem Struct (`struct Foo;` statt `struct Foo {}` / `struct Foo();`) | `items#structs-and-unions`, `items#tuple-structs` | adopt | Clippy `empty_structs_with_brackets` ist `restriction`, also per Default aus. Test: `struct EmptyBrace {}` und `struct EmptyParen();` blieben ungemeldet. Der Guide erlaubt die Ausnahme "if you must" (Codegen), daher Severity niedrig. | `struct_item` ohne Felder in `field_declaration_list`/`ordered_field_declaration_list`. Generische Structs mit `PhantomData` sind separat zu prüfen. |
| B3 | Doc Comments stehen **vor** Attributen | `index#doc-comments` | adopt | Test: `#[repr(C)]` gefolgt von `/// doc` blieb ungemeldet (rustfmt-Diff nur für anderes). Clippy-Lint `empty_line_after_doc_comments` betrifft nur Leerzeilen, nicht die Reihenfolge. | Relationale Regel (`follows`/`precedes`) zwischen `attribute_item` und `line_comment` mit `outer`-Feld. |
| B4 | Doc Comments als `///`, nicht `/** */` | `index#doc-comments` | adopt | Test: `/** block doc */` ungemeldet. `normalize_comments` (unstable, Default `false`) nennt in der Doku nur `/* */` zu `//`; die Wirkung auf `/** */` ist nicht belegt. | `block_comment` mit `outer`-Marker. |
| B5 | `#[path]` auf Modulen vermeiden | `advice#modules` | adopt | Test: `#[path = "x.rs"] mod pathed;` ungemeldet. Clippy-Lints rund um Module (`mod_module_files`) sind `restriction` und prüfen etwas anderes. Legitime Fälle (plattformabhängig) gibt es, daher Severity niedrig. | `attribute_item` mit `path` vor `mod_item`. |
| B6 | `extern crate` muss ganz oben stehen | `items` (Einleitung) | adopt | Test: `extern crate core;` nach `use`/`mod` blieb stehen, weder Diff noch Warnung. `reorder_imports` sortiert nur innerhalb einer Gruppe, verschiebt nicht über andere Items. | Relationale Regel auf `source_file`-Kindern. |
| B7 | `use`- und `mod foo;`-Deklarationen stehen vor anderen Items | `items` (Einleitung) | adopt | Test: `fn early() {}` vor `use`/`mod` blieb ungemeldet. Clippy `arbitrary_source_item_ordering` ist `restriction` und ordnet nach Item-Art, nicht nach dem Guide. | Relationale Regel auf Modulebene. Funktionslokale `use` sind laut Guide-Wortlaut ("module level") auszuklammern; `macro_rules!`/cfg-Fälle sind in 04 zu prüfen. |
| B8 | Imports stehen vor Modul-Deklarationen | `items` (Einleitung) | adopt | Test: `mod inner_decl;` vor `extern crate`/`use` blieb ungemeldet (`reorder_modules` sortiert nur `mod`-Namen in einer Gruppe). | wie B7 |
| B9 | `macro_rules!`-Definition mit `{}` | `items#macro_rules` | adopt | Test: `macro_rules! paren_macro ( ... );` und `[...]`-Variante wurden von rustfmt **nicht** umgeschrieben und von keinem Lint gemeldet. Machbarkeit am unsichersten. | Delimiter sind anonyme Tokens in `macro_definition`; ob ast-grep sie ansprechen kann, muss `test_pattern` in 04 zeigen. |
| B10 | Leerzeichen nach `//` | `index#comments` | adopt | Test: `//no space` und `///no space doc` blieben unverändert (kein `normalize_comments` per Default). Viel Rauschen bei auskommentiertem Code (`//foo();`). Banner `////` und `//!`/`///` ausnehmen. | `line_comment`-Text per Regex. |

## Tabelle D: defer (5)

| # | Regel (Paraphrase) | Abschnitt/Anker | Entscheid | Begründung |
|---|---|---|---|---|
| D1 | `//` statt `/* */` bevorzugen | `index#comments` | defer | Syntaktisch trivial (`block_comment` ohne Doc-Marker), aber der Guide nennt es "recommendations only". Lizenzköpfe und `/* c */`-Inline-Kommentare sind legitim. Nach Dogfooding entscheiden. |
| D2 | Einbuchstabige Namen für Generics | `items#generics` | defer | Echte "Prefer"-Regel. `type_parameters` lässt sich prüfen, aber beschreibende Namen (`Item`, `Key`, `Value`) sind weit verbreitet, hohe Trefferzahl erwartet. In Ticket 05 an `src/` messen. |
| D3 | Reine Kommentarzeilen ≤ 80 Zeichen (bzw. kleinere Restbreite) | `index#comments` | defer | `comment_width`/`wrap_comments` sind unstable und Default aus, also nicht abgedeckt (Test: 100+-Zeichen-Kommentarzeile blieb unverändert). Der Knotentext eines Kommentars enthält keine Einrückung, passt also zum "excluding indentation". Die zweite Bedingung (Restbreite bis 100) braucht die Spalte. |
| D4 | Bei zusammengesetzten Range-Grenzen Klammern setzen | `expressions#ranges` | defer | Die Beispiele des Guides widersprechen sich (siehe Überraschungen). Nur die `binary_expression`-Teilmenge ist eindeutig; `0..n - 1` ist verbreitet, `rustfmt` lässt es stehen (getestet: `0..1 + 2` unverändert). Treffer vorher messen. |
| D5 | Keine Kommentare innerhalb der Funktionssignatur | `items#function-definitions` | defer | "Avoid", nicht "must". Test: `fn b(x: u8, /* c */ y: u8)` blieb unverändert. Syntaktisch prüfbar (Kommentar als Kind von `parameters`), aber advisory. |

## Tabelle R: reject, über Formatierung hinaus (26)

| # | Regel (Paraphrase) | Abschnitt/Anker | Entscheid | Konkreter Grund | Beleg |
|---|---|---|---|---|---|
| R1 | Typen und Enum-Varianten `UpperCamelCase` | `advice#names` | reject | rustc `non_camel_case_types` (warn-by-default, "types, variants, traits and type parameters") | getestet (Variante `snake_variant` gemeldet) |
| R2 | Felder, Funktionen, Methoden, Locals `snake_case` | `advice#names` | reject | rustc `non_snake_case` (warn) | getestet (Feld `CamelField` gemeldet) |
| R3 | Konstanten und `static` in `SCREAMING_SNAKE_CASE` | `advice#names` | reject | rustc `non_upper_case_globals` (warn). Doku nennt nur `static`, der Test zeigt, dass `const lower_const` ebenfalls gemeldet wird. | getestet |
| R4 | Expression-orientiert statt `let x; if ... { x = 1 } ...` | `advice#expressions` | reject | Clippy `needless_late_init` (`style`, warn) | getestet |
| R5 | Keine überflüssigen Klammern bei `if`/`while` | `expressions#control-flow` | reject | rustc `unused_parens` (warn, Gruppe `unused`) | getestet |
| R6 | Bei `extern` die ABI angeben | `items#extern-items` | reject | rustfmt `force_explicit_abi` (stable, Default `true`) schreibt `extern "C"` hin; rustc `missing_abi` (warn) meldet es. Bare `unsafe extern { }` wird ebenfalls zu `unsafe extern "C" {}`. | getestet |
| R7 | Nur ein `derive`-Attribut | `index#attributes` | reject | rustfmt `merge_derives` (stable, Default `true`) | getestet (`#[derive(Debug)] #[derive(Clone)]` wurde zusammengelegt) |
| R8 | Match-Arm nie mit führendem `\|` | `expressions#match` | reject | rustfmt `match_arm_leading_pipes="Never"` (stable) | getestet |
| R9 | `vec!` und ähnliche Array-Macros mit `[]` | `expressions#array-literals` | reject | rustfmt schreibt `vec!{..}`/`vec!(..)` zu `vec![..]` um; zusätzlich Clippy `nonstandard_macro_braces` (`style`, warn) | getestet |
| R10 | Hex-Literale nicht gemischt gross/klein | `expressions#hexadecimal-literals` | reject | Clippy `mixed_case_hex_literals` (`style`, warn). Projektweit einheitliche Schreibweise nur per `hex_literal_case` (stable, Default `Preserve`), und der Guide gibt ausdrücklich keine Empfehlung. | getestet |
| R11 | `where`-Klauseln auf Type Aliases nicht verwenden | `items#where-clauses` | reject | rustc `type_alias_bounds` (warn): "where clauses on type aliases are not enforced" | getestet |
| R12 | Imports: Sortierung, Normalisierung, kein automatisches (Un-)Merge | `items#ordering-of-imports`, `#normalisation`, `#mergingun-merging-imports` | reject | rustfmt `reorder_imports`, `reorder_modules`; Normalisierung findet statt. Zum Merge sagt der Guide selbst, Tools dürften es nicht standardmässig tun, also existiert keine Regel. | getestet |
| R13 | `*t op u` statt `t op &u` | `expressions#binary-operations` | reject | Braucht Typinformation (`t: &T`, `u: U`). Clippy `op_ref` greift nur teilweise; im Test blieb `t == &u` ungemeldet. | getestet |
| R14 | Semikolon, wenn ein Ausdruck Typ `()` hat | `statements#expressions-in-statement-position` | reject | Braucht Typinfo. Clippy `semicolon_if_nothing_returned` ist `pedantic` (aus). Test: `fn unit_tail() { early() }` ungemeldet. Das reine Semikolonsetzen erzwingt rustc. | getestet |
| R15 | Enum-Literale mit Enum-Namen qualifizieren (ausser Prelude) | `expressions#enum-literals` | reject | Ob ein Pfad eine Enum-Variante ist, braucht Namensauflösung. Clippy `enum_glob_use` ist `pedantic`. | Doku |
| R16 | Tuple-Structs mit "more than a few fields" zu benannten Structs machen | `items#tuple-structs` | reject | Kein Schwellwert im Guide, jede Zahl wäre erfunden. | Doku |
| R17 | Enum mit mehrzeiligen Struct-Varianten: Felder in eigenes Struct auslagern | `items#enums` | reject | Der Guide formuliert es als Hinweis ("might be an indication"), nicht als Regel. | Doku |
| R18 | Reservierte Wörter als Namen: `r#`-Präfix oder `_`-Suffix, keine Falschschreibung (`krate`) | `advice#names` | reject | Falschschreibungen sind nicht abschliessend aufzählbar (`krate`, `typ`, ...). Zudem ist das Beispiel `r#crate` des Guides ungültig (siehe Überraschungen). | getestet (`r#crate` führt zum Fehler "`crate` cannot be a raw identifier") |
| R19 | Sehr kurze `where`-Klausel: Inline-Bound bevorzugen | `items#where-clauses` | reject | "very short" ist nicht definiert. | Doku |
| R20 | Äussere statt innere Attribute bevorzugen | `index#attributes` | reject | "where possible" ist syntaktisch nicht entscheidbar. Crate-/Modulattribute müssen innen stehen. | Doku |
| R21 | `//!`/`/*! */` nur für Modul-/Crate-Doku | `index#doc-comments` | reject | rustc lehnt falsch platzierte innere Doc Comments als Fehler `E0753` ab; der Rest (am Anfang eines Funktionskörpers) ist selten. | getestet |
| R22 | Kommentare als vollständige Sätze (gross, Punkt) | `index#comments` | reject | Natürliche Sprache, nicht syntaktisch. | Doku |
| R23 | Keine Kommentare auf der Zeile einer Klammer | `expressions#blocks` | reject | Zeilenposition, nicht AST-Struktur. Test: `fn c() { /* in brace */ }` blieb ohne Meldung (rustfmt zog nur die Klammer um). | getestet |
| R24 | Mehrere Imports statt mehrzeiliger Listen | `items#large-list-imports` | reject | Hängt von der Zeilenbreite ab ("does not fit"). Tools sollen laut Guide nicht splitten. | Doku |
| R25 | `Cargo.toml`: Formatierung (Sortierung, Quoting, Arrays, Tabellen) | `cargo#formatting-conventions` | reject | `ast-grep-language 0.45.3` enthält **kein TOML** (28 Sprachen, `impl_lang!`-Liste geprüft). Kein offizieller Formatter dafür. | getestet (Quelltext) |
| R26 | `Cargo.toml`: Metadaten (authors, SPDX-license, homepage, description) | `cargo#metadata-conventions` | reject | Gleiche Ursache wie R25. Zusätzlich validiert Cargo/crates.io SPDX-Ausdrücke selbst. | Doku |

Zählung der drei Tabellen B, D, R: 10 + 5 + 26 = 41 Einzelaussagen, plus 12 Formatierungszeilen
in Tabelle F (alle reject) = 53 Zeilen insgesamt, davon 38 reject, 10 adopt, 5 defer.

## Nightly-only rustfmt-Optionen mit Bezug zu den Kandidaten

Alle als "Stable: No" in `Configurations.md` markiert; wirksam nur auf Nightly mit
`unstable_features = true`. Sie ändern nichts am Befund, weil sie per Default aus sind.

| Option | Default | Bezug |
|---|---|---|
| `normalize_comments` | `false` | `/* */` zu `//` (D1). Wirkung auf `/** */` (B4) nicht dokumentiert. |
| `normalize_doc_attributes` | `false` | wandelt `#[doc]`/`#![doc]` in `///`/`//!`, nicht `/** */` (B4). |
| `wrap_comments`, `comment_width` | `false`, `80` | Kommentarbreite (D3). Kein Wrap bei URLs und Markdown-Überschriften. |
| `format_code_in_doc_comments` | `false` | Code in Doc Comments; im Guide nicht gefordert. |
| `imports_granularity`, `group_imports` | `Preserve` | Merge und Gruppierung; der Guide verbietet Default-Merge (R12). |
| `error_on_line_overflow` | `false` | würde überlange Kommentarzeilen melden (D3), aber Nightly. |
| `condense_wildcard_suffixes` | `false` | nicht Teil des Guides. |

Stabile Optionen mit Guide-Bezug: `max_width`, `tab_spaces`, `hard_tabs`, `reorder_imports`,
`reorder_modules`, `merge_derives`, `match_arm_leading_pipes`, `force_explicit_abi`,
`hex_literal_case`, `remove_nested_parens`, `style_edition`.

## Was mich überrascht hat

1. **Der Guide widerspricht rustc.** `advice#names` empfiehlt `r#crate` für verbotene Namen.
   `rustc` weist `fn rawy(r#crate: i32)` ab: "`crate` cannot be a raw identifier". Das gilt auch
   für `self`, `super`, `Self`. Die Regel ist deshalb (R18) nicht belastbar.
2. **Der Guide widerspricht sich bei Ranges.** Derselbe Abschnitt schreibt `..x.len()` ohne
   Klammern vor und verlangt für zusammengesetzte Grenzen `(x.f)..(x.f.len())`. Das macht D4
   uneindeutig.
3. **Die Lücke ist grösser als erwartet.** Entgegen der Erwartung "fast alles rustfmt" prüft
   `rustfmt` bei Items praktisch nichts über Gruppen hinweg (B6 bis B8), Kommentare nur mit
   unstable Optionen und Namen gar nicht. Gerade `macro_rules!`-Namen (B1) sind eine reale
   Lücke in rustc **und** Clippy.
4. **Viele Naming-Regeln sind bereits Default-Lints in rustc, nicht Clippy.** Die
   Linter-Grenze in `map.md` sagt "rustfmt oder Clippy (Default-Lints)". rustc-Lints
   (`non_*_case_*`, `unused_parens`, `missing_abi`, `type_alias_bounds`) gehören sinngemäss
   dazu. In der Entscheidungsliste sind sie als reject geführt; die Map sollte das präzisieren.
5. **Der Guide ist teilweise "nightly" und im Fluss.** `editions` kennt "Rust next style
   edition" (Never break within nullary calls) und Rust 2024 mit Version-Sort, das laut
   Guide "currently nightly-only" ist. Im Guide selbst verschieben sich also
   Regeln; Pack-Regeln sollten nur Stilelemente nehmen, die über Editions hinweg stabil sind
   (B1 bis B10 sind es).
6. **Der Guide schreibt von sich aus keine Verbindlichkeit vor.** Er empfiehlt Tools, den
   Default zu folgen, verbietet aber nicht ausdrücklich Abweichungen. Ein Pack ist
   damit strenger als die Quelle.
7. `print.html` der Nightly-Seite entspricht dem Master-Quelltext, die Seite `editions.html`
   enthielt beim naiven `grep` das Wort "nullary" jedoch nicht, weil Tags dazwischenliegen.
   Das ist kein Inhaltsunterschied.

## Für Ticket 04 (Übergabe)

- Vorschlag für Rule-IDs: Abschnitts-Anker plus Kürzel, zum Beispiel `style-names-macro-snake-case`,
  `style-items-order-use-before-items`, `style-doc-comments-before-attributes`.
- Zu belegen per `test_pattern`: Relationale Regeln B3, B6 bis B8 (`follows`/`precedes` mit
  `stopBy`), Delimiter-Zugriff für B9, Doc-Marker-Felder für B3/B4.
- Offene Fragen: Soll B2 `PhantomData`-Structs ausnehmen? Gehören B7/B8 auch für `#[cfg]`-markierte
  `use` am Dateiende? Erst nach dem Dogfooding-Check (Ticket 05) ist die Rauschquote bekannt.
- Überschneidung mit Ticket 01 (Google) und 03 (API Guidelines): Namen- und Kommentarregeln
  tauchen vermutlich auch im Google-Guide auf. Wenn dessen Formulierung präziser ist, verweist
  dieser Katalog (B1, B10) dorthin, siehe `map.md` Notes "Überschneidungen".

## Quellen

- <https://doc.rust-lang.org/nightly/style-guide/> (`index`, `items`, `statements`, `expressions`,
  `types`, `advice`, `cargo`, `principles`, `editions`, `nightly`)
- `rust-lang/rust`: `src/doc/style-guide/src/*.md`, `COPYRIGHT`, `LICENSE-MIT`, `REUSE.toml`, `README.md`
- `rust-lang/rustfmt`: `Configurations.md`, `README.md`
- <https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html>, `.../deny-by-default.html`,
  `.../allowed-by-default.html`
- Clippy-Default-Level: `cargo clippy -- -W help` (clippy 0.1.99)
- `tree-sitter-rust 0.24.2` (`src/node-types.json`), `ast-grep-language 0.45.3` (`src/lib.rs`)
