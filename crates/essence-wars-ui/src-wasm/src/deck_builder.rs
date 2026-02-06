//! Deck builder functionality for WASM.
//!
//! Provides card browsing, commander listing, and deck validation.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use cardgame::cards::CardDefinition;

use crate::types::{faction_from_card_id, card_faction_to_string, CommanderDto};
use crate::WasmGameManager;

// =============================================================================
// Deck Builder Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowsableCard {
    pub id: u16,
    pub name: String,
    pub cost: u8,
    pub card_type: String,
    pub faction: String,
    pub description: String,
    pub flavor_text: String,
    pub rarity: String,
    pub attack: Option<i8>,
    pub health: Option<i8>,
    pub keywords: Vec<String>,
    /// Whether this card can be added to a deck of the given faction
    pub is_playable: bool,
}

impl BrowsableCard {
    pub fn from_card(card: &CardDefinition, target_faction: Option<&str>) -> Self {
        // Use card methods to get optional stats
        let attack = card.attack().map(|a| a as i8);
        let health = card.health().map(|h| h as i8);

        let card_faction = faction_from_card_id(card.id);

        // Get card type name
        let card_type = match &card.card_type {
            cardgame::cards::CardType::Creature { .. } => "Creature",
            cardgame::cards::CardType::Spell { .. } => "Spell",
            cardgame::cards::CardType::Support { .. } => "Support",
        };

        // A card is playable if it's neutral or matches the target faction
        let is_playable = target_faction
            .map(|f| card_faction == "neutral" || card_faction == f)
            .unwrap_or(true);

        Self {
            id: card.id,
            name: card.name.clone(),
            cost: card.cost,
            card_type: card_type.to_string(),
            faction: card_faction.to_string(),
            description: String::new(), // Not stored in CardDefinition
            flavor_text: String::new(), // Not stored in CardDefinition
            rarity: format!("{:?}", card.rarity),
            attack,
            health,
            keywords: card.keywords().to_names().iter().map(|s| s.to_string()).collect(),
            is_playable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomDeck {
    pub id: String,
    pub name: String,
    pub commander: u16,
    pub cards: Vec<u16>,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaystyleScore {
    pub playstyle: String,
    pub aggro_score: f32,
    pub control_score: f32,
    pub tempo_score: f32,
    pub midrange_score: f32,
}

// =============================================================================
// Deck Builder Implementation
// =============================================================================

#[wasm_bindgen]
impl WasmGameManager {
    /// List all cards in the game, optionally filtered by faction.
    ///
    /// If faction is provided, returns cards from that faction plus neutral.
    /// Returns a JSON array of BrowsableCard objects.
    #[wasm_bindgen]
    pub fn list_all_cards(&self, faction: Option<String>) -> String {
        let target_faction_str = faction.as_deref();

        let cards: Vec<BrowsableCard> = self
            .card_db
            .iter()
            .filter(|card| {
                // If faction specified, only show that faction + neutral
                target_faction_str
                    .map(|f| {
                        let card_faction = faction_from_card_id(card.id);
                        card_faction == "neutral" || card_faction == f
                    })
                    .unwrap_or(true)
            })
            .map(|card| BrowsableCard::from_card(card, target_faction_str))
            .collect();

        serde_json::to_string(&cards).unwrap_or_else(|_| "[]".to_string())
    }

    /// List all commanders.
    ///
    /// Returns a JSON array of CommanderDto objects.
    #[wasm_bindgen]
    pub fn list_commanders(&self) -> String {
        let commanders: Vec<CommanderDto> = self
            .card_db
            .iter_commanders()
            .map(CommanderDto::from_commander)
            .collect();

        serde_json::to_string(&commanders).unwrap_or_else(|_| "[]".to_string())
    }

    /// Validate a custom deck configuration.
    ///
    /// Returns a JSON DeckValidation object.
    #[wasm_bindgen]
    pub fn validate_custom_deck(&self, deck_json: &str) -> Result<String, JsError> {
        let deck: CustomDeck = serde_json::from_str(deck_json)
            .map_err(|e| JsError::new(&format!("Invalid deck: {}", e)))?;

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check commander exists
        let commander = self.card_db.get_commander(cardgame::CardId(deck.commander));
        if commander.is_none() {
            errors.push(format!("Commander {} not found", deck.commander));
        }

        // Check deck size
        const MIN_DECK_SIZE: usize = 30;
        const MAX_DECK_SIZE: usize = 40;
        if deck.cards.len() < MIN_DECK_SIZE {
            errors.push(format!(
                "Deck has {} cards, minimum is {}",
                deck.cards.len(),
                MIN_DECK_SIZE
            ));
        }
        if deck.cards.len() > MAX_DECK_SIZE {
            errors.push(format!(
                "Deck has {} cards, maximum is {}",
                deck.cards.len(),
                MAX_DECK_SIZE
            ));
        }

        // Check card validity and faction
        let commander_faction_str = commander.map(|c| card_faction_to_string(c.faction));

        for &card_id in &deck.cards {
            match self.card_db.get(cardgame::CardId(card_id)) {
                Some(card) => {
                    // Check faction compatibility
                    let card_faction = faction_from_card_id(card.id);
                    if let Some(cmd_faction) = commander_faction_str {
                        if card_faction != "neutral" && card_faction != cmd_faction {
                            errors.push(format!(
                                "Card {} ({}) is from {} faction, but commander is {}",
                                card.name,
                                card_id,
                                card_faction,
                                cmd_faction
                            ));
                        }
                    }
                }
                None => {
                    errors.push(format!("Card {} not found", card_id));
                }
            }
        }

        // Check for duplicates (3 copies max in typical card games)
        let mut card_counts = std::collections::HashMap::new();
        for &card_id in &deck.cards {
            *card_counts.entry(card_id).or_insert(0) += 1;
        }
        for (card_id, count) in card_counts {
            if count > 3 {
                if let Some(card) = self.card_db.get(cardgame::CardId(card_id)) {
                    warnings.push(format!(
                        "Card {} ({}) appears {} times (max recommended: 3)",
                        card.name, card_id, count
                    ));
                }
            }
        }

        let validation = DeckValidation {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        };

        serde_json::to_string(&validation)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    /// Calculate the playstyle for a deck.
    ///
    /// Returns a JSON PlaystyleScore object.
    #[wasm_bindgen]
    pub fn calculate_deck_playstyle(&self, cards_json: &str, _commander_id: u16) -> Result<String, JsError> {
        let cards: Vec<u16> = serde_json::from_str(cards_json)
            .map_err(|e| JsError::new(&format!("Invalid cards: {}", e)))?;

        // Simple playstyle calculation based on card costs and types
        let mut aggro_score = 0.0f32;
        let mut control_score = 0.0f32;
        let mut tempo_score = 0.0f32;
        let total_cards = cards.len() as f32;

        if total_cards == 0.0 {
            let score = PlaystyleScore {
                playstyle: "Unknown".to_string(),
                aggro_score: 0.0,
                control_score: 0.0,
                tempo_score: 0.0,
                midrange_score: 0.0,
            };
            return serde_json::to_string(&score)
                .map_err(|e| JsError::new(&format!("Serialization error: {}", e)));
        }

        for &card_id in &cards {
            if let Some(card) = self.card_db.get(cardgame::CardId(card_id)) {
                // Low cost = aggro, high cost = control
                if card.cost <= 2 {
                    aggro_score += 1.0;
                } else if card.cost >= 5 {
                    control_score += 1.0;
                } else {
                    tempo_score += 1.0;
                }

                // Keywords affect playstyle
                let keywords = card.keywords();
                if keywords.has_rush() || keywords.has_quick() {
                    aggro_score += 0.5;
                }
                if keywords.has_guard() || keywords.has_shield() {
                    control_score += 0.5;
                }
                if keywords.has_lifesteal() {
                    control_score += 0.3;
                    tempo_score += 0.2;
                }
            }
        }

        // Normalize scores
        aggro_score /= total_cards;
        control_score /= total_cards;
        tempo_score /= total_cards;

        // Midrange is balanced between aggro and control
        let midrange_score = 1.0 - (aggro_score - control_score).abs();

        // Determine dominant playstyle
        let playstyle = if aggro_score > control_score && aggro_score > tempo_score {
            "Aggro"
        } else if control_score > aggro_score && control_score > tempo_score {
            "Control"
        } else if tempo_score > aggro_score && tempo_score > control_score {
            "Tempo"
        } else {
            "Midrange"
        };

        let score = PlaystyleScore {
            playstyle: playstyle.to_string(),
            aggro_score,
            control_score,
            tempo_score,
            midrange_score,
        };

        serde_json::to_string(&score)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }
}

