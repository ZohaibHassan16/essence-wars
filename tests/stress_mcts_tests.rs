//! MCTS stress tests for long-running validation.
//!
//! These tests are ignored by default because they take a long time.
//! Run with: `cargo test --release --test stress_mcts_tests -- --ignored --nocapture`
//!
//! Tests verify:
//! - MCTS vs MCTS games complete without crashes
//! - MCTS vs Greedy maintains expected win rates
//! - All bot combinations work correctly with invariant checking
//! - MCTS fork integrity under stress

mod common;

use std::collections::HashSet;

use cardgame::bots::{Bot, GreedyBot, MctsBot, MctsConfig, RandomBot};
use cardgame::cards::CardDatabase;
use cardgame::engine::GameEngine;
use cardgame::types::PlayerId;
use common::*;

/// Helper to create an MCTS config
fn mcts_config(simulations: u32) -> MctsConfig {
    MctsConfig {
        simulations,
        exploration: 1.414,
        max_rollout_depth: 50,
        parallel_trees: 1,
        leaf_rollouts: 1,
    }
}

/// Comprehensive state invariant checker (copied from simulation_tests for independence)
fn verify_invariants(engine: &GameEngine, context: &str) {
    let state = &engine.state;

    assert!(
        state.current_turn >= 1,
        "{}: Invalid turn number {}",
        context,
        state.current_turn
    );

    assert!(
        state.active_player == PlayerId::PLAYER_ONE
            || state.active_player == PlayerId::PLAYER_TWO,
        "{}: Invalid active player",
        context
    );

    let mut all_instance_ids: HashSet<u32> = HashSet::new();

    for (player_idx, player) in state.players.iter().enumerate() {
        let player_ctx = format!("{} P{}", context, player_idx + 1);
        let expected_owner = PlayerId(player_idx as u8);

        assert!(
            player.life <= 30,
            "{}: Life {} exceeds max 30",
            player_ctx,
            player.life
        );

        assert!(
            player.current_essence <= player.max_essence,
            "{}: Current essence {} > max {}",
            player_ctx,
            player.current_essence,
            player.max_essence
        );

        assert!(
            player.creatures.len() <= 5,
            "{}: {} creatures exceeds 5 slots",
            player_ctx,
            player.creatures.len()
        );

        for creature in &player.creatures {
            assert!(
                creature.current_health > 0,
                "{}: Creature in slot {} has {} health",
                player_ctx,
                creature.slot.0,
                creature.current_health
            );

            assert_eq!(
                creature.owner, expected_owner,
                "{}: Creature in slot {} has wrong owner",
                player_ctx,
                creature.slot.0
            );

            assert!(
                all_instance_ids.insert(creature.instance_id.0),
                "{}: Duplicate creature instance_id {:?}",
                player_ctx,
                creature.instance_id
            );
        }

        assert!(
            player.supports.len() <= 2,
            "{}: {} supports exceeds 2 slots",
            player_ctx,
            player.supports.len()
        );

        for support in &player.supports {
            assert!(
                support.current_durability > 0,
                "{}: Support in slot {} has {} durability",
                player_ctx,
                support.slot.0,
                support.current_durability
            );

            assert_eq!(
                support.owner, expected_owner,
                "{}: Support in slot {} has wrong owner",
                player_ctx,
                support.slot.0
            );
        }
    }

    if state.is_terminal() {
        assert!(
            state.result.is_some(),
            "{}: Game is terminal but no result set",
            context
        );
    }
}

/// Run a single game between two bots with invariant checking
fn run_bot_game(
    card_db: &CardDatabase,
    bot1: &mut dyn Bot,
    bot2: &mut dyn Bot,
    seed: u64,
    check_invariants: bool,
) -> Option<PlayerId> {
    let mut engine = GameEngine::new(card_db);
    engine.start_game(valid_yaml_deck(), valid_yaml_deck(), seed);

    let mut action_count = 0;
    let max_actions = 500;

    while !engine.is_game_over() && action_count < max_actions {
        if check_invariants && action_count % 10 == 0 {
            verify_invariants(&engine, &format!("seed={} action={}", seed, action_count));
        }

        let state_tensor = engine.get_state_tensor();
        let legal_mask = engine.get_legal_action_mask();
        let legal_actions = engine.get_legal_actions();

        let action = if engine.current_player() == PlayerId::PLAYER_ONE {
            bot1.select_action(&state_tensor, &legal_mask, &legal_actions)
        } else {
            bot2.select_action(&state_tensor, &legal_mask, &legal_actions)
        };

        engine.apply_action(action).unwrap();
        action_count += 1;
    }

    engine.winner()
}

