# GoF Pattern Set v2 -- Design

**Status:** approved for planning
**Depends on:** ADR 0001 (single binary, shared core), ADR 0002 (Pattern single-language, full RuleConfig YAML), the MVP implementation (`docs/superpowers/plans/2026-09-20-norma-mvp-implementation.md`), and Ticket 06 (`.scratch/norma-architecture/issues/06-gof-pattern-set-v2-scope.md`), which holds the 16 verified `rule` YAML documents this design references but does not repeat in full.

## Goal

Extend norma's default pattern set from the MVP's four `no-debug-print` patterns to also cover four classic Gang-of-Four patterns -- Singleton, Factory, Observer, Strategy -- across all four MVP languages (Java, Python, Rust, TypeScript), deliberately deferred out of the MVP by Ticket 05.

## Scope

**In scope:**
- 16 new `DefaultPattern` entries in `src/default_patterns.rs` (4 patterns x 4 languages), using the YAML verified in Ticket 06.
- A `pattern_engine::validate` change so an `info`/`hint`/`off`-severity match is reported (visible in `violations`) but does not count against `passed`/`score` -- needed so `observer-presence-*` doesn't misrepresent a well-formed Observer as a quality problem.
- Test coverage mirroring the MVP's structure: engine-level severity/score tests, an extended `default_patterns` coverage test, and pattern-specific engine tests for at least one representative GoF pattern per direction (a/b/c, see below).

