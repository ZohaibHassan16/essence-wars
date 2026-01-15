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
    /// P1/P2 diagnostic analysis.
    pub diagnostics: MatchupDiagnostics,
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
    /// P1/P2 diagnostic summary.
    pub p1_p2_diagnostics: P1P2Summary,
}

/// Summary of P1/P2 asymmetry analysis across all matchups.
#[derive(Debug, Clone, Serialize)]
pub struct P1P2Summary {
    /// Overall P1 win rate across all matchups.
    pub overall_p1_win_rate: f64,
    /// 95% confidence interval lower bound.
    pub overall_p1_ci_lower: f64,
    /// 95% confidence interval upper bound.
    pub overall_p1_ci_upper: f64,
    /// Significance level.
    pub significance: String,
    /// Balance assessment.
    pub assessment: String,
    /// Per-matchup P1 statistics.
    pub by_matchup: HashMap<String, MatchupP1Stats>,
}

/// P1 statistics for a single matchup.
#[derive(Debug, Clone, Serialize)]
pub struct MatchupP1Stats {
    /// P1 win rate for this matchup.
    pub p1_rate: f64,
    /// Significance level.
    pub significance: String,
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
    /// Aggregated diagnostic data for this direction.
    pub diagnostics: DirectionDiagnostics,
}

/// Aggregated diagnostic data for games in one direction.
#[derive(Debug, Clone, Default)]
pub struct DirectionDiagnostics {
    /// Number of games where P1 got first blood.
    pub p1_first_blood_count: u32,
    /// Number of games where P2 got first blood.
    pub p2_first_blood_count: u32,
    /// Sum of board advantage scores across all games.
    pub total_board_advantage: f64,
    /// Total turns where P1 was ahead.
    pub total_turns_p1_ahead: u32,
    /// Total turns where P2 was ahead.
    pub total_turns_p2_ahead: u32,
    /// Total turns even.
    pub total_turns_even: u32,
    /// Total turn count (for averaging).
    pub total_turn_count: u32,
    /// Total essence spent by P1.
    pub total_p1_essence: u32,
    /// Total essence spent by P2.
    pub total_p2_essence: u32,
    /// Total face damage by P1.
    pub total_p1_face_damage: u32,
    /// Total face damage by P2.
    pub total_p2_face_damage: u32,
    /// Total creatures killed by P1.
    pub total_p1_kills: u32,
    /// Total creatures killed by P2.
    pub total_p2_kills: u32,
    /// Total creatures lost by P1.
    pub total_p1_losses: u32,
    /// Total creatures lost by P2.
    pub total_p2_losses: u32,
    /// Game lengths for percentile calculation.
    pub game_lengths: Vec<u32>,
}

/// P1/P2 diagnostic analysis for a matchup.
#[derive(Debug, Clone, Default, Serialize)]
pub struct MatchupDiagnostics {
    // P1/P2 win rate with statistical analysis
    /// P1 win rate (excluding draws).
    pub p1_win_rate: f64,
    /// 95% confidence interval lower bound.
    pub p1_win_rate_ci_lower: f64,
    /// 95% confidence interval upper bound.
    pub p1_win_rate_ci_upper: f64,
    /// Chi-square statistic.
    pub p1_chi_square: f64,
    /// P-value for significance test.
    pub p1_p_value: f64,
    /// Significance level string.
    pub p1_significance: String,

    // First blood metrics
    /// P1 first blood rate.
    pub first_blood_p1_rate: f64,
    /// First blood CI lower bound.
    pub first_blood_p1_ci_lower: f64,
    /// First blood CI upper bound.
    pub first_blood_p1_ci_upper: f64,

    // Board advantage
    /// Average board advantage score (positive = P1 ahead).
    pub avg_board_advantage: f64,
    /// Percentage of turns P1 was ahead.
    pub p1_ahead_pct: f64,
    /// Percentage of turns P2 was ahead.
    pub p2_ahead_pct: f64,
    /// Percentage of turns even.
    pub even_pct: f64,

