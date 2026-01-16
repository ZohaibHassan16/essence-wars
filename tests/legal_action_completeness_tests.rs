//! Legal action completeness tests.
//!
//! These tests verify:
//! - Guard enforcement with multiple guards
//! - Full board states prevent PlayCard actions
//! - Spell targeting validation
//! - Edge cases in action generation
//!
//! Complements tests/unit/legal_tests.rs with comprehensive edge cases.

mod common;

use cardgame::actions::Action;
use cardgame::cards::{CardDatabase, CardDefinition, CardType, EffectDefinition};
use cardgame::effects::{CreatureFilter, TargetingRule};
use cardgame::keywords::Keywords;
use cardgame::legal::legal_actions;
use cardgame::state::{CardInstance, Creature, CreatureStatus, GameState, Support};
use cardgame::types::{CardId, CreatureInstanceId, PlayerId, Rarity, Slot};

/// Helper to set up resources for a player in test scenarios.
fn setup_resources(state: &mut GameState, player_idx: usize, ap: u8, essence: u8) {
    state.players[player_idx].action_points = ap;
    state.players[player_idx].max_essence = essence;
    state.players[player_idx].current_essence = essence;
}

// ============================================================================
// Test Card Database Builder
// ============================================================================

/// Create extended test card database with targeting spells
fn extended_card_db() -> CardDatabase {
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
        // Guard creature: cost 2, 1/4
        CardDefinition {
            id: 2,
            name: "Guard Creature".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 1,
                health: 4,
                keywords: vec!["Guard".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Ranged creature: cost 3, 2/2
        CardDefinition {
            id: 3,
            name: "Ranged Creature".to_string(),
            cost: 3,
            card_type: CardType::Creature {
                attack: 2,
                health: 2,
                keywords: vec!["Ranged".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Spell: target any creature
        CardDefinition {
            id: 4,
            name: "Targeted Spell".to_string(),
            cost: 1,
            card_type: CardType::Spell {
                targeting: TargetingRule::TargetCreature(CreatureFilter::any()),
                effects: vec![EffectDefinition::Damage { amount: 2, filter: None }],
                conditional_effects: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Spell: target enemy creature
        CardDefinition {
            id: 5,
            name: "Enemy Spell".to_string(),
            cost: 1,
            card_type: CardType::Spell {
                targeting: TargetingRule::TargetEnemyCreature,
                effects: vec![EffectDefinition::Damage { amount: 3, filter: None }],
                conditional_effects: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Spell: target friendly creature
        CardDefinition {
            id: 6,
            name: "Friendly Spell".to_string(),
            cost: 1,
            card_type: CardType::Spell {
                targeting: TargetingRule::TargetAllyCreature,
                effects: vec![EffectDefinition::Heal { amount: 2, filter: None }],
                conditional_effects: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Spell: no target
        CardDefinition {
            id: 7,
            name: "AoE Spell".to_string(),
            cost: 2,
            card_type: CardType::Spell {
                targeting: TargetingRule::NoTarget,
                effects: vec![],
                conditional_effects: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Support: cost 3
        CardDefinition {
            id: 8,
            name: "Test Support".to_string(),
            cost: 3,
            card_type: CardType::Support {
                durability: 2,
                passive_effects: vec![],
                triggered_effects: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
    ];
    CardDatabase::new(cards)
}

/// Create a test creature with specified parameters
fn make_creature(
    card_id: u16,
    slot: u8,
    owner: PlayerId,
    turn_played: u16,
    keywords: Keywords,
) -> Creature {
    Creature {
        instance_id: CreatureInstanceId(slot as u32),
        card_id: CardId(card_id),
        owner,
        slot: Slot(slot),
        attack: 2,
        current_health: 3,
        max_health: 3,
        base_attack: 2,
        base_health: 3,
        keywords,
        status: CreatureStatus::default(),
        turn_played,
        frenzy_stacks: 0,
    }
}

/// Create a test support
fn make_support(card_id: u16, slot: u8, owner: PlayerId) -> Support {
    Support {
        card_id: CardId(card_id),
        owner,
        slot: Slot(slot),
        current_durability: 2,
    }
}

// ============================================================================
// Multiple Guards Tests
// ============================================================================

/// Test with two guards - can attack either
#[test]
fn test_multiple_guards_can_attack_either() {
    let card_db = extended_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Place an attacker
    let attacker = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none());
    state.players[0].creatures.push(attacker);

    // Place TWO guard creatures
    let guard1 = make_creature(2, 1, PlayerId::PLAYER_TWO, 1, Keywords::none().with_guard());
    let guard2 = make_creature(2, 3, PlayerId::PLAYER_TWO, 1, Keywords::none().with_guard());
    state.players[1].creatures.push(guard1);
    state.players[1].creatures.push(guard2);

    // Place a non-guard
    let non_guard = make_creature(1, 0, PlayerId::PLAYER_TWO, 1, Keywords::none());
    state.players[1].creatures.push(non_guard);

    let actions = legal_actions(&state, &card_db);

    // Should be able to attack EITHER guard (both are in range)
    let attacks: Vec<_> = actions
        .iter()
        .filter_map(|a| match a {
            Action::Attack { attacker: _, defender } => Some(defender.0),
            _ => None,
        })
        .collect();

    assert_eq!(attacks.len(), 2, "Should be able to attack exactly 2 guards");
    assert!(attacks.contains(&1), "Should be able to attack guard at slot 1");
    assert!(attacks.contains(&3), "Should be able to attack guard at slot 3");
    assert!(!attacks.contains(&0), "Should NOT be able to attack non-guard");
}

/// Test with guards but attacker can't reach one
#[test]
fn test_guard_out_of_range_allows_other_guard() {
    let card_db = extended_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Place an attacker at slot 0 (can only reach slots 0, 1)
    let attacker = make_creature(1, 0, PlayerId::PLAYER_ONE, 1, Keywords::none());
    state.players[0].creatures.push(attacker);

    // Place guard at slot 1 (in range)
    let guard1 = make_creature(2, 1, PlayerId::PLAYER_TWO, 1, Keywords::none().with_guard());
    state.players[1].creatures.push(guard1);

    // Place guard at slot 4 (out of range for non-ranged)
    let guard2 = make_creature(2, 4, PlayerId::PLAYER_TWO, 1, Keywords::none().with_guard());
    state.players[1].creatures.push(guard2);

    let actions = legal_actions(&state, &card_db);

    // Can only attack guard in range
    let attacks: Vec<_> = actions
        .iter()
        .filter_map(|a| match a {
            Action::Attack { attacker: _, defender } => Some(defender.0),
            _ => None,
        })
        .collect();

    assert_eq!(attacks.len(), 1, "Should only attack guard in range");
    assert!(attacks.contains(&1), "Should attack guard at slot 1");
}

/// Test ranged attacker with multiple guards
#[test]
fn test_ranged_with_multiple_guards() {
    let card_db = extended_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Place a RANGED attacker
    let attacker = make_creature(3, 0, PlayerId::PLAYER_ONE, 1, Keywords::none().with_ranged());
    state.players[0].creatures.push(attacker);

    // Place two guards at opposite ends
    let guard1 = make_creature(2, 0, PlayerId::PLAYER_TWO, 1, Keywords::none().with_guard());
    let guard2 = make_creature(2, 4, PlayerId::PLAYER_TWO, 1, Keywords::none().with_guard());
    state.players[1].creatures.push(guard1);
    state.players[1].creatures.push(guard2);

    // Place non-guard in middle
    let non_guard = make_creature(1, 2, PlayerId::PLAYER_TWO, 1, Keywords::none());
    state.players[1].creatures.push(non_guard);

    let actions = legal_actions(&state, &card_db);

    // Ranged can attack either guard but still can't bypass them
    let attacks: Vec<_> = actions
        .iter()
        .filter_map(|a| match a {
            Action::Attack { attacker: _, defender } => Some(defender.0),
            _ => None,
        })
        .collect();

    assert_eq!(attacks.len(), 2, "Ranged should attack either guard");
    assert!(attacks.contains(&0), "Should attack guard at slot 0");
    assert!(attacks.contains(&4), "Should attack guard at slot 4");
    assert!(!attacks.contains(&2), "Should NOT attack non-guard");
}

/// Test no guards allows normal targeting
#[test]
fn test_no_guards_normal_targeting() {
    let card_db = extended_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Place an attacker at slot 2
    let attacker = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none());
    state.players[0].creatures.push(attacker);

    // Place NON-guard creatures
    let creature1 = make_creature(1, 1, PlayerId::PLAYER_TWO, 1, Keywords::none());
    let creature2 = make_creature(1, 3, PlayerId::PLAYER_TWO, 1, Keywords::none());
    state.players[1].creatures.push(creature1);
    state.players[1].creatures.push(creature2);

    let actions = legal_actions(&state, &card_db);

    // Can attack both creatures and empty slot 2 (face)
    let attacks: Vec<_> = actions
        .iter()
        .filter_map(|a| match a {
            Action::Attack { attacker: _, defender } => Some(defender.0),
            _ => None,
        })
        .collect();

    assert_eq!(attacks.len(), 3, "Should attack creatures + empty slot");
    assert!(attacks.contains(&1), "Should attack creature at slot 1");
    assert!(attacks.contains(&2), "Should attack empty slot 2 (face)");
    assert!(attacks.contains(&3), "Should attack creature at slot 3");
}

// ============================================================================
// Full Board Tests
// ============================================================================

/// Test full creature board prevents creature PlayCard
#[test]
fn test_full_creature_board_no_creature_play() {
    let card_db = extended_card_db();
    let mut state = GameState::new();

    // Fill all 5 creature slots
    for slot in 0..5 {
        let creature = make_creature(1, slot, PlayerId::PLAYER_ONE, 0, Keywords::none());
        state.players[0].creatures.push(creature);
    }

    // Give player AP, essence, and a creature card
    setup_resources(&mut state, 0, 5, 10);
    state.players[0].hand.push(CardInstance::new(CardId(1))); // Creature

    let actions = legal_actions(&state, &card_db);

    // Should NOT have any PlayCard for creature
    let play_creature_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_creature_count, 0, "Full board = no creature plays");

    // Should still have EndTurn
    assert!(actions.contains(&Action::EndTurn));
}

/// Test full support slots prevents support PlayCard
#[test]
fn test_full_support_slots_no_support_play() {
    let card_db = extended_card_db();
    let mut state = GameState::new();

    // Fill both support slots
    let support1 = make_support(8, 0, PlayerId::PLAYER_ONE);
    let support2 = make_support(8, 1, PlayerId::PLAYER_ONE);
    state.players[0].supports.push(support1);
    state.players[0].supports.push(support2);

    // Give player AP, essence, and a support card
    setup_resources(&mut state, 0, 5, 10);
    state.players[0].hand.push(CardInstance::new(CardId(8))); // Support

    let actions = legal_actions(&state, &card_db);

    // Should NOT have any PlayCard for support
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 0, "Full support slots = no support plays");
}

/// Test full creature board but support playable
#[test]
fn test_full_creatures_support_playable() {
    let card_db = extended_card_db();
    let mut state = GameState::new();

    // Fill all 5 creature slots
    for slot in 0..5 {
        let creature = make_creature(1, slot, PlayerId::PLAYER_ONE, 0, Keywords::none());
        state.players[0].creatures.push(creature);
    }

    // Give player AP, essence, and a support card
    setup_resources(&mut state, 0, 5, 10);
    state.players[0].hand.push(CardInstance::new(CardId(8))); // Support

    let actions = legal_actions(&state, &card_db);

    // Should have PlayCard for 2 support slots
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 2, "Full creatures but support slots open");
}

/// Test full supports but creatures playable
#[test]
fn test_full_supports_creatures_playable() {
    let card_db = extended_card_db();
    let mut state = GameState::new();

    // Fill both support slots
    let support1 = make_support(8, 0, PlayerId::PLAYER_ONE);
    let support2 = make_support(8, 1, PlayerId::PLAYER_ONE);
    state.players[0].supports.push(support1);
    state.players[0].supports.push(support2);

    // Give player AP, essence, and a creature card
    setup_resources(&mut state, 0, 5, 10);
    state.players[0].hand.push(CardInstance::new(CardId(1))); // Creature

    let actions = legal_actions(&state, &card_db);

    // Should have PlayCard for 5 creature slots
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 5, "Full supports but creature slots open");
}

// ============================================================================
// Spell Action Generation Tests
// ============================================================================
// Note: The engine generates exactly one PlayCard action per spell with slot 0.
// Target validation happens during spell resolution, not action generation.
// This design keeps the action space simple and fixed-size for neural networks.

/// Test all spells generate exactly one PlayCard action
#[test]
fn test_spell_generates_one_action() {
    let card_db = extended_card_db();
    let mut state = GameState::new();

    // Give player AP, essence, and a targeted spell
    setup_resources(&mut state, 0, 5, 10);
    state.players[0].hand.push(CardInstance::new(CardId(5))); // Enemy spell

    let actions = legal_actions(&state, &card_db);

    // All spells generate exactly one PlayCard action (slot 0)
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 1, "Spell generates exactly 1 PlayCard action");

    // Verify the action uses slot 0
    let spell_action = actions
        .iter()
        .find(|a| matches!(a, Action::PlayCard { .. }))
        .unwrap();
    assert!(matches!(
        spell_action,
        Action::PlayCard { hand_index: 0, slot } if slot.0 == 0
    ));
}

/// Test multiple spells each generate one action
#[test]
fn test_multiple_spells_multiple_actions() {
    let card_db = extended_card_db();
    let mut state = GameState::new();

    // Give player AP, essence, and multiple spells
    setup_resources(&mut state, 0, 10, 10);
    state.players[0].hand.push(CardInstance::new(CardId(4))); // Any creature spell
    state.players[0].hand.push(CardInstance::new(CardId(5))); // Enemy spell
    state.players[0].hand.push(CardInstance::new(CardId(7))); // NoTarget spell

    let actions = legal_actions(&state, &card_db);

    // Each spell generates one PlayCard action
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 3, "3 spells = 3 PlayCard actions");
}

/// Test NoTarget spell always playable
#[test]
fn test_no_target_spell_always_playable() {
    let card_db = extended_card_db();
    let mut state = GameState::new();

    // Give player AP, essence, and no-target spell
    setup_resources(&mut state, 0, 5, 10);
    state.players[0].hand.push(CardInstance::new(CardId(7))); // NoTarget spell

    // Empty board

    let actions = legal_actions(&state, &card_db);

    // Spell should be playable (one action)
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 1, "NoTarget spell always playable");
}

/// Test spell with creatures on board
#[test]
fn test_spell_with_targets_available() {
    let card_db = extended_card_db();
    let mut state = GameState::new();

    // Give player AP, essence, and enemy-target spell
    setup_resources(&mut state, 0, 5, 10);
    state.players[0].hand.push(CardInstance::new(CardId(5))); // Enemy spell

    // Place enemy creatures
    let creature1 = make_creature(1, 0, PlayerId::PLAYER_TWO, 0, Keywords::none());
    let creature2 = make_creature(1, 2, PlayerId::PLAYER_TWO, 0, Keywords::none());
    state.players[1].creatures.push(creature1);
    state.players[1].creatures.push(creature2);

    let actions = legal_actions(&state, &card_db);

    // Still generates exactly one action (targeting resolved at execution)
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 1, "Spell action count unchanged by targets");
}

// ============================================================================
// Edge Case Tests
// ============================================================================

/// Test zero AP means no cards playable
#[test]
fn test_zero_ap_no_plays() {
    let card_db = extended_card_db();
    let mut state = GameState::new();

    // Give player 0 AP (but essence) and cards - should still not be playable due to 0 AP
    setup_resources(&mut state, 0, 0, 10);
    state.players[0].hand.push(CardInstance::new(CardId(1))); // cost 2
    state.players[0].hand.push(CardInstance::new(CardId(7))); // cost 2

    let actions = legal_actions(&state, &card_db);

    // No PlayCard actions
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 0, "Zero AP = no plays");

    // EndTurn still available
    assert!(actions.contains(&Action::EndTurn));
}

/// Test empty hand means no PlayCard
#[test]
fn test_empty_hand_no_plays() {
    let card_db = extended_card_db();
    let mut state = GameState::new();

    // Give player AP and essence but no cards
    setup_resources(&mut state, 0, 10, 10);

    let actions = legal_actions(&state, &card_db);

    // No PlayCard actions
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 0, "Empty hand = no plays");
}

/// Test essence exactly matching card cost (1 AP is enough)
#[test]
fn test_exact_essence_for_card() {
    let card_db = extended_card_db();
    let mut state = GameState::new();

    // Give player 1 AP and exactly 2 essence for a cost-2 card
    setup_resources(&mut state, 0, 1, 2);
    state.players[0].hand.push(CardInstance::new(CardId(1))); // cost 2

    let actions = legal_actions(&state, &card_db);

    // Should be playable (1 AP is enough, 2 essence covers cost)
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 5, "1 AP + exact essence = playable in all 5 slots");
}

/// Test one-less essence than card cost
#[test]
fn test_one_less_essence_than_cost() {
    let card_db = extended_card_db();
    let mut state = GameState::new();

    // Give player 3 AP but only 1 essence for a cost-2 card
    setup_resources(&mut state, 0, 3, 1);
    state.players[0].hand.push(CardInstance::new(CardId(1))); // cost 2

    let actions = legal_actions(&state, &card_db);

    // Should NOT be playable (not enough essence)
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 0, "One less essence than cost = not playable");
}

