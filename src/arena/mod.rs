//! Arena module for running games between bots.
//!
//! Provides infrastructure for:
//! - Running individual games with full traceability
//! - Running matches (multiple games) with statistics
//! - Debug logging for game analysis
//! - Combat resolution tracing for debugging
//! - Effect queue tracing for debugging
//!
//! ## High-Level API (Recommended)
//!
//! Use [`run_match_parallel`] for performance or [`run_match_sequential`] for debugging:
//!
//! ```ignore
//! use cardgame::arena::{MatchConfig, run_match_parallel};
//! use cardgame::bots::BotType;
//!
//! let config = MatchConfig::new(
//!     BotType::Random,
//!     BotType::Random,
//!     deck1,
//!     deck2,
//!     100,
//!     42,
//! );
//! let stats = run_match_parallel(&card_db, &config);
//! ```

mod config;
mod deck_utils;
mod executor;
mod logger;
mod runner;
mod stats;

// Configuration types
pub use config::{MatchConfig, SequentialConfig};

// Deck utilities
pub use deck_utils::{create_default_deck, load_deck, validate_faction_deck_binding, LoadedDeck};

// Execution functions
pub use executor::{run_match_parallel, run_match_sequential};

// Re-export tracing types from core::tracing for convenience
pub use crate::core::tracing::{
    CombatPhase, CombatStep, CombatTrace, CombatTracer, EffectEvent, EffectEventType, EffectTracer,
};

// Logger types
pub use logger::{
    ActionLogger, ActionRecord, CombatTrace as LoggerCombatTrace, CreatureSnapshot, LogOutput,
    StateSnapshot,
};

// Runner types (low-level API)
pub use runner::{GameResult, GameRunner};

// Statistics types
pub use stats::{MatchStats, MatchupStats};
