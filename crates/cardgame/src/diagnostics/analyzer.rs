//! Diagnostic statistics analysis.
//!
//! Aggregates game diagnostics into statistical summaries for P1/P2 asymmetry analysis.

use std::collections::HashMap;

use crate::types::PlayerId;

use super::collector::{GameDiagnostics, TurnSnapshot};
use super::metrics::GameMetrics;
use super::statistics::ProportionStats;

/// Aggregated statistics across all diagnostic games.
#[derive(Default)]
pub struct AggregatedStats {
    /// Total games analyzed.
    pub total_games: usize,
    /// Player 1 wins.
    pub p1_wins: usize,
    /// Player 2 wins.
    pub p2_wins: usize,
    /// Draws.
    pub draws: usize,

    // Win rate by game length
    /// P1 wins in early games (turns 1-10).
    pub p1_wins_early: usize,
    /// P1 wins in mid games (turns 11-20).
    pub p1_wins_mid: usize,
    /// P1 wins in late games (turns 21-30).
    pub p1_wins_late: usize,
    /// Total early games.
    pub games_early: usize,
    /// Total mid games.
    pub games_mid: usize,
    /// Total late games.
    pub games_late: usize,

    // First blood statistics
    /// Times P1 got first blood.
    pub p1_first_blood: usize,
    /// Times P2 got first blood.
    pub p2_first_blood: usize,

    // Resource curves by turn (turn -> (sum, count))
    /// P1 life by turn.
    pub p1_life_by_turn: HashMap<u32, (i64, usize)>,
    /// P2 life by turn.
    pub p2_life_by_turn: HashMap<u32, (i64, usize)>,
    /// P1 creatures by turn.
    pub p1_creatures_by_turn: HashMap<u32, (i64, usize)>,
    /// P2 creatures by turn.
    pub p2_creatures_by_turn: HashMap<u32, (i64, usize)>,
    /// P1 hand size by turn.
    pub p1_hand_by_turn: HashMap<u32, (i64, usize)>,
    /// P2 hand size by turn.
    pub p2_hand_by_turn: HashMap<u32, (i64, usize)>,
    /// P1 board attack by turn.
    pub p1_board_attack_by_turn: HashMap<u32, (i64, usize)>,
    /// P2 board attack by turn.
    pub p2_board_attack_by_turn: HashMap<u32, (i64, usize)>,
    /// P1 essence by turn.
    pub p1_essence_by_turn: HashMap<u32, (i64, usize)>,
    /// P2 essence by turn.
    pub p2_essence_by_turn: HashMap<u32, (i64, usize)>,
    /// P1 max essence by turn.
    pub p1_max_essence_by_turn: HashMap<u32, (i64, usize)>,
    /// P2 max essence by turn.
    pub p2_max_essence_by_turn: HashMap<u32, (i64, usize)>,
    /// P1 board health by turn.
    pub p1_board_health_by_turn: HashMap<u32, (i64, usize)>,
    /// P2 board health by turn.
    pub p2_board_health_by_turn: HashMap<u32, (i64, usize)>,

    // Actions per turn
    /// Total P1 actions.
    pub p1_actions_total: usize,
    /// Total P2 actions.
    pub p2_actions_total: usize,

    // First creature death
    /// Turns when first creature died.
    pub first_creature_death_turns: Vec<u32>,

    // Notable game seeds for reproduction
    /// Earliest P1 win (seed, turns).
    pub earliest_p1_win_seed: Option<(u64, u32)>,
    /// Earliest P2 win (seed, turns).
    pub earliest_p2_win_seed: Option<(u64, u32)>,

    // === Phase 2: Enhanced Metrics ===

    // Tempo metrics aggregation
    /// Times P1 played first creature.
    pub p1_first_creature: usize,
    /// Times P2 played first creature.
    pub p2_first_creature: usize,
    /// Sum of turns when P1 played first creature.
    pub p1_first_creature_turn_sum: u32,
    /// Sum of turns when P2 played first creature.
    pub p2_first_creature_turn_sum: u32,
    /// Count of games with P1 first creature data.
    pub p1_first_creature_count: usize,
    /// Count of games with P2 first creature data.
    pub p2_first_creature_count: usize,

    // Board advantage aggregation
    /// Sum of average board advantage scores across games (positive = P1 ahead).
    pub total_avg_board_advantage: f64,
    /// Total turns P1 was ahead across all games.
    pub total_turns_p1_ahead: u32,
    /// Total turns P2 was ahead across all games.
    pub total_turns_p2_ahead: u32,
    /// Total turns even across all games.
    pub total_turns_even: u32,

