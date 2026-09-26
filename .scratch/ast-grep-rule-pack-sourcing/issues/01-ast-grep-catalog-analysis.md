Type: research
Status: resolved

## Question

Survey the [ast-grep catalog](https://ast-grep.github.io/catalog) (across
its languages, prioritizing java/python/rust/typescript since those are
norma's default-pattern languages, but noting broadly-relevant rules for
other languages too) and decide, per candidate rule, adopt/defer/reject
against the map's linter boundary (see `../map.md` Notes): only
design/architecture-shaped or cross-cutting coding-standard rules that a
mainstream linter for that language can't already express well.

For every "adopt", draft its complete ast-grep `RuleConfig` YAML
(`id`/`message`/`severity`/`language`/`rule`/optionally `fix`, per ADR
0002 -- copy-pasteable straight from the catalog's own rule page) and
place it into `rule-packs/ast-grep-catalog.yaml` as one document in a
multi-document YAML stream.

Record the full decided list (rule name, adopt/defer/reject, one-line
rationale each) as this ticket's resolution.

## Answer

Surveyed all 63 rules currently on the catalog (fetched live 2026-09-26).
The catalog skews heavily toward one-off codemods/migrations and
"find code" search demos rather than standing anti-pattern checks, so
the adopt rate is low and Java/Python contributed zero adopts.

**Adopted (4)** -- drafted into `rule-packs/ast-grep-catalog.yaml`,
verified by a real `norma import` against a throwaway DB (18 rules
imported across all forks' packs so far, 0 skipped/parse errors):

- `avoid-duplicate-export-rust` ([source](https://ast-grep.github.io/catalog/rust/avoid-duplicated-exports)) -- re-exporting one item under two paths bloats API surface; not caught by Clippy.
- `redundant-unsafe-function-rust` ([source](https://ast-grep.github.io/catalog/rust/redundant-unsafe-function)) -- an `unsafe fn` with no `unsafe` block inside is a safety-contract smell; genuinely distinct from rustc's `unsafe_op_in_unsafe_fn` lint, which only fires the opposite way (unwrapped unsafe *ops* present, not a spuriously-marked fn with none).
- `no-await-in-promise-all-typescript` ([source](https://ast-grep.github.io/catalog/typescript/no-await-in-promise-all)) -- `await` inside a `Promise.all([...])` array defeats parallelism; no mainstream ESLint core/plugin rule catches this exact shape. Upstream targets `language: javascript`; adapted to `TypeScript` since that's norma's default-pattern language and the grammar is a strict superset for this pattern.
- `unnecessary-react-hook-tsx` ([source](https://ast-grep.github.io/catalog/tsx/unnecessary-react-hook)) -- a `use*`-named function that calls no other hook should be a plain function; mirrors norma's existing "overuse of a construct" family (Factory/Strategy Overuse). Not covered by `eslint-plugin-react-hooks` (which checks dependency arrays, not overuse).

**Rejected as linter/tooling territory** -- Java: `no-unused-vars` (compiler/Checkstyle/ErrorProne already do this); Python: `optional-to-none-union` (duplicates Ruff's `UP007`), `use-walrus-operator-in-if` (style, not design), `prefer-generator-expressions` (overlaps Ruff `C4`/`PERF`); TS: `use-logical-assignment` (core ESLint rule already), `no-console-except-catch` (refines norma's own existing `no-debug-print-typescript`, not a new concept); TSX: `avoid-jsx-short-circuit` (covered by `eslint-plugin-react`'s `jsx-no-leaked-render`); Go: `loopvar-capture`, `range-over-int`, `prefer-*`/`clear`/`time-since` (all covered by `golangci-lint`/the official Go `modernize` analyzer).

**Rejected as codemods/one-off migrations, not standing rules** -- Python: `migrate-openai-sdk`, `refactor-pytest-fixtures`, `remove-async-await`, `rewrite-sqlalchemy-mapped-column`; Rust: `rewrite-indoc-macro`, `rust-2024-let-chain-candidate`; TS/TSX: `migrate-xstate-v5`, `switch-from-should-to-expect`, `rename-svg-attribute`, `reverse-react-compiler`, `rewrite-mobx-component`.

**Rejected as search-utility demos, not violation rules** -- Java: `find-field-with-type`; Rust: `get-digit-count-in-usize`; TS: `find-import-file-without-extension`, `find-import-identifiers`, `find-import-usage`.

**Deferred (narrow/uncertain fit, not adopted)** -- Rust: `boshen-footgun` (char-offset string slicing; likely already covered by Clippy's opt-in `string_slice` restriction lint); TS: `speed-up-barrel-import` (real but bundler-dependent, not universal); TSX: `avoid-nested-links` (HTML-validity/a11y territory, `eslint-plugin-jsx-a11y`'s domain), `redundant-usestate-type` (TS redundancy, style-adjacent), `missing-component-decorator` (Angular-specific, narrow, likely covered by `angular-eslint`).

**Noteworthy but out of the java/python/rust/typescript core** (flagged, not drafted): Kotlin's `ensure-clean-architecture` is a strong signal that "layering rules" (a domain not-yet in norma's default set) are exactly norma's niche -- worth remembering if Kotlin is ever added. Go's `defer-func-call-antipattern` is plausibly adopt-worthy on its own merits but Go isn't a default-pattern language and evaluating Go-linter overlap fully was out of this ticket's scope.

**For ticket 03 (GitHub search):** found `coderabbitai/ast-grep-essentials` ("Community-led collection of essential ast-grep rules") via `gh search repos ast-grep` while scoping this ticket -- didn't investigate further (out of this ticket's scope), flagging for that ticket if not already found independently.

**Process note for map assembly:** `norma import <dir>` applies exactly
one `--category` per invocation across every file found recursively
under `<dir>` (no per-document category field in ast-grep's own YAML).
All 4 rules here share `code-quality`, so `rule-packs/ast-grep-catalog.yaml`
alone is import-clean as-is -- but if the assembled `rule-packs/` tree
ends up mixing categories across sibling files, a flat `norma import
rule-packs/` would mis-stamp everything with one category. Worth
resolving whether that needs subdirectories per category before this
effort is called done (relates to the existing "Not yet specified" note
on `rule-packs/community.yaml`'s structure).
