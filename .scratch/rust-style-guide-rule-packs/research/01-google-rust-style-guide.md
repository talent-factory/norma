# Research 01: Google Rust Style Guide

Ticket: `../issues/01-google-rust-style-guide.md`. Kontext: `../map.md`.
Stand: 2026-10-28. Es werden keine Regeltexte kopiert, nur paraphrasiert; jeder Kandidat verweist auf den Abschnitts-Anker.

## Quellen

- Primärquelle: <https://google.github.io/styleguide/rustguide.html> (vollständig gelesen, alle Abschnitte inkl. Unterabschnitte; Anker `#...` unten beziehen sich auf diese Seite).
- Quelltext der Seite: `rustguide.md` im Repo `google/styleguide`, Branch `gh-pages`.
- Lizenz: `README.md` und `LICENSE` im selben Repo.
- Clippy-Lints: offizielle Lint-Ausgabe von `clippy-driver -W help` (Clippy 0.1.99, 2026-10-13, lokal installiert). Gruppe und Default-Level stammen daraus. Die Web-Liste (`rust-lang.github.io/rust-clippy/master`) ist JavaScript-getrieben und `lints.json` liefert 404, daher nicht verwendbar. "Default" heisst hier: Level `warn`/`deny` ohne Zusatzkonfiguration (Gruppen style, complexity, perf, suspicious, correctness). Gruppen `pedantic`, `restriction`, `nursery` sind `allow` und gelten als nicht abgedeckt.
- rustc-Lints und rustfmt-Optionen: `clippy-driver -W help` bzw. `rustfmt --help=config` (rustfmt 1.10.0-stable).
- API-Guidelines-IDs: <https://rust-lang.github.io/api-guidelines/checklist.html>.

## Lizenz-Befund

- Die Guides im Repo `google/styleguide` stehen unter **CC BY 3.0 (Unported)**. Code-Samples stehen unter **Apache 2.0**. (README, Abschnitt "Licensing"; `LICENSE` enthält den CC-BY-3.0-Rechtstext.)
- Paraphrase mit Link ist zulässig. CC BY 3.0 erlaubt Bearbeitungen und Weitergabe, verlangt aber Namensnennung (Urheber, Titel, Link zur Lizenz) und bei Bearbeitungen einen Hinweis, dass es eine Bearbeitung ist (Rechtstext Abschnitt 4(b): "credit identifying the use of the Work in the Adaptation"). Die Pflicht ist nur erfüllt, wenn die Pack-Dateien einen Quellenhinweis tragen.
- Folge für Ticket 04: In jedes Pack einen Kurzhinweis ("Regeln paraphrasiert nach dem Google Rust Style Guide, CC BY 3.0, <Link>; von norma bearbeitet") aufnehmen. Wörtliche Zitate vermeiden. Die Code-Samples des Guides nicht ungeprüft in `message`-Texte übernehmen (Apache 2.0 verlangt bei Übernahme ebenfalls Hinweise).
- Die HTML-Seite selbst zeigt keinen Lizenzhinweis, nur "This site is open source" mit Link zum Repo. Die Lizenz steht ausschliesslich im Repo. Das ist leicht zu übersehen.
- Keine offene Frage zur Zulässigkeit; kein Rechtsrat, nur Lektüre der Lizenztexte.

## Kriterien

- **adopt**: Das Muster ist rein syntaktisch zuverlässig erkennbar, und rustfmt und die Clippy-Default-Lints prüfen es nicht. Ein Treffer entspricht dem vom Guide abgelehnten Konstrukt. Wo der Guide begründete Ausnahmen erlaubt, ist der Treffer ein Hinweis (niedrige Severity).
- **defer**: Syntaktisch teilweise erkennbar, aber Trefferqualität, Ausnahmen oder Kontext (Library vs. Application) sind ungeklärt. Ein Spike in Ticket 04 oder der Plausibilitätscheck gegen `src/` entscheidet.
- **reject**: Mit Grund: rustfmt, Clippy-Default-Lint (benannt), braucht Typinfo/Namensauflösung, Semantik/Urteil, Build-/Codebase-spezifisch (Chromium, Fuchsia) oder kein Verstoss-Signal im Quelltext.
- **Verweis**: Der Guide verweist auf offiziellen Style Guide oder API Guidelines. Bewertung dort (Ticket 02 bzw. 03).

## Zusammenfassung