/// Test AP exactly matching 1 (minimum needed for an action)
#[test]
fn test_exact_ap_for_card() {
    let card_db = extended_card_db();
    let mut state = GameState::new();

    // Give player exactly 1 AP and enough essence
    setup_resources(&mut state, 0, 1, 10);
    state.players[0].hand.push(CardInstance::new(CardId(1))); // cost 2

    let actions = legal_actions(&state, &card_db);

    // Should be playable (1 AP is minimum needed)
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 5, "Exactly 1 AP + essence = playable");
}

/// Test multiple attackers with different ranges
#[test]
fn test_multiple_attackers_different_ranges() {
    let card_db = extended_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Place normal attacker at slot 0
    let attacker1 = make_creature(1, 0, PlayerId::PLAYER_ONE, 1, Keywords::none());
    state.players[0].creatures.push(attacker1);

    // Place ranged attacker at slot 4
    let attacker2 = make_creature(3, 4, PlayerId::PLAYER_ONE, 1, Keywords::none().with_ranged());
    state.players[0].creatures.push(attacker2);

    // Place enemy at slot 2
    let enemy = make_creature(1, 2, PlayerId::PLAYER_TWO, 1, Keywords::none());
    state.players[1].creatures.push(enemy);

    let actions = legal_actions(&state, &card_db);

    // Normal attacker at slot 0 can attack slots 0, 1 (enemy at 2 is out of range)
    // Ranged attacker at slot 4 can attack all slots including 2
    let attacks: Vec<_> = actions
        .iter()
        .filter(|a| matches!(a, Action::Attack { .. }))
        .collect();

    // Slot 0 attacker: 2 attacks (slots 0, 1)
    // Slot 4 ranged: 5 attacks (all slots)
    // Total: 7 attacks
    assert_eq!(attacks.len(), 7, "2 normal + 5 ranged attacks");

    // Verify ranged can hit slot 2
    assert!(actions.contains(&Action::Attack {
        attacker: Slot(4),
        defender: Slot(2),
    }));
}

