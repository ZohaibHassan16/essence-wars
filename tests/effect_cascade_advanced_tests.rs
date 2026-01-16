//! Advanced effect cascade and death chain tests.
//!
//! These tests verify complex scenarios:
//! - Death chain reactions (OnDeath triggers that cause more deaths)
//! - Multiple OnAllyDeath triggers from a single death
//! - Effect queue FIFO ordering
//! - Buff/debuff accumulation leading to death
//! - Terminal state halting effect queue
//!
//! TODO NOTE: The engine's `effect_def_to_effect` function ignores the `TargetingRule`
//! in ability definitions. For triggered abilities:
//! - Damage defaults to AllEnemyCreatures (not the enemy player)
//! - Heal/Buff defaults to self (source creature)
//! - Draw defaults to source player
//! This is by design for simplicity but limits what OnDeath effects can do.

mod common;

use cardgame::cards::{AbilityDefinition, CardDatabase, CardDefinition, CardType, EffectDefinition};
use cardgame::effects::{Effect, EffectSource, EffectTarget, TargetingRule, Trigger};
use cardgame::engine::EffectQueue;
use cardgame::keywords::Keywords;
use cardgame::state::{GameResult, WinReason};
use cardgame::types::{CardId, PlayerId, Rarity, Slot};
use common::*;

// ============================================================================
// Test Database with Death-Triggering Cards
// ============================================================================

/// Create a card database specifically for death cascade testing
///
/// NOTE: Due to engine limitations, OnDeath damage effects always target
/// AllEnemyCreatures regardless of the TargetingRule specified.
fn death_cascade_test_db() -> CardDatabase {
    let cards = vec![
        // Card 1: Basic creature for testing
        CardDefinition {
            id: 1,
            name: "Basic Creature".to_string(),
            cost: 1,
            card_type: CardType::Creature {
                attack: 2,
                health: 2,
                keywords: vec![],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Card 2: OnDeath: Deal 2 damage to all enemy creatures
        // (Engine ignores TargetingRule, always uses AllEnemyCreatures for damage)
        CardDefinition {
            id: 2,
            name: "Death Striker".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 1,
                health: 2,
                keywords: vec![],
                abilities: vec![AbilityDefinition {
                    trigger: Trigger::OnDeath,
                    targeting: TargetingRule::NoTarget, // Ignored by engine
                    effects: vec![EffectDefinition::Damage { amount: 2, filter: None }],
                    conditional_effects: vec![],
                }],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
        // Card 3: OnDeath: Deal 3 damage to all enemy creatures
        CardDefinition {
            id: 3,
            name: "Explosive Creature".to_string(),
            cost: 3,
            card_type: CardType::Creature {
                attack: 2,
                health: 3,
                keywords: vec![],
                abilities: vec![AbilityDefinition {
                    trigger: Trigger::OnDeath,
                    targeting: TargetingRule::NoTarget, // Ignored by engine
                    effects: vec![EffectDefinition::Damage { amount: 3, filter: None }],
                    conditional_effects: vec![],
                }],
            },
            rarity: Rarity::Rare,
            tags: vec![],
        },
        // Card 4: OnAllyDeath: +1/+1 to self
        CardDefinition {
            id: 4,
            name: "Mourning Champion".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 1,
                health: 3,
                keywords: vec![],
                abilities: vec![AbilityDefinition {
                    trigger: Trigger::OnAllyDeath,
                    targeting: TargetingRule::NoTarget,
                    effects: vec![EffectDefinition::BuffStats { attack: 1, health: 1, filter: None }],
                    conditional_effects: vec![],
                }],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
        // Card 5: OnAllyDeath: Draw a card
        CardDefinition {
            id: 5,
            name: "Soul Collector".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 1,
                health: 2,
                keywords: vec![],
                abilities: vec![AbilityDefinition {
                    trigger: Trigger::OnAllyDeath,
                    targeting: TargetingRule::NoTarget,
                    effects: vec![EffectDefinition::Draw { count: 1 }],
                    conditional_effects: vec![],
                }],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
    ];
    CardDatabase::new(cards)
}

// ============================================================================
// Basic OnDeath Trigger Tests
// ============================================================================

/// Test that OnDeath triggers fire when a creature dies from damage
/// Death Striker's OnDeath deals 2 damage to all enemy creatures.
#[test]
fn test_ondeath_triggers_on_damage() {
    let mut state = create_effect_test_state();
    let card_db = death_cascade_test_db();
    let mut queue = EffectQueue::new();

    // Create "Death Striker" (OnDeath: 2 damage to all enemy creatures)
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        1,
        2,
        Keywords::none(),
    );
    // Set the card_id to match Death Striker
    state.players[0].creatures[0].card_id = CardId(2);

    // Create enemy creature that will receive the OnDeath damage
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        5,
        Keywords::none(),
    );

    // Queue lethal damage to the Death Striker
    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Death Striker should be dead
    assert!(state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).is_none());

    // Enemy creature should have taken 2 damage from OnDeath
    let enemy = state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).unwrap();
    assert_eq!(enemy.current_health, 3); // 5 - 2 = 3
}

/// Test OnAllyDeath triggers when an ally dies
#[test]
fn test_onallydeath_triggers() {
    let mut state = create_effect_test_state();
    let card_db = death_cascade_test_db();
    let mut queue = EffectQueue::new();

    // Create "Mourning Champion" (OnAllyDeath: +1/+1)
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        1,
        3,
        Keywords::none(),
    );
    state.players[0].creatures[0].card_id = CardId(4);

    // Create a basic ally that will die
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(1),
        2,
        2,
        Keywords::none(),
    );

    // Kill the ally
    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(1),
            },
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Ally should be dead
    assert!(state.get_creature(PlayerId::PLAYER_ONE, Slot(1)).is_none());

    // Mourning Champion should have +1/+1 from OnAllyDeath
    let champion = state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert_eq!(champion.attack, 2); // 1 + 1 = 2
    assert_eq!(champion.current_health, 4); // 3 + 1 = 4
}

