//! Card definitions and the card database.
//!
//! Cards are defined in YAML files and loaded at runtime into a CardDatabase.
//! The database is immutable and shared across all game instances via Arc.

use std::sync::Arc;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::core::types::*;
use crate::core::keywords::Keywords;
use crate::core::effects::{Condition, Trigger, TargetingRule, CreatureFilter, TokenDefinition};

/// Definition of a triggered ability on a creature
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AbilityDefinition {
    pub trigger: Trigger,
    #[serde(default)]
    pub targeting: TargetingRule,
    pub effects: Vec<EffectDefinition>,
    /// Effects that trigger conditionally based on the result of the primary effects
    #[serde(default)]
    pub conditional_effects: Vec<ConditionalEffectGroup>,
}

/// A group of effects that trigger if a condition is met
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConditionalEffectGroup {
    /// The condition that must be met for these effects to trigger
    pub condition: Condition,
    /// The effects to apply if the condition is met
    pub effects: Vec<EffectDefinition>,
}

/// Token definition for YAML (uses string keywords instead of bitmask)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TokenDef {
    /// Display name for the token
    pub name: String,
    /// Attack value
    pub attack: u8,
    /// Health value
    pub health: u8,
    /// Keywords as string list (converted to bitmask at runtime)
    #[serde(default)]
    pub keywords: Vec<String>,
}

impl TokenDef {
    /// Convert to TokenDefinition with keywords as bitmask
    pub fn to_token_definition(&self) -> TokenDefinition {
        let keyword_refs: Vec<&str> = self.keywords.iter().map(|s| s.as_str()).collect();
        let keywords = Keywords::from_names(&keyword_refs);
        TokenDefinition {
            name: self.name.clone(),
            attack: self.attack,
            health: self.health,
            keywords: keywords.0,
        }
    }
}

/// Definition of an effect (serializable from YAML)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EffectDefinition {
    Damage {
        amount: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<CreatureFilter>,
    },
    Heal {
        amount: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<CreatureFilter>,
    },
    Draw { count: u8 },
    BuffStats {
        attack: i8,
        health: i8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<CreatureFilter>,
    },
    Destroy {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<CreatureFilter>,
    },
    GrantKeyword {
        keyword: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<CreatureFilter>,
    },
    RemoveKeyword {
        keyword: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<CreatureFilter>,
    },
    Silence {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<CreatureFilter>,
    },
    GainEssence { amount: u8 },
    RefreshCreature,
    Bounce {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<CreatureFilter>,
    },
    /// Summon a token creature
    SummonToken {
        token: TokenDef,
    },
    /// Transform target creature into a token
    Transform {
        into: TokenDef,
    },
    /// Create a copy of target creature
    Copy,
}

/// Definition of a passive effect (for supports)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PassiveEffectDefinition {
    pub modifier: PassiveModifier,
}

/// Types of passive modifiers for supports
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PassiveModifier {
    AttackBonus(i8),
    HealthBonus(i8),
    GrantKeyword(String),
}

/// Card type with type-specific data
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "card_type", rename_all = "snake_case")]
pub enum CardType {
    Creature {
        attack: u8,
        health: u8,
        #[serde(default)]
        keywords: Vec<String>,
        #[serde(default)]
        abilities: Vec<AbilityDefinition>,
    },
    Spell {
        #[serde(default)]
        targeting: TargetingRule,
        effects: Vec<EffectDefinition>,
        /// Effects that trigger conditionally based on the result of the primary effects
        #[serde(default)]
        conditional_effects: Vec<ConditionalEffectGroup>,
    },
    Support {
        durability: u8,
        #[serde(default)]
        passive_effects: Vec<PassiveEffectDefinition>,
        #[serde(default)]
        triggered_effects: Vec<AbilityDefinition>,
    },
}

