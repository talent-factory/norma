// norma: Developer-grade code pattern enforcement everywhere you code
//
// This library provides the core functionality for pattern validation and enforcement
// via the Model Context Protocol (MCP), enabling seamless integration with Claude Code
// and other AI-powered development tools.

pub mod models;
pub mod mcp_server;
pub mod pattern_engine;
pub mod pattern_store;

pub use models::*;
pub use mcp_server::NormaMcpServer;
pub use pattern_engine::PatternEngine;
pub use pattern_store::PatternStore;