// ============================================================================
// MCTS vs MCTS Stress Tests
// ============================================================================

/// Stress test: 100 MCTS vs MCTS games with invariant checking.
///
/// Run with: `cargo test --release stress_test_mcts_vs_mcts -- --ignored --nocapture`
#[test]
#[ignore]
fn stress_test_mcts_vs_mcts_100_games() {
    const NUM_GAMES: u64 = 100;

    let card_db = CardDatabase::load_from_directory("data/cards")
        .expect("Failed to load cards");

    let mut p1_wins = 0u64;
    let mut p2_wins = 0u64;
    let mut draws = 0u64;

    eprintln!("Running {} MCTS vs MCTS games...", NUM_GAMES);

    for seed in 0..NUM_GAMES {
        let mut bot1 = MctsBot::with_config(&card_db, mcts_config(50), seed);
        let mut bot2 = MctsBot::with_config(&card_db, mcts_config(50), seed + 10000);

        match run_bot_game(&card_db, &mut bot1, &mut bot2, seed, true) {
            Some(PlayerId::PLAYER_ONE) => p1_wins += 1,
            Some(PlayerId::PLAYER_TWO) => p2_wins += 1,
            _ => draws += 1,
        }

        if (seed + 1) % 10 == 0 {
            eprintln!(
                "Progress: {}/{} games (P1: {}, P2: {}, Draws: {})",
                seed + 1,
                NUM_GAMES,
                p1_wins,
                p2_wins,
                draws
            );
        }
    }

    eprintln!(
        "\nMCTS vs MCTS complete: P1 wins: {}, P2 wins: {}, Draws: {}",
        p1_wins, p2_wins, draws
    );

    // Both players should win some games (game isn't completely one-sided)
    assert!(
        p1_wins > 0 && p2_wins > 0,
        "Expected both players to win some games"
    );
}

/// Stress test: 500 MCTS vs Greedy games.
///
/// Run with: `cargo test --release stress_test_mcts_vs_greedy_500 -- --ignored --nocapture`
#[test]
#[ignore]
fn stress_test_mcts_vs_greedy_500_games() {
    const NUM_GAMES: u64 = 500;

    let card_db = CardDatabase::load_from_directory("data/cards")
        .expect("Failed to load cards");

    let mut mcts_wins = 0u64;
    let mut greedy_wins = 0u64;
    let mut draws = 0u64;

    eprintln!("Running {} MCTS vs Greedy games...", NUM_GAMES);

    for seed in 0..NUM_GAMES {
        let mut mcts_bot = MctsBot::with_config(&card_db, mcts_config(100), seed);
        let mut greedy_bot = GreedyBot::new(&card_db, seed + 10000);

        match run_bot_game(&card_db, &mut mcts_bot, &mut greedy_bot, seed, seed % 50 == 0) {
            Some(PlayerId::PLAYER_ONE) => mcts_wins += 1,
            Some(PlayerId::PLAYER_TWO) => greedy_wins += 1,
            _ => draws += 1,
        }

        if (seed + 1) % 50 == 0 {
            let win_rate = mcts_wins as f64 / (mcts_wins + greedy_wins + draws) as f64 * 100.0;
            eprintln!(
                "Progress: {}/{} games | MCTS: {} ({:.1}%), Greedy: {}, Draws: {}",
                seed + 1,
                NUM_GAMES,
                mcts_wins,
                win_rate,
                greedy_wins,
                draws
            );
        }
    }

    let win_rate = mcts_wins as f64 / (mcts_wins + greedy_wins) as f64 * 100.0;
    eprintln!(
        "\nMCTS vs Greedy complete: MCTS wins: {} ({:.1}%), Greedy wins: {}, Draws: {}",
        mcts_wins, win_rate, greedy_wins, draws
    );

    // MCTS should win at least 50% against Greedy
    assert!(
        mcts_wins >= greedy_wins,
        "MCTS should win at least as often as Greedy: {} vs {}",
        mcts_wins,
        greedy_wins
    );
}

