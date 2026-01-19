//! Effect queue system tests.
//!
//! Tests for the FIFO effect queue that processes game effects without recursion.

mod common;

use cardgame::cards::CardDatabase;
use cardgame::effects::{Effect, EffectSource, EffectTarget};
use cardgame::engine::EffectQueue;
use cardgame::keywords::Keywords;
use cardgame::state::{CardInstance, Creature, CreatureStatus, GameResult, WinReason};
use cardgame::types::{CardId, PlayerId, Slot};
use common::*;

#[test]
fn test_effect_queue_single_damage() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a creature with 5 health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        5,
        Keywords::none(),
    );

    // Queue damage effect
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

    // Check creature took damage
    let creature = state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).unwrap();
    assert_eq!(creature.current_health, 2);
}

#[test]
fn test_effect_queue_damage_kills_creature() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a creature with 3 health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        3,
        Keywords::none(),
    );

    // Queue lethal damage
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

    queue.process_all(&mut state, &card_db);

    // Check creature is dead
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).is_none());
}

#[test]
fn test_effect_queue_shield_blocks_damage() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a creature with shield
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        3,
        Keywords::none().with_shield(),
    );

    // Queue damage effect
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

    queue.process_all(&mut state, &card_db);

    // Check creature survived with full health, shield removed
    let creature = state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).unwrap();
    assert_eq!(creature.current_health, 3);
    assert!(!creature.keywords.has_shield());
}

#[test]
fn test_effect_queue_heal() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a damaged creature
    let instance_id = state.next_creature_instance_id();
    let creature = Creature {
        instance_id,
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        attack: 2,
        current_health: 2,
        max_health: 5,
        base_attack: 2,
        base_health: 5,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
        frenzy_stacks: 0,
    };
    state.players[0].creatures.push(creature);

    // Queue heal effect
    queue.push(
        Effect::Heal {
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

    // Check creature was healed
    let creature = state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert_eq!(creature.current_health, 4);
}

#[test]
fn test_effect_queue_heal_max_cap() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a slightly damaged creature
    let instance_id = state.next_creature_instance_id();
    let creature = Creature {
        instance_id,
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        attack: 2,
        current_health: 4,
        max_health: 5,
        base_attack: 2,
        base_health: 5,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
        frenzy_stacks: 0,
    };
    state.players[0].creatures.push(creature);

    // Queue large heal effect
    queue.push(
        Effect::Heal {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            amount: 10,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check creature is at max health
    let creature = state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert_eq!(creature.current_health, 5);
}

#[test]
fn test_effect_queue_draw() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Add cards to deck
    state.players[0].deck.push(CardInstance::new(CardId(1)));
    state.players[0].deck.push(CardInstance::new(CardId(2)));
    state.players[0].deck.push(CardInstance::new(CardId(3)));

    // Queue draw effect
    queue.push(
        Effect::Draw {
            player: PlayerId::PLAYER_ONE,
            count: 2,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check cards were drawn
    assert_eq!(state.players[0].hand.len(), 2);
    assert_eq!(state.players[0].deck.len(), 1);
}

#[test]
fn test_effect_queue_fifo_order() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create two creatures
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        10,
        Keywords::none(),
    );
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(1),
        2,
        10,
        Keywords::none(),
    );

    // Queue effects in order: damage slot 0, then damage slot 1
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
    queue.push(
        Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(1),
            },
            amount: 5,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Both should have taken damage
    assert_eq!(
        state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).unwrap().current_health,
        7
    );
    assert_eq!(
        state.get_creature(PlayerId::PLAYER_TWO, Slot(1)).unwrap().current_health,
        5
    );
}

#[test]
fn test_effect_queue_buff() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a creature
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        3,
        Keywords::none(),
    );

    // Queue buff effect
    queue.push(
        Effect::BuffStats {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            attack: 2,
            health: 2,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check creature was buffed
    let creature = state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert_eq!(creature.attack, 4);
    assert_eq!(creature.current_health, 5);
    assert_eq!(creature.max_health, 5);
}

#[test]
fn test_effect_queue_debuff_death() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a creature with 2 health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        2,
        Keywords::none(),
    );

    // Queue debuff that reduces health to 0
    queue.push(
        Effect::BuffStats {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            attack: 0,
            health: -3,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check creature is dead
    assert!(state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).is_none());
}

#[test]
fn test_effect_queue_destroy() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a creature
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        5,
        10,
        Keywords::none(),
    );

    // Queue destroy effect
    queue.push(
        Effect::Destroy {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check creature is destroyed
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).is_none());
}

