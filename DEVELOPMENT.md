# norma Development Guide

This document outlines the current state of the norma project and the next steps for development.

## 📊 Current Status

### ✅ Completed

- [x] Project scaffolding and structure
- [x] Core data models (Pattern, Violation, ValidationResult)
- [x] MCP server definition with mcpkit
- [x] Pattern storage with SQLite
- [x] Basic pattern engine (regex-based)
- [x] Default patterns (Java, TypeScript)
- [x] README and documentation
- [x] License setup (MIT/Apache-2.0)

### 🔄 In Progress / Next Steps

1. **AST-Grep Integration** (Priority: High)
   - Replace regex-based matching with real AST analysis
   - Integrate ast-grep-core for multi-language support
   - Create AST patterns for Java factory pattern detection
   - Location: `src/pattern_engine.rs`

2. **Pre-Commit Hook Integration** (Priority: High)
   - Add CLI validation subcommand
   - Create `.pre-commit-config.yaml` template
   - Test with real Git workflow
   - Location: `src/main.rs`

3. **Test Suite Expansion** (Priority: Medium)
   - Unit tests for pattern matching
   - Integration tests for SQLite operations
   - E2E tests for MCP protocol
   - Location: `tests/` directory

4. **Documentation** (Priority: Medium)
   - API documentation (cargo doc)
   - Tutorial: "Your First Pattern"
   - Integration guide for Claude Code
   - Location: various

5. **Performance Optimization** (Priority: Low)
   - Benchmark pattern matching
   - Cache compiled patterns
   - Connection pooling tuning
   - Location: `src/pattern_engine.rs`, `src/pattern_store.rs`

## 🎯 Quick Tasks for Claude Code

### Immediate (Session 1-2)

- [ ] Build and test the project
  ```bash
  cargo build
  cargo test
  ```

- [ ] Implement AST-grep integration in `pattern_engine.rs`
  - Use `ast-grep-core` crate for real pattern matching
  - Test with a Java factory pattern example

- [ ] Add more default patterns
  - Singleton pattern (Java)
  - Observer pattern (TypeScript)
  - Strategy pattern (Python)

### Short-term (Session 3-4)

- [ ] Implement CLI for validation
  - `norma validate --file src/Main.java --lang java`
  - `norma check-patterns`

- [ ] Add pre-commit hook support
  - Generate `.pre-commit-config.yaml`
  - Test integration with real Git repo

- [ ] Improve error messages
  - Better violation descriptions
  - Actionable suggestions

### Medium-term (Week 2+)

- [ ] Build web UI for pattern management (optional)
- [ ] Create pattern marketplace (optional)
- [ ] Publish crate to crates.io (optional)

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

### Java Singleton

```rust
Pattern::new(
    "java-singleton".to_string(),
    "Enforce proper Singleton pattern implementation".to_string(),
    r"class\s+\w+\s*\{[^}]*private\s+static\s+\w+\s+instance[^}]*public\s+static\s+synchronized".to_string(),
    vec!["java".to_string()],
)
```

### TypeScript Factory

```rust
Pattern::new(
    "ts-factory-pattern".to_string(),
    "Use Factory pattern for object creation".to_string(),
    r"new\s+\w+\(".to_string(),  // Simplified; improve with AST
    vec!["typescript".to_string(), "javascript".to_string()],
)
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

**Last updated:** 2025-09-19
**Maintainer:** Talent Factory GmbH
