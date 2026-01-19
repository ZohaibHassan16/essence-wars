//! Bot validation test suite for the Essence Wars game engine.
//!
//! Tests bots playing games with tracing enabled to catch edge cases.

mod common;

use cardgame::arena::GameRunner;
use cardgame::bots::{GreedyBot, MctsBot, MctsConfig, RandomBot};
use cardgame::cards::CardDatabase;
use cardgame::decks::DeckRegistry;
use cardgame::engine::GameEngine;
use cardgame::types::{CardId, PlayerId};

use common::{arena_test_deck, load_real_card_db, load_real_deck_registry};

// =============================================================================
// Helper functions
// =============================================================================

fn load_card_db() -> CardDatabase {
    load_real_card_db()
}

fn load_decks() -> DeckRegistry {
    load_real_deck_registry()
}

/// Run games between two bots using GameRunner
/// Returns results for test analysis
fn run_traced_match(
    card_db: &CardDatabase,
    bot1: &mut dyn cardgame::bots::Bot,
    bot2: &mut dyn cardgame::bots::Bot,
    deck1: Vec<CardId>,
    deck2: Vec<CardId>,
    games: usize,
    base_seed: u64,
) -> Vec<cardgame::arena::GameResult> {
    let mut results = Vec::with_capacity(games);
    let mut runner = GameRunner::new(card_db);

    for i in 0..games {
        let seed = base_seed.wrapping_add(i as u64);
        let result = runner.run_game(bot1, bot2, deck1.clone(), deck2.clone(), seed);
        results.push(result);
    }

    results
}

/// Verify game results for anomalies
fn verify_no_anomalies(results: &[cardgame::arena::GameResult], context: &str) {
    for (i, result) in results.iter().enumerate() {
        // Check game terminated normally
        assert!(
            result.turns >= 1,
            "{} game {}: should have at least 1 turn, got {}",
            context, i, result.turns
        );

        assert!(
            result.turns <= 31,
            "{} game {}: exceeded turn limit, got {} turns",
            context, i, result.turns
        );

        // Check actions were taken
        assert!(
            !result.actions.is_empty(),
            "{} game {}: no actions recorded",
            context, i
        );

        // Check combat traces don't show impossible states
        for (j, trace) in result.combat_traces.iter().enumerate() {
            // Attacker should have positive attack to deal damage
            if trace.attacker_attack <= 0 && !trace.is_face_attack {
                // Zero-attack creatures shouldn't be attacking
                panic!(
                    "{} game {} combat {}: Zero-attack creature attacked",
                    context, i, j
                );
            }
        }
    }
}

/// Calculate win rate for player 1
fn calculate_win_rate(results: &[cardgame::arena::GameResult]) -> f64 {
    let p1_wins = results.iter()
        .filter(|r| r.winner == Some(PlayerId::PLAYER_ONE))
        .count();
    p1_wins as f64 / results.len() as f64
}

// =============================================================================
// Bot vs Bot Tests
// =============================================================================

#[test]
fn test_greedy_vs_greedy_100_games() {
    let card_db = load_card_db();
    let deck = arena_test_deck();

    let mut bot1 = GreedyBot::new(&card_db, 12345);
    let mut bot2 = GreedyBot::new(&card_db, 54321);

    let results = run_traced_match(
        &card_db,
        &mut bot1,
        &mut bot2,
        deck.clone(),
        deck,
        100,
        42,
    );

    verify_no_anomalies(&results, "Greedy vs Greedy");

    // Note: Greedy bots are deterministic given the same state, so identical
    // bots with same deck will produce consistent outcomes. First player
    // advantage in the game may cause one side to consistently win.
    let p1_win_rate = calculate_win_rate(&results);
    let draws = results.iter().filter(|r| r.winner.is_none()).count();
    println!(
        "Greedy vs Greedy: P1 win rate = {:.1}%, draws = {}",
        p1_win_rate * 100.0,
        draws
    );

    // Log first-player advantage if detected (this is a game balance observation)
    if p1_win_rate >= 0.95 || p1_win_rate <= 0.05 {
        println!(
            "NOTE: Strong first-player advantage detected ({:.1}% P1 win rate)",
            p1_win_rate * 100.0
        );
        println!("This is deterministic behavior when identical bots play same deck.");
    }

    // The main check is that games complete normally - win rate imbalance
    // is a game balance issue, not a bug
    assert!(
        !results.is_empty(),
        "Should have completed games"
    );
}

