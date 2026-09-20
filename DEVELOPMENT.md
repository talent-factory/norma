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

### 🔄 Next Steps

1. **Test strategy** (Priority: Medium)
   - The wayfinder map (`.scratch/norma-architecture/map.md`, section "Not yet specified") deliberately left the overall unit/integration/E2E test strategy unresolved for the spec phase -- it didn't block implementation, and each task supplied its own tests as it went (unit tests next to `pattern_engine`, `pattern_store`, `cli`, `default_patterns`, plus the `tests/dogfooding.rs` integration test). Revisit explicitly if a real gap shows up, e.g. dedicated end-to-end MCP-protocol coverage.

2. **v2 pattern set, including GoF patterns** (Priority: Medium)
   - The MVP deliberately shipped one pattern per language and deferred classic Gang-of-Four checks (Singleton, Factory, Observer, Strategy, ...) -- see the "MVP-Pattern-Set-Scope" ticket (`.scratch/norma-architecture/issues/05-mvp-pattern-set-scope.md`). A v2 set should add these, each verified against real `ast-grep-core` the same way the v1 defaults were.
   - Location: `src/default_patterns.rs`

3. **Performance optimization** (Priority: Low)
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

See `src/default_patterns.rs` for the four patterns norma ships with (one
per MVP language, all expressing "no debug print"), and the README's
[Pattern Definition Format](README.md#pattern-definition-format) section
for the shape a `rule` YAML document needs. There is no `Pattern::new`
constructor -- `Pattern::from_rule` (`src/pattern_engine.rs`) is the only
way to build one, and it derives `id`/`language`/`severity` from the YAML
rather than accepting them separately (see docs/adr/0002.md).

A v2 GoF pattern (see "Next Steps" above) would look like this for Java
Singleton -- a `kind`/`has` rule rather than a plain string pattern, since
it needs to express a structural relationship (a private static field
*inside* the class), not just a code shape:

```yaml
id: java-singleton
message: Enforce proper Singleton pattern implementation
severity: warning
language: Java
rule:
  kind: class_declaration
  has:
    kind: field_declaration
    # ... the actual field/method shape is still to be worked out
    # against real ast-grep-core, the same way the v1 defaults were.
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
