//! Determinism verification tests for the game engine.
//!
//! These tests ensure that:
//! - Same seed always produces identical game outcomes
//! - Fork produces isolated independent states
//! - Tensor output is consistent across calls
//! - Different seeds produce different outcomes
//!
//! Determinism is critical for:
//! - Reproducible AI training
//! - Debugging game issues
//! - MCTS tree search correctness

mod common;

use cardgame::bots::{Bot, GreedyBot};
use cardgame::cards::CardDatabase;
use cardgame::engine::GameEngine;
use cardgame::types::PlayerId;
use common::*;

/// Simple LCG for deterministic action selection
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state
    }

    fn range(&mut self, max: usize) -> usize {
        (self.next() as usize) % max
    }
}

/// Captures game outcome for comparison
#[derive(Debug, Clone, PartialEq)]
struct GameOutcome {
    winner: Option<PlayerId>,
    final_turn: u16,
    p1_life: i16,
    p2_life: i16,
    action_count: usize,
    action_indices: Vec<usize>,
}

/// Run a game with random actions and return the outcome
fn run_game_with_seed(card_db: &CardDatabase, seed: u64) -> GameOutcome {
    let mut engine = GameEngine::new(card_db);
    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    engine.start_game(deck1, deck2, seed);

    let mut rng = SimpleRng::new(seed);
    let mut action_indices = Vec::new();
    let mut action_count = 0;

    while !engine.is_game_over() && action_count < 500 {
        let actions = engine.get_legal_actions();
        if actions.is_empty() {
            break;
        }
        let idx = rng.range(actions.len());
        action_indices.push(idx);
        engine.apply_action(actions[idx]).unwrap();
        action_count += 1;
    }

    let winner = engine.state.result.as_ref().map(|r| match r {
        cardgame::state::GameResult::Win { winner, .. } => *winner,
        cardgame::state::GameResult::Draw { .. } => PlayerId::PLAYER_ONE, // Placeholder for draw
    });

    GameOutcome {
        winner,
        final_turn: engine.state.current_turn,
        p1_life: engine.state.players[0].life,
        p2_life: engine.state.players[1].life,
        action_count,
        action_indices,
    }
}

// ============================================================================
// Seed Reproducibility Tests
// ============================================================================

/// Test that same seed produces identical outcomes
#[test]
fn test_identical_seed_identical_outcome() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    for seed in [0u64, 42, 12345, 99999, u64::MAX / 2] {
        let outcome1 = run_game_with_seed(&card_db, seed);
        let outcome2 = run_game_with_seed(&card_db, seed);

        assert_eq!(
            outcome1.winner, outcome2.winner,
            "Seed {}: Winner mismatch",
            seed
        );
        assert_eq!(
            outcome1.final_turn, outcome2.final_turn,
            "Seed {}: Turn count mismatch",
            seed
        );
        assert_eq!(
            outcome1.p1_life, outcome2.p1_life,
            "Seed {}: P1 life mismatch",
            seed
        );
        assert_eq!(
            outcome1.p2_life, outcome2.p2_life,
            "Seed {}: P2 life mismatch",
            seed
        );
        assert_eq!(
            outcome1.action_count, outcome2.action_count,
            "Seed {}: Action count mismatch",
            seed
        );
        assert_eq!(
            outcome1.action_indices, outcome2.action_indices,
            "Seed {}: Action sequence mismatch",
            seed
        );
    }
}

/// Test multiple runs of the same seed
#[test]
fn test_repeated_identical_outcomes() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    let seed = 77777u64;
    let baseline = run_game_with_seed(&card_db, seed);

    for i in 0..10 {
        let outcome = run_game_with_seed(&card_db, seed);
        assert_eq!(
            baseline, outcome,
            "Run {}: Outcome differs from baseline",
            i
        );
    }
}

/// Test that different seeds produce different initial states
#[test]
fn test_different_seeds_different_initial_states() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    let deck = valid_yaml_deck();

    // Collect initial hand contents (first 4 cards drawn) for different seeds
    let initial_hands: Vec<Vec<u16>> = (0u64..20)
        .map(|seed| {
            let mut engine = GameEngine::new(&card_db);
            engine.start_game(deck.clone(), deck.clone(), seed);
            // Get card IDs from hand
            engine.state.players[0]
                .hand
                .iter()
                .map(|c| c.card_id.0)
                .collect()
        })
        .collect();

    // Count unique hand contents
    let unique_hands: std::collections::HashSet<Vec<u16>> = initial_hands.into_iter().collect();

    // With 20 different seeds, we should get several unique starting hands
    assert!(
        unique_hands.len() >= 5,
        "Only {} unique starting hands from 20 seeds (expected >= 5)",
        unique_hands.len()
    );
}

