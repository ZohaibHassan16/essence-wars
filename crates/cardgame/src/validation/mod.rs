//! Balance validation module.
//!
//! Provides infrastructure for running comprehensive faction matchup testing
//! and analyzing game balance.
//!
//! ## Overview
//!
//! This module is used to validate that the game is balanced across factions
//! and player positions. It runs MCTS agents against each other in all
//! faction combinations and analyzes the results.
//!
//! ## Usage
//!
//! ```ignore
//! use cardgame::validation::{
//!     MatchupBuilder, ValidationExecutor, BalanceAnalyzer,
//!     FactionWeights, ValidationConfig, ValidationResults,
//! };
//!
//! // Build matchups
//! let builder = MatchupBuilder::new(&deck_registry, &card_db);
//! let matchups = builder.build_faction_matchups();
//!
//! // Run validation
//! let executor = ValidationExecutor::new(&card_db, 100);
//! let results = executor.run_all(&matchups, &faction_weights, 500, 42);
//!
//! // Analyze balance
//! let analyzer = BalanceAnalyzer::new();
//! let summary = analyzer.analyze(&results);
//! ```

mod analyzer;
mod executor;
mod game_diagnostics;
mod matchup;
mod report;
mod types;

// Re-export types
pub use types::{
    BalanceStatus, BalanceSummary, DirectionDiagnostics, DirectionResults, FactionWeights,
    MatchupDefinition, MatchupDiagnostics, MatchupP1Stats, MatchupResult, P1P2Summary,
    ValidationConfig, ValidationResults,
};

// Re-export diagnostic types
pub use game_diagnostics::{GameDiagnosticCollector, GameDiagnosticData};

// Re-export matchup builder
pub use matchup::{filter_matchups, MatchupBuilder};

// Re-export executor
pub use executor::ValidationExecutor;

// Re-export analyzer
pub use analyzer::BalanceAnalyzer;

// Re-export report utilities
pub use report::{capitalize, export_json, print_results, save_validation_results, ExportError};
