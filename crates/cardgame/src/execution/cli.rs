//! Shared CLI utilities for cardgame binaries.
//!
//! This module provides common argument patterns, parsing helpers, and configuration
//! builders used across multiple CLI binaries (arena, validate, diagnose, tune, etc.).
//!
//! # Usage
//!
//! ```ignore
//! use cardgame::execution::cli::{parse_bot_type_or_exit, resolve_seed, MctsArgs, AlphaBetaArgs};
//!
//! // Parse bot type with helpful error message
//! let bot_type = parse_bot_type_or_exit(&args.bot);
//!
//! // Resolve seed (use provided or generate from time)
//! let seed = resolve_seed(args.seed);
//!
//! // Build MCTS config from args
//! let mcts_config = mcts_args.to_config();
//! ```

use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::bots::{AlphaBetaConfig, BotType, MctsConfig};

// ============================================================================
// Default Paths
// ============================================================================

/// Default path to card definitions.
pub const DEFAULT_CARDS_PATH: &str = "data/cards/core_set";

/// Default path to commander definitions.
pub const DEFAULT_COMMANDERS_PATH: &str = "data/commanders";

/// Default path to deck definitions.
pub const DEFAULT_DECKS_PATH: &str = "data/decks";

/// Default path to weight files.
pub const DEFAULT_WEIGHTS_PATH: &str = "data/weights";

// ============================================================================
// Bot Type Parsing
// ============================================================================

/// List of all valid bot type strings for error messages.
pub const VALID_BOT_TYPES: &[&str] = &[
    "random",
    "greedy",
    "mcts",
    "alphabeta (or alpha-beta, ab)",
    "agent-argentum",
    "agent-symbiote",
    "agent-obsidion",
    "agent-generalist",
];

/// Parse a bot type string or exit with a helpful error message.
///
/// This is the preferred way to parse bot types in CLI binaries as it provides
/// consistent, user-friendly error messages.
///
/// # Example
///
/// ```ignore
/// let bot_type = parse_bot_type_or_exit(&args.bot);
/// ```
pub fn parse_bot_type_or_exit(s: &str) -> BotType {
    match s.parse() {
        Ok(bot_type) => bot_type,
        Err(_) => {
            eprintln!("Error: Unknown bot type '{}'", s);
            eprintln!();
            eprintln!("Valid bot types:");
            for bot in VALID_BOT_TYPES {
                eprintln!("  - {}", bot);
            }
            process::exit(1);
        }
    }
}

/// Parse a bot type string, returning an error message suitable for display.
///
/// Use this when you want to handle the error yourself rather than exiting.
pub fn parse_bot_type(s: &str) -> Result<BotType, String> {
    s.parse().map_err(|_| {
        format!(
            "Unknown bot type '{}'. Valid options: {}",
            s,
            VALID_BOT_TYPES.join(", ")
        )
    })
}

// ============================================================================
// Seed Resolution
// ============================================================================

/// Resolve an optional seed, using system time if not provided.
///
/// This provides consistent seed handling across binaries:
/// - If a seed is provided, use it directly
/// - If not, generate one from the current system time
///
/// # Example
///
/// ```ignore
/// #[arg(long, short = 's')]
/// seed: Option<u64>,
///
/// let seed = resolve_seed(args.seed);
/// ```
pub fn resolve_seed(seed: Option<u64>) -> u64 {
    seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(42)
    })
}

/// Resolve an optional seed with a default value.
///
/// Use this when you want a specific default rather than time-based.
pub fn resolve_seed_with_default(seed: Option<u64>, default: u64) -> u64 {
    seed.unwrap_or(default)
}

// ============================================================================
// MCTS Configuration
// ============================================================================

/// Common MCTS configuration arguments.
///
/// Use with `#[command(flatten)]` in clap to include these args in your CLI.
///
/// # Example
///
/// ```ignore
/// #[derive(Parser)]
/// struct Args {
///     #[command(flatten)]
///     mcts: MctsArgs,
/// }
///
/// let mcts_config = args.mcts.to_config();
/// ```
#[derive(Debug, Clone, clap::Args)]
pub struct MctsArgs {
    /// Number of MCTS simulations per move
    #[arg(long, default_value = "500")]
    pub mcts_sims: u32,

    /// Number of parallel trees for MCTS root parallelization (1 = sequential)
    #[arg(long, default_value = "1")]
    pub mcts_trees: u32,

