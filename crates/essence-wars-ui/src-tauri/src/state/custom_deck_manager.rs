//! Custom deck manager for saving, loading, listing, and validating custom decks.
//!
//! Custom decks are stored in `~/.essence-wars/custom_decks/` as TOML files.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use cardgame::{CardDatabase, CardType, DeckDefinition};
use cardgame::core::cards::Faction as CardFaction;
use cardgame::core::types::Rarity;
use cardgame::types::CardId;

use super::custom_deck::{
    copy_limit_for_rarity, BrowsableCard, CustomDeck, CustomDeckInfo,
    DeckValidation, PlaystyleScore, MAX_DECK_SIZE, MIN_DECK_SIZE,
};
use super::playstyle::calculate_playstyle;
use super::serialization::{card_type_to_string, faction_from_card_id, CommanderDto};

/// Manager for custom deck file operations.
#[derive(Clone)]
pub struct CustomDeckManager {
    decks_dir: PathBuf,
    card_db: Arc<CardDatabase>,
}

// CustomDeckManager is Send + Sync because it only holds Arc references and PathBuf
unsafe impl Send for CustomDeckManager {}
unsafe impl Sync for CustomDeckManager {}

impl CustomDeckManager {
    /// Create a new CustomDeckManager.
    ///
    /// Creates the custom_decks directory if it doesn't exist.
    pub fn new(card_db: Arc<CardDatabase>) -> Result<Self, String> {
        // Get the decks directory: ~/.essence-wars/custom_decks/
        let decks_dir = dirs::data_dir()
            .or_else(dirs::home_dir)
            .ok_or_else(|| "Could not determine home directory".to_string())?
            .join(".essence-wars")
            .join("custom_decks");

        // Create directory if it doesn't exist
        fs::create_dir_all(&decks_dir)
            .map_err(|e| format!("Failed to create custom_decks directory: {}", e))?;

        Ok(Self { decks_dir, card_db })
    }

    /// List all custom decks (metadata only).
    pub fn list_decks(&self) -> Result<Vec<CustomDeckInfo>, String> {
        let mut decks = Vec::new();

        let entries = fs::read_dir(&self.decks_dir)
            .map_err(|e| format!("Failed to read custom_decks directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(deck) = self.load_deck_from_path(&path) {
                    if let Some(info) = self.deck_to_info(&deck) {
                        decks.push(info);
                    }
                }
            }
        }

        // Sort by modified_at descending (newest first), fall back to name
        decks.sort_by(|a, b| {
            match (&b.modified_at, &a.modified_at) {
                (Some(b_time), Some(a_time)) => b_time.cmp(a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.name.cmp(&b.name),
            }
        });

        Ok(decks)
    }

    /// Load a custom deck by ID.
    pub fn load_deck(&self, id: &str) -> Result<CustomDeck, String> {
        let filename = format!("{}.toml", id);
        let path = self.decks_dir.join(&filename);

        if !path.exists() {
            return Err(format!("Custom deck '{}' not found", id));
        }

        self.load_deck_from_path(&path)
    }

    /// Save a custom deck.
    ///
    /// Returns the deck ID.
    pub fn save_deck(&self, deck: &CustomDeck) -> Result<String, String> {
        // Update metadata timestamps
        let mut deck = deck.clone();
        let now = chrono::Utc::now().to_rfc3339();

        if deck.metadata.created_at.is_none() {
            deck.metadata.created_at = Some(now.clone());
        }
        deck.metadata.modified_at = Some(now);
        deck.metadata.version = 1;

        // Ensure tags include faction
        let faction = self.get_commander_faction(deck.commander);
        if let Some(faction_tag) = faction {
            if !deck.tags.contains(&faction_tag) {
                deck.tags.push(faction_tag);
            }
        }
        if !deck.tags.contains(&"custom".to_string()) {
            deck.tags.push("custom".to_string());
        }

        let filename = format!("{}.toml", deck.id);
        let path = self.decks_dir.join(&filename);

        let content = toml::to_string_pretty(&deck)
            .map_err(|e| format!("Failed to serialize deck: {}", e))?;

        fs::write(&path, content)
            .map_err(|e| format!("Failed to write deck file: {}", e))?;

        Ok(deck.id.clone())
    }

    /// Delete a custom deck.
    pub fn delete_deck(&self, id: &str) -> Result<(), String> {
        let filename = format!("{}.toml", id);
        let path = self.decks_dir.join(&filename);

        if !path.exists() {
            return Err(format!("Custom deck '{}' not found", id));
        }

        fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete deck file: {}", e))?;

        Ok(())
    }

