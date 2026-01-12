//! Game interface tests for neural network integration.
//!
//! Tests for the AI/ML interface including tensors, action masks, rewards, and game forking.

mod common;

use cardgame::actions::Action;
use cardgame::engine::{GameEngine, GameEnvironment};
use cardgame::state::GameResult;
use cardgame::types::PlayerId;
use common::*;

#[test]
fn test_game_state_size() {
    use cardgame::state::GameState;
    // GameState should be under 2KB for efficient MCTS cloning
    let size = std::mem::size_of::<GameState>();
    assert!(
        size < 2048,
        "GameState is {} bytes, should be under 2048 for efficient MCTS",
        size
    );
}

#[test]
fn test_clone_performance() {
    // Cloning should be fast (no heap allocations with ArrayVec)
    let db = test_card_db();
    let engine = GameEngine::new(&db);

    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = engine.clone_state();
    }
    let elapsed = start.elapsed();

    // Should clone 10k states in under 100ms
    assert!(
        elapsed.as_millis() < 100,
        "Cloning 10k states took {}ms, should be under 100ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_get_state_tensor_returns_correct_size() {
    let db = test_card_db();
    let engine = GameEngine::new(&db);

    let tensor = engine.get_state_tensor();
    assert_eq!(tensor.len(), cardgame::tensor::STATE_TENSOR_SIZE);
}

#[test]
fn test_legal_action_mask_has_correct_indices() {
    let db = test_card_db();
    let mut engine = GameEngine::new(&db);
    engine.start_game(simple_deck(), simple_deck(), 12345);

    let mask = engine.get_legal_action_mask();

    // EndTurn (index 255) should always be legal
    assert_eq!(mask[255], 1.0);

    // All values should be 0.0 or 1.0
    for val in mask.iter() {
        assert!(*val == 0.0 || *val == 1.0);
    }

    // Count legal actions from mask
    let legal_count: usize = mask.iter().filter(|&&v| v == 1.0).count();

    // Should have at least EndTurn legal
    assert!(legal_count >= 1);

    // Compare with get_legal_actions
    let legal_actions = engine.get_legal_actions();
    assert_eq!(legal_count, legal_actions.len());
}

#[test]
fn test_apply_action_by_index_works() {
    let db = test_card_db();
    let mut engine = GameEngine::new(&db);
    engine.start_game(simple_deck(), simple_deck(), 12345);

    // EndTurn is index 255
    let result = engine.apply_action_by_index(255);
    assert!(result.is_ok());

    // Should be player 2's turn now
    assert_eq!(engine.current_player(), PlayerId::PLAYER_TWO);
}

#[test]
fn test_apply_action_by_index_invalid() {
    let db = test_card_db();
    let mut engine = GameEngine::new(&db);
    engine.start_game(simple_deck(), simple_deck(), 12345);

    // An attack action that's not legal (no creatures on board)
    // Index 50 is Attack(0, 0)
    let result = engine.apply_action_by_index(50);
    assert!(result.is_err());
}

#[test]
fn test_fork_creates_independent_game() {
    let db = test_card_db();
    let mut engine = GameEngine::new(&db);
    engine.start_game(simple_deck(), simple_deck(), 12345);

    // Fork the engine
    let mut forked = engine.fork();

    // End turn on forked engine
    forked.apply_action(Action::EndTurn).unwrap();

    // Forked engine should be on player 2
    assert_eq!(forked.current_player(), PlayerId::PLAYER_TWO);

    // Original engine should still be on player 1
    assert_eq!(engine.current_player(), PlayerId::PLAYER_ONE);
}

#[test]
fn test_reward_values_correct_for_win_loss() {
    let db = test_card_db();
    let mut engine = GameEngine::new(&db);
    engine.start_game(simple_deck(), simple_deck(), 12345);

    // Game ongoing - both players should get 0 reward
    assert_eq!(engine.get_reward(PlayerId::PLAYER_ONE), 0.0);
    assert_eq!(engine.get_reward(PlayerId::PLAYER_TWO), 0.0);

    // Set player 2's life to 0 to end the game
    engine.state.players[1].life = 0;
    engine.check_life_victory();

    // Player 1 wins, player 2 loses
    assert_eq!(engine.get_reward(PlayerId::PLAYER_ONE), 1.0);
    assert_eq!(engine.get_reward(PlayerId::PLAYER_TWO), -1.0);
}

#[test]
fn test_reward_for_draw() {
    let db = test_card_db();
    let mut engine = GameEngine::new(&db);
    engine.start_game(simple_deck(), simple_deck(), 12345);

    // Manually set game result to draw
    engine.state.result = Some(GameResult::Draw);

    // Both players should get 0 reward in a draw
    assert_eq!(engine.get_reward(PlayerId::PLAYER_ONE), 0.0);
    assert_eq!(engine.get_reward(PlayerId::PLAYER_TWO), 0.0);
}

#[test]
fn test_current_player_and_turn_number() {
    let db = test_card_db();
    let mut engine = GameEngine::new(&db);
    engine.start_game(simple_deck(), simple_deck(), 12345);

    assert_eq!(engine.current_player(), PlayerId::PLAYER_ONE);
    assert_eq!(engine.turn_number(), 1);

    engine.apply_action(Action::EndTurn).unwrap();

    assert_eq!(engine.current_player(), PlayerId::PLAYER_TWO);
    assert_eq!(engine.turn_number(), 2);
}

#[test]
fn test_is_game_over() {
    let db = test_card_db();
    let mut engine = GameEngine::new(&db);
    engine.start_game(simple_deck(), simple_deck(), 12345);

    assert!(!engine.is_game_over());

    // End the game
    engine.state.players[1].life = 0;
    engine.check_life_victory();

    assert!(engine.is_game_over());
}

#[test]
fn test_game_environment_trait() {
    let db = test_card_db();
    let mut engine = GameEngine::new(&db);
    engine.start_game(simple_deck(), simple_deck(), 12345);

    // get_state
    let state = engine.get_state();
    assert_eq!(state.current_turn, 1);

    // get_legal_actions (via trait)
    let actions = GameEnvironment::get_legal_actions(&engine);
    assert!(actions.contains(&Action::EndTurn));

    // is_terminal
    assert!(!GameEnvironment::is_terminal(&engine));

    // get_reward
    assert_eq!(GameEnvironment::get_reward(&engine, 0), 0.0);

    // clone_for_search
    let forked = engine.clone_for_search();
    assert_eq!(forked.turn_number(), engine.turn_number());

    // apply_action (via trait)
    GameEnvironment::apply_action(&mut engine, Action::EndTurn).unwrap();
    assert_eq!(engine.current_player(), PlayerId::PLAYER_TWO);
}

#[test]
fn test_legal_actions_consistent_with_mask() {
    let db = test_card_db();
    let mut engine = GameEngine::new(&db);
    engine.start_game(simple_deck(), simple_deck(), 12345);

    let actions = engine.get_legal_actions();
    let mask = engine.get_legal_action_mask();

    // Each legal action should have a 1.0 in the mask
    for action in &actions {
        let idx = action.to_index() as usize;
        assert_eq!(
            mask[idx], 1.0,
            "Action {:?} (index {}) should be marked legal in mask",
            action, idx
        );
    }

    // Each 1.0 in mask should correspond to an action in legal_actions
    for (idx, &val) in mask.iter().enumerate() {
        if val == 1.0 {
            if let Some(action) = Action::from_index(idx as u8) {
                assert!(
                    actions.contains(&action),
                    "Mask index {} marked legal but action {:?} not in legal_actions",
                    idx, action
                );
            }
        }
    }
}
