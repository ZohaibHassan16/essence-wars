//! Core engine functionality tests.
//!
//! Tests for game setup, turn flow, win conditions, and basic gameplay mechanics.

mod common;

use cardgame::actions::Action;
use cardgame::engine::{seeded_shuffle, GameEngine, AP_PER_TURN, MAX_TURNS};
use cardgame::state::{Creature, CreatureStatus, GameResult, WinReason};
use cardgame::keywords::Keywords;
use cardgame::types::{CardId, PlayerId, Slot};
use common::*;

#[test]
fn test_new_game_setup() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    let deck1 = simple_deck();
    let deck2 = simple_deck();

    engine.start_game(deck1, deck2, 12345);

    // Both players should start with 30 life
    assert_eq!(engine.state.players[0].life, 30);
    assert_eq!(engine.state.players[1].life, 30);

    // Player 1 should be active
    assert_eq!(engine.state.active_player, PlayerId::PLAYER_ONE);

    // Turn counter should be 1
    assert_eq!(engine.state.current_turn, 1);

    // Player 1 should have 3 AP (restored at turn start)
    assert_eq!(engine.state.players[0].action_points, AP_PER_TURN);

    // Player 2 should have 0 AP (not their turn yet)
    assert_eq!(engine.state.players[1].action_points, 0);

    // Each player should have 4 cards in hand (3 initial + 1 at turn start for P1)
    // P1: 3 initial + 1 turn start = 4
    // P2: 3 initial = 3
    assert_eq!(engine.state.players[0].hand.len(), 4);
    assert_eq!(engine.state.players[1].hand.len(), 3);

    // Each player should have remaining cards in deck
    // P1: 30 - 4 = 26, P2: 30 - 3 = 27
    assert_eq!(engine.state.players[0].deck.len(), 26);
    assert_eq!(engine.state.players[1].deck.len(), 27);

    // Game should not be terminal
    assert!(!engine.is_terminal());
    assert!(engine.winner().is_none());
}

#[test]
fn test_turn_start_ap_restored() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game(simple_deck(), simple_deck(), 12345);

    // Spend some AP
    engine.state.players[0].action_points = 0;

    // End turn (switches to P2)
    engine.apply_action(Action::EndTurn).unwrap();

    // P2 should now have 3 AP
    assert_eq!(engine.state.players[1].action_points, AP_PER_TURN);

    // P1's AP should still be 0 (not their turn)
    assert_eq!(engine.state.players[0].action_points, 0);
}

#[test]
fn test_turn_start_card_drawn() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game(simple_deck(), simple_deck(), 12345);

    // P1 has 4 cards, P2 has 3 cards after game start
    let p1_hand_before = engine.state.players[0].hand.len();
    let p2_hand_before = engine.state.players[1].hand.len();

    // End P1's turn
    engine.apply_action(Action::EndTurn).unwrap();

    // P2 should have drawn a card (turn start)
    assert_eq!(engine.state.players[1].hand.len(), p2_hand_before + 1);

    // P1's hand should be unchanged
    assert_eq!(engine.state.players[0].hand.len(), p1_hand_before);
}

#[test]
fn test_turn_start_creatures_can_attack() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game(simple_deck(), simple_deck(), 12345);

    // Add a creature to P1's board from previous turn
    let creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
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
        turn_played: 0, // Played on a previous turn
    };
    engine.state.players[0].creatures.push(creature);

    // Mark it as exhausted (attacked last turn)
    engine.state.players[0].creatures[0].status.set_exhausted(true);

    // End P1's turn, then P2's turn to get back to P1
    engine.apply_action(Action::EndTurn).unwrap();
    engine.apply_action(Action::EndTurn).unwrap();

    // Creature should no longer be exhausted
    assert!(!engine.state.players[0].creatures[0].status.is_exhausted());
}

