//! Tauri IPC command handlers.

mod deck_builder;
mod game;
mod mcp_sync;
mod replay;
mod spectator;

pub use deck_builder::*;
pub use game::*;
pub use mcp_sync::*;
pub use replay::*;
pub use spectator::*;
