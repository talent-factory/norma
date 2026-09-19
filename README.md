# norma — Developer-grade code pattern enforcement

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/talent-factory/norma#license)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**norma** is a Model Context Protocol (MCP) server for validating and enforcing design patterns and coding standards. Built in Rust, it integrates seamlessly with Claude Code and other AI-assisted development tools.

## 🎯 Features

- ✅ **Pattern Validation** — Check code against registered design patterns
- ✅ **Multi-Language Support** — Java, TypeScript, Python, Go, and more
- ✅ **MCP Integration** — Works with Claude Code, Cursor, and other MCP clients
- ✅ **Persistent Storage** — SQLite-backed pattern registry
- ✅ **Real-Time Feedback** — Instant violation detection with suggestions
- ✅ **Custom Patterns** — Define team-specific coding standards
- ✅ **Educational Focus** — Perfect for teaching design patterns

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- SQLite 3.0+

### Installation

```bash
git clone https://github.com/talent-factory/norma.git
cd norma
cargo build --release
```

The binary will be at `target/release/norma`.

### Running the MCP Server

```bash
./target/release/norma
```

The server listens on stdin/stdout and is ready for MCP client connections.

### Using with Claude Code

1. Add to `.claude/mcp.json`:

```json
{
  "mcpServers": {
    "norma": {
      "command": "/path/to/norma"
    }
  }
}
```

2. Restart Claude Code
3. Use the norma tools in your prompts:

```
Validate this Java code against our design patterns
```

## 📚 Usage

### Validate Code

```bash
# In Claude Code or via MCP client
tool: validate_pattern_compliance
code: "public class Singleton { ... }"
language: "java"
```

**Response:**
```json
{
  "violations": [
    {
      "pattern_name": "Singleton Pattern",
      "severity": "warning",
      "location": { "file": "code", "line": 1, "column": 0 },
      "message": "Missing synchronized keyword on getInstance()",
      "suggestion": "Add synchronized to getInstance() method"
    }
  ],
  "passed": false,
  "score": 0.7,
  "duration_ms": 42
}
```

### Get Pattern Checklist

```bash
tool: get_pattern_checklist
language: "java"
```

**Response:**
```json
[
  {
    "id": "java-singleton",
    "name": "Singleton Pattern",
    "description": "Ensure proper Singleton implementation",
    "severity": "warning",
    "enabled": true
  },
  {
    "id": "java-factory",
    "name": "Factory Pattern",
    "description": "Use Factory pattern for object creation",
    "severity": "info",
    "enabled": true
  }
]
```

### Register Custom Pattern

```bash
tool: register_pattern
name: "My Custom Pattern"
description: "Check for X in code"
rule: "regex pattern or AST-grep rule"
languages: ["java", "typescript"]
severity: "warning"
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