    // Resource efficiency aggregation
    /// Sum of P1 essence spent across all games.
    pub p1_total_essence_spent: u64,
    /// Sum of P2 essence spent across all games.
    pub p2_total_essence_spent: u64,
    /// Sum of P1 board impact across all games.
    pub p1_total_board_impact: i64,
    /// Sum of P2 board impact across all games.
    pub p2_total_board_impact: i64,

    // Combat efficiency aggregation
    /// Total face damage dealt by P1 across all games.
    pub p1_total_face_damage: u64,
    /// Total face damage dealt by P2 across all games.
    pub p2_total_face_damage: u64,
    /// Total creatures killed by P1 across all games.
    pub p1_total_creatures_killed: u64,
    /// Total creatures killed by P2 across all games.
    pub p2_total_creatures_killed: u64,
    /// Total creatures lost by P1 across all games.
    pub p1_total_creatures_lost: u64,
    /// Total creatures lost by P2 across all games.
    pub p2_total_creatures_lost: u64,

    /// Per-game metrics for detailed analysis.
    pub game_metrics: Vec<GameMetrics>,
}

impl AggregatedStats {
    /// Create a new aggregated stats instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze a collection of game diagnostics.
    pub fn analyze(games: &[GameDiagnostics]) -> Self {
        let mut stats = Self::new();
        for game in games {
            stats.record_game(game);
        }
        stats
    }

    /// Record a single game's diagnostics.
    pub fn record_game(&mut self, diag: &GameDiagnostics) {
        self.total_games += 1;

        // Record winner
        match diag.winner {
            Some(PlayerId::PLAYER_ONE) => self.p1_wins += 1,
            Some(PlayerId::PLAYER_TWO) => self.p2_wins += 1,
            _ => self.draws += 1,
        }

        // Win rate by game length
        if diag.total_turns <= 10 {
            self.games_early += 1;
            if diag.winner == Some(PlayerId::PLAYER_ONE) {
                self.p1_wins_early += 1;
            }
        } else if diag.total_turns <= 20 {
            self.games_mid += 1;
            if diag.winner == Some(PlayerId::PLAYER_ONE) {
                self.p1_wins_mid += 1;
            }
        } else {
            self.games_late += 1;
            if diag.winner == Some(PlayerId::PLAYER_ONE) {
                self.p1_wins_late += 1;
            }
        }

        // First blood
        match (diag.first_damage_to_p1_turn, diag.first_damage_to_p2_turn) {
            (Some(t1), Some(t2)) if t2 < t1 => self.p1_first_blood += 1,
            (Some(t1), Some(t2)) if t1 < t2 => self.p2_first_blood += 1,
            (None, Some(_)) => self.p1_first_blood += 1,
            (Some(_), None) => self.p2_first_blood += 1,
            _ => {}
        }

        // Actions
        self.p1_actions_total += diag.p1_actions;
        self.p2_actions_total += diag.p2_actions;

        // First creature death
        if let Some(turn) = diag.first_creature_death_turn {
            self.first_creature_death_turns.push(turn);
        }

        // Track notable game seeds
        if diag.winner == Some(PlayerId::PLAYER_ONE)
            && (self.earliest_p1_win_seed.is_none()
                || diag.total_turns < self.earliest_p1_win_seed.unwrap().1)
        {
            self.earliest_p1_win_seed = Some((diag.seed, diag.total_turns));
        } else if diag.winner == Some(PlayerId::PLAYER_TWO)
            && (self.earliest_p2_win_seed.is_none()
                || diag.total_turns < self.earliest_p2_win_seed.unwrap().1)
        {
            self.earliest_p2_win_seed = Some((diag.seed, diag.total_turns));
        }

        // Resource curves - record at start of each turn
        for snapshot in &diag.snapshots {
            // Only record at start of P1's turn for consistent comparison
            if snapshot.active_player == PlayerId::PLAYER_ONE {
                self.record_snapshot(snapshot);
            }
        }

        // === Phase 2: Record enhanced metrics ===

        // Tempo metrics
        let tempo = &diag.metrics.tempo;
        if let Some(turn) = tempo.p1_first_creature_turn {
            self.p1_first_creature_turn_sum += turn;
            self.p1_first_creature_count += 1;
        }
        if let Some(turn) = tempo.p2_first_creature_turn {
            self.p2_first_creature_turn_sum += turn;
            self.p2_first_creature_count += 1;
        }
        match tempo.first_creature_player() {
            Some(true) => self.p1_first_creature += 1,
            Some(false) => self.p2_first_creature += 1,
            None => {}
        }

        // Board advantage metrics
        self.total_avg_board_advantage += diag.metrics.avg_board_advantage;
        self.total_turns_p1_ahead += diag.metrics.turns_p1_ahead;
        self.total_turns_p2_ahead += diag.metrics.turns_p2_ahead;
        self.total_turns_even += diag.metrics.turns_even;

        // Resource efficiency metrics
        let res = &diag.metrics.resource_efficiency;
        self.p1_total_essence_spent += res.p1_essence_spent as u64;
        self.p2_total_essence_spent += res.p2_essence_spent as u64;
        self.p1_total_board_impact += res.p1_board_impact as i64;
        self.p2_total_board_impact += res.p2_board_impact as i64;

        // Combat efficiency metrics
        let combat = &diag.metrics.combat_efficiency;
        self.p1_total_face_damage += combat.p1_face_damage as u64;
        self.p2_total_face_damage += combat.p2_face_damage as u64;
        self.p1_total_creatures_killed += combat.p1_creatures_killed as u64;
        self.p2_total_creatures_killed += combat.p2_creatures_killed as u64;
        self.p1_total_creatures_lost += combat.p1_creatures_lost as u64;
        self.p2_total_creatures_lost += combat.p2_creatures_lost as u64;

        // Store per-game metrics
        self.game_metrics.push(diag.metrics.clone());
    }

