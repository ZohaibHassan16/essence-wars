//! Common test utilities and helpers shared across all test files.

// Allow dead_code because each test binary is compiled independently,
// so functions used by other test files appear "unused" to each binary.
#![allow(dead_code)]

use cardgame::cards::{
    AbilityDefinition, CardDatabase, CardDefinition, CardType, EffectDefinition,
    PassiveEffectDefinition, PassiveModifier,
};
use cardgame::decks::DeckRegistry;
use cardgame::effects::{TargetingRule, Trigger};
use cardgame::keywords::Keywords;
use cardgame::state::{Creature, CreatureStatus, GameState};
use cardgame::types::{CardId, PlayerId, Rarity, Slot};

/// Load the real card database from the data directory.
/// Used by integration tests that need actual game cards.
pub fn load_real_card_db() -> CardDatabase {
    let cards_path = cardgame::data_dir().join("cards/core_set");
    CardDatabase::load_from_directory(cards_path)
        .expect("Failed to load card database")
}

/// Load the real deck registry from the data directory.
/// Used by integration tests that need actual deck definitions.
pub fn load_real_deck_registry() -> DeckRegistry {
    let decks_path = cardgame::data_dir().join("decks");
    DeckRegistry::load_from_directory(decks_path)
        .expect("Failed to load deck registry")
}

