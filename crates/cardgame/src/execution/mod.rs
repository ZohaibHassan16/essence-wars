//! Execution utilities for parallel game running.
//!
//! This module provides shared infrastructure for running games in parallel,
//! used by arena, validate, and tune binaries.
//!
//! # Modules
//!
//! ## Core Execution
//! - [`seeds`] - Deterministic seed derivation for parallel execution
//! - [`progress`] - Progress reporting for batch operations
//! - [`parallel`] - Parallel batch execution with rayon
//!
//! ## Data Loading
//! - [`data_loader`] - Unified loading of cards, decks, commanders, and weights
//!
//! ## Matchups
//! - [`matchup`] - Unified matchup definition with commander support
//! - [`matchup_builder`] - Generate matchups at various granularities
//!
//! ## Metrics
//! - [`metrics`] - Comprehensive per-game and aggregated metrics

mod data_loader;
mod matchup;
mod matchup_builder;
mod metrics;
mod parallel;
mod progress;
mod seeds;

// Data loading
pub use data_loader::{GameData, GameDataError};

// Matchups
pub use matchup::UnifiedMatchup;
pub use matchup_builder::{MatchupBuilder, MatchupSummary};

// Metrics
pub use metrics::{AggregatedMetrics, GameMetricsData};

// Parallel execution (basic)
pub use parallel::{
    configure_thread_pool, run_batch_parallel, run_batch_parallel_matchup, BatchConfig,
    BatchResult, GameOutcome,
};

// Parallel execution (with metrics)
pub use parallel::{
    run_batch_parallel_matchup_with_metrics, run_batch_parallel_with_metrics,
    BatchResultWithMetrics, GameOutcomeWithMetrics,
};

// Progress reporting
pub use progress::{maybe_progress, ProgressReporter, ProgressStyle};

// Seed derivation
pub use seeds::{offsets, GameSeeds};
