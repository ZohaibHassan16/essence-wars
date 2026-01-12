//! Card game engine library
//!
//! This crate provides a complete card game engine with support for:
//! - Card definitions and keywords
//! - Game state management
//! - Combat resolution
//! - Effect processing
//! - AI tensor representation
//! - Bot implementations and arena for running matches

// Core engine modules
pub mod types;
pub mod keywords;
pub mod effects;
pub mod state;
pub mod cards;
pub mod actions;
pub mod legal;
pub mod engine;
pub mod combat;
pub mod tensor;
pub mod config;

// Bot and arena modules (engine-agnostic)
pub mod bots;
pub mod arena;
pub mod decks;
pub mod tuning;

// Re-export key types at crate root for convenience
pub use types::*;
pub use keywords::*;
pub use state::*;
pub use cards::*;
pub use actions::*;
pub use engine::*;