// ============================================================================
// Mixed Bot Stress Tests
// ============================================================================

/// Stress test all bot combinations with invariant checking.
///
/// Run with: `cargo test --release stress_test_all_bot_combinations -- --ignored --nocapture`
#[test]
#[ignore]
fn stress_test_all_bot_combinations() {
    let card_db = CardDatabase::load_from_directory("data/cards")
        .expect("Failed to load cards");

    eprintln!("=== All Bot Combinations Stress Test ===\n");

    // Random vs Greedy (100 games)
    {
        eprintln!("Random vs Greedy (100 games)...");
        let mut random_wins = 0;
        let mut greedy_wins = 0;

        for seed in 0u64..100 {
            let mut random_bot = RandomBot::new(seed);
            let mut greedy_bot = GreedyBot::new(&card_db, seed + 1000);

            match run_bot_game(&card_db, &mut random_bot, &mut greedy_bot, seed, true) {
                Some(PlayerId::PLAYER_ONE) => random_wins += 1,
                Some(PlayerId::PLAYER_TWO) => greedy_wins += 1,
                _ => {}
            }
        }

        eprintln!("  Random: {}, Greedy: {}", random_wins, greedy_wins);
        assert!(
            greedy_wins > random_wins,
            "Greedy should beat Random most of the time"
        );
    }

    // Random vs MCTS (50 games)
    {
        eprintln!("Random vs MCTS (50 games)...");
        let mut random_wins = 0;
        let mut mcts_wins = 0;

        for seed in 0u64..50 {
            let mut random_bot = RandomBot::new(seed);
            let mut mcts_bot = MctsBot::with_config(&card_db, mcts_config(50), seed + 1000);

            match run_bot_game(&card_db, &mut random_bot, &mut mcts_bot, seed, true) {
                Some(PlayerId::PLAYER_ONE) => random_wins += 1,
                Some(PlayerId::PLAYER_TWO) => mcts_wins += 1,
                _ => {}
            }
        }

        eprintln!("  Random: {}, MCTS: {}", random_wins, mcts_wins);
        assert!(
            mcts_wins > random_wins,
            "MCTS should beat Random most of the time"
        );
    }

    // Greedy vs MCTS (100 games)
    {
        eprintln!("Greedy vs MCTS (100 games)...");
        let mut greedy_wins = 0;
        let mut mcts_wins = 0;

        for seed in 0u64..100 {
            let mut greedy_bot = GreedyBot::new(&card_db, seed);
            let mut mcts_bot = MctsBot::with_config(&card_db, mcts_config(100), seed + 1000);

            match run_bot_game(&card_db, &mut greedy_bot, &mut mcts_bot, seed, seed % 10 == 0) {
                Some(PlayerId::PLAYER_ONE) => greedy_wins += 1,
                Some(PlayerId::PLAYER_TWO) => mcts_wins += 1,
                _ => {}
            }
        }

        eprintln!("  Greedy: {}, MCTS: {}", greedy_wins, mcts_wins);
        // MCTS should be competitive with Greedy
        assert!(
            mcts_wins >= greedy_wins / 2,
            "MCTS should not lose badly to Greedy"
        );
    }

    // MCTS vs MCTS (50 games)
    {
        eprintln!("MCTS vs MCTS (50 games)...");
        let mut p1_wins = 0;
        let mut p2_wins = 0;

        for seed in 0u64..50 {
            let mut mcts1 = MctsBot::with_config(&card_db, mcts_config(50), seed);
            let mut mcts2 = MctsBot::with_config(&card_db, mcts_config(50), seed + 1000);

            match run_bot_game(&card_db, &mut mcts1, &mut mcts2, seed, true) {
                Some(PlayerId::PLAYER_ONE) => p1_wins += 1,
                Some(PlayerId::PLAYER_TWO) => p2_wins += 1,
                _ => {}
            }
        }

        eprintln!("  P1: {}, P2: {}", p1_wins, p2_wins);
    }

    eprintln!("\n=== All Bot Combinations Complete ===");
}

// ============================================================================
// MCTS Fork Integrity Under Stress
// ============================================================================