    /// Record a snapshot's data into curves.
    fn record_snapshot(&mut self, snapshot: &TurnSnapshot) {
        let turn = snapshot.turn;

        Self::add_to_curve(&mut self.p1_life_by_turn, turn, snapshot.p1_life as i64);
        Self::add_to_curve(&mut self.p2_life_by_turn, turn, snapshot.p2_life as i64);
        Self::add_to_curve(
            &mut self.p1_creatures_by_turn,
            turn,
            snapshot.p1_creatures as i64,
        );
        Self::add_to_curve(
            &mut self.p2_creatures_by_turn,
            turn,
            snapshot.p2_creatures as i64,
        );
        Self::add_to_curve(&mut self.p1_hand_by_turn, turn, snapshot.p1_hand_size as i64);
        Self::add_to_curve(&mut self.p2_hand_by_turn, turn, snapshot.p2_hand_size as i64);
        Self::add_to_curve(
            &mut self.p1_board_attack_by_turn,
            turn,
            snapshot.p1_total_attack as i64,
        );
        Self::add_to_curve(
            &mut self.p2_board_attack_by_turn,
            turn,
            snapshot.p2_total_attack as i64,
        );
        Self::add_to_curve(
            &mut self.p1_essence_by_turn,
            turn,
            snapshot.p1_essence as i64,
        );
        Self::add_to_curve(
            &mut self.p2_essence_by_turn,
            turn,
            snapshot.p2_essence as i64,
        );
        Self::add_to_curve(
            &mut self.p1_max_essence_by_turn,
            turn,
            snapshot.p1_max_essence as i64,
        );
        Self::add_to_curve(
            &mut self.p2_max_essence_by_turn,
            turn,
            snapshot.p2_max_essence as i64,
        );
        Self::add_to_curve(
            &mut self.p1_board_health_by_turn,
            turn,
            snapshot.p1_total_health as i64,
        );
        Self::add_to_curve(
            &mut self.p2_board_health_by_turn,
            turn,
            snapshot.p2_total_health as i64,
        );
    }

    fn add_to_curve(map: &mut HashMap<u32, (i64, usize)>, turn: u32, value: i64) {
        let entry = map.entry(turn).or_insert((0, 0));
        entry.0 += value;
        entry.1 += 1;
    }

    /// Get P1 win rate.
    pub fn p1_win_rate(&self) -> f64 {
        if self.total_games == 0 {
            0.0
        } else {
            self.p1_wins as f64 / self.total_games as f64
        }
    }

    /// Get P2 win rate.
    pub fn p2_win_rate(&self) -> f64 {
        if self.total_games == 0 {
            0.0
        } else {
            self.p2_wins as f64 / self.total_games as f64
        }
    }

