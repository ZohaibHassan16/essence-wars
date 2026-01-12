//! Full game simulation tests with random action selection.
//!
//! These tests verify:
//! - Games with random actions complete without panics
//! - Games terminate within the turn limit (30 turns)
//! - No invalid states occur during random play

mod common;

use cardgame::actions::Action;
use cardgame::cards::CardDatabase;
use cardgame::engine::GameEngine;
use cardgame::types::PlayerId;
use common::*;

/// Simple linear congruential generator for deterministic "random" selection
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        // LCG parameters (same as glibc)
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state
    }

    fn range(&mut self, max: usize) -> usize {
        (self.next() as usize) % max
    }
}

/// Test that games with truly random action selection complete without panics
#[test]
fn test_random_action_games() {
    let card_db = CardDatabase::load_from_directory("data/cards/sets")
        .expect("Failed to load cards");

    // Run 20 games with different seeds
    for game_seed in 0u64..20 {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        engine.start_game(deck1.clone(), deck2.clone(), game_seed * 1000);

        let mut rng = SimpleRng::new(game_seed);
        let mut action_count = 0;
        let max_actions = 500; // Safety limit

        while !engine.is_game_over() && action_count < max_actions {
            let actions = engine.get_legal_actions();

            // Verify we have legal actions
            assert!(
                !actions.is_empty(),
                "Game {}: No legal actions but game not over at turn {}",
                game_seed, engine.turn_number()
            );

            // Select random action
            let action_idx = rng.range(actions.len());
            let action = actions[action_idx];

            // Apply action
            let result = engine.apply_action(action);
            assert!(
                result.is_ok(),
                "Game {}: Legal action {:?} failed: {:?}",
                game_seed, action, result
            );

            action_count += 1;
        }

        // Verify game terminated properly
        assert!(
            engine.is_game_over(),
            "Game {} did not terminate within {} actions",
            game_seed, max_actions
        );
    }
}

/// Test that games respect the 30 turn limit
#[test]
fn test_turn_limit_enforcement() {
    let card_db = CardDatabase::load_from_directory("data/cards/sets")
        .expect("Failed to load cards");

    // Use a seed that tends to produce longer games (mostly EndTurn actions)
    let mut engine = GameEngine::new(&card_db);
    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    engine.start_game(deck1, deck2, 99999);

    // Always end turn immediately to maximize turn count
    while !engine.is_game_over() {
        let actions = engine.get_legal_actions();

        // Find EndTurn action
        let end_turn = actions.iter()
            .find(|a| matches!(a, Action::EndTurn))
            .expect("EndTurn should always be available");

        engine.apply_action(*end_turn).unwrap();
    }

    // Turn limit is 30 (each player gets 15 turns)
    // Game should end at or before turn 30
    assert!(
        engine.turn_number() <= 31, // 31 because turn increments after last turn
        "Game exceeded turn limit: turn {}",
        engine.turn_number()
    );
}

/// Test state validity is maintained throughout random play
#[test]
fn test_state_validity_during_random_play() {
    let card_db = CardDatabase::load_from_directory("data/cards/sets")
        .expect("Failed to load cards");

    for game_seed in [42u64, 12345, 99999, 7777, 31415] {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        engine.start_game(deck1, deck2, game_seed);

        let mut rng = SimpleRng::new(game_seed);
        let mut action_count = 0;

        while !engine.is_game_over() && action_count < 300 {
            // Verify state invariants before each action
            verify_state_invariants(&engine, game_seed);

            let actions = engine.get_legal_actions();
            let action_idx = rng.range(actions.len());
            engine.apply_action(actions[action_idx]).unwrap();
            action_count += 1;
        }

        // Final state check
        if engine.is_game_over() {
            assert!(
                engine.state.result.is_some(),
                "Game {}: Game over but no result set",
                game_seed
            );
        }
    }
}

