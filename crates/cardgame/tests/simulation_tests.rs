//! Full game simulation tests with random action selection and bot-driven fuzzing.
//!
//! These tests verify:
//! - Games with random actions complete without panics
//! - Games terminate within the turn limit (30 turns)
//! - No invalid states occur during random play
//! - GreedyBot games maintain state invariants
//! - Comprehensive invariant checking after every action

mod common;

use std::collections::HashSet;

use cardgame::actions::Action;
use cardgame::bots::{Bot, GreedyBot};
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
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
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
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
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
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
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
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
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
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
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

        // Victory points should be within reasonable bounds (allowing for high-damage games)
        // A typical game might deal 30+ to life and 100+ to creatures total
        assert!(p1_vp <= 300 || engine.is_game_over(), "P1 VP tracking error: {}", p1_vp);
        assert!(p2_vp <= 300 || engine.is_game_over(), "P2 VP tracking error: {}", p2_vp);

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
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
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

// ============================================================================
// Enhanced Invariant Checking
// ============================================================================

/// Comprehensive state invariant checker
fn verify_comprehensive_invariants(engine: &GameEngine, context: &str) {
    let state = &engine.state;

    // Turn number should be positive
    assert!(
        state.current_turn >= 1,
        "{}: Invalid turn number {}",
        context,
        state.current_turn
    );

    // Active player should be valid
    assert!(
        state.active_player == PlayerId::PLAYER_ONE
            || state.active_player == PlayerId::PLAYER_TWO,
        "{}: Invalid active player",
        context
    );

    // Collect all creature instance IDs across both players to check uniqueness
    let mut all_instance_ids: HashSet<u32> = HashSet::new();

    for (player_idx, player) in state.players.iter().enumerate() {
        let player_ctx = format!("{} P{}", context, player_idx + 1);
        let expected_owner = PlayerId(player_idx as u8);

        // Life should be at most 30 (can go negative during game over)
        assert!(
            player.life <= 30,
            "{}: Life {} exceeds max 30",
            player_ctx,
            player.life
        );

        // Essence constraints
        assert!(
            player.current_essence <= player.max_essence,
            "{}: Current essence {} > max {}",
            player_ctx,
            player.current_essence,
            player.max_essence
        );
        assert!(
            player.max_essence <= 10,
            "{}: Max essence {} > cap 10",
            player_ctx,
            player.max_essence
        );

        // AP constraints
        assert!(
            player.action_points <= 5,
            "{}: AP {} unreasonably high",
            player_ctx,
            player.action_points
        );

        // Creature slot constraints
        assert!(
            player.creatures.len() <= 5,
            "{}: {} creatures exceeds 5 slots",
            player_ctx,
            player.creatures.len()
        );

        // No duplicate creature slots
        let creature_slots: Vec<u8> = player.creatures.iter().map(|c| c.slot.0).collect();
        let unique_slots: HashSet<u8> = creature_slots.iter().copied().collect();
        assert_eq!(
            creature_slots.len(),
            unique_slots.len(),
            "{}: Duplicate creature slots detected: {:?}",
            player_ctx,
            creature_slots
        );

        // All creature slots should be valid (0-4)
        for slot in &creature_slots {
            assert!(
                *slot < 5,
                "{}: Invalid creature slot {}",
                player_ctx,
                slot
            );
        }

        // Creatures should have positive health (dead ones should be removed)
        for creature in &player.creatures {
            assert!(
                creature.current_health > 0,
                "{}: Creature in slot {} has {} health (should be removed)",
                player_ctx,
                creature.slot.0,
                creature.current_health
            );
            // Use i16 to prevent overflow when adding the buff allowance
            assert!(
                (creature.current_health as i16) <= (creature.max_health as i16) + 20, // Allow for buffs
                "{}: Creature health {} unreasonably high (max {})",
                player_ctx,
                creature.current_health,
                creature.max_health
            );

            // Creature owner should match player
            assert_eq!(
                creature.owner, expected_owner,
                "{}: Creature in slot {} has wrong owner {:?} (expected {:?})",
                player_ctx,
                creature.slot.0,
                creature.owner,
                expected_owner
            );

            // Check instance ID uniqueness across both players
            assert!(
                all_instance_ids.insert(creature.instance_id.0),
                "{}: Duplicate creature instance_id {:?} detected",
                player_ctx,
                creature.instance_id
            );
        }

        // Support slot constraints
        assert!(
            player.supports.len() <= 2,
            "{}: {} supports exceeds 2 slots",
            player_ctx,
            player.supports.len()
        );

        // No duplicate support slots
        let support_slots: Vec<u8> = player.supports.iter().map(|s| s.slot.0).collect();
        let unique_support_slots: HashSet<u8> = support_slots.iter().copied().collect();
        assert_eq!(
            support_slots.len(),
            unique_support_slots.len(),
            "{}: Duplicate support slots detected: {:?}",
            player_ctx,
            support_slots
        );

        // All support slots should be valid (0-1)
        for slot in &support_slots {
            assert!(
                *slot < 2,
                "{}: Invalid support slot {}",
                player_ctx,
                slot
            );
        }

        // Supports should have positive durability
        for support in &player.supports {
            assert!(
                support.current_durability > 0,
                "{}: Support in slot {} has {} durability (should be removed)",
                player_ctx,
                support.slot.0,
                support.current_durability
            );

            // Support owner should match player
            assert_eq!(
                support.owner, expected_owner,
                "{}: Support in slot {} has wrong owner {:?} (expected {:?})",
                player_ctx,
                support.slot.0,
                support.owner,
                expected_owner
            );
        }

        // Hand size
        assert!(
            player.hand.len() <= 10,
            "{}: Hand size {} exceeds max 10",
            player_ctx,
            player.hand.len()
        );
    }

    // Game result consistency
    if state.players[0].life <= 0 || state.players[1].life <= 0 {
        assert!(
            state.is_terminal(),
            "{}: Player has <= 0 life but game not terminal",
            context
        );
    }

    if state.is_terminal() {
        assert!(
            state.result.is_some(),
            "{}: Game is terminal but no result set",
            context
        );
    }
}

