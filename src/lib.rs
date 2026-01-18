//! Card game engine library
//!
//! This crate provides a complete card game engine with support for:
//! - Card definitions and keywords
//! - Game state management
//! - Combat resolution
//! - Effect processing
//! - AI tensor representation
//! - Bot implementations and arena for running matches

// Version info for reproducibility
pub mod version;

// Core engine module (contains all game logic)
pub mod core;

// AI tensor representation (depends on core)
pub mod tensor;

// Bot and arena modules (engine-agnostic)
pub mod bots;
pub mod arena;
pub mod decks;
pub mod tuning;

// Execution utilities for parallel game running
pub mod execution;

// Validation module for balance testing
pub mod validation;

// Diagnostics module for P1/P2 asymmetry analysis
pub mod diagnostics;

// Python bindings (only compiled with --features python)
#[cfg(feature = "python")]
pub mod python;

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

// Re-export deck types
pub use decks::{DeckDefinition, DeckRegistry, DeckError, Faction, FactionParseError};