/// Helper to verify state invariants
fn verify_state_invariants(engine: &GameEngine, game_seed: u64) {
    let state = &engine.state;

    // Turn number should be positive
    assert!(
        state.current_turn >= 1,
        "Game {}: Invalid turn number {}",
        game_seed, state.current_turn
    );

    // Active player should be valid
    assert!(
        state.active_player == PlayerId::PLAYER_ONE || state.active_player == PlayerId::PLAYER_TWO,
        "Game {}: Invalid active player",
        game_seed
    );

    // Check player states
    for (i, player) in state.players.iter().enumerate() {
        // Life should be reasonable
        assert!(
            player.life <= 30,
            "Game {}: Player {} life {} exceeds max",
            game_seed, i, player.life
        );

        // Essence should not exceed max
        assert!(
            player.current_essence <= player.max_essence,
            "Game {}: Player {} essence {} > max {}",
            game_seed, i, player.current_essence, player.max_essence
        );

        // Max essence should not exceed 10
        assert!(
            player.max_essence <= 10,
            "Game {}: Player {} max essence {} exceeds limit",
            game_seed, i, player.max_essence
        );

        // Action points should not exceed 3 normally
        assert!(
            player.action_points <= 5, // Allow some buffer for effects
            "Game {}: Player {} AP {} seems too high",
            game_seed, i, player.action_points
        );

        // Creature count should not exceed slots
        assert!(
            player.creatures.len() <= 5,
            "Game {}: Player {} has {} creatures (max 5)",
            game_seed, i, player.creatures.len()
        );

        // Support count should not exceed slots
        assert!(
            player.supports.len() <= 2,
            "Game {}: Player {} has {} supports (max 2)",
            game_seed, i, player.supports.len()
        );

        // Hand size should not exceed max
        assert!(
            player.hand.len() <= 10,
            "Game {}: Player {} hand size {} exceeds max",
            game_seed, i, player.hand.len()
        );
    }
}

/// Test legal action mask consistency with legal actions list
#[test]
fn test_mask_consistency_during_random_play() {
    let card_db = CardDatabase::load_from_directory("data/cards/sets")
        .expect("Failed to load cards");

    let mut engine = GameEngine::new(&card_db);
    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    engine.start_game(deck1, deck2, 55555);

    let mut rng = SimpleRng::new(55555);
    let mut action_count = 0;

    while !engine.is_game_over() && action_count < 200 {
        let actions = engine.get_legal_actions();
        let mask = engine.get_legal_action_mask();

        // Every action in the list should have mask = 1.0
        for action in &actions {
            let idx = action.to_index() as usize;
            assert_eq!(
                mask[idx], 1.0,
                "Action {:?} at index {} not marked legal in mask",
                action, idx
            );
        }

        // Count of 1.0s in mask should equal action count
        let mask_count: usize = mask.iter().filter(|&&v| v == 1.0).count();
        assert_eq!(
            mask_count, actions.len(),
            "Mask legal count {} != actions list length {}",
            mask_count, actions.len()
        );

        let action_idx = rng.range(actions.len());
        engine.apply_action(actions[action_idx]).unwrap();
        action_count += 1;
    }
}

/// Test victory points win condition
#[test]
fn test_victory_points_tracking() {
    let card_db = CardDatabase::load_from_directory("data/cards/sets")
        .expect("Failed to load cards");

    let mut engine = GameEngine::new(&card_db);
    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    engine.start_game(deck1, deck2, 11111);

    let mut rng = SimpleRng::new(11111);

    while !engine.is_game_over() {
        // Track damage dealt (victory points)
        let p1_vp = engine.state.players[0].total_damage_dealt;
        let p2_vp = engine.state.players[1].total_damage_dealt;

        // Victory points should be non-negative
        assert!(p1_vp <= 50 || engine.is_game_over(), "P1 VP tracking error");
        assert!(p2_vp <= 50 || engine.is_game_over(), "P2 VP tracking error");

        let actions = engine.get_legal_actions();
        if actions.is_empty() {
            break;
        }
        let action_idx = rng.range(actions.len());
        engine.apply_action(actions[action_idx]).unwrap();
    }
}

/// Stress test with many rapid games
#[test]
fn test_rapid_game_stress() {
    let card_db = CardDatabase::load_from_directory("data/cards/sets")
        .expect("Failed to load cards");

    let mut games_completed = 0;
    let mut total_actions = 0;

    for seed in 0u64..50 {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        engine.start_game(deck1, deck2, seed);

        let mut rng = SimpleRng::new(seed);
        let mut action_count = 0;

        while !engine.is_game_over() && action_count < 200 {
            let actions = engine.get_legal_actions();
            let action_idx = rng.range(actions.len());
            engine.apply_action(actions[action_idx]).unwrap();
            action_count += 1;
        }

        if engine.is_game_over() {
            games_completed += 1;
        }
        total_actions += action_count;
    }

    // Most games should complete
    assert!(
        games_completed >= 45,
        "Only {} of 50 games completed (expected at least 45)",
        games_completed
    );

    // Average actions per game should be reasonable
    let avg_actions = total_actions / 50;
    assert!(
        avg_actions > 20 && avg_actions < 200,
        "Unusual average actions per game: {}",
        avg_actions
    );
}
