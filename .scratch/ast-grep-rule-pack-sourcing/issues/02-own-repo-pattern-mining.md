Type: research
Status: resolved

## Question

Mine the top active repos under `/Users/daniel/GitRepository/` (per the
map's Notes: llm-gateway, SubscribeFlow, specula, specula-client-python,
ratum, examcraft-private, website, norma) for recurring patterns and
anti-patterns worth generalizing into a reusable ast-grep rule for future
projects, filtered by the map's linter boundary (see `../map.md` Notes).

Decide adopt/defer/reject per candidate. For every "adopt", draft its
complete ast-grep `RuleConfig` YAML (per ADR 0002) into
`rule-packs/team-conventions.yaml` as one document in a multi-document
YAML stream -- generalized (no project-specific names/paths), and
verified against at least one real snippet from the source repo it came
from (cite repo + rough location, not necessarily an exact line number).

Record the full decided list (pattern name, source repo(s) it was
observed in, adopt/defer/reject, one-line rationale each) as this
ticket's resolution.

## Answer

**Correction to the map's Notes first:** the "Rust in examcraft-private"
language signal was a false positive from the original recency/language
scan -- all ~179 `.rs` hits there are vendored third-party code under
`core/backend/.venv/lib/python3.13/site-packages/temporalio/bridge/**`
(the `temporalio` Python package's Rust extension source), not first-party
code. examcraft-private has no first-party Rust; Rust in this survey's
scope is norma only.

Grep/Glob/Read across llm-gateway, SubscribeFlow, specula,
specula-client-python, ratum, examcraft-private, website, norma (excluding
`.venv`/`node_modules`/`.git`), looking for idioms recurring across more
than one repo:

| Candidate | Observed in | Decision | Rationale |
|---|---|---|---|
| Bare `except:` / `except Exception: pass` (silent failure swallow) | none found (0 hits in any of the 8 repos) | Reject | No evidence -- these repos are already disciplined here (likely ruff `BLE001`/E722 already enforced + prior silent-failure-hunter-style review), nothing to generalize |
| Module-level Python singleton idiom (`global _instance` guard function, distinct from norma's existing class-based `_instance = None` check) | none found | Reject | No evidence |
| Mutable default arguments (`def f(x=[])`) | not surveyed in depth | Defer | Already well-covered by Ruff (B006)/most linters -- squarely on the wrong side of the map's linter boundary even before checking occurrence |
| Scattered raw `requests`/`httpx` calls outside a shared HTTP client wrapper (Python) | ratum (`telegram.py`, `email.py`, `notifications.py` -- 3 files, one external integration each) | Reject | Too few, too legitimately single-purpose (one webhook/API integration per file) to call it drift; not a repeated anti-pattern at meaningful scale |
| Manual retry loop without a backoff library (`for attempt in range(N): try: ... except: ...`, no `tenacity`/`backoff` dependency anywhere) | SubscribeFlow (4), specula (1), ratum (4), examcraft-private (18, mostly test-loop false positives) | Reject | Spot-checked the strongest instance (`examcraft-private/core/backend/tasks/question_tasks.py:159-169`) -- it already implements its own explicit backoff array (`_JOB_STATUS_UPDATE_BACKOFFS`) with attempt logging; a deliberate, quality implementation, not a naive anti-pattern. Most other hits were test files iterating a fixed count, unrelated to retry semantics. No genuine repeated anti-pattern here |
| `get_settings()` without `@lru_cache` (repeated Settings() instantiation) | only 1 repo has this function at all (SubscribeFlow), and it already caches | Defer | Single data point either way -- can't responsibly generalize a rule from one repo having it right; revisit if a future repo gets this wrong |
| **Direct `fetch()`/`axios.*()` call in a page/route component, bypassing the project's established api-client + data-fetching-hook layer** | **SubscribeFlow**: `apps/web/src/routes/admin/pages/settings.tsx:543` and `apps/web/src/routes/admin/pages/api-key-ip-allowlist-dialog.tsx` both call `fetch()`/reference a raw API URL directly, vs. 28 other files in the same repo that go through `lib/*Api.ts` + `useQuery`/`useMutation` | **Adopt** | Concrete, verified drift against the repo's own dominant convention (28 files follow the layered pattern vs. 2 that bypass it) -- exactly the kind of architecture-boundary rule a mainstream linter (ESLint/Biome) can't express out of the box. Drafted as `direct-fetch-bypass-typescript` in `rule-packs/team-conventions.yaml`, `category: architecture`. Ships with an explicit caveat comment: the rule can't itself distinguish "inside the api-client layer" from "inside a component" (ast-grep `RuleConfig` is path-agnostic), so adopting projects must exclude their own api/lib directory via their `.pre-commit-config.yaml` `exclude:` glob -- same mechanism norma's own hook uses to exclude `src/main.rs` |
| React frontend surface (ratum) | `frontend/src/{pages,components}` | Reject | Zero raw `fetch`/`axios` calls found at all -- already fully routed through its data layer, nothing to flag |

**Net result: 1 adopted, 2 deferred (insufficient evidence either way, revisit later), 4 rejected.** Overall finding: these repos are already unusually clean against classic anti-patterns (no bare excepts, no bad singletons, no naive retries) -- plausibly because CI/ruff/mypy plus prior agent-assisted review (e.g. `pr-review-toolkit:silent-failure-hunter`) already catch most of what norma's Java-derived GoF+code-quality pattern set would otherwise be first to find. The one real, generalizable finding is architectural (layering discipline), not a classic GoF/code-quality shape.
