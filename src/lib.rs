//! Card game engine library
//!
//! This crate provides a complete card game engine with support for:
//! - Card definitions and keywords
//! - Game state management
//! - Combat resolution
//! - Effect processing
//! - AI tensor representation
//! - Bot implementations and arena for running matches

// Core engine module (contains all game logic)
pub mod core;

// AI tensor representation (depends on core)
pub mod tensor;

// Bot and arena modules (engine-agnostic)
pub mod bots;
pub mod arena;
pub mod decks;
pub mod tuning;

// Re-export modules from core at crate root for backward compatibility
pub use core::types;
pub use core::keywords;
pub use core::effects;
pub use core::state;
pub use core::cards;
pub use core::actions;
pub use core::legal;
pub use core::engine;
pub use core::combat;
pub use core::config;

// Re-export key types at crate root for convenience
pub use core::types::*;
pub use core::keywords::*;
pub use core::state::*;
pub use core::cards::*;
pub use core::actions::*;
pub use core::engine::*;
