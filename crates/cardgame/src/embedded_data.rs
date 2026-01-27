//! Embedded game data for WASM/web builds.
//!
//! This module embeds card and deck data at compile time, eliminating
//! the need for filesystem access in browser environments.
//!
//! Only compiled when the `web` feature is enabled.

use crate::cards::{CardDatabase, CardDefinition};
use crate::decks::{DeckDefinition, DeckRegistry};

// Embed card YAML files at compile time
const ARGENTUM_CARDS_YAML: &str = include_str!("../../../data/cards/core_set/argentum.yaml");
const SYMBIOTE_CARDS_YAML: &str = include_str!("../../../data/cards/core_set/symbiote.yaml");
const OBSIDION_CARDS_YAML: &str = include_str!("../../../data/cards/core_set/obsidion.yaml");
const NEUTRAL_CARDS_YAML: &str = include_str!("../../../data/cards/core_set/neutral.yaml");

// Embed deck TOML files at compile time
// Argentum decks
const ARGENTUM_ARCHITECT_TOML: &str = include_str!("../../../data/decks/argentum/architect_fortify.toml");
const ARGENTUM_SANCTUM_HEALER_TOML: &str = include_str!("../../../data/decks/argentum/sanctum_healer.toml");
const ARGENTUM_VEX_TOML: &str = include_str!("../../../data/decks/argentum/vex_piercing.toml");
const ARGENTUM_ARTIFICER_TOML: &str = include_str!("../../../data/decks/argentum/artificer_tokens.toml");

// Symbiote decks
const SYMBIOTE_BROODMOTHER_TOML: &str = include_str!("../../../data/decks/symbiote/broodmother_swarm.toml");
const SYMBIOTE_ALPHA_TOML: &str = include_str!("../../../data/decks/symbiote/alpha_frenzy.toml");
const SYMBIOTE_GROVE_TOML: &str = include_str!("../../../data/decks/symbiote/grove_regenerate.toml");
const SYMBIOTE_PLAGUE_TOML: &str = include_str!("../../../data/decks/symbiote/plague_volatile.toml");

// Obsidion decks
const OBSIDION_SHADOW_WEAVER_TOML: &str = include_str!("../../../data/decks/obsidion/shadow_weaver.toml");
const OBSIDION_SOVEREIGN_TOML: &str = include_str!("../../../data/decks/obsidion/sovereign_lifesteal.toml");
const OBSIDION_DEATHMASTER_TOML: &str = include_str!("../../../data/decks/obsidion/deathmaster_assassin.toml");
const OBSIDION_ARCHON_TOML: &str = include_str!("../../../data/decks/obsidion/archon_burst.toml");

/// Wrapper struct for the faction card file format.
/// The YAML files have a structure like: { name: "Faction", cards: [...] }
#[derive(serde::Deserialize)]
struct FactionCards {
    #[allow(dead_code)]
    name: String,
    cards: Vec<CardDefinition>,
}

/// Load the complete card database from embedded YAML data.
///
/// This function parses all four faction card files that are embedded
/// at compile time and returns a complete CardDatabase.
///
/// # Example
/// ```ignore
/// use cardgame::embedded_data::load_embedded_cards;
///
/// let card_db = load_embedded_cards().expect("Failed to load cards");
/// assert!(card_db.len() >= 300);
/// ```
pub fn load_embedded_cards() -> Result<CardDatabase, String> {
    let mut all_cards: Vec<CardDefinition> = Vec::with_capacity(300);

    // Parse each faction's cards (YAML files have { name, cards: [...] } structure)
    let argentum: FactionCards = serde_yaml::from_str(ARGENTUM_CARDS_YAML)
        .map_err(|e| format!("Failed to parse Argentum cards: {}", e))?;
    let symbiote: FactionCards = serde_yaml::from_str(SYMBIOTE_CARDS_YAML)
        .map_err(|e| format!("Failed to parse Symbiote cards: {}", e))?;
    let obsidion: FactionCards = serde_yaml::from_str(OBSIDION_CARDS_YAML)
        .map_err(|e| format!("Failed to parse Obsidion cards: {}", e))?;
    let neutral: FactionCards = serde_yaml::from_str(NEUTRAL_CARDS_YAML)
        .map_err(|e| format!("Failed to parse Neutral cards: {}", e))?;

    all_cards.extend(argentum.cards);
    all_cards.extend(symbiote.cards);
    all_cards.extend(obsidion.cards);
    all_cards.extend(neutral.cards);

    Ok(CardDatabase::new(all_cards))
}