/// Complete definition of a card
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CardDefinition {
    pub id: u16,
    pub name: String,
    pub cost: u8,
    #[serde(flatten)]
    pub card_type: CardType,
    #[serde(default)]
    pub rarity: Rarity,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl CardDefinition {
    /// Get keywords for a creature card (parsed from string list)
    pub fn keywords(&self) -> Keywords {
        match &self.card_type {
            CardType::Creature { keywords, .. } => {
                let refs: Vec<&str> = keywords.iter().map(|s| s.as_str()).collect();
                Keywords::from_names(&refs)
            }
            _ => Keywords::none(),
        }
    }

    /// Get attack for a creature card
    pub fn attack(&self) -> Option<u8> {
        match &self.card_type {
            CardType::Creature { attack, .. } => Some(*attack),
            _ => None,
        }
    }

    /// Get health for a creature card
    pub fn health(&self) -> Option<u8> {
        match &self.card_type {
            CardType::Creature { health, .. } => Some(*health),
            _ => None,
        }
    }

    /// Get durability for a support card
    pub fn durability(&self) -> Option<u8> {
        match &self.card_type {
            CardType::Support { durability, .. } => Some(*durability),
            _ => None,
        }
    }

    /// Check if this is a creature card
    pub fn is_creature(&self) -> bool {
        matches!(self.card_type, CardType::Creature { .. })
    }

    /// Check if this is a spell card
    pub fn is_spell(&self) -> bool {
        matches!(self.card_type, CardType::Spell { .. })
    }

    /// Check if this is a support card
    pub fn is_support(&self) -> bool {
        matches!(self.card_type, CardType::Support { .. })
    }

    /// Get targeting rule for a spell
    pub fn spell_targeting(&self) -> Option<&TargetingRule> {
        match &self.card_type {
            CardType::Spell { targeting, .. } => Some(targeting),
            _ => None,
        }
    }

    /// Get effects for a spell
    pub fn spell_effects(&self) -> Option<&[EffectDefinition]> {
        match &self.card_type {
            CardType::Spell { effects, .. } => Some(effects),
            _ => None,
        }
    }

    /// Get abilities for a creature
    pub fn creature_abilities(&self) -> Option<&[AbilityDefinition]> {
        match &self.card_type {
            CardType::Creature { abilities, .. } => Some(abilities),
            _ => None,
        }
    }

    /// Get passive effects for a support
    pub fn support_passives(&self) -> Option<&[PassiveEffectDefinition]> {
        match &self.card_type {
            CardType::Support { passive_effects, .. } => Some(passive_effects),
            _ => None,
        }
    }
}

/// Card set loaded from YAML (container for multiple cards)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CardSet {
    pub name: String,
    pub cards: Vec<CardDefinition>,
}

/// The complete card database - immutable, shared across game instances
#[derive(Clone)]
pub struct CardDatabase {
    cards: Arc<Vec<CardDefinition>>,
    /// Lookup table: card ID -> index in cards vec
    id_to_index: Arc<Vec<Option<usize>>>,
}

impl CardDatabase {
    /// Create a new database from a list of cards
    pub fn new(cards: Vec<CardDefinition>) -> Self {
        // Find max ID to size the lookup table
        let max_id = cards.iter().map(|c| c.id).max().unwrap_or(0) as usize;

        // Build lookup table
        let mut id_to_index = vec![None; max_id + 1];
        for (index, card) in cards.iter().enumerate() {
            id_to_index[card.id as usize] = Some(index);
        }

        Self {
            cards: Arc::new(cards),
            id_to_index: Arc::new(id_to_index),
        }
    }

    /// Create an empty database (for testing)
    pub fn empty() -> Self {
        Self {
            cards: Arc::new(Vec::new()),
            id_to_index: Arc::new(Vec::new()),
        }
    }

    /// Get a card by ID (O(1) lookup)
    pub fn get(&self, id: CardId) -> Option<&CardDefinition> {
        let idx = id.0 as usize;
        if idx < self.id_to_index.len() {
            self.id_to_index[idx].map(|i| &self.cards[i])
        } else {
            None
        }
    }

    /// Get total number of cards
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Check if database is empty
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Iterate over all cards
    pub fn iter(&self) -> impl Iterator<Item = &CardDefinition> {
        self.cards.iter()
    }

    /// Get all card IDs
    pub fn card_ids(&self) -> impl Iterator<Item = CardId> + '_ {
        self.cards.iter().map(|c| CardId(c.id))
    }
}

impl std::fmt::Debug for CardDatabase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CardDatabase")
            .field("num_cards", &self.cards.len())
            .finish()
    }
}

/// Errors that can occur when loading cards
#[derive(Error, Debug)]
pub enum CardLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Card validation error: {0}")]
    Validation(String),
}

impl CardDatabase {
    /// Validate an effect definition for problematic values
    fn validate_effect_definition(card: &CardDefinition, effect: &EffectDefinition) -> Result<(), CardLoadError> {
        match effect {
            EffectDefinition::BuffStats { attack, health, .. } => {
                // Check for extreme buff values (likely data entry errors)
                if attack.abs() > 20 {
                    return Err(CardLoadError::Validation(format!(
                        "Card {} '{}' has extreme attack buff: {} (abs max 20)",
                        card.id, card.name, attack
                    )));
                }
                if health.abs() > 20 {
                    return Err(CardLoadError::Validation(format!(
                        "Card {} '{}' has extreme health buff: {} (abs max 20)",
                        card.id, card.name, health
                    )));
                }
            },
            EffectDefinition::Transform { into, .. } => {
                if into.health == 0 {
                    return Err(CardLoadError::Validation(format!(
                        "Card {} '{}' has Transform effect with invalid health: 0 (must be > 0)",
                        card.id, card.name
                    )));
                }
                if into.health > 50 {
                    return Err(CardLoadError::Validation(format!(
                        "Card {} '{}' has Transform effect with extreme health: {} (max 50)",
                        card.id, card.name, into.health
                    )));
                }
                if into.attack > 50 {
                    return Err(CardLoadError::Validation(format!(
                        "Card {} '{}' has Transform effect with extreme attack: {} (max 50)",
                        card.id, card.name, into.attack
                    )));
                }
            },
            EffectDefinition::SummonToken { token } => {
                if token.health == 0 {
                    return Err(CardLoadError::Validation(format!(
                        "Card {} '{}' has SummonToken effect with invalid health: 0 (must be > 0)",
                        card.id, card.name
                    )));
                }
                if token.health > 50 {
                    return Err(CardLoadError::Validation(format!(
                        "Card {} '{}' has SummonToken effect with extreme health: {} (max 50)",
                        card.id, card.name, token.health
                    )));
                }
                if token.attack > 50 {
                    return Err(CardLoadError::Validation(format!(
                        "Card {} '{}' has SummonToken effect with extreme attack: {} (max 50)",
                        card.id, card.name, token.attack
                    )));
                }
            },
            EffectDefinition::Copy => {
                // Copy creates a clone, so validation is deferred to the source creature
            },
            _ => {
                // Other effect types don't create creatures or modify stats in dangerous ways
            }
        }
        Ok(())
    }

