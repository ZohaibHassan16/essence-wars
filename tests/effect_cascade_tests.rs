//! Effect cascade and queue ordering tests.
//!
//! These tests verify:
//! - Multiple effects process in FIFO order
//! - Terminal states (player death) stop queue processing
//! - Chain deaths from buff/debuff effects
//! - Effect queue handles complex scenarios correctly

mod common;

use cardgame::cards::CardDatabase;
use cardgame::effects::{Effect, EffectSource, EffectTarget};
use cardgame::engine::EffectQueue;
use cardgame::keywords::Keywords;
use cardgame::state::{GameResult, WinReason};
use cardgame::types::{PlayerId, Slot};
use common::*;

// ============================================================================
// FIFO Order Tests
// ============================================================================

/// Test that multiple damage effects process in FIFO order
#[test]
fn test_effect_queue_fifo_order() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create creature with 10 health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        10,
        Keywords::none(),
    );

    // Queue 3 damage effects
    for amount in [3, 2, 5] {
        queue.push(
            Effect::Damage {
                target: EffectTarget::Creature {
                    owner: PlayerId::PLAYER_TWO,
                    slot: Slot(0),
                },
                amount,
                filter: None,
            },
            EffectSource::System,
        );
    }

    queue.process_all(&mut state, &card_db);

    // Creature should be dead (took 3 + 2 + 5 = 10 damage)
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).is_none());
}

/// Test that effects process in order even with mixed types
#[test]
fn test_mixed_effects_fifo() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create creature with 5 health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        3,
        5,
        Keywords::none(),
    );

    // Queue: damage 2, heal 1, damage 4
    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            amount: 2,
            filter: None,
        },
        EffectSource::System,
    );

    queue.push(
        Effect::Heal {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            amount: 1,
            filter: None,
        },
        EffectSource::System,
    );

    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            amount: 4,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Creature should be dead: 5 - 2 = 3, + 1 = 4, - 4 = 0
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).is_none());
}

// ============================================================================
// Terminal State Detection Tests
// ============================================================================

/// Test that queue stops when player reaches 0 life
#[test]
fn test_terminal_stops_queue() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Set player 2 to low life
    state.players[1].life = 5;

    // Queue more damage than player has life
    queue.push(
        Effect::Damage {
            target: EffectTarget::Player(PlayerId::PLAYER_TWO),
            amount: 10,
            filter: None,
        },
        EffectSource::System,
    );

    // Queue another effect that shouldn't process
    queue.push(
        Effect::Damage {
            target: EffectTarget::Player(PlayerId::PLAYER_ONE),
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Player 2 should be dead
    assert!(state.players[1].life <= 0);
    assert!(state.is_terminal());

    // Player 1 should NOT have taken damage (queue stopped)
    assert_eq!(state.players[0].life, 30);
}

/// Test game end detection with win condition
#[test]
fn test_win_condition_detection() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Set player 2 to exact lethal
    state.players[1].life = 5;

    // Deal exactly lethal damage
    queue.push(
        Effect::Damage {
            target: EffectTarget::Player(PlayerId::PLAYER_TWO),
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Game should be over with player 1 winning
    assert!(state.is_terminal());
    assert!(matches!(
        state.result,
        Some(GameResult::Win {
            winner: PlayerId::PLAYER_ONE,
            reason: WinReason::LifeReachedZero,
            ..
        })
    ));
}

// ============================================================================
// Multiple Creature Death Tests
// ============================================================================

/// Test multiple creatures dying from AoE effect
#[test]
fn test_aoe_kills_multiple_creatures() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create 3 creatures with low health
    for slot in 0..3 {
        create_test_creature(
            &mut state,
            PlayerId::PLAYER_TWO,
            Slot(slot),
            2,
            2,
            Keywords::none(),
        );
    }

    // Queue damage to all enemy creatures
    queue.push(
        Effect::Damage {
            target: EffectTarget::AllEnemyCreatures(PlayerId::PLAYER_ONE),
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // All creatures should be dead
    assert!(state.players[1].creatures.is_empty());
}

/// Test selective damage only kills weak creatures
#[test]
fn test_selective_damage() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create creatures with varying health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        2, // Will die
        Keywords::none(),
    );
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(1),
        2,
        5, // Will survive
        Keywords::none(),
    );
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(2),
        2,
        3, // Will die
        Keywords::none(),
    );

    // Queue 3 damage to all
    queue.push(
        Effect::Damage {
            target: EffectTarget::AllEnemyCreatures(PlayerId::PLAYER_ONE),
            amount: 3,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Only creature in slot 1 should survive
    assert_eq!(state.players[1].creatures.len(), 1);
    let survivor = state.get_creature(PlayerId::PLAYER_TWO, Slot(1)).unwrap();
    assert_eq!(survivor.current_health, 2); // 5 - 3 = 2
}

// ============================================================================
// Chain Effect Tests
// ============================================================================

/// Test debuff causing creature death
#[test]
fn test_debuff_causes_death() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create creature with 3 health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        4,
        3,
        Keywords::none(),
    );

    // Apply -0/-4 debuff (more than health)
    queue.push(
        Effect::BuffStats {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            attack: 0,
            health: -4,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Creature should be dead
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).is_none());
}

