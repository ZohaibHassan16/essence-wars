//! Core game engine modules for Essence Wars.
//!
//! This module contains all the foundational game logic:
//! - Type definitions
//! - Card and effect systems
//! - Game state management
//! - Engine and combat resolution
//! - Configuration constants

// Foundation modules (no internal deps)
pub mod types;
pub mod keywords;
pub mod config;

// Effect system (depends on types)
pub mod effects;

// State management (depends on types, keywords)
pub mod state;

// Card definitions (depends on types, keywords, effects)
pub mod cards;

// Action system (depends on types)
pub mod actions;

// Legal action generation (depends on types, actions, state, cards, effects)
pub mod legal;

// Combat resolution (depends on most modules)
pub mod combat;

// Main game engine (depends on all above)
pub mod engine;

// Tracing infrastructure for debugging (depends on types, keywords, effects)
pub mod tracing;

// Re-export commonly used items for convenience
pub use types::*;
pub use keywords::*;
pub use state::*;
pub use cards::*;
pub use actions::*;
pub use engine::*;
