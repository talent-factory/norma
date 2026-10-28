# Research 03: Rust API Guidelines

Ticket: `issues/03-rust-api-guidelines.md` (Map: `map.md`, Linter-Grenze unter "Notes").
Stand: 2026-10-28. Keine YAML (das macht Ticket 04).

## Quellen und Stand

| Quelle | Zweck | Stand |
| --- | --- | --- |
| <https://rust-lang.github.io/api-guidelines/> (about, checklist, 11 Kapitelseiten) | Primärquelle, alle 54 Guidelines gelesen | Repo `rust-lang/api-guidelines`, HEAD `97a0969` |
| <https://github.com/rust-lang/api-guidelines> (README, `LICENSE-MIT`, `LICENSE-APACHE`) | Lizenz | HEAD `97a0969` |
| Offizielle Clippy-Lintliste <https://clippy.rust-lang.org/master/index.html> (Gruppen) und `clippy-driver -W help` 0.1.99 (Default-Level) | Lint-Namen, Gruppe, Default-Level | Clippy 0.1.99 / rustc 1.99.1 |
| `rustc -W help`, `rustdoc -W help` | rustc-/rustdoc-Lints | rustc 1.99.1 |
| Experimente (Scratch-Crate ausserhalb des Repos, `cargo clippy`, `ast-grep scan`) | Verhalten belegen statt vermuten | siehe "Experimente" |

Die Seite `about.html` sagt selbst: "These are only guidelines, some more firm than
others ... should not in any way be considered a mandate". Das spricht für
niedrige Severity (hint/info) in jedem daraus entstehenden Pack.

## Lizenz-Befund

- Die Guidelines liegen im Repo `rust-lang/api-guidelines` (mdBook-Quelltexte unter `src/`).
  Das README sagt: "licensed under either of Apache License, Version 2.0 or MIT license, at your option"
  (`LICENSE-APACHE`, `LICENSE-MIT`, Copyright "2017 The Rust Project Developers"). Beide Dateien existieren.
- Es gibt keine separate Inhaltslizenz (z. B. CC) für den Text; die Dual-Lizenz gilt für das ganze Projekt,
  inklusive der Guideline-Texte und Codebeispiele.
- Folge: Beide Lizenzen sind permissiv; Zitieren wäre mit Lizenzhinweis erlaubt. Die Map-Vorgabe
  (paraphrasieren, verlinken, keinen Regeltext kopieren) ist damit mehr als erfüllt und bleibt ausreichend.
  Die C-IDs und Titel sind kurze Bezeichner und können als Rule-IDs/Anker dienen. Ein Attributionshinweis
  ("nach den Rust API Guidelines, MIT OR Apache-2.0, Link") im Pack-README ist sauber, aber nicht erzwungen,
  solange nur paraphrasiert wird.
- Nicht geprüft: Marken-/Logo-Nutzung (Rust-Logo) ist hier irrelevant, da keine Assets übernommen werden.

## Methode und Kriterien

Linter-Grenze aus der Map: aufnehmen nur, was ast-grep rein syntaktisch zuverlässig erkennt **und**
was weder rustfmt noch Clippy (Default-Lints) schon prüfen. Ergänzend habe ich festgehalten:

- **Abdeckung** nennt rustc-/Clippy-Lints mit Gruppe und Default-Level. "Default" heisst `warn`/`deny`
  ohne Konfiguration; `allow` bedeutet pedantic/restriction/nursery/cargo, also opt-in.
- Entscheid **adopt**: syntaktisch zuverlässig, kein Default-Lint deckt es ab, Machbarkeit mit ast-grep
  im Experiment gezeigt. **defer**: syntaktisch teilweise möglich, aber Heuristik, Mehrdeutigkeit oder offene
  Grenzfrage (v. a. Überlappung mit opt-in Clippy-Lints). **reject**: mit Grund (rustc/Clippy-Default,
  braucht Typinfo/Semantik, ausserhalb von Rust-Quelltext, Prozess/Prosa).
