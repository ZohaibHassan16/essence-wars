//! Embedded game data for builds without filesystem access.
//!
//! This module embeds card and deck data at compile time, useful for
//! Python bindings or other scenarios where filesystem access is limited.

use crate::cards::{CardDatabase, CardDefinition, CommanderDefinition};
use crate::decks::{DeckDefinition, DeckRegistry};

// Embed card YAML files at compile time (organized by faction and card type)
// Argentum
const ARGENTUM_CREATURES_YAML: &str = include_str!("../../../data/cards/core_set/argentum/creatures.yaml");
const ARGENTUM_SPELLS_YAML: &str = include_str!("../../../data/cards/core_set/argentum/spells.yaml");
const ARGENTUM_SUPPORTS_YAML: &str = include_str!("../../../data/cards/core_set/argentum/supports.yaml");
// Symbiote
const SYMBIOTE_CREATURES_YAML: &str = include_str!("../../../data/cards/core_set/symbiote/creatures.yaml");
const SYMBIOTE_SPELLS_YAML: &str = include_str!("../../../data/cards/core_set/symbiote/spells.yaml");
const SYMBIOTE_SUPPORTS_YAML: &str = include_str!("../../../data/cards/core_set/symbiote/supports.yaml");
// Obsidion
const OBSIDION_CREATURES_YAML: &str = include_str!("../../../data/cards/core_set/obsidion/creatures.yaml");
const OBSIDION_SPELLS_YAML: &str = include_str!("../../../data/cards/core_set/obsidion/spells.yaml");
const OBSIDION_SUPPORTS_YAML: &str = include_str!("../../../data/cards/core_set/obsidion/supports.yaml");
// Neutral
const NEUTRAL_CREATURES_YAML: &str = include_str!("../../../data/cards/core_set/neutral/creatures.yaml");
const NEUTRAL_SPELLS_YAML: &str = include_str!("../../../data/cards/core_set/neutral/spells.yaml");
const NEUTRAL_SUPPORTS_YAML: &str = include_str!("../../../data/cards/core_set/neutral/supports.yaml");

// Embed commander YAML files at compile time
const ARGENTUM_COMMANDERS_YAML: &str = include_str!("../../../data/commanders/argentum.yaml");
const SYMBIOTE_COMMANDERS_YAML: &str = include_str!("../../../data/commanders/symbiote.yaml");
const OBSIDION_COMMANDERS_YAML: &str = include_str!("../../../data/commanders/obsidion.yaml");

// Embed deck TOML files at compile time
// Argentum decks
const ARGENTUM_ARCHITECT_TOML: &str = include_str!("../../../data/decks/argentum/architect_fortify.toml");
const ARGENTUM_SANCTUM_HEALER_TOML: &str = include_str!("../../../data/decks/argentum/sanctum_healer.toml");
const ARGENTUM_VEX_TOML: &str = include_str!("../../../data/decks/argentum/vex_piercing.toml");
const ARGENTUM_ARTIFICER_TOML: &str = include_str!("../../../data/decks/argentum/artificer_tokens.toml");

// Symbiote decks
const SYMBIOTE_BROODMOTHER_TOML: &str = include_str!("../../../data/decks/symbiote/broodmother_pack.toml");
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
    let mut all_cards: Vec<CardDefinition> = Vec::with_capacity(320);

    // All embedded card YAML files (organized by faction and card type)
    let card_yamls: &[(&str, &str)] = &[
        // Argentum
        ("Argentum creatures", ARGENTUM_CREATURES_YAML),
        ("Argentum spells", ARGENTUM_SPELLS_YAML),
        ("Argentum supports", ARGENTUM_SUPPORTS_YAML),
        // Symbiote
        ("Symbiote creatures", SYMBIOTE_CREATURES_YAML),
        ("Symbiote spells", SYMBIOTE_SPELLS_YAML),
        ("Symbiote supports", SYMBIOTE_SUPPORTS_YAML),
        // Obsidion
        ("Obsidion creatures", OBSIDION_CREATURES_YAML),
        ("Obsidion spells", OBSIDION_SPELLS_YAML),
        ("Obsidion supports", OBSIDION_SUPPORTS_YAML),
        // Neutral
        ("Neutral creatures", NEUTRAL_CREATURES_YAML),
        ("Neutral spells", NEUTRAL_SPELLS_YAML),
        ("Neutral supports", NEUTRAL_SUPPORTS_YAML),
    ];

    for (name, yaml_str) in card_yamls {
        let faction_cards: FactionCards = serde_yaml::from_str(yaml_str)
            .map_err(|e| format!("Failed to parse {}: {}", name, e))?;
        all_cards.extend(faction_cards.cards);
    }

    Ok(CardDatabase::new(all_cards))
}

