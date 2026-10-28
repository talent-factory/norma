Type: task
Status: resolved
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

## Answer

Alle drei Guides behalten nach der Linter-Grenze ein Pack. Es entstehen 40
Regeln in drei Verzeichnissen (je eine `rust.yaml`, Kategorie = Verzeichnisname).
Es sind keine Dateien in `src/`, `default_patterns.rs` oder README geändert.

### Linie für opt-in-Clippy-Gruppen

Nur Default-an-Lints (Clippy `style`/`complexity`/`suspicious`/`perf`/`correctness`,
rustc warn/deny) zählen als Abdeckung. `pedantic`/`restriction`/`nursery`/`cargo`
zählen nicht. Die Grenzfälle G-09 (`imports_granularity` nur Nightly-rustfmt) und
G-21 (`ref_option` ist pedantic) bleiben adopt, ebenso C-CUSTOM-TYPE
(`fn_params_excessive_bools` ist pedantic). C-FAILURE (`missing_errors_doc`
pedantic) bleibt defer, weil es Typ-Aliase und Panic-Erreichbarkeit braucht, nicht
wegen der Gruppe. Die Google-Regel zu `Shape::Triangle | _` widerspricht dem
Default-Lint `wildcard_in_or_patterns` (G-82) und wird nicht gebaut (war schon reject).

### Google Rust Style Guide: `rule-packs/google-rust-style/rust.yaml`, 20 Regeln

Quellenhinweis CC BY 3.0 samt "paraphrased/adapted by norma" steht im Pack-Kopf.
Alle 20 adopt-Kandidaten wurden gebaut, keiner kippte.

| Kandidat | Rule-ID | Severity |
|---|---|---|
| G-07 | `google-no-glob-import-rust` | info |
| G-08 | `google-no-custom-prelude-rust` | warning |
| G-09 | `google-use-one-module-per-use-rust` | info |
| G-17 | `google-rustfmt-skip-needs-reason-rust` | hint |
| G-21 | `google-ref-option-param-rust` | warning |
| G-24 | `google-impl-into-param-rust` | hint |
| G-28 | `google-tuple-struct-multiple-fields-rust` | warning |
| G-44 | `google-ignore-mutex-poison-rust` | warning |
| G-50 | `google-lifetime-name-descriptive-rust` | hint |
| G-52 | `google-new-returning-result-rust` | warning |
| G-65 | `google-unsafe-needs-safety-comment-rust` | warning |
| G-67 | `google-no-unstable-features-rust` | warning |
| G-74 | `google-non-exhaustive-pub-enum-rust` | hint |
| G-75 | `google-non-exhaustive-pub-struct-rust` | hint |
| G-78 | `google-no-code-before-main-rust` | warning |
| G-79 | `google-annotate-inferred-type-rust` | hint |
| G-81 | `google-prefer-expect-over-unwrap-rust` | warning |
| G-84 | `google-avoid-for-each-rust` | hint |
| G-85 | `google-no-combinator-chain-in-for-head-rust` | hint |
| G-87 | `google-no-catch-unwind-rust` | hint |

Defer (nicht gebaut, unverändert aus Ticket 01): G-06, G-18, G-29, G-33, G-83, G-88.
Reject: die 57 Zeilen aus Ticket 01. Verweise: G-25 -> `api-c-custom-type-bool-params-rust`,
G-27 -> API C-NEWTYPE (reject, Semantik), G-49 -> Clippy `wrong_self_convention` bzw.
`api-c-conv-mut-suffix-rust`, G-62 -> `style-doc-comment-*-rust`, G-68 ->
`api-c-getter-rust`, G-01 -> die beiden anderen Packs.

Grenzen, die in den `note`-Feldern stehen:
- G-65 (SAFETY-Kommentar) war als Spike-Kandidat riskant und liess sich bauen
  (Kommentar über dem Block, über dem umgebenden `let`/Statement, mehrzeilig).
  Bekannte Lücken: Ein Trailing-Kommentar `foo(); // SAFETY: ..` auf der Zeile des
  vorigen Statements wird als Kommentar akzeptiert (False Negative); ein
  `// SAFETY:` hinter dem Block auf derselben Zeile wird gemeldet; `unsafe` in
  Macro-Bodies ist unsichtbar.
- G-81 überspringt `#[test]`-artige Funktionen und `#[cfg(test)]`-Module, nicht aber
  `tests/`-Dateien (Pfad ist aus dem Inhalt nicht erkennbar, Hinweis auf
  pre-commit `exclude:`).
