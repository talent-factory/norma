# norma Quick Start

Get **norma** running in 5 minutes. For the full reference see
[README.md](README.md); for contributor workflow see
[DEVELOPMENT.md](DEVELOPMENT.md).

## 1. Prerequisites

- Rust stable (see `rust-toolchain.toml`) — [install rustup](https://rustup.rs/)
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

On first run norma seeds its 20 default patterns -- the four MVP "no debug
print" patterns plus 16 Gang-of-Four patterns (Singleton, Factory,
Observer, Strategy), one per language -- and prints them:

```
factory-overuse-java         java       [warning] Factory Overuse (Type Switch)
no-debug-print-java          java       [warning] No Debug Print
observer-presence-java       java       [info] Observer Presence
singleton-quality-java       java       [warning] Singleton Implementation Quality
strategy-overuse-java        java       [warning] Strategy Overuse (Type Switch)
factory-overuse-python       python     [warning] Factory Overuse (Type Switch)
no-debug-print-python        python     [warning] No Debug Print
observer-presence-python     python     [info] Observer Presence
singleton-quality-python     python     [warning] Singleton Implementation Quality
strategy-overuse-python      python     [warning] Strategy Overuse (Type Switch)
factory-overuse-rust         rust       [warning] Factory Overuse (Type Switch)
no-debug-print-rust          rust       [warning] No Debug Print
observer-presence-rust       rust       [info] Observer Presence
singleton-quality-rust       rust       [warning] Singleton Implementation Quality
strategy-overuse-rust        rust       [warning] Strategy Overuse (Type Switch)
factory-overuse-typescript   typescript [warning] Factory Overuse (Type Switch)
no-debug-print-typescript    typescript [warning] No Debug Print
observer-presence-typescript typescript [info] Observer Presence
singleton-quality-typescript typescript [warning] Singleton Implementation Quality
strategy-overuse-typescript  typescript [warning] Strategy Overuse (Type Switch)
```

The registry lives at `$XDG_DATA_HOME/norma/norma.db` (or
`$HOME/.local/share/norma/norma.db` if `$XDG_DATA_HOME` isn't set). Point
norma somewhere else with the global `--db <PATH>` flag or the `NORMA_DB`
environment variable.

## 5. Validate a file

```bash
norma validate --language rust src/models.rs
```

(`src/main.rs` is a bad first example here: its whole job is printing to
stdout, so it trips the "no debug print" pattern -- see
`tests/dogfooding.rs`'s comment on excluding it from norma's self-check.)

`validate` takes one or more files as positional arguments, so globs work
too. Add `--json` for machine-readable output (a JSON array with one
`{"file": ..., "result": ...}` entry per file):

```bash
norma validate --language rust --json src/*.rs
```

The exit status is non-zero if any file has violations — which is what
makes it usable as a pre-commit hook. `--language` accepts any of the 28
languages `ast-grep-language` supports (e.g. `go`, `css`, `markdown`, ...,
not just `java`/`python`/`rust`/`typescript`); anything else is a hard
error rather than a silent pass. Default patterns currently only ship
for `java`/`python`/`rust`/`typescript`, though -- validating a file in
one of the other 24 languages succeeds but reports "no enabled patterns
are registered for language ..." rather than "no violations".

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

**`Error: unsupported language: "cobol"`** — norma supports every
language `ast-grep-language` ships (28 total; see the error's own
"norma supports: ..." list), but nothing outside that set. This is
deliberate: a language with no patterns would otherwise report a
meaningless "no violations" instead of erroring loudly.

**`norma serve` prints nothing** — expected. It is an stdio MCP server.
Use `RUST_LOG=info norma serve` to see its (stderr) logs.

**Rust not installed:**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```
