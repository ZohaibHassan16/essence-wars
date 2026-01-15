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
mod report;

pub use analyzer::AggregatedStats;
pub use collector::{DiagnosticConfig, DiagnosticRunner, GameDiagnostics, TurnSnapshot};
pub use report::print_report;