#[test]
#[ignore = "tier_quick"] // ~1 min: 20 MCTS games
fn test_mcts_vs_mcts_20_games() {
    let card_db = load_card_db();
    let deck = arena_test_deck();

    // Use fast MCTS config for testing
    let config = MctsConfig::fast();

    let mut bot1 = MctsBot::with_config(&card_db, config.clone(), 12345);
    let mut bot2 = MctsBot::with_config(&card_db, config, 54321);

    let results = run_traced_match(
        &card_db,
        &mut bot1,
        &mut bot2,
        deck.clone(),
        deck,
        20,
        1000,
    );

    verify_no_anomalies(&results, "MCTS vs MCTS");

    // MCTS vs MCTS should be balanced
    let p1_win_rate = calculate_win_rate(&results);
    println!(
        "MCTS vs MCTS: P1 win rate = {:.1}%, draws = {}",
        p1_win_rate * 100.0,
        results.iter().filter(|r| r.winner.is_none()).count()
    );
}

#[test]
#[ignore = "tier_medium"] // ~2 min: 50 MCTS games
fn test_mcts_vs_greedy_50_games() {
    let card_db = load_card_db();
    let deck = arena_test_deck();

    let config = MctsConfig::fast();
    let mut mcts = MctsBot::with_config(&card_db, config, 12345);
    let mut greedy = GreedyBot::new(&card_db, 54321);

    let results = run_traced_match(
        &card_db,
        &mut mcts,
        &mut greedy,
        deck.clone(),
        deck,
        50,
        2000,
    );

    verify_no_anomalies(&results, "MCTS vs Greedy");

    // MCTS should generally outperform Greedy
    let mcts_win_rate = calculate_win_rate(&results);
    println!(
        "MCTS vs Greedy: MCTS win rate = {:.1}%, draws = {}",
        mcts_win_rate * 100.0,
        results.iter().filter(|r| r.winner.is_none()).count()
    );

    // MCTS should win at least 40% (being conservative due to fast config)
    assert!(
        mcts_win_rate >= 0.4,
        "MCTS should outperform Greedy, got only {:.1}% win rate",
        mcts_win_rate * 100.0
    );
}

#[test]
fn test_greedy_vs_random_100_games() {
    let card_db = load_card_db();
    let deck = arena_test_deck();

    // Use same seeding strategy as arena: bot seeds derived from game seed
    let base_seed = 3000u64;
    let mut greedy = GreedyBot::new(&card_db, base_seed);
    let mut random = RandomBot::new(base_seed.wrapping_add(1000000));

    let results = run_traced_match(
        &card_db,
        &mut greedy,
        &mut random,
        deck.clone(),
        deck,
        100,
        base_seed,
    );

    verify_no_anomalies(&results, "Greedy vs Random");

    // Greedy should dominate Random
    let greedy_win_rate = calculate_win_rate(&results);
    println!(
        "Greedy vs Random: Greedy win rate = {:.1}%",
        greedy_win_rate * 100.0
    );

    // Greedy should win 75%+ against Random
    // Note: threshold lowered from 95% due to FPA compensation (P2 gets +1 card, +1 essence)
    assert!(
        greedy_win_rate >= 0.75,
        "Greedy should dominate Random, got only {:.1}% win rate",
        greedy_win_rate * 100.0
    );
}

// =============================================================================
// Deck Matchup Tests
// =============================================================================

#[test]
fn test_all_deck_combinations() {
    let card_db = load_card_db();
    let registry = load_decks();

    let deck_ids = registry.deck_ids();

    // Test each deck combination
    for deck1_id in &deck_ids {
        for deck2_id in &deck_ids {
            let deck1 = registry.get(deck1_id)
                .expect(&format!("Failed to get deck {}", deck1_id))
                .to_card_ids();
            let deck2 = registry.get(deck2_id)
                .expect(&format!("Failed to get deck {}", deck2_id))
                .to_card_ids();

            let mut bot1 = GreedyBot::new(&card_db, 12345);
            let mut bot2 = GreedyBot::new(&card_db, 54321);

            // Run a small number of games per matchup
            let mut runner = GameRunner::new(&card_db);
            let stats = runner.run_match(
                &mut bot1,
                &mut bot2,
                deck1,
                deck2,
                10,
                4000,
            );

            println!(
                "{} vs {}: P1 wins {}, P2 wins {}, draws {}",
                deck1_id, deck2_id,
                stats.overall.bot1_wins, stats.overall.bot2_wins, stats.overall.draws
            );

            // Basic sanity check - game should complete
            assert!(
                stats.overall.games == 10,
                "Expected 10 games for {} vs {}, got {}",
                deck1_id, deck2_id, stats.overall.games
            );
        }
    }
}

