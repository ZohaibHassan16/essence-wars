//! Game state management for the UI client.
//!
//! Manages active games and provides thread-safe access to game state.

mod game_manager;
mod serialization;

pub use game_manager::GameManager;
pub use serialization::*;
