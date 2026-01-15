//! Parallel batch execution utilities for game matches.
//!
//! Provides reusable infrastructure for running games in parallel
//! with progress reporting.

use rayon::prelude::*;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::core::PlayerId;

use super::progress::{ProgressReporter, ProgressStyle};
use super::seeds::GameSeeds;

/// Outcome of a single game.
#[derive(Clone, Debug)]
pub struct GameOutcome {
    /// Winner of the game (None for draw)
    pub winner: Option<PlayerId>,
    /// Number of turns played
    pub turns: u32,
    /// Duration of the game
    pub duration: Duration,
}

impl GameOutcome {
    /// Create a new game outcome.
    pub fn new(winner: Option<PlayerId>, turns: u32, duration: Duration) -> Self {
        Self {
            winner,
            turns,
            duration,
        }
    }
}

/// Configuration for batch game execution.
#[derive(Clone, Debug)]
pub struct BatchConfig {
    /// Number of games to run
    pub games: usize,
    /// Base seed for deterministic execution
    pub base_seed: u64,
    /// Whether to show progress indicator
    pub show_progress: bool,
    /// Progress style (Simple or Rich)
    pub progress_style: ProgressStyle,
    /// Optional prefix for progress output
    pub progress_prefix: String,
}

impl BatchConfig {
    /// Create a new batch configuration.
    pub fn new(games: usize, base_seed: u64) -> Self {
        Self {
            games,
            base_seed,
            show_progress: false,
            progress_style: ProgressStyle::Rich,
            progress_prefix: String::new(),
        }
    }

    /// Enable progress reporting with the specified style.
    pub fn with_progress(mut self, style: ProgressStyle) -> Self {
        self.show_progress = true;
        self.progress_style = style;
        self
    }

    /// Set a prefix for progress output.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.progress_prefix = prefix.into();
        self
    }
}

/// Result of a batch execution.
#[derive(Debug)]
pub struct BatchResult {
    /// Individual game outcomes
    pub outcomes: Vec<GameOutcome>,
    /// Total wall-clock time for the batch
    pub wall_clock_time: Duration,
}

impl BatchResult {
    /// Count wins for Player 1.
    pub fn p1_wins(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| o.winner == Some(PlayerId::PLAYER_ONE))
            .count()
    }

    /// Count wins for Player 2.
    pub fn p2_wins(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| o.winner == Some(PlayerId::PLAYER_TWO))
            .count()
    }

    /// Count draws.
    pub fn draws(&self) -> usize {
        self.outcomes.iter().filter(|o| o.winner.is_none()).count()
    }

    /// Total number of games.
    pub fn total_games(&self) -> usize {
        self.outcomes.len()
    }

    /// Total turns across all games.
    pub fn total_turns(&self) -> u32 {
        self.outcomes.iter().map(|o| o.turns).sum()
    }

    /// Average turns per game.
    pub fn avg_turns(&self) -> f64 {
        if self.outcomes.is_empty() {
            0.0
        } else {
            self.total_turns() as f64 / self.outcomes.len() as f64
        }
    }

    /// Total CPU time (sum of individual game durations).
    pub fn cpu_time(&self) -> Duration {
        self.outcomes.iter().map(|o| o.duration).sum()
    }
}

/// Run a batch of games in parallel.
///
/// # Arguments
/// * `config` - Batch execution configuration
/// * `run_game` - Closure that takes `GameSeeds` and returns `GameOutcome`
///
/// # Returns
/// A `BatchResult` containing all outcomes and timing information.
///
/// # Example
/// ```ignore
/// let config = BatchConfig::new(100, 42).with_progress(ProgressStyle::Rich);
/// let result = run_batch_parallel(&config, |seeds| {
///     // Run game with seeds.game, seeds.bot1, seeds.bot2
///     GameOutcome::new(Some(PlayerId::PLAYER_ONE), 25, Duration::from_millis(50))
/// });
/// ```
pub fn run_batch_parallel<F>(config: &BatchConfig, run_game: F) -> BatchResult
where
    F: Fn(GameSeeds) -> GameOutcome + Send + Sync,
{
    let start_time = Instant::now();

    // Set up progress reporting
    let progress = if config.show_progress {
        Some(
            ProgressReporter::new(config.games)
                .with_style(config.progress_style)
                .with_prefix(config.progress_prefix.clone())
                .start(),
        )
    } else {
        None
    };

    let counter = progress.as_ref().map(|p| p.counter());

    // Run games in parallel
    let outcomes: Vec<GameOutcome> = (0..config.games)
        .into_par_iter()
        .map(|i| {
            let seeds = GameSeeds::for_game(config.base_seed, i);
            let outcome = run_game(seeds);
            if let Some(ref c) = counter {
                c.fetch_add(1, Ordering::Relaxed);
            }
            outcome
        })
        .collect();

    // Finish progress reporting
    if let Some(p) = progress {
        p.finish();
    }

    BatchResult {
        outcomes,
        wall_clock_time: start_time.elapsed(),
    }
}