**Out of scope (unchanged from Ticket 05/06):**
- Any pattern beyond these four; a v3 set is a separate future decision.
- Perfect static-analysis precision for Factory/Strategy -- see "Known limitations" below. norma is an FFHS teaching artifact (see the MVP plan's Global Constraints), not a production-grade analyzer; the heuristics are intentionally readable and explainable over exhaustive.
- Any change to `PatternStore`, the MCP tool surface, or the CLI -- this is purely a content addition plus the one engine fix above; nothing about how patterns are registered, stored, or invoked changes.

## Pattern catalog

| Pattern | Direction | Category | Severity | ID scheme | Shared `name` |
|---|---|---|---|---|---|
| Singleton | (a) flawed implementation: looks like a Singleton but the constructor isn't private | `creational` | `warning` | `singleton-quality-<lang>` | "Singleton Implementation Quality" |
| Factory | (b) anti-pattern/overuse: a type-switch (`if`/`elif` chain) that directly constructs a different type per branch | `creational` | `warning` | `factory-overuse-<lang>` | "Factory Overuse (Type Switch)" |
| Observer | (c) pure presence, no value judgment: a listener/observer collection plus a notify-shaped method | `behavioral` | `info` | `observer-presence-<lang>` | "Observer Presence" |
| Strategy | (b) anti-pattern/overuse: a type-switch that calls a different *method* (behavior, not construction) per branch | `behavioral` | `warning` | `strategy-overuse-<lang>` | "Strategy Overuse (Type Switch)" |

This mixture was a deliberate choice (see Ticket 06): Singleton is the one GoF pattern in this set that's structurally checkable for *correctness*; Factory/Strategy absence is better expressed as "here's a code smell that suggests you're missing this pattern" than as a presence/absence check; Observer has no reliable structural "correctness" signal across four languages, so it stays informational.

All 16 `rule` YAML documents are already written and verified against real `ast-grep-core` 0.45.3 -- see Ticket 06 for the full text and the two general pitfalls found while writing them (`has` needs `stopBy: end` to search descendants rather than only direct children; a `pattern` fragment that isn't valid standalone code needs the `context`/`selector` long form).

## Architecture

**No new modules, no schema change.** This is purely a `src/default_patterns.rs` content addition (Task-5-shaped, same as the MVP), following the exact same `DefaultPattern` struct and `PatternStore::seed_defaults` mechanism already in place. `pattern_engine::parse_rule` and `find_violations` already handle arbitrary `RuleConfig` shapes (`kind`/`has`/`all`/`not`/`pattern`/`regex`) generically -- GoF rules use more of that vocabulary than the MVP's plain `pattern` rules did, but nothing about the engine's *parsing* needs to change.

**The one real code change** is in `pattern_engine::validate`'s scoring: today, every match found (regardless of severity) is a `PatternViolation` that counts equally against `score` and can flip `passed` to `false`. That's correct for `warning`/`error` (Singleton, Factory, Strategy) but wrong for `info` (Observer) -- finding a well-formed Observer isn't a defect. The fix: only `warning`/`error` severity matches count toward the violation tally that drives `score`/`passed`; `off`/`hint`/`info` matches are still collected and returned in `ValidationResult.violations` (so `get_pattern_checklist`/`list_patterns`-style visibility is unaffected, and a human or the MCP client can still see "an Observer was found here"), they just don't make the run "fail".

Concretely, `ValidationResult` and `validate` need to distinguish "counted" violations from "informational" ones for the score computation. The simplest change that preserves the existing `ValidationResult` shape (no field additions, no consumer-visible break): filter by severity when computing `checked`'s denominator... no -- `checked` already counts *patterns*, not matches, and stays correct. Only the numerator (violations counted against `passed`/`score`) changes: partition `violations` into scoring vs. informational by severity when computing `passed` and `score`, while `violations` (the full list, unchanged) keeps every match for visibility.

```rust
// pattern_engine.rs, inside validate(), after `violations` is fully populated:
let scoring_violations = violations
    .iter()
    .filter(|v| matches!(v.severity, Severity::Warning | Severity::Error))
    .count();
let passed = scoring_violations == 0;
let score = if checked_patterns == 0 {
    1.0
} else {
    (1.0 - scoring_violations as f64 / checked_patterns as f64).max(0.0)
};
```

This keeps the existing `coverage_warning` synthetic entry's behavior intact (it's `Warning`-severity, so it still counts, which is correct -- degraded coverage should fail a run, unlike a benign Observer sighting).

## Testing

Mirrors the MVP's per-module test structure:

- **`pattern_engine.rs`:** two new tests -- an `info`-severity match appears in `violations` but leaves `passed: true` and `score: 1.0`; a `warning`-severity match still fails the run exactly as before (regression guard for the scoring change).
- **`default_patterns.rs`:** the existing `every_default_rule_parses_and_covers_one_mvp_language_each`-style test extended to 20 entries (4 MVP + 16 GoF), asserting every rule parses, every rule's derived language is one of the four supported languages, and (new) that the id prefix (`singleton-`/`factory-`/`observer-`/`strategy-`) maps to the expected `category` (`creational`/`behavioral`).
- **New pattern-behavior tests:** for each of the 4 GoF patterns, at least one positive-match and one negative-match test in `pattern_engine.rs`'s test module (or a new `tests/gof_patterns.rs` integration test, whichever the implementation plan's task breakdown finds cleaner) -- using the exact source snippets already verified in Ticket 06, so the plan's tests aren't inventing new unverified examples.
- **`tests/dogfooding.rs`:** unchanged; optionally note in a comment that norma's own source also contains no Singleton/Factory/Observer/Strategy shapes, but this isn't asserted as a new test (norma's source is simple enough that this would be redundant with the existing "no debug print" dogfooding check).

## Known limitations (documented, not fixed here)

- **Factory/Strategy can't distinguish "two branches" from "many branches"** -- `ast-grep`'s declarative rule format has no counting/aggregation primitive, so the rule matches on the *shape* "at least one `if`/`else if` pair, each doing X", not "N or more branches". A two-branch `if`/`else` that happens to construct two different logging adapters would technically match; this is an acceptable false-positive rate for a teaching tool flagging a *smell to think about*, not a hard rule violation like `no-debug-print`.
- **Rust and TypeScript don't have Java/Python-style classes**, so Singleton/Observer for those two languages target the nearest idiomatic equivalent (`struct`+`impl` for Rust, unchanged `class` for TypeScript) rather than a literal translation of the Java shape.
- **Severity `info` matches still show up in `PatternViolation`s named field `message`/`matched_text`** exactly like warnings -- consumers that don't check `severity` before deciding "is this bad" will still see Observer sightings mixed into the same list. This is intentional per the design above (visibility without penalty), not a bug to fix in this pass.
- **`factory-overuse-python` and `strategy-overuse-python` are structurally the same rule** -- tree-sitter-python's `call` node carries no type information, so there is no AST-kind distinction between "constructing a type" (`Dog()`) and "calling a method" (`pay_by_card()`) for ast-grep to match on; both rules end up matching identically on any `if`/`elif` type-switch, under different metavariable names. This means every Python type-switch is double-reported with contradictory advice ("extract a Factory" vs. "extract a Strategy") for the same code, and the validation score is doubly penalized for Python relative to the other three languages. Java, Rust, and TypeScript aren't affected, because their grammars distinguish construction from plain calls at the AST-kind level (`object_creation_expression`/`new_expression` vs. `method_invocation`/`call_expression` in Java/TypeScript, struct-literal expressions vs. `call_expression` in Rust). Properly disambiguating the two Python rules would require re-verifying changed YAML against real `ast-grep-core`, which is out of scope for this release; this is accepted as a known limitation and deferred to a future pattern-set revision.

## Open questions

None -- Ticket 06 resolved the scope, direction-per-pattern, and rule verification; this design resolved the one engine question (severity-aware scoring). Ready for `writing-plans`.
