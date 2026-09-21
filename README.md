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

## 🆚 norma vs. ast-grep's own `ast-grep-mcp`

ast-grep ships its own experimental MCP server, [`ast-grep-mcp`](https://github.com/ast-grep/ast-grep-mcp). It's worth knowing about, because it answers a different question than norma does:

| | [`ast-grep-mcp`](https://github.com/ast-grep/ast-grep-mcp) | norma |
|---|---|---|
| Answers | "Where does X occur in this code, and how do I write a rule for it?" | "Does this code violate one of our team's standing rules?" |
| Rules | ephemeral — built by the AI agent per call, never stored | persistent — registered once via `register_pattern`, enforced on every later call |
| Implementation | Python, shells out to the `ast-grep` CLI as a subprocess | Rust, links `ast-grep-core`/`-config`/`-language` directly as a library (no subprocess) |
| Tools | `dump_syntax_tree`, `test_match_code_rule`, `find_code`, `find_code_by_rule` — a search/debug workflow | `validate_pattern_compliance`, `apply_pattern_fix`, `get_pattern_checklist`, `test_pattern`, `register_pattern`, `list_patterns` — a compliance workflow |
| State | none (SQLite-free) | SQLite-backed `PatternStore`, survives restarts and project switches |
| Status | explicitly experimental | in production use here, with tests and ADRs (`docs/adr/`) |

In short: `ast-grep-mcp` is `ast-grep --pattern`, reachable over MCP for interactive exploration. norma is closer to `eslint`/`checkstyle` with a fixed, versioned rule set — using ast-grep as its match engine instead of a hand-rolled one. The two are complementary, not competing: use `ast-grep-mcp` (or the plain `ast-grep` CLI) to *discover and iterate on* a rule, then register the finished rule in norma to *enforce* it from then on — see [Adopting an existing ast-grep rule](#adopting-an-existing-ast-grep-rule) below.

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

Rewrite files in-place with every non-conflicting pattern fix applied (like
`eslint --fix`/`biome --fix`), then report what's left:

```bash
norma validate --language rust --fix src/main.rs
```

There's no dry-run mode in v1 -- keeping a clean git working tree beforehand
so you can review or revert the rewrite is on you, not enforced by norma.
If two patterns' fixes overlap on the same code, *neither* is applied and a
`[warning] fix conflict` line is printed instead (fail-safe over guessing a
winner).

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

### Adopting an existing ast-grep rule

Because `rule` stores ast-grep's `RuleConfig` YAML verbatim (see [ADR 0002](docs/adr/0002-pattern-single-language-full-rule-config.md)), a rule documented in [ast-grep's own catalog](https://ast-grep.github.io/catalog/), produced by `ast-grep-mcp`'s `test_match_code_rule`, or copied from `ast-grep --pattern` CLI output needs no reshaping to become a norma `Pattern` -- only its `language:` value may need to change to whichever of norma's four supported languages it belongs to (`Java` | `Python` | `Rust` | `TypeScript`).

Take this rule, unmodified from ast-grep's catalog:

```yaml
id: no-await-in-promise-all
severity: error
language: JavaScript
message: No await in Promise.all
rule:
  pattern: await $A
  inside:
    pattern: Promise.all($_)
    stopBy:
      not: { any: [{ kind: array }, { kind: arguments }] }
fix: $A
```

Swap `language: JavaScript` for `language: TypeScript` (norma's four canonical keys are `java`/`python`/`rust`/`typescript`; `TypeScript`'s grammar is a superset of the plain-JS pattern here) and pass the whole document through `register_pattern` unchanged otherwise:

```jsonc
register_pattern(
  name: "No await inside Promise.all",
  description: "Promise.all already awaits each element; awaiting inside the array is redundant and usually a sign the loop was meant to run in parallel.",
  category: "code-quality",
  rule: "id: no-await-in-promise-all\nseverity: error\nlanguage: TypeScript\nmessage: No await in Promise.all\nrule:\n  pattern: await $A\n  inside:\n    pattern: Promise.all($_)\n    stopBy:\n      not: { any: [{ kind: array }, { kind: arguments }] }\nfix: $A\n"
)
```

`register_pattern` only guarantees the YAML *parses* -- not that it matches what you intend (`RegisterPatternError::InvalidRule` rejects malformed YAML before it ever reaches storage, per `src/mcp_server.rs`). Sanity-check the rule against a snippet *before* registering it with the `test_pattern` MCP tool -- it runs a rule against example code and reports matches (with `suggested_fix`, if the rule has a `fix:`) without storing anything:

```jsonc
test_pattern(
  rule: "id: no-await-in-promise-all\nseverity: error\nlanguage: TypeScript\nmessage: No await in Promise.all\nrule:\n  pattern: await $A\n  inside:\n    pattern: Promise.all($_)\n    stopBy:\n      not: { any: [{ kind: array }, { kind: arguments }] }\nfix: $A\n",
  code: "await Promise.all([await doA(), doB()])"
)
```

`test_pattern` applies the same `id`/`language` checks `register_pattern` does, so a rule it accepts is guaranteed to also be accepted by `register_pattern`. It has no `dump_syntax_tree` equivalent -- for raw AST inspection, use ast-grep-mcp's `dump_syntax_tree` (or the plain `ast-grep` CLI) instead; norma stays complementary to it rather than duplicating it.

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

## 🎓 For Students

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
- **Maintainer** — Daniel (daniel.senften@talent-factory.ch)

---

**Built with ❤️ for teaching AI-assisted software engineering**