| Entscheid | Anzahl | IDs |
|---|---|---|
| adopt | 20 | G-07 G-08 G-09 G-17 G-21 G-24 G-28 G-44 G-50 G-52 G-65 G-67 G-74 G-75 G-78 G-79 G-81 G-84 G-85 G-87 |
| defer | 6 | G-06 G-18 G-29 G-33 G-83 G-88 |
| reject | 57 | (siehe Tabellen) |
| Verweis | 6 | G-01 G-25 G-27 G-49 G-62 G-68 |
| **Total Kandidaten** | **89** | |

Die Zählung stammt aus den Tabellen unten (maschinell ausgezählt). Mehrere Regeln des Guides sind in einen Kandidaten zusammengefasst, wenn sie dieselbe Begründung teilen (z. B. G-15, G-69); Regeln mit unterschiedlicher Prüfbarkeit sind getrennt (z. B. G-20/G-21, G-28/G-29, G-84/G-85/G-86).

## Kandidaten

Anker = Fragment der Seite `rustguide.html`. "[chrome]"/"[Fuchsia]" im Titel markieren codebase-spezifische Regeln des Guides.

### Rahmen und Code-Organisation

| ID | Regel (paraphrasiert) | Anker | Entscheid | Begründung |
|---|---|---|---|---|
| G-01 | Offiziellen Rust Style Guide und API Guidelines befolgen, soweit kein Konflikt | `#follow-public-rust-style-rules` | Verweis | Verweis auf offiziellen Style Guide (Ticket 02) und API Guidelines (Ticket 03). |
| G-02 | API-Design-Regeln gelten für Library-Code verbindlich, für Application-Code als starke Empfehlung | `#apply-api-design-rules-in-context` | reject | Kontextregel; ob eine Datei Library oder Application ist, steht nicht im Quelltext. Wirkt als Vorbehalt auf G-22, G-23, G-24, G-33. |
| G-03 | Kleine Crates bevorzugen [chrome] | `#prefer-small-crates-chrome` | reject | Build-Struktur (`rust_static_library`), Chromium-spezifisch. |
| G-04 | Flaches Dateilayout pro Crate [chrome] | `#use-flat-file-layout-for-rust_library-sources-chrome` | reject | Dateisystem-Layout, Chromium-spezifisch; widerspricht dem Cargo-Standard. |
| G-05 | Dateien und Leaf-Module nach dem primären Item benennen (`mod foo` / `foo.rs`) | `#name-source-files-and-leaf-modules-after-the-primary-user-defined-item` | reject | Dateiname gegen Inhalt; "primäres Item" ist ein Urteil, ast-grep prüft Knoten, keine Dateinamen. |
| G-06 | In Nicht-Leaf-Modulen nur `mod`-Deklarationen und Re-Exports | `#dont-implement-apis-in-non-leaf-modules` | defer | Erkennbar als Datei mit `mod x;` plus Item-Definitionen, aber Treffer wäre die ganze Datei, "kleine Menge Kern-Code" ist erlaubt, und `main.rs` mit `mod` plus `fn main` würde fälschlich anschlagen. Spike nötig. |
| G-07 | Keine Glob-Imports (Ausnahmen: `use super::*` in Inline-Modulen, `use Enum::*` in Enum-Code, `pub use sub::*` in Nicht-Leaf-Modulen, C++-Interop) | `#dont-use-glob-imports` | adopt | Rein syntaktisch (`use ...::*`). Clippy `wildcard_imports` und `enum_glob_use` sind pedantic, also nicht default. `use super::*` in `mod tests` ausnehmen (in norma `src/` 6 Treffer, alle diese Form). |
| G-08 | Keine eigenen Preludes definieren | `#dont-use-glob-imports` (letzter Absatz) | adopt | Syntaktisch (`mod prelude` / `pub mod prelude`). Kein Clippy-Lint. Klein, aber eindeutig. |
| G-09 | Ein `use` importiert aus einem Modul; kein `::` innerhalb von `{..}` | `#a-use-statement-should-import-items-from-one-module` | adopt | Syntaktisch zuverlässig (verschachtelter Pfad im `use_list`). Der Guide verweist auf rustfmt `imports_granularity = Module`, die Option fehlt aber in `rustfmt --help=config` der Stable-Toolchain (nur Nightly), also prüft stable rustfmt das nicht. **Grenzfall** zur Linter-Grenze, bitte in Ticket 04 oder per Entscheid bestätigen. |
| G-10 | Traits lokal in Funktionskörpern importieren, ausser breite Nutzung in der Datei | `#prefer-locally-scoped-trait-imports` | reject | Ob ein importiertes Item ein Trait ist, braucht Namensauflösung; "extensiv genutzt" ist ein Urteil. |
| G-11 | Top-Level-Funktionen über den Modulnamen aufrufen (`mem::transmute`, nicht `transmute`) | `#qualify-calls-to-top-level-functions-with-a-module-name` | reject | `use a::b;` kann Modul oder Funktion sein; ohne Namensauflösung nicht unterscheidbar. Mehrere weiche Ausnahmen. |
| G-12 | Typen unqualifiziert verwenden (`Regex`, nicht `regex::Regex`) | `#type-names-should-be-imported-and-used-unqualified` | reject | Heuristik über CamelCase-Segmente träfe `io::Error`, `fmt::Result`, `fmt::Formatter` (idiomatisch und im Guide ausdrücklich erlaubt). Guide nennt als Zweck selbst "Reviewer-Streit vermeiden". |
| G-13 | Import-Mechanismus der Codebase nutzen (`chromium::import!`) [chrome] | `#use-the-appropriate-import-mechanism-chrome` | reject | Chromium-spezifisch. |

