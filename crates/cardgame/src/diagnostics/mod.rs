//! Diagnostic tools for P1/P2 asymmetry analysis.
//!
//! This module provides tools to collect, analyze, and report game statistics
//! for investigating first-player advantage/disadvantage.
//!
//! # Example
//!
//! ```no_run
//! use cardgame::cards::CardDatabase;
//! use cardgame::diagnostics::{
//!     AggregatedStats, DiagnosticConfig, DiagnosticRunner, print_report
//! };
//! use cardgame::types::CardId;
//!
//! let card_db = CardDatabase::load_from_directory("data/cards/core_set").unwrap();
//! let deck: Vec<CardId> = vec![CardId(1000); 30]; // Example deck
//!
//! let config = DiagnosticConfig::new(deck, 100);
//! let runner = DiagnosticRunner::new(&card_db);
//! let games = runner.run(&config);
//!
//! let stats = AggregatedStats::analyze(&games);
//! print_report(&stats);
//! ```

mod analyzer;
mod collector;
pub mod export;
pub mod metrics;
mod report;
pub mod statistics;

pub use analyzer::{AggregatedStats, BalanceAssessment};
pub use collector::{DiagnosticConfig, DiagnosticRunner, GameDiagnostics, TurnSnapshot};
pub use export::{export_csv, export_json, ExportFormat};
pub use metrics::{
    BoardAdvantage, CombatEfficiency, GameMetrics, ResourceEfficiency, TempoMetrics, TurnMetrics,
};
pub use report::print_report;
pub use statistics::{
    chi_square_test, correlation_p_value, mean_variance, pearson_correlation, percentile,
    wilson_score_interval, ProportionStats, SignificanceLevel,
};
