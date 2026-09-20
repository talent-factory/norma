---
Status: accepted
---

# Single binary with subcommands, shared async validation core

norma needs two interfaces: an MCP tool server (long-running stdio loop via `rmcp`) and a CLI for pre-commit hooks (one-shot invocation). We considered a Cargo workspace with separate crates/binaries for each, but decided on a **single binary with `clap` subcommands** (`norma serve`, `norma validate`, ...) that both call into the same async `pattern_engine`/`pattern_store` core — no synchronous duplicate of the validation logic.

Rejected: a multi-crate workspace, which is the more common shape for "server + CLI" Rust projects. We chose the simpler single-binary shape instead because norma is meant to become a teaching artifact for FFHS students, and a workspace split would add structural ceremony (multiple `Cargo.toml`s, crate boundaries) without a concrete benefit at this scale — it would obscure the code for the audience it's meant to teach, not clarify it.

The CLI's `validate` subcommand runs the same async functions as the MCP server via `#[tokio::main]`, rather than a separate sync code path — tokio is already a dependency, and the overhead of an async runtime for a one-shot process is negligible next to the maintenance cost of two implementations of the same validation logic.
