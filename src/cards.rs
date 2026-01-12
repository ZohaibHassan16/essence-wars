//! Card definitions and the card database.
//!
//! Cards are defined in YAML files and loaded at runtime into a CardDatabase.
//! The database is immutable and shared across all game instances via Arc.

use std::sync::Arc;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::types::*;
use crate::keywords::Keywords;
use crate::effects::{Trigger, TargetingRule, CreatureFilter};

/// Definition of a triggered ability on a creature
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AbilityDefinition {
    pub trigger: Trigger,
    #[serde(default)]
    pub targeting: TargetingRule,
    pub effects: Vec<EffectDefinition>,
}

/// Definition of an effect (serializable from YAML)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EffectDefinition {
    Damage { amount: u8 },
    Heal { amount: u8 },
    Draw { count: u8 },
    BuffStats { attack: i8, health: i8 },
    Destroy,
    GrantKeyword { keyword: String },
    RemoveKeyword { keyword: String },
    Silence,
    GainEssence { amount: u8 },
    RefreshCreature,
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
    /// Load cards from a directory containing YAML files
    pub fn load_from_directory<P: AsRef<Path>>(path: P) -> Result<Self, CardLoadError> {
        let mut all_cards = Vec::new();
        let sets_path = path.as_ref().join("sets");

        if sets_path.exists() {
            for entry in fs::read_dir(&sets_path)? {
                let entry = entry?;
                let file_path = entry.path();

                if file_path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                    let yaml_content = fs::read_to_string(&file_path)?;
                    let card_set: CardSet = serde_yaml::from_str(&yaml_content)?;
                    all_cards.extend(card_set.cards);
                }
            }
        }

        // Validate no duplicate IDs
        let mut seen_ids = std::collections::HashSet::new();
        for card in &all_cards {
            if !seen_ids.insert(card.id) {
                return Err(CardLoadError::Validation(
                    format!("Duplicate card ID: {}", card.id)
                ));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_creature() -> CardDefinition {
        CardDefinition {
            id: 1,
            name: "Test Creature".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 2,
                health: 3,
                keywords: vec!["Rush".to_string(), "Guard".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec!["Soldier".to_string()],
        }
    }

    fn create_test_spell() -> CardDefinition {
        CardDefinition {
            id: 2,
            name: "Test Spell".to_string(),
            cost: 3,
            card_type: CardType::Spell {
                targeting: TargetingRule::TargetCreature(CreatureFilter::any()),
                effects: vec![EffectDefinition::Damage { amount: 4 }],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        }
    }

    fn create_test_support() -> CardDefinition {
        CardDefinition {
            id: 3,
            name: "Test Support".to_string(),
            cost: 4,
            card_type: CardType::Support {
                durability: 3,
                passive_effects: vec![PassiveEffectDefinition {
                    modifier: PassiveModifier::AttackBonus(1),
                }],
                triggered_effects: vec![],
            },
            rarity: Rarity::Rare,
            tags: vec![],
        }
    }

    #[test]
    fn test_card_type_checks() {
        let creature = create_test_creature();
        let spell = create_test_spell();
        let support = create_test_support();

        assert!(creature.is_creature());
        assert!(!creature.is_spell());
        assert!(!creature.is_support());

        assert!(!spell.is_creature());
        assert!(spell.is_spell());
        assert!(!spell.is_support());

        assert!(!support.is_creature());
        assert!(!support.is_spell());
        assert!(support.is_support());
    }

    #[test]
    fn test_creature_helpers() {
        let creature = create_test_creature();

        assert_eq!(creature.attack(), Some(2));
        assert_eq!(creature.health(), Some(3));

        let keywords = creature.keywords();
        assert!(keywords.has_rush());
        assert!(keywords.has_guard());
        assert!(!keywords.has_lethal());
    }

    #[test]
    fn test_spell_helpers() {
        let spell = create_test_spell();

        assert!(spell.spell_targeting().is_some());
        assert!(spell.spell_effects().is_some());
        assert_eq!(spell.spell_effects().unwrap().len(), 1);
    }

    #[test]
    fn test_support_helpers() {
        let support = create_test_support();

        assert_eq!(support.durability(), Some(3));
        assert!(support.support_passives().is_some());
        assert_eq!(support.support_passives().unwrap().len(), 1);
    }

    #[test]
    fn test_card_database() {
        let cards = vec![
            create_test_creature(),
            create_test_spell(),
            create_test_support(),
        ];

        let db = CardDatabase::new(cards);

        assert_eq!(db.len(), 3);
        assert!(!db.is_empty());

        // Test lookup by ID
        assert!(db.get(CardId(1)).is_some());
        assert!(db.get(CardId(2)).is_some());
        assert!(db.get(CardId(3)).is_some());
        assert!(db.get(CardId(99)).is_none());

        // Verify contents
        assert_eq!(db.get(CardId(1)).unwrap().name, "Test Creature");
        assert_eq!(db.get(CardId(2)).unwrap().name, "Test Spell");
    }

    #[test]
    fn test_card_database_iteration() {
        let cards = vec![
            create_test_creature(),
            create_test_spell(),
        ];

        let db = CardDatabase::new(cards);

        let names: Vec<_> = db.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"Test Creature"));
        assert!(names.contains(&"Test Spell"));
    }

    #[test]
    fn test_empty_database() {
        let db = CardDatabase::empty();
        assert!(db.is_empty());
        assert_eq!(db.len(), 0);
        assert!(db.get(CardId(1)).is_none());
    }

    #[test]
    fn test_yaml_loading() {
        let yaml = r#"
name: "Test Set"
cards:
  - id: 10
    name: "YAML Creature"
    cost: 2
    card_type: creature
    attack: 3
    health: 2
    keywords:
      - Rush
    rarity: Common
    tags:
      - Soldier
  - id: 11
    name: "YAML Spell"
    cost: 1
    card_type: spell
    targeting: NoTarget
    effects:
      - type: draw
        count: 2
"#;

        let db = CardDatabase::load_from_yaml(yaml).expect("Failed to parse YAML");
        assert_eq!(db.len(), 2);

        let creature = db.get(CardId(10)).expect("Card 10 not found");
        assert_eq!(creature.name, "YAML Creature");
        assert!(creature.is_creature());
        assert!(creature.keywords().has_rush());

        let spell = db.get(CardId(11)).expect("Card 11 not found");
        assert_eq!(spell.name, "YAML Spell");
        assert!(spell.is_spell());
    }

    #[test]
    fn test_load_from_directory() {
        let db = CardDatabase::load_from_directory("data/cards")
            .expect("Failed to load cards from directory");

        // Verify we loaded the starter set (15 cards)
        assert_eq!(db.len(), 15);

        // Verify specific cards exist
        let eager_recruit = db.get(CardId(1)).expect("Card 1 not found");
        assert_eq!(eager_recruit.name, "Eager Recruit");
        assert!(eager_recruit.is_creature());

        let quick_strike = db.get(CardId(32)).expect("Card 32 not found");
        assert_eq!(quick_strike.name, "Quick Strike");
        assert!(quick_strike.is_spell());

        let war_drums = db.get(CardId(40)).expect("Card 40 not found");
        assert_eq!(war_drums.name, "War Drums");
        assert!(war_drums.is_support());
    }

    #[test]
    fn test_duplicate_id_detection() {
        let yaml = r#"
name: "Test Set"
cards:
  - id: 1
    name: "Card One"
    cost: 1
    card_type: creature
    attack: 1
    health: 1
  - id: 1
    name: "Duplicate Card"
    cost: 2
    card_type: creature
    attack: 2
    health: 2
"#;

        let result = CardDatabase::load_from_yaml(yaml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CardLoadError::Validation(_)));
    }

    #[test]
    fn test_passive_modifier_yaml_format() {
        // Test what YAML format the PassiveModifier enum expects
        let yaml = r#"
name: "Test Set"
cards:
  - id: 100
    name: "Test Support"
    cost: 3
    card_type: support
    durability: 3
    passive_effects:
      - modifier:
          attack_bonus: 1
    rarity: Common
"#;
        let db = CardDatabase::load_from_yaml(yaml).expect("Failed to parse YAML");
        let card = db.get(CardId(100)).expect("Card not found");
        assert!(card.is_support());
    }
}