- Mehrere Guidelines bestehen aus Teilforderungen mit unterschiedlichem Ergebnis; ich führe sie als
  Unterzeilen (`C-XXX / Teil`). Daraus ergeben sich 60 Zeilen für 54 Guidelines.

## Ergebnis in Zahlen

| Entscheid | Zeilen | davon Unterzeilen |
| --- | --- | --- |
| adopt | 11 | 4 (C-CASE/Akronym, C-CONV/_mut, C-CONV-TRAITS/TryInto, C-GOOD-ERR/description) |
| defer | 9 | 2 (C-CASE/Ein-Buchstaben-Wort, C-GOOD-ERR/Fehlertext) |
| reject | 40 | 0 |
| Summe | 60 | |

Nach Primärentscheid je Guideline (54 C-IDs, Unterzeilen nicht gezählt): adopt 7, defer 7, reject 40.
Dabei zählt C-CASE, C-CONV, C-CONV-TRAITS, C-GOOD-ERR und C-FEATURE nach ihrer jeweils ersten Zeile
(C-CASE, C-CONV, C-CONV-TRAITS, C-GOOD-ERR = reject, C-FEATURE = adopt).

## Entscheidungstabelle

Kürzel: R = rustc-Lint, C = Clippy-Lint (Gruppe, Default-Level in Klammern).

### Naming

| C-ID | Entscheid | Begründung / Abdeckung |
| --- | --- | --- |
| C-CASE | reject | R `non_camel_case_types`, `non_snake_case`, `non_upper_case_globals` (warn, verifiziert). Crate-Namen `-rs` stehen in Cargo.toml, nicht im Rust-AST. |
| C-CASE / Akronym | adopt | Akronyme zählen als ein Wort (`Uuid`, nicht `UUID`). Weder rustc noch C `upper_case_acronyms` (style, warn) melden `HTTPResponse`/`HttpURL`: im Experiment keine Warnung, der Lint greift per Default nur bei komplett grossgeschriebenen Namen. Schmale Regex auf Typ-/Trait-/Variantennamen (>= 3 Grossbuchstaben in Folge in gemischtem Namen) trifft `HTTPResponse`, `HttpURL`. Risiko siehe "Überraschungen" (std selbst: `BTreeMap`). |
| C-CASE / Ein-Buchstaben-Wort | defer | `b_tree_map` statt `btree_map`: rustc meldet es nicht (Experiment), aber Regex auf `_x_` hat False Positives (`x_axis`, `p_value`). |
| C-CONV | reject | Präfixe `as_`/`to_`/`into_` gegen Self-Art: C `wrong_self_convention` (style, warn). Achtung: bei exportierter API per Default aus (`avoid-breaking-exported-api = true`), im Experiment mit `false` greift er. Kosten-Semantik (billig/teuer) ist ohnehin nicht syntaktisch. |
| C-CONV / `_mut`-Suffix | adopt | `as_mut_slice` statt `as_slice_mut`: auch mit `avoid-breaking-exported-api = false` meldet `wrong_self_convention` `as_slice_mut` nicht (Experiment). Name-Regex `^(as\|to\|into)_.+_mut$` auf Funktionsnamen ist eindeutig. |
| C-GETTER | adopt | `get_`-Präfix bei Gettern. Kein Lint (nur C `get_first`, anderes Thema). Muster `pub fn get_x(&self) -> T`; Ausnahmen (`Cell::get`, "einzig offensichtliches Ding") rechtfertigen hint-Severity, auf inherent impls beschränken. |
| C-ITER | reject | Ob eine Methode einen Iterator liefert, braucht Typinfo; Paarung iter/into_iter deckt C `iter_without_into_iter`, `into_iter_without_iter`, `iter_not_returning_iterator` (alle pedantic, allow). |
| C-ITER-TY | defer | `iter()` sollte `Iter` liefern. Syntaktisch am Rückgabetyp möglich, aber `impl Iterator` (von C-NEWTYPE-HIDE selbst empfohlen), Aliase und Boxed-Iteratoren erzeugen False Positives. |
| C-FEATURE | adopt | Feature-Namen ohne `use-`/`with-`/`no-`. Die Cargo.toml-Seite ist für ast-grep unerreichbar (und C `redundant_feature_names`/`negative_feature_names` sind cargo-Gruppe, allow). Adoptierbar ist nur die Quelltext-Seite: String in `#[cfg(feature = "use-std")]`. Im Experiment getroffen. |
| C-WORD-ORDER | reject | Konsistenz crate-weit und gegen std (`ParseAddrError` vs. `AddrParseError`), nicht lokal prüfbar. |

