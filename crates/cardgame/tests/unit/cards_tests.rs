//! Unit tests for cards module.

use cardgame::cards::{
    CardDatabase, CardDefinition, CardLoadError, CardType, EffectDefinition, PassiveEffectDefinition,
    PassiveModifier, Faction,
};
use cardgame::effects::{CreatureFilter, TargetingRule};
use cardgame::types::{CardId, Rarity};

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
            keywords_bits: cardgame::keywords::Keywords::from_names(&["Rush", "Guard"]),
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
            effects: vec![EffectDefinition::Damage { amount: 4, filter: None }],
            conditional_effects: vec![],
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
    let cards = vec![create_test_creature(), create_test_spell()];

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
    let db =
        CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards from directory");

    // Verify we loaded all card sets:
    // - Core Set: 300 cards (Foundation Set complete!) + 8 activated ability creatures
    //   - Argentum: IDs 1000-1077 (77 cards) - includes 4 commanders + 2 activated ability creatures
    //   - Symbiote: IDs 2000-2076 (77 cards) - includes 4 commanders + 2 activated ability creatures
    //   - Obsidion: IDs 3000-3076 (77 cards) - includes 4 commanders + 2 activated ability creatures
    //   - Free-Walkers: IDs 4000-4076 (77 cards) - no commanders + 2 activated ability creatures
    assert_eq!(db.len(), 309);

    // Verify specific cards exist from each faction
    let brass_sentinel = db.get(CardId(1000)).expect("Card 1000 not found");
    assert_eq!(brass_sentinel.name, "Brass Sentinel");
    assert!(brass_sentinel.is_creature());

    let reinforce = db.get(CardId(1010)).expect("Card 1010 not found");
    assert_eq!(reinforce.name, "Reinforce");
    assert!(reinforce.is_spell());

    let assembly_line = db.get(CardId(1013)).expect("Card 1013 not found");
    assert_eq!(assembly_line.name, "Assembly Line");
    assert!(assembly_line.is_support());
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

#[test]
fn test_load_from_directory_nonexistent_path() {
    let result = CardDatabase::load_from_directory("nonexistent/path/to/cards");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, CardLoadError::Validation(_)));
    assert!(
        err.to_string().contains("does not exist"),
        "Error should mention path doesn't exist: {}",
        err
    );
}

#[test]
fn test_load_from_directory_empty_directory() {
    // Create a temp directory for this test
    let temp_dir = std::env::temp_dir().join("cardgame_test_empty_dir");
    let _ = std::fs::remove_dir_all(&temp_dir); // Clean up from previous runs
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    let result = CardDatabase::load_from_directory(&temp_dir);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, CardLoadError::Validation(_)));
    assert!(
        err.to_string().contains("No cards found"),
        "Error should mention no cards found: {}",
        err
    );

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_load_from_directory_file_not_directory() {
    // Use a known file that exists (now in subfolder)
    let file_path = cardgame::data_dir().join("cards/core_set/argentum/creatures.yaml");
    let result = CardDatabase::load_from_directory(&file_path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, CardLoadError::Validation(_)));
    assert!(
        err.to_string().contains("not a directory"),
        "Error should mention path is not a directory: {}",
        err
    );
}

// =============================================================================
// COMMANDER TESTS
// =============================================================================

#[test]
fn test_commander_yaml_parsing() {
    let yaml = r#"
commanders:
  - id: 5000
    name: "Test Commander"
    faction: Argentum
    rarity: Legendary
    passive_ability:
      description: "Your creatures have +1 Attack"
      effect:
        type: buff_stats
        attack: 1
        health: 0
    flavor: "Test flavor text"
"#;

    let commanders = CardDatabase::load_commanders_from_yaml(yaml)
        .expect("Failed to parse commander YAML");

    assert_eq!(commanders.len(), 1);
    let commander = &commanders[0];
    assert_eq!(commander.id, 5000);
    assert_eq!(commander.name, "Test Commander");
    assert_eq!(commander.faction, Faction::Argentum);
    assert!(commander.has_passive());
    assert!(!commander.has_triggered());
}

#[test]
fn test_commander_triggered_ability_yaml() {
    let yaml = r#"
commanders:
  - id: 5001
    name: "Token Summoner"
    faction: Symbiote
    rarity: Legendary
    triggered_ability:
      trigger: StartOfTurn
      description: "At the start of your turn, summon a 1/1 token"
      effects:
        - type: summon_token
          token:
            name: "Spawn"
            attack: 1
            health: 1
            keywords: []
"#;

    let commanders = CardDatabase::load_commanders_from_yaml(yaml)
        .expect("Failed to parse commander YAML");

    assert_eq!(commanders.len(), 1);
    let commander = &commanders[0];
    assert!(!commander.has_passive());
    assert!(commander.has_triggered());
}

