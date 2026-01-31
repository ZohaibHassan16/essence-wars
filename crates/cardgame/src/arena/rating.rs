//! ELO rating persistence for deck/commander tracking.
//!
//! Tracks win/loss records and ELO ratings per deck across arena matches.
//! Ratings are automatically updated after each match and persisted to disk.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::MatchupStats;

/// Default starting ELO rating for new decks.
pub const DEFAULT_RATING: f64 = 1500.0;

/// Default K-factor for rating changes.
pub const DEFAULT_K_FACTOR: f64 = 32.0;

/// Match result for history tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MatchResult {
    Win,
    Loss,
    Draw,
}

/// A single rating change record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingChange {
    /// Date of the match (ISO 8601).
    pub date: String,
    /// Opponent deck ID.
    pub opponent: String,
    /// Rating change (positive or negative).
    pub delta: i32,
    /// Match result from this deck's perspective.
    pub result: MatchResult,
    /// Number of games in the match.
    pub games: u32,
}

/// Rating data for a single deck.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckRating {
    /// Current ELO rating.
    pub rating: f64,
    /// Total games played.
    pub games: u32,
    /// Total wins.
    pub wins: u32,
    /// Total losses.
    pub losses: u32,
    /// Total draws.
    pub draws: u32,
    /// Recent rating changes (last 50).
    #[serde(default)]
    pub history: Vec<RatingChange>,
}

impl Default for DeckRating {
    fn default() -> Self {
        Self {
            rating: DEFAULT_RATING,
            games: 0,
            wins: 0,
            losses: 0,
            draws: 0,
            history: Vec::new(),
        }
    }
}

impl DeckRating {
    /// Win rate as a percentage (0-100).
    pub fn win_rate(&self) -> f64 {
        if self.games == 0 {
            50.0
        } else {
            (self.wins as f64 / self.games as f64) * 100.0
        }
    }

    /// Add a rating change to history, keeping only the last 50.
    fn add_history(&mut self, change: RatingChange) {
        self.history.push(change);
        if self.history.len() > 50 {
            self.history.remove(0);
        }
    }
}

/// Persistent storage format for deck ratings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingStorage {
    /// Storage format version.
    pub version: u32,
    /// Last update timestamp (ISO 8601).
    pub last_updated: String,
    /// Ratings by deck ID.
    pub ratings: HashMap<String, DeckRating>,
}

impl Default for RatingStorage {
    fn default() -> Self {
        Self {
            version: 1,
            last_updated: chrono::Utc::now().to_rfc3339(),
            ratings: HashMap::new(),
        }
    }
}

/// ELO rating tracker with persistence.
pub struct EloTracker {
    /// In-memory ratings.
    storage: RatingStorage,
    /// K-factor for rating calculations.
    k_factor: f64,
    /// File path for persistence.
    file_path: PathBuf,
}