/// Enhanced invariant checker that also validates card IDs against the database
fn verify_comprehensive_invariants_with_db(
    engine: &GameEngine,
    card_db: &CardDatabase,
    context: &str,
) {
    // First run the standard invariant checks
    verify_comprehensive_invariants(engine, context);

    let state = &engine.state;

    // Validate all card IDs exist in the database
    for (player_idx, player) in state.players.iter().enumerate() {
        let player_ctx = format!("{} P{}", context, player_idx + 1);

        // Check hand card IDs
        for card in &player.hand {
            assert!(
                card_db.get(card.card_id).is_some(),
                "{}: Hand contains invalid card_id {:?}",
                player_ctx,
                card.card_id
            );
        }

        // Check deck card IDs
        for card in &player.deck {
            assert!(
                card_db.get(card.card_id).is_some(),
                "{}: Deck contains invalid card_id {:?}",
                player_ctx,
                card.card_id
            );
        }

        // Check creature card IDs
        for creature in &player.creatures {
            assert!(
                card_db.get(creature.card_id).is_some(),
                "{}: Creature in slot {} has invalid card_id {:?}",
                player_ctx,
                creature.slot.0,
                creature.card_id
            );
        }

        // Check support card IDs
        for support in &player.supports {
            assert!(
                card_db.get(support.card_id).is_some(),
                "{}: Support in slot {} has invalid card_id {:?}",
                player_ctx,
                support.slot.0,
                support.card_id
            );
        }
    }
}

// ============================================================================
// Bot-Driven Fuzzing Tests
// ============================================================================

/// Test with full database validation (checks all card IDs are valid)
#[test]
fn test_database_validated_games() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    // Verify deck card IDs exist in the database first
    let deck = valid_yaml_deck();
    for card_id in &deck {
        assert!(
            card_db.get(*card_id).is_some(),
            "valid_yaml_deck contains invalid card_id {:?} - not found in loaded database",
            card_id
        );
    }

    let mut rng = SimpleRng::new(777);

    for seed in 0u64..20 {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        engine.start_game(deck1, deck2, seed);

        let mut action_count = 0;

        while !engine.is_game_over() && action_count < 100 {
            // Run enhanced invariant check with database validation
            let context = format!("DBVal Game {} action {}", seed, action_count);
            verify_comprehensive_invariants_with_db(&engine, &card_db, &context);

            let actions = engine.get_legal_actions();
            let action_idx = rng.range(actions.len());
            engine.apply_action(actions[action_idx]).unwrap();
            action_count += 1;
        }

        // Final check
        verify_comprehensive_invariants_with_db(
            &engine,
            &card_db,
            &format!("DBVal Game {} final", seed),
        );
    }
}