// ============================================================================
// Multiple Trigger Tests
// ============================================================================

/// Test multiple OnAllyDeath triggers from a single death
#[test]
fn test_multiple_onallydeath_same_death() {
    let mut state = create_effect_test_state();
    let card_db = death_cascade_test_db();
    let mut queue = EffectQueue::new();

    // Create two "Mourning Champions" (OnAllyDeath: +1/+1)
    for slot in [Slot(0), Slot(1)] {
        create_test_creature(
            &mut state,
            PlayerId::PLAYER_ONE,
            slot,
            1,
            3,
            Keywords::none(),
        );
        let idx = state.players[0].creatures.len() - 1;
        state.players[0].creatures[idx].card_id = CardId(4);
    }

    // Create a basic ally that will die
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(2),
        2,
        2,
        Keywords::none(),
    );

    // Kill the ally
    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(2),
            },
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Both Mourning Champions should have +1/+1
    let champion1 = state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert_eq!(champion1.attack, 2);
    assert_eq!(champion1.current_health, 4);

    let champion2 = state.get_creature(PlayerId::PLAYER_ONE, Slot(1)).unwrap();
    assert_eq!(champion2.attack, 2);
    assert_eq!(champion2.current_health, 4);
}

/// Test OnDeath effect that deals AoE damage, killing multiple enemies
#[test]
fn test_ondeath_aoe_kills_multiple() {
    let mut state = create_effect_test_state();
    let card_db = death_cascade_test_db();
    let mut queue = EffectQueue::new();

    // Create "Explosive Creature" (OnDeath: 3 damage to all enemy creatures)
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        3,
        Keywords::none(),
    );
    state.players[0].creatures[0].card_id = CardId(3);

    // Create multiple enemy creatures with 3 or less health
    for slot in [Slot(0), Slot(1), Slot(2)] {
        create_test_creature(
            &mut state,
            PlayerId::PLAYER_TWO,
            slot,
            2,
            3,
            Keywords::none(),
        );
    }

    // Kill the Explosive Creature
    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Explosive Creature should be dead
    assert!(state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).is_none());

    // All enemy creatures should be dead from the OnDeath AoE
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).is_none());
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(1)).is_none());
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(2)).is_none());
}

// ============================================================================
// Death Chain Cascade Tests
// ============================================================================

