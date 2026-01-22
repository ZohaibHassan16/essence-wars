//! Game state management for the UI client.
//!
//! Manages active games and provides thread-safe access to game state.

mod game_manager;
mod replay_manager;
mod replay_types;
mod serialization;
mod spectator;

pub use game_manager::{GameManager, SpectatorComputer};
pub use replay_manager::ReplayManager;
pub use replay_types::*;
pub use serialization::*;
pub use spectator::*;