/// Test creature at slot 0 range
#[test]
fn test_slot_0_attack_range() {
    let card_db = extended_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Place attacker at slot 0
    let attacker = make_creature(1, 0, PlayerId::PLAYER_ONE, 1, Keywords::none());
    state.players[0].creatures.push(attacker);

    let actions = legal_actions(&state, &card_db);

    // Slot 0 should attack slots 0, 1 only
    let attacks: Vec<_> = actions
        .iter()
        .filter_map(|a| match a {
            Action::Attack { attacker, defender } => Some((attacker.0, defender.0)),
            _ => None,
        })
        .collect();

    assert_eq!(attacks.len(), 2);
    assert!(attacks.contains(&(0, 0)));
    assert!(attacks.contains(&(0, 1)));
}

/// Test creature at slot 4 range
#[test]
fn test_slot_4_attack_range() {
    let card_db = extended_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Place attacker at slot 4
    let attacker = make_creature(1, 4, PlayerId::PLAYER_ONE, 1, Keywords::none());
    state.players[0].creatures.push(attacker);

    let actions = legal_actions(&state, &card_db);

    // Slot 4 should attack slots 3, 4 only
    let attacks: Vec<_> = actions
        .iter()
        .filter_map(|a| match a {
            Action::Attack { attacker, defender } => Some((attacker.0, defender.0)),
            _ => None,
        })
        .collect();

    assert_eq!(attacks.len(), 2);
    assert!(attacks.contains(&(4, 3)));
    assert!(attacks.contains(&(4, 4)));
}