### Formatierung

| ID | Regel (paraphrasiert) | Anker | Entscheid | Begründung |
|---|---|---|---|---|
| G-14 | Das Formatierungstool der Codebase verwenden | `#use-the-appropriate-formatting-tool` | reject | rustfmt/Prozess. |
| G-15 | rustfmt-Konfiguration: max. 100 Spalten, `format_macro_matchers`, Escapes für unsichtbare Zeichen | `#use-the-appropriate-formatting-tool` | reject | rustfmt-Optionen; Zeilenlänge ist nicht syntaktisch. |
| G-16 | Hex-Literale in Kleinbuchstaben (`hex_literal_case = Lower`) | `#use-the-appropriate-formatting-tool` | reject | rustfmt-Option, in der Stable-Toolchain vorhanden (Default `Preserve`), also dort lösbar. |
| G-17 | `#[rustfmt::skip]` nur für manuelle Ausrichtung oder generierten Code | `#opting-out-of-automatic-formatting` | adopt | Das Attribut ist syntaktisch eindeutig und selten. Jeder Treffer ist ein Anlass zur Begründung (Hinweis-Severity), die Berechtigung der Ausnahme bleibt ein Urteil. Kein Clippy-Lint. |

### Ownership, Funktions-Ein-/Ausgaben, Typen

| ID | Regel (paraphrasiert) | Anker | Entscheid | Begründung |
|---|---|---|---|---|
| G-18 | `Rc`/`Arc` vermeiden, Borrowing und Unique Ownership bevorzugen | `#avoid-rc-and-arc` | defer | Konstrukt ist erkennbar, aber legitime Nutzung (async, Threads) ist die Regel in Servern. In norma `src/` 3 Treffer, aber in einem MCP-Server wäre die Trefferqualität schlecht. Plausibilitätscheck entscheidet; Clippy deckt nur Teilfälle ab (`rc_buffer`, `rc_mutex`, beide restriction). |
| G-19 | `.clone()` nicht als Ersatz für Borrowing | `#prefer-borrows-over-deep-copies` | reject | Absicht nicht syntaktisch; `.clone()` ist allgegenwärtig. Clippy `clone_on_copy` (complexity, default); `redundant_clone` ist nursery. |
| G-20 | `&Box<T>`, `&Vec<T>`, `&String` statt `&T`, `&[T]`, `&str` übergeben | `#accept-flexible-references-for-immutable-data` | reject | Clippy `borrowed_box` (complexity, default) und `ptr_arg` (style, default); der Guide nennt beide selbst. |
| G-21 | `&Option<T>` statt `Option<&T>` übergeben | `#accept-flexible-references-for-immutable-data` | adopt | Syntaktisch im Parametertyp erkennbar. Das passende Clippy `ref_option` ist pedantic, nicht default. Geringer Mehrwert; falls die Linter-Grenze "pedantic" als abgedeckt wertet, auf reject kippen. |
| G-22 | Library-Rückgabewerte als Iterator statt `Vec<T>` | `#iterators-vs-owned-collections-in-return-types` | reject | Sechs Ausnahmen, Library/Application-Kontext; `-> Vec<T>` anzuschlagen wäre fast nur Rauschen. |
| G-23 | Library-Parameter als Iterator/`impl IntoIterator` statt Slice | `#iterators-vs-slices-in-function-parameters` | reject | Wie G-22: Kontext und Urteil, `&[T]` ist meist richtig. |
| G-24 | `impl Into<T>`-Parameter sparsam (Application-Code fast nie) | `#accepting-into` | adopt | Syntaktisch (`impl Into<..>` im Parametertyp). Clippy `impl_trait_in_params` (restriction) trifft jedes `impl Trait`, nicht default. Hinweis-Severity, weil Library-Code es erwägen darf. |
| G-25 | `enum` statt `bool`-Parameter, wenn der Zweck unklar ist | `#prefer-enum-over-bool-in-apis` | Verweis | Verweis auf API Guidelines C-CUSTOM-TYPE (Ticket 03). "Unklar" ist ohnehin ein Urteil; Clippy `fn_params_excessive_bools` (pedantic) prüft nur die Anzahl. |
| G-26 | Fehler melden: `Option` für einfache, `Result` für behebbare, `panic!` für Bugs/Invarianten | `#reporting-errors` | reject | Designentscheid, kein Syntaxsignal. |
| G-27 | Newtype statt Type-Alias bei semantischer Differenz | `#newtypes-versus-type-aliases` | Verweis | Verweis auf API Guidelines C-NEWTYPE (Ticket 03); `type X = ..` allein sagt nichts über die Absicht. |
| G-28 | Tuple-Structs mit mehreren Feldern vermeiden, benannte Felder bevorzugen | `#prefer-named-fields-over-tuples-and-non-newtype-tuple-structs` | adopt | Syntaktisch: Tuple-Struct mit 2 oder mehr Feldern. Newtypes (ein Feld) sind ausgenommen. Kein Clippy-Default-Lint. |
| G-29 | Tupel-Typen in Signaturen und `.0`/`.1`-Zugriffe vermeiden | `#prefer-named-fields-over-tuples-and-non-newtype-tuple-structs` | defer | Ausnahme "sehr lokaler Kontext" ist weit; `(K, V)`-Paare und `enumerate()` sind alltäglich. Spike nötig, ob eine Beschränkung auf `pub fn`-Signaturen trägt. |

