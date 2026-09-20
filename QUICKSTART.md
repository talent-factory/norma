# norma Quick Start

Get **norma** running in 5 minutes. For the full reference see
[README.md](README.md); for contributor workflow see
[DEVELOPMENT.md](DEVELOPMENT.md).

## 1. Prerequisites

- Rust (see `rust-toolchain.toml` for the pinned version) — [install rustup](https://rustup.rs/)
- Git

SQLite is bundled with norma, so there is nothing to install for it.

## 2. Build

```bash
cargo build --release
```

Takes a few minutes on the first build.

## 3. Install (optional but recommended)

```bash
cargo install --path .
```

This puts `norma` on your `PATH`. Without it, replace every `norma ...`
below with `cargo run -- ...` (debug build) or
`./target/release/norma ...`.

## 4. Look at the built-in patterns

```bash
norma list-patterns
```

On first run norma seeds its four MVP patterns — one "no debug print"
rule each for Java, Python, Rust and TypeScript — and prints them:

```
no-debug-print-java          java       [warning] No Debug Print
no-debug-print-python        python     [warning] No Debug Print
no-debug-print-rust          rust       [warning] No Debug Print
no-debug-print-typescript    typescript [warning] No Debug Print
```

The registry lives at `$HOME/.local/share/norma/norma.db`. Point norma
somewhere else with the global `--db <PATH>` flag or the `NORMA_DB`
environment variable.

## 5. Validate a file

```bash
norma validate --language rust src/main.rs
```

`validate` takes one or more files as positional arguments, so globs work
too. Add `--json` for machine-readable output (a JSON array with one
`{"file": ..., "result": ...}` entry per file):

```bash
norma validate --language rust --json src/*.rs
```

The exit status is non-zero if any file has violations — which is what
makes it usable as a pre-commit hook. Supported languages are `java`,
`python`, `rust` and `typescript`; anything else is a hard error rather
than a silent pass.

## 6. Run the MCP server

```bash
norma serve
```

`serve` speaks JSON-RPC over stdio, so it looks like it is hanging — that
is correct; it is waiting for an MCP client. Logs go to stderr, never
stdout. Stop it with `Ctrl-C`.

Register it with Claude Code:

```bash
claude mcp add norma -- norma serve
```

Or try it with the MCP Inspector:

```bash
npx @modelcontextprotocol/inspector norma serve
```

Then call, for example, `get_pattern_checklist` with `language: "java"`,
or `validate_pattern_compliance` with some code and a language.

## 7. Wire up the pre-commit hook

With `norma` installed (step 3) and [pre-commit](https://pre-commit.com/)
available:

```bash
pre-commit install
```

`.pre-commit-config.yaml` runs `norma validate --language rust --json`
over every staged `.rs` file on each commit.

## Troubleshooting

**`norma: command not found`** — you skipped `cargo install --path .`, or
`~/.cargo/bin` is not on your `PATH`.

**`Error: unsupported language: "go"`** — norma's MVP set covers Java,
Python, Rust and TypeScript only. This is deliberate: a language with no
patterns would otherwise report a meaningless "no violations".

**`norma serve` prints nothing** — expected. It is an stdio MCP server.
Use `RUST_LOG=info norma serve` to see its (stderr) logs.

**Rust not installed:**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```
