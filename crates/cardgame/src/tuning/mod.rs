//! Weight tuning pipeline for bot optimization.
//!
//! This module provides:
//! - CMA-ES optimizer for evolving bot weights
//! - Evaluator for measuring bot performance via matches
//! - Experiment directory management
//! - Generalist mode (optimize across multiple decks)
//! - Specialist mode (optimize for a single deck matchup)
//! - Checkpoint/resume functionality for long runs

pub mod checkpoint;
mod cmaes;
mod evaluator;
mod experiment;

pub use checkpoint::{
    is_interrupted, load_checkpoint, register_interrupt_handler, save_checkpoint,
    CheckpointError, CmaEsCheckpoint, EvaluatorCheckpoint, TuningCheckpoint, TuningConfig,
};
pub use cmaes::{CmaEs, CmaEsConfig};
pub use evaluator::{CandidateType, Evaluator, EvaluatorConfig, FitnessResult, TuningMode};
pub use experiment::{deploy_weights, ExperimentConfig, ExperimentDir};
