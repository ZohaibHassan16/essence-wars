//! Property-based tests for the Essence Wars game engine.
//!
//! Uses proptest to verify invariants across randomly generated game states.

mod common;

use proptest::prelude::*;

use cardgame::actions::Action;
use cardgame::cards::CardDatabase;
use cardgame::config::{game, player};
use cardgame::engine::GameEngine;
use cardgame::state::GameState;
use cardgame::types::{CardId, Slot};

use common::{test_card_db, valid_yaml_deck};

// =============================================================================
// Strategy generators for proptest
// =============================================================================

/// Strategy to generate a valid card ID from the starter set (1-43)
fn card_id_strategy() -> impl Strategy<Value = CardId> {
    (1u16..=43).prop_map(CardId)
}

/// Strategy to generate a valid deck (20 cards from starter set)
fn deck_strategy() -> impl Strategy<Value = Vec<CardId>> {
    prop::collection::vec(card_id_strategy(), 20..=20)
}

/// Strategy to generate a random seed
fn seed_strategy() -> impl Strategy<Value = u64> {
    0u64..1_000_000
}

// =============================================================================
// Helper functions
// =============================================================================

/// Load the full card database
fn load_full_card_db() -> CardDatabase {
    CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load card database")
}

/// Run a game with random actions until terminal or max steps
fn run_random_game(
    engine: &mut GameEngine,
    seed: u64,
    max_steps: usize,
) -> (bool, usize) {
    // Simple LCG PRNG for deterministic randomness
    let mut rng_state = seed;
    let mut steps = 0;

    while !engine.is_game_over() && steps < max_steps {
        let legal_actions = engine.get_legal_actions();
        if legal_actions.is_empty() {
            break;
        }

        // LCG step
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let idx = (rng_state >> 32) as usize % legal_actions.len();
        let action = legal_actions[idx];

        if engine.apply_action(action).is_err() {
            break;
        }
        steps += 1;
    }

    (engine.is_game_over(), steps)
}

/// Verify game state invariants
fn verify_state_invariants(state: &GameState, context: &str) {
    // Turn number is positive
    assert!(
        state.current_turn >= 1,
        "{}: Turn number should be >= 1, got {}",
        context,
        state.current_turn
    );

    // Turn limit check
    assert!(
        state.current_turn <= game::TURN_LIMIT as u16 + 1,
        "{}: Turn {} exceeds limit {}",
        context,
        state.current_turn,
        game::TURN_LIMIT
    );

    // Check both players
    for (i, player_state) in state.players.iter().enumerate() {
        // Life can be negative (damage dealt) but shouldn't be absurdly low
        assert!(
            player_state.life >= -100,
            "{}: Player {} life is too low: {}",
            context,
            i + 1,
            player_state.life
        );

        // Life cap
        assert!(
            player_state.life <= player::STARTING_LIFE as i16 + 50,
            "{}: Player {} life is too high: {}",
            context,
            i + 1,
            player_state.life
        );

        // Action points
        assert!(
            player_state.action_points <= player::AP_PER_TURN + 5,
            "{}: Player {} has too many AP: {}",
            context,
            i + 1,
            player_state.action_points
        );

        // Essence
        assert!(
            player_state.current_essence <= player::MAX_ESSENCE + 5,
            "{}: Player {} has too much essence: {}",
            context,
            i + 1,
            player_state.current_essence
        );

        // Creatures on board have valid health
        for creature in player_state.creatures.iter() {
            // Current health can be 0 or negative briefly during combat
            // but shouldn't be absurdly negative
            assert!(
                creature.current_health >= -50,
                "{}: Player {} creature at slot {:?} has health {}",
                context,
                i + 1,
                creature.slot,
                creature.current_health
            );
        }

        // Hand size limit
        assert!(
            player_state.hand.len() <= player::MAX_HAND_SIZE + 1,
            "{}: Player {} hand size {} exceeds max",
            context,
            i + 1,
            player_state.hand.len()
        );
    }
}