### Traits

| ID | Regel (paraphrasiert) | Anker | Entscheid | Begründung |
|---|---|---|---|---|
| G-30 | Trait-Impls erst bei Bedarf (Application) bzw. proaktiv (Low-Level-Library) ergänzen | `#add-trait-implementations-when-they-become-necessary` | reject | Prinzip ohne Syntaxsignal. |
| G-31 | Standard-Traits statt eigener gleichnamiger Methoden (`from_str` -> `FromStr`) | `#implement-known-standard-library-traits-instead-of-custom-methods` | reject | Clippy `should_implement_trait` (style, default); der Guide nennt den Lint selbst. |
| G-32 | `#[derive]` statt manueller Impl, wenn möglich | `#use-derive-if-possible` | reject | Äquivalenz der manuellen Impl ist nicht syntaktisch prüfbar. Teilabdeckung: Clippy `derivable_impls` (complexity, default, nur `Default`). |
| G-33 | In Libraries Generics (`T: Trait`, `impl Trait`) statt `dyn Trait` in Parametern | `#generics-versus-dyn-trait` | defer | `dyn` im Parametertyp ist erkennbar, aber Ausnahme (Codegrösse), Library-Kontext und heterogene Sammlungen machen die Treffer unsicher. |
| G-34 | Traits nur für Generics, `impl`/`dyn Trait` oder Extension-Traits; nicht zur Code-Organisation | `#when-to-define-a-trait-versus-an-inherent-implementation` | reject | Absicht des Traits ist nicht syntaktisch. |
| G-35 | Extension-Traits: Eigentümer-Team, Namenssuffix `Ext` | `#extension-traits` | reject | "Fremder Typ" braucht Namensauflösung. Das Suffix lässt sich nur positiv prüfen, ein Verstoss ist nicht erkennbar. |
| G-36 | Öffentliche Felder: Traits der Teilfelder erwägen | `#specific-guidance-for-important-traits` | reject | Braucht Typinfo. |
| G-37 | Fehlertypen möglichst `Copy` | `#specific-guidance-for-important-traits` | reject | Braucht Typinfo (Feldtypen); "wenn möglich". |
| G-38 | `Copy` bei stabiler Absicht, `Clone` für die meisten Typen | `#specific-guidance-for-important-traits` | reject | Urteil über Zukunft/Ressourcenbezug. |
| G-39 | `Default` implementieren, wenn `new()` existiert | `#specific-guidance-for-important-traits` | reject | Clippy `new_without_default` (style, default); der Guide nennt den Lint. |
| G-40 | `PartialEq`/`PartialOrd` statt `Eq`/`Ord`, wenn nur Operatoren gebraucht werden | `#specific-guidance-for-important-traits` | reject | Empfehlung ohne Verstoss-Signal. Gegenläufig zu Clippy `derive_partial_eq_without_eq` (nursery), das `Eq` ergänzen will. |
| G-41 | Als Map-Key verwendete Typen: `Hash` + `Eq` (ggf. `Ord`) | `#specific-guidance-for-important-traits` | reject | Verwendung steht nicht an der Typdefinition. |
| G-42 | Wichtige `From`-Impls mit `::new()`/`::from_foo()` paaren | `#pair-important-from-impls-with-explicit-constructor-methods` | reject | "Wichtig" ist ein Urteil; Paarung typübergreifend. |
| G-43 | Options-Structs (alle Felder `pub`, "Bag of Options") implementieren `Default` | `#options-structs-should-implement-default` | reject | "Bag of Options" ist semantisch. (Siehe G-39 für den `new()`-Fall.) |

