//! Game bridge module - connects cardgame engine to Bevy
//!
//! This module provides the integration layer between the Rust cardgame
//! engine and the Bevy 3D client.

mod bridge;
#[cfg(not(target_arch = "wasm32"))]
pub mod screenshot;
mod state;
mod stats;
pub mod turn_loop;

pub use bridge::GameBridge;
#[cfg(not(target_arch = "wasm32"))]
pub use screenshot::{ScreenshotPlugin, ScreenshotState};
pub use state::{AppState, DragState, DragType, GamePlugin};
pub use stats::HeadlessStats;
pub use turn_loop::GameEventWrapper;
