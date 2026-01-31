//! Core types for card statistics tracking.

use std::collections::{HashMap, HashSet};

use crate::types::{CardId, PlayerId};

/// Statistics for a single card across all games.
#[derive(Clone, Debug, Default)]
pub struct CardPlayStats {
    /// Total times this card was played.
    pub play_count: u32,
    /// Number of games where this card was played at least once.
    pub games_played_in: u32,
    /// Number of wins in games where this card was played.
    pub wins_when_played: u32,
    /// Number of times played in early game (turns 1-5).
    pub early_plays: u32,
    /// Number of times played in mid game (turns 6-15).
    pub mid_plays: u32,
    /// Number of times played in late game (turns 16+).
    pub late_plays: u32,
}

impl CardPlayStats {
    /// Calculate win rate when this card was played.
    pub fn win_rate(&self) -> f64 {
        if self.games_played_in == 0 {
            0.0
        } else {
            self.wins_when_played as f64 / self.games_played_in as f64
        }
    }

    /// Calculate win rate delta from baseline (typically 0.5).
    pub fn win_delta(&self, baseline: f64) -> f64 {
        self.win_rate() - baseline
    }

    /// Calculate impact score (delta weighted by frequency).
    pub fn impact_score(&self, baseline: f64, total_games: u32) -> f64 {
        if total_games == 0 || self.games_played_in == 0 {
            return 0.0;
        }
        let delta = self.win_delta(baseline);
        let frequency = self.games_played_in as f64 / total_games as f64;
        delta * frequency.sqrt() * 100.0
    }

    /// Get early game play percentage.
    pub fn early_pct(&self) -> f64 {
        if self.play_count == 0 {
            0.0
        } else {
            self.early_plays as f64 / self.play_count as f64 * 100.0
        }
    }

    /// Get mid game play percentage.
    pub fn mid_pct(&self) -> f64 {
        if self.play_count == 0 {
            0.0
        } else {
            self.mid_plays as f64 / self.play_count as f64 * 100.0
        }
    }

    /// Get late game play percentage.
    pub fn late_pct(&self) -> f64 {
        if self.play_count == 0 {
            0.0
        } else {
            self.late_plays as f64 / self.play_count as f64 * 100.0
        }
    }
}

/// Turn timing bucket for categorizing when cards are played.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnBucket {
    /// Turns 1-5
    Early,
    /// Turns 6-15
    Mid,
    /// Turns 16+
    Late,
}

impl TurnBucket {
    /// Categorize a turn number into a bucket.
    pub fn from_turn(turn: u16) -> Self {
        match turn {
            1..=5 => TurnBucket::Early,
            6..=15 => TurnBucket::Mid,
            _ => TurnBucket::Late,
        }
    }
}

/// Tracks card plays within a single game.
#[derive(Clone, Debug, Default)]
pub struct GameCardTracker {
    /// Cards played by Player 1 (card_id -> play count).
    pub p1_plays: HashMap<CardId, u32>,
    /// Cards played by Player 2 (card_id -> play count).
    pub p2_plays: HashMap<CardId, u32>,
    /// Unique cards played by P1 (for games_played_in tracking).
    pub p1_cards: HashSet<CardId>,
    /// Unique cards played by P2 (for games_played_in tracking).
    pub p2_cards: HashSet<CardId>,
    /// Turn bucket distribution per card for P1.
    pub p1_turn_buckets: HashMap<CardId, (u32, u32, u32)>, // (early, mid, late)
    /// Turn bucket distribution per card for P2.
    pub p2_turn_buckets: HashMap<CardId, (u32, u32, u32)>, // (early, mid, late)
}

impl GameCardTracker {
    /// Create a new empty tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a card being played.
    pub fn record_play(&mut self, player: PlayerId, card_id: CardId, turn: u16) {
        let bucket = TurnBucket::from_turn(turn);

        let turn_buckets = if player == PlayerId::PLAYER_ONE {
            *self.p1_plays.entry(card_id).or_insert(0) += 1;
            self.p1_cards.insert(card_id);
            &mut self.p1_turn_buckets
        } else {
            *self.p2_plays.entry(card_id).or_insert(0) += 1;
            self.p2_cards.insert(card_id);
            &mut self.p2_turn_buckets
        };

        // Track turn bucket per player
        let buckets = turn_buckets.entry(card_id).or_insert((0, 0, 0));
        match bucket {
            TurnBucket::Early => buckets.0 += 1,
            TurnBucket::Mid => buckets.1 += 1,
            TurnBucket::Late => buckets.2 += 1,
        }
    }

    /// Reset tracker for a new game.
    pub fn reset(&mut self) {
        self.p1_plays.clear();
        self.p2_plays.clear();
        self.p1_cards.clear();
        self.p2_cards.clear();
        self.p1_turn_buckets.clear();
        self.p2_turn_buckets.clear();
    }

    /// Get all unique cards played in this game.
    pub fn all_cards(&self) -> HashSet<CardId> {
        self.p1_cards.union(&self.p2_cards).copied().collect()
    }
}
