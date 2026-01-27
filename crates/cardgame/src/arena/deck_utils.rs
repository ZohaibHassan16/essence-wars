//! Deck loading and validation utilities for arena matches.
//!
//! Provides helpers for loading decks from the registry and validating
//! faction-specialist bindings.

use crate::bots::BotType;
use crate::cards::CardDatabase;
use crate::decks::{DeckDefinition, DeckRegistry};

/// Load a deck by ID.
///
/// # Arguments
/// * `deck_id` - Optional deck ID to load
/// * `registry` - The deck registry to search
/// * `card_db` - Card database for validation
/// * `player_label` - Label for error messages (e.g., "1" or "2")
///
/// # Returns
/// * `Ok(DeckDefinition)` - The loaded deck definition (includes commander)
/// * `Err(String)` - Error message if deck not specified, not found, or invalid
pub fn load_deck(
    deck_id: Option<&str>,
    registry: &DeckRegistry,
    card_db: &CardDatabase,
    player_label: &str,
) -> Result<DeckDefinition, String> {
    match deck_id {
        Some(id) => match registry.get(id) {
            Some(deck) => {
                if let Err(e) = deck.validate(card_db) {
                    return Err(format!("Deck '{}' validation error: {}", id, e));
                }
                Ok(deck.clone())
            }
            None => Err(format!(
                "Deck '{}' not found. Use --list-decks to see available decks.",
                id
            )),
        },
        None => Err(format!(
            "No deck specified for player {}. Use --deck{} <DECK_ID> to specify a deck, or --list-decks to see available decks.",
            player_label, player_label
        )),
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_load_deck_none_returns_error() {
        let registry = DeckRegistry::new();
        let cards_path = crate::data_dir().join("cards/core_set");
        let card_db = CardDatabase::load_from_directory(cards_path).unwrap();
        let result = load_deck(None, &registry, &card_db, "1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No deck specified"));
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