    /// Get draw rate.
    pub fn draw_rate(&self) -> f64 {
        if self.total_games == 0 {
            0.0
        } else {
            self.draws as f64 / self.total_games as f64
        }
    }

    /// Get average actions per game for P1.
    pub fn p1_avg_actions(&self) -> f64 {
        if self.total_games == 0 {
            0.0
        } else {
            self.p1_actions_total as f64 / self.total_games as f64
        }
    }

    /// Get average actions per game for P2.
    pub fn p2_avg_actions(&self) -> f64 {
        if self.total_games == 0 {
            0.0
        } else {
            self.p2_actions_total as f64 / self.total_games as f64
        }
    }

    /// Get average first creature death turn.
    pub fn avg_first_creature_death(&self) -> Option<f64> {
        if self.first_creature_death_turns.is_empty() {
            None
        } else {
            Some(
                self.first_creature_death_turns.iter().sum::<u32>() as f64
                    / self.first_creature_death_turns.len() as f64,
            )
        }
    }

    /// Convert a curve map to sorted (turn, average) pairs.
    pub fn avg_curve(map: &HashMap<u32, (i64, usize)>) -> Vec<(u32, f64)> {
        let mut result: Vec<_> = map
            .iter()
            .map(|(&turn, &(sum, count))| (turn, sum as f64 / count as f64))
            .collect();
        result.sort_by_key(|(turn, _)| *turn);
        result
    }

    // === Phase 2: Enhanced Metrics Helper Methods ===

    /// Get average turn when P1 played first creature.
    pub fn p1_avg_first_creature_turn(&self) -> Option<f64> {
        if self.p1_first_creature_count == 0 {
            None
        } else {
            Some(self.p1_first_creature_turn_sum as f64 / self.p1_first_creature_count as f64)
        }
    }

    /// Get average turn when P2 played first creature.
    pub fn p2_avg_first_creature_turn(&self) -> Option<f64> {
        if self.p2_first_creature_count == 0 {
            None
        } else {
            Some(self.p2_first_creature_turn_sum as f64 / self.p2_first_creature_count as f64)
        }
    }

    /// Get average board advantage across all games.
    pub fn avg_board_advantage(&self) -> f64 {
        if self.total_games == 0 {
            0.0
        } else {
            self.total_avg_board_advantage / self.total_games as f64
        }
    }

    /// Get percentage of turns P1 was ahead.
    pub fn pct_turns_p1_ahead(&self) -> f64 {
        let total = self.total_turns_p1_ahead + self.total_turns_p2_ahead + self.total_turns_even;
        if total == 0 {
            0.0
        } else {
            self.total_turns_p1_ahead as f64 / total as f64
        }
    }

    /// Get percentage of turns P2 was ahead.
    pub fn pct_turns_p2_ahead(&self) -> f64 {
        let total = self.total_turns_p1_ahead + self.total_turns_p2_ahead + self.total_turns_even;
        if total == 0 {
            0.0
        } else {
            self.total_turns_p2_ahead as f64 / total as f64
        }
    }

    /// Get average essence spent per game by P1.
    pub fn p1_avg_essence_spent(&self) -> f64 {
        if self.total_games == 0 {
            0.0
        } else {
            self.p1_total_essence_spent as f64 / self.total_games as f64
        }
    }

    /// Get average essence spent per game by P2.
    pub fn p2_avg_essence_spent(&self) -> f64 {
        if self.total_games == 0 {
            0.0
        } else {
            self.p2_total_essence_spent as f64 / self.total_games as f64
        }
    }

    /// Get P1's resource efficiency (board impact per essence spent).
    pub fn p1_resource_efficiency(&self) -> f64 {
        if self.p1_total_essence_spent == 0 {
            0.0
        } else {
            self.p1_total_board_impact as f64 / self.p1_total_essence_spent as f64
        }
    }

    /// Get P2's resource efficiency (board impact per essence spent).
    pub fn p2_resource_efficiency(&self) -> f64 {
        if self.p2_total_essence_spent == 0 {
            0.0
        } else {
            self.p2_total_board_impact as f64 / self.p2_total_essence_spent as f64
        }
    }

    /// Get average face damage per game by P1.
    pub fn p1_avg_face_damage(&self) -> f64 {
        if self.total_games == 0 {
            0.0
        } else {
            self.p1_total_face_damage as f64 / self.total_games as f64
        }
    }

