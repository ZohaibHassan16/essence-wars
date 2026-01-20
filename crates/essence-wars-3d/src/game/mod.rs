//! Game bridge module - connects cardgame engine to Bevy
//!
//! This module provides the integration layer between the Rust cardgame
//! engine and the Bevy 3D client.

mod bridge;
mod state;
mod stats;
pub mod turn_loop;

pub use bridge::GameBridge;
pub use state::{AppState, GamePlugin};
pub use stats::HeadlessStats;
pub use turn_loop::GameEventWrapper;
