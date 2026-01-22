//! Game state management for the UI client.
//!
//! Manages active games and provides thread-safe access to game state.

mod game_manager;
mod serialization;
mod spectator;

pub use game_manager::{GameManager, SpectatorComputer};
pub use serialization::*;
pub use spectator::*;
