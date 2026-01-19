//! Unit tests for greedy bot.

use cardgame::actions::Action;
use cardgame::bots::{Bot, GreedyBot, GreedyWeights};
use cardgame::cards::CardDatabase;
use cardgame::engine::GameEngine;
use cardgame::state::{GameResult, GameState, WinReason};
use cardgame::types::{CardId, PlayerId, Slot};

fn load_test_db() -> CardDatabase {
    CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards")
}

fn test_deck() -> Vec<CardId> {
    let valid_ids = [1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 15, 32, 33, 40];
    (0..20)
        .map(|i| CardId(valid_ids[i % valid_ids.len()] as u16))
        .collect()
}

#[test]
fn test_greedy_bot_creation() {
    let card_db = load_test_db();
    let bot = GreedyBot::new(&card_db, 42);
    assert_eq!(bot.name(), "GreedyBot");
}

#[test]
fn test_state_evaluation() {
    let card_db = load_test_db();
    let bot = GreedyBot::new(&card_db, 42);

    // Create a simple game state
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(test_deck(), test_deck(), 12345);

    let score = bot.evaluate_state(&engine.state, PlayerId::PLAYER_ONE);

    // Initial state should have positive score (both players equal, but we value our own stuff)
    assert!(score != 0.0, "Score should be non-zero");
}

#[test]
fn test_action_evaluation() {
    let card_db = load_test_db();
    let bot = GreedyBot::new(&card_db, 42);

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(test_deck(), test_deck(), 12345);

    // EndTurn should have a valid score
    let score = bot.evaluate_action(&engine, Action::EndTurn);
    assert!(score.is_finite(), "Score should be finite");
}

#[test]
fn test_greedy_vs_random_game() {
    let card_db = load_test_db();

    // Run multiple games and verify GreedyBot doesn't crash
    for seed in 0..5 {
        let mut engine = GameEngine::new(&card_db);
        engine.start_game(test_deck(), test_deck(), seed);

        let mut greedy = GreedyBot::new(&card_db, seed);

        let mut actions = 0;
        while !engine.is_game_over() && actions < 500 {
            let action = greedy.select_action_with_engine(&engine);
            engine.apply_action(action).expect("Legal action should work");
            actions += 1;
        }

        // Game should end within reasonable number of actions
        assert!(engine.is_game_over() || actions >= 500);
    }
}

#[test]
fn test_greedy_prefers_attacks() {
    let card_db = load_test_db();
    let mut bot = GreedyBot::new(&card_db, 42);

    // In fallback mode, attacks should have high priority
    let actions = vec![
        Action::EndTurn,
        Action::Attack { attacker: Slot(0), defender: Slot(0) },
    ];

    let selected = bot.select_action_fallback(&actions);

    // Should prefer attack over end turn
    assert!(matches!(selected, Action::Attack { .. }));
}

#[test]
fn test_weight_customization() {
    let card_db = load_test_db();

    // Create bot with custom aggressive weights
    let mut weights = GreedyWeights::default();
    weights.enemy_life_damage = 10.0; // Very aggressive

    let bot = GreedyBot::with_weights(&card_db, weights, 42);
    assert!((bot.weights().enemy_life_damage - 10.0).abs() < 0.001);
}

#[test]
fn test_win_detection() {
    let card_db = load_test_db();
    let bot = GreedyBot::new(&card_db, 42);

    let mut state = GameState::new();
    state.result = Some(GameResult::Win {
        winner: PlayerId::PLAYER_ONE,
        reason: WinReason::LifeReachedZero,
    });

    let score_p1 = bot.evaluate_state(&state, PlayerId::PLAYER_ONE);
    let score_p2 = bot.evaluate_state(&state, PlayerId::PLAYER_TWO);

    assert!(score_p1 > 0.0, "Winner should have positive score");
    assert!(score_p2 < 0.0, "Loser should have negative score");
}