### Interoperability

| C-ID | Entscheid | Begründung / Abdeckung |
| --- | --- | --- |
| C-COMMON-TRAITS | reject | "wo anwendbar" braucht Semantik. Echte Linter: R `missing_debug_implementations`, `missing_copy_implementations` (beide allow); C `new_without_default` (style, warn), `derive_partial_eq_without_eq` (nursery). Ein reiner `derive`-Check hätte False Positives bei handgeschriebenen impls. |
| C-CONV-TRAITS | reject | `Into` statt `From` implementieren: C `from_over_into` (style, warn). |
| C-CONV-TRAITS / TryInto | adopt | `impl TryInto<..> for X` statt `TryFrom`: kein Lint (Experiment: keine Warnung), Muster ist eindeutig. |
| C-COLLECT | reject | "Collections" erkennen braucht Semantik. |
| C-SERDE | reject | Fehlen von `Serialize` braucht Semantik ("ist Datenstruktur"); das `cfg_attr`-Gating betrifft nur Libraries mit Cargo-Feature. |
| C-SEND-SYNC | reject | Auto-Traits sind Typsystem. C `arc_with_non_send_sync` (suspicious, warn), `non_send_fields_in_send_ty` (nursery), `future_not_send` (allow). |
| C-GOOD-ERR | reject | `Result<_, ()>` in pub-API: C `result_unit_err` (style, warn, im Experiment gemeldet). `Error + Send + Sync + 'static` braucht Typinfo. |
| C-GOOD-ERR / description | adopt | `Error::description()` nicht implementieren (veraltet). Weder rustc noch Clippy melden die Implementierung (Experiment). Muster `impl .. Error for X { fn description }` im Experiment getroffen; selten, aber eindeutig. |
| C-GOOD-ERR / Fehlertext | defer | Text klein, ohne Schlusspunkt. Prüfbar nur an String-Literalen in `#[error("..")]` (thiserror, Drittcrate) oder `write!` in `Display`; Gross am Anfang hat False Positives (Eigennamen). Machbarkeit auf Makro-Token-Ebene ist in 04 zu prüfen. |
| C-NUM-FMT | reject | Braucht Typ-/Semantik ("Binary number types"). |
| C-RW-VALUE | defer | `R: Read` per Wert statt `&mut R`: Form `fn f<R: Read>(r: &mut R)` ist syntaktisch fassbar (auch `where`, `&mut impl Read`), aber viele Formen und `&mut dyn Read` ist legitim. Kein Lint. |

### Macros

| C-ID | Entscheid | Begründung |
| --- | --- | --- |
| C-EVOCATIVE | reject | Subjektive Syntaxgestaltung. |
| C-MACRO-ATTR | reject | tree-sitter-rust parst `macro_rules!`-Bodies als flache `token_tree` (im Experiment: `pub`, `struct`, `identifier` als lose Tokens). Nur Regex auf Token möglich, kein AST. |
| C-ANYWHERE | reject | Verhalten nach Expansion (Test-Frage), nicht Syntax. |
| C-MACRO-VIS | reject | Wie C-MACRO-ATTR: `$vis:vis` im Matcher gegen Transcriber ist Token-Ebene. |
| C-MACRO-TY | reject | Verhalten nach Expansion. |

### Documentation