// =============================================================================
// Property tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 1: All legal actions can be successfully applied
    #[test]
    fn prop_legal_actions_are_playable(seed in seed_strategy()) {
        let card_db = load_full_card_db();
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();

        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck1, deck2, seed);

        // Play several turns and verify each legal action can be applied
        for _ in 0..50 {
            if engine.is_game_over() {
                break;
            }

            let legal_actions = engine.get_legal_actions();
            for action in &legal_actions {
                // Fork engine to test without modifying original
                let mut test_engine = engine.fork();
                let result = test_engine.apply_action(*action);
                prop_assert!(
                    result.is_ok(),
                    "Legal action {:?} failed: {:?}",
                    action,
                    result.err()
                );
            }

            // Apply a random legal action to advance the game
            if !legal_actions.is_empty() {
                let rng_val = seed.wrapping_mul(6364136223846793005).wrapping_add(legal_actions.len() as u64);
                let idx = (rng_val as usize) % legal_actions.len();
                let _ = engine.apply_action(legal_actions[idx]);
            }
        }
    }

    /// Property 2: Legal action mask matches legal actions list
    #[test]
    fn prop_legal_mask_matches_actions(seed in seed_strategy()) {
        let card_db = load_full_card_db();
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();

        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck1, deck2, seed);

        for _ in 0..30 {
            if engine.is_game_over() {
                break;
            }

            let legal_actions = engine.get_legal_actions();
            let legal_mask = engine.get_legal_action_mask();

            // Count legal actions from mask
            let mask_count: usize = legal_mask.iter().map(|&x| if x > 0.5 { 1 } else { 0 }).sum();

            prop_assert_eq!(
                legal_actions.len(),
                mask_count,
                "Legal actions count ({}) doesn't match mask count ({})",
                legal_actions.len(),
                mask_count
            );

            // Verify each legal action has mask = 1.0
            for action in &legal_actions {
                let idx = action.to_index();
                prop_assert!(
                    legal_mask[idx as usize] > 0.5,
                    "Action {:?} (index {}) is legal but mask is {}",
                    action,
                    idx,
                    legal_mask[idx as usize]
                );
            }

            // Advance game
            if !legal_actions.is_empty() {
                let _ = engine.apply_action(legal_actions[0]);
            }
        }
    }

    /// Property 3: Action index encode/decode roundtrip
    #[test]
    fn prop_action_index_roundtrip(
        hand_index in 0u8..10,
        slot in 0u8..5,
        target in 0u8..5,
    ) {
        // Test PlayCard
        let play_action = Action::PlayCard {
            hand_index,
            slot: Slot(slot),
        };
        let play_idx = play_action.to_index();
        let play_decoded = Action::from_index(play_idx);
        prop_assert_eq!(Some(play_action), play_decoded);

        // Test Attack
        let attack_action = Action::Attack {
            attacker: Slot(slot),
            defender: Slot(target),
        };
        let attack_idx = attack_action.to_index();
        let attack_decoded = Action::from_index(attack_idx);
        prop_assert_eq!(Some(attack_action), attack_decoded);

        // Test EndTurn
        let end_action = Action::EndTurn;
        let end_idx = end_action.to_index();
        let end_decoded = Action::from_index(end_idx);
        prop_assert_eq!(Some(end_action), end_decoded);
    }

    /// Property 4: Terminal states have no legal actions
    #[test]
    fn prop_terminal_state_no_actions(seed in seed_strategy()) {
        let card_db = load_full_card_db();
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();

        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck1, deck2, seed);

        // Run game to completion
        let (finished, _) = run_random_game(&mut engine, seed, 500);

        if finished && engine.is_game_over() {
            let legal_actions = engine.get_legal_actions();
            // Terminal state should have no legal actions
            prop_assert!(
                legal_actions.is_empty(),
                "Terminal state has {} legal actions: {:?}",
                legal_actions.len(),
                legal_actions
            );
        }
    }

    /// Property 5: Game state invariants hold after any action sequence
    #[test]
    fn prop_state_invariants_maintained(seed in seed_strategy()) {
        let card_db = load_full_card_db();
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();

        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck1, deck2, seed);

        // Verify initial state
        verify_state_invariants(&engine.state, "Initial state");

        // Run random game, verifying invariants at each step
        let mut rng_state = seed;
        for step in 0..100 {
            if engine.is_game_over() {
                break;
            }

            let legal_actions = engine.get_legal_actions();
            if legal_actions.is_empty() {
                break;
            }

            // Pick random action
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let idx = (rng_state >> 32) as usize % legal_actions.len();
            let action = legal_actions[idx];

            if engine.apply_action(action).is_err() {
                break;
            }

            // Verify invariants after each action
            verify_state_invariants(&engine.state, &format!("After step {}", step + 1));
        }
    }

    /// Property 6: Determinism - same seed produces identical game
    #[test]
    fn prop_determinism(seed in seed_strategy()) {
        let card_db = load_full_card_db();
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();

        // Run game twice with same seed and random action selection
        let mut engine1 = GameEngine::new(&card_db);
        engine1.start_game(deck1.clone(), deck2.clone(), seed);
        let (_, steps1) = run_random_game(&mut engine1, seed, 100);

        let mut engine2 = GameEngine::new(&card_db);
        engine2.start_game(deck1, deck2, seed);
        let (_, steps2) = run_random_game(&mut engine2, seed, 100);

        // Same number of steps
        prop_assert_eq!(steps1, steps2, "Step counts differ");

        // Same winner
        prop_assert_eq!(
            engine1.winner(),
            engine2.winner(),
            "Winners differ"
        );

        // Same turn
        prop_assert_eq!(
            engine1.turn_number(),
            engine2.turn_number(),
            "Turn numbers differ"
        );

        // Same life totals
        prop_assert_eq!(
            engine1.state.players[0].life,
            engine2.state.players[0].life,
            "Player 1 life differs"
        );
        prop_assert_eq!(
            engine1.state.players[1].life,
            engine2.state.players[1].life,
            "Player 2 life differs"
        );
    }

    /// Property 7: Fork produces independent states
    #[test]
    fn prop_fork_isolation(seed in seed_strategy()) {
        let card_db = load_full_card_db();
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();

        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck1, deck2, seed);

        // Advance a few steps
        for _ in 0..10 {
            let legal = engine.get_legal_actions();
            if legal.is_empty() || engine.is_game_over() {
                break;
            }
            let _ = engine.apply_action(legal[0]);
        }

        // Fork the engine
        let mut forked = engine.fork();

        // Verify they start identical
        prop_assert_eq!(engine.turn_number(), forked.turn_number());
        prop_assert_eq!(engine.state.players[0].life, forked.state.players[0].life);

        // Modify forked state
        let forked_legal = forked.get_legal_actions();
        if !forked_legal.is_empty() && !forked.is_game_over() {
            let _ = forked.apply_action(forked_legal[0]);
        }

        // Original should be unchanged (if forked took an action)
        if forked.turn_number() != engine.turn_number() || !forked_legal.is_empty() {
            // At minimum, some state should differ after forked advanced
            // (unless they were already terminal)
            if !engine.is_game_over() {
                let engine_tensor = engine.get_state_tensor();
                let forked_tensor = forked.get_state_tensor();
                let tensors_differ = engine_tensor.iter()
                    .zip(forked_tensor.iter())
                    .any(|(a, b)| (a - b).abs() > 0.001);
                prop_assert!(
                    tensors_differ,
                    "Forked state should differ from original after action"
                );
            }
        }
    }

    /// Property 8: State tensor has correct size and bounded values
    #[test]
    fn prop_tensor_validity(seed in seed_strategy()) {
        let card_db = load_full_card_db();
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();

        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck1, deck2, seed);

        // Check tensor at various game states
        for _ in 0..30 {
            let tensor = engine.get_state_tensor();

            // Correct size
            prop_assert_eq!(tensor.len(), 326, "Tensor should have 326 elements");

            // All values should be finite
            for (i, &val) in tensor.iter().enumerate() {
                prop_assert!(
                    val.is_finite(),
                    "Tensor[{}] is not finite: {}",
                    i,
                    val
                );
            }

            // Most values should be in reasonable range [-10, 10]
            // (some may be larger for card IDs or other raw values)
            let extreme_count = tensor.iter()
                .filter(|&&v| v.abs() > 100.0)
                .count();
            prop_assert!(
                extreme_count < 50,
                "Too many extreme values in tensor: {}",
                extreme_count
            );

            // Advance game
            let legal = engine.get_legal_actions();
            if legal.is_empty() || engine.is_game_over() {
                break;
            }
            let _ = engine.apply_action(legal[0]);
        }
    }

    /// Property 9: Random deck compositions don't crash the engine
    /// This tests a wider variety of deck combinations than the fixed valid_yaml_deck()
    #[test]
    fn prop_random_decks_playable(
        seed in seed_strategy(),
        deck1 in deck_strategy(),
        deck2 in deck_strategy(),
    ) {
        let card_db = load_full_card_db();

        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck1, deck2, seed);

        // Verify game started correctly
        prop_assert!(!engine.is_game_over(), "Game should not be over at start");
        prop_assert!(engine.turn_number() >= 1, "Turn should be at least 1");

        // Run random game to completion - should not panic
        let (finished, steps) = run_random_game(&mut engine, seed, 200);

        // Verify invariants held throughout
        verify_state_invariants(&engine.state, &format!("After {} steps", steps));

        // If game finished, verify it has a proper result
        if finished {
            prop_assert!(
                engine.state.result.is_some(),
                "Finished game should have a result"
            );
        }
    }
}

