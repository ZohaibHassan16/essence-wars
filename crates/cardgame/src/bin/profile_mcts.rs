//! MCTS Profiling - Measure where time is spent.

use std::time::Instant;

use cardgame::bots::{Bot, GreedyBot, MctsBot, MctsConfig};
use cardgame::cards::CardDatabase;
use cardgame::engine::GameEngine;
use cardgame::types::{CardId, PlayerId};

fn test_deck() -> Vec<CardId> {
    vec![
        CardId(1), CardId(1),
        CardId(3), CardId(3),
        CardId(6), CardId(6),
        CardId(8), CardId(8),
        CardId(11), CardId(11),
        CardId(12), CardId(12),
        CardId(16), CardId(16),
        CardId(20), CardId(20),
        CardId(34), CardId(34),
    ]
}

fn main() {
    let cards_path = cardgame::data_dir().join("cards/core_set");
    let card_db = CardDatabase::load_from_directory(&cards_path)
        .expect("Failed to load cards");
    let deck = test_deck();

    println!("MCTS Performance Profiling");
    println!("==========================\n");

    // Profile engine.fork()
    {
        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck.clone(), deck.clone(), 42);

        // Play a few moves to get realistic state
        let mut greedy = GreedyBot::new(&card_db, 42);
        for _ in 0..10 {
            if engine.is_game_over() { break; }
            let action = greedy.select_action_with_engine(&engine);
            let _ = engine.apply_action(action);
        }

        let iterations = 10000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = engine.fork();
        }
        let elapsed = start.elapsed();
        println!("engine.fork(): {:.2}µs per call ({} iterations)",
            elapsed.as_micros() as f64 / iterations as f64, iterations);
    }

    // Profile GreedyBot action selection
    {
        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck.clone(), deck.clone(), 42);

        let mut greedy = GreedyBot::new(&card_db, 42);

        let iterations = 10000;
        let start = Instant::now();
        for i in 0..iterations {
            greedy.select_action_with_engine(&engine);
            if i % 100 == 0 {
                greedy.reset();
            }
        }
        let elapsed = start.elapsed();
        println!("GreedyBot.select_action(): {:.2}µs per call ({} iterations)",
            elapsed.as_micros() as f64 / iterations as f64, iterations);
    }

    // Profile single rollout
    {
        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck.clone(), deck.clone(), 42);

        let iterations = 1000;
        let mut total_actions = 0u64;
        let start = Instant::now();

        for i in 0..iterations {
            let mut sim_engine = engine.fork();
            let mut greedy = GreedyBot::new(&card_db, i);
            let mut actions = 0;

            while !sim_engine.is_game_over() && actions < 100 {
                let action = greedy.select_action_with_engine(&sim_engine);
                let _ = sim_engine.apply_action(action);
                actions += 1;
            }
            total_actions += actions as u64;
        }
        let elapsed = start.elapsed();
        let avg_actions = total_actions as f64 / iterations as f64;
        println!("Full rollout: {:.2}µs per rollout, avg {:.1} actions ({} iterations)",
            elapsed.as_micros() as f64 / iterations as f64, avg_actions, iterations);
    }

    // Profile MCTS search
    {
        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck.clone(), deck.clone(), 42);

        // Play a few moves
        let mut greedy = GreedyBot::new(&card_db, 42);
        for _ in 0..5 {
            if engine.is_game_over() { break; }
            let action = greedy.select_action_with_engine(&engine);
            let _ = engine.apply_action(action);
        }

        for sims in [100, 500, 1000] {
            let config = MctsConfig {
                simulations: sims,
                exploration: 1.414,
                max_rollout_depth: 100,
                parallel_trees: 1,
                leaf_rollouts: 1,
            };

            let iterations = 10;
            let start = Instant::now();
            for i in 0..iterations {
                let mut mcts = MctsBot::with_config(&card_db, config.clone(), i);
                mcts.select_action_with_engine(&engine);
            }
            let elapsed = start.elapsed();
            println!("MCTS search ({} sims): {:.2}ms per search ({} iterations)",
                sims, elapsed.as_millis() as f64 / iterations as f64, iterations);
        }
    }

    // Profile full game
    {
        println!("\nFull Game Profiling:");

        for (name, sims) in [("MCTS-100", 100), ("MCTS-500", 500)] {
            let config = MctsConfig {
                simulations: sims,
                exploration: 1.414,
                max_rollout_depth: 100,
                parallel_trees: 1,
                leaf_rollouts: 1,
            };

            let games = 5;
            let start = Instant::now();
            let mut total_moves = 0;

            for seed in 0..games {
                let mut engine = GameEngine::new(&card_db);
                engine.start_game(deck.clone(), deck.clone(), seed);

                let mut mcts = MctsBot::with_config(&card_db, config.clone(), seed);
                let mut greedy = GreedyBot::new(&card_db, seed + 1000);

                while !engine.is_game_over() {
                    let action = if engine.current_player() == PlayerId::PLAYER_ONE {
                        total_moves += 1;
                        mcts.select_action_with_engine(&engine)
                    } else {
                        greedy.select_action_with_engine(&engine)
                    };
                    let _ = engine.apply_action(action);
                }
            }

            let elapsed = start.elapsed();
            let avg_moves = total_moves as f64 / games as f64;
            println!("{}: {:.0}ms/game, {:.1} MCTS moves/game",
                name, elapsed.as_millis() as f64 / games as f64, avg_moves);
        }
    }

    println!("\n=== Analysis ===");
    println!("If MCTS-500 takes ~300ms/game with ~15 moves:");
    println!("  → ~20ms per MCTS search (500 simulations)");
    println!("  → ~40µs per simulation");
    println!("  → Rollout dominates: fork + rollout ≈ 35-40µs");
    println!("\nParallelization targets:");
    println!("  1. Arena: Run multiple games in parallel");
    println!("  2. MCTS: Root parallelization (multiple trees)");
    println!("  3. MCTS: Leaf parallelization (parallel rollouts)");
}