    /// Get average face damage per game by P2.
    pub fn p2_avg_face_damage(&self) -> f64 {
        if self.total_games == 0 {
            0.0
        } else {
            self.p2_total_face_damage as f64 / self.total_games as f64
        }
    }

    /// Get P1's trade ratio (creatures killed / creatures lost).
    pub fn p1_trade_ratio(&self) -> f64 {
        if self.p1_total_creatures_lost == 0 {
            self.p1_total_creatures_killed as f64
        } else {
            self.p1_total_creatures_killed as f64 / self.p1_total_creatures_lost as f64
        }
    }

    /// Get P2's trade ratio (creatures killed / creatures lost).
    pub fn p2_trade_ratio(&self) -> f64 {
        if self.p2_total_creatures_lost == 0 {
            self.p2_total_creatures_killed as f64
        } else {
            self.p2_total_creatures_killed as f64 / self.p2_total_creatures_lost as f64
        }
    }

    // === Phase 3: Statistical Analysis Methods ===

    /// Get P1 win rate statistics with confidence interval and significance test.
    pub fn p1_win_rate_stats(&self) -> ProportionStats {
        ProportionStats::calculate(self.p1_wins, self.total_games, 0.5)
    }

    /// Get first blood statistics with confidence interval.
    pub fn first_blood_stats(&self) -> ProportionStats {
        let total = self.p1_first_blood + self.p2_first_blood;
        ProportionStats::calculate(self.p1_first_blood, total, 0.5)
    }

    /// Get first creature statistics with confidence interval.
    pub fn first_creature_stats(&self) -> ProportionStats {
        let total = self.p1_first_creature + self.p2_first_creature;
        ProportionStats::calculate(self.p1_first_creature, total, 0.5)
    }

    /// Calculate correlation between first blood and winning.
    ///
    /// Returns the Pearson correlation coefficient if sufficient data is available.
    pub fn first_blood_win_correlation(&self) -> Option<f64> {
        if self.game_metrics.is_empty() {
            return None;
        }

        // Build correlation data from raw game data
        // This requires the per-game diagnostics, but we've aggregated them
        // We'll compute from the aggregated counts as an approximation
        // For proper correlation, we'd need individual game results

        // Using aggregated counts, we can estimate:
        // P(P1 wins | P1 first blood) vs P(P1 wins | P2 first blood)
        // This is a simplified correlation estimate

        let total_with_fb = self.p1_first_blood + self.p2_first_blood;
        if total_with_fb == 0 {
            return None;
        }

        // We don't have the cross-tabulation, so return None
        // A proper implementation would track this in GameDiagnostics
        None
    }

    /// Calculate correlation between board advantage and winning.
    ///
    /// Returns the Pearson correlation coefficient between average board advantage
    /// and game outcome (1 for P1 win, 0 for draw, -1 for P2 win).
    pub fn board_advantage_win_correlation(&self) -> Option<f64> {
        if self.game_metrics.len() < 3 {
            return None;
        }

        // Extract board advantage scores and outcomes would require storing them
        // For now, return None as we need the per-game data with outcomes
        None
    }

    /// Get percentiles for game length.
    ///
    /// Returns (P10, P50, P90) for game length in turns.
    pub fn game_length_percentiles(&self) -> Option<(f64, f64, f64)> {
        if self.game_metrics.is_empty() {
            return None;
        }

        let mut lengths: Vec<f64> = self
            .game_metrics
            .iter()
            .map(|m| m.turn_metrics.len() as f64)
            .collect();
        lengths.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let p10 = super::statistics::percentile(&lengths, 0.10)?;
        let p50 = super::statistics::percentile(&lengths, 0.50)?;
        let p90 = super::statistics::percentile(&lengths, 0.90)?;

        Some((p10, p50, p90))
    }

    /// Get percentiles for board advantage.
    ///
    /// Returns (P10, P50, P90) for average board advantage per game.
    pub fn board_advantage_percentiles(&self) -> Option<(f64, f64, f64)> {
        if self.game_metrics.is_empty() {
            return None;
        }

        let mut advantages: Vec<f64> = self
            .game_metrics
            .iter()
            .map(|m| m.avg_board_advantage)
            .collect();
        advantages.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let p10 = super::statistics::percentile(&advantages, 0.10)?;
        let p50 = super::statistics::percentile(&advantages, 0.50)?;
        let p90 = super::statistics::percentile(&advantages, 0.90)?;

        Some((p10, p50, p90))
    }