#[test]
fn test_load_commanders_from_directory() {
    let commanders = CardDatabase::load_commanders_from_directory(
        cardgame::data_dir().join("commanders")
    ).expect("Failed to load commanders from directory");

    // We should have 12 commanders (4 per faction)
    assert_eq!(commanders.len(), 12, "Expected 12 commanders, got {}", commanders.len());

    // Verify commanders by faction
    let argentum_count = commanders.iter().filter(|c| c.faction == Faction::Argentum).count();
    let symbiote_count = commanders.iter().filter(|c| c.faction == Faction::Symbiote).count();
    let obsidion_count = commanders.iter().filter(|c| c.faction == Faction::Obsidion).count();

    assert_eq!(argentum_count, 4, "Expected 4 Argentum commanders");
    assert_eq!(symbiote_count, 4, "Expected 4 Symbiote commanders");
    assert_eq!(obsidion_count, 4, "Expected 4 Obsidion commanders");
}

#[test]
fn test_card_database_with_commanders() {
    // Load cards and commanders
    let card_db = CardDatabase::load_from_directory(
        cardgame::data_dir().join("cards/core_set")
    ).expect("Failed to load cards");

    let commanders = CardDatabase::load_commanders_from_directory(
        cardgame::data_dir().join("commanders")
    ).expect("Failed to load commanders");

    let full_db = card_db.with_commanders(commanders);

    // Verify cards (309 after activated ability creatures added)
    assert_eq!(full_db.len(), 309);

    // Verify commanders
    assert_eq!(full_db.commander_count(), 12);

    // Verify commander lookup (using new ID range 5000+)
    let high_artificer = full_db.get_commander(CardId(5000));
    assert!(high_artificer.is_some(), "High Artificer (5000) not found");
    assert_eq!(high_artificer.unwrap().name, "The High Artificer");

    // Verify card ID and commander ID don't conflict
    // Card 1000 is Brass Sentinel
    let card_1000 = full_db.get(CardId(1000));
    assert!(card_1000.is_some());
    assert_eq!(card_1000.unwrap().name, "Brass Sentinel");

    // Commander 5000 is The High Artificer
    let commander_5000 = full_db.get_commander(CardId(5000));
    assert!(commander_5000.is_some());
    assert_eq!(commander_5000.unwrap().name, "The High Artificer");
}

#[test]
fn test_load_with_commanders_convenience() {
    let full_db = CardDatabase::load_with_commanders(
        cardgame::data_dir().join("cards/core_set"),
        cardgame::data_dir().join("commanders"),
    ).expect("Failed to load cards and commanders");

    assert_eq!(full_db.len(), 309);
    assert_eq!(full_db.commander_count(), 12);
}

#[test]
fn test_commander_iterator() {
    let commanders = CardDatabase::load_commanders_from_directory(
        cardgame::data_dir().join("commanders")
    ).expect("Failed to load commanders");

    let db = CardDatabase::empty().with_commanders(commanders);

    let commander_ids: Vec<u16> = db.commander_ids().map(|id| id.0).collect();
    assert_eq!(commander_ids.len(), 12);

    // All IDs should be in the 5000 range
    for id in commander_ids {
        assert!(id >= 5000 && id <= 5011, "Commander ID {} out of expected range 5000-5011", id);
    }
}

#[test]
fn test_commander_ability_helpers() {
    let yaml = r#"
commanders:
  - id: 5100
    name: "Passive Commander"
    faction: Argentum
    rarity: Legendary
    passive_ability:
      description: "Your creatures have Guard"
      effect:
        type: grant_keyword
        keyword: Guard
  - id: 5101
    name: "Triggered Commander"
    faction: Obsidion
    rarity: Legendary
    triggered_ability:
      trigger: OnEnemyDeath
      description: "Draw a card when an enemy dies"
      effects:
        - type: draw
          count: 1
"#;

    let commanders = CardDatabase::load_commanders_from_yaml(yaml)
        .expect("Failed to parse commander YAML");

    // Test passive commander
    let passive_cmd = &commanders[0];
    assert!(passive_cmd.has_passive());
    assert!(!passive_cmd.has_triggered());
    assert!(passive_cmd.passive_ability().is_some());
    assert!(passive_cmd.triggered_ability().is_none());

    // Test triggered commander
    let triggered_cmd = &commanders[1];
    assert!(!triggered_cmd.has_passive());
    assert!(triggered_cmd.has_triggered());
    assert!(triggered_cmd.passive_ability().is_none());
    assert!(triggered_cmd.triggered_ability().is_some());
}

#[test]
fn test_duplicate_commander_id_detection() {
    let yaml = r#"
commanders:
  - id: 5000
    name: "Commander One"
    faction: Argentum
    rarity: Legendary
    passive_ability:
      description: "Ability one"
      effect:
        type: grant_keyword
        keyword: Guard
  - id: 5000
    name: "Commander Duplicate"
    faction: Argentum
    rarity: Legendary
    passive_ability:
      description: "Ability two"
      effect:
        type: grant_keyword
        keyword: Shield
"#;

    let result = CardDatabase::load_commanders_from_yaml(yaml);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, CardLoadError::Validation(_)));
}
