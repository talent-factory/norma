# norma Development Guide

This document outlines the current state of the norma project and the next steps for development.

## 📊 Current Status

### ✅ Completed (MVP -- see `docs/superpowers/plans/2026-09-20-norma-mvp-implementation.md`)

- [x] MCP server on `rmcp`, the official Rust MCP SDK -- replaces the original `mcpkit` scaffold, which never built (see docs/adr/0001.md and `.scratch/norma-architecture/issues/01-mcpkit-vs-alternative.md`)
- [x] Real AST-based pattern matching via `ast-grep-core` (`ast-grep-core`/`-config`/`-language`, version-pinned exactly to `0.45.3` since the Rust API is "not stable yet") -- no regex matching remains
- [x] Single binary, `clap` subcommands (`norma serve`, `norma validate --file --language [--json]`, `norma list-patterns`) sharing one async core with the MCP server -- `src/cli.rs`, `src/main.rs`, per docs/adr/0001.md
- [x] SQLite-backed `PatternStore` following the ADR 0002 schema: one row per language (no join table), `rule` stores the full ast-grep `RuleConfig` YAML verbatim, `language`/`severity` always derived by parsing that YAML rather than accepted as separate fields, fail-fast validation in `register_pattern` (nothing is written if the YAML doesn't parse) -- `src/pattern_store.rs`, docs/adr/0002.md
- [x] Four default patterns, one per MVP language (Java, Python, Rust, TypeScript), all expressing the same "no debug print in production code" idea under category `code-quality` -- `src/default_patterns.rs`
- [x] Dogfooding integration test: the Rust "no debug print" default pattern run against norma's own `src/`, skipping `main.rs` (its `println!` calls are legitimate CLI output, not a debug leftover) -- `tests/dogfooding.rs`
- [x] Pre-commit hook template (`.pre-commit-config.yaml`), `language: system` so pre-commit calls the already-installed `norma` binary instead of compiling Rust on every run
- [x] README and DEVELOPMENT docs brought in line with the above
- [x] Four GoF default patterns (Singleton, Factory, Observer, Strategy), one per MVP language (16 new GoF patterns, 20 default patterns in total) -- `singleton-quality-*`/`factory-overuse-*` under category `creational`, `observer-presence-*`/`strategy-overuse-*` under category `behavioral`; `observer-presence-*` is `info`-severity and (per `pattern_engine::validate`'s severity-aware scoring) visible without failing a run -- `src/default_patterns.rs`, `docs/superpowers/specs/2026-09-20-gof-pattern-set-v2-design.md`

### ⚠️ Upgrading from the MVP pattern set

Existing installations with a pre-existing `norma.db` will **not**
automatically receive the 16 new GoF patterns: `PatternStore::seed_defaults`
(`src/pattern_store.rs`) only seeds the default pattern set when the store
is completely empty, so a database created under the MVP (four
"no debug print" patterns only) is left as-is on upgrade, silently, with
no warning. To pick up the new patterns, delete the existing database
(default location `$HOME/.local/share/norma/norma.db`) or point `--db`/
`$NORMA_DB` at a fresh path, then re-run norma -- it will reseed all 20
default patterns.

### 🔄 Next Steps

1. **Test strategy** (Priority: Medium)
   - The wayfinder map (`.scratch/norma-architecture/map.md`, section "Not yet specified") deliberately left the overall unit/integration/E2E test strategy unresolved for the spec phase -- it didn't block implementation, and each task supplied its own tests as it went (unit tests next to `pattern_engine`, `pattern_store`, `cli`, `default_patterns`, plus the `tests/dogfooding.rs` integration test). Revisit explicitly if a real gap shows up, e.g. dedicated end-to-end MCP-protocol coverage.

2. **Performance optimization** (Priority: Low)
   - Benchmark pattern matching, cache compiled ast-grep rules, tune the SQLite connection pool.
   - Location: `src/pattern_engine.rs`, `src/pattern_store.rs`

3. **Per-id pattern seeding** (Priority: Medium)
   - `seed_defaults` currently gates on "store is completely empty" (see
     "⚠️ Upgrading from the MVP pattern set" above), so an existing
     installation never receives newly added default patterns. Instead, it
     should insert each `DefaultPattern` whose parsed `id` isn't already a
     row, so future default-pattern additions reach existing installations
     automatically without resetting user customizations.
   - Location: `src/pattern_store.rs`'s `seed_defaults`.

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
*inside* the class, and a constructor that is not private), not just a
code shape:

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

**Last updated:** 2026-09-20
**Maintainer:** Talent Factory GmbH