    /// Load cards from a directory containing YAML files.
    ///
    /// The directory should contain `.yaml` or `.yml` files with card definitions.
    /// For example, if your cards are in `data/cards/core_set/`, pass that full path.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The directory doesn't exist
    /// - No cards are found (empty directory or no valid YAML files)
    /// - Duplicate card IDs are found
    /// - YAML parsing fails
    pub fn load_from_directory<P: AsRef<Path>>(path: P) -> Result<Self, CardLoadError> {
        let dir_path = path.as_ref();

        if !dir_path.exists() {
            return Err(CardLoadError::Validation(format!(
                "Card directory does not exist: {}",
                dir_path.display()
            )));
        }

        if !dir_path.is_dir() {
            return Err(CardLoadError::Validation(format!(
                "Path is not a directory: {}",
                dir_path.display()
            )));
        }

        let mut all_cards = Vec::new();

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().is_some_and(|ext| ext == "yaml" || ext == "yml") {
                let yaml_content = fs::read_to_string(&file_path)?;
                let card_set: CardSet = serde_yaml::from_str(&yaml_content)?;
                all_cards.extend(card_set.cards);
            }
        }

        // Validate that we loaded at least one card
        if all_cards.is_empty() {
            return Err(CardLoadError::Validation(format!(
                "No cards found in directory: {}. Expected .yaml or .yml files with card definitions.",
                dir_path.display()
            )));
        }

        // Validate no duplicate IDs
        let mut seen_ids = std::collections::HashSet::new();
        for card in &all_cards {
            if !seen_ids.insert(card.id) {
                return Err(CardLoadError::Validation(format!(
                    "Duplicate card ID: {}",
                    card.id
                )));
            }
        }

        // Validate card definitions for problematic values
        for card in &all_cards {
            match &card.card_type {
                CardType::Creature { attack, health, abilities, .. } => {
                    // Check creature base stats
                    if *health == 0 {
                        return Err(CardLoadError::Validation(format!(
                            "Card {} '{}' has invalid base health: 0 (must be > 0)",
                            card.id, card.name
                        )));
                    }
                    // Sanity check for extreme values (likely data entry errors)
                    if *health > 50 {
                        return Err(CardLoadError::Validation(format!(
                            "Card {} '{}' has suspiciously high health: {} (max 50)",
                            card.id, card.name, health
                        )));
                    }
                    if *attack > 50 {
                        return Err(CardLoadError::Validation(format!(
                            "Card {} '{}' has suspiciously high attack: {} (max 50)",
                            card.id, card.name, attack
                        )));
                    }

                    // Validate abilities
                    for ability in abilities {
                        for effect in &ability.effects {
                            Self::validate_effect_definition(card, effect)?;
                        }
                        for conditional_group in &ability.conditional_effects {
                            for effect in &conditional_group.effects {
                                Self::validate_effect_definition(card, effect)?;
                            }
                        }
                    }
                },
                CardType::Spell { effects, conditional_effects, .. } => {
                    // Validate spell effects
                    for effect in effects {
                        Self::validate_effect_definition(card, effect)?;
                    }
                    for conditional_group in conditional_effects {
                        for effect in &conditional_group.effects {
                            Self::validate_effect_definition(card, effect)?;
                        }
                    }
                },
                CardType::Support { triggered_effects, .. } => {
                    // Validate support triggered effects
                    for ability in triggered_effects {
                        for effect in &ability.effects {
                            Self::validate_effect_definition(card, effect)?;
                        }
                        for conditional_group in &ability.conditional_effects {
                            for effect in &conditional_group.effects {
                                Self::validate_effect_definition(card, effect)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(Self::new(all_cards))
    }

    /// Load cards from a single YAML string (useful for testing)
    pub fn load_from_yaml(yaml: &str) -> Result<Self, CardLoadError> {
        let card_set: CardSet = serde_yaml::from_str(yaml)?;

        // Validate no duplicate IDs
        let mut seen_ids = std::collections::HashSet::new();
        for card in &card_set.cards {
            if !seen_ids.insert(card.id) {
                return Err(CardLoadError::Validation(
                    format!("Duplicate card ID: {}", card.id)
                ));
            }
        }

        Ok(Self::new(card_set.cards))
    }
}
