//! Checkpoint system for CMA-ES tuning runs.
//!
//! Provides save/load functionality for optimizer state to enable:
//! - Resume from interruptions (Ctrl+C)
//! - Continue training from previous runs
//! - Checkpoint every generation for crash recovery

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Checkpoint format version for forward compatibility.
pub const CHECKPOINT_VERSION: u32 = 1;

/// Global interrupt flag for graceful shutdown.
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Check if an interrupt has been received.
pub fn is_interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// Register SIGINT handler for graceful checkpoint on Ctrl+C.
/// Only available when the `cli` feature is enabled.
#[cfg(feature = "cli")]
pub fn register_interrupt_handler() {
    ctrlc::set_handler(move || {
        if INTERRUPTED.swap(true, Ordering::SeqCst) {
            // Second Ctrl+C - force exit
            eprintln!("\nForce exit (second interrupt)");
            std::process::exit(1);
        }
        eprintln!("\nInterrupt received, saving checkpoint...");
    })
    .expect("Error setting Ctrl+C handler");
}

/// No-op interrupt handler for WASM builds.
#[cfg(not(feature = "cli"))]
pub fn register_interrupt_handler() {
    // No signal handling in WASM
}

/// Errors that can occur during checkpoint operations.
#[derive(Debug, Error)]
pub enum CheckpointError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("TOML serialization error: {0}")]
    Serialize(#[from] toml::ser::Error),

    #[error("TOML parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Dimension mismatch: expected {expected}, found {found}")]
    DimensionMismatch { expected: usize, found: usize },

    #[error("Invalid covariance matrix dimensions")]
    InvalidCovarianceMatrix,

    #[error("Checkpoint version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },

    #[error("Config mismatch: {field} differs (checkpoint: {checkpoint}, current: {current})")]
    ConfigMismatch {
        field: String,
        checkpoint: String,
        current: String,
    },

    #[error("Checkpoint file not found: {0}")]
    NotFound(PathBuf),

    #[error("Run ID not found: {0}")]
    RunNotFound(String),
}

/// Complete checkpoint for resuming a tuning run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TuningCheckpoint {
    /// Checkpoint format version
    pub version: u32,
    /// Timestamp when checkpoint was created
    pub created_at: String,
    /// Run ID (matches experiment directory name)
    pub run_id: String,
    /// CMA-ES optimizer state
    pub cmaes: CmaEsCheckpoint,
    /// Evaluator state
    pub evaluator: EvaluatorCheckpoint,
    /// Original configuration
    pub config: TuningConfig,
    /// Total elapsed time before checkpoint (seconds)
    pub elapsed_secs: f64,
}

impl TuningCheckpoint {
    /// Current checkpoint version.
    pub const VERSION: u32 = CHECKPOINT_VERSION;
}

/// Serializable snapshot of CMA-ES optimizer state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CmaEsCheckpoint {
    /// Problem dimensionality
    pub dim: usize,
    /// Population size (lambda)
    pub lambda: usize,
    /// Number of parents (mu)
    pub mu: usize,
    /// Current mean (best estimate)
    pub mean: Vec<f64>,
    /// Step size (sigma)
    pub sigma: f64,
    /// Covariance matrix (flattened row-major, dim*dim values)
    pub cov_flat: Vec<f64>,
    /// Evolution path for sigma
    pub ps: Vec<f64>,
    /// Evolution path for covariance
    pub pc: Vec<f64>,
    /// Current generation number
    pub generation: u32,
    /// Fitness history for stagnation detection
    pub fitness_history: Vec<f64>,
    /// Best fitness seen so far
    pub best_fitness: f64,
    /// Best weights found so far
    pub best_weights: Vec<f64>,
    /// Best win rate achieved
    pub best_win_rate: f64,
    /// Base seed for RNG derivation
    pub base_seed: u64,
}

impl CmaEsCheckpoint {
    /// Seed offset per generation for deterministic RNG.
    pub const SEED_OFFSET: u64 = 1_000_000;

