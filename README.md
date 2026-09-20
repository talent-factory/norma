# norma — Developer-grade code pattern enforcement

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/talent-factory/norma#license)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**norma** is a Model Context Protocol (MCP) server for validating and enforcing design patterns and coding standards. Built in Rust, it integrates seamlessly with Claude Code and other AI-assisted development tools.

## 🎯 Features

- ✅ **Pattern Validation** — Check code against registered design patterns
- ✅ **Multi-Language Support** — Java, Python, Rust and TypeScript (an
  unsupported `--language` is rejected, never silently skipped)
- ✅ **MCP Integration** — Works with Claude Code, Cursor, and other MCP clients
- ✅ **Persistent Storage** — SQLite-backed pattern registry
- ✅ **Real-Time Feedback** — Instant violation detection with suggestions
- ✅ **Custom Patterns** — Define team-specific coding standards
- ✅ **Educational Focus** — Perfect for teaching design patterns

## Usage

Build and install once:

```bash
cargo build --release
cargo install --path .
```

Run the MCP tool server (for Claude Code / MCP Inspector):

```bash
norma serve
```

Validate one or more files from the command line (files are positional,
so shell globs and `pre-commit`'s staged-file list both work):

```bash
norma validate --language rust src/main.rs
norma validate --language rust --json src/*.rs
```

Exit status is non-zero if *any* file has violations.

List every registered pattern:

```bash
norma list-patterns
```

### Where the pattern database lives

norma stores its pattern registry in SQLite at a fixed per-user location
(`$HOME/.local/share/norma/norma.db`) so the same registry is used no
matter which directory norma is launched from. Override it per invocation
with `--db <PATH>` (a global flag, valid on every subcommand) or globally
with the `NORMA_DB` environment variable:

```bash
norma --db ./team-patterns.db list-patterns
NORMA_DB=/srv/norma/patterns.db norma serve
```

Enable the pre-commit hook (see `.pre-commit-config.yaml` -- requires
`norma` already installed via `cargo install --path .`):

```bash
pre-commit install
```

## 🏗️ Architecture

```
norma/
├── src/
│   ├── main.rs              # Entry point & transport setup
│   ├── lib.rs               # Library exports
│   ├── models.rs            # Data structures (Pattern, Violation, etc.)
│   ├── mcp_server.rs        # MCP tool definitions & handlers
│   ├── pattern_engine.rs    # Pattern matching & validation logic
│   └── pattern_store.rs     # SQLite persistence layer
├── Cargo.toml               # Rust dependencies
└── README.md
```

## 📋 Pattern Definition Format

Patterns are stored as JSON-serialized `Pattern` structs with:

```json
{
  "id": "java-singleton",
  "name": "Singleton Pattern",
  "description": "Ensure proper Singleton implementation",
  "rule": "regex pattern",
  "rewrite": "suggested fix",
  "severity": "warning",
  "languages": ["java"],
  "enabled": true,
  "created_at": "2025-09-19T...",
  "updated_at": "2025-09-19T..."
}
```

## 🔧 Development

### Running Tests

```bash
cargo test
```

### Building Documentation

```bash
cargo doc --open
```

### Code Style

This project follows Rust conventions enforced by:
- `rustfmt` — code formatting
- `clippy` — linting

Check formatting:
```bash
cargo fmt --check
cargo clippy
```

## 🎓 For FFHS Students

norma is designed to help you:

1. **Learn Design Patterns** — See real violations and fixes
2. **Enforce Team Standards** — Define patterns for your projects
3. **Understand AST-Based Analysis** — See how pattern matching works
4. **Integrate with Claude Code** — Use AI to help you follow patterns

### Example: Java Factory Pattern

Define a pattern to check that object creation uses factories:

```rust
Pattern::new(
    "java-factory-pattern".to_string(),
    "Require Factory pattern for object creation".to_string(),
    r"new\s+\w+\(".to_string(),  // Simplified; use AST in production
    vec!["java".to_string()],
)
```

Then validate:

```
validate_pattern_compliance(
    code: "MyObject obj = factory.create();",  // ✓ PASS
    language: "java"
)
```

## 📝 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit changes (`git commit -am 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing`)
5. Open a Pull Request

## 📞 Contact

- **Talent Factory GmbH** — https://github.com/talent-factory
- **Maintainer** — Daniel (daniel@talentfactory.ch)

---

**Built with ❤️ for teaching AI-assisted software engineering at FFHS**
