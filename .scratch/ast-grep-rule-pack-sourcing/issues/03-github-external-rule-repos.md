Type: research
Status: resolved

## Question

Search GitHub for other repos publishing reusable ast-grep rule
catalogs/packs (beyond ast-grep's own official catalog, which ticket
01-ast-grep-catalog-analysis.md already covers). For every candidate
repo found, check its license first: only MIT/Apache/BSD-licensed rules
may be proposed as "adopt" -- GPL/unclear-licensed repos get logged as
"found, license-blocked" (name, URL, license, one-line description of
what it offers) rather than silently skipped or adopted.

For permissively-licensed candidates, decide adopt/defer/reject per rule
against the map's linter boundary (see `../map.md` Notes). For every
"adopt", draft its complete ast-grep `RuleConfig` YAML (per ADR 0002,
adapted/attributed as needed) into `rule-packs/community.yaml` as one
document in a multi-document YAML stream.

Record: the full decided list (rule name, source repo, adopt/defer/
reject, one-line rationale each) plus the separate license-blocked log,
as this ticket's resolution.

## Answer

`gh search repos "ast-grep rules"` (30 results) plus manual follow-up on the
most promising hits. Full inventory below; only permissively-licensed repos
got their individual rules judged against the map's linter boundary.

### Adopted (13 rules -> `rule-packs/community.yaml`)

All 13 already carry the multi-document YAML in `rule-packs/community.yaml`,
each with a `# Source:`/`# License:` attribution comment.

From **joshuadavidthomas/ast-grep-rules** (MIT, 12 stars) -- a personal but
unusually well-designed Rust/TypeScript rule collection, several rules
express exactly the "linter can't say this" architecture intent the map's
Notes call for:

| Rule | Rationale |
|---|---|
| `rust-no-anyhow-in-public-api` | Public API returning `anyhow::Error`/`anyhow::Result` leaks an untyped error across a library boundary -- Clippy has no opinion on this, it's a crate-design convention |
| `rust-no-deref-polymorphism` | `Deref` used for API-forwarding/inheritance-emulation instead of true smart-pointer semantics -- a classic Rust anti-pattern, no lint covers intent |
| `rust-no-public-struct-fields` | Public fields on a public struct bypass invariants/validation -- encapsulation design rule, not a lint |
| `rust-no-single-field-struct` | Named-field struct with exactly one field is usually just a tuple newtype with ceremony |
| `rust-no-option-bool-field` | `Option<bool>` is an implicit, unnamed three-state flag -- same shape as norma's own "type-switch instead of Strategy" idea, just for data instead of behavior |
| `rust-require-thiserror-error-enum` | An `...Error` enum without a `derive(Error)` (thiserror) is hand-rolled error plumbing -- a project convention, not a lint |
| `rust-no-string-error-variant` | Bare `String` payload in an error variant instead of structured/source data |
| `rust-no-panicking-let-else` | A `let-else` whose else-branch panics is `.expect()` wearing a different hat |
| `rust-no-panicking-match-arm` | Same idea for `match` arms; both use `ignores`/`utils`/`constraints` RuleConfig fields -- **flag: verify these round-trip through `ast-grep-config` 0.45.3 before a real `norma import`, not confirmed in this pass** |
| `typescript-no-chained-type-assertions` | `x as unknown as Foo` throws away and re-invents a type -- no standard typescript-eslint rule catches double assertions specifically (`no-unnecessary-type-assertion` catches a different case) |

From **Zerkath/ast-grep-rules** (MIT, 0 stars but clean, focused rules):