/// Test GreedyBot vs GreedyBot games with comprehensive invariant checking
#[test]
fn test_greedy_vs_greedy_with_invariants() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    let mut games_completed = 0;
    let mut total_actions = 0;

    // Use arena_test_deck which is optimized for faster game conclusions
    for seed in 0u64..100 {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = arena_test_deck();
        let deck2 = arena_test_deck();
        engine.start_game(deck1, deck2, seed);

        let mut bot1 = GreedyBot::new(&card_db, seed);
        let mut bot2 = GreedyBot::new(&card_db, seed + 1000);

        let mut action_count = 0;
        let max_actions = 500;

        while !engine.is_game_over() && action_count < max_actions {
            let context = format!("Game {} turn {} action {}", seed, engine.turn_number(), action_count);

            // Verify invariants before action
            verify_comprehensive_invariants(&engine, &context);

            // Select action based on active player
            let action = if engine.state.active_player == PlayerId::PLAYER_ONE {
                bot1.select_action_with_engine(&engine)
            } else {
                bot2.select_action_with_engine(&engine)
            };

            // Apply action
            let result = engine.apply_action(action);
            assert!(
                result.is_ok(),
                "{}: Action {:?} failed: {:?}",
                context,
                action,
                result
            );

            action_count += 1;
        }

        // Final invariant check
        let final_context = format!("Game {} final", seed);
        verify_comprehensive_invariants(&engine, &final_context);

        if engine.is_game_over() {
            games_completed += 1;
        }
        total_actions += action_count;
    }

    // Most games should complete (some may hit action limit due to certain card combinations)
    // Using 80% threshold to allow for edge cases while still catching major regressions
    assert!(
        games_completed >= 80,
        "Only {} of 100 GreedyBot games completed (expected at least 80)",
        games_completed
    );

    println!(
        "GreedyBot fuzzing: {} games, {} total actions, {:.1} avg actions/game",
        games_completed,
        total_actions,
        total_actions as f64 / 100.0
    );
}

/// Extended stress test with 500 random games
#[test]
fn test_extended_random_stress() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    let mut games_completed = 0;
    let mut invariant_violations = 0;

    for seed in 0u64..500 {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        engine.start_game(deck1, deck2, seed);

        let mut rng = SimpleRng::new(seed);
        let mut action_count = 0;

        while !engine.is_game_over() && action_count < 300 {
            // Quick invariant check (every 10 actions for performance)
            if action_count % 10 == 0 {
                let context = format!("Game {} action {}", seed, action_count);
                // Use a closure to catch panics
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    verify_comprehensive_invariants(&engine, &context);
                }));
                if result.is_err() {
                    invariant_violations += 1;
                }
            }

            let actions = engine.get_legal_actions();
            if actions.is_empty() {
                break;
            }
            let action_idx = rng.range(actions.len());
            engine.apply_action(actions[action_idx]).unwrap();
            action_count += 1;
        }

        if engine.is_game_over() {
            games_completed += 1;
        }
    }

    assert_eq!(
        invariant_violations, 0,
        "{} invariant violations in 500 games",
        invariant_violations
    );

    assert!(
        games_completed >= 490,
        "Only {} of 500 games completed",
        games_completed
    );
}