#[test]
fn test_turn_end_player_switches() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game(simple_deck(), simple_deck(), 12345);

    assert_eq!(engine.state.active_player, PlayerId::PLAYER_ONE);

    engine.apply_action(Action::EndTurn).unwrap();
    assert_eq!(engine.state.active_player, PlayerId::PLAYER_TWO);

    engine.apply_action(Action::EndTurn).unwrap();
    assert_eq!(engine.state.active_player, PlayerId::PLAYER_ONE);

    // Turn counter should have advanced
    assert_eq!(engine.state.current_turn, 3);
}

#[test]
fn test_win_by_damage() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game(simple_deck(), simple_deck(), 12345);

    // Add a powerful creature to P1's board
    let creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
        card_id: CardId(3),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(2),
        attack: 30, // Enough to kill in one hit
        current_health: 5,
        max_health: 5,
        base_attack: 30,
        base_health: 5,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0, // Not summoning sick
    };
    engine.state.players[0].creatures.push(creature);

    // Attack P2's face (slot 2 can attack 1, 2, 3)
    engine
        .apply_action(Action::Attack {
            attacker: Slot(2),
            defender: Slot(2),
        })
        .unwrap();

    // P2 should have 0 or less life
    assert!(engine.state.players[1].life <= 0);

    // Game should be terminal
    assert!(engine.is_terminal());

    // P1 should be the winner
    assert_eq!(engine.winner(), Some(PlayerId::PLAYER_ONE));

    // Win reason should be life reached zero
    if let Some(GameResult::Win { reason, .. }) = &engine.state.result {
        assert_eq!(*reason, WinReason::LifeReachedZero);
    } else {
        panic!("Expected win result");
    }
}

#[test]
fn test_win_by_turn_limit_higher_life() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game(simple_deck(), simple_deck(), 12345);

    // Set P1 life higher
    engine.state.players[0].life = 25;
    engine.state.players[1].life = 15;

    // Set turn to just before limit
    engine.state.current_turn = MAX_TURNS;

    // End turn to trigger turn limit
    engine.apply_action(Action::EndTurn).unwrap();

    // Game should be terminal
    assert!(engine.is_terminal());

    // P1 should win (higher life)
    assert_eq!(engine.winner(), Some(PlayerId::PLAYER_ONE));

    if let Some(GameResult::Win { reason, .. }) = &engine.state.result {
        assert_eq!(*reason, WinReason::TurnLimitHigherLife);
    }
}

#[test]
fn test_win_by_turn_limit_tie_p1_wins() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game(simple_deck(), simple_deck(), 12345);

    // Equal life
    engine.state.players[0].life = 20;
    engine.state.players[1].life = 20;

    // Set turn to just before limit
    engine.state.current_turn = MAX_TURNS;

    // End turn to trigger turn limit
    engine.apply_action(Action::EndTurn).unwrap();

    // Game should be terminal
    assert!(engine.is_terminal());

    // P1 should win on tie
    assert_eq!(engine.winner(), Some(PlayerId::PLAYER_ONE));
}

#[test]
fn test_seeded_shuffle_deterministic() {
    let mut items1 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut items2 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    seeded_shuffle(&mut items1, 42);
    seeded_shuffle(&mut items2, 42);

    // Same seed should produce same result
    assert_eq!(items1, items2);

    // Different seed should (almost certainly) produce different result
    let mut items3 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    seeded_shuffle(&mut items3, 12345);
    assert_ne!(items1, items3);
}

#[test]
fn test_seeded_shuffle_actually_shuffles() {
    let original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut shuffled = original.clone();

    seeded_shuffle(&mut shuffled, 999);

    // Should not be in original order (very unlikely to be unchanged)
    assert_ne!(original, shuffled);

    // Should contain all the same elements
    let mut sorted = shuffled.clone();
    sorted.sort();
    assert_eq!(original, sorted);
}