// ============================================================================
// Fork State Isolation Tests
// ============================================================================

/// Test that fork creates an isolated copy
#[test]
fn test_fork_isolation() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    let mut engine = GameEngine::new(&card_db);
    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    engine.start_game(deck1, deck2, 12345);

    let mut rng = SimpleRng::new(12345);

    // Play some actions
    for _ in 0..10 {
        if engine.is_game_over() {
            break;
        }
        let actions = engine.get_legal_actions();
        let idx = rng.range(actions.len());
        engine.apply_action(actions[idx]).unwrap();
    }

    // Snapshot original state
    let original_turn = engine.state.current_turn;
    let original_p1_life = engine.state.players[0].life;
    let original_p2_life = engine.state.players[1].life;

    // Fork
    let mut fork = engine.fork();

    // Play different actions in fork
    while !fork.is_game_over() {
        let actions = fork.get_legal_actions();
        // Always pick first action (different from random)
        fork.apply_action(actions[0]).unwrap();
        // Stop after a few actions
        if fork.state.current_turn > original_turn + 5 {
            break;
        }
    }

    // Original should be unchanged
    assert_eq!(
        engine.state.current_turn, original_turn,
        "Original turn changed after fork modification"
    );
    assert_eq!(
        engine.state.players[0].life, original_p1_life,
        "Original P1 life changed after fork modification"
    );
    assert_eq!(
        engine.state.players[1].life, original_p2_life,
        "Original P2 life changed after fork modification"
    );
}

/// Test that fork and original produce identical results with same actions
#[test]
fn test_fork_identical_with_same_actions() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    for seed in [42u64, 12345, 99999] {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        engine.start_game(deck1, deck2, seed);

        let mut rng = SimpleRng::new(seed);

        // Play some actions
        for _ in 0..10 {
            if engine.is_game_over() {
                break;
            }
            let actions = engine.get_legal_actions();
            let idx = rng.range(actions.len());
            engine.apply_action(actions[idx]).unwrap();
        }

        // Fork
        let mut fork = engine.fork();

        // Play same actions in both
        let mut rng2 = SimpleRng::new(seed + 1000);
        for _ in 0..10 {
            if engine.is_game_over() || fork.is_game_over() {
                break;
            }

            let actions = engine.get_legal_actions();
            let fork_actions = fork.get_legal_actions();

            assert_eq!(
                actions.len(),
                fork_actions.len(),
                "Seed {}: Legal action count differs after fork",
                seed
            );

            let idx = rng2.range(actions.len());
            engine.apply_action(actions[idx]).unwrap();
            fork.apply_action(fork_actions[idx]).unwrap();

            // States should be identical
            assert_eq!(
                engine.state.current_turn, fork.state.current_turn,
                "Seed {}: Turn mismatch after same action",
                seed
            );
            assert_eq!(
                engine.state.players[0].life, fork.state.players[0].life,
                "Seed {}: P1 life mismatch after same action",
                seed
            );
        }
    }
}

// ============================================================================
// Tensor Consistency Tests
// ============================================================================

/// Test that tensor output is consistent across calls
#[test]
fn test_tensor_determinism() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    let mut engine = GameEngine::new(&card_db);
    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    engine.start_game(deck1, deck2, 55555);

    let mut rng = SimpleRng::new(55555);

    // Play some actions and check tensor consistency
    for _ in 0..20 {
        if engine.is_game_over() {
            break;
        }

        // Get tensor twice
        let tensor1 = engine.get_state_tensor();
        let tensor2 = engine.get_state_tensor();

        // Should be identical
        assert_eq!(
            tensor1, tensor2,
            "Tensor changed between calls without state change"
        );

        // Get mask twice
        let mask1 = engine.get_legal_action_mask();
        let mask2 = engine.get_legal_action_mask();

        assert_eq!(
            mask1, mask2,
            "Legal mask changed between calls without state change"
        );

        let actions = engine.get_legal_actions();
        let idx = rng.range(actions.len());
        engine.apply_action(actions[idx]).unwrap();
    }
}

/// Test that tensor accurately reflects state changes
#[test]
fn test_tensor_reflects_state_changes() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    let mut engine = GameEngine::new(&card_db);
    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    engine.start_game(deck1, deck2, 33333);

    let tensor_before = engine.get_state_tensor();

    // End turn (changes active player, draws card, etc.)
    engine.apply_action(cardgame::actions::Action::EndTurn).unwrap();

    let tensor_after = engine.get_state_tensor();

    // Tensor should be different
    assert_ne!(
        tensor_before, tensor_after,
        "Tensor unchanged after EndTurn"
    );
}