/// Load all embedded decks into a DeckRegistry.
///
/// This function parses all 12 commander deck TOML files that are embedded
/// at compile time and returns a complete DeckRegistry.
///
/// # Example
/// ```ignore
/// use cardgame::embedded_data::load_embedded_decks;
///
/// let deck_registry = load_embedded_decks().expect("Failed to load decks");
/// assert_eq!(deck_registry.len(), 12);
/// ```
pub fn load_embedded_decks() -> Result<DeckRegistry, String> {
    let mut registry = DeckRegistry::new();

    // All embedded deck TOML strings with their IDs
    let deck_tomls = [
        // Argentum
        ("architect_fortify", ARGENTUM_ARCHITECT_TOML),
        ("sanctum_healer", ARGENTUM_SANCTUM_HEALER_TOML),
        ("vex_piercing", ARGENTUM_VEX_TOML),
        ("artificer_tokens", ARGENTUM_ARTIFICER_TOML),
        // Symbiote
        ("broodmother_swarm", SYMBIOTE_BROODMOTHER_TOML),
        ("alpha_frenzy", SYMBIOTE_ALPHA_TOML),
        ("grove_regenerate", SYMBIOTE_GROVE_TOML),
        ("plague_volatile", SYMBIOTE_PLAGUE_TOML),
        // Obsidion
        ("shadow_weaver", OBSIDION_SHADOW_WEAVER_TOML),
        ("sovereign_lifesteal", OBSIDION_SOVEREIGN_TOML),
        ("deathmaster_assassin", OBSIDION_DEATHMASTER_TOML),
        ("archon_burst", OBSIDION_ARCHON_TOML),
    ];

    for (id, toml_str) in deck_tomls {
        let deck: DeckDefinition = toml::from_str(toml_str)
            .map_err(|e| format!("Failed to parse deck {}: {}", id, e))?;
        registry.add(deck)
            .map_err(|e| format!("Failed to add deck {}: {}", id, e))?;
    }

    Ok(registry)
}

/// Load both cards and decks from embedded data.
///
/// Convenience function that loads both the card database and deck registry
/// from embedded data in a single call.
///
/// # Returns
/// A tuple of (CardDatabase, DeckRegistry)
pub fn load_embedded_game_data() -> Result<(CardDatabase, DeckRegistry), String> {
    let cards = load_embedded_cards()?;
    let decks = load_embedded_decks()?;
    Ok((cards, decks))
}

/// Get the list of available deck IDs.
///
/// Returns the IDs of all 12 commander decks that are embedded in the build.
pub fn get_embedded_deck_ids() -> &'static [&'static str] {
    &[
        // Argentum
        "architect_fortify",
        "sanctum_healer",
        "vex_piercing",
        "artificer_tokens",
        // Symbiote
        "broodmother_swarm",
        "alpha_frenzy",
        "grove_regenerate",
        "plague_volatile",
        // Obsidion
        "shadow_weaver",
        "sovereign_lifesteal",
        "deathmaster_assassin",
        "archon_burst",
    ]
}

/// Get the MVP deck IDs (one per faction) for the vertical slice.
///
/// Returns the three decks chosen for the Phase 5A MVP:
/// - The Sanctum Healer (Argentum - defensive)
/// - The Broodmother (Symbiote - aggressive)
/// - The Blood Sovereign (Obsidion - sustain)
pub fn get_mvp_deck_ids() -> &'static [&'static str] {
    &[
        "sanctum_healer",      // The Sanctum Healer
        "broodmother_swarm",   // The Broodmother
        "sovereign_lifesteal", // The Blood Sovereign
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_embedded_cards() {
        let card_db = load_embedded_cards().expect("Failed to load embedded cards");
        assert!(card_db.len() >= 300, "Expected at least 300 cards, got {}", card_db.len());
    }

    #[test]
    fn test_load_embedded_decks() {
        let registry = load_embedded_decks().expect("Failed to load embedded decks");
        assert_eq!(registry.len(), 12, "Expected 12 decks");
    }

    #[test]
    fn test_load_embedded_game_data() {
        let (cards, decks) = load_embedded_game_data().expect("Failed to load game data");
        assert!(cards.len() >= 300);
        assert_eq!(decks.len(), 12);
    }

    #[test]
    fn test_mvp_decks_exist() {
        let registry = load_embedded_decks().expect("Failed to load decks");
        for deck_id in get_mvp_deck_ids() {
            assert!(registry.get(deck_id).is_some(), "MVP deck {} not found", deck_id);
        }
    }

    #[test]
    fn test_all_deck_ids_valid() {
        let registry = load_embedded_decks().expect("Failed to load decks");
        for deck_id in get_embedded_deck_ids() {
            assert!(registry.get(deck_id).is_some(), "Deck {} not found", deck_id);
        }
    }
}
