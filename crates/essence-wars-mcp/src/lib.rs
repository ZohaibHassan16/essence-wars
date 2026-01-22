//! Essence Wars MCP Server Library
//!
//! This module provides the MCP service handler for the Essence Wars card game.
//! Implements the Model Context Protocol (MCP) for Claude Code integration.

pub mod ascii;
pub mod mcp;
pub mod session;
pub mod tools;

pub use mcp::McpServer;