// ============================================================================
// GreedyBot Determinism Tests
// ============================================================================

/// Test that GreedyBot produces identical decisions with same state
#[test]
fn test_greedy_bot_determinism() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    for seed in [42u64, 12345, 99999] {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        engine.start_game(deck1, deck2, seed);

        // Create two identical bots
        let mut bot1 = GreedyBot::new(&card_db, seed);
        let mut bot2 = GreedyBot::new(&card_db, seed);

        // Both should select same action from same state
        for _ in 0..20 {
            if engine.is_game_over() {
                break;
            }

            let action1 = bot1.select_action_with_engine(&engine);
            let action2 = bot2.select_action_with_engine(&engine);

            assert_eq!(
                action1, action2,
                "Seed {}: GreedyBot produced different actions from same state",
                seed
            );

            engine.apply_action(action1).unwrap();
        }
    }
}

/// Test GreedyBot vs GreedyBot game determinism
#[test]
fn test_greedy_vs_greedy_determinism() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    for seed in [100u64, 200, 300] {
        // Run game twice with same seed
        let outcome1 = run_greedy_game(&card_db, seed);
        let outcome2 = run_greedy_game(&card_db, seed);

        assert_eq!(
            outcome1.winner, outcome2.winner,
            "Seed {}: GreedyBot game winner mismatch",
            seed
        );
        assert_eq!(
            outcome1.final_turn, outcome2.final_turn,
            "Seed {}: GreedyBot game turn mismatch",
            seed
        );
        assert_eq!(
            outcome1.action_count, outcome2.action_count,
            "Seed {}: GreedyBot game action count mismatch",
            seed
        );
    }
}

/// Run a GreedyBot vs GreedyBot game
fn run_greedy_game(card_db: &CardDatabase, seed: u64) -> GameOutcome {
    let mut engine = GameEngine::new(card_db);
    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    engine.start_game(deck1, deck2, seed);

    let mut bot1 = GreedyBot::new(card_db, seed);
    let mut bot2 = GreedyBot::new(card_db, seed + 1000);

    let mut action_count = 0;
    let mut action_indices = Vec::new();

    while !engine.is_game_over() && action_count < 500 {
        let action = if engine.state.active_player == PlayerId::PLAYER_ONE {
            bot1.select_action_with_engine(&engine)
        } else {
            bot2.select_action_with_engine(&engine)
        };

        // Record action index for comparison
        let actions = engine.get_legal_actions();
        let idx = actions.iter().position(|a| *a == action).unwrap_or(0);
        action_indices.push(idx);

        engine.apply_action(action).unwrap();
        action_count += 1;
    }

    let winner = engine.state.result.as_ref().map(|r| match r {
        cardgame::state::GameResult::Win { winner, .. } => *winner,
        cardgame::state::GameResult::Draw { .. } => PlayerId::PLAYER_ONE,
    });

    GameOutcome {
        winner,
        final_turn: engine.state.current_turn,
        p1_life: engine.state.players[0].life,
        p2_life: engine.state.players[1].life,
        action_count,
        action_indices,
    }
}

// ============================================================================
// Deck Shuffle Determinism Tests
// ============================================================================

/// Test that deck shuffling is deterministic
#[test]
fn test_deck_shuffle_determinism() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    for seed in [42u64, 12345, 99999] {
        let mut engine1 = GameEngine::new(&card_db);
        let mut engine2 = GameEngine::new(&card_db);

        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();

        engine1.start_game(deck1.clone(), deck2.clone(), seed);
        engine2.start_game(deck1, deck2, seed);

        // Decks should be in same order
        assert_eq!(
            engine1.state.players[0].deck,
            engine2.state.players[0].deck,
            "Seed {}: P1 deck order differs",
            seed
        );
        assert_eq!(
            engine1.state.players[1].deck,
            engine2.state.players[1].deck,
            "Seed {}: P2 deck order differs",
            seed
        );

        // Hands should be same
        assert_eq!(
            engine1.state.players[0].hand,
            engine2.state.players[0].hand,
            "Seed {}: P1 hand differs",
            seed
        );
        assert_eq!(
            engine1.state.players[1].hand,
            engine2.state.players[1].hand,
            "Seed {}: P2 hand differs",
            seed
        );
    }
}