| C-ID | Entscheid | Begründung / Abdeckung |
| --- | --- | --- |
| C-CRATE-DOC | reject | Rustdoc `missing_crate_level_docs` (allow); Qualität ("thorough") ist nicht prüfbar. |
| C-EXAMPLE | reject | Rustdoc `missing_doc_code_examples` (allow), R `missing_docs` (allow); "within reason" ist Urteil. |
| C-QUESTION-MARK | reject | Doc-Beispiele sind für ast-grep nur `doc_comment`-Text (Experiment: `line_comment` mit `doc_comment`-Kind), nicht Code. `unwrap` in normalem Code: C `unwrap_used` (restriction). |
| C-FAILURE | defer | `# Safety` bei `unsafe fn`: C `missing_safety_doc` (style, warn). `# Errors`/`# Panics`: C `missing_errors_doc`, `missing_panics_doc` (pedantic, allow). Für `# Errors` wäre `pub fn .. -> Result` ohne Doc-Abschnitt syntaktisch möglich (Doc-Kommentare sind Geschwisterknoten), aber Typ-Aliase und Panic-Erreichbarkeit nicht. Grenzfrage Opt-in-Clippy, siehe unten. |
| C-LINK | reject | Prosa; Rustdoc `broken_intra_doc_links`, `bare_urls` (warn); C `doc_markdown` (pedantic). |
| C-METADATA | reject | Cargo.toml; C `cargo_common_metadata` (cargo, allow). |
| C-RELNOTES | reject | Prozess. |
| C-HIDDEN | reject | Semantik ("unhelpful"), kein syntaktisches Kriterium. |

### Predictability

| C-ID | Entscheid | Begründung / Abdeckung |
| --- | --- | --- |
| C-SMART-PTR | reject | Braucht Wissen, dass der Typ `Deref` implementiert (Cross-Item). |
| C-CONV-SPECIFIC | reject | Welcher Typ "spezifischer" ist, ist Semantik. |
| C-METHOD | reject | "klarer Empfänger" ist Urteil; `fn f(x: &Foo)` allein wäre extrem laut. |
| C-NO-OUT | defer | Out-Parameter nur über Namensheuristik (`out`, `output`) erkennbar; `&mut`-Parameter sind meist legitim (Puffer, Writer). Kein Lint. |
| C-OVERLOAD | reject | Semantik. |
| C-DEREF | adopt | `impl Deref/DerefMut for X`: Konstrukt eindeutig erkennbar, kein Lint (nur `deref_by_slicing`, `explicit_deref_methods`, anderes Thema). Urteil "Smart Pointer?" bleibt beim Menschen, daher Meldung als Prüfaufforderung, hint. |
| C-CTOR | reject | C `new_ret_no_self`, `should_implement_trait`, `self_named_constructors`, `new_without_default` (alle style, warn). |

### Flexibility

| C-ID | Entscheid | Begründung / Abdeckung |
| --- | --- | --- |
| C-INTERMEDIATE | reject | Designfrage. |
| C-CALLER-CONTROL | defer | Borrow-dann-clone (`fn f(b: &Bar) { let b = b.clone(); .. }`) ist syntaktisch fassbar, aber `Copy`-Typen und erzwungene Trait-Signaturen erzeugen False Positives. Umgekehrte Richtung (Wert nehmen, nur borrowen): C `needless_pass_by_value` (pedantic, allow) braucht Typinfo. |
| C-GENERIC | reject | `&Vec<T>`/`&String`: C `ptr_arg` (style, warn). `IntoIterator`-Empfehlung ist Semantik; Gegenteil C `impl_trait_in_params` (restriction). |
| C-OBJECT | reject | Object Safety ist Typsystem (rustc E0038 an der Nutzungsstelle). |

### Type safety