    /// Number of parallel rollouts per MCTS leaf (1 = sequential)
    #[arg(long, default_value = "1")]
    pub mcts_rollouts: u32,
}

impl Default for MctsArgs {
    fn default() -> Self {
        Self {
            mcts_sims: 500,
            mcts_trees: 1,
            mcts_rollouts: 1,
        }
    }
}

impl MctsArgs {
    /// Convert to MctsConfig for bot creation.
    pub fn to_config(&self) -> MctsConfig {
        MctsConfig {
            simulations: self.mcts_sims,
            exploration: 1.414,
            max_rollout_depth: 100,
            parallel_trees: self.mcts_trees,
            leaf_rollouts: self.mcts_rollouts,
            ..MctsConfig::default()
        }
    }

    /// Create with custom simulation count.
    pub fn with_sims(sims: u32) -> Self {
        Self {
            mcts_sims: sims,
            ..Default::default()
        }
    }
}

// ============================================================================
// Alpha-Beta Configuration
// ============================================================================

/// Common Alpha-Beta configuration arguments.
///
/// Use with `#[command(flatten)]` in clap to include these args in your CLI.
///
/// # Example
///
/// ```ignore
/// #[derive(Parser)]
/// struct Args {
///     #[command(flatten)]
///     alphabeta: AlphaBetaArgs,
/// }
///
/// let ab_config = args.alphabeta.to_config();
/// ```
#[derive(Debug, Clone, clap::Args)]
pub struct AlphaBetaArgs {
    /// Alpha-Beta search depth (plies)
    #[arg(long, default_value = "6")]
    pub ab_depth: u32,
}

impl Default for AlphaBetaArgs {
    fn default() -> Self {
        Self { ab_depth: 6 }
    }
}

impl AlphaBetaArgs {
    /// Convert to AlphaBetaConfig for bot creation.
    pub fn to_config(&self) -> AlphaBetaConfig {
        AlphaBetaConfig::with_depth(self.ab_depth)
    }

    /// Create with custom depth.
    pub fn with_depth(depth: u32) -> Self {
        Self { ab_depth: depth }
    }
}

// ============================================================================
// Data Paths
// ============================================================================

/// Standard data paths with defaults.
///
/// This struct provides consistent path handling across binaries.
/// All paths have sensible defaults that can be overridden.
#[derive(Debug, Clone)]
pub struct DataPaths {
    /// Path to card definitions directory.
    pub cards: PathBuf,
    /// Path to commander definitions directory.
    pub commanders: PathBuf,
    /// Path to deck definitions directory.
    pub decks: PathBuf,
    /// Path to weights directory.
    pub weights: PathBuf,
}

impl Default for DataPaths {
    fn default() -> Self {
        Self {
            cards: PathBuf::from(DEFAULT_CARDS_PATH),
            commanders: PathBuf::from(DEFAULT_COMMANDERS_PATH),
            decks: PathBuf::from(DEFAULT_DECKS_PATH),
            weights: PathBuf::from(DEFAULT_WEIGHTS_PATH),
        }
    }
}

impl DataPaths {
    /// Create new DataPaths with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override cards path.
    pub fn with_cards(mut self, path: impl Into<PathBuf>) -> Self {
        self.cards = path.into();
        self
    }

    /// Override commanders path.
    pub fn with_commanders(mut self, path: impl Into<PathBuf>) -> Self {
        self.commanders = path.into();
        self
    }

    /// Override decks path.
    pub fn with_decks(mut self, path: impl Into<PathBuf>) -> Self {
        self.decks = path.into();
        self
    }

    /// Override weights path.
    pub fn with_weights(mut self, path: impl Into<PathBuf>) -> Self {
        self.weights = path.into();
        self
    }

    /// Create from optional overrides, using defaults where not specified.
    pub fn from_overrides(
        cards: Option<&PathBuf>,
        commanders: Option<&PathBuf>,
        decks: Option<&PathBuf>,
        weights: Option<&PathBuf>,
    ) -> Self {
        Self {
            cards: cards.cloned().unwrap_or_else(|| PathBuf::from(DEFAULT_CARDS_PATH)),
            commanders: commanders.cloned().unwrap_or_else(|| PathBuf::from(DEFAULT_COMMANDERS_PATH)),
            decks: decks.cloned().unwrap_or_else(|| PathBuf::from(DEFAULT_DECKS_PATH)),
            weights: weights.cloned().unwrap_or_else(|| PathBuf::from(DEFAULT_WEIGHTS_PATH)),
        }
    }
}