### Nebenläufigkeit, Makros

| ID | Regel (paraphrasiert) | Anker | Entscheid | Begründung |
|---|---|---|---|---|
| G-44 | Nie in einen vergifteten Mutex greifen | `#handling-poison` | adopt | Die gängigen Idiome sind syntaktisch erkennbar: `PoisonError::into_inner` und `lock()...unwrap_or_else(\|e\| e.into_inner())`. Kein Clippy-Lint (in der Lint-Liste nichts zu Poison). Deckt nur diese Idiome, nicht jede Variante. |
| G-45 | Kein `async`/`.await` [chrome] | `#dont-use-async--await-chrome` | reject | Chromium-spezifisch; norma ist async. |
| G-46 | Keine Std-Threads [chrome] | `#dont-start-new-threads-through-the-rust-standard-library-chrome` | reject | Chromium-spezifisch. |
| G-47 | `fuchsia_sync::Mutex` nutzen [Fuchsia] | `#use-fuchsia_syncmutex-fuchsia` | reject | Fuchsia-spezifisch. |
| G-48 | Reihenfolge: reguläres Rust, `macro_rules!`, Codegenerierung, Proc-Macros | `#macros` | reject | Designpräferenz; jede Makrodefinition anzuschlagen wäre kein Verstoss-Signal. |

### Naming

| ID | Regel (paraphrasiert) | Anker | Entscheid | Begründung |
|---|---|---|---|---|
| G-49 | Methoden `as_`/`to_`/`into_` mit passender `self`-Konvention | `#naming-methods` | Verweis | Verweis auf API Guidelines C-CONV (Ticket 03). Clippy `wrong_self_convention` (style, default) deckt es ab; der Guide nennt den Lint. |
| G-50 | Lifetimes beschreibend benennen, besonders Typ-Lifetimes | `#naming-lifetimes` | adopt | Syntaktisch: einbuchstabiger Lifetime in der Deklaration von `struct`/`enum`/`trait`/`impl`. Funktionslokale `'a`/`'b` (bis zu zwei) und `'_` sind erlaubt, also ausnehmen. Clippy `single_char_lifetime_names` ist restriction. Hinweis-Severity (Cow-artige Abstraktionen sind Ausnahme). In norma `src/` 1 Treffer für `<'a>`. |
| G-51 | Generische Parameter kurz und sinnvoll benennen | `#naming-generic-parameters` | reject | "Hinweisend auf das Objekt" ist ein Urteil. |
| G-52 | Konstruktoren heissen `new`; liefert er `Result`, `try_new` | `#naming-initialization-functions` | adopt | Syntaktisch: `fn new(..) -> Result<..>`. Clippy `new_ret_no_self` (style, default) lässt `Result<Self, _>` zu, deckt das also nicht ab. Der Rest der Regel (`new_with_foo`, `from_foo`) ist positiv und nicht prüfbar (siehe G-53). |
| G-53 | `new_with_foo` bei mehreren Konstruktoren, `from_foo` mit `From`-Impl | `#naming-initialization-functions` | reject | Positive Namenskonvention; ein Verstoss hat kein Syntaxsignal. Der Guide ist hier intern uneinheitlich (siehe Auffälligkeiten). |
| G-54 | Extension-Traits mit Suffix `Ext` benennen | `#naming-extension-traits` | reject | Doppelt zu G-35, gleiche Begründung. |
| G-55 | Keine Abkürzungen, die Aussenstehende nicht kennen | `#avoid-abbreviations` | reject | Braucht Wörterbuch/Urteil. |

### Dokumentation