| C-ID | Entscheid | Begründung / Abdeckung |
| --- | --- | --- |
| C-NEWTYPE | reject | Semantik (`f64` als Meilen?). |
| C-CUSTOM-TYPE | adopt | pub fn mit >= 2 `bool`-Parametern (das Beispiel der Guideline ist `Widget::new(true, false)`). C `fn_params_excessive_bools` (pedantic, allow, Schwelle 3, konfigurierbar) deckt es nur opt-in ab. `Option`-Parameter nicht aufnehmen (zu laut). Hinweis zur Form: Muster mit mehreren `$$$` und zwei `bool` matchte im Experiment nicht, ein festes Arity-Muster schon; Formulierung in 04 über `kind` prüfen. Grenzfall wegen Clippy-Opt-in, siehe unten. |
| C-BITFLAG | reject | Flags vs. Codes erkennen braucht Semantik und Zahlenauswertung. |
| C-BUILDER | reject | Designfrage; C `return_self_not_must_use`, `must_use_candidate` (pedantic) sind Nachbarthemen. |

### Dependability

| C-ID | Entscheid | Begründung / Abdeckung |
| --- | --- | --- |
| C-VALIDATE | reject | Semantik. |
| C-DTOR-FAIL | adopt | `unwrap()`, `expect(..)`, `panic!` in `impl Drop`. Kein Lint (Clippy meldet nur `unwrap` auf bekanntes `None`, Experiment). Im Experiment getroffen. Weitere Panic-Makros (`assert!`, `unreachable!`, `todo!`) in 04 ergänzen. |
| C-DTOR-BLOCK | defer | Eindeutige Bezeichner (`thread::sleep`, `block_on`) in `impl Drop` wären sicher, `join`/`lock`/`recv` sind mehrdeutig. Gleiches Gerüst wie C-DTOR-FAIL, in 04 mit diesem zusammenlegen oder weglassen. |

### Debuggability

| C-ID | Entscheid | Begründung / Abdeckung |
| --- | --- | --- |
| C-DEBUG | reject | R `missing_debug_implementations` (allow) ist typbewusst, ein `derive`-Check hat Fehlalarme bei manuellem `impl Debug`. |
| C-DEBUG-NONEMPTY | reject | Semantik; C `missing_fields_in_debug` (pedantic) ist ein anderes Problem. |

### Future proofing

| C-ID | Entscheid | Begründung / Abdeckung |
| --- | --- | --- |
| C-SEALED | reject | Ob ein Trait versiegelt sein soll, ist Designfrage. |
| C-STRUCT-PRIVATE | adopt | `pub struct` mit `pub`-Feldern. Kein Default-Lint (C `partial_pub_fields` restriction, `pub_underscore_fields` pedantic). Guideline nennt Ausnahme "passive Datenstrukturen". Nur für Library-Crates sinnvoll; hint-Severity. Auf norma `src/` im Experiment 57 Treffer (Rauschen für Ticket 05 einplanen). |
| C-NEWTYPE-HIDE | defer | Rückgabetyp aus std-Iterator-Adaptern (`Enumerate<Skip<I>>`) in pub-Signatur wäre per Namensliste erkennbar, aber ohne Import-Auflösung False Positives, und `impl Trait` ist erlaubte Alternative. |
| C-STRUCT-BOUNDS | adopt | Bounds aus der Verbotsliste (`Clone`, `PartialEq`, `PartialOrd`, `Debug`, `Display`, `Default`, `Error`, `Serialize`, `Deserialize`, `DeserializeOwned`) auf Struct-/Enum-Generics, auch in `where`. Kein Lint (Experiment). Ausnahmen der Guideline (assoziierter Typ, `?Sized`, `Drop`-impl) sind selten; `?Sized` fällt mit der Verbotsliste nicht zusammen. Im Experiment getroffen. |

### Necessities

| C-ID | Entscheid | Begründung |
| --- | --- | --- |
| C-STABLE | reject | Cargo-/SemVer-Frage. |
| C-PERMISSIVE | reject | Lizenzpolitik (Cargo.toml, `cargo deny`), kein Rust-AST. |

## Experimente