- G-28 meldet auch `(T, PhantomData<T>)`.
- G-07 nimmt `use super::*` und jedes `pub use` aus; `use Enum::*` ist nicht
  unterscheidbar.

### Offizieller Rust Style Guide: `rule-packs/rust-style-guide/rust.yaml`, 10 Regeln

Lizenzhinweis MIT OR Apache-2.0 samt Link steht im Kopf, ebenso der Hinweis, dass
der Guide Abweichungen nicht verbietet und das Pack strenger ist. Alle 10
adopt-Kandidaten (B1-B10) wurden gebaut, auch B3 und B9.

| Kandidat | Rule-ID | Severity |
|---|---|---|
| B1 | `style-macro-name-snake-case-rust` | warning |
| B9 | `style-macro-rules-curly-braces-rust` | warning |
| B2 | `style-unit-struct-no-empty-brackets-rust` | hint |
| B3 | `style-doc-comment-before-attributes-rust` | warning |
| B4 | `style-doc-comment-line-style-rust` | warning |
| B10 | `style-comment-space-after-slashes-rust` | hint |
| B5 | `style-no-path-attribute-rust` | hint |
| B6 | `style-extern-crate-first-rust` | warning |
| B7 | `style-use-before-other-items-rust` | info |
| B8 | `style-imports-before-mod-declarations-rust` | info |

Entscheidungen zu den offenen Fragen aus Ticket 02:
- B9: Die Klammern sind anonyme Tokens, und `kind: '('` bzw. `'['` sind als
  Selektor ungültig. Die Regel prüft deshalb den Definitionstext
  (`^macro_rules!\s+NAME\s*[(\[]`). Das ist zuverlässig, weil die Syntax starr ist.
- B3: Funktioniert über `follows: attribute_item` auf dem äusseren Doc-Kommentar,
  meldet nur die erste Doc-Zeile hinter dem Attribut. Ein Trailing-`//` auf der
  Attributzeile versteckt den Fall.
- B7: `macro_rules!` und Macro-Aufrufe zählen nicht als "andere Items" (häufig
  legitim vorn). `use` in Funktionskörpern ist ausgenommen.
- B8: `#[macro_use] mod` ist ausgenommen, wie vom Guide verlangt. Die Regel trifft
  das gängige "erst `mod`, dann `use`"-Muster in lib.rs/main.rs, daher info.
- B2: `PhantomData`-Structs haben Felder und werden nicht gemeldet, die Frage
  erübrigt sich.

Defer (unverändert, Ticket 02): D1-D5. Reject: Tabelle F und R. Verweise: glob/Import-
Granularität -> Google-Pack, Import-Sortierung -> rustfmt.

### Rust API Guidelines: `rule-packs/rust-api-guidelines/rust.yaml`, 10 Regeln

Lizenzhinweis MIT OR Apache-2.0 samt Link und "richtet sich an Library-Crates" im
Kopf. Von den 11 adopt-Zeilen aus Ticket 03 sind 10 gebaut. Eine ist ein Overlap.

| Kandidat | Rule-ID | Severity |
|---|---|---|
| C-CASE / Akronym | `api-c-case-acronym-rust` | hint |
| C-CONV / `_mut` | `api-c-conv-mut-suffix-rust` | info |
| C-GETTER | `api-c-getter-rust` | hint |
| C-CONV-TRAITS / TryInto | `api-c-conv-traits-tryinto-rust` | warning |
| C-GOOD-ERR / description | `api-c-good-err-description-rust` | warning |
| C-FEATURE (Quelltext-Seite) | `api-c-feature-placeholder-words-rust` | hint |
| C-DEREF | `api-c-deref-review-rust` | hint |
| C-CUSTOM-TYPE | `api-c-custom-type-bool-params-rust` | info |
| C-DTOR-FAIL | `api-c-dtor-fail-rust` | warning |
| C-STRUCT-BOUNDS | `api-c-struct-bounds-rust` | info |

Overlap: C-STRUCT-PRIVATE (`pub struct` mit `pub`-Feldern) und G-75 treffen
dieselbe Konstruktion. Gebaut ist nur `google-non-exhaustive-pub-struct-rust`
(konkretere, prüfbare Abhilfe). Der Gegenzug: Ein `#[non_exhaustive]`-Struct mit
`pub`-Feldern erfüllt G-75, aber nicht die Forderung nach privaten Feldern aus
C-STRUCT-PRIVATE. Das steht im `note` von G-75 und im Pack-Kopf.
C-CUSTOM-TYPE ist die einzige Quelle für G-25, C-GETTER für G-68.

