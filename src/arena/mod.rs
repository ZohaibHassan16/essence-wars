//! Arena module for running games between bots.
//!
//! Provides infrastructure for:
//! - Running individual games with full traceability
//! - Running matches (multiple games) with statistics
//! - Debug logging for game analysis
//! - Combat resolution tracing for debugging
//! - Effect queue tracing for debugging

mod logger;
mod runner;
mod stats;

// Re-export tracing types from core::tracing for convenience
pub use crate::core::tracing::{
    CombatPhase, CombatStep, CombatTrace, CombatTracer,
    EffectEvent, EffectEventType, EffectTracer,
};
pub use logger::{ActionLogger, ActionRecord, CombatTrace as LoggerCombatTrace, CreatureSnapshot, LogOutput, StateSnapshot};
pub use runner::{GameRunner, GameResult};
pub use stats::{MatchStats, MatchupStats};