    /// Validate a custom deck.
    pub fn validate_deck(&self, deck: &CustomDeck) -> DeckValidation {
        let mut validation = DeckValidation::valid();

        // Check deck size
        let card_count = deck.cards.len();
        if card_count < MIN_DECK_SIZE {
            validation.add_error(format!(
                "Deck has {} cards, minimum is {}",
                card_count, MIN_DECK_SIZE
            ));
        }
        if card_count > MAX_DECK_SIZE {
            validation.add_error(format!(
                "Deck has {} cards, maximum is {}",
                card_count, MAX_DECK_SIZE
            ));
        }

        // Check commander exists
        let commander = self.card_db.get_commander(CardId(deck.commander));
        if commander.is_none() {
            validation.add_error(format!(
                "Commander {} not found",
                deck.commander
            ));
            return validation; // Can't continue without commander
        }
        let commander = commander.unwrap();
        let commander_faction = &commander.faction;

        // Check all cards exist and validate faction/copy limits
        let mut card_counts: HashMap<u16, u8> = HashMap::new();

        for &card_id in &deck.cards {
            *card_counts.entry(card_id).or_insert(0) += 1;

            if let Some(card) = self.card_db.get(CardId(card_id)) {
                // Check faction restriction
                let card_faction = faction_from_card_id(card_id);
                let commander_faction_str = faction_to_string(commander_faction);

                if card_faction != "neutral" && card_faction != commander_faction_str {
                    validation.add_error(format!(
                        "Card '{}' ({}) doesn't match commander faction ({})",
                        card.name, card_faction, commander_faction_str
                    ));
                }

                // Check copy limits
                let copies = card_counts[&card_id];
                let limit = copy_limit_for_rarity(card.rarity);
                if copies > limit {
                    validation.add_error(format!(
                        "Card '{}' has {} copies, maximum for {:?} is {}",
                        card.name, copies, card.rarity, limit
                    ));
                }
            } else {
                validation.add_error(format!("Card {} not found", card_id));
            }
        }

        // Add warnings for unusual deck compositions
        if card_count < 35 {
            validation.add_warning("Deck has fewer than 35 cards, which may be inconsistent".to_string());
        }

        // Check for very low curve (warning, not error)
        let avg_cost = self.calculate_avg_cost(&deck.cards);
        if avg_cost < 2.0 {
            validation.add_warning(format!(
                "Very low average mana cost ({:.1}), deck may lack late-game options",
                avg_cost
            ));
        } else if avg_cost > 4.5 {
            validation.add_warning(format!(
                "High average mana cost ({:.1}), deck may struggle early game",
                avg_cost
            ));
        }

        validation
    }

    /// Calculate playstyle for a deck.
    pub fn calculate_playstyle(&self, cards: &[u16], commander_id: u16) -> PlaystyleScore {
        calculate_playstyle(cards, commander_id, &self.card_db)
    }

    /// Convert a CustomDeck to a DeckDefinition for use in games.
    pub fn to_deck_definition(&self, deck: &CustomDeck) -> Result<DeckDefinition, String> {
        // Calculate playstyle for the playstyle field
        let playstyle = self.calculate_playstyle(&deck.cards, deck.commander);
        let playstyle_str = format!("{:?}", playstyle.primary);

        Ok(DeckDefinition {
            id: deck.id.clone(),
            name: deck.name.clone(),
            description: deck.description.clone(),
            playstyle: playstyle_str,
            commander: deck.commander,
            cards: deck.cards.clone(),
            tags: deck.tags.clone(),
        })
    }

    /// Get all cards browsable by the deck builder (all factions + neutral).
    pub fn list_all_cards(&self) -> Vec<BrowsableCard> {
        self.card_db
            .iter()
            .map(|card| self.card_to_browsable(card))
            .collect()
    }

    /// Get cards filtered by faction (faction + neutral).
    pub fn list_cards_for_faction(&self, faction: &str) -> Vec<BrowsableCard> {
        self.card_db
            .iter()
            .filter(|card| {
                let card_faction = faction_from_card_id(card.id);
                card_faction == faction || card_faction == "neutral"
            })
            .map(|card| self.card_to_browsable(card))
            .collect()
    }

    /// Get all commanders.
    pub fn list_commanders(&self) -> Vec<CommanderDto> {
        self.card_db
            .iter_commanders()
            .map(CommanderDto::from_commander_def)
            .collect()
    }

    // --- Private helpers ---