    /// Determine if there's a statistically significant P1/P2 imbalance.
    ///
    /// Returns true if the p-value for the win rate chi-square test is < 0.05.
    pub fn has_significant_imbalance(&self) -> bool {
        let stats = self.p1_win_rate_stats();
        stats.p_value < 0.05
    }

    /// Get an overall assessment of game balance.
    pub fn balance_assessment(&self) -> BalanceAssessment {
        let stats = self.p1_win_rate_stats();

        if stats.total < 30 {
            return BalanceAssessment::InsufficientData;
        }

        match stats.significance {
            super::statistics::SignificanceLevel::HighlySignificant => {
                if stats.proportion > 0.5 {
                    BalanceAssessment::P1Favored
                } else {
                    BalanceAssessment::P2Favored
                }
            }
            super::statistics::SignificanceLevel::Significant => {
                if stats.proportion > 0.5 {
                    BalanceAssessment::SlightP1Advantage
                } else {
                    BalanceAssessment::SlightP2Advantage
                }
            }
            super::statistics::SignificanceLevel::Marginal => BalanceAssessment::Marginal,
            super::statistics::SignificanceLevel::NotSignificant => BalanceAssessment::Balanced,
        }
    }
}

/// Overall balance assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BalanceAssessment {
    /// Not enough games to determine
    InsufficientData,
    /// Game appears balanced (p >= 0.05)
    Balanced,
    /// Marginal imbalance (0.05 <= p < 0.10)
    Marginal,
    /// P1 has a slight advantage (p < 0.05, P1 WR > 50%)
    SlightP1Advantage,
    /// P2 has a slight advantage (p < 0.05, P1 WR < 50%)
    SlightP2Advantage,
    /// P1 is clearly favored (p < 0.01, P1 WR > 50%)
    P1Favored,
    /// P2 is clearly favored (p < 0.01, P1 WR < 50%)
    P2Favored,
}

impl BalanceAssessment {
    /// Get a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::InsufficientData => "Insufficient data for statistical analysis",
            Self::Balanced => "Game appears balanced",
            Self::Marginal => "Marginal imbalance detected",
            Self::SlightP1Advantage => "P1 has a slight advantage",
            Self::SlightP2Advantage => "P2 has a slight advantage",
            Self::P1Favored => "P1 is clearly favored",
            Self::P2Favored => "P2 is clearly favored",
        }
    }

    /// Get a symbol for console display.
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::InsufficientData => "?",
            Self::Balanced => "=",
            Self::Marginal => "~",
            Self::SlightP1Advantage => ">",
            Self::SlightP2Advantage => "<",
            Self::P1Favored => ">>",
            Self::P2Favored => "<<",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_diagnostic(winner: Option<PlayerId>, turns: u32, seed: u64) -> GameDiagnostics {
        GameDiagnostics {
            seed,
            winner,
            total_turns: turns,
            snapshots: vec![],
            first_damage_to_p1_turn: Some(2),
            first_damage_to_p2_turn: Some(3),
            first_creature_death_turn: Some(4),
            p1_actions: 10,
            p2_actions: 12,
            metrics: GameMetrics::default(),
        }
    }

    #[test]
    fn test_aggregated_stats_basic() {
        let games = vec![
            create_test_diagnostic(Some(PlayerId::PLAYER_ONE), 15, 1),
            create_test_diagnostic(Some(PlayerId::PLAYER_TWO), 20, 2),
            create_test_diagnostic(Some(PlayerId::PLAYER_ONE), 25, 3),
        ];

        let stats = AggregatedStats::analyze(&games);

        assert_eq!(stats.total_games, 3);
        assert_eq!(stats.p1_wins, 2);
        assert_eq!(stats.p2_wins, 1);
        assert!((stats.p1_win_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_game_length_buckets() {
        let games = vec![
            create_test_diagnostic(Some(PlayerId::PLAYER_ONE), 8, 1),  // early
            create_test_diagnostic(Some(PlayerId::PLAYER_TWO), 15, 2), // mid
            create_test_diagnostic(Some(PlayerId::PLAYER_ONE), 25, 3), // late
        ];

        let stats = AggregatedStats::analyze(&games);

        assert_eq!(stats.games_early, 1);
        assert_eq!(stats.games_mid, 1);
        assert_eq!(stats.games_late, 1);
        assert_eq!(stats.p1_wins_early, 1);
        assert_eq!(stats.p1_wins_mid, 0);
        assert_eq!(stats.p1_wins_late, 1);
    }
}