/// Test that engine fork produces valid states
#[test]
fn test_fork_state_validity() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    for seed in [42u64, 12345, 99999] {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        engine.start_game(deck1, deck2, seed);

        let mut rng = SimpleRng::new(seed);

        // Play some actions
        for i in 0..20 {
            if engine.is_game_over() {
                break;
            }

            let actions = engine.get_legal_actions();
            let action_idx = rng.range(actions.len());
            engine.apply_action(actions[action_idx]).unwrap();

            // Fork at action 10 and verify fork state
            if i == 10 {
                let fork = engine.fork();
                verify_comprehensive_invariants(&fork, &format!("Fork from seed {}", seed));

                // Fork should have same state
                assert_eq!(
                    engine.state.current_turn,
                    fork.state.current_turn,
                    "Fork turn mismatch"
                );
                assert_eq!(
                    engine.state.players[0].life,
                    fork.state.players[0].life,
                    "Fork P1 life mismatch"
                );
            }
        }
    }
}

// ============================================================================
// Long-Running Stress Tests (use --ignored to run)
// ============================================================================

/// Stress test: 100,000 random games with invariant checking after every action.
///
/// Run with: cargo test --release stress_test_100k_random -- --ignored --nocapture
///
/// This test is ignored by default because it takes ~10-15 minutes to run.
/// Use this for overnight or CI stress testing.
#[test]
#[ignore = "tier_overnight"] // ~15 min: 100k random games
fn stress_test_100k_random() {
    const NUM_GAMES: u64 = 100_000;

    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    let mut games_completed = 0u64;
    let mut total_actions = 0u64;

    for seed in 0..NUM_GAMES {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        engine.start_game(deck1, deck2, seed);

        let mut rng = SimpleRng::new(seed);
        let mut action_count = 0;

        while !engine.is_game_over() && action_count < 500 {
            let actions = engine.get_legal_actions();
            if actions.is_empty() {
                break;
            }

            let action_idx = rng.range(actions.len());
            let action = actions[action_idx];
            engine.apply_action(action).unwrap();

            // Verify invariants after every action
            verify_comprehensive_invariants(
                &engine,
                &format!("seed={}, action #{}", seed, action_count),
            );

            action_count += 1;
            total_actions += 1;
        }

        games_completed += 1;

        // Progress every 10k games
        if games_completed % 10_000 == 0 {
            eprintln!(
                "Progress: {}/{} games ({:.1}%), {} total actions",
                games_completed,
                NUM_GAMES,
                (games_completed as f64 / NUM_GAMES as f64) * 100.0,
                total_actions
            );
        }
    }

    eprintln!(
        "\nStress test complete: {} games, {} total actions, no invariant violations",
        games_completed, total_actions
    );
}

/// Stress test: 100,000 GreedyBot vs GreedyBot games with invariant checking.
///
/// Run with: cargo test --release stress_test_100k_greedy -- --ignored --nocapture
///
/// This exercises smarter play patterns and catches bugs that random play misses.
#[test]
#[ignore = "tier_overnight"] // ~30 min: 100k greedy games
fn stress_test_100k_greedy() {
    const NUM_GAMES: u64 = 100_000;

    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    let mut games_completed = 0u64;
    let mut total_actions = 0u64;

    for seed in 0..NUM_GAMES {
        let mut engine = GameEngine::new(&card_db);
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        engine.start_game(deck1, deck2, seed);

        let mut bot1 = GreedyBot::new(&card_db, seed);
        let mut bot2 = GreedyBot::new(&card_db, seed.wrapping_add(1_000_000));

        let mut action_count = 0;

        while !engine.is_game_over() && action_count < 500 {
            let action = if engine.current_player() == PlayerId::PLAYER_ONE {
                bot1.select_action_with_engine(&engine)
            } else {
                bot2.select_action_with_engine(&engine)
            };

            engine.apply_action(action).unwrap();

            // Verify invariants after every action
            verify_comprehensive_invariants(
                &engine,
                &format!("greedy seed={}, action #{}", seed, action_count),
            );

            action_count += 1;
            total_actions += 1;
        }

        games_completed += 1;

        // Progress every 10k games
        if games_completed % 10_000 == 0 {
            eprintln!(
                "Progress: {}/{} games ({:.1}%), {} total actions",
                games_completed,
                NUM_GAMES,
                (games_completed as f64 / NUM_GAMES as f64) * 100.0,
                total_actions
            );
        }
    }

    eprintln!(
        "\nStress test complete: {} games, {} total actions, no invariant violations",
        games_completed, total_actions
    );
}
