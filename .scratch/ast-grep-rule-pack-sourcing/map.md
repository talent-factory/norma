# norma — ast-grep Rule-Pack Sourcing (wayfinder map)

## Destination

A set of importable ast-grep rule-pack YAML files under `rule-packs/`
(loadable via `norma import rule-packs/<category> --category <category>`,
`default_patterns.rs` untouched) reflecting every pattern adopted from
(a) the [ast-grep catalog](https://ast-grep.github.io/catalog), (b)
Daniel's own active repos, (c) other permissively-licensed GitHub
ast-grep rule-set repos, and (d) a curated subset of
`coderabbitai/ast-grep-essentials` (widened into scope after tickets
01-03 landed -- see ticket 04). Each source's ticket ends in a decided
adopt/defer/reject list (with one-line rationale per candidate) plus the
drafted `RuleConfig` YAML for every "adopt".

## Notes

- **Execution carried into this map** (override of wayfinder's default
  "plan, don't do"): resolving a ticket directly produces its rule-pack
  YAML file(s), not just a decision handed off for later implementation.
- **Linter boundary:** only patterns expressing design/architecture intent
  or cross-cutting coding standards that a mainstream linter for that
  language (Ruff/ESLint/Biome/Clippy/Checkstyle/...) can't already
  express -- mirrors norma's existing GoF (Singleton/Factory/
  Observer/Strategy) + "no debug print" default set. Reject anything that
  duplicates what such a linter already covers well.
