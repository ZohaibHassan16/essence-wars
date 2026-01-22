//! Essence Wars MCP Server
//!
//! This binary provides an MCP (Model Context Protocol) server for playing
//! the Essence Wars card game via Claude Code CLI.

use essence_wars_mcp::McpServer;

fn main() -> anyhow::Result<()> {
    // Create and run the MCP server
    let server = McpServer::new()?;
    server.run()?;

    Ok(())
}
