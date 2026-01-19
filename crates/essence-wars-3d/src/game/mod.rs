//! Game bridge module - connects cardgame engine to Bevy
//!
//! This module provides the integration layer between the Rust cardgame
//! engine and the Bevy 3D client.

mod bridge;
mod state;

pub use bridge::GameBridge;
pub use state::{AppState, GamePlugin};
