//! Comprehensive tests for the Essence and Action Point (AP) resource system.
//!
//! The resource system is fundamental to game balance:
//! - **Essence**: Mana for playing cards. Grows +1 max per turn, refills each turn. Cap: 10.
//! - **Action Points (AP)**: Actions per turn. Fixed at 3 per turn. Each action costs 1 AP.
//!
//! These tests ensure the resource system works exactly as designed and prevent regression.

use cardgame::actions::Action;
use cardgame::cards::{CardDatabase, CardDefinition, CardType};
use cardgame::config::player;
use cardgame::effects::TargetingRule;
use cardgame::engine::GameEngine;
use cardgame::legal::legal_actions;
use cardgame::state::{CardInstance, GameState};
use cardgame::types::{CardId, PlayerId, Rarity, Slot};

// =============================================================================
// TEST FIXTURES
// =============================================================================

/// Create a test card database with cards of various costs
fn resource_test_db() -> CardDatabase {
    let cards = vec![
        // Cost 1 creature
        CardDefinition {
            id: 1,
            name: "Cheap Creature".to_string(),
            cost: 1,
            card_type: CardType::Creature {
                attack: 1,
                health: 1,
                keywords: vec![],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Cost 2 creature
        CardDefinition {
            id: 2,
            name: "Medium Creature".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 2,
                health: 2,
                keywords: vec![],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Cost 5 creature
        CardDefinition {
            id: 3,
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
        // Cost 10 creature (max essence cost)
        CardDefinition {
            id: 4,
            name: "Ultimate Creature".to_string(),
            cost: 10,
            card_type: CardType::Creature {
                attack: 10,
                health: 10,
                keywords: vec![],
                abilities: vec![],
            },
            rarity: Rarity::Legendary,
            tags: vec![],
        },
        // Cost 1 spell
        CardDefinition {
            id: 5,
            name: "Cheap Spell".to_string(),
            cost: 1,
            card_type: CardType::Spell {
                targeting: TargetingRule::NoTarget,
                effects: vec![],
                conditional_effects: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Cost 3 support
        CardDefinition {
            id: 6,
            name: "Medium Support".to_string(),
            cost: 3,
            card_type: CardType::Support {
                durability: 3,
                passive_effects: vec![],
                triggered_effects: vec![],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
    ];
    CardDatabase::new(cards)
}

/// Create a simple deck for testing
fn simple_test_deck() -> Vec<CardId> {
    // 30 cards: mix of costs
    let mut deck = Vec::new();
    for _ in 0..10 {
        deck.push(CardId(1)); // Cost 1
        deck.push(CardId(2)); // Cost 2
        deck.push(CardId(3)); // Cost 5
    }
    deck
}

// =============================================================================
// ESSENCE GROWTH TESTS
// =============================================================================

#[test]
fn test_essence_starts_at_zero() {
    let card_db = resource_test_db();
    let engine = GameEngine::new(&card_db);

    // Before start_game, essence should be 0
    assert_eq!(engine.state.players[0].max_essence, 0);
    assert_eq!(engine.state.players[0].current_essence, 0);
    assert_eq!(engine.state.players[1].max_essence, 0);
    assert_eq!(engine.state.players[1].current_essence, 0);
}

#[test]
fn test_first_turn_gives_one_essence() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // After first turn starts (P1's turn), P1 should have 1 essence
    assert_eq!(engine.state.players[0].max_essence, 1);
    assert_eq!(engine.state.players[0].current_essence, 1);

    // P2 hasn't had a turn yet, max_essence and current_essence are 0 until their turn starts
    // (FPA compensation is now via bonus cards, not essence)
    assert_eq!(engine.state.players[1].max_essence, 0);
    assert_eq!(engine.state.players[1].current_essence, 0);
}

#[test]
fn test_essence_grows_each_turn() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // Turn 1: P1 has 1 essence
    assert_eq!(engine.state.players[0].max_essence, 1);
    assert_eq!(engine.turn_number(), 1);

    // End P1's turn, P2's turn starts
    // Note: P2 starts with 1 essence (same as P1; FPA compensation is via bonus cards)
    engine.apply_action(Action::EndTurn).unwrap();
    assert_eq!(engine.state.players[1].max_essence, 1);
    assert_eq!(engine.state.players[1].current_essence, 1);

    // End P2's turn, P1's turn 2 starts
    engine.apply_action(Action::EndTurn).unwrap();
    assert_eq!(engine.state.players[0].max_essence, 2);
    assert_eq!(engine.state.players[0].current_essence, 2);

    // End P1's turn, P2's turn 2 starts
    engine.apply_action(Action::EndTurn).unwrap();
    assert_eq!(engine.state.players[1].max_essence, 2);
    assert_eq!(engine.state.players[1].current_essence, 2);
}

#[test]
fn test_essence_caps_at_max() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // Simulate many turns to reach max essence
    for _ in 0..25 {
        engine.apply_action(Action::EndTurn).unwrap();
        if engine.is_game_over() {
            break;
        }
    }

    // Both players should be capped at MAX_ESSENCE
    assert!(engine.state.players[0].max_essence <= player::MAX_ESSENCE);
    assert!(engine.state.players[1].max_essence <= player::MAX_ESSENCE);

    // Verify the cap value is correct (10)
    assert_eq!(player::MAX_ESSENCE, 10);
}

#[test]
fn test_essence_refills_each_turn() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // Advance to turn where we have 3 essence
    for _ in 0..4 {
        engine.apply_action(Action::EndTurn).unwrap();
    }

    // P1 should have 3 essence
    assert_eq!(engine.state.active_player, PlayerId::PLAYER_ONE);
    assert_eq!(engine.state.players[0].max_essence, 3);
    assert_eq!(engine.state.players[0].current_essence, 3);

    // Spend some essence by playing a card
    if !engine.state.players[0].hand.is_empty() {
        let card = &engine.state.players[0].hand[0];
        let cost = card_db.get(card.card_id).map(|c| c.cost).unwrap_or(0);

        if cost <= 3 {
            engine.apply_action(Action::PlayCard {
                hand_index: 0,
                slot: Slot(0),
            }).unwrap();

            // Essence should be reduced
            assert_eq!(engine.state.players[0].current_essence, 3 - cost);

            // End turns to get back to P1
            engine.apply_action(Action::EndTurn).unwrap();
            engine.apply_action(Action::EndTurn).unwrap();

            // Essence should be refilled (and max increased by 1)
            assert_eq!(engine.state.players[0].max_essence, 4);
            assert_eq!(engine.state.players[0].current_essence, 4);
        }
    }
}

// =============================================================================
// ACTION POINT TESTS
// =============================================================================

#[test]
fn test_ap_per_turn_is_three() {
    // Verify the constant
    assert_eq!(player::AP_PER_TURN, 3);
}

#[test]
fn test_ap_given_each_turn() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // P1 should have 3 AP on first turn
    assert_eq!(engine.state.players[0].action_points, 3);

    // End turn, P2 should have 3 AP
    engine.apply_action(Action::EndTurn).unwrap();
    assert_eq!(engine.state.players[1].action_points, 3);

    // End turn, P1 should have 3 AP again
    engine.apply_action(Action::EndTurn).unwrap();
    assert_eq!(engine.state.players[0].action_points, 3);
}

#[test]
fn test_action_costs_one_ap() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // Give player enough essence
    engine.state.players[0].max_essence = 10;
    engine.state.players[0].current_essence = 10;

    let initial_ap = engine.state.players[0].action_points;
    assert_eq!(initial_ap, 3);

    // Play a card - should cost 1 AP regardless of card cost
    engine.apply_action(Action::PlayCard {
        hand_index: 0,
        slot: Slot(0),
    }).unwrap();

    assert_eq!(engine.state.players[0].action_points, 2);

    // Play another card
    if !engine.state.players[0].hand.is_empty() {
        engine.apply_action(Action::PlayCard {
            hand_index: 0,
            slot: Slot(1),
        }).unwrap();

        assert_eq!(engine.state.players[0].action_points, 1);
    }
}

#[test]
fn test_cannot_act_with_zero_ap() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // Give player enough essence but 0 AP
    engine.state.players[0].max_essence = 10;
    engine.state.players[0].current_essence = 10;
    engine.state.players[0].action_points = 0;

    // Try to play a card - should fail
    let result = engine.apply_action(Action::PlayCard {
        hand_index: 0,
        slot: Slot(0),
    });

    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Not enough AP") || err_msg.contains("Illegal action"),
            "Expected AP error, got: {}", err_msg);
}

