//! Lightweight diagnostic collection for validation games.
//!
//! Unlike the full DiagnosticRunner, this captures only what we need
//! for aggregate statistics without storing per-turn snapshots.

use serde::{Deserialize, Serialize};

use crate::engine::GameEngine;
use crate::types::PlayerId;

/// Per-game diagnostic data collected during validation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameDiagnosticData {
    /// Who dealt face damage first (None if no damage dealt).
    pub first_blood: Option<PlayerId>,

    // Essence tracking
    /// Total essence spent by P1.
    pub p1_essence_spent: u32,
    /// Total essence spent by P2.
    pub p2_essence_spent: u32,

    // Face damage tracking
    /// Total face damage dealt by P1.
    pub p1_face_damage: u32,
    /// Total face damage dealt by P2.
    pub p2_face_damage: u32,

    // Creature combat tracking
    /// Creatures killed by P1.
    pub p1_creatures_killed: u32,
    /// Creatures killed by P2.
    pub p2_creatures_killed: u32,
    /// Creatures lost by P1.
    pub p1_creatures_lost: u32,
    /// Creatures lost by P2.
    pub p2_creatures_lost: u32,

    // Board advantage tracking
    /// Sum of board advantage scores (for averaging later).
    pub board_advantage_sum: f64,
    /// Number of turns P1 was ahead.
    pub turns_p1_ahead: u32,
    /// Number of turns P2 was ahead.
    pub turns_p2_ahead: u32,
    /// Number of turns even.
    pub turns_even: u32,
    /// Total turn count for averaging.
    pub turn_count: u32,
}

/// Tracks diagnostic data during a game.
///
/// Call `record_turn()` at the start of each turn to capture state changes,
/// then `finalize()` to get the collected data.
pub struct GameDiagnosticCollector {
    data: GameDiagnosticData,
    // Previous state for delta tracking
    prev_p1_life: i16,
    prev_p2_life: i16,
    prev_p1_creature_count: usize,
    prev_p2_creature_count: usize,
    prev_p1_essence_spent: u32,
    prev_p2_essence_spent: u32,
    initialized: bool,
}

impl Default for GameDiagnosticCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl GameDiagnosticCollector {
    /// Create a new collector.
    pub fn new() -> Self {
        Self {
            data: GameDiagnosticData::default(),
            prev_p1_life: 30,
            prev_p2_life: 30,
            prev_p1_creature_count: 0,
            prev_p2_creature_count: 0,
            prev_p1_essence_spent: 0,
            prev_p2_essence_spent: 0,
            initialized: false,
        }
    }

    /// Record turn state and compute deltas.
    pub fn record_turn(&mut self, engine: &GameEngine) {
        let state = &engine.state;

        let p1_life = state.players[0].life;
        let p2_life = state.players[1].life;
        let p1_creatures = state.players[0].creatures.len();
        let p2_creatures = state.players[1].creatures.len();

        // Track essence spent (max_essence - current_essence shows what was available)
        // We track cumulative essence spent by looking at max_essence growth
        let p1_essence_spent = state.players[0].max_essence.saturating_sub(state.players[0].current_essence) as u32;
        let p2_essence_spent = state.players[1].max_essence.saturating_sub(state.players[1].current_essence) as u32;

        if self.initialized {
            // Track face damage (life lost)
            let p1_damage_taken = (self.prev_p1_life - p1_life).max(0) as u32;
            let p2_damage_taken = (self.prev_p2_life - p2_life).max(0) as u32;

            if p2_damage_taken > 0 {
                self.data.p1_face_damage += p2_damage_taken;
                // First blood check
                if self.data.first_blood.is_none() {
                    self.data.first_blood = Some(PlayerId::PLAYER_ONE);
                }
            }
            if p1_damage_taken > 0 {
                self.data.p2_face_damage += p1_damage_taken;
                // First blood check
                if self.data.first_blood.is_none() {
                    self.data.first_blood = Some(PlayerId::PLAYER_TWO);
                }
            }

            // Track creature deaths
            if p1_creatures < self.prev_p1_creature_count {
                let lost = (self.prev_p1_creature_count - p1_creatures) as u32;
                self.data.p1_creatures_lost += lost;
                self.data.p2_creatures_killed += lost;
            }
            if p2_creatures < self.prev_p2_creature_count {
                let lost = (self.prev_p2_creature_count - p2_creatures) as u32;
                self.data.p2_creatures_lost += lost;
                self.data.p1_creatures_killed += lost;
            }

            // Track essence spent delta
            if p1_essence_spent > self.prev_p1_essence_spent {
                self.data.p1_essence_spent += p1_essence_spent - self.prev_p1_essence_spent;
            }
            if p2_essence_spent > self.prev_p2_essence_spent {
                self.data.p2_essence_spent += p2_essence_spent - self.prev_p2_essence_spent;
            }
        }

        // Calculate board advantage
        let p1_attack: i32 = state.players[0]
            .creatures
            .iter()
            .map(|c| c.attack as i32)
            .sum();
        let p2_attack: i32 = state.players[1]
            .creatures
            .iter()
            .map(|c| c.attack as i32)
            .sum();
        let p1_health: i32 = state.players[0]
            .creatures
            .iter()
            .map(|c| c.current_health as i32)
            .sum();
        let p2_health: i32 = state.players[1]
            .creatures
            .iter()
            .map(|c| c.current_health as i32)
            .sum();

        let life_diff = (p1_life - p2_life) as i32;
        let attack_diff = p1_attack - p2_attack;
        let health_diff = p1_health - p2_health;

        // Board advantage score (positive = P1 ahead)
        let board_score = (life_diff as f64 + attack_diff as f64 + health_diff as f64) / 3.0;
        self.data.board_advantage_sum += board_score;
        self.data.turn_count += 1;

        // Track who's ahead
        let threshold = 2.0;
        if board_score > threshold {
            self.data.turns_p1_ahead += 1;
        } else if board_score < -threshold {
            self.data.turns_p2_ahead += 1;
        } else {
            self.data.turns_even += 1;
        }

        // Update previous state
        self.prev_p1_life = p1_life;
        self.prev_p2_life = p2_life;
        self.prev_p1_creature_count = p1_creatures;
        self.prev_p2_creature_count = p2_creatures;
        self.prev_p1_essence_spent = p1_essence_spent;
        self.prev_p2_essence_spent = p2_essence_spent;
        self.initialized = true;
    }

    /// Finalize and return the collected data.
    pub fn finalize(self) -> GameDiagnosticData {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_initialization() {
        let collector = GameDiagnosticCollector::new();
        let data = collector.finalize();

        assert!(data.first_blood.is_none());
        assert_eq!(data.p1_face_damage, 0);
        assert_eq!(data.p2_face_damage, 0);
        assert_eq!(data.turn_count, 0);
    }

    #[test]
    fn test_game_diagnostic_data_default() {
        let data = GameDiagnosticData::default();
        assert!(data.first_blood.is_none());
        assert_eq!(data.board_advantage_sum, 0.0);
    }
}