/// Wrapper struct for the commander file format.
/// The YAML files have a structure like: { commanders: [...] }
#[derive(serde::Deserialize)]
struct CommanderFile {
    commanders: Vec<CommanderDefinition>,
}

/// Load embedded commanders from YAML data.
fn load_embedded_commanders() -> Result<Vec<CommanderDefinition>, String> {
    let commander_yamls = [
        ("Argentum", ARGENTUM_COMMANDERS_YAML),
        ("Symbiote", SYMBIOTE_COMMANDERS_YAML),
        ("Obsidion", OBSIDION_COMMANDERS_YAML),
    ];

    let mut all_commanders = Vec::with_capacity(12);
    for (name, yaml_str) in commander_yamls {
        let file: CommanderFile = serde_yaml::from_str(yaml_str)
            .map_err(|e| format!("Failed to parse {} commanders: {}", name, e))?;
        all_commanders.extend(file.commanders);
    }

    Ok(all_commanders)
}

/// Load the complete card database with commanders from embedded data.
///
/// This is the main function for Python/WASM builds that need embedded data.
pub fn load_embedded_cards_with_commanders() -> Result<CardDatabase, String> {
    let mut all_cards: Vec<CardDefinition> = Vec::with_capacity(320);

    let card_yamls: &[(&str, &str)] = &[
        ("Argentum creatures", ARGENTUM_CREATURES_YAML),
        ("Argentum spells", ARGENTUM_SPELLS_YAML),
        ("Argentum supports", ARGENTUM_SUPPORTS_YAML),
        ("Symbiote creatures", SYMBIOTE_CREATURES_YAML),
        ("Symbiote spells", SYMBIOTE_SPELLS_YAML),
        ("Symbiote supports", SYMBIOTE_SUPPORTS_YAML),
        ("Obsidion creatures", OBSIDION_CREATURES_YAML),
        ("Obsidion spells", OBSIDION_SPELLS_YAML),
        ("Obsidion supports", OBSIDION_SUPPORTS_YAML),
        ("Neutral creatures", NEUTRAL_CREATURES_YAML),
        ("Neutral spells", NEUTRAL_SPELLS_YAML),
        ("Neutral supports", NEUTRAL_SUPPORTS_YAML),
    ];

    for (name, yaml_str) in card_yamls {
        let faction_cards: FactionCards = serde_yaml::from_str(yaml_str)
            .map_err(|e| format!("Failed to parse {}: {}", name, e))?;
        all_cards.extend(faction_cards.cards);
    }

    let commanders = load_embedded_commanders()?;
    Ok(CardDatabase::new_with_commanders(all_cards, commanders))
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
        ("broodmother_pack", SYMBIOTE_BROODMOTHER_TOML),
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
        "broodmother_pack",
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
        "broodmother_pack",   // The Broodmother
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

    #[test]
    fn test_load_embedded_cards_with_commanders() {
        use crate::cards::Faction;

        // This is the function Python bindings use - must test it!
        let card_db = load_embedded_cards_with_commanders()
            .expect("Failed to load embedded cards with commanders");

        // Verify cards loaded
        assert!(card_db.len() >= 300, "Expected at least 300 cards, got {}", card_db.len());

        // Verify commanders loaded (12 total: 4 per faction × 3 factions)
        let commander_count = card_db.commander_count();
        assert_eq!(commander_count, 12, "Expected 12 commanders, got {}", commander_count);

        // Verify commanders from each faction
        let argentum_count = card_db.iter_commanders()
            .filter(|c| c.faction == Faction::Argentum).count();
        let symbiote_count = card_db.iter_commanders()
            .filter(|c| c.faction == Faction::Symbiote).count();
        let obsidion_count = card_db.iter_commanders()
            .filter(|c| c.faction == Faction::Obsidion).count();

        assert_eq!(argentum_count, 4, "Expected 4 Argentum commanders");
        assert_eq!(symbiote_count, 4, "Expected 4 Symbiote commanders");
        assert_eq!(obsidion_count, 4, "Expected 4 Obsidion commanders");
    }

    #[test]
    fn test_embedded_commanders_have_abilities() {
        // Ensure commander data is complete (not just parsed but valid)
        let card_db = load_embedded_cards_with_commanders()
            .expect("Failed to load cards with commanders");

        for commander in card_db.iter_commanders() {
            // Every commander should have either a passive or triggered ability
            let has_passive = commander.has_passive();
            let has_triggered = commander.has_triggered();
            assert!(
                has_passive || has_triggered,
                "Commander {} has no abilities", commander.name
            );
        }
    }
}