#[test]
fn test_effect_queue_player_damage() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Queue damage to player
    queue.push(
        Effect::Damage {
            target: EffectTarget::Player(PlayerId::PLAYER_TWO),
            amount: 10,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check player took damage
    assert_eq!(state.players[1].life, 20);
}

#[test]
fn test_effect_queue_player_damage_game_over() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Queue lethal damage to player
    queue.push(
        Effect::Damage {
            target: EffectTarget::Player(PlayerId::PLAYER_TWO),
            amount: 30,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check game over
    assert!(state.is_terminal());
    match state.result {
        Some(GameResult::Win { winner, reason }) => {
            assert_eq!(winner, PlayerId::PLAYER_ONE);
            assert_eq!(reason, WinReason::LifeReachedZero);
        }
        _ => panic!("Expected Player One to win"),
    }
}

#[test]
fn test_effect_queue_grant_keyword() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a creature without rush
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        3,
        Keywords::none(),
    );

    // Queue grant Rush
    queue.push(
        Effect::GrantKeyword {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            keyword: Keywords::RUSH,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check creature has Rush
    let creature = state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert!(creature.keywords.has_rush());
}

#[test]
fn test_effect_queue_remove_keyword() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a creature with shield
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        3,
        Keywords::none().with_shield(),
    );

    // Queue remove shield
    queue.push(
        Effect::RemoveKeyword {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            keyword: Keywords::SHIELD,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check creature lost Shield
    let creature = state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert!(!creature.keywords.has_shield());
}

#[test]
fn test_effect_queue_silence() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a creature with keywords
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        3,
        Keywords::none().with_rush().with_guard().with_shield(),
    );

    // Queue silence
    queue.push(
        Effect::Silence {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check creature is silenced and lost all keywords
    let creature = state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert!(creature.status.is_silenced());
    assert!(!creature.keywords.has_rush());
    assert!(!creature.keywords.has_guard());
    assert!(!creature.keywords.has_shield());
}

#[test]
fn test_effect_queue_all_enemy_creatures_damage() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create enemy creatures
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        5,
        Keywords::none(),
    );
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(2),
        3,
        4,
        Keywords::none(),
    );

    // Create friendly creature
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        5,
        Keywords::none(),
    );

    // Queue damage to all enemy creatures (from player one's perspective)
    queue.push(
        Effect::Damage {
            target: EffectTarget::AllEnemyCreatures(PlayerId::PLAYER_ONE),
            amount: 2,
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check enemy creatures took damage
    assert_eq!(
        state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).unwrap().current_health,
        3
    );
    assert_eq!(
        state.get_creature(PlayerId::PLAYER_TWO, Slot(2)).unwrap().current_health,
        2
    );
    // Friendly creature should be unharmed
    assert_eq!(
        state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap().current_health,
        5
    );
}

#[test]
fn test_effect_queue_gain_essence() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    state.players[0].current_essence = 3;

    // Queue gain essence
    queue.push(
        Effect::GainEssence {
            player: PlayerId::PLAYER_ONE,
            amount: 2,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check essence was gained
    assert_eq!(state.players[0].current_essence, 5);
}

#[test]
fn test_effect_queue_refresh_creature() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create an exhausted creature
    let instance_id = state.next_creature_instance_id();
    let mut creature = Creature {
        instance_id,
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        attack: 2,
        current_health: 3,
        max_health: 3,
        base_attack: 2,
        base_health: 3,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
        frenzy_stacks: 0,
    };
    creature.status.set_exhausted(true);
    state.players[0].creatures.push(creature);

    // Verify it's exhausted
    assert!(state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap().status.is_exhausted());

    // Queue refresh
    queue.push(
        Effect::RefreshCreature {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check creature is refreshed
    assert!(!state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap().status.is_exhausted());
}

#[test]
fn test_effect_queue_empty_after_processing() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    queue.push(
        Effect::Draw {
            player: PlayerId::PLAYER_ONE,
            count: 1,
        },
        EffectSource::System,
    );
    queue.push(
        Effect::GainEssence {
            player: PlayerId::PLAYER_ONE,
            amount: 1,
        },
        EffectSource::System,
    );

    assert!(!queue.is_empty());
    assert_eq!(queue.len(), 2);

    queue.process_all(&mut state, &card_db);

    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
}

// ============================================================================
// Bounce Effect Tests
// ============================================================================

/// Test bouncing a single creature returns it to hand
#[test]
fn test_bounce_single_creature() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create enemy creature
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        3,
        4,
        Keywords::none(),
    );

    // Record original hand size
    let original_hand_size = state.players[1].hand.len();

    // Queue bounce effect
    queue.push(
        Effect::Bounce {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Creature should be removed from board
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).is_none());

    // Card should be added to hand
    assert_eq!(state.players[1].hand.len(), original_hand_size + 1);
}

