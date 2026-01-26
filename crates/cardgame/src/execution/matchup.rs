//! Unified matchup definitions for game execution.
//!
//! Provides a complete matchup specification that preserves all deck information
//! including commanders, unlike the legacy `MatchupDefinition` which lost
//! commander information during construction.

use crate::decks::{DeckDefinition, Faction};
use crate::types::CardId;

/// A fully-specified matchup for game execution.
///
/// Unlike `validation::MatchupDefinition`, this type preserves the complete
/// `DeckDefinition` for each player, including commander IDs. This ensures
/// that commanders are properly used when starting games.
///
/// # Example
///
/// ```ignore
/// let matchup = UnifiedMatchup::new(deck1.clone(), deck2.clone());
/// println!("Matchup: {} vs {}", matchup.deck1.name, matchup.deck2.name);
/// println!("Commanders: {} vs {}", matchup.commander1(), matchup.commander2());
/// ```
#[derive(Debug, Clone)]
pub struct UnifiedMatchup {
    /// Unique identifier for this matchup.
    ///
    /// Format: "{deck1_id}_vs_{deck2_id}"
    pub id: String,

    /// First deck (complete definition including commander).
    pub deck1: DeckDefinition,

    /// Second deck (complete definition including commander).
    pub deck2: DeckDefinition,
}

impl UnifiedMatchup {
    /// Create a new matchup from two deck definitions.
    ///
    /// The matchup ID is automatically generated from the deck IDs.
    pub fn new(deck1: DeckDefinition, deck2: DeckDefinition) -> Self {
        let id = format!("{}_vs_{}", deck1.id, deck2.id);
        Self { id, deck1, deck2 }
    }

    /// Create a matchup with a custom ID.
    pub fn with_id(id: impl Into<String>, deck1: DeckDefinition, deck2: DeckDefinition) -> Self {
        Self {
            id: id.into(),
            deck1,
            deck2,
        }
    }

    /// Get faction of deck 1 (if it has one).
    pub fn faction1(&self) -> Option<Faction> {
        self.deck1.faction()
    }

    /// Get faction of deck 2 (if it has one).
    pub fn faction2(&self) -> Option<Faction> {
        self.deck2.faction()
    }

    /// Get faction of deck 1, defaulting to Neutral if unset.
    pub fn faction1_or_neutral(&self) -> Faction {
        self.deck1.faction().unwrap_or(Faction::Neutral)
    }

    /// Get faction of deck 2, defaulting to Neutral if unset.
    pub fn faction2_or_neutral(&self) -> Faction {
        self.deck2.faction().unwrap_or(Faction::Neutral)
    }

    /// Get commander ID for deck 1.
    pub fn commander1(&self) -> CardId {
        CardId(self.deck1.commander)
    }

    /// Get commander ID for deck 2.
    pub fn commander2(&self) -> CardId {
        CardId(self.deck2.commander)
    }

    /// Check if this is a mirror match (same deck).
    pub fn is_mirror(&self) -> bool {
        self.deck1.id == self.deck2.id
    }

    /// Check if this is a same-faction matchup.
    pub fn is_same_faction(&self) -> bool {
        match (self.faction1(), self.faction2()) {
            (Some(f1), Some(f2)) => f1 == f2,
            _ => false,
        }
    }

    /// Get a display string for this matchup.
    ///
    /// Format: "{deck1_name} vs {deck2_name}"
    pub fn display(&self) -> String {
        format!("{} vs {}", self.deck1.name, self.deck2.name)
    }

    /// Get a short faction display string.
    ///
    /// Format: "{faction1} vs {faction2}" using faction tags.
    pub fn faction_display(&self) -> String {
        format!(
            "{} vs {}",
            self.faction1_or_neutral().as_tag(),
            self.faction2_or_neutral().as_tag()
        )
    }

    /// Create the reverse matchup (swap deck positions).
    ///
    /// This is useful for testing both player orders.
    pub fn reversed(&self) -> Self {
        Self::new(self.deck2.clone(), self.deck1.clone())
    }
}

impl std::fmt::Display for UnifiedMatchup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::GameData;

    fn load_test_data() -> GameData {
        GameData::load_default(true).expect("Failed to load test data")
    }

    #[test]
    fn test_matchup_creation() {
        let data = load_test_data();
        let decks: Vec<_> = data.deck_registry.decks().collect();
        assert!(decks.len() >= 2, "Need at least 2 decks for test");

        let matchup = UnifiedMatchup::new(decks[0].clone(), decks[1].clone());

        assert!(!matchup.id.is_empty());
        assert!(matchup.id.contains("_vs_"));
        assert!(!matchup.is_mirror());
    }

    #[test]
    fn test_commander_access() {
        let data = load_test_data();
        let decks: Vec<_> = data.deck_registry.decks().collect();

        let matchup = UnifiedMatchup::new(decks[0].clone(), decks[1].clone());

        // Commander IDs should be non-zero (real commanders)
        assert!(matchup.commander1().0 > 0);
        assert!(matchup.commander2().0 > 0);

        // Commander IDs should match the deck definitions
        assert_eq!(matchup.commander1().0, decks[0].commander);
        assert_eq!(matchup.commander2().0, decks[1].commander);
    }

    #[test]
    fn test_mirror_detection() {
        let data = load_test_data();
        let decks: Vec<_> = data.deck_registry.decks().collect();

        // Same deck = mirror
        let mirror = UnifiedMatchup::new(decks[0].clone(), decks[0].clone());
        assert!(mirror.is_mirror());

        // Different decks = not mirror
        let non_mirror = UnifiedMatchup::new(decks[0].clone(), decks[1].clone());
        assert!(!non_mirror.is_mirror());
    }

    #[test]
    fn test_reversed_matchup() {
        let data = load_test_data();
        let decks: Vec<_> = data.deck_registry.decks().collect();

        let matchup = UnifiedMatchup::new(decks[0].clone(), decks[1].clone());
        let reversed = matchup.reversed();

        assert_eq!(matchup.deck1.id, reversed.deck2.id);
        assert_eq!(matchup.deck2.id, reversed.deck1.id);
        assert_eq!(matchup.commander1(), reversed.commander2());
        assert_eq!(matchup.commander2(), reversed.commander1());
    }

    #[test]
    fn test_faction_access() {
        let data = load_test_data();

        // Get decks from different factions
        let argentum_decks = data.deck_registry.decks_for_faction(Faction::Argentum);
        let symbiote_decks = data.deck_registry.decks_for_faction(Faction::Symbiote);

        assert!(!argentum_decks.is_empty(), "No Argentum decks found");
        assert!(!symbiote_decks.is_empty(), "No Symbiote decks found");

        let matchup = UnifiedMatchup::new(argentum_decks[0].clone(), symbiote_decks[0].clone());

        assert_eq!(matchup.faction1(), Some(Faction::Argentum));
        assert_eq!(matchup.faction2(), Some(Faction::Symbiote));
        assert!(!matchup.is_same_faction());
    }
}
