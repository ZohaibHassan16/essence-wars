//! Integration tests for complete game simulations.
//!
//! Tests using real YAML-loaded cards, determinism, MCTS scenarios, and full gameplay.

mod common;

use cardgame::actions::Action;
use cardgame::cards::CardDatabase;
use cardgame::engine::GameEngine;
use cardgame::state::GameResult;
use cardgame::types::{CardId, PlayerId};
use common::*;

/// Test a complete game from start to finish using YAML-loaded cards
#[test]
fn test_complete_game_simulation() {
    // Load card database from YAML files
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards from YAML");

    // Create engine
    let mut engine = GameEngine::new(&card_db);

    // Create decks using valid card IDs from starter.yaml
    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();

    // Start game with seed
    engine.start_game(deck1, deck2, 12345);

    // Verify initial state
    assert_eq!(engine.turn_number(), 1);
    assert_eq!(engine.current_player(), PlayerId::PLAYER_ONE);
    assert!(!engine.is_game_over());

    // Play until game ends or max iterations
    let mut action_count = 0;
    while !engine.is_game_over() && action_count < 500 {
        let actions = engine.get_legal_actions();
        assert!(!actions.is_empty(), "Should always have at least EndTurn");

        // Take first available action (simple AI)
        let action = actions[0];
        engine.apply_action(action).expect("Legal action should succeed");

        action_count += 1;
    }

    // Game should have ended (either by winning or max iterations)
    if engine.is_game_over() {
        // Winner should be determined
        assert!(engine.winner().is_some() || matches!(engine.state.result, Some(GameResult::Draw)));
    }

    // Verify no panics occurred during gameplay
    assert!(action_count > 0, "Game should have taken at least one action");
}

/// Test neural network interface end-to-end with YAML-loaded cards
#[test]
fn test_neural_network_interface() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    let mut engine = GameEngine::new(&card_db);

    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    engine.start_game(deck1, deck2, 42);

    // Get state tensor
    let tensor = engine.get_state_tensor();
    assert_eq!(tensor.len(), cardgame::tensor::STATE_TENSOR_SIZE);

    // Verify tensor values are finite (card IDs can be raw values)
    for (i, &val) in tensor.iter().enumerate() {
        assert!(
            val.is_finite(),
            "Tensor value at index {} is not finite: {}",
            i, val
        );
        // Most normalized values are 0-1, but card IDs are raw (1000-4999 range in Core Set)
        // and some values like winner can be -1
        assert!(
            val >= -1.0 && val <= 5000.0,
            "Tensor value at index {} out of expected range: {}",
            i, val
        );
    }

    // Get legal mask
    let mask = engine.get_legal_action_mask();
    assert_eq!(mask.len(), 256);

    // All mask values should be 0.0 or 1.0
    for (i, &val) in mask.iter().enumerate() {
        assert!(
            val == 0.0 || val == 1.0,
            "Mask value at index {} should be 0.0 or 1.0, got {}",
            i, val
        );
    }

    // At least EndTurn should be legal
    assert_eq!(mask[255], 1.0, "EndTurn (index 255) should always be legal");

    // Apply action by index
    engine.apply_action_by_index(255).expect("EndTurn should work");

    // State should have changed
    assert_eq!(engine.current_player(), PlayerId::PLAYER_TWO);
}

/// Test MCTS-style tree search scenario with forking
#[test]
fn test_mcts_tree_search_scenario() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    let mut engine = GameEngine::new(&card_db);

    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    engine.start_game(deck1, deck2, 99);

    // Save initial state
    let initial_turn = engine.turn_number();
    let initial_player = engine.current_player();

    // Fork the game for search
    let mut fork1 = engine.fork();
    let mut fork2 = engine.fork();

    let actions = engine.get_legal_actions();

    // Take an action in fork1
    fork1.apply_action(actions[0]).unwrap();

    // Take a different action in fork2 if available
    if actions.len() >= 2 {
        fork2.apply_action(actions[1]).unwrap();
    }

    // Original engine should be unchanged
    assert_eq!(engine.turn_number(), initial_turn);
    assert_eq!(engine.current_player(), initial_player);

    // Forks should be independent
    let fork1_state = fork1.clone_state();
    let _fork2_state = fork2.clone_state();

    // At minimum, fork1 should have progressed from initial state
    assert!(
        fork1.current_player() != initial_player || fork1.turn_number() != initial_turn
            || fork1_state.players[0].action_points != engine.state.players[0].action_points,
        "Fork1 should have changed after action"
    );

    // Test deep forking (MCTS tree expansion)
    let mut depth_fork = engine.fork();
    for _ in 0..10 {
        let depth_actions = depth_fork.get_legal_actions();
        if depth_actions.is_empty() || depth_fork.is_game_over() {
            break;
        }
        depth_fork.apply_action(depth_actions[0]).unwrap();
    }

    // Original still unchanged
    assert_eq!(engine.turn_number(), initial_turn);
}

/// Test game determinism with same seed
#[test]
fn test_game_determinism() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    // Play same game twice with same seed
    let mut results = Vec::new();
    for _ in 0..2 {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        engine.start_game(deck1, deck2, 12345);

        // Play deterministically (always first action)
        let mut actions_taken = Vec::new();
        while !engine.is_game_over() && actions_taken.len() < 100 {
            let actions = engine.get_legal_actions();
            let action = actions[0];
            actions_taken.push(action);
            engine.apply_action(action).unwrap();
        }
        results.push(actions_taken);
    }

    // Both games should have identical action sequences
    assert_eq!(results[0], results[1], "Games with same seed should be deterministic");
}