    // Resource efficiency
    /// Average essence spent per game by P1.
    pub p1_essence_avg: f64,
    /// Average essence spent per game by P2.
    pub p2_essence_avg: f64,

    // Combat efficiency
    /// Average face damage per game by P1.
    pub p1_face_damage_avg: f64,
    /// Average face damage per game by P2.
    pub p2_face_damage_avg: f64,
    /// P1 trade ratio (kills / losses).
    pub p1_trade_ratio: f64,
    /// P2 trade ratio (kills / losses).
    pub p2_trade_ratio: f64,

    // Game length percentiles
    /// 10th percentile game length.
    pub game_length_p10: u32,
    /// 50th percentile game length (median).
    pub game_length_p50: u32,
    /// 90th percentile game length.
    pub game_length_p90: u32,
}

impl MatchupDiagnostics {
    /// Build matchup diagnostics from two direction results.
    pub fn from_directions(
        dir1: &DirectionDiagnostics,
        dir2: &DirectionDiagnostics,
        total_games: u32,
    ) -> Self {
        use crate::diagnostics::statistics::{percentile, wilson_score_interval};

        // Combine data from both directions
        let p1_first_blood = dir1.p1_first_blood_count + dir2.p1_first_blood_count;
        let p2_first_blood = dir1.p2_first_blood_count + dir2.p2_first_blood_count;
        let total_first_blood = p1_first_blood + p2_first_blood;

        let total_board_advantage = dir1.total_board_advantage + dir2.total_board_advantage;
        let total_turns_p1_ahead = dir1.total_turns_p1_ahead + dir2.total_turns_p1_ahead;
        let total_turns_p2_ahead = dir1.total_turns_p2_ahead + dir2.total_turns_p2_ahead;
        let total_turns_even = dir1.total_turns_even + dir2.total_turns_even;
        let total_turn_count = dir1.total_turn_count + dir2.total_turn_count;

        let total_p1_essence = dir1.total_p1_essence + dir2.total_p1_essence;
        let total_p2_essence = dir1.total_p2_essence + dir2.total_p2_essence;
        let total_p1_face_damage = dir1.total_p1_face_damage + dir2.total_p1_face_damage;
        let total_p2_face_damage = dir1.total_p2_face_damage + dir2.total_p2_face_damage;
        let total_p1_kills = dir1.total_p1_kills + dir2.total_p1_kills;
        let total_p2_kills = dir1.total_p2_kills + dir2.total_p2_kills;
        let total_p1_losses = dir1.total_p1_losses + dir2.total_p1_losses;
        let total_p2_losses = dir1.total_p2_losses + dir2.total_p2_losses;

        // Combine game lengths and sort for percentiles
        let mut all_lengths: Vec<f64> = dir1
            .game_lengths
            .iter()
            .chain(dir2.game_lengths.iter())
            .map(|&x| x as f64)
            .collect();
        all_lengths.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // P1 win rate calculation - we need p1_wins from parent context
        // For now, use first blood as proxy or calculate from direction totals
        // This will be properly calculated in run_matchup
        let p1_win_rate = 0.5; // Placeholder - filled in by caller
        let (p1_ci_lower, p1_ci_upper) = (0.0, 1.0); // Placeholder
        let (p1_chi, p1_p) = (0.0, 1.0); // Placeholder

        // First blood statistics
        let (fb_ci_lower, fb_ci_upper) = if total_first_blood > 0 {
            wilson_score_interval(p1_first_blood as usize, total_first_blood as usize, 0.95)
        } else {
            (0.0, 1.0)
        };
        let first_blood_p1_rate = if total_first_blood > 0 {
            p1_first_blood as f64 / total_first_blood as f64
        } else {
            0.5
        };

        // Board advantage
        let avg_board_advantage = if total_turn_count > 0 {
            total_board_advantage / total_turn_count as f64
        } else {
            0.0
        };
        let total_turns_tracked = total_turns_p1_ahead + total_turns_p2_ahead + total_turns_even;
        let (p1_ahead_pct, p2_ahead_pct, even_pct) = if total_turns_tracked > 0 {
            (
                total_turns_p1_ahead as f64 / total_turns_tracked as f64,
                total_turns_p2_ahead as f64 / total_turns_tracked as f64,
                total_turns_even as f64 / total_turns_tracked as f64,
            )
        } else {
            (0.0, 0.0, 1.0)
        };

        // Resource efficiency
        let games_f64 = total_games as f64;
        let p1_essence_avg = if total_games > 0 {
            total_p1_essence as f64 / games_f64
        } else {
            0.0
        };
        let p2_essence_avg = if total_games > 0 {
            total_p2_essence as f64 / games_f64
        } else {
            0.0
        };

        // Combat efficiency
        let p1_face_damage_avg = if total_games > 0 {
            total_p1_face_damage as f64 / games_f64
        } else {
            0.0
        };
        let p2_face_damage_avg = if total_games > 0 {
            total_p2_face_damage as f64 / games_f64
        } else {
            0.0
        };
        let p1_trade_ratio = if total_p1_losses > 0 {
            total_p1_kills as f64 / total_p1_losses as f64
        } else if total_p1_kills > 0 {
            f64::INFINITY
        } else {
            1.0
        };
        let p2_trade_ratio = if total_p2_losses > 0 {
            total_p2_kills as f64 / total_p2_losses as f64
        } else if total_p2_kills > 0 {
            f64::INFINITY
        } else {
            1.0
        };

        // Game length percentiles (percentile function expects 0.0-1.0)
        let game_length_p10 = percentile(&all_lengths, 0.10).unwrap_or(0.0) as u32;
        let game_length_p50 = percentile(&all_lengths, 0.50).unwrap_or(0.0) as u32;
        let game_length_p90 = percentile(&all_lengths, 0.90).unwrap_or(0.0) as u32;

        Self {
            p1_win_rate,
            p1_win_rate_ci_lower: p1_ci_lower,
            p1_win_rate_ci_upper: p1_ci_upper,
            p1_chi_square: p1_chi,
            p1_p_value: p1_p,
            p1_significance: "not_calculated".to_string(),
            first_blood_p1_rate,
            first_blood_p1_ci_lower: fb_ci_lower,
            first_blood_p1_ci_upper: fb_ci_upper,
            avg_board_advantage,
            p1_ahead_pct,
            p2_ahead_pct,
            even_pct,
            p1_essence_avg,
            p2_essence_avg,
            p1_face_damage_avg,
            p2_face_damage_avg,
            p1_trade_ratio,
            p2_trade_ratio,
            game_length_p10,
            game_length_p50,
            game_length_p90,
        }
    }

    /// Update with P1 win rate statistics (called after combining directions).
    pub fn with_p1_stats(mut self, p1_wins: u32, total_decisive: u32) -> Self {
        use crate::diagnostics::statistics::{chi_square_test, wilson_score_interval};

        if total_decisive > 0 {
            self.p1_win_rate = p1_wins as f64 / total_decisive as f64;
            let (ci_lower, ci_upper) =
                wilson_score_interval(p1_wins as usize, total_decisive as usize, 0.95);
            self.p1_win_rate_ci_lower = ci_lower;
            self.p1_win_rate_ci_upper = ci_upper;

            let (chi, p_value) =
                chi_square_test(p1_wins as usize, total_decisive as usize, 0.5);
            self.p1_chi_square = chi;
            self.p1_p_value = p_value;

            self.p1_significance = if p_value < 0.01 {
                "highly_significant".to_string()
            } else if p_value < 0.05 {
                "significant".to_string()
            } else if p_value < 0.10 {
                "marginal".to_string()
            } else {
                "not_significant".to_string()
            };
        }
        self
    }
}