C-DTOR-FAIL deckt zusätzlich `assert*!`, `unreachable!`, `todo!`, `unimplemented!`
und `debug_assert*!` ab. C-DTOR-BLOCK bleibt defer (wie in Ticket 03, mehrdeutige
Bezeichner). Defer (unverändert): C-CASE/Ein-Buchstaben-Wort, C-GOOD-ERR/Fehlertext,
C-RW-VALUE, C-NO-OUT, C-CALLER-CONTROL, C-ITER-TY, C-NEWTYPE-HIDE, C-FAILURE.
Reject: die 40 Zeilen aus Ticket 03.

ast-grep-Fallen, die sich bestätigten und in den Regeln umgangen sind: `fn $N(&self)`
matcht `pub fn` nicht (Regeln arbeiten mit `kind: function_item` und `has`); zwei
`bool`-Parameter laufen über `follows` zwischen `parameter`-Knoten statt `$$$`;
Macro-Bodies sind flache Token-Bäume (deshalb nur Regeln auf `macro_invocation`-Ebene).
Neu gefunden:
- `kind: ','`/`'('`/`'['` als Regel-`kind` schlägt mit "Missing Selector" fehl.
- `follows`/`has` mit nur `field:` und ohne positiven Matcher ist ungültig.
- `stopBy` verlangt einen positiven Matcher. "Nicht-Attribut" geht nur als
  `all: [{regex: '(?s).*'}, {not: ...}]`.
- `nthChild` mit `field:` zusammen verhält sich unbrauchbar. G-28 zählt
  stattdessen Geschwister mit `ofRule` (ohne `visibility_modifier`/Attribut/Kommentar).
- norma meldet `location.line` nullbasiert.

### Verifikation

Binary: `./target/debug/norma` (Build vom 2026-10-28, ast-grep 0.45.3).

1. Pro Regel positive und negative Beispiele (Scratchpad `fx/*.rs`, nicht im Repo,
   `// POS`/`// NEG`-markierte Zeilen) erst mit `ast-grep scan -r`, danach über
   `norma validate --language rust --json` gegen eine Temp-DB mit den drei Packs.
   Ergebnis: alle 40 Regeln feuern auf ihren POS-Fällen, kein NEG-Fall wird
   gemeldet. Abweichungen von meinen ersten Labels waren Fixture-Fehler (ein NEG
   war ein echter Treffer: `m.lock().unwrap()` ist ein normales `.unwrap()`), keine Regelfehler. Nur die oben
   genannten Grenzen (G-65 Trailing-Kommentar, B3 Trailing-`//`) sind bekannt offen.
2. Eine sauber geordnete, idiomatische Datei (`fx/clean.rs`: Extern-Crate, `use`,
   `#[macro_use] mod`, Doc vor Attribut, `non_exhaustive`, `try_new`, SAFETY-Kommentar,
   `#[cfg(test)]`-Modul mit `unwrap`) erzeugt über alle drei Packs 0 Treffer.
3. Import je Verzeichnis in frische Temp-DB:
   `norma import rule-packs/google-rust-style --category google-rust-style` ->
   20 imported, 0 skipped; `rust-style-guide` -> 10 imported, 0 skipped;
   `rust-api-guidelines` -> 10 imported, 0 skipped.
4. Kollisionscheck: eine frische DB mit allen Bestands-Packs (architecture 2,
   code-quality 6, encapsulation 7, safety 3, security 21 = 39, je 0 skipped), den
   20 Defaults und den drei neuen Packs enthält 99 Patterns (20 + 39 + 40), keine
   doppelte ID.
5. Plausibilitätslauf gegen `src/` und `tests/` (kein Fix, nur Zählung): 18x
   `google-non-exhaustive-pub-struct-rust`, 3x `google-non-exhaustive-pub-enum-rust`,
   3x `google-prefer-expect-over-unwrap-rust`, je 1x `google-use-one-module-per-use-rust`,
   `google-new-returning-result-rust`, `style-use-before-other-items-rust`, 2x
   `google-no-combinator-chain-in-for-head-rust`. Das ist tragbar; die `non_exhaustive`-
   Treffer sind erwartbares Rauschen für eine Anwendung (hint).

Nicht getestet: Verhalten auf grossen fremden Codebasen; Lauf mit `--fix` (keine
der Regeln hat ein `fix:`); die Rauschquote von B7/B8/B10 ausserhalb von norma.