// =============================================================================
// CARD PLAYING COST TESTS
// =============================================================================

#[test]
fn test_card_cost_deducts_from_essence() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // Set up resources
    engine.state.players[0].max_essence = 10;
    engine.state.players[0].current_essence = 10;
    engine.state.players[0].action_points = 3;

    // Clear hand and add a known cost card
    engine.state.players[0].hand.clear();
    engine.state.players[0].hand.push(CardInstance::new(CardId(2))); // Cost 2

    let initial_essence = engine.state.players[0].current_essence;

    engine.apply_action(Action::PlayCard {
        hand_index: 0,
        slot: Slot(0),
    }).unwrap();

    // Essence should be reduced by card cost (2)
    assert_eq!(engine.state.players[0].current_essence, initial_essence - 2);
}

#[test]
fn test_cannot_play_without_enough_essence() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // Set up: 3 AP but only 1 essence
    engine.state.players[0].max_essence = 1;
    engine.state.players[0].current_essence = 1;
    engine.state.players[0].action_points = 3;

    // Clear hand and add a cost-5 card
    engine.state.players[0].hand.clear();
    engine.state.players[0].hand.push(CardInstance::new(CardId(3))); // Cost 5

    // Try to play - should fail
    let result = engine.apply_action(Action::PlayCard {
        hand_index: 0,
        slot: Slot(0),
    });

    assert!(result.is_err());
}

