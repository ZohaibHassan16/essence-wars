//! Execution utilities for parallel game running.
//!
//! This module provides shared infrastructure for running games in parallel,
//! used by arena, validate, and tune binaries.
//!
//! # Modules
//! - [`seeds`] - Deterministic seed derivation for parallel execution
//! - [`progress`] - Progress reporting for batch operations
//! - [`parallel`] - Parallel batch execution with rayon

mod parallel;
mod progress;
mod seeds;

pub use parallel::{
    configure_thread_pool, run_batch_parallel, run_batch_parallel_matchup, BatchConfig,
    BatchResult, GameOutcome,
};
pub use progress::{maybe_progress, ProgressReporter, ProgressStyle};
pub use seeds::{offsets, GameSeeds};
