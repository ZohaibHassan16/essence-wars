//! Common test utilities and helpers shared across all test files.

// Allow dead_code because each test binary is compiled independently,
// so functions used by other test files appear "unused" to each binary.
#![allow(dead_code)]

use cardgame::cards::{
    AbilityDefinition, CardDatabase, CardDefinition, CardType, EffectDefinition,
    PassiveEffectDefinition, PassiveModifier,
};
use cardgame::effects::{TargetingRule, Trigger};
use cardgame::keywords::Keywords;
use cardgame::state::{Creature, CreatureStatus, GameState};
use cardgame::types::{CardId, PlayerId, Rarity, Slot};

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
                effects: vec![EffectDefinition::Damage { amount: 3 }],
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
                effects: vec![EffectDefinition::BuffStats { attack: 2, health: 2 }],
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
                    effects: vec![EffectDefinition::Heal { amount: 2 }],
                }],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
    ];
    CardDatabase::new(cards)
}

/// Create a valid deck for integration tests using YAML card IDs
pub fn valid_yaml_deck() -> Vec<CardId> {
    let valid_ids = [1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 15, 32, 33, 40];
    (0..20).map(|i| CardId(valid_ids[i % valid_ids.len()] as u16)).collect()
}

/// Create the standard arena deck for bot testing.
/// This is the same deck used by the arena binary's default deck.
/// It's a well-balanced "Aggressive Assault" style deck that works
/// well for testing bot performance comparisons.
pub fn arena_test_deck() -> Vec<CardId> {
    let card_ids = [
        1, 1,   // Eager Recruit x2
        3, 3,   // Nimble Scout x2
        6, 6,   // Frontier Ranger x2
        8, 8,   // Shielded Squire x2
        11, 11, // Centaur Charger x2
        12, 12, // Blade Dancer x2
        16, 16, // Piercing Striker x2
        20, 20, // Siege Breaker x2
        34, 34, // Lightning Bolt x2
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