impl EloTracker {
    /// Load ratings from file or create new tracker.
    pub fn load_or_create(path: &Path) -> Self {
        let storage = if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => RatingStorage::default(),
            }
        } else {
            RatingStorage::default()
        };

        Self {
            storage,
            k_factor: DEFAULT_K_FACTOR,
            file_path: path.to_path_buf(),
        }
    }

    /// Get rating for a deck (returns default if not tracked).
    pub fn get_rating(&self, deck_id: &str) -> f64 {
        self.storage
            .ratings
            .get(deck_id)
            .map(|r| r.rating)
            .unwrap_or(DEFAULT_RATING)
    }

    /// Get full rating data for a deck.
    pub fn get_deck_rating(&self, deck_id: &str) -> Option<&DeckRating> {
        self.storage.ratings.get(deck_id)
    }

    /// Calculate expected score for player A against player B.
    fn expected_score(rating_a: f64, rating_b: f64) -> f64 {
        1.0 / (1.0 + 10.0_f64.powf((rating_b - rating_a) / 400.0))
    }

    /// Update ratings based on match results.
    ///
    /// Returns (deck1_delta, deck2_delta) as integer changes.
    pub fn update_match(
        &mut self,
        deck1_id: &str,
        deck2_id: &str,
        stats: &MatchupStats,
    ) -> (i32, i32) {
        // Get current ratings
        let rating1 = self.get_rating(deck1_id);
        let rating2 = self.get_rating(deck2_id);

        // Calculate expected scores
        let expected1 = Self::expected_score(rating1, rating2);
        let expected2 = 1.0 - expected1;

        // Calculate actual scores (considering all games)
        let total_games = stats.games as f64;
        if total_games == 0.0 {
            return (0, 0);
        }

        // Actual score: wins + 0.5 * draws, normalized to [0, 1]
        let actual1 = (stats.bot1_wins as f64 + 0.5 * stats.draws as f64) / total_games;
        let actual2 = (stats.bot2_wins as f64 + 0.5 * stats.draws as f64) / total_games;

        // Calculate rating changes
        let delta1 = (self.k_factor * (actual1 - expected1)).round() as i32;
        let delta2 = (self.k_factor * (actual2 - expected2)).round() as i32;

        // Update deck 1
        let deck1_rating = self
            .storage
            .ratings
            .entry(deck1_id.to_string())
            .or_default();
        deck1_rating.rating += delta1 as f64;
        deck1_rating.games += stats.games as u32;
        deck1_rating.wins += stats.bot1_wins as u32;
        deck1_rating.losses += stats.bot2_wins as u32;
        deck1_rating.draws += stats.draws as u32;

        // Determine result for deck 1
        let result1 = if stats.bot1_wins > stats.bot2_wins {
            MatchResult::Win
        } else if stats.bot2_wins > stats.bot1_wins {
            MatchResult::Loss
        } else {
            MatchResult::Draw
        };

        deck1_rating.add_history(RatingChange {
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            opponent: deck2_id.to_string(),
            delta: delta1,
            result: result1,
            games: stats.games as u32,
        });

        // Update deck 2
        let deck2_rating = self
            .storage
            .ratings
            .entry(deck2_id.to_string())
            .or_default();
        deck2_rating.rating += delta2 as f64;
        deck2_rating.games += stats.games as u32;
        deck2_rating.wins += stats.bot2_wins as u32;
        deck2_rating.losses += stats.bot1_wins as u32;
        deck2_rating.draws += stats.draws as u32;

        // Determine result for deck 2
        let result2 = if stats.bot2_wins > stats.bot1_wins {
            MatchResult::Win
        } else if stats.bot1_wins > stats.bot2_wins {
            MatchResult::Loss
        } else {
            MatchResult::Draw
        };

        deck2_rating.add_history(RatingChange {
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            opponent: deck1_id.to_string(),
            delta: delta2,
            result: result2,
            games: stats.games as u32,
        });

        // Update timestamp
        self.storage.last_updated = chrono::Utc::now().to_rfc3339();

        (delta1, delta2)
    }

    /// Save ratings to disk.
    pub fn save(&self) -> io::Result<()> {
        // Create parent directory if needed
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&self.storage)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        fs::write(&self.file_path, json)
    }

    /// Format a brief ELO summary for console output.
    pub fn format_summary(&self, deck1_id: &str, deck2_id: &str, delta1: i32, delta2: i32) -> String {
        let rating1 = self.get_rating(deck1_id);
        let rating2 = self.get_rating(deck2_id);

        format!(
            "{} {:+} -> {:.0}, {} {:+} -> {:.0}",
            deck1_id, delta1, rating1, deck2_id, delta2, rating2
        )
    }

    /// Get all ratings sorted by rating descending.
    pub fn leaderboard(&self) -> Vec<(&str, &DeckRating)> {
        let mut ratings: Vec<_> = self
            .storage
            .ratings
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        ratings.sort_by(|a, b| b.1.rating.partial_cmp(&a.1.rating).unwrap());
        ratings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_score() {
        // Equal ratings -> 0.5 expected
        let expected = EloTracker::expected_score(1500.0, 1500.0);
        assert!((expected - 0.5).abs() < 0.001);

        // Higher rating -> higher expected
        let expected = EloTracker::expected_score(1600.0, 1400.0);
        assert!(expected > 0.75);

        // Lower rating -> lower expected
        let expected = EloTracker::expected_score(1400.0, 1600.0);
        assert!(expected < 0.25);
    }

    #[test]
    fn test_rating_update() {
        let mut tracker = EloTracker {
            storage: RatingStorage::default(),
            k_factor: 32.0,
            file_path: PathBuf::from("/tmp/test_elo.json"),
        };

        // Deck A beats Deck B convincingly
        let stats = MatchupStats {
            games: 10,
            bot1_wins: 8,
            bot2_wins: 2,
            draws: 0,
            total_turns: 150,
            total_time: std::time::Duration::from_secs(10),
        };

        let (delta1, delta2) = tracker.update_match("deck_a", "deck_b", &stats);

        // Winner gains, loser loses
        assert!(delta1 > 0);
        assert!(delta2 < 0);

        // Ratings updated
        assert!(tracker.get_rating("deck_a") > DEFAULT_RATING);
        assert!(tracker.get_rating("deck_b") < DEFAULT_RATING);

        // Stats recorded
        let deck_a = tracker.get_deck_rating("deck_a").unwrap();
        assert_eq!(deck_a.games, 10);
        assert_eq!(deck_a.wins, 8);
        assert_eq!(deck_a.losses, 2);
    }

    #[test]
    fn test_draw_handling() {
        let mut tracker = EloTracker {
            storage: RatingStorage::default(),
            k_factor: 32.0,
            file_path: PathBuf::from("/tmp/test_elo.json"),
        };

        // All draws
        let stats = MatchupStats {
            games: 10,
            bot1_wins: 0,
            bot2_wins: 0,
            draws: 10,
            total_turns: 150,
            total_time: std::time::Duration::from_secs(10),
        };

        let (delta1, delta2) = tracker.update_match("deck_a", "deck_b", &stats);

        // Equal ratings + draws = no change
        assert_eq!(delta1, 0);
        assert_eq!(delta2, 0);
    }
}
