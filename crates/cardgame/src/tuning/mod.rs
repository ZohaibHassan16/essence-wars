//! Weight tuning pipeline for bot optimization.
//!
//! This module provides:
//! - CMA-ES optimizer for evolving bot weights
//! - Evaluator for measuring bot performance via matches
//! - Experiment directory management
//! - Generalist mode (optimize across multiple decks)
//! - Specialist mode (optimize for a single deck matchup)

mod cmaes;
mod evaluator;
mod experiment;

pub use cmaes::{CmaEs, CmaEsConfig};
pub use evaluator::{Evaluator, EvaluatorConfig, FitnessResult, TuningMode};
pub use experiment::{deploy_weights, ExperimentConfig, ExperimentDir};
