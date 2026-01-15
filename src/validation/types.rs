//! Validation data types.
//!
//! Contains all the data structures used for balance validation,
//! including matchup results, balance status, and validation configuration.

use std::collections::HashMap;

use serde::Serialize;

use crate::bots::BotWeights;
use crate::decks::Faction;
use crate::types::CardId;
use crate::version::VersionInfo;

/// Configuration for a validation run.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationConfig {
    /// Number of games per matchup per player order.
    pub games_per_matchup: usize,
    /// MCTS simulations per move.
    pub mcts_simulations: u32,
    /// Random seed for reproducibility.
    pub seed: u64,
    /// Number of threads used.
    pub threads: usize,
    /// Optional matchup filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matchup_filter: Option<String>,
}

impl ValidationConfig {
    /// Create a new validation configuration.
    pub fn new(games_per_matchup: usize, mcts_simulations: u32, seed: u64, threads: usize) -> Self {
        Self {
            games_per_matchup,
            mcts_simulations,
            seed,
            threads,
            matchup_filter: None,
        }
    }

    /// Set matchup filter.
    pub fn with_matchup_filter(mut self, filter: Option<String>) -> Self {
        self.matchup_filter = filter;
        self
    }
}

/// Results for a single faction pair (both player orders).
#[derive(Debug, Clone, Serialize)]
pub struct MatchupResult {
    /// First faction name.
    pub faction1: String,
    /// Second faction name.
    pub faction2: String,
    /// Deck ID used for faction 1.
    pub deck1_id: String,
    /// Deck ID used for faction 2.
    pub deck2_id: String,
    /// Faction 1 wins when playing as Player 1.
    pub f1_as_p1_wins: u32,
    /// Games played with faction 1 as Player 1.
    pub f1_as_p1_games: u32,
    /// Faction 1 wins when playing as Player 2.
    pub f1_as_p2_wins: u32,
    /// Games played with faction 1 as Player 2.
    pub f1_as_p2_games: u32,
    /// Total wins for faction 1.
    pub faction1_total_wins: u32,
    /// Total wins for faction 2.
    pub faction2_total_wins: u32,
    /// Total draws.
    pub draws: u32,
    /// Total games played.
    pub total_games: u32,
    /// Win rate for faction 1.
    pub faction1_win_rate: f64,
    /// Win rate for faction 2.
    pub faction2_win_rate: f64,
    /// Average turns per game.
    pub avg_turns: f64,
    /// Total time in seconds.
    pub total_time_secs: f64,
}

/// Balance status classification.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BalanceStatus {
    /// All metrics within acceptable range.
    Balanced,
    /// Some metrics outside ideal range but not critical.
    Warning,
    /// Significant imbalance detected.
    Imbalanced,
}

impl std::fmt::Display for BalanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalanceStatus::Balanced => write!(f, "BALANCED"),
            BalanceStatus::Warning => write!(f, "WARNING"),
            BalanceStatus::Imbalanced => write!(f, "IMBALANCED"),
        }
    }
}

/// Balance analysis summary.
#[derive(Debug, Clone, Serialize)]
pub struct BalanceSummary {
    /// Player 1 overall win rate.
    pub p1_win_rate: f64,
    /// Player 1 balance status.
    pub p1_status: BalanceStatus,
    /// Win rate for each faction.
    pub faction_win_rates: HashMap<String, f64>,
    /// Maximum difference between faction win rates.
    pub max_faction_delta: f64,
    /// Faction balance status.
    pub faction_status: BalanceStatus,
    /// Overall balance status (worst of P1 and faction).
    pub overall_status: BalanceStatus,
    /// Warning messages.
    pub warnings: Vec<String>,
}

/// Complete validation results (for JSON output).
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResults {
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Engine version info.
    pub version: VersionInfo,
    /// Validation configuration.
    pub config: ValidationConfig,
    /// Results for each matchup.
    pub matchups: Vec<MatchupResult>,
    /// Balance analysis summary.
    pub summary: BalanceSummary,
}

/// A matchup definition for testing.
#[derive(Debug, Clone)]
pub struct MatchupDefinition {
    /// First faction.
    pub faction1: Faction,
    /// Second faction.
    pub faction2: Faction,
    /// Deck ID for faction 1.
    pub deck1_id: String,
    /// Cards in deck 1.
    pub deck1_cards: Vec<CardId>,
    /// Deck ID for faction 2.
    pub deck2_id: String,
    /// Cards in deck 2.
    pub deck2_cards: Vec<CardId>,
}

/// Holder for faction-specific weights.
#[derive(Debug, Default)]
pub struct FactionWeights {
    /// Argentum faction weights.
    pub argentum: Option<BotWeights>,
    /// Symbiote faction weights.
    pub symbiote: Option<BotWeights>,
    /// Obsidion faction weights.
    pub obsidion: Option<BotWeights>,
}

impl FactionWeights {
    /// Create empty faction weights.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get weights for a specific faction.
    pub fn get(&self, faction: Faction) -> Option<&BotWeights> {
        match faction {
            Faction::Argentum => self.argentum.as_ref(),
            Faction::Symbiote => self.symbiote.as_ref(),
            Faction::Obsidion => self.obsidion.as_ref(),
            Faction::Neutral => None,
        }
    }

    /// Set weights for a specific faction.
    pub fn set(&mut self, faction: Faction, weights: Option<BotWeights>) {
        match faction {
            Faction::Argentum => self.argentum = weights,
            Faction::Symbiote => self.symbiote = weights,
            Faction::Obsidion => self.obsidion = weights,
            Faction::Neutral => {}
        }
    }

    /// Load faction weights from a directory.
    ///
    /// Looks for `specialists/{faction}.toml` files.
    pub fn load_from_directory(weights_dir: &std::path::Path, quiet: bool) -> Self {
        let specialists_dir = weights_dir.join("specialists");
        let mut weights = Self::new();

        for (faction, name) in [
            (Faction::Argentum, "argentum"),
            (Faction::Symbiote, "symbiote"),
            (Faction::Obsidion, "obsidion"),
        ] {
            let path = specialists_dir.join(format!("{}.toml", name));
            match BotWeights::load(&path) {
                Ok(w) => {
                    if !quiet {
                        println!("Loaded {} weights: {}", name, w.name);
                    }
                    weights.set(faction, Some(w));
                }
                Err(_) => {
                    if !quiet {
                        println!("Note: {} using default weights", name);
                    }
                }
            }
        }

        weights
    }
}

/// Raw game results for aggregation.
#[derive(Debug, Clone)]
pub struct DirectionResults {
    /// Player 1 wins.
    pub p1_wins: u32,
    /// Total turns across all games.
    pub total_turns: u32,
    /// Number of draws.
    pub draws: u32,
    /// Total games played.
    pub games: u32,
    /// Time taken.
    pub duration_secs: f64,
}
