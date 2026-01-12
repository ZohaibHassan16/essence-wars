//! Arena module for running games between bots.
//!
//! Provides infrastructure for:
//! - Running individual games with full traceability
//! - Running matches (multiple games) with statistics
//! - Debug logging for game analysis

mod logger;
mod runner;
mod stats;

pub use logger::{ActionLogger, ActionRecord, LogOutput, StateSnapshot};
pub use runner::{GameRunner, GameResult};
pub use stats::{MatchStats, MatchupStats};