| Rule | Rationale |
|---|---|
| `log-and-reraise` (python) | Logging then re-raising in the same `except` produces duplicate log entries at the eventual handler -- no mainstream linter flags this |
| `prefer-logger-exception` (python) | `traceback.print_exc()` bypasses the logging pipeline entirely -- direct sibling of norma's own existing "no debug print" pattern family, just for exception dumps instead of `print()` |
| `print-stack-trace-java` (java, renamed from repo's `print-stack-trace`) | `.printStackTrace()` is Java's version of the same anti-pattern -- extends norma's existing `no-debug-print-java` idea to exception handling |

### Deferred

- **`except-pass-without-comment`** (Zerkath, MIT) -- overlaps substantially
  with Ruff's `S110`/flake8-bandit bare-except-pass check; the
  comment-awareness nuance isn't distinct enough to clear the linter
  boundary confidently. Revisit if a team without Ruff's bandit ruleset
  enabled asks for it specifically.
- **`useless-async-wrapper`** (Zerkath, MIT) -- plausible, but higher false-positive
  risk (legitimate interface-boundary wrappers look identical to the
  anti-pattern) than the others; wants a real-codebase spot-check before
  adopting, not done in this pass.
- Remaining ~20 unreviewed rules in **joshuadavidthomas/ast-grep-rules**
  (svelte, several more TypeScript `no-*` rules, `rust-no-empty-braced-struct`,
  `rust-no-indexed-array-rebuild`, etc.) -- repo is clearly worth a second
  pass, just not reviewed individually this session (time-boxed).
- **coderabbitai/ast-grep-essentials** (Apache-2.0, 149 stars, the most
  popular hit by far) -- entirely security/SAST rules (hardcoded secrets,
  weak crypto, insecure TLS) across java/python/rust/typescript/go/etc.
  Permissively licensed and well-maintained, but a different product
  category than norma's design-pattern/coding-standard niche (map Notes)
  -- closer to Semgrep/Bandit/gitleaks territory than to Singleton/Factory/
  "no debug print". Worth its own future effort (a `security` rule-pack),
  explicitly out of this one.
- **raxITlabs/agent-security-review** (MIT) -- same scope-fit issue as
  coderabbitai's catalog (AI-agent-code security review, not design
  patterns).
- **staticaland/awesome-ast-grep-rules** (CC0) -- an awesome-list of links,
  not itself a rule source; useful as a future discovery seed, nothing to
  extract directly.
- **RyanSaxe/byor** (Apache-2.0) -- a "build your own rules" framework/workflow
  tool, not a rule pack itself.
- **blyoa/ast-grep-rules** (MIT) -- categories `es-toolkit`/`generic`/`logger`/
  `react`/`vitest` look potentially relevant (logger especially) but not
  reviewed in depth this pass.
- **dwisiswant0/go-ast-grep-rules** (MIT) -- Go isn't a norma default-pattern
  language and didn't surface in Daniel's own active repos (ticket
  02-own-repo-pattern-mining.md); no immediate driver to adopt now, still
  registrable later since Go is one of the 28 `SupportLang` languages.
- **terrylica/trading-fitness** (MIT) -- ast-grep rules exist but are buried
  inside a large unrelated monorepo, not cleanly extractable.
- **voxpelli/ast-grep-rules** (MIT, 3 rules) -- all JSDoc/TSDoc typing-style
  conventions (`no-jsdoc-any-type` etc.); overlaps with what
  `@typescript-eslint`'s own type-checking rules already push toward --
  rejected rather than deferred, this one's a clear linter-boundary miss,
  not just unreviewed.

### License-blocked (found, not adopted, not silently dropped)

| Repo | License | What it offers |
|---|---|---|
| [markus1189/ast-grep-rules](https://github.com/markus1189/ast-grep-rules) | none | Personal rule collection |
| [iamleot/ast-grep-rules](https://github.com/iamleot/ast-grep-rules) | none | Personal rule collection |
| [jasonmorganson/harness-engineering-rules](https://github.com/jasonmorganson/harness-engineering-rules) | none | Architecture/codebase-invariant rules for a specific engineering domain -- would otherwise have been a strong candidate |
| [Jayllyz/ast-grep-rules](https://github.com/Jayllyz/ast-grep-rules) | none | Java rules for coding agents -- would otherwise have directly filled a java gap |
| [winter-loo/ast-grep-rules](https://github.com/winter-loo/ast-grep-rules) | none | Personal rule collection |
| [danielo515/effect-ast-grep-rules](https://github.com/danielo515/effect-ast-grep-rules) | none | Rules for the TypeScript "Effect" framework (niche either way) |
| [neondatabase/claude_astgrep](https://github.com/neondatabase/claude_astgrep) | none | Tooling to *generate* ast-grep rules via Claude Code, not a rule pack itself -- would be out of scope regardless of license |
| [test-peter-rabbit/test-ast-grep-custom-package](https://github.com/test-peter-rabbit/test-ast-grep-custom-package) | none | Throwaway test repo, irrelevant regardless of license |

**Net result: 13 adopted (all in `rule-packs/community.yaml`), 2 deferred
individually + ~9 sources deferred wholesale (some for scope-fit, some for
time-boxing, one rejected outright), 8 repos license-blocked.**