/// Run a batch of games in parallel for a specific matchup.
///
/// Similar to `run_batch_parallel` but uses matchup-aware seed derivation.
///
/// # Arguments
/// * `config` - Batch execution configuration
/// * `matchup_index` - Index of this matchup (for seed derivation)
/// * `reverse_direction` - Whether this is the reverse direction
/// * `run_game` - Closure that takes `GameSeeds` and returns `GameOutcome`
pub fn run_batch_parallel_matchup<F>(
    config: &BatchConfig,
    matchup_index: usize,
    reverse_direction: bool,
    run_game: F,
) -> BatchResult
where
    F: Fn(GameSeeds) -> GameOutcome + Send + Sync,
{
    let start_time = Instant::now();

    // Set up progress reporting
    let progress = if config.show_progress {
        Some(
            ProgressReporter::new(config.games)
                .with_style(config.progress_style)
                .with_prefix(config.progress_prefix.clone())
                .start(),
        )
    } else {
        None
    };

    let counter = progress.as_ref().map(|p| p.counter());

    // Run games in parallel
    let outcomes: Vec<GameOutcome> = (0..config.games)
        .into_par_iter()
        .map(|i| {
            let seeds =
                GameSeeds::for_matchup(config.base_seed, matchup_index, i, reverse_direction);
            let outcome = run_game(seeds);
            if let Some(ref c) = counter {
                c.fetch_add(1, Ordering::Relaxed);
            }
            outcome
        })
        .collect();

    // Finish progress reporting
    if let Some(p) = progress {
        p.finish();
    }

    BatchResult {
        outcomes,
        wall_clock_time: start_time.elapsed(),
    }
}

/// Configure the global rayon thread pool.
///
/// # Arguments
/// * `threads` - Number of threads (0 = use rayon's default)
///
/// # Returns
/// The actual number of threads configured.
pub fn configure_thread_pool(threads: usize) -> usize {
    if threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .ok(); // Ignore if already initialized
        threads
    } else {
        rayon::current_num_threads()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_builder() {
        let config = BatchConfig::new(100, 42);
        assert_eq!(config.games, 100);
        assert_eq!(config.base_seed, 42);
        assert!(!config.show_progress);

        let config = config.with_progress(ProgressStyle::Simple).with_prefix("  ");
        assert!(config.show_progress);
        assert_eq!(config.progress_style, ProgressStyle::Simple);
        assert_eq!(config.progress_prefix, "  ");
    }

    #[test]
    fn test_game_outcome() {
        let outcome = GameOutcome::new(Some(PlayerId::PLAYER_ONE), 25, Duration::from_secs(1));
        assert_eq!(outcome.winner, Some(PlayerId::PLAYER_ONE));
        assert_eq!(outcome.turns, 25);
        assert_eq!(outcome.duration, Duration::from_secs(1));
    }

    #[test]
    fn test_batch_result_stats() {
        let outcomes = vec![
            GameOutcome::new(Some(PlayerId::PLAYER_ONE), 20, Duration::from_millis(100)),
            GameOutcome::new(Some(PlayerId::PLAYER_TWO), 25, Duration::from_millis(150)),
            GameOutcome::new(None, 30, Duration::from_millis(200)),
            GameOutcome::new(Some(PlayerId::PLAYER_ONE), 22, Duration::from_millis(120)),
        ];

        let result = BatchResult {
            outcomes,
            wall_clock_time: Duration::from_millis(200),
        };

        assert_eq!(result.total_games(), 4);
        assert_eq!(result.p1_wins(), 2);
        assert_eq!(result.p2_wins(), 1);
        assert_eq!(result.draws(), 1);
        assert_eq!(result.total_turns(), 97);
        assert!((result.avg_turns() - 24.25).abs() < 0.01);
        assert_eq!(result.cpu_time(), Duration::from_millis(570));
    }

    #[test]
    fn test_run_batch_parallel() {
        let config = BatchConfig::new(10, 42);
        let result = run_batch_parallel(&config, |seeds| {
            // Simple mock game - player 1 wins if game seed is even
            let winner = if seeds.game % 2 == 0 {
                Some(PlayerId::PLAYER_ONE)
            } else {
                Some(PlayerId::PLAYER_TWO)
            };
            GameOutcome::new(winner, 20, Duration::from_millis(10))
        });

        assert_eq!(result.total_games(), 10);
        // Seeds 42, 44, 46, 48, 50 are even (5 games)
        // Seeds 43, 45, 47, 49, 51 are odd (5 games)
        assert_eq!(result.p1_wins(), 5);
        assert_eq!(result.p2_wins(), 5);
    }

    #[test]
    fn test_configure_thread_pool_default() {
        let threads = configure_thread_pool(0);
        assert!(threads > 0);
    }
}
