//! norma: a design-pattern and code-quality validator built on ast-grep,
//! exposed both as an MCP tool server (`norma serve`) and a CLI
//! (`norma validate`, `norma list-patterns`). See docs/adr/0001.md and
//! docs/adr/0002.md for the architecture decisions this crate follows.
//!
//! Modules are added here one at a time by the tasks in
//! docs/superpowers/plans/2026-09-20-norma-mvp-implementation.md; each
//! addition should keep `cargo test --lib` green.

pub mod cli;
pub mod default_patterns;
pub mod mcp_server;
pub mod models;
pub mod pattern_engine;
pub mod pattern_store;