/// Test all creatures exhausted
#[test]
fn test_all_creatures_exhausted() {
    let card_db = extended_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Place exhausted creatures
    for slot in 0..3 {
        let mut creature = make_creature(1, slot, PlayerId::PLAYER_ONE, 1, Keywords::none());
        creature.status.set_exhausted(true);
        state.players[0].creatures.push(creature);
    }

    let actions = legal_actions(&state, &card_db);

    // No attacks should be possible
    let attack_count = actions
        .iter()
        .filter(|a| matches!(a, Action::Attack { .. }))
        .count();
    assert_eq!(attack_count, 0, "All exhausted = no attacks");
}

/// Test mixed exhausted and ready creatures
#[test]
fn test_mixed_exhausted_ready() {
    let card_db = extended_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Exhausted creature at slot 0
    let mut exhausted = make_creature(1, 0, PlayerId::PLAYER_ONE, 1, Keywords::none());
    exhausted.status.set_exhausted(true);
    state.players[0].creatures.push(exhausted);

    // Ready creature at slot 2
    let ready = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none());
    state.players[0].creatures.push(ready);

    let actions = legal_actions(&state, &card_db);

    // Only slot 2 creature should be able to attack
    let attacks: Vec<_> = actions
        .iter()
        .filter_map(|a| match a {
            Action::Attack { attacker, .. } => Some(attacker.0),
            _ => None,
        })
        .collect();

    assert!(attacks.iter().all(|&s| s == 2), "Only slot 2 attacks");
    assert_eq!(attacks.len(), 3, "Slot 2 has 3 adjacent targets");
}