/// Verify MCTS fork integrity doesn't corrupt state under heavy use.
///
/// Run with: `cargo test --release stress_test_mcts_fork_integrity -- --ignored --nocapture`
#[test]
#[ignore]
fn stress_test_mcts_fork_integrity() {
    const NUM_GAMES: u64 = 50;

    let card_db = CardDatabase::load_from_directory("data/cards")
        .expect("Failed to load cards");

    eprintln!("Testing MCTS fork integrity over {} games...", NUM_GAMES);

    for seed in 0..NUM_GAMES {
        let mut engine = GameEngine::new(&card_db);
        engine.start_game(valid_yaml_deck(), valid_yaml_deck(), seed);

        let mut mcts_bot = MctsBot::with_config(&card_db, mcts_config(100), seed);

        let mut action_count = 0;
        while !engine.is_game_over() && action_count < 100 {
            // Record state before MCTS
            let turn_before = engine.turn_number();
            let p1_life_before = engine.state.players[0].life;
            let p2_life_before = engine.state.players[1].life;
            let p1_creatures_before = engine.state.players[0].creatures.len();
            let p2_creatures_before = engine.state.players[1].creatures.len();

            // Run MCTS (which forks internally)
            let state_tensor = engine.get_state_tensor();
            let legal_mask = engine.get_legal_action_mask();
            let legal_actions = engine.get_legal_actions();
            let action = mcts_bot.select_action(&state_tensor, &legal_mask, &legal_actions);

            // Verify state unchanged after MCTS selection
            assert_eq!(
                engine.turn_number(),
                turn_before,
                "Turn changed after MCTS at seed={} action={}",
                seed,
                action_count
            );
            assert_eq!(
                engine.state.players[0].life,
                p1_life_before,
                "P1 life changed after MCTS"
            );
            assert_eq!(
                engine.state.players[1].life,
                p2_life_before,
                "P2 life changed after MCTS"
            );
            assert_eq!(
                engine.state.players[0].creatures.len(),
                p1_creatures_before,
                "P1 creatures changed after MCTS"
            );
            assert_eq!(
                engine.state.players[1].creatures.len(),
                p2_creatures_before,
                "P2 creatures changed after MCTS"
            );

            // Now actually apply the action
            engine.apply_action(action).unwrap();
            action_count += 1;
        }

        if (seed + 1) % 10 == 0 {
            eprintln!("  Completed {} games", seed + 1);
        }
    }

    eprintln!("Fork integrity test passed!");
}

// ============================================================================
// High Simulation Count Test
// ============================================================================

/// Test MCTS with high simulation count for quality assessment.
///
/// Run with: `cargo test --release stress_test_mcts_high_sims -- --ignored --nocapture`
#[test]
#[ignore]
fn stress_test_mcts_high_sims() {
    const NUM_GAMES: u64 = 20;

    let card_db = CardDatabase::load_from_directory("data/cards")
        .expect("Failed to load cards");

    let mut high_sim_wins = 0u64;
    let mut low_sim_wins = 0u64;

    eprintln!("Testing high-sim MCTS (500) vs low-sim MCTS (20)...");

    for seed in 0..NUM_GAMES {
        let mut high_sim_bot = MctsBot::with_config(&card_db, mcts_config(500), seed);
        let mut low_sim_bot = MctsBot::with_config(&card_db, mcts_config(20), seed + 1000);

        match run_bot_game(&card_db, &mut high_sim_bot, &mut low_sim_bot, seed, false) {
            Some(PlayerId::PLAYER_ONE) => high_sim_wins += 1,
            Some(PlayerId::PLAYER_TWO) => low_sim_wins += 1,
            _ => {}
        }

        eprintln!("  Game {}: High={}, Low={}", seed + 1, high_sim_wins, low_sim_wins);
    }

    let high_win_rate = high_sim_wins as f64 / NUM_GAMES as f64 * 100.0;
    eprintln!(
        "\nHigh-sim wins: {} ({:.1}%), Low-sim wins: {}",
        high_sim_wins, high_win_rate, low_sim_wins
    );

    // High simulation count should generally perform better
    assert!(
        high_sim_wins >= low_sim_wins,
        "Higher simulation count should not lose to lower"
    );
}