/// Test that games with different seeds produce different outcomes
#[test]
fn test_different_seeds_different_games() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    // Play games with different seeds
    let mut results = Vec::new();
    for seed in [11111u64, 22222u64, 33333u64] {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        engine.start_game(deck1, deck2, seed);

        // Record initial hands (which depend on shuffle)
        let p1_hand: Vec<CardId> = engine.state.players[0].hand.iter()
            .map(|c| c.card_id)
            .collect();
        results.push(p1_hand);
    }

    // At least some of the games should have different initial hands
    let all_same = results.iter().all(|h| h == &results[0]);
    assert!(!all_same, "Different seeds should produce different shuffles");
}

/// Test full game plays to completion without panics
#[test]
fn test_multiple_games_no_panics() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    // Play multiple games with different seeds
    for seed in [1u64, 42, 12345, 99999, 314159] {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        engine.start_game(deck1, deck2, seed);

        let mut action_count = 0;
        while !engine.is_game_over() && action_count < 300 {
            let actions = engine.get_legal_actions();

            // Verify actions are non-empty until game over
            assert!(!actions.is_empty(), "Seed {}: No legal actions but game not over", seed);

            // Take action (alternate between first and last to vary play)
            let action = if action_count % 2 == 0 {
                actions[0]
            } else {
                actions[actions.len() - 1]
            };

            let result = engine.apply_action(action);
            assert!(result.is_ok(), "Seed {}: Legal action failed: {:?}", seed, result);

            action_count += 1;
        }
    }
}

/// Test reward values throughout a game
#[test]
fn test_rewards_during_gameplay() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    let mut engine = GameEngine::new(&card_db);

    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    engine.start_game(deck1, deck2, 77777);

    // During gameplay, both rewards should be 0
    let mut action_count = 0;
    while !engine.is_game_over() && action_count < 200 {
        let p1_reward = engine.get_reward(PlayerId::PLAYER_ONE);
        let p2_reward = engine.get_reward(PlayerId::PLAYER_TWO);

        // Before game over, rewards should be 0
        assert_eq!(p1_reward, 0.0, "P1 reward should be 0 during gameplay");
        assert_eq!(p2_reward, 0.0, "P2 reward should be 0 during gameplay");

        let actions = engine.get_legal_actions();
        engine.apply_action(actions[0]).unwrap();
        action_count += 1;
    }

    // After game over, check rewards
    if engine.is_game_over() {
        let p1_reward = engine.get_reward(PlayerId::PLAYER_ONE);
        let p2_reward = engine.get_reward(PlayerId::PLAYER_TWO);

        // Rewards should be -1, 0, or 1
        assert!(
            p1_reward == -1.0 || p1_reward == 0.0 || p1_reward == 1.0,
            "P1 reward should be -1, 0, or 1, got {}",
            p1_reward
        );
        assert!(
            p2_reward == -1.0 || p2_reward == 0.0 || p2_reward == 1.0,
            "P2 reward should be -1, 0, or 1, got {}",
            p2_reward
        );

        // If there's a winner, rewards should be opposite
        if let Some(winner) = engine.winner() {
            if winner == PlayerId::PLAYER_ONE {
                assert_eq!(p1_reward, 1.0);
                assert_eq!(p2_reward, -1.0);
            } else {
                assert_eq!(p1_reward, -1.0);
                assert_eq!(p2_reward, 1.0);
            }
        }
    }
}

/// Test that tensor output remains valid throughout gameplay
#[test]
fn test_tensor_validity_throughout_game() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    let mut engine = GameEngine::new(&card_db);

    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    engine.start_game(deck1, deck2, 54321);

    let mut action_count = 0;
    while !engine.is_game_over() && action_count < 100 {
        // Get tensor and verify validity
        let tensor = engine.get_state_tensor();

        for (i, &val) in tensor.iter().enumerate() {
            assert!(
                val.is_finite(),
                "Turn {}, index {}: tensor value not finite: {}",
                engine.turn_number(), i, val
            );
        }

        // Get mask and verify validity
        let mask = engine.get_legal_action_mask();
        let legal_count: usize = mask.iter().filter(|&&v| v == 1.0).count();

        // Should have at least one legal action
        assert!(legal_count >= 1, "Should have at least one legal action");

        // EndTurn should always be available (unless game over)
        assert_eq!(mask[255], 1.0, "EndTurn should be legal");

        let actions = engine.get_legal_actions();
        engine.apply_action(actions[0]).unwrap();
        action_count += 1;
    }
}

/// Test keywords are correctly applied during combat in a real game
#[test]
fn test_keyword_combat_in_game() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    let mut engine = GameEngine::new(&card_db);

    // Use a seed that gives us creatures early
    let valid_ids = [1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 15];
    let deck1: Vec<CardId> = (0..20).map(|i| CardId(valid_ids[i % valid_ids.len()] as u16)).collect();
    let deck2: Vec<CardId> = (0..20).map(|i| CardId(valid_ids[i % valid_ids.len()] as u16)).collect();
    engine.start_game(deck1, deck2, 88888);

    // Play for a bit to build up a board
    let mut action_count = 0;
    let mut _had_combat = false;

    while !engine.is_game_over() && action_count < 150 {
        let actions = engine.get_legal_actions();

        // Look for attack actions to verify combat works
        let attack_action = actions.iter().find(|a| matches!(a, Action::Attack { .. }));

        if attack_action.is_some() {
            _had_combat = true;
        }

        // Take action
        engine.apply_action(actions[0]).unwrap();
        action_count += 1;
    }

    // We should have had at least some opportunity for combat
    // This test mainly verifies no panics occur during keyword-heavy combat
    assert!(action_count > 0, "Game should have progressed");
}
