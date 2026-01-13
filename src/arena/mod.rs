//! Arena module for running games between bots.
//!
//! Provides infrastructure for:
//! - Running individual games with full traceability
//! - Running matches (multiple games) with statistics
//! - Debug logging for game analysis
//! - Combat resolution tracing for debugging
//! - Effect queue tracing for debugging

mod combat_tracer;
mod effect_tracer;
mod logger;
mod runner;
mod stats;

pub use combat_tracer::{CombatPhase, CombatStep, CombatTrace, CombatTracer};
pub use effect_tracer::{EffectEvent, EffectEventType, EffectTracer};
pub use logger::{ActionLogger, ActionRecord, LogOutput, StateSnapshot};
pub use runner::{GameRunner, GameResult};
pub use stats::{MatchStats, MatchupStats};