| ID | Regel (paraphrasiert) | Anker | Entscheid | Begründung |
|---|---|---|---|---|
| G-56 | Öffentliche APIs dokumentieren (Hinweis auf `missing_docs`) | `#when-and-how-to-write-documentation` | reject | rustc-Lint `missing_docs` (allow by default, per `#![warn(missing_docs)]` aktivierbar); der Guide nennt ihn selbst. |
| G-57 | Beispiele in Docs | `#when-and-how-to-write-documentation` | reject | Urteil ("genug Beispiele irgendwo"). |
| G-58 | Panic-Fälle dokumentieren | `#when-and-how-to-write-documentation` | reject | Clippy `missing_panics_doc` (pedantic) erkennt transitive Panics besser; eine ast-grep-Näherung (`unwrap`/`panic!` im `pub fn`-Körper ohne `# Panics`) wäre unvollständig und laut. |
| G-59 | Invarianten für `unsafe`-Funktionen/-Traits dokumentieren | `#when-and-how-to-write-documentation` | reject | Clippy `missing_safety_doc` (style, default). |
| G-60 | Gross-/Kleinschreibung und Satzzeichen in Docs | `#when-and-how-to-write-documentation` | reject | Nicht zuverlässig prüfbar. |
| G-61 | Sprachgrundlagen und triviale Wiederholungen nicht dokumentieren | `#when-and-how-to-write-documentation` | reject | Inhaltliches Urteil. |
| G-62 | Doc-Kommentar-Syntax: `///` für Items, `//!` für Modul, `//` für Implementierung, Markdown | `#how-to-format-documentation-comments` | Verweis | Verweis auf offiziellen Style Guide (Ticket 02). Falsch platzierte Doc-Kommentare fängt rustc `unused_doc_comments` (warn); `///!`-Varianten Clippy `suspicious_doc_comments` (suspicious, default). |
| G-63 | Doku direkt per `///`/`//!` statt per Makro erzeugen | `#how-to-format-documentation-comments` | reject | `#[doc = ..]` ist legitim (`include_str!` für README); kein sauberes Verstoss-Signal. |

### Sonstige Sprachfeatures

| ID | Regel (paraphrasiert) | Anker | Entscheid | Begründung |
|---|---|---|---|---|
| G-64 | `unsafe` nur, wenn es keine sichere Alternative gibt; kapseln | `#unsafe-code` | reject | rustc-Lint `unsafe_code` (allow by default, `#![forbid(unsafe_code)]`). Kapselung ist semantisch. |
| G-65 | Jeder `unsafe`-Block bekommt einen Kommentar, der mit `SAFETY:` beginnt | `#unsafe-code` | adopt | Syntaktisch: `unsafe`-Block/-Impl ohne vorangehenden `// SAFETY:`-Kommentar. Clippy `undocumented_unsafe_blocks` ist restriction. Achtung Spike in Ticket 04: bei `let x = unsafe {..}` steht der Kommentar über dem `let`, nicht über dem Block. norma `src/` enthält kein `unsafe` (0 Treffer). |
| G-66 | Technische Rechtfertigung für `unsafe` (Warum nicht sicher? Benchmarks?) | `#unsafe-code` | reject | Semantisch. |
| G-67 | Keine unstabilen Features | `#no-unstable-features` | adopt | Syntaktisch: `#![feature(..)]`. rustc-Lint `unstable_features` ist allow by default; auf Stable bricht der Compiler ohnehin ab, die Regel wirkt nur in Nightly-Projekten. Niedriger Wert, aber billig und präzise. |
| G-68 | Getter-Namen: kein `get_`-Präfix (implizit über die Beispiele `foo()`/`foo_mut()`) | `#getters-and-setters` | Verweis | Verweis auf API Guidelines C-GETTER (Ticket 03). Der Guide verbietet `get_` nicht ausdrücklich. |
| G-69 | Accessor-Namen `foo_mut`, `take_foo`, `set_foo`, `with_foo`, `replace_foo` mit definierten Semantiken | `#getters-and-setters`, `#more-on-accessors-that-accept-closures` | reject | Positive Konvention; Abweichung hat kein Syntaxsignal. |
| G-70 | `len()` immer mit `is_empty()` | `#implement-expected-api-sets-together` | reject | Clippy `len_without_is_empty` (style, default); der Guide nennt den Lint. |
| G-71 | `#[doc(hidden)]`-Items fremder Crates nicht verwenden | `#do-not-access-items-marked-dochidden-that-you-dont-own` | reject | Braucht crate-übergreifende Auflösung. |
| G-72 | Öffentliche `const fn` nur bei stabiler Zusage | `#use-const-functions-responsibly` | reject | Zukunftsurteil. Clippy `missing_const_for_fn` (nursery) zielt in die Gegenrichtung. |
| G-73 | Operator-Überladung nur für mathematische/logische Zwecke | `#avoid-overloading-for-non-mathematical-purposes` | reject | Zweck der Impl ist nicht syntaktisch. Clippy `suspicious_arithmetic_impl` (suspicious, default) deckt nur verdächtige Operatoren im Körper. |
| G-74 | Öffentliche Enums der API mit `#[non_exhaustive]` | `#apply-non_exhaustive-to-publicly-exposed-enums` | adopt | Syntaktisch: `pub enum` ohne Attribut. Clippy `exhaustive_enums` ist restriction. Ausnahmen (stabile Ontologien, Zwei-Zustands-Enums, LSC-Zusage) und "von anderen Teams genutzt" sind Urteil, daher Hinweis-Severity; in Binary-Crates wertlos. |
| G-75 | Öffentliche Structs mit `pub`-Feldern mit `#[non_exhaustive]` | `#apply-non_exhaustive-to-public-structs-with-public-fields` | adopt | Syntaktisch: `pub struct` mit mindestens einem `pub`-Feld ohne Attribut. Clippy `exhaustive_structs` ist restriction. Hinweis-Severity; das Attribut verbietet Struct-Literale ausserhalb der Crate, was dem Funktionalen-Record-Update-Ausnahmefall des Guides widerspricht. |
| G-76 | `#[must_use]` auf Funktionen, deren Rückgabewert nicht ignoriert werden darf | `#must_use-on-functions` | reject | "Nicht korrekt nutzbar ohne Rückgabewert" ist semantisch. Clippy `must_use_candidate` (pedantic), `double_must_use` und `must_use_unit` (style, default) decken Randfälle. |
| G-77 | `#[must_use]` auf Typen (Fehlertypen, lazy Berechnungen) | `#must_use-on-types` | reject | Semantisch. |
| G-78 | Kein Code vor `main()` (z. B. `ctor`-Crate) | `#do-not-run-code-before-main` | adopt | Syntaktisch: `#[ctor]`, `#[ctor::ctor]`, `#[dtor]` sowie `#[link_section = ".init_array"]`-Varianten. Kein Clippy-Lint dafür gefunden. Selten, aber eindeutig. |