    /// Flatten a 2D covariance matrix to 1D (row-major).
    pub fn flatten_cov(cov: &[Vec<f64>]) -> Vec<f64> {
        cov.iter().flat_map(|row| row.iter().copied()).collect()
    }

    /// Unflatten a 1D vector to 2D covariance matrix.
    pub fn unflatten_cov(flat: &[f64], dim: usize) -> Result<Vec<Vec<f64>>, CheckpointError> {
        if flat.len() != dim * dim {
            return Err(CheckpointError::InvalidCovarianceMatrix);
        }
        let mut cov = vec![vec![0.0; dim]; dim];
        for i in 0..dim {
            for j in 0..dim {
                cov[i][j] = flat[i * dim + j];
            }
        }
        Ok(cov)
    }
}

/// Serializable snapshot of Evaluator state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvaluatorCheckpoint {
    /// Total evaluations performed (for deterministic seeding)
    pub eval_count: u64,
}

/// Configuration subset needed for resume validation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TuningConfig {
    /// Tuning mode (generalist, specialist, etc.)
    pub mode: String,
    /// Maximum generations
    pub generations: u32,
    /// Games per evaluation
    pub games_per_eval: usize,
    /// Population size (if specified)
    pub population_size: Option<usize>,
    /// Initial sigma
    pub initial_sigma: f64,
    /// Base seed
    pub seed: u64,
    /// MCTS simulations
    pub mcts_sims: u32,
    /// Alpha-Beta depth
    pub ab_depth: u32,
    /// Deck ID (for specialist mode)
    pub deck: Option<String>,
    /// Opponent deck ID
    pub opponent: Option<String>,
    /// Faction filter
    pub faction: Option<String>,
    /// Archetype filter
    pub archetype: Option<String>,
}

/// Save a checkpoint to disk with atomic write.
///
/// Uses write-to-temp-then-rename for crash safety.
pub fn save_checkpoint(
    checkpoint: &TuningCheckpoint,
    experiment_dir: &Path,
) -> Result<PathBuf, CheckpointError> {
    let checkpoint_path = experiment_dir.join("checkpoint.toml");
    let temp_path = experiment_dir.join("checkpoint.toml.tmp");
    let backup_path = experiment_dir.join("checkpoint.toml.bak");

    // Serialize to TOML
    let content = toml::to_string_pretty(checkpoint)?;

    // Write to temp file first
    fs::write(&temp_path, &content)?;

    // Backup existing checkpoint if present
    if checkpoint_path.exists() {
        // Ignore backup errors (not critical)
        let _ = fs::rename(&checkpoint_path, &backup_path);
    }

    // Atomic rename temp -> final
    fs::rename(&temp_path, &checkpoint_path)?;

    Ok(checkpoint_path)
}

/// Load a checkpoint from a run ID.
///
/// Searches in experiments/{category}/{run_id}/checkpoint.toml
/// Supports partial ID matching (e.g., "baseline" matches "2026-01-31_1430_baseline")
pub fn load_checkpoint(
    run_id: &str,
    base_dir: &Path,
    category: &str,
) -> Result<TuningCheckpoint, CheckpointError> {
    let category_dir = base_dir.join(category);

    // Try exact match first
    let experiment_dir = category_dir.join(run_id);
    if experiment_dir.exists() {
        let checkpoint_path = experiment_dir.join("checkpoint.toml");
        if checkpoint_path.exists() {
            return load_checkpoint_file(&checkpoint_path);
        }
    }

    // Try suffix/substring match
    if let Ok(entries) = fs::read_dir(&category_dir) {
        let mut matches: Vec<_> = entries
            .flatten()
            .filter(|entry| {
                entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
            })
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                // Match if run_id is a suffix or substring
                name.ends_with(run_id) || name.contains(run_id)
            })
            .filter(|entry| {
                entry.path().join("checkpoint.toml").exists()
            })
            .collect();

        // Sort by name (most recent first due to timestamp prefix)
        matches.sort_by_key(|b| std::cmp::Reverse(b.file_name()));

        if let Some(entry) = matches.first() {
            let checkpoint_path = entry.path().join("checkpoint.toml");
            return load_checkpoint_file(&checkpoint_path);
        }
    }

    Err(CheckpointError::RunNotFound(run_id.to_string()))
}

