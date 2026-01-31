//! Card statistics collector and aggregator.

use std::collections::HashMap;

use crate::types::{CardId, PlayerId};

use super::types::{CardPlayStats, GameCardTracker};

/// Collects and aggregates card statistics across multiple games.
#[derive(Clone, Debug)]
pub struct CardStatsCollector {
    /// Per-card statistics.
    pub stats: HashMap<CardId, CardPlayStats>,
    /// Total games analyzed.
    pub total_games: u32,
    /// Games won by Player 1.
    pub p1_wins: u32,
    /// Games won by Player 2.
    pub p2_wins: u32,
    /// Draws.
    pub draws: u32,
}

impl Default for CardStatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl CardStatsCollector {
    /// Create a new empty collector.
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
            total_games: 0,
            p1_wins: 0,
            p2_wins: 0,
            draws: 0,
        }
    }

    /// Aggregate a completed game's card plays into the collector.
    pub fn aggregate_game(&mut self, tracker: &GameCardTracker, winner: Option<PlayerId>) {
        self.total_games += 1;

        match winner {
            Some(PlayerId::PLAYER_ONE) => self.p1_wins += 1,
            Some(PlayerId::PLAYER_TWO) => self.p2_wins += 1,
            _ => self.draws += 1,
        }

        // Process P1's cards
        for &card_id in &tracker.p1_cards {
            let stats = self.stats.entry(card_id).or_default();
            stats.games_played_in += 1;

            // Count as win if P1 won
            if winner == Some(PlayerId::PLAYER_ONE) {
                stats.wins_when_played += 1;
            }

            // Add play count
            if let Some(&count) = tracker.p1_plays.get(&card_id) {
                stats.play_count += count;
            }

            // Add turn bucket distribution from P1's plays
            if let Some(&(early, mid, late)) = tracker.p1_turn_buckets.get(&card_id) {
                stats.early_plays += early;
                stats.mid_plays += mid;
                stats.late_plays += late;
            }
        }

        // Process P2's cards
        for &card_id in &tracker.p2_cards {
            let stats = self.stats.entry(card_id).or_default();
            stats.games_played_in += 1;

            // Count as win if P2 won
            if winner == Some(PlayerId::PLAYER_TWO) {
                stats.wins_when_played += 1;
            }

            // Add play count
            if let Some(&count) = tracker.p2_plays.get(&card_id) {
                stats.play_count += count;
            }

            // Add turn bucket distribution from P2's plays
            if let Some(&(early, mid, late)) = tracker.p2_turn_buckets.get(&card_id) {
                stats.early_plays += early;
                stats.mid_plays += mid;
                stats.late_plays += late;
            }
        }
    }

    /// Get the baseline win rate (should be ~0.5 for balanced games).
    pub fn baseline_win_rate(&self) -> f64 {
        if self.total_games == 0 {
            0.5
        } else {
            // In a two-player game, baseline is 50% if balanced
            0.5
        }
    }

    /// Get cards sorted by impact score (descending).
    pub fn cards_by_impact(&self) -> Vec<(CardId, &CardPlayStats)> {
        let baseline = self.baseline_win_rate();
        let mut cards: Vec<_> = self.stats.iter().map(|(&id, stats)| (id, stats)).collect();
        cards.sort_by(|a, b| {
            let impact_a = a.1.impact_score(baseline, self.total_games);
            let impact_b = b.1.impact_score(baseline, self.total_games);
            impact_b
                .partial_cmp(&impact_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        cards
    }

    /// Get cards sorted by win rate (descending).
    pub fn cards_by_win_rate(&self) -> Vec<(CardId, &CardPlayStats)> {
        let mut cards: Vec<_> = self.stats.iter().map(|(&id, stats)| (id, stats)).collect();
        cards.sort_by(|a, b| {
            let wr_a = a.1.win_rate();
            let wr_b = b.1.win_rate();
            wr_b.partial_cmp(&wr_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        cards
    }

    /// Get cards sorted by play count (descending).
    pub fn cards_by_plays(&self) -> Vec<(CardId, &CardPlayStats)> {
        let mut cards: Vec<_> = self.stats.iter().map(|(&id, stats)| (id, stats)).collect();
        cards.sort_by(|a, b| b.1.play_count.cmp(&a.1.play_count));
        cards
    }

    /// Filter cards by minimum play count.
    pub fn filter_by_min_plays(&self, min_plays: u32) -> Vec<(CardId, &CardPlayStats)> {
        self.stats
            .iter()
            .filter(|(_, stats)| stats.games_played_in >= min_plays)
            .map(|(&id, stats)| (id, stats))
            .collect()
    }

    /// Merge another collector into this one.
    pub fn merge(&mut self, other: &CardStatsCollector) {
        self.total_games += other.total_games;
        self.p1_wins += other.p1_wins;
        self.p2_wins += other.p2_wins;
        self.draws += other.draws;

        for (&card_id, other_stats) in &other.stats {
            let stats = self.stats.entry(card_id).or_default();
            stats.play_count += other_stats.play_count;
            stats.games_played_in += other_stats.games_played_in;
            stats.wins_when_played += other_stats.wins_when_played;
            stats.early_plays += other_stats.early_plays;
            stats.mid_plays += other_stats.mid_plays;
            stats.late_plays += other_stats.late_plays;
        }
    }
}
