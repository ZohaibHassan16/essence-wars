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

// Bot modules (engine-agnostic)
pub mod bots;
pub mod decks;

// Arena module for batch game running (CLI only - uses execution)
#[cfg(feature = "cli")]
pub mod arena;

// Tuning module (requires parallel feature for heavy computation)
#[cfg(feature = "parallel")]
pub mod tuning;

// Execution utilities for game running (CLI only - uses clap, indicatif, etc.)
#[cfg(feature = "cli")]
pub mod execution;

// Validation module for balance testing (requires parallel feature)
#[cfg(feature = "parallel")]
pub mod validation;

// Diagnostics module for P1/P2 asymmetry analysis (CLI only - uses execution)
#[cfg(feature = "cli")]
pub mod diagnostics;

// Card statistics module for per-card performance analysis (CLI only - uses execution)
#[cfg(feature = "cli")]
pub mod stats;

// Client API for game integration (web, JRPG, training)
pub mod client_api;

// Replay system for game recording and playback
pub mod replay;

// Embedded data for builds without filesystem access (e.g., Python bindings)
pub mod embedded_data;

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

/// Get the path to the data directory.
///
/// Resolution order:
/// 1. CARDGAME_DATA_DIR environment variable
/// 2. Relative to workspace root (crates/cardgame -> ../../data)
/// 3. Fallback to "data" (for when running from workspace root)
pub fn data_dir() -> std::path::PathBuf {
    std::env::var("CARDGAME_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            // Look for data/ relative to workspace root
            let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            manifest_dir
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.join("data"))
                .unwrap_or_else(|| std::path::PathBuf::from("data"))
        })
}