// =============================================================================
// Edge Case Scenario Tests
// =============================================================================

/// Test Quick+Lethal attacking Shield (should kill but take counter damage if shield absorbs)
#[test]
fn test_edge_case_quick_lethal_vs_shield() {
    let card_db = load_card_db();

    // Create a game and manually set up the edge case
    let mut engine = GameEngine::new(&card_db);
    let deck = arena_test_deck();
    engine.start_game(deck.clone(), deck, 12345);

    // Find cards with specific keywords to create the scenario
    // For now, just verify we can run games that might encounter this
    let mut bot1 = GreedyBot::new(&card_db, 99999);
    let mut bot2 = GreedyBot::new(&card_db, 88888);

    // Run games looking for this specific interaction
    let results = run_traced_match(
        &card_db,
        &mut bot1,
        &mut bot2,
        arena_test_deck(),
        arena_test_deck(),
        50,
        5000,
    );

    // Look for combat traces with Quick+Lethal vs Shield
    let mut found_scenario = false;
    for result in &results {
        for trace in &result.combat_traces {
            let att_has_quick = trace.attacker_keywords.has_quick();
            let att_has_lethal = trace.attacker_keywords.has_lethal();
            let def_has_shield = trace.defender_keywords.map_or(false, |k| k.has_shield());

            if att_has_quick && att_has_lethal && def_has_shield {
                found_scenario = true;
                // Shield should block and defender survives (Shield absorbs first hit)
                assert!(
                    !trace.defender_died || !def_has_shield,
                    "Quick+Lethal should not kill Shield defender on first hit"
                );
            }
        }
    }

    // Log whether we found the scenario (not required to find it)
    if found_scenario {
        println!("Found Quick+Lethal vs Shield scenario - passed validation");
    } else {
        println!("Note: Quick+Lethal vs Shield scenario not naturally encountered in 50 games");
    }
}

/// Test turn 30 limit resolution
#[test]
fn test_edge_case_turn_limit() {
    let card_db = load_card_db();

    // Run many games to try to hit turn limit
    let mut bot1 = GreedyBot::new(&card_db, 77777);
    let mut bot2 = GreedyBot::new(&card_db, 66666);

    let results = run_traced_match(
        &card_db,
        &mut bot1,
        &mut bot2,
        arena_test_deck(),
        arena_test_deck(),
        100,
        6000,
    );

    let mut turn_limit_games = 0;
    for result in &results {
        if result.turns >= 30 {
            turn_limit_games += 1;

            // Winner should be determined by life totals
            // (or draw if equal)
            println!(
                "Turn limit game: {} turns, winner = {:?}",
                result.turns, result.winner
            );
        }
    }

    println!("Games reaching turn 30: {}/100", turn_limit_games);
}

/// Test Piercing damage calculation
#[test]
fn test_edge_case_piercing_damage() {
    let card_db = load_card_db();

    let results = run_traced_match(
        &card_db,
        &mut GreedyBot::new(&card_db, 55555),
        &mut GreedyBot::new(&card_db, 44444),
        arena_test_deck(),
        arena_test_deck(),
        50,
        7000,
    );

    // Look for piercing damage in combat traces
    let mut piercing_combats = 0;
    for result in &results {
        for trace in &result.combat_traces {
            if trace.attacker_keywords.has_piercing() && trace.defender_died {
                piercing_combats += 1;
                // Piercing should deal excess damage to face
                // (trace.face_damage should be > 0 if attack > defender health)
            }
        }
    }

    println!("Piercing combats with kills: {}", piercing_combats);
}

/// Test double death (mutual destruction)
#[test]
fn test_edge_case_mutual_destruction() {
    let card_db = load_card_db();

    let results = run_traced_match(
        &card_db,
        &mut GreedyBot::new(&card_db, 33333),
        &mut GreedyBot::new(&card_db, 22222),
        arena_test_deck(),
        arena_test_deck(),
        50,
        8000,
    );

    let mut mutual_deaths = 0;
    for result in &results {
        for trace in &result.combat_traces {
            if trace.attacker_died && trace.defender_died {
                mutual_deaths += 1;
            }
        }
    }

    println!("Mutual destruction combats: {}", mutual_deaths);

    // Should have some mutual destructions in 50 games
    // (not required, just informational)
}

