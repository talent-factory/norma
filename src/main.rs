use mcpkit::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

mod models;
mod mcp_server;
mod pattern_engine;
mod pattern_store;

use mcp_server::NormaMcpServer;
use pattern_store::PatternStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    info!("Starting norma MCP server v0.1.0");

    // Initialize pattern store (SQLite)
    let pattern_store = PatternStore::new("norma.db").await?;
    
    // Ensure default patterns are loaded
    pattern_store.initialize_defaults().await?;
    
    info!("Pattern store initialized");

    // Create MCP server instance
    let server = NormaMcpServer::new(Arc::new(pattern_store));

    // Setup transport (stdio)
    let transport = (tokio::io::stdin(), tokio::io::stdout());
    
    info!("MCP transport established, waiting for connections...");

    // Run server
    let server = server.serve(transport).await?;
    
    info!("Server initialized, waiting for shutdown...");
    
    let quit_reason = server.waiting().await?;
    
    info!("Server shutting down: {:?}", quit_reason);

    Ok(())
}
