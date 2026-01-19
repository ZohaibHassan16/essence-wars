//! Client API for game integration.
//!
//! This module provides a high-level API for building game clients,
//! including web clients, 3D JRPG integration, and AI visualization.
//!
//! # Overview
//!
//! The client_api module provides:
//!
//! - **GameClient**: High-level game controller with event emission
//! - **GameEvent**: All game state changes as discrete events
//! - **PlayerInput**: Unified interface for human and bot players
//! - **State Diffing**: Automatic change detection between states
//!
//! # Example
//!
//! ```ignore
//! use cardgame::client_api::{GameClient, GameEvent};
//! use cardgame::cards::CardDatabase;
//! use std::sync::Arc;
//!
//! // Load card database
//! let db = Arc::new(CardDatabase::load_all().unwrap());
//!
//! // Create game client
//! let mut client = GameClient::new(db);
//!
//! // Start a game
//! let deck1 = vec![/* card IDs */];
//! let deck2 = vec![/* card IDs */];
//! client.start_game(deck1, deck2, 12345);
//!
//! // Game loop
//! while !client.is_game_over() {
//!     // Get events for UI updates
//!     let events = client.drain_events();
//!     for event in events {
//!         match event {
//!             GameEvent::CreatureSpawned { player, slot, .. } => {
//!                 // Update UI with new creature
//!             }
//!             GameEvent::LifeChanged { player, new_life, .. } => {
//!                 // Update life display
//!             }
//!             _ => {}
//!         }
//!     }
//!
//!     // Get and apply player action
//!     let legal = client.get_legal_actions();
//!     let action = /* get from player */;
//!     client.apply_action(action).unwrap();
//! }
//! ```
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │              Game Clients               │
//! │  (Web, JRPG, Training, CLI, etc.)      │
//! └─────────────────┬───────────────────────┘
//!                   │
//!                   ▼
//! ┌─────────────────────────────────────────┐
//! │            client_api Module            │
//! │  ┌─────────────┐  ┌──────────────────┐ │
//! │  │ GameClient  │  │   GameEvent      │ │
//! │  │ + events    │  │   (serializable) │ │
//! │  └─────────────┘  └──────────────────┘ │
//! │  ┌─────────────┐  ┌──────────────────┐ │
//! │  │ PlayerInput │  │ State Diffing    │ │
//! │  │ (trait)     │  │ (change detect)  │ │
//! │  └─────────────┘  └──────────────────┘ │
//! └─────────────────┬───────────────────────┘
//!                   │
//!                   ▼
//! ┌─────────────────────────────────────────┐
//! │          Core Engine (unchanged)        │
//! │  GameEngine, GameState, Actions, etc.  │
//! └─────────────────────────────────────────┘
//! ```

mod client;
mod diff;
mod events;
mod input;

pub use client::{GameClient, GameClientBuilder, GameClientConfig};
pub use diff::{
    detect_card_drawn, detect_overdraw, diff_states, CreatureSnapshot, PlayerSnapshot,
    StateSnapshot, SupportSnapshot,
};
pub use events::{
    DamageSource, DeathCause, DiscardReason, GameEvent, LifeChangeSource, SupportRemovalReason,
};
pub use input::{BotInput, HumanInput, PlayerInput, ScriptedInput};