/// Create a test card database with basic cards
pub fn test_card_db() -> CardDatabase {
    let cards = vec![
        CardDefinition {
            id: 1,
            name: "Test Creature".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 2,
                health: 3,
                keywords: vec![],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        CardDefinition {
            id: 2,
            name: "Rush Creature".to_string(),
            cost: 1,
            card_type: CardType::Creature {
                attack: 1,
                health: 1,
                keywords: vec!["Rush".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        CardDefinition {
            id: 3,
            name: "Big Creature".to_string(),
            cost: 3,
            card_type: CardType::Creature {
                attack: 5,
                health: 5,
                keywords: vec![],
                abilities: vec![],
            },
            rarity: Rarity::Rare,
            tags: vec![],
        },
    ];
    CardDatabase::new(cards)
}

/// Create a simple deck of card IDs (30 cards: 10 each of IDs 1, 2, 3)
pub fn simple_deck() -> Vec<CardId> {
    let mut deck = Vec::new();
    for _ in 0..10 {
        deck.push(CardId(1));
        deck.push(CardId(2));
        deck.push(CardId(3));
    }
    deck
}

/// Helper to create a test game state for effect queue tests
pub fn create_effect_test_state() -> GameState {
    let mut state = GameState::new();
    state.players[0].life = 30;
    state.players[1].life = 30;
    state.current_turn = 1;
    state
}

/// Helper to create a simple test creature
pub fn create_test_creature(
    state: &mut GameState,
    owner: PlayerId,
    slot: Slot,
    attack: i8,
    health: i8,
    keywords: Keywords,
) {
    let instance_id = state.next_creature_instance_id();
    let creature = Creature {
        instance_id,
        card_id: CardId(1),
        owner,
        slot,
        attack,
        current_health: health,
        max_health: health,
        base_attack: attack as u8,
        base_health: health as u8,
        keywords,
        status: CreatureStatus::default(),
        turn_played: 0,
        frenzy_stacks: 0,
    };
    state.players[owner.index()].creatures.push(creature);
}

/// Create a card database with cards for testing card playing logic
pub fn card_playing_test_db() -> CardDatabase {
    // Uses AbilityDefinition, EffectDefinition, TargetingRule, Trigger from module-level imports
    let cards = vec![
        // Basic creature: cost 2, 2/3
        CardDefinition {
            id: 1,
            name: "Test Creature".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 2,
                health: 3,
                keywords: vec![],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Rush creature: cost 1, 1/1, Rush
        CardDefinition {
            id: 2,
            name: "Rush Creature".to_string(),
            cost: 1,
            card_type: CardType::Creature {
                attack: 1,
                health: 1,
                keywords: vec!["Rush".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Creature with OnPlay draw effect
        CardDefinition {
            id: 3,
            name: "Draw Creature".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 1,
                health: 1,
                keywords: vec![],
                abilities: vec![AbilityDefinition {
                    trigger: Trigger::OnPlay,
                    targeting: TargetingRule::NoTarget,
                    effects: vec![EffectDefinition::Draw { count: 1 }],
                    conditional_effects: vec![],
                }],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
        // Damage spell: cost 1, deal 3 damage to target creature
        CardDefinition {
            id: 4,
            name: "Damage Spell".to_string(),
            cost: 1,
            card_type: CardType::Spell {
                targeting: TargetingRule::TargetEnemyCreature,
                effects: vec![EffectDefinition::Damage { amount: 3, filter: None }],
                conditional_effects: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // NoTarget spell: cost 1, draw 2 cards
        CardDefinition {
            id: 5,
            name: "Draw Spell".to_string(),
            cost: 1,
            card_type: CardType::Spell {
                targeting: TargetingRule::NoTarget,
                effects: vec![EffectDefinition::Draw { count: 2 }],
                conditional_effects: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Support card: cost 3, durability 2
        CardDefinition {
            id: 6,
            name: "Test Support".to_string(),
            cost: 3,
            card_type: CardType::Support {
                durability: 2,
                passive_effects: vec![],
                triggered_effects: vec![],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
        // Support with OnPlay effect
        CardDefinition {
            id: 7,
            name: "Draw Support".to_string(),
            cost: 2,
            card_type: CardType::Support {
                durability: 3,
                passive_effects: vec![],
                triggered_effects: vec![AbilityDefinition {
                    trigger: Trigger::OnPlay,
                    targeting: TargetingRule::NoTarget,
                    effects: vec![EffectDefinition::Draw { count: 1 }],
                    conditional_effects: vec![],
                }],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
        // Buff spell: cost 2, +2/+2 to target creature
        CardDefinition {
            id: 8,
            name: "Buff Spell".to_string(),
            cost: 2,
            card_type: CardType::Spell {
                targeting: TargetingRule::TargetAllyCreature,
                effects: vec![EffectDefinition::BuffStats { attack: 2, health: 2, filter: None }],
                conditional_effects: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Expensive creature: cost 5
        CardDefinition {
            id: 9,
            name: "Expensive Creature".to_string(),
            cost: 5,
            card_type: CardType::Creature {
                attack: 5,
                health: 5,
                keywords: vec![],
                abilities: vec![],
            },
            rarity: Rarity::Rare,
            tags: vec![],
        },
        // Support with attack bonus passive effect
        CardDefinition {
            id: 10,
            name: "War Banner".to_string(),
            cost: 2,
            card_type: CardType::Support {
                durability: 3,
                passive_effects: vec![PassiveEffectDefinition {
                    modifier: PassiveModifier::AttackBonus(1),
                }],
                triggered_effects: vec![],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
        // Support with health bonus passive effect
        CardDefinition {
            id: 11,
            name: "Barrier Shield".to_string(),
            cost: 2,
            card_type: CardType::Support {
                durability: 3,
                passive_effects: vec![PassiveEffectDefinition {
                    modifier: PassiveModifier::HealthBonus(2),
                }],
                triggered_effects: vec![],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
        // Support that grants Rush keyword
        CardDefinition {
            id: 12,
            name: "Haste Totem".to_string(),
            cost: 3,
            card_type: CardType::Support {
                durability: 2,
                passive_effects: vec![PassiveEffectDefinition {
                    modifier: PassiveModifier::GrantKeyword("Rush".to_string()),
                }],
                triggered_effects: vec![],
            },
            rarity: Rarity::Rare,
            tags: vec![],
        },
        // Support with StartOfTurn heal effect
        CardDefinition {
            id: 13,
            name: "Healing Shrine".to_string(),
            cost: 3,
            card_type: CardType::Support {
                durability: 4,
                passive_effects: vec![],
                triggered_effects: vec![AbilityDefinition {
                    trigger: Trigger::StartOfTurn,
                    targeting: TargetingRule::NoTarget,
                    effects: vec![EffectDefinition::Heal { amount: 2, filter: None }],
                    conditional_effects: vec![],
                }],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
    ];
    CardDatabase::new(cards)
}

/// Create a valid deck for integration tests using Core Set card IDs.
/// Uses a mix of creatures from all factions for testing.
/// Returns a proper 30-card deck for realistic game play.
pub fn valid_yaml_deck() -> Vec<CardId> {
    // Balanced mix from all factions with variety of costs
    // This ensures games can progress naturally to completion
    let card_ids: [u16; 30] = [
        // Argentum (10 cards)
        1000, 1000, // Brass Sentinel (2/4 Guard)
        1001, 1001, // Steam Knight (3/3 Piercing)
        1002, 1002, // Iron Golem (4/6 Guard)
        1003, 1003, // Clockwork Archer (2/2 Ranged)
        1004, 1004, // Steel Vanguard (3/5 Guard)
        // Symbiote (10 cards)
        2000, 2000, // Spore Crawler (1/2)
        2001, 2001, // Venom Fang (2/3 Lethal)
        2003, 2003, // Broodling (1/1 Rush)
        2005, 2005, // Pack Hunter (2/2 Rush)
        2002, 2002, // Regenerating Ooze (2/5 Regenerate)
        // Obsidion (10 cards)
        3000, 3000, // Shadow Initiate (2/2 Lifesteal)
        3001, 3001, // Void Stalker (3/2 Stealth)
        3002, 3002, // Twilight Reaper (4/3 Lifesteal)
        3003, 3003, // Nightblade (3/2 Quick)
        3004, 3004, // Phantom Assassin (2/1 Stealth+Lethal)
    ];
    card_ids.iter().map(|&id| CardId(id)).collect()
}

/// Create the standard arena deck for bot testing.
/// Uses Symbiote Aggro style deck (same as data/decks/symbiote/aggro.toml).
pub fn arena_test_deck() -> Vec<CardId> {
    let card_ids: [u16; 30] = [
        // Symbiote Core (21 cards)
        2003, 2003, // Broodling (1/1 Rush)
        2006, 2006, // Parasitic Larva (1/2 Lethal)
        2000, 2000, // Spore Crawler (1/2 vanilla)
        2001, 2001, // Venom Fang (2/3 Lethal)
        2005, 2005, // Pack Hunter (2/2 Rush)
        2002, 2002, // Regenerating Ooze (2/5 Regenerate)
        2010, 2010, // Acid Spitter (3/3 Ranged)
        2007, 2007, // Evolution Chamber (2/4, buff +1/+1)
        2011, 2011, // Carapace Warrior (2/6 Regenerate)
        2008,       // Alpha Predator (5/5 Rush+Lethal)
        2009,       // Swarm Mother (4/6 Regenerate)
        2012,       // Rapid Mutation spell
        // Free-Walker Splash (9 cards)
        4007, 4007, // Reckless Charger (4/1 Charge+Rush)
        4001, 4001, // Berserker (3/2 Charge)
        4011, 4011, // Precision Shot spell
        4003, 4003, // Hired Blade (3/3)
        4009,       // The Warbringer (7/7 finisher)
    ];
    card_ids.iter().map(|&id| CardId(id)).collect()
}

/// Helper to set up essence for a player in test scenarios.
/// This simulates having reached the specified turn with normal essence growth.
pub fn setup_test_essence(state: &mut GameState, player: PlayerId, essence: u8) {
    let player_state = &mut state.players[player.index()];
    player_state.max_essence = essence;
    player_state.current_essence = essence;
}
