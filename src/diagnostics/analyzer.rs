//! Diagnostic statistics analysis.
//!
//! Aggregates game diagnostics into statistical summaries for P1/P2 asymmetry analysis.

use std::collections::HashMap;

use crate::types::PlayerId;

use super::collector::{GameDiagnostics, TurnSnapshot};

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
