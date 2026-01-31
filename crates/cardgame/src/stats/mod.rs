//! Card statistics collection and analysis.
//!
//! This module provides tools for tracking per-card performance metrics
//! during game execution, enabling identification of problematic cards
//! for balance analysis.
//!
//! # Example
//!
//! ```ignore
//! use cardgame::stats::{CardStatsCallback, CardStatsCollector};
//! use cardgame::execution::game_loop::{run_game_loop, GameLoopConfig};
//!
//! let mut collector = CardStatsCollector::new();
//! let mut callback = CardStatsCallback::new(&mut collector);
//!
//! // Run game with callback
//! let config = GameLoopConfig::new(seed);
//! run_game_loop(&mut engine, &mut bot1, &mut bot2, &config, &mut callback);
//!
//! // Analyze results
//! for (card_id, stats) in collector.cards_by_impact() {
//!     println!("{}: {:.1}% win rate", card_id.0, stats.win_rate() * 100.0);
//! }
//! ```

mod callback;
mod collector;
pub mod report;
mod types;

pub use callback::CardStatsCallback;
pub use collector::CardStatsCollector;
pub use report::{
    build_card_report, export_csv, export_json, print_report, CardReport, FullReport, ReportConfig,
    ReportSummary,
};
pub use types::{CardPlayStats, GameCardTracker, TurnBucket};
