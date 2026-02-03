//! Game state management for the UI client.
//!
//! Manages active games and provides thread-safe access to game state.

mod custom_deck;
mod custom_deck_manager;
mod game_manager;
mod playstyle;
mod replay_manager;
mod replay_types;
mod serialization;
mod spectator;

pub use custom_deck::*;
pub use custom_deck_manager::CustomDeckManager;
pub use game_manager::{GameManager, SpectatorComputer};
pub use replay_manager::ReplayManager;
pub use replay_types::*;
pub use serialization::*;
pub use spectator::*;