#[test]
fn test_can_play_with_exact_essence() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // Set up: exactly 2 essence for a cost-2 card
    engine.state.players[0].max_essence = 2;
    engine.state.players[0].current_essence = 2;
    engine.state.players[0].action_points = 3;

    // Clear hand and add a cost-2 card
    engine.state.players[0].hand.clear();
    engine.state.players[0].hand.push(CardInstance::new(CardId(2))); // Cost 2

    // Should succeed with exactly enough essence
    let result = engine.apply_action(Action::PlayCard {
        hand_index: 0,
        slot: Slot(0),
    });

    assert!(result.is_ok());
    assert_eq!(engine.state.players[0].current_essence, 0);
}

#[test]
fn test_can_play_multiple_cards_if_resources_allow() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // Set up: 3 AP and 5 essence
    engine.state.players[0].max_essence = 5;
    engine.state.players[0].current_essence = 5;
    engine.state.players[0].action_points = 3;

    // Clear hand and add three cost-1 cards
    engine.state.players[0].hand.clear();
    engine.state.players[0].hand.push(CardInstance::new(CardId(1))); // Cost 1
    engine.state.players[0].hand.push(CardInstance::new(CardId(1))); // Cost 1
    engine.state.players[0].hand.push(CardInstance::new(CardId(1))); // Cost 1

    // Play all three cards
    engine.apply_action(Action::PlayCard { hand_index: 0, slot: Slot(0) }).unwrap();
    assert_eq!(engine.state.players[0].action_points, 2);
    assert_eq!(engine.state.players[0].current_essence, 4);

    engine.apply_action(Action::PlayCard { hand_index: 0, slot: Slot(1) }).unwrap();
    assert_eq!(engine.state.players[0].action_points, 1);
    assert_eq!(engine.state.players[0].current_essence, 3);

    engine.apply_action(Action::PlayCard { hand_index: 0, slot: Slot(2) }).unwrap();
    assert_eq!(engine.state.players[0].action_points, 0);
    assert_eq!(engine.state.players[0].current_essence, 2);
}

#[test]
fn test_ap_limits_actions_even_with_essence() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // Set up: only 2 AP but lots of essence
    engine.state.players[0].max_essence = 10;
    engine.state.players[0].current_essence = 10;
    engine.state.players[0].action_points = 2;

    // Clear hand and add three cost-1 cards
    engine.state.players[0].hand.clear();
    engine.state.players[0].hand.push(CardInstance::new(CardId(1))); // Cost 1
    engine.state.players[0].hand.push(CardInstance::new(CardId(1))); // Cost 1
    engine.state.players[0].hand.push(CardInstance::new(CardId(1))); // Cost 1

    // Play first two cards - should work
    engine.apply_action(Action::PlayCard { hand_index: 0, slot: Slot(0) }).unwrap();
    engine.apply_action(Action::PlayCard { hand_index: 0, slot: Slot(1) }).unwrap();

    // Third card should fail - out of AP
    let result = engine.apply_action(Action::PlayCard { hand_index: 0, slot: Slot(2) });
    assert!(result.is_err());
}

// =============================================================================
// LEGAL ACTION GENERATION TESTS
// =============================================================================

#[test]
fn test_legal_actions_respects_essence() {
    let card_db = resource_test_db();
    let mut state = GameState::new();

    // Set up: 3 AP but only 1 essence
    state.players[0].action_points = 3;
    state.players[0].max_essence = 1;
    state.players[0].current_essence = 1;

    // Add a cost-5 card
    state.players[0].hand.push(CardInstance::new(CardId(3))); // Cost 5

    let actions = legal_actions(&state, &card_db);

    // Should only have EndTurn (card is too expensive)
    assert_eq!(actions.len(), 1);
    assert!(actions.contains(&Action::EndTurn));
}