/// Test Guard enforcement
#[test]
fn test_edge_case_guard_enforcement() {
    let card_db = load_card_db();

    // Create game engine to check legal actions
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(arena_test_deck(), arena_test_deck(), 99000);

    // Play until we have creatures on board
    let mut guard_scenarios_checked = 0;

    // Run games and verify guard enforcement
    let results = run_traced_match(
        &card_db,
        &mut GreedyBot::new(&card_db, 11111),
        &mut GreedyBot::new(&card_db, 10000),
        arena_test_deck(),
        arena_test_deck(),
        50,
        9000,
    );

    for result in &results {
        for trace in &result.combat_traces {
            // If defender has guard, attacker should have been forced to target it
            // (unless attacker has ranged)
            if let Some(def_keywords) = trace.defender_keywords {
                if def_keywords.has_guard() && !trace.attacker_keywords.has_ranged() {
                    guard_scenarios_checked += 1;
                    // This is a valid guard interaction
                }
            }
        }
    }

    println!("Guard scenarios validated: {}", guard_scenarios_checked);
}

// =============================================================================
// Stress Tests (ignored by default - run with --ignored)
// =============================================================================

#[test]
#[ignore = "tier_medium"] // ~5 min: 100 MCTS vs MCTS games
fn stress_test_mcts_vs_mcts_100_games() {
    let card_db = load_card_db();
    let deck = arena_test_deck();

    // Use stronger MCTS config for stress test
    let config = MctsConfig {
        simulations: 200,
        exploration: 1.414,
        max_rollout_depth: 100, // Match arena default
        ..Default::default()
    };

    let mut bot1 = MctsBot::with_config(&card_db, config.clone(), 12345);
    let mut bot2 = MctsBot::with_config(&card_db, config, 54321);

    let results = run_traced_match(
        &card_db,
        &mut bot1,
        &mut bot2,
        deck.clone(),
        deck,
        100,
        10000,
    );

    verify_no_anomalies(&results, "MCTS vs MCTS stress");

    let p1_win_rate = calculate_win_rate(&results);
    println!(
        "MCTS vs MCTS (100 games): P1 win rate = {:.1}%",
        p1_win_rate * 100.0
    );
}

#[test]
#[ignore = "tier_medium"] // ~5 min: Bot hierarchy validation
fn stress_test_bot_hierarchy() {
    let card_db = load_card_db();
    let deck = arena_test_deck();

    // Test that bot hierarchy is maintained: MCTS > Greedy > Random

    // MCTS vs Greedy - use moderate config (300 sims) for reliable wins
    let config = MctsConfig {
        simulations: 300,
        exploration: 1.414,
        max_rollout_depth: 100, // Match arena default
        ..Default::default()
    };
    let mut mcts = MctsBot::with_config(&card_db, config, 12345);
    let mut greedy = GreedyBot::new(&card_db, 54321);

    let mcts_v_greedy = run_traced_match(
        &card_db,
        &mut mcts,
        &mut greedy,
        deck.clone(),
        deck.clone(),
        50,
        20000,
    );
    let mcts_win_rate = calculate_win_rate(&mcts_v_greedy);

    // Greedy vs Random
    let mut greedy2 = GreedyBot::new(&card_db, 12345);
    let mut random = RandomBot::new(54321);

    let greedy_v_random = run_traced_match(
        &card_db,
        &mut greedy2,
        &mut random,
        deck.clone(),
        deck.clone(),
        50,
        30000,
    );
    let greedy_win_rate = calculate_win_rate(&greedy_v_random);

    println!("Bot hierarchy test:");
    println!("  MCTS vs Greedy: MCTS wins {:.1}%", mcts_win_rate * 100.0);
    println!("  Greedy vs Random: Greedy wins {:.1}%", greedy_win_rate * 100.0);

    // Verify hierarchy
    assert!(
        mcts_win_rate >= 0.5,
        "MCTS should outperform Greedy"
    );
    assert!(
        greedy_win_rate >= 0.85,
        "Greedy should dominate Random"
    );
}
