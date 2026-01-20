//! Headless mode statistics tracking.
//!
//! This module provides statistics collection for headless benchmark runs.

use std::time::{Duration, Instant};

use bevy::prelude::*;
use serde::Serialize;

/// Resource for tracking headless mode statistics across multiple games.
#[derive(Resource, Default)]
pub struct HeadlessStats {
    /// Number of games completed
    pub games_played: usize,
    /// Total games to play
    pub games_total: usize,
    /// Player 1 wins
    pub player1_wins: usize,
    /// Player 2 wins
    pub player2_wins: usize,
    /// Draws
    pub draws: usize,
    /// Total turns across all games
    pub total_turns: u32,
    /// Total actions across all games
    pub total_actions: u32,
    /// When the benchmark started
    pub start_time: Option<Instant>,
    /// Individual game durations
    pub game_times: Vec<Duration>,
    /// Start time of current game
    pub current_game_start: Option<Instant>,
}

impl HeadlessStats {
    /// Create new stats for a multi-game run.
    pub fn new(games_total: usize) -> Self {
        Self {
            games_total,
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    /// Record the start of a new game.
    pub fn start_game(&mut self) {
        self.current_game_start = Some(Instant::now());
    }

    /// Record a game result.
    pub fn record_game(&mut self, winner: Option<cardgame::types::PlayerId>, turns: u32, actions: u32) {
        self.games_played += 1;
        self.total_turns += turns;
        self.total_actions += actions;

        match winner {
            Some(p) if p == cardgame::types::PlayerId::PLAYER_ONE => self.player1_wins += 1,
            Some(p) if p == cardgame::types::PlayerId::PLAYER_TWO => self.player2_wins += 1,
            Some(_) => self.draws += 1, // Unexpected player ID, treat as draw
            None => self.draws += 1,
        }

        if let Some(start) = self.current_game_start.take() {
            self.game_times.push(start.elapsed());
        }
    }

    /// Check if all games have been played.
    pub fn is_complete(&self) -> bool {
        self.games_played >= self.games_total
    }

    /// Calculate total elapsed time.
    pub fn total_elapsed(&self) -> Duration {
        self.start_time.map(|t| t.elapsed()).unwrap_or_default()
    }

    /// Calculate average game duration.
    pub fn avg_game_duration(&self) -> Duration {
        if self.game_times.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.game_times.iter().sum();
        total / self.game_times.len() as u32
    }

    /// Calculate average turns per game.
    pub fn avg_turns(&self) -> f64 {
        if self.games_played == 0 {
            return 0.0;
        }
        self.total_turns as f64 / self.games_played as f64
    }

    /// Calculate average actions per game.
    pub fn avg_actions(&self) -> f64 {
        if self.games_played == 0 {
            return 0.0;
        }
        self.total_actions as f64 / self.games_played as f64
    }

    /// Calculate games per second.
    pub fn games_per_second(&self) -> f64 {
        let elapsed = self.total_elapsed().as_secs_f64();
        if elapsed == 0.0 {
            return 0.0;
        }
        self.games_played as f64 / elapsed
    }
}

/// JSON output format for benchmark results.
#[derive(Serialize)]
pub struct BenchmarkOutput {
    pub version: String,
    pub config: BenchmarkConfig,
    pub results: BenchmarkResults,
    pub timing: BenchmarkTiming,
    pub metrics: BenchmarkMetrics,
}

#[derive(Serialize)]
pub struct BenchmarkConfig {
    pub deck1: String,
    pub deck2: String,
    pub seed: u64,
    pub games: usize,
}

#[derive(Serialize)]
pub struct BenchmarkResults {
    pub player1_wins: usize,
    pub player2_wins: usize,
    pub draws: usize,
    pub player1_win_rate: f64,
    pub player2_win_rate: f64,
}

#[derive(Serialize)]
pub struct BenchmarkTiming {
    pub total_seconds: f64,
    pub avg_game_ms: f64,
    pub games_per_second: f64,
}

#[derive(Serialize)]
pub struct BenchmarkMetrics {
    pub avg_turns: f64,
    pub avg_actions: f64,
}

impl HeadlessStats {
    /// Generate JSON output for benchmark results.
    pub fn to_json_output(&self, deck1: &str, deck2: &str, seed: u64) -> BenchmarkOutput {
        let total_games = self.games_played.max(1) as f64;

        BenchmarkOutput {
            version: env!("CARGO_PKG_VERSION").to_string(),
            config: BenchmarkConfig {
                deck1: deck1.to_string(),
                deck2: deck2.to_string(),
                seed,
                games: self.games_total,
            },
            results: BenchmarkResults {
                player1_wins: self.player1_wins,
                player2_wins: self.player2_wins,
                draws: self.draws,
                player1_win_rate: self.player1_wins as f64 / total_games,
                player2_win_rate: self.player2_wins as f64 / total_games,
            },
            timing: BenchmarkTiming {
                total_seconds: self.total_elapsed().as_secs_f64(),
                avg_game_ms: self.avg_game_duration().as_secs_f64() * 1000.0,
                games_per_second: self.games_per_second(),
            },
            metrics: BenchmarkMetrics {
                avg_turns: self.avg_turns(),
                avg_actions: self.avg_actions(),
            },
        }
    }
}