### Implementierung und Tests

| ID | Regel (paraphrasiert) | Anker | Entscheid | Begründung |
|---|---|---|---|---|
| G-79 | Typinferenz nicht überstrapazieren; Typen bei `.into()`/`.parse()` und komplexen Iterator-Ketten ergänzen | `#dont-overuse-type-inference` | adopt | Eng gefasst syntaktisch: `let`-Bindung ohne Typannotation, deren Initialisierer `.into()` oder `.parse()` ohne Turbofish enthält. Der Teil "Ergebnis komplexer Ketten" ist nicht erkennbar. Kein Clippy-Lint. Hinweis-Severity. |
| G-80 | Pattern Matching statt check-then-unwrap | `#prefer-pattern-matching-over-check-then-unwrap` | reject | Clippy `unnecessary_unwrap` (complexity, default). |
| G-81 | `expect` statt `unwrap`, wenn ein Panic nötig ist | `#prefer-expect-over-unwrap` | adopt | Syntaktisch: `.unwrap()` ausserhalb von `#[cfg(test)]`/`#[test]` (Tests dürfen paniken, laut Abschnitt "Reporting errors"). Clippy `unwrap_used` ist restriction. In norma `src/` 197 `.unwrap()`-Treffer insgesamt (mehrheitlich vermutlich Tests, ungeprüft). |
| G-82 | Bei `#[non_exhaustive]`-Enums alle bekannten Varianten auflisten, Rest im Wildcard-Arm | `#dont-omit-patterns-on-non_exhaustive-enums-when-matching` | reject | Ob ein Enum `non_exhaustive` ist, braucht Typinfo (oft fremde Crate). Siehe Auffälligkeiten zum Beispiel `Triangle \| _`. |
| G-83 | Kontext-Strings in `assert!` | `#use-context-strings-to-clarify-assertions` | defer | Formulierung ist "stets erwägen", nicht "müssen". Bare `assert!(x)` ist erkennbar, aber in Tests massenhaft. Entscheidung nach Trefferzahl in `src/`. |
| G-84 | `Iterator::for_each` vermeiden, ausser mit gutem Grund | `#dont-use-iterator-combinator-chains-when-direct-iteration-is-clearer` | adopt | Syntaktisch: Aufruf `.for_each(..)`. Clippy `needless_for_each` ist pedantic. Hinweis-Severity (z. B. Rayon-`par_iter().for_each` ist legitim). |
| G-85 | Keine Combinator-Kette im Kopf einer `for`-Schleife; Kette zuerst an Variable binden | `#dont-use-iterator-combinator-chains-when-direct-iteration-is-clearer` | adopt | Syntaktisch: `for`-Iterable ist eine Methodenkette mit Closure-Argument. Reine Adapter wie `.iter().enumerate()` ausnehmen. Kein Clippy-Default-Lint. |
| G-86 | Lange/komplexe Combinator-Ketten vermeiden | `#dont-use-iterator-combinator-chains-when-direct-iteration-is-clearer` | reject | "Lang und komplex" ist subjektiv; Schwellenwert wäre willkürlich. |
| G-87 | Kein `panic::catch_unwind` in Eigencode | `#dont-call-paniccatch_unwind` | adopt | Syntaktisch: Aufruf von `catch_unwind` (qualifiziert oder importiert). Kein Clippy-Lint dafür (nur über konfigurierbares `disallowed_methods`, standardmässig leer). Ausnahme "Defense in Depth bei `panic=unwind`" bleibt Urteil, daher Hinweis-Severity. |
| G-88 | `format!`/`write!` statt `+` bei mehr als zwei Strings und statt `str::replace`-Templating | `#prefer-format--write-for-concatenation-and-substitution` | defer | Ein Operand als String-Literal ist erkennbar (`&str + &str` kompiliert nie), reine Variablenketten nicht. Teilabdeckung. Clippy `string_add` (restriction) und `string_add_assign` (pedantic) sind typbewusst. |
| G-89 | Test-Framework der Codebase verwenden | `#use-the-right-per-codebase-test-framework` | reject | Chromium-/Fuchsia-spezifisch. |