Scratch-Crate unter `/tmp/ag-exp` (nicht im Repo), Clippy 0.1.99, ast-grep (`ast-grep scan`).
Belegt wurde:

- Clippy/rustc melden keine `HTTPResponse`/`HttpURL`/`USize`, kein `b_tree_map`, kein `get_a`, kein
  `as_slice_mut`, kein `impl Error { fn description }`, kein `impl TryInto`, kein `impl Deref` auf Newtype,
  keine Struct-Bounds `T: Clone + Debug + PartialEq`, kein `unwrap()` in `Drop` (ausser bei bekanntem `None`),
  kein `use-std` im `cfg`.
- Gemeldet wurden: `Result<_, ()>` (`result_unit_err`), `&Vec<u8>` (`ptr_arg`). Mit
  `avoid-breaking-exported-api = false`: `to_thing(self)` und `as_other(self)` (`wrong_self_convention`).
- ast-grep trifft alle elf adopt-Kandidaten in der Form, die ich getestet habe (C-CUSTOM-TYPE nur mit
  festem Arity-Muster). Eine Suche in norma `src/` ergab ausser `pub-fields` (57) keine Treffer für die
  geprüften Regeln.
- Macro-Bodies sind flache `token_tree`, Doc-Kommentare sind `line_comment`-Knoten mit `doc_comment`.

## Überraschungen und offene Punkte für Ticket 04

1. **Grenzfrage Opt-in-Clippy.** Die Map-Grenze nennt "Clippy (Default-Lints)". Viele passende Lints sind
   pedantic/restriction/nursery/cargo (`missing_errors_doc`, `fn_params_excessive_bools`, `iter_without_into_iter`,
   `cargo_common_metadata`). Streng gelesen blieben deren Themen übrig und adoptierbar. Ich habe sie
   case-by-case entschieden (C-CUSTOM-TYPE adopt, C-FAILURE defer, C-ITER/C-METADATA reject). Ticket 04 sollte
   die Linie einheitlich festlegen; es betrifft C-CUSTOM-TYPE und C-FAILURE.
2. **`wrong_self_convention` greift bei exportierter API per Default nicht.** Das Gegenteil von dem, was
   man für ein Library-Pack erwartet, und der Grund, warum C-CONV nur "reject" mit Fussnote ist.
3. **Einige Guidelines haben gar keinen Linter**, obwohl sie präzise sind: `TryInto`-impl, `description()`,
   Panic in `Drop`, Struct-Bounds, `Deref`-Missbrauch. Das sind die sauberen Pack-Kandidaten.
4. **rustc statt Clippy** deckt Guidelines ab, die man bei Clippy sucht (`missing_debug_implementations`,
   `missing_docs`, `non_*_case`). Ein Pack-README sollte darauf verweisen statt Regeln zu duplizieren.
5. **std verletzt die Akronym-Regel selbst** (`BTreeMap`, `CString`), obwohl die Guideline sagt "the whole
   standard library. This guideline should be easy". Die Regex ist deshalb absichtlich eng (>= 3 Grossbuchstaben).
6. **Pattern-Falle ast-grep:** `fn $N(&self)` matcht `pub fn` nicht; die Sichtbarkeit muss im Muster stehen
   oder per `kind` umgangen werden. Ebenso matchte ein Muster mit zwei `$$$`-Lücken um zwei `bool`-Parameter nicht.
7. **Zielgruppe:** norma ist eine Anwendung, die API-Guidelines richten sich an Library-Crates. Ein
   Teil der adopt-Kandidaten (C-STRUCT-PRIVATE, C-DEREF, C-STRUCT-BOUNDS, C-GETTER) ist für Apps
   weniger relevant; das Pack sollte als "für Library-Crates" beschrieben werden.
8. **Ticket-Dateien liegen nicht im Worktree.** `map.md` und `issues/*` fehlen im Git-Stand des
   Worktrees (vermutlich im Haupt-Checkout noch nicht committet); ich habe sie dort nur gelesen.
