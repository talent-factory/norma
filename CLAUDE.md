# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

norma is a Model Context Protocol (MCP) server, written in Rust, for validating and enforcing design patterns and coding standards via [ast-grep](https://ast-grep.github.io/). It ships both as an MCP tool server (`norma serve`) and a CLI (`norma validate`, `norma list-patterns`, `norma import`), sharing one async core. See `README.md` for the user-facing feature set and `DEVELOPMENT.md` for current status.

## Commands

```bash
just                        # list all recipes, grouped
just check                  # fmt-check + clippy + test -- run before committing
just watch                  # rebuild + retest on every change

cargo test                  # full suite (unit tests colocated in src/*.rs + tests/)
cargo test --lib            # unit tests only
cargo test <test_name>      # a single test by name (substring match)
cargo test --test dogfooding -- --nocapture     # norma validates its own src/
cargo test --test gof_patterns -- --nocapture   # behavioral coverage for the GoF default patterns

cargo fmt --all -- --check
cargo clippy --all-targets

cargo run -- serve                                  # MCP server on stdio
cargo run -- validate --language rust src/main.rs   # CLI validate (add --fix, --json as needed)
cargo run -- list-patterns
cargo run -- import path/to/rule-dir --category foo
```

## Architecture

**Single binary, shared core** (`docs/adr/0001-single-binary-shared-validation-core.md`): one crate (`src/lib.rs`), one `clap`-subcommand binary (`src/main.rs`). `norma serve` (long-running MCP stdio loop) and `norma validate`/`list-patterns`/`import` (one-shot CLI) both call into the same async `pattern_engine`/`pattern_store` core — no duplicate validation logic. Deliberately not a multi-crate workspace, to keep the code legible for the FFHS teaching audience this project is also built for.

Module map:
- `mcp_server.rs` — the 7 MCP tools (`#[tool_router]`/`#[tool]` via `rmcp`): `validate_pattern_compliance`, `apply_pattern_fix`, `get_pattern_checklist`, `test_pattern`, `register_pattern`, `import_rules`, `list_patterns`. Error convention: `invalid_params` for client-caused failures (bad YAML, unsupported language), `internal_error` for storage faults — never conflate the two, an MCP client needs to know which is retryable.
- `cli.rs` — `clap` `Cli`/`Command` enum, plus the CLI-only logic MCP doesn't need (file I/O for `validate`/`--fix`/`import`, human-readable rendering, DB path resolution).
- `pattern_engine.rs` — the actual AST matching: parses a `Pattern`'s `rule` YAML into a real `ast_grep_config::RuleConfig`, finds violations, applies fixes, resolves/validates language strings.
- `pattern_store.rs` — SQLite persistence (`sqlx`), default-pattern seeding, bulk import.
- `models.rs` — `Pattern`, `PatternViolation`, `ValidationResult`, `FixResult`, `Severity`, etc.
- `default_patterns.rs` — the 20 shipped patterns (data, not logic).

**Pattern model** (`docs/adr/0002-pattern-single-language-full-rule-config.md`): one `Pattern` row is always single-language — an idea spanning several languages (e.g. "no debug prints") becomes several rows sharing a `name`. `rule` stores the **complete** ast-grep `RuleConfig` YAML document verbatim (`id`/`message`/`severity`/`language`/`rule`/optionally `fix`), copy-pasteable straight from ast-grep's own docs or CLI output. `id`, `language`, and `severity` are never accepted as separate inputs — `Pattern::from_rule` (the only constructor) always derives them by parsing `rule`, so they can't drift from what the YAML actually says.

`ast-grep-core`/`-config`/`-language` are pinned to an **exact** version (`=0.45.3`) — the Rust API is explicitly documented upstream as "not stable yet". Bump all three together, deliberately, never via a caret range.

**Language support**: all 28 languages `ast-grep-language` ships are registrable (`pattern_engine::language_key`/`SUPPORTED_LANGUAGES` derive from `SupportLang::all_langs()`, not a hardcoded list) — an unsupported/misspelled `--language` is always a loud error, never a silent "0 violations" pass. Default patterns currently ship only for `java`/`python`/`rust`/`typescript`; the other 24 are registrable but start with no patterns (`get_pattern_checklist` surfaces this via a `coverage_warning` field, distinct from "compliant").

**DB location** resolution order: `--db` flag > `$NORMA_DB` > `$XDG_DATA_HOME/norma/norma.db` > `$HOME/.local/share/norma/norma.db`.

**Logging**: `tracing` output goes to stderr only, never stdout — `norma serve` speaks JSON-RPC over stdio, so anything on stdout corrupts the protocol stream.

**norma vs. `ast-grep-mcp`**: ast-grep ships its own experimental MCP server; norma is deliberately complementary, not a superset (see README's "norma vs. ast-grep's own `ast-grep-mcp`" section). `ast-grep-mcp` is ephemeral search/debug tooling; norma is a persistent compliance gate. This boundary was actively enforced when scoping new features — e.g. an ad-hoc/ephemeral search tool and a `dump_syntax_tree` equivalent were both explicitly rejected as redundant with `ast-grep-mcp` (see `.scratch/ast-grep-feature-parity/`).

## Planning history

Nontrivial features and architecture decisions here go through a "wayfinder" map/ticket process before implementation: a map file plus one file per decision, each recording the question and its resolved answer. Two efforts exist under `.scratch/`: `norma-architecture/` (original MVP scope) and `ast-grep-feature-parity/` (autofix, rule-testing, language coverage, bulk import). Read a map's `Decisions so far` section for the reasoning behind a given design choice before assuming it was arbitrary. Larger decisions also get a proper ADR under `docs/adr/`.
