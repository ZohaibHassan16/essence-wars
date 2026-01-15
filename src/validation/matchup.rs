//! Matchup generation for balance validation.
//!
//! Provides utilities for building faction matchups from deck registries.

use std::collections::HashMap;

use crate::cards::CardDatabase;
use crate::decks::{DeckRegistry, Faction};
use crate::types::CardId;

use super::types::MatchupDefinition;

/// Builder for creating faction matchups.
pub struct MatchupBuilder<'a> {
    registry: &'a DeckRegistry,
    card_db: &'a CardDatabase,
}

impl<'a> MatchupBuilder<'a> {
    /// Create a new matchup builder.
    pub fn new(registry: &'a DeckRegistry, card_db: &'a CardDatabase) -> Self {
        Self { registry, card_db }
    }

    /// Build all faction pair matchups (no mirrors).
    ///
    /// Creates matchups for: Argentum vs Symbiote, Argentum vs Obsidion, Symbiote vs Obsidion.
    /// Uses the first valid deck found for each faction.
    pub fn build_faction_matchups(&self) -> Vec<MatchupDefinition> {
        let factions = [Faction::Argentum, Faction::Symbiote, Faction::Obsidion];
        let mut matchups = Vec::new();

        // Get first valid deck for each faction
        let faction_decks = self.get_faction_decks(&factions);

        // Create all pairs (no mirrors)
        for i in 0..factions.len() {
            for j in (i + 1)..factions.len() {
                let f1 = factions[i];
                let f2 = factions[j];

                if let (Some((id1, cards1)), Some((id2, cards2))) =
                    (faction_decks.get(&f1), faction_decks.get(&f2))
                {
                    matchups.push(MatchupDefinition {
                        faction1: f1,
                        faction2: f2,
                        deck1_id: id1.clone(),
                        deck1_cards: cards1.clone(),
                        deck2_id: id2.clone(),
                        deck2_cards: cards2.clone(),
                    });
                }
            }
        }

        matchups
    }

    /// Build a single matchup between two factions.
    pub fn build_single_matchup(
        &self,
        faction1: Faction,
        faction2: Faction,
    ) -> Option<MatchupDefinition> {
        let deck1 = self.get_first_valid_deck(faction1)?;
        let deck2 = self.get_first_valid_deck(faction2)?;

        Some(MatchupDefinition {
            faction1,
            faction2,
            deck1_id: deck1.0,
            deck1_cards: deck1.1,
            deck2_id: deck2.0,
            deck2_cards: deck2.1,
        })
    }

    /// Get the first valid deck for each faction.
    fn get_faction_decks(&self, factions: &[Faction]) -> HashMap<Faction, (String, Vec<CardId>)> {
        let mut faction_decks = HashMap::new();

        for faction in factions {
            if let Some(deck) = self.get_first_valid_deck(*faction) {
                faction_decks.insert(*faction, deck);
            }
        }

        faction_decks
    }

    /// Get the first valid deck for a faction.
    fn get_first_valid_deck(&self, faction: Faction) -> Option<(String, Vec<CardId>)> {
        let decks = self.registry.decks_for_faction(faction);
        for deck in decks {
            if deck.validate(self.card_db).is_ok() {
                return Some((deck.id.clone(), deck.to_card_ids()));
            }
        }
        None
    }
}

/// Filter matchups by name pattern.
///
/// Supports formats like:
/// - "argentum-symbiote" - exact matchup (either order)
/// - "argentum" - any matchup involving Argentum
pub fn filter_matchups(matchups: Vec<MatchupDefinition>, filter: &str) -> Vec<MatchupDefinition> {
    let filter_lower = filter.to_lowercase();
    let parts: Vec<&str> = filter_lower.split('-').collect();

    matchups
        .into_iter()
        .filter(|m| {
            let f1 = m.faction1.as_tag().to_lowercase();
            let f2 = m.faction2.as_tag().to_lowercase();

            // Match either order: "argentum-symbiote" or "symbiote-argentum"
            if parts.len() == 2 {
                (f1 == parts[0] && f2 == parts[1]) || (f1 == parts[1] && f2 == parts[0])
            } else {
                // Single faction name matches any matchup involving that faction
                f1.contains(&filter_lower) || f2.contains(&filter_lower)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_card_db() -> CardDatabase {
        CardDatabase::load_from_directory("data/cards/core_set").unwrap()
    }

    fn test_deck_registry() -> DeckRegistry {
        DeckRegistry::load_from_directory("data/decks").unwrap()
    }

    #[test]
    fn test_build_faction_matchups() {
        let card_db = test_card_db();
        let registry = test_deck_registry();
        let builder = MatchupBuilder::new(&registry, &card_db);

        let matchups = builder.build_faction_matchups();

        // Should have 3 matchups: A-S, A-O, S-O
        assert_eq!(matchups.len(), 3);

        // Verify all matchups have valid decks
        for m in &matchups {
            assert!(!m.deck1_cards.is_empty());
            assert!(!m.deck2_cards.is_empty());
        }
    }

    #[test]
    fn test_filter_matchups_exact() {
        let card_db = test_card_db();
        let registry = test_deck_registry();
        let builder = MatchupBuilder::new(&registry, &card_db);

        let matchups = builder.build_faction_matchups();
        let filtered = filter_matchups(matchups, "argentum-symbiote");

        assert_eq!(filtered.len(), 1);
        assert!(
            (filtered[0].faction1 == Faction::Argentum
                && filtered[0].faction2 == Faction::Symbiote)
                || (filtered[0].faction1 == Faction::Symbiote
                    && filtered[0].faction2 == Faction::Argentum)
        );
    }

    #[test]
    fn test_filter_matchups_single_faction() {
        let card_db = test_card_db();
        let registry = test_deck_registry();
        let builder = MatchupBuilder::new(&registry, &card_db);

        let matchups = builder.build_faction_matchups();
        let filtered = filter_matchups(matchups, "argentum");

        // Should match 2: A-S and A-O
        assert_eq!(filtered.len(), 2);
    }
}
