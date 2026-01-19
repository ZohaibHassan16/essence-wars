//! Deck loading and validation utilities for arena matches.
//!
//! Provides helpers for loading decks from the registry, validating
//! faction-specialist bindings, and creating default decks.

use crate::bots::BotType;
use crate::cards::CardDatabase;
use crate::decks::DeckRegistry;
use crate::types::CardId;

/// Result of loading a deck.
#[derive(Debug)]
pub struct LoadedDeck {
    /// The deck's card IDs.
    pub cards: Vec<CardId>,
    /// Display name for the deck.
    pub name: String,
}

/// Load a deck by ID or use default.
///
/// # Arguments
/// * `deck_id` - Optional deck ID to load
/// * `registry` - The deck registry to search
/// * `card_db` - Card database for validation
/// * `player_label` - Label for error messages (e.g., "1" or "2")
///
/// # Returns
/// * `Ok(LoadedDeck)` - The loaded deck with cards and name
/// * `Err(String)` - Error message if deck not found or invalid
pub fn load_deck(
    deck_id: Option<&str>,
    registry: &DeckRegistry,
    card_db: &CardDatabase,
    player_label: &str,
) -> Result<LoadedDeck, String> {
    match deck_id {
        Some(id) => match registry.get(id) {
            Some(deck) => {
                if let Err(e) = deck.validate(card_db) {
                    return Err(format!("Deck '{}' validation error: {}", id, e));
                }
                Ok(LoadedDeck {
                    cards: deck.to_card_ids(),
                    name: deck.name.clone(),
                })
            }
            None => Err(format!(
                "Deck '{}' not found. Use --list-decks to see available decks.",
                id
            )),
        },
        None => Ok(LoadedDeck {
            cards: create_default_deck(),
            name: format!("Default Deck {}", player_label),
        }),
    }
}

/// Validate that specialist agents are paired with their faction's decks.
///
/// Specialist agents should only play decks of their faction.
/// Returns a warning message if there's a mismatch, or None if valid.
///
/// # Arguments
/// * `bot_type` - The bot type being validated
/// * `deck_id` - Optional deck ID being used
/// * `deck_registry` - Registry to look up deck faction
/// * `bot_label` - Label for warning messages (e.g., "Bot 1")
///
/// # Returns
/// * `Some(String)` - Warning message if mismatch detected
/// * `None` - No issues found
pub fn validate_faction_deck_binding(
    bot_type: &BotType,
    deck_id: Option<&str>,
    deck_registry: &DeckRegistry,
    bot_label: &str,
) -> Option<String> {
    // Only validate for specialist agents
    let specialist_faction = match bot_type {
        BotType::AgentSpecialist(faction) => faction,
        _ => return None,
    };

    // Only validate if a specific deck was chosen
    let deck_id = deck_id?;

    // Get the deck and check its faction
    let deck = deck_registry.get(deck_id)?;

    match deck.faction() {
        Some(deck_faction) => {
            if deck_faction != *specialist_faction {
                Some(format!(
                    "Warning: {} ({}) is using a {} deck ('{}'), but specialists work best with their faction's decks.\n  Recommended: Use a {} deck for {} specialists.",
                    bot_label,
                    bot_type.name(),
                    deck_faction.display_name(),
                    deck_id,
                    specialist_faction.display_name(),
                    specialist_faction.display_name()
                ))
            } else {
                None
            }
        }
        None => Some(format!(
            "Warning: {} ({}) is using a non-faction deck ('{}').\n  Recommended: Use a {} deck for {} specialists.",
            bot_label,
            bot_type.name(),
            deck_id,
            specialist_faction.display_name(),
            specialist_faction.display_name()
        )),
    }
}

/// Create a default deck for testing.
///
/// Returns a simple deck with a mix of creatures for basic testing
/// when no specific deck is specified.
pub fn create_default_deck() -> Vec<CardId> {
    // Aggressive Assault deck from design doc (simplified)
    let card_ids = [
        1, 1, // Eager Recruit x2
        3, 3, // Nimble Scout x2
        6, 6, // Frontier Ranger x2
        8, 8, // Shielded Squire x2
        11, 11, // Centaur Charger x2
        12, 12, // Blade Dancer x2
        16, 16, // Piercing Striker x2
        20, 20, // Siege Breaker x2
        34, 34, // Lightning Bolt x2
    ];
    card_ids.iter().map(|&id| CardId(id)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_deck_size() {
        let deck = create_default_deck();
        assert_eq!(deck.len(), 18);
    }

    #[test]
    fn test_load_deck_not_found() {
        let registry = DeckRegistry::new();
        let cards_path = crate::data_dir().join("cards/core_set");
        let card_db = CardDatabase::load_from_directory(cards_path).unwrap();
        let result = load_deck(Some("nonexistent"), &registry, &card_db, "1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_load_deck_default() {
        let registry = DeckRegistry::new();
        let cards_path = crate::data_dir().join("cards/core_set");
        let card_db = CardDatabase::load_from_directory(cards_path).unwrap();
        let result = load_deck(None, &registry, &card_db, "1");
        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.name, "Default Deck 1");
        assert_eq!(loaded.cards.len(), 18);
    }

    #[test]
    fn test_validate_faction_binding_random_bot() {
        let registry = DeckRegistry::new();
        let result = validate_faction_deck_binding(
            &BotType::Random,
            Some("argentum_control"),
            &registry,
            "Bot 1",
        );
        // Random bot should not trigger validation
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_faction_binding_no_deck() {
        let registry = DeckRegistry::new();
        let result = validate_faction_deck_binding(
            &BotType::AgentSpecialist(crate::decks::Faction::Argentum),
            None,
            &registry,
            "Bot 1",
        );
        // No deck specified should not trigger validation
        assert!(result.is_none());
    }
}
