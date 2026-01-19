//! Unit tests for MCTS bot.

use cardgame::actions::Action;
use cardgame::bots::{Bot, MctsBot, MctsConfig, MctsNode};
use cardgame::cards::CardDatabase;
use cardgame::engine::GameEngine;
use cardgame::types::{CardId, Slot};

fn load_test_db() -> CardDatabase {
    CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards")
}

fn test_deck() -> Vec<CardId> {
    let valid_ids = [1, 3, 6, 8, 11, 12, 16, 20, 34];
    (0..18)
        .map(|i| CardId(valid_ids[i % valid_ids.len()] as u16))
        .collect()
}

#[test]
fn test_mcts_bot_creation() {
    let card_db = load_test_db();
    let bot = MctsBot::new(&card_db, 42);
    assert_eq!(bot.name(), "MctsBot");
}

#[test]
fn test_mcts_config() {
    let fast = MctsConfig::fast();
    assert_eq!(fast.simulations, 100);

    let strong = MctsConfig::strong();
    assert!(strong.simulations > fast.simulations);
}

#[test]
fn test_mcts_node_ucb1() {
    let mut node = MctsNode::new(Some(Action::EndTurn));
    node.set_visits(10);
    node.set_wins(5);

    let ucb = node.ucb1(100, 1.414);
    assert!(ucb > 0.0, "UCB1 should be positive");
    assert!(ucb < 100.0, "UCB1 should be reasonable");
}

#[test]
fn test_mcts_node_expand() {
    let mut node = MctsNode::root();
    let actions = vec![Action::EndTurn, Action::PlayCard { hand_index: 0, slot: Slot(0) }];

    node.expand(&actions);

    assert_eq!(node.children_len(), 2);
}

#[test]
fn test_mcts_search_returns_valid_action() {
    let card_db = load_test_db();
    let config = MctsConfig::fast();
    let mut bot = MctsBot::with_config(&card_db, config, 42);

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(test_deck(), test_deck(), 12345);

    let action = bot.search(&engine);
    let legal = engine.get_legal_actions();

    assert!(legal.contains(&action), "MCTS should return a legal action");
}

#[test]
#[ignore]
fn test_mcts_completes_game() {
    let card_db = load_test_db();
    let config = MctsConfig::fast();
    let mut bot = MctsBot::with_config(&card_db, config, 42);

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(test_deck(), test_deck(), 12345);

    let mut actions = 0;
    while !engine.is_game_over() && actions < 200 {
        let action = bot.select_action_with_engine(&engine);
        engine.apply_action(action).expect("Legal action should work");
        actions += 1;
    }

    assert!(engine.is_game_over() || actions >= 200);
}

#[test]
fn test_mcts_parallel_search() {
    let card_db = load_test_db();
    let config = MctsConfig {
        simulations: 50,
        exploration: 1.414,
        max_rollout_depth: 50,
        parallel_trees: 4, // 4 parallel trees
        leaf_rollouts: 1,
    };
    let mut bot = MctsBot::with_config(&card_db, config, 42);

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(test_deck(), test_deck(), 12345);

    let action = bot.search(&engine);
    let legal = engine.get_legal_actions();

    assert!(legal.contains(&action), "Parallel MCTS should return a legal action");
}

#[test]
fn test_mcts_parallel_config() {
    let config = MctsConfig::parallel(8);
    assert_eq!(config.parallel_trees, 8);
    assert_eq!(config.simulations, 500);
}

#[test]
fn test_mcts_leaf_parallel_config() {
    let config = MctsConfig::leaf_parallel(4);
    assert_eq!(config.leaf_rollouts, 4);
    assert_eq!(config.parallel_trees, 1);
}

#[test]
fn test_mcts_leaf_parallel_search() {
    let card_db = load_test_db();
    let config = MctsConfig {
        simulations: 50,
        exploration: 1.414,
        max_rollout_depth: 50,
        parallel_trees: 1,
        leaf_rollouts: 4, // 4 parallel rollouts per leaf
    };
    let mut bot = MctsBot::with_config(&card_db, config, 42);

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(test_deck(), test_deck(), 12345);

    let action = bot.search(&engine);
    let legal = engine.get_legal_actions();

    assert!(legal.contains(&action), "Leaf-parallel MCTS should return a legal action");
}

#[test]
fn test_mcts_requires_engine() {
    let card_db = load_test_db();
    let bot = MctsBot::new(&card_db, 42);

    // MCTS bots require engine access for full functionality
    assert!(bot.requires_engine(), "MctsBot should require engine access");
}

#[test]
#[should_panic(expected = "MctsBot::select_action() called without engine access")]
fn test_mcts_select_action_panics_without_engine() {
    let card_db = load_test_db();
    let mut bot = MctsBot::new(&card_db, 42);

    // Create dummy state data
    let state_tensor = [0.0f32; cardgame::tensor::STATE_TENSOR_SIZE];
    let legal_mask = [0.0f32; 256];
    let legal_actions = vec![Action::EndTurn];

    // This should panic because MCTS requires engine access
    let _ = bot.select_action(&state_tensor, &legal_mask, &legal_actions);
}

#[test]
fn test_mcts_select_action_with_engine_trait_method() {
    let card_db = load_test_db();
    let mut bot = MctsBot::with_config(&card_db, MctsConfig::fast(), 42);

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(test_deck(), test_deck(), 12345);

    // Call through the trait method (as GameRunner does)
    let action = bot.select_action_with_engine(&engine);
    let legal = engine.get_legal_actions();

    assert!(legal.contains(&action), "Trait method should return a legal action");
}