/// Test bouncing multiple creatures with AoE
#[test]
fn test_bounce_all_enemy_creatures() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create multiple enemy creatures
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

    // Create friendly creature that should NOT be bounced
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        3,
        Keywords::none(),
    );

    let original_p2_hand_size = state.players[1].hand.len();

    // Queue bounce all enemy creatures
    queue.push(
        Effect::Bounce {
            target: EffectTarget::AllEnemyCreatures(PlayerId::PLAYER_ONE),
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // All enemy creatures should be gone
    assert!(state.players[1].creatures.is_empty());

    // Cards should be in hand
    assert_eq!(state.players[1].hand.len(), original_p2_hand_size + 3);

    // Friendly creature should still be on board
    assert!(state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).is_some());
}

/// Test bounce with filter only affects matching creatures
#[test]
fn test_bounce_with_filter() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create enemy creature with low health (should be bounced)
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        2, // Low health
        Keywords::none(),
    );

    // Create enemy creature with high health (should NOT be bounced)
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(1),
        3,
        5, // High health
        Keywords::none(),
    );

    let original_hand_size = state.players[1].hand.len();

    // Queue bounce with max_health filter
    use cardgame::effects::CreatureFilter;
    queue.push(
        Effect::Bounce {
            target: EffectTarget::AllEnemyCreatures(PlayerId::PLAYER_ONE),
            filter: Some(CreatureFilter::any().with_max_health(3)),
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Low health creature should be bounced
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).is_none());

    // High health creature should remain
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(1)).is_some());

    // Only one card added to hand
    assert_eq!(state.players[1].hand.len(), original_hand_size + 1);
}

/// Test bounce when hand is full discards the card
#[test]
fn test_bounce_hand_full_discards() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Fill player 2's hand to maximum
    while !state.players[1].is_hand_full() {
        state.players[1].hand.push(cardgame::state::CardInstance::new(cardgame::types::CardId(1)));
    }
    let max_hand_size = state.players[1].hand.len();

    // Create enemy creature
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        3,
        4,
        Keywords::none(),
    );

    // Queue bounce effect
    queue.push(
        Effect::Bounce {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Creature should be removed from board
    assert!(state.get_creature(PlayerId::PLAYER_TWO, Slot(0)).is_none());

    // Hand should still be at max (card was discarded, not added)
    assert_eq!(state.players[1].hand.len(), max_hand_size);
}

// ============================================================================
// Conditional Trigger Tests (Phase 4B.1)
// ============================================================================

#[test]
fn test_accumulated_result_tracks_target_death_from_damage() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a creature with 2 health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        2,
        Keywords::none(),
    );

    // Reset accumulated result
    queue.reset_accumulated_result();

    // Queue lethal damage
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

    // Check that target_died was tracked
    assert!(queue.accumulated_result().target_died);
}

#[test]
fn test_accumulated_result_no_death_when_creature_survives() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a creature with 5 health
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        5,
        Keywords::none(),
    );

    // Reset accumulated result
    queue.reset_accumulated_result();

    // Queue non-lethal damage
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

    queue.process_all(&mut state, &card_db);

    // Check that target_died is false
    assert!(!queue.accumulated_result().target_died);
}

#[test]
fn test_accumulated_result_tracks_target_death_from_destroy() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a creature
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        5,
        Keywords::none(),
    );

    // Reset accumulated result
    queue.reset_accumulated_result();

    // Queue destroy effect
    queue.push(
        Effect::Destroy {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_TWO,
                slot: Slot(0),
            },
            filter: None,
        },
        EffectSource::System,
    );

    queue.process_all(&mut state, &card_db);

    // Check that target_died was tracked
    assert!(queue.accumulated_result().target_died);
}

#[test]
fn test_accumulated_result_reset_clears_previous_state() {
    let mut state = create_effect_test_state();
    let card_db = CardDatabase::empty();
    let mut queue = EffectQueue::new();

    // Create a creature
    create_test_creature(
        &mut state,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        2,
        Keywords::none(),
    );

    // Reset accumulated result
    queue.reset_accumulated_result();

    // Queue lethal damage
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

    queue.process_all(&mut state, &card_db);

    // Verify target died
    assert!(queue.accumulated_result().target_died);

    // Reset accumulated result
    queue.reset_accumulated_result();

    // Verify it's cleared
    assert!(!queue.accumulated_result().target_died);
}