// =============================================================================
// Additional non-proptest tests for edge cases
// =============================================================================

#[test]
fn test_action_index_boundaries() {
    // Test boundary action indices
    assert_eq!(Action::EndTurn.to_index(), 255);

    // PlayCard boundaries: hand 0-9, slot 0-4 = indices 0-49
    // Encoding: hand_index * 5 + slot
    let play_first = Action::PlayCard {
        hand_index: 0,
        slot: Slot(0),
    };
    assert_eq!(play_first.to_index(), 0);

    let play_last = Action::PlayCard {
        hand_index: 9,
        slot: Slot(4),
    };
    assert_eq!(play_last.to_index(), 49); // 9*5+4 = 49

    // Attack boundaries: attacker 0-4, defender 0-4 = indices 50-74
    // Encoding: 50 + attacker * 5 + defender
    let attack_first = Action::Attack {
        attacker: Slot(0),
        defender: Slot(0),
    };
    assert_eq!(attack_first.to_index(), 50);

    let attack_last = Action::Attack {
        attacker: Slot(4),
        defender: Slot(4),
    };
    assert_eq!(attack_last.to_index(), 74); // 50 + 4*5 + 4 = 74
}

#[test]
fn test_empty_legal_actions_terminal() {
    let card_db = test_card_db();
    let deck1 = common::simple_deck();
    let deck2 = common::simple_deck();

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck1, deck2, 12345);

    // Run game to completion instead of manually setting state
    let (_, _) = run_random_game(&mut engine, 12345, 500);

    if engine.is_game_over() {
        let legal = engine.get_legal_actions();
        assert!(
            legal.is_empty(),
            "Terminal state should have no legal actions"
        );
    }
}

#[test]
fn test_all_action_indices_valid() {
    // Verify all 256 indices can be decoded (some may be None for invalid indices)
    for i in 0u8..=255 {
        let action_opt = Action::from_index(i);
        if let Some(action) = action_opt {
            let roundtrip = action.to_index();
            assert_eq!(
                i, roundtrip,
                "Index {} roundtrip failed: {:?} -> {}",
                i, action, roundtrip
            );
        }
    }
}