#[test]
fn test_legal_actions_respects_ap() {
    let card_db = resource_test_db();
    let mut state = GameState::new();

    // Set up: 0 AP but lots of essence
    state.players[0].action_points = 0;
    state.players[0].max_essence = 10;
    state.players[0].current_essence = 10;

    // Add a cost-1 card
    state.players[0].hand.push(CardInstance::new(CardId(1))); // Cost 1

    let actions = legal_actions(&state, &card_db);

    // Should only have EndTurn (no AP)
    assert_eq!(actions.len(), 1);
    assert!(actions.contains(&Action::EndTurn));
}

#[test]
fn test_legal_actions_with_mixed_costs() {
    let card_db = resource_test_db();
    let mut state = GameState::new();

    // Set up: 3 AP and 3 essence
    state.players[0].action_points = 3;
    state.players[0].max_essence = 3;
    state.players[0].current_essence = 3;

    // Add cards of different costs
    state.players[0].hand.push(CardInstance::new(CardId(1))); // Cost 1 - playable
    state.players[0].hand.push(CardInstance::new(CardId(2))); // Cost 2 - playable
    state.players[0].hand.push(CardInstance::new(CardId(3))); // Cost 5 - NOT playable

    let actions = legal_actions(&state, &card_db);

    // Cost-1 card: 5 slots
    // Cost-2 card: 5 slots
    // Cost-5 card: 0 (too expensive)
    // + EndTurn
    let play_count = actions.iter().filter(|a| matches!(a, Action::PlayCard { .. })).count();
    assert_eq!(play_count, 10); // 5 + 5 slots for playable cards
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[test]
fn test_full_turn_cycle_resources() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // Turn 1: P1 has 1 essence, 3 AP
    assert_eq!(engine.state.players[0].current_essence, 1);
    assert_eq!(engine.state.players[0].action_points, 3);

    // End P1 turn
    engine.apply_action(Action::EndTurn).unwrap();

    // P2's turn: 1 essence (same as P1; FPA compensation is via bonus cards), 3 AP
    assert_eq!(engine.state.players[1].current_essence, 1);
    assert_eq!(engine.state.players[1].action_points, 3);

    // End P2 turn
    engine.apply_action(Action::EndTurn).unwrap();

    // Turn 2: P1 has 2 essence, 3 AP
    assert_eq!(engine.state.players[0].current_essence, 2);
    assert_eq!(engine.state.players[0].action_points, 3);
}

#[test]
fn test_high_cost_cards_playable_late_game() {
    let card_db = resource_test_db();
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(simple_test_deck(), simple_test_deck(), 42);

    // Add a cost-5 card to P1's hand
    engine.state.players[0].hand.clear();
    engine.state.players[0].hand.push(CardInstance::new(CardId(3))); // Cost 5

    // Simulate turns until P1 has 5 essence
    // Each full round (P1 turn + P2 turn) gives both players +1 max essence
    for _ in 0..8 { // 4 full rounds = turn 5
        if engine.is_game_over() { break; }
        engine.apply_action(Action::EndTurn).unwrap();
    }

    // P1 should now have 5 essence
    assert!(engine.state.players[0].current_essence >= 5);

    // Now P1 should be able to play the cost-5 card
    let actions = engine.get_legal_actions();
    let can_play = actions.iter().any(|a| matches!(a, Action::PlayCard { hand_index: 0, .. }));
    assert!(can_play, "Cost-5 card should be playable with 5+ essence");
}

#[test]
fn test_essence_constants_match_design() {
    // Verify the constants match the design doc
    assert_eq!(player::MAX_ESSENCE, 10, "Max essence should be 10");
    assert_eq!(player::ESSENCE_PER_TURN, 1, "Essence growth should be +1 per turn");
    assert_eq!(player::AP_PER_TURN, 3, "AP per turn should be 3");
}

#[test]
fn test_multiple_games_consistent_resources() {
    let card_db = resource_test_db();

    // Run 5 games with different seeds, verify resource consistency
    for seed in [1, 42, 100, 999, 12345] {
        let mut engine = GameEngine::new(&card_db);
        engine.start_game(simple_test_deck(), simple_test_deck(), seed);

        // Turn 1: P1 should always have 1 essence, 3 AP
        assert_eq!(engine.state.players[0].current_essence, 1, "Seed {}: P1 essence should be 1", seed);
        assert_eq!(engine.state.players[0].action_points, 3, "Seed {}: P1 AP should be 3", seed);

        // End P1 turn
        engine.apply_action(Action::EndTurn).unwrap();

        // P2 should have 1 essence (same as P1; FPA compensation is via bonus cards), 3 AP
        assert_eq!(engine.state.players[1].current_essence, 1, "Seed {}: P2 essence should be 1", seed);
        assert_eq!(engine.state.players[1].action_points, 3, "Seed {}: P2 AP should be 3", seed);
    }
}
