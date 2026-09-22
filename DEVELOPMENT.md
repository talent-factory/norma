# norma Development Guide

This document outlines the current state of the norma project and the next steps for development.

## 📊 Current Status

### ✅ Completed (MVP -- see `docs/superpowers/plans/2026-09-20-norma-mvp-implementation.md`)

- [x] MCP server on `rmcp`, the official Rust MCP SDK -- replaces the original `mcpkit` scaffold, which never built (see docs/adr/0001.md and `.scratch/norma-architecture/issues/01-mcpkit-vs-alternative.md`)
- [x] Real AST-based pattern matching via `ast-grep-core` (`ast-grep-core`/`-config`/`-language`, version-pinned exactly to `0.45.3` since the Rust API is "not stable yet") -- no regex matching remains
- [x] Single binary, `clap` subcommands (`norma serve`, `norma validate --language [--json] [--fix] <files...>`, `norma list-patterns`, `norma import <dir> [--category]`) sharing one async core with the MCP server -- `src/cli.rs`, `src/main.rs`, per docs/adr/0001.md
- [x] SQLite-backed `PatternStore` following the ADR 0002 schema: one row per language (no join table), `rule` stores the full ast-grep `RuleConfig` YAML verbatim, `language`/`severity` always derived by parsing that YAML rather than accepted as separate fields, fail-fast validation in `register_pattern` (nothing is written if the YAML doesn't parse) -- `src/pattern_store.rs`, docs/adr/0002.md
- [x] Four default patterns, one per MVP language (Java, Python, Rust, TypeScript), all expressing the same "no debug print in production code" idea under category `code-quality` -- `src/default_patterns.rs`
- [x] Dogfooding integration test: the Rust "no debug print" default pattern run against norma's own `src/`, skipping `main.rs` (its `println!` calls are legitimate CLI output, not a debug leftover) -- `tests/dogfooding.rs`
- [x] Pre-commit hook template (`.pre-commit-config.yaml`), `language: system` so pre-commit calls the already-installed `norma` binary instead of compiling Rust on every run
- [x] README and DEVELOPMENT docs brought in line with the above
- [x] Four GoF default patterns (Singleton, Factory, Observer, Strategy), one per MVP language (16 new GoF patterns, 20 default patterns in total) -- `singleton-quality-*`/`factory-overuse-*` under category `creational`, `observer-presence-*`/`strategy-overuse-*` under category `behavioral`; `observer-presence-*` is `info`-severity and (per `pattern_engine::validate`'s severity-aware scoring) visible without failing a run -- `src/default_patterns.rs`, `docs/superpowers/specs/2026-09-20-gof-pattern-set-v2-design.md`
- [x] Per-id pattern seeding: `PatternStore::seed_defaults` inserts every default pattern whose id isn't already a row, instead of gating on "store is completely empty" -- an existing installation (e.g. an MVP-era database with only the four `no-debug-print` patterns) now picks up newly added default patterns like the GoF v2 set on the next run, with no manual steps, while never touching a row that already exists (so a user's own edits are never silently reset) -- `src/pattern_store.rs`'s `seed_defaults`
- [x] Autofix/rewrite support (TF-890, see `.scratch/ast-grep-feature-parity/issues/01-autofix-rewrite-support.md`) -- a `suggested_fix` field on `PatternViolation`, populated from the matched pattern's own `fix`/`fixer` (`pattern_engine::find_violations`); a new `apply_pattern_fix` MCP tool that rewrites `code` and returns the result without touching disk, deliberately kept separate from `validate_pattern_compliance` rather than an `apply_fixes` flag on it; and `norma validate --fix`, which writes each file in-place with every non-conflicting fix applied (like `eslint --fix`), then re-validates and reports what's left. Overlapping fix ranges from different patterns are fail-safe: neither is applied, a `[warning] fix conflict` line is printed instead of guessing a winner -- `src/models.rs`, `src/pattern_engine.rs`, `src/mcp_server.rs`, `src/cli.rs`
- [x] `test_pattern` MCP tool (TF-891, see `.scratch/ast-grep-feature-parity/issues/02-rule-testing-ast-debug-tooling.md`) -- dry-runs a candidate ast-grep `RuleConfig` YAML against example code without registering it, applying the same `id`/language checks `register_pattern` does (so acceptance/rejection there matches what `register_pattern` would do) and reporting `suggested_fix` on a match. Deliberately no `dump_syntax_tree` equivalent -- that would be a value-free clone of `ast-grep-mcp`'s own tool, not a norma-specific need -- `src/pattern_engine.rs`, `src/mcp_server.rs`
- [x] All 28 `SupportLang` languages generically registrable, not just the MVP four (TF-893) -- `pattern_engine::language_key`/`SUPPORTED_LANGUAGES` now derive from `SupportLang::all_langs()` instead of a hardcoded allowlist; `get_pattern_checklist` distinguishes "no coverage for this language" from "compliant" via a `coverage_warning` field so an empty patterns list for one of the 24 newly-unlocked, default-pattern-less languages isn't mistaken for the latter -- `src/pattern_engine.rs`, `src/mcp_server.rs`
- [x] Bulk-import of an existing ast-grep rule set (TF-894, see `.scratch/ast-grep-feature-parity/issues/04-bulk-import-existing-rules.md`) -- `pattern_engine::parse_rules` (multi-document `from_yaml_string`, `parse_rule` is now this with only the first document kept), `PatternStore::import_rules` (per-document `name`/`description` derived from the rule's own raw `id`/`message`, skip-with-reason for a document that fails to register rather than aborting the batch, an empty or not-a-rule document -- no `id:`/`rule:` at all -- silently excluded from both lists, 1:1 upsert semantics with `register_pattern` including within one call, a storage failure logged with its partial progress before aborting), the `import_rules` MCP tool, and the `norma import <dir>` CLI subcommand (recursive `.yml`/`.yaml` file discovery skipping dot-directories, one `import_rules` call per file so a syntax error in one file can't abort the rest, `norma import`'s exit status non-zero on any skip) -- `src/pattern_engine.rs`, `src/pattern_store.rs`, `src/mcp_server.rs`, `src/cli.rs`. Hardened after a multi-agent PR review caught a real bug: concatenating raw file bytes with a synthetic `---` separator broke on rule files that already start with their own `---` marker (the common case for a cloned rule catalog) -- fixed by dropping the one-big-concatenated-string approach for the per-file-call design above.
- [x] Automatic `CHANGELOG.md` generation -- [git-cliff](https://github.com/orhun/git-cliff) (`cliff.toml`) builds it from the full commit history, grouped by commit type (`feat`/`fix`/`docs`/`wayfinder`/...), with `(TF-xxx)` references linked to Linear; not restricted to strict Conventional Commits since this repo's early history predates that convention, so everything else falls into a catch-all group rather than being silently dropped. `.github/workflows/changelog.yml` regenerates and commits it on every push to `main` *or* `develop` (i.e. every merge into either) -- both, not just `main`, so `develop`'s copy never drifts stale between releases -- guarded against re-triggering itself via `paths-ignore: CHANGELOG.md` on the workflow trigger rather than an in-job check. Since `main`/`develop` both require changes via PR (0 approvals, but no direct push), the workflow opens and immediately self-merges a single-file PR rather than pushing directly; this needed removing the (redundant, given 0-approval PRs already gate everything) "Restrict who can push" branch-protection rule, which previously blocked even that PR-merge for any actor other than the repo owner.

### 🔄 Next Steps

1. **Test strategy** (Priority: Medium)
   - The wayfinder map (`.scratch/norma-architecture/map.md`, section "Not yet specified") deliberately left the overall unit/integration/E2E test strategy unresolved for the spec phase -- it didn't block implementation, and each task supplied its own tests as it went (unit tests next to `pattern_engine`, `pattern_store`, `cli`, `default_patterns`, plus the `tests/dogfooding.rs` integration test). Revisit explicitly if a real gap shows up, e.g. dedicated end-to-end MCP-protocol coverage.

2. **Performance optimization** (Priority: Low)
   - Benchmark pattern matching, cache compiled ast-grep rules, tune the SQLite connection pool.
   - Location: `src/pattern_engine.rs`, `src/pattern_store.rs`

## 🛠️ Development Environment

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
rustup update stable
cargo install cargo-edit cargo-watch
```

### Useful Commands

```bash
# Watch mode (rebuilds on file changes)
cargo watch -x build -x test

# Run with tracing output
RUST_LOG=debug ./target/debug/norma

# Format code
cargo fmt

# Run linter
cargo clippy

# Build release binary
cargo build --release
```

## 📝 Pattern Definition Examples

See `src/default_patterns.rs` for the 20 patterns norma ships with (the
four MVP "no debug print" patterns, one per language, plus 16 GoF
patterns -- Singleton, Factory, Observer, Strategy, one per MVP language
each), and the README's
[Pattern Definition Format](README.md#pattern-definition-format) section
for the shape a `rule` YAML document needs. There is no `Pattern::new`
constructor -- `Pattern::from_rule` (`src/pattern_engine.rs`) is the only
way to build one, and it derives `id`/`language`/`severity` from the YAML
rather than accepting them separately (see docs/adr/0002.md).

Here is the real, shipped Java Singleton pattern (`singleton-quality-java`)
-- a `kind`/`has` rule rather than a plain string pattern, since it needs
to express a structural relationship (a private static instance field
*inside* the class, and either no explicit constructor at all or one that
is not private), not just a code shape:

```yaml
id: singleton-quality-java
message: Class looks like a Singleton (private static instance field) but its constructor is not private
severity: warning
language: Java
rule:
  kind: class_declaration
  all:
    - has:
        stopBy: end
        kind: field_declaration
        pattern:
          context: 'class C { private static $TYPE instance; }'
          selector: field_declaration
    - any:
        - not:
            has:
              stopBy: end
              kind: constructor_declaration
        - has:
            stopBy: end
            kind: constructor_declaration
            not:
              has:
                kind: modifiers
                regex: private
```

## 🔍 Testing Checklist

- [ ] Pattern store creates SQLite database
- [ ] Default patterns load on first run
- [ ] Pattern validation runs without crashing
- [ ] MCP tools respond correctly
- [ ] Multi-language support works
- [ ] Violations are reported with correct locations
- [ ] Pattern registration persists to database
- [ ] Pre-commit hook integration works

## 📚 Resources

- [MCP Specification](https://modelcontextprotocol.io/)
- [ast-grep Documentation](https://ast-grep.github.io/)
- [SQLx for Rust](https://github.com/launchbadge/sqlx)
- [Tree-sitter](https://tree-sitter.github.io/)

## 🚀 Deployment Considerations

- [ ] Binary size optimization
- [ ] Memory usage profiling
- [ ] Database migration strategy
- [ ] Version management
- [ ] Update mechanism

## 📞 Questions?

When working in Claude Code, reference:
- `src/pattern_engine.rs` for pattern matching logic
- `src/pattern_store.rs` for database operations
- `src/mcp_server.rs` for tool definitions
- `Cargo.toml` for dependency management

---

**Last updated:** 2026-09-22
**Maintainer:** Talent Factory GmbH
