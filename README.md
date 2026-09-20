# norma — Developer-grade code pattern enforcement

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/talent-factory/norma#license)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

**norma** is a Model Context Protocol (MCP) server for validating and enforcing design patterns and coding standards. Built in Rust, it integrates seamlessly with Claude Code and other AI-assisted development tools.

## 🎯 Features

- ✅ **Pattern Validation** — Check code against registered design patterns
- ✅ **Multi-Language Support** — Java, Python, Rust and TypeScript (an
  unsupported `--language` is rejected, never silently skipped)
- ✅ **MCP Integration** — Works with Claude Code, Cursor, and other MCP clients
- ✅ **Persistent Storage** — SQLite-backed pattern registry
- ✅ **Real-Time Feedback** — Instant violation detection with file:line:column locations
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
(`$XDG_DATA_HOME/norma/norma.db`, or `$HOME/.local/share/norma/norma.db`
if `$XDG_DATA_HOME` isn't set -- the XDG Base Directory spec's own
documented default) so the same registry is used no matter which
directory norma is launched from. Override it per invocation with
`--db <PATH>` (a global flag, valid on every subcommand) or globally with
the `NORMA_DB` environment variable:

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
│   ├── main.rs              # Entry point: wires the CLI to the shared core
│   ├── lib.rs               # Library exports
│   ├── cli.rs                # clap Cli/Command, validate_files, resolve_db_path
│   ├── models.rs            # Data structures (Pattern, PatternViolation, etc.)
│   ├── mcp_server.rs        # MCP tool definitions & handlers
│   ├── pattern_engine.rs    # Pattern matching & validation logic
│   ├── pattern_store.rs     # SQLite persistence layer
│   └── default_patterns.rs # 20 default patterns: 4 MVP "no debug print" + 16 GoF
├── tests/
│   ├── dogfooding.rs        # norma validates its own src/ with its own Rust pattern
│   └── gof_patterns.rs      # Behavioral tests for the GoF v2 pattern set
├── docs/adr/                # Architecture decision records
├── Cargo.toml               # Rust dependencies
└── README.md
```

## 📋 Pattern Definition Format

A `Pattern` is single-language (see [ADR 0002](docs/adr/0002-pattern-single-language-full-rule-config.md)): an idea that should hold across several languages -- like "no debug prints" -- becomes several `Pattern` rows, one per language, linked only by a shared `name`. `id`, `language`, and `severity` are never set directly; they're derived from `rule`, which holds the *complete* ast-grep `RuleConfig` YAML document:

```json
{
  "id": "no-debug-print-java",
  "name": "No Debug Print",
  "description": "System.out.println left in production code should go through a proper logger instead.",
  "category": "code-quality",
  "language": "java",
  "severity": "warning",
  "rule": "id: no-debug-print-java\nmessage: Avoid System.out.println in production code\nseverity: warning\nlanguage: Java\nrule:\n  pattern: System.out.println($$$ARGS)\n",
  "enabled": true,
  "created_at": "2026-09-20T...",
  "updated_at": "2026-09-20T..."
}
```

Register one via the `register_pattern` MCP tool, or in Rust via `PatternStore::register_pattern(name, description, category, rule_yaml)` -- see `src/default_patterns.rs` for the 20 patterns norma ships with.

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

### Example: catching `System.out.println` in Java

This is one of the 20 patterns norma ships with (`src/default_patterns.rs`),
which now also includes four Gang-of-Four patterns (Singleton, Factory,
Observer, Strategy) per language -- see DEVELOPMENT.md's
[Pattern Definition Examples](DEVELOPMENT.md#pattern-definition-examples)
section for the Java Singleton example. The rule shown here is a full
ast-grep `RuleConfig` YAML document:

```yaml
id: no-debug-print-java
message: Avoid System.out.println in production code
severity: warning
language: Java
rule:
  pattern: System.out.println($$$ARGS)
```

Then validate:

```
validate_pattern_compliance(
    code: "System.out.println(\"debug\");",  // ✗ flagged
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