// ============================================================================
// Execution Settings
// ============================================================================

/// Common execution settings.
///
/// Provides consistent handling of threads, progress, and seed across binaries.
#[derive(Debug, Clone)]
pub struct ExecutionSettings {
    /// Random seed for reproducibility.
    pub seed: u64,
    /// Number of threads (0 = use all cores).
    pub threads: usize,
    /// Whether to show progress indicators.
    pub show_progress: bool,
}

impl Default for ExecutionSettings {
    fn default() -> Self {
        Self {
            seed: 42,
            threads: 0,
            show_progress: false,
        }
    }
}

impl ExecutionSettings {
    /// Create with a specific seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Create with optional seed (resolves to time-based if None).
    pub fn with_optional_seed(mut self, seed: Option<u64>) -> Self {
        self.seed = resolve_seed(seed);
        self
    }

    /// Set thread count.
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.threads = threads;
        self
    }

    /// Enable or disable progress display.
    pub fn with_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }

    /// Configure the global thread pool based on these settings.
    ///
    /// Returns the actual number of threads configured.
    pub fn configure_threads(&self) -> usize {
        crate::execution::configure_thread_pool(self.threads)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bot_type_valid() {
        assert!(parse_bot_type("random").is_ok());
        assert!(parse_bot_type("greedy").is_ok());
        assert!(parse_bot_type("mcts").is_ok());
        assert!(parse_bot_type("alphabeta").is_ok());
        assert!(parse_bot_type("alpha-beta").is_ok());
        assert!(parse_bot_type("ab").is_ok());
        assert!(parse_bot_type("agent-argentum").is_ok());
        assert!(parse_bot_type("agent-generalist").is_ok());
    }

    #[test]
    fn test_parse_bot_type_invalid() {
        assert!(parse_bot_type("invalid").is_err());
        assert!(parse_bot_type("").is_err());
    }

    #[test]
    fn test_resolve_seed_with_value() {
        assert_eq!(resolve_seed(Some(12345)), 12345);
    }

    #[test]
    fn test_resolve_seed_without_value() {
        let seed = resolve_seed(None);
        // Should be a reasonable timestamp value
        assert!(seed > 1_000_000_000);
    }

    #[test]
    fn test_mcts_args_default() {
        let args = MctsArgs::default();
        assert_eq!(args.mcts_sims, 500);
        assert_eq!(args.mcts_trees, 1);
        assert_eq!(args.mcts_rollouts, 1);
    }

    #[test]
    fn test_mcts_args_to_config() {
        let args = MctsArgs {
            mcts_sims: 1000,
            mcts_trees: 4,
            mcts_rollouts: 2,
        };
        let config = args.to_config();
        assert_eq!(config.simulations, 1000);
        assert_eq!(config.parallel_trees, 4);
        assert_eq!(config.leaf_rollouts, 2);
    }

    #[test]
    fn test_alphabeta_args_default() {
        let args = AlphaBetaArgs::default();
        assert_eq!(args.ab_depth, 6);
    }

    #[test]
    fn test_data_paths_default() {
        let paths = DataPaths::default();
        assert_eq!(paths.cards, PathBuf::from("data/cards/core_set"));
        assert_eq!(paths.commanders, PathBuf::from("data/commanders"));
        assert_eq!(paths.decks, PathBuf::from("data/decks"));
        assert_eq!(paths.weights, PathBuf::from("data/weights"));
    }

    #[test]
    fn test_data_paths_builder() {
        let paths = DataPaths::new()
            .with_cards("custom/cards")
            .with_decks("custom/decks");
        assert_eq!(paths.cards, PathBuf::from("custom/cards"));
        assert_eq!(paths.decks, PathBuf::from("custom/decks"));
        // Others should remain default
        assert_eq!(paths.commanders, PathBuf::from("data/commanders"));
    }

    #[test]
    fn test_execution_settings_builder() {
        let settings = ExecutionSettings::default()
            .with_seed(999)
            .with_threads(4)
            .with_progress(true);
        assert_eq!(settings.seed, 999);
        assert_eq!(settings.threads, 4);
        assert!(settings.show_progress);
    }
}