## Auffälligkeiten

- **Das gute Beispiel verletzt einen Default-Clippy-Lint.** Im Abschnitt `#dont-omit-patterns-on-non_exhaustive-enums-when-matching` ist `Shape::Triangle | _ => {}` als gutes Beispiel markiert (CSS-Klasse `goodcode`). Clippy `wildcard_in_or_patterns` (complexity, default) schlägt genau dieses Muster an. Der Guide kollidiert hier mit den eigenen Lint-Defaults.
- **Interne Uneinheitlichkeit bei Konstruktoren.** `#naming-initialization-functions` nennt `new_with_foo()`, `#pair-important-from-impls-with-explicit-constructor-methods` aber `::new_from_foo()`. Deshalb wird kein Präfix-Verbot aufgenommen (G-53).
- **Gegenläufige Lints.** G-40 (Partial vor Total) läuft gegen Clippy `derive_partial_eq_without_eq` (nursery); G-72 (`const` zurückhaltend) gegen `missing_const_for_fn` (nursery). Wer beide Packs aktiviert, bekommt widersprüchliche Hinweise.
- **Viele Regeln sind codebase-gebunden.** Sieben Kandidaten sind Chromium-/Fuchsia-spezifisch (G-03, G-04, G-13, G-45, G-46, G-47, G-89); vier weitere hängen am Library-vs.-Application-Kontext (G-22, G-23, G-24, G-33). Der Guide ist stellenweise ein Chromium-Dokument mit allgemeinem Mantel.
- **Der Guide benennt Clippy-Lints selbst** (`borrowed_box`, `ptr_arg`, `should_implement_trait`, `new_without_default`, `wrong_self_convention`, `len_without_is_empty`, `exhaustive_enums`). Davon sind sechs Default-Lints und ein restriction-Lint (`exhaustive_enums`, daher G-74 adopt). Die eigenen Zuordnungen wurden gegen `clippy-driver -W help` geprüft; die Level stimmen.
- **Ein Teil der Regeln ist nur Nightly-rustfmt.** `imports_granularity` (G-09) ist in der Stable-Ausgabe von `rustfmt --help=config` nicht gelistet; `hex_literal_case` (G-16) schon. Darum die unterschiedliche Behandlung.
- **Die HTML-Seite nennt keine Lizenz.** Nur das Repo-README tut es (siehe Lizenz-Befund).
- **Die Web-Lint-Liste war nicht automatisch lesbar.** Deshalb Clippy-Nachweis über die lokale Clippy-Version 0.1.99. Gruppen können in anderen Clippy-Versionen abweichen (Lints wandern zwischen Gruppen).

## Hinweise für Ticket 04 (keine YAML)

- Alle adopt-Regeln mit "Ausnahme"-Vermerk sind als Hinweis-Severity gedacht.
- Spike-Punkte: G-65 (Kommentar über dem umgebenden Statement), G-44 (Variantenzahl der Poison-Idiome), G-85 (Abgrenzung "reine Adapter").
- G-09 und G-21 sind Grenzfälle zur Linter-Grenze und brauchen eine Bestätigung.
- Plausibilitätscheck gegen `src/` betrifft vor allem G-18, G-81, G-83.
- Nutzer-Umgebung: bei einer Sprache pro Pattern (ADR 0002) entstehen alle diese Regeln als `language: rust`.