/// Test death chain: OnDeath AoE kills creature with OnDeath AoE
#[test]
fn test_ondeath_chain_double_aoe() {
    let mut state = create_effect_test_state();
    let card_db = death_cascade_test_db();
    let mut queue = EffectQueue::new();

    // Create "Explosive Creature" for P1 (OnDeath: 3 damage to all enemy creatures)
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        3,
        Keywords::none(),
    );
    state.players[0].creatures[0].card_id = CardId(3);

    // Create "Explosive Creature" for P2
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        3,
        Keywords::none(),
    );
    state.players[1].creatures[0].card_id = CardId(3);

    // Create another P1 creature that will be hit by P2's OnDeath AoE
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(1),
        2,
        3,
        Keywords::none(),
    );

    // Kill P1's Explosive Creature
    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Chain:
    // 1. P1 Explosive dies -> OnDeath: 3 damage to all P2 creatures
    // 2. P2 Explosive dies -> OnDeath: 3 damage to all P1 creatures
    // 3. P1's other creature dies from P2's AoE

    assert!(state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).is_none());
    assert!(state.get_creature(PlayerId::PLAYER_ONE, Slot(1)).is_none());
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).is_none());
}

/// Test death chain with OnAllyDeath buffs from multiple deaths
#[test]
fn test_death_chain_with_onallydeath_buffs() {
    let mut state = create_effect_test_state();
    let card_db = death_cascade_test_db();
    let mut queue = EffectQueue::new();

    // Create "Mourning Champion" (OnAllyDeath: +1/+1)
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        1,
        2,
        Keywords::none(),
    );
    state.players[0].creatures[0].card_id = CardId(4);

    // Create 3 allies that will die
    for slot in [Slot(1), Slot(2), Slot(3)] {
        create_test_creature(
            &mut state,
            PlayerId::PLAYER_ONE,
            slot,
            1,
            1,
            Keywords::none(),
        );
    }

    // Kill all 3 allies via AoE
    queue.push(
        Effect::Damage {
            target: EffectTarget::AllAllyCreatures(PlayerId::PLAYER_ONE),
            amount: 1,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Mourning Champion should have +3/+3 from 3 ally deaths
    // Starting at 1/2, should be 4/5
    let champion = state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert_eq!(champion.attack, 4); // 1 + 3 = 4
    assert_eq!(champion.current_health, 4); // 2 - 1 (AoE) + 3 = 4
}

// ============================================================================
// Terminal State Tests
// ============================================================================

/// Test that effect queue stops processing when game ends
/// This tests the terminal state detection in the effect queue.
#[test]
fn test_cascade_stops_on_game_over() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Set P2 to low life
    state.players[1].life = 5;

    // Queue damage that will kill P2
    queue.push(
        Effect::Damage {
            target: EffectTarget::Player(PlayerId::PLAYER_TWO),
            amount: 10,
            filter: None,
        },
        EffectSource::System,
    );

    // Queue more effects that shouldn't process after game over
    queue.push(
        Effect::Damage {
            target: EffectTarget::Player(PlayerId::PLAYER_ONE),
            amount: 100,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Game should be over (P2 life <= 0)
    assert!(state.result.is_some());
    assert!(matches!(
        state.result,
        Some(GameResult::Win {
            winner: PlayerId::PLAYER_ONE,
            reason: WinReason::LifeReachedZero,
        })
    ));

    // P1 should NOT have taken 100 damage (queue stopped on game over)
    assert_eq!(state.players[0].life, 30);
}

// ============================================================================
// Debuff Accumulation Death Tests
// ============================================================================

/// Test creature dying from accumulated debuffs
#[test]
fn test_debuff_accumulation_kills_creature() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create creature with 4 health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        3,
        4,
        Keywords::none(),
    );

    // Apply -1 health debuffs until dead
    for _ in 0..5 {
        queue.push(
            Effect::BuffStats {
                target: EffectTarget::Creature {
                    owner: PlayerId::PLAYER_ONE,
                    slot: Slot(0),
                },
                attack: 0,
                health: -1,
                filter: None,
            },
            EffectSource::System,
        );
    }

    queue.process_all(&mut state, &card_db);

    // Creature should be dead (4 - 5 = -1 health)
    assert!(state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).is_none());
}