/// Load a checkpoint from a specific file path.
pub fn load_checkpoint_file(path: &Path) -> Result<TuningCheckpoint, CheckpointError> {
    if !path.exists() {
        return Err(CheckpointError::NotFound(path.to_path_buf()));
    }

    let content = fs::read_to_string(path)?;
    let checkpoint: TuningCheckpoint = toml::from_str(&content)?;

    // Version check
    if checkpoint.version != TuningCheckpoint::VERSION {
        return Err(CheckpointError::VersionMismatch {
            expected: TuningCheckpoint::VERSION,
            found: checkpoint.version,
        });
    }

    Ok(checkpoint)
}

/// Validate that current config is compatible with checkpoint.
///
/// Critical fields must match exactly; some fields can differ (e.g., generations can increase).
pub fn validate_checkpoint_config(
    checkpoint: &TuningCheckpoint,
    current: &TuningConfig,
) -> Result<(), CheckpointError> {
    let ckpt = &checkpoint.config;

    // Critical fields that must match exactly
    if ckpt.mode != current.mode {
        return Err(CheckpointError::ConfigMismatch {
            field: "mode".to_string(),
            checkpoint: ckpt.mode.clone(),
            current: current.mode.clone(),
        });
    }

    if ckpt.seed != current.seed {
        return Err(CheckpointError::ConfigMismatch {
            field: "seed".to_string(),
            checkpoint: ckpt.seed.to_string(),
            current: current.seed.to_string(),
        });
    }

    // Warn if games_per_eval differs (affects fitness comparability)
    if ckpt.games_per_eval != current.games_per_eval {
        eprintln!(
            "Warning: games_per_eval differs (checkpoint: {}, current: {})",
            ckpt.games_per_eval, current.games_per_eval
        );
    }

    Ok(())
}

/// Find the experiment directory for a run ID.
pub fn find_experiment_dir(
    run_id: &str,
    base_dir: &Path,
    category: &str,
) -> Result<PathBuf, CheckpointError> {
    let category_dir = base_dir.join(category);

    // Try exact match first
    let experiment_dir = category_dir.join(run_id);
    if experiment_dir.exists() {
        return Ok(experiment_dir);
    }

    // Try suffix/substring match
    if let Ok(entries) = fs::read_dir(&category_dir) {
        let mut matches: Vec<_> = entries
            .flatten()
            .filter(|entry| {
                entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
            })
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                name.ends_with(run_id) || name.contains(run_id)
            })
            .collect();

        // Sort by name (most recent first)
        matches.sort_by_key(|b| std::cmp::Reverse(b.file_name()));

        if let Some(entry) = matches.first() {
            return Ok(entry.path());
        }
    }

    Err(CheckpointError::RunNotFound(run_id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cov_flatten_unflatten() {
        let cov = vec![
            vec![1.0, 0.1, 0.2],
            vec![0.1, 2.0, 0.3],
            vec![0.2, 0.3, 3.0],
        ];

        let flat = CmaEsCheckpoint::flatten_cov(&cov);
        assert_eq!(flat.len(), 9);

        let restored = CmaEsCheckpoint::unflatten_cov(&flat, 3).unwrap();
        assert_eq!(restored, cov);
    }

    #[test]
    fn test_unflatten_cov_wrong_size() {
        let flat = vec![1.0, 2.0, 3.0]; // 3 elements, not 4 (2x2)
        let result = CmaEsCheckpoint::unflatten_cov(&flat, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_checkpoint_version() {
        assert_eq!(TuningCheckpoint::VERSION, CHECKPOINT_VERSION);
    }
}