/// Test multiple debuffs eventually killing creature
#[test]
fn test_sequential_debuffs() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create creature with 6 health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        3,
        6,
        Keywords::none(),
    );

    // Queue multiple small debuffs
    for _ in 0..3 {
        queue.push(
            Effect::BuffStats {
                target: EffectTarget::Creature {
                    owner: PlayerId::PLAYER_TWO,
                    slot: Slot(0),
                },
                attack: 0,
                health: -2,
                filter: None,
            },
            EffectSource::System,
        );
    }

    queue.process_all(&mut state, &card_db);

    // Creature should be dead (6 - 2 - 2 - 2 = 0)
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).is_none());
}

// ============================================================================
// Shield Interaction Tests
// ============================================================================

/// Test shield blocks damage but not debuff
#[test]
fn test_shield_vs_debuff() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create shielded creature
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        3,
        3,
        Keywords::none().with_shield(),
    );

    // Queue damage (should be blocked) then debuff (should apply)
    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.push(
        Effect::BuffStats {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            attack: 0,
            health: -3,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Creature should be dead from debuff (shield blocked damage but not debuff)
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).is_none());
}

/// Test multiple damage instances with shield
#[test]
fn test_shield_blocks_first_damage_only() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create shielded creature with 5 health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        3,
        5,
        Keywords::none().with_shield(),
    );

    // Queue two damage effects
    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            amount: 2,
            filter: None,
        },
        EffectSource::System,
    );

    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            amount: 3,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // First damage blocked, second deals 3 damage
    let creature = state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).unwrap();
    assert_eq!(creature.current_health, 2); // 5 - 3 = 2
    assert!(!creature.keywords.has_shield()); // Shield consumed
}

// ============================================================================
// Empty Queue Tests
// ============================================================================

/// Test processing empty queue doesn't crash
#[test]
fn test_empty_queue() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Process empty queue - should not crash
    queue.process_all(&mut state, &card_db);

    // State should be unchanged
    assert_eq!(state.players[0].life, 30);
    assert_eq!(state.players[1].life, 30);
}

// ============================================================================
// Effect on Non-existent Target Tests
// ============================================================================

/// Test damage to empty slot doesn't crash
#[test]
fn test_damage_empty_slot() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Queue damage to empty slot
    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    // Should not crash
    queue.process_all(&mut state, &card_db);

    // State unchanged
    assert_eq!(state.players[1].life, 30);
}

/// Test heal to empty slot doesn't crash
#[test]
fn test_heal_empty_slot() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Queue heal to empty slot
    queue.push(
        Effect::Heal {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    // Should not crash
    queue.process_all(&mut state, &card_db);
}

// ============================================================================
// Buff Overflow Tests
// ============================================================================

/// Test that buff correctly increases stats
#[test]
fn test_buff_increases_stats() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create creature with base 2/3
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        3,
        Keywords::none(),
    );

    // Apply +2/+2 buff
    queue.push(
        Effect::BuffStats {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            attack: 2,
            health: 2,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Creature should be 4/5
    let creature = state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).unwrap();
    assert_eq!(creature.attack, 4);
    assert_eq!(creature.current_health, 5);
}

/// Test healing doesn't exceed max health
#[test]
fn test_heal_capped_at_max_health() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create creature with 5 max health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        5,
        Keywords::none(),
    );

    // Damage it first
    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            amount: 3,
            filter: None,
        },
        EffectSource::System,
    );

    // Then heal for more than missing
    queue.push(
        Effect::Heal {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            amount: 10,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Should be at max health (5), not 5 - 3 + 10 = 12
    let creature = state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).unwrap();
    assert_eq!(creature.current_health, 5);
}

// ============================================================================
// Draw Effect Tests
// ============================================================================

/// Test draw effect adds cards to hand
#[test]
fn test_draw_effect() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Add cards to deck
    state.players[0].deck.push(cardgame::state::CardInstance {
        card_id: cardgame::types::CardId(1),
    });
    state.players[0].deck.push(cardgame::state::CardInstance {
        card_id: cardgame::types::CardId(2),
    });

    let initial_hand_size = state.players[0].hand.len();

    // Queue draw 2
    queue.push(
        Effect::Draw {
            player: PlayerId::PLAYER_ONE,
            count: 2,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Hand should have 2 more cards
    assert_eq!(state.players[0].hand.len(), initial_hand_size + 2);
}