#[test]
fn test_play_creature_card() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game(simple_deck(), simple_deck(), 12345);

    // Find a creature card in hand
    let creature_card_idx = engine.state.players[0]
        .hand
        .iter()
        .position(|c| c.card_id.0 == 1 || c.card_id.0 == 2 || c.card_id.0 == 3);

    if let Some(idx) = creature_card_idx {
        let initial_ap = engine.state.players[0].action_points;
        let card_id = engine.state.players[0].hand[idx].card_id;
        let card_cost = card_db.get(card_id).unwrap().cost;

        engine
            .apply_action(Action::PlayCard {
                hand_index: idx as u8,
                slot: Slot(0),
            })
            .unwrap();

        // Creature should be on board
        assert_eq!(engine.state.players[0].creatures.len(), 1);
        assert_eq!(engine.state.players[0].creatures[0].slot, Slot(0));

        // AP should be reduced
        assert_eq!(
            engine.state.players[0].action_points,
            initial_ap - card_cost
        );

        // Card should be removed from hand
        assert!(!engine.state.players[0]
            .hand
            .iter()
            .any(|c| c.card_id == card_id)
            || engine.state.players[0]
                .hand
                .iter()
                .filter(|c| c.card_id == card_id)
                .count()
                < engine.state.players[0]
                    .hand
                    .iter()
                    .filter(|c| c.card_id == card_id)
                    .count()
                    + 1);
    }
}

#[test]
fn test_creature_combat() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game(simple_deck(), simple_deck(), 12345);

    // Add creatures to both sides
    let p1_creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(2),
        attack: 3,
        current_health: 4,
        max_health: 4,
        base_attack: 3,
        base_health: 4,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
    };
    engine.state.players[0].creatures.push(p1_creature);

    let p2_creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_TWO,
        slot: Slot(2),
        attack: 2,
        current_health: 3,
        max_health: 3,
        base_attack: 2,
        base_health: 3,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
    };
    engine.state.players[1].creatures.push(p2_creature);

    // P1 attacks P2's creature
    engine
        .apply_action(Action::Attack {
            attacker: Slot(2),
            defender: Slot(2),
        })
        .unwrap();

    // P2's creature should be dead (3 health - 3 damage = 0)
    assert_eq!(engine.state.players[1].creatures.len(), 0);

    // P1's creature should be damaged but alive (4 health - 2 damage = 2)
    assert_eq!(engine.state.players[0].creatures.len(), 1);
    assert_eq!(engine.state.players[0].creatures[0].current_health, 2);

    // P1's creature should be exhausted
    assert!(engine.state.players[0].creatures[0].status.is_exhausted());
}

#[test]
fn test_direct_face_attack() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game(simple_deck(), simple_deck(), 12345);

    let initial_life = engine.state.players[1].life;

    // Add creature to P1's board
    let creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(2),
        attack: 5,
        current_health: 5,
        max_health: 5,
        base_attack: 5,
        base_health: 5,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
    };
    engine.state.players[0].creatures.push(creature);

    // Attack empty slot (face damage)
    engine
        .apply_action(Action::Attack {
            attacker: Slot(2),
            defender: Slot(1),
        })
        .unwrap();

    // P2's life should be reduced
    assert_eq!(engine.state.players[1].life, initial_life - 5);
}

#[test]
fn test_illegal_action_rejected() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game(simple_deck(), simple_deck(), 12345);

    // Try to attack with non-existent creature
    let result = engine.apply_action(Action::Attack {
        attacker: Slot(0),
        defender: Slot(0),
    });

    assert!(result.is_err());
}

#[test]
fn test_game_over_no_more_actions() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game(simple_deck(), simple_deck(), 12345);

    // Set P2's life to 0 to end the game
    engine.state.players[1].life = 0;
    engine.check_life_victory();

    assert!(engine.is_terminal());

    // Any action should be rejected
    let result = engine.apply_action(Action::EndTurn);
    assert!(result.is_err());
}