/// Test single large debuff killing creature
#[test]
fn test_single_large_debuff_kills() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create creature with 3 health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        3,
        Keywords::none(),
    );

    // Apply -5 health debuff
    queue.push(
        Effect::BuffStats {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            attack: 0,
            health: -5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Creature should be dead
    assert!(state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).is_none());
}

// ============================================================================
// Shield Interaction Tests During Cascades
// ============================================================================

/// Test Shield blocks damage during cascade, preventing death
#[test]
fn test_shield_survives_cascade_damage() {
    let mut state = create_effect_test_state();
    let card_db = death_cascade_test_db();
    let mut queue = EffectQueue::new();

    // Create "Explosive Creature" (OnDeath: 3 damage to all enemy creatures)
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        3,
        Keywords::none(),
    );
    state.players[0].creatures[0].card_id = CardId(3);

    // Create enemy creature WITH Shield
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        3,
        Keywords::none().with_shield(),
    );

    // Create enemy creature WITHOUT Shield
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(1),
        2,
        3,
        Keywords::none(),
    );

    // Kill the Explosive Creature
    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Shielded creature should survive (Shield absorbed the damage)
    let shielded = state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).unwrap();
    assert_eq!(shielded.current_health, 3); // Full health
    assert!(!shielded.keywords.has_shield()); // Shield consumed

    // Non-shielded creature should be dead
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(1)).is_none());
}

// ============================================================================
// Effect Queue FIFO Order Verification
// ============================================================================

/// Verify effects queue in order and process FIFO
#[test]
fn test_effect_queue_fifo_during_cascade() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Queue: damage 10, heal 5
    // P2 starts at 30: 30 - 10 = 20, 20 + 5 = 25
    queue.push(
        Effect::Damage {
            target: EffectTarget::Player(PlayerId::PLAYER_TWO),
            amount: 10,
            filter: None,
        },
        EffectSource::System,
    );

    queue.push(
        Effect::Heal {
            target: EffectTarget::Player(PlayerId::PLAYER_TWO),
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    assert_eq!(state.players[1].life, 25); // 30 - 10 + 5 = 25
}

/// Test mixed damage/heal effects on creature maintain FIFO
#[test]
fn test_creature_damage_heal_fifo() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create creature with 10 health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        10,
        Keywords::none(),
    );

    // Queue: damage 5, heal 3, damage 2
    // Result: 10 - 5 = 5, 5 + 3 = 8, 8 - 2 = 6
    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.push(
        Effect::Heal {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            amount: 3,
            filter: None,
        },
        EffectSource::System,
    );

    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            amount: 2,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    let creature = state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert_eq!(creature.current_health, 6);
}

// ============================================================================
// Simultaneous Death Tests
// ============================================================================

/// Test multiple creatures dying at once trigger all OnDeath effects
/// 3 Death Strikers die, each dealing 2 damage to all enemy creatures.
#[test]
fn test_simultaneous_deaths_all_trigger() {
    let mut state = create_effect_test_state();
    let card_db = death_cascade_test_db();
    let mut queue = EffectQueue::new();

    // Create 3 Death Strikers (OnDeath: 2 damage to all enemy creatures each)
    for slot in [Slot(0), Slot(1), Slot(2)] {
        create_test_creature(
            &mut state,
            PlayerId::PLAYER_ONE,
            slot,
            1,
            2,
            Keywords::none(),
        );
        let idx = state.players[0].creatures.len() - 1;
        state.players[0].creatures[idx].card_id = CardId(2);
    }

    // Create enemy creature that will receive 3x OnDeath damage
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        10,
        Keywords::none(),
    );

    // Kill all 3 Death Strikers with AoE
    queue.push(
        Effect::Damage {
            target: EffectTarget::AllAllyCreatures(PlayerId::PLAYER_ONE),
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // All 3 should be dead
    assert!(state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).is_none());
    assert!(state.get_creature(PlayerId::PLAYER_ONE, Slot(1)).is_none());
    assert!(state.get_creature(PlayerId::PLAYER_ONE, Slot(2)).is_none());

    // Enemy creature should have taken 6 damage (2 from each OnDeath)
    let enemy = state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).unwrap();
    assert_eq!(enemy.current_health, 4); // 10 - 6 = 4
}