- **Delivery:** opt-in importable rule-pack YAML files in a new top-level
  `rule-packs/` directory, structured **by category, not by source**
  (`norma import <dir>` takes exactly one `--category` per invocation and
  recurses, so a source-per-file layout would have mis-stamped
  `community.yaml`'s mixed categories): `rule-packs/<category>/<source>.yaml`
  -- e.g. `rule-packs/code-quality/community.yaml`,
  `rule-packs/encapsulation/community.yaml`. Verified end-to-end with a
  real `norma import rule-packs/<category> --category <category>` per
  category (18/18 imported, 0 skipped, no id collisions with the 20
  existing defaults).
- **Own-repo mining scope:** the top active repos under
  `/Users/daniel/GitRepository/` by last-commit-date as of chartering
  (2026-09-26): llm-gateway, SubscribeFlow, specula,
  specula-client-python, ratum, examcraft-private, website, norma.
  Dominant languages found: Python (heavy across all), TypeScript/TSX
  (SubscribeFlow, ratum, examcraft-private), Rust (norma only -- the
  Rust hits originally attributed to examcraft-private turned out to be
  vendored third-party code under a `.venv`, not first-party, per
  [Own-Repo Pattern Mining](issues/02-own-repo-pattern-mining.md)). No
  Java in this set.
- **License boundary (GitHub search track):** only rules from
  MIT/Apache/BSD-licensed source repos may be proposed as "adopt";
  GPL/unclear-licensed repos get catalogued as "found, license-blocked",
  never silently dropped and never adopted.
- **No Linear graduation for this effort** -- this map plus its merged PR
  history is the record. Linear is only for a finding that needs a real
  norma engine change (new MCP tool, new language plumbing, etc.), not a
  rule -- that would be its own, separate effort.
- Every ticket here is `research`/AFK, resolved by a background agent
  (fork, since it inherits this map's full chartering context).
- **Scope widened 2026-09-26 (ticket 04):** after discussing ticket 03's
  deferred `coderabbitai/ast-grep-essentials` finding, Daniel confirmed
  he runs no secret-/SAST-scanner in his active repos today (so this is
  net-new value, not redundant with existing tooling) and opted for a
  **curated subset**, not the full 98 java/python/rust/typescript rules
  wholesale: dedupe the mechanically-repeated per-library variants (e.g.
  ~40 Python "hardcoded secret in driver X" rules) down to one generic
  rule per distinct concept (e.g. one generic hardcoded-secret-in-DB-call
  rule, one generic empty-password rule) per language, plus any rule
  that's conceptually distinct on its own (not a library-specific
  duplicate). New category: `rule-packs/security/`.

## Decisions so far

- [Own-Repo Pattern Mining](issues/02-own-repo-pattern-mining.md) — Surveyed 8 active repos; 1 adopted (`direct-fetch-bypass-typescript`, an architecture-layering rule, in `rule-packs/team-conventions.yaml`), 2 deferred for insufficient evidence, 4 rejected (either zero occurrences or, on inspection, not genuine anti-patterns). These repos are already unusually clean against classic anti-patterns.
- [ast-grep Catalog Analysis](issues/01-ast-grep-catalog-analysis.md) — Surveyed all 63 catalog rules; 4 adopted into `rule-packs/ast-grep-catalog.yaml` (Rust: `avoid-duplicate-export`, `redundant-unsafe-function`; TypeScript: `no-await-in-promise-all`; TSX: `unnecessary-react-hook`), rest rejected/deferred as linter-territory, one-off codemods, or search demos -- Java and Python contributed zero adopts. Flags a category-per-import-invocation constraint worth resolving before final assembly.
- [GitHub External Rule Repos](issues/03-github-external-rule-repos.md) — `gh search repos` + manual follow-up; 13 rules adopted into `rule-packs/community.yaml` (9 Rust + 1 TypeScript from joshuadavidthomas/ast-grep-rules, 3 Python/Java from Zerkath/ast-grep-rules -- 2 of those are direct siblings of norma's existing "no debug print" family). The most popular hit by far, coderabbitai/ast-grep-essentials (Apache-2.0, 149 stars), deferred wholesale: it's entirely security/SAST rules (hardcoded secrets, weak crypto), a different product category than this map's design-pattern/coding-standard scope -- candidate for its own future effort. 8 repos license-blocked (logged, not adopted), including two -- Java rules and architecture-invariant rules -- that would otherwise have been strong candidates.
- [coderabbitai Security Curation](issues/04-coderabbitai-security-curation.md) — Daniel confirmed no existing secret-/SAST-scanner in his active repos (net-new value) and opted for a curated subset over the full 98-rule import. 21 generalized rules drafted into `rule-packs/security/{python,rust,typescript,java}.yaml`, collapsing ~80 mechanically-repeated per-library originals (e.g. ~25 Python "hardcoded secret in driver X" variants → 1 generic rule) down to one rule per distinct security concept. Confidence varies per rule (see ticket's table) -- `hardcoded-secret-in-credentials-java` is flagged as least-verified.

## Not yet specified

_(none remaining -- all three tickets resolved; see "Status" below for what's left before this map closes)_

## Out of scope

- Engine/feature changes to norma itself (new MCP tools, new language
  support, `default_patterns.rs` edits) -- this effort only produces
  importable rule-pack YAML; a finding that needs real code changes
  starts its own, separate effort.
- Repos under `/Users/daniel/GitRepository/` outside the top active set
  listed in Notes -- not mined for this effort.
- Adopting rules from GPL/unclear-licensed external repos -- catalogued
  as found-but-license-blocked, never proposed as "adopt".

## Status

Chartered 2026-09-26, all four tickets resolved and verified same day.
`rule-packs/` assembled across 5 category directories (code-quality,
architecture, encapsulation, safety, security), each import-clean --
39 rules total (18 from tickets 01-03 + 21 curated from
`coderabbitai/ast-grep-essentials` in ticket 04), 0 id collisions with
the 20 existing defaults. Destination reached; map closed.

**Follow-up spot-checks done 2026-09-26:** every rule in the security
pack that carried a Medium confidence rating (or lower) has now been
checked against its real upstream file(s) and, where wrong, fixed and
re-verified with a real `norma import` + `norma validate` run:

- `hardcoded-secret-in-credentials-java` -- missed
  `setConnectionPassword`/`updatePassword`/`Credentials.basic`. Fixed.
- `hardcoded-secret-in-call-typescript` / `empty-password-accepted-typescript`
  -- both silently missed every `new X(...)` constructor call (`kind:
  call_expression`-only gate); the latter also false-positived on any
  unrelated call with an empty-string argument. Fixed.
- `weak-rsa-key-size-typescript` -- threshold regex missed the
  2000-2047 range. Fixed.
- `weak-crypto-algorithm-java` -- didn't catch "ALGO/MODE/PADDING"
  transformation strings (e.g. "DES/CBC/PKCS5Padding") for any algorithm
  except (accidentally) none at all. Fixed for every algorithm except
  AES, which must stay exact-match ("AES/GCM/NoPadding" is the secure
  form and must not be flagged).
- `empty-password-accepted-rust` -- checked, no bug found; the
  naming-symmetry inference that drafted it was correct.

Full before/after detail and verification commands are in ticket 04's
"Follow-up" and "Follow-up 2" sections. All 21 security-pack rules
still import cleanly (0 skipped) after every fix.