    fn load_deck_from_path(&self, path: &PathBuf) -> Result<CustomDeck, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read deck file: {}", e))?;

        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse deck file: {}", e))
    }

    fn deck_to_info(&self, deck: &CustomDeck) -> Option<CustomDeckInfo> {
        let commander = self.card_db.get_commander(CardId(deck.commander))?;
        let faction = faction_to_string(&commander.faction);
        let playstyle = Some(self.calculate_playstyle(&deck.cards, deck.commander));

        Some(CustomDeckInfo {
            id: deck.id.clone(),
            name: deck.name.clone(),
            description: deck.description.clone(),
            commander_id: deck.commander,
            commander_name: commander.name.clone(),
            faction: faction.to_string(),
            card_count: deck.cards.len(),
            playstyle,
            created_at: deck.metadata.created_at.clone(),
            modified_at: deck.metadata.modified_at.clone(),
        })
    }

    fn card_to_browsable(&self, card: &cardgame::CardDefinition) -> BrowsableCard {
        let faction = faction_from_card_id(card.id);
        let card_type_str = card_type_to_string(&card.card_type);
        let rarity_str = rarity_to_string(card.rarity);

        let (attack, health, keywords) = match &card.card_type {
            CardType::Creature { attack, health, keywords, .. } => {
                (Some(*attack), Some(*health), keywords.clone())
            }
            _ => (None, None, Vec::new()),
        };

        let durability = card.durability();
        let copy_limit = copy_limit_for_rarity(card.rarity);
        let art_path = format!("cards/core_set/{}.webp", card.id);

        // Generate effect description for spells/supports
        let effect_description = match &card.card_type {
            CardType::Spell { .. } | CardType::Support { .. } => {
                Some(self.generate_effect_description(card))
            }
            _ => None,
        };

        BrowsableCard {
            card_id: card.id,
            name: card.name.clone(),
            cost: card.cost,
            card_type: card_type_str.to_string(),
            faction: faction.to_string(),
            rarity: rarity_str.to_string(),
            attack,
            health,
            keywords,
            durability,
            art_path,
            copy_limit,
            effect_description,
        }
    }

    fn generate_effect_description(&self, card: &cardgame::CardDefinition) -> String {
        // Simple description generation - could be enhanced later
        match &card.card_type {
            CardType::Spell { effects, .. } => {
                if effects.is_empty() {
                    "No effects".to_string()
                } else {
                    format!("{} effect(s)", effects.len())
                }
            }
            CardType::Support { passive_effects, triggered_effects, .. } => {
                let passive_count = passive_effects.len();
                let triggered_count = triggered_effects.len();
                if passive_count == 0 && triggered_count == 0 {
                    "No effects".to_string()
                } else if passive_count > 0 && triggered_count > 0 {
                    format!("{} passive, {} triggered", passive_count, triggered_count)
                } else if passive_count > 0 {
                    format!("{} passive effect(s)", passive_count)
                } else {
                    format!("{} triggered effect(s)", triggered_count)
                }
            }
            _ => "".to_string(),
        }
    }

    fn get_commander_faction(&self, commander_id: u16) -> Option<String> {
        self.card_db
            .get_commander(CardId(commander_id))
            .map(|c| faction_to_string(&c.faction).to_string())
    }

    fn calculate_avg_cost(&self, cards: &[u16]) -> f32 {
        if cards.is_empty() {
            return 0.0;
        }

        let total: u32 = cards
            .iter()
            .filter_map(|&id| self.card_db.get(CardId(id)))
            .map(|c| c.cost as u32)
            .sum();

        total as f32 / cards.len() as f32
    }
}

/// Convert Faction enum to string
fn faction_to_string(faction: &CardFaction) -> &'static str {
    match faction {
        CardFaction::Argentum => "argentum",
        CardFaction::Symbiote => "symbiote",
        CardFaction::Obsidion => "obsidion",
        CardFaction::Neutral => "neutral",
    }
}

/// Convert Rarity enum to string
fn rarity_to_string(rarity: Rarity) -> &'static str {
    match rarity {
        Rarity::Common => "Common",
        Rarity::Uncommon => "Uncommon",
        Rarity::Rare => "Rare",
        Rarity::Legendary => "Legendary",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faction_to_string() {
        assert_eq!(faction_to_string(&CardFaction::Argentum), "argentum");
        assert_eq!(faction_to_string(&CardFaction::Symbiote), "symbiote");
        assert_eq!(faction_to_string(&CardFaction::Obsidion), "obsidion");
        assert_eq!(faction_to_string(&CardFaction::Neutral), "neutral");
    }

    #[test]
    fn test_rarity_to_string() {
        assert_eq!(rarity_to_string(Rarity::Common), "Common");
        assert_eq!(rarity_to_string(Rarity::Uncommon), "Uncommon");
        assert_eq!(rarity_to_string(Rarity::Rare), "Rare");
        assert_eq!(rarity_to_string(Rarity::Legendary), "Legendary");
    }
}
