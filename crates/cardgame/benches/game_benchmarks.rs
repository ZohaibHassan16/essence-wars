//! Benchmarks for game engine performance.
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use cardgame::bots::{Bot, GreedyBot, MctsBot, MctsConfig, RandomBot};
use cardgame::cards::CardDatabase;
use cardgame::decks::DeckRegistry;
use cardgame::engine::GameEngine;

/// Load card database with commanders.
fn load_card_db() -> CardDatabase {
    CardDatabase::load_with_commanders(
        cardgame::data_dir().join("cards/core_set"),
        cardgame::data_dir().join("commanders"),
    )
    .expect("Failed to load cards")
}

/// Load deck registry.
fn load_decks() -> DeckRegistry {
    DeckRegistry::load_from_directory(cardgame::data_dir().join("decks"))
        .expect("Failed to load decks")
}

/// Advance the game to a state with multiple legal actions (at least `min_actions`).
/// This ensures MCTS benchmarks actually run simulations instead of early-exiting.
fn advance_to_decision_point(engine: &mut GameEngine, min_actions: usize, seed: u64) {
    let mut random = RandomBot::new(seed);
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 200;

    while !engine.is_game_over() && iterations < MAX_ITERATIONS {
        let legal_actions = engine.get_legal_actions();

        // Stop if we have enough legal actions for a meaningful decision
        if legal_actions.len() >= min_actions {
            break;
        }

        // Play a random action
        let state_tensor = engine.get_state_tensor();
        let legal_mask = engine.get_legal_action_mask();
        let action = random.select_action(&state_tensor, &legal_mask, &legal_actions);
        let _ = engine.apply_action(action);
        iterations += 1;
    }
}

/// Benchmark single game with random bots.
fn bench_random_game(c: &mut Criterion) {
    let card_db = load_card_db();
    let deck_registry = load_decks();
    let deck = deck_registry
        .get("architect_fortify")
        .expect("Deck should exist");

    c.bench_function("random_game", |b| {
        b.iter(|| {
            let mut engine = GameEngine::new(&card_db);
            engine.start_game(deck, deck, 42).unwrap();

            let mut random1 = RandomBot::new(42);
            let mut random2 = RandomBot::new(43);

            let mut actions = 0;
            while !engine.is_game_over() && actions < 500 {
                let state_tensor = engine.get_state_tensor();
                let legal_mask = engine.get_legal_action_mask();
                let legal_actions = engine.get_legal_actions();

                let action = if engine.current_player() == cardgame::types::PlayerId::PLAYER_ONE {
                    random1.select_action(&state_tensor, &legal_mask, &legal_actions)
                } else {
                    random2.select_action(&state_tensor, &legal_mask, &legal_actions)
                };

                let _ = engine.apply_action(black_box(action));
                actions += 1;
            }
            engine.winner()
        })
    });
}

/// Benchmark single game with greedy bots.
fn bench_greedy_game(c: &mut Criterion) {
    let card_db = load_card_db();
    let deck_registry = load_decks();
    let deck = deck_registry
        .get("architect_fortify")
        .expect("Deck should exist");

    c.bench_function("greedy_game", |b| {
        b.iter(|| {
            let mut engine = GameEngine::new(&card_db);
            engine.start_game(deck, deck, 42).unwrap();

            let mut greedy1 = GreedyBot::new(&card_db, 42);
            let mut greedy2 = GreedyBot::new(&card_db, 43);

            let mut actions = 0;
            while !engine.is_game_over() && actions < 500 {
                let action = if engine.current_player() == cardgame::types::PlayerId::PLAYER_ONE {
                    greedy1.select_action_with_engine(&engine)
                } else {
                    greedy2.select_action_with_engine(&engine)
                };

                let _ = engine.apply_action(black_box(action));
                actions += 1;
            }
            engine.winner()
        })
    });
}

/// Benchmark state tensor generation.
fn bench_state_tensor(c: &mut Criterion) {
    let card_db = load_card_db();
    let deck_registry = load_decks();
    let deck = deck_registry
        .get("architect_fortify")
        .expect("Deck should exist");

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck, deck, 42).unwrap();

    // Play a few turns to get a more complex state
    let mut random = RandomBot::new(42);
    for _ in 0..20 {
        if engine.is_game_over() {
            break;
        }
        let state_tensor = engine.get_state_tensor();
        let legal_mask = engine.get_legal_action_mask();
        let legal_actions = engine.get_legal_actions();
        let action = random.select_action(&state_tensor, &legal_mask, &legal_actions);
        let _ = engine.apply_action(action);
    }

    c.bench_function("state_tensor", |b| {
        b.iter(|| black_box(engine.get_state_tensor()))
    });
}

/// Benchmark legal action generation.
fn bench_legal_actions(c: &mut Criterion) {
    let card_db = load_card_db();
    let deck_registry = load_decks();
    let deck = deck_registry
        .get("architect_fortify")
        .expect("Deck should exist");

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck, deck, 42).unwrap();

    // Play a few turns to get a more complex state
    let mut random = RandomBot::new(42);
    for _ in 0..20 {
        if engine.is_game_over() {
            break;
        }
        let state_tensor = engine.get_state_tensor();
        let legal_mask = engine.get_legal_action_mask();
        let legal_actions = engine.get_legal_actions();
        let action = random.select_action(&state_tensor, &legal_mask, &legal_actions);
        let _ = engine.apply_action(action);
    }

    c.bench_function("legal_actions", |b| {
        b.iter(|| black_box(engine.get_legal_actions()))
    });
}

/// Benchmark engine fork operation.
fn bench_engine_fork(c: &mut Criterion) {
    let card_db = load_card_db();
    let deck_registry = load_decks();
    let deck = deck_registry
        .get("architect_fortify")
        .expect("Deck should exist");

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck, deck, 42).unwrap();

    // Play a few turns to get a more complex state
    let mut random = RandomBot::new(42);
    for _ in 0..20 {
        if engine.is_game_over() {
            break;
        }
        let state_tensor = engine.get_state_tensor();
        let legal_mask = engine.get_legal_action_mask();
        let legal_actions = engine.get_legal_actions();
        let action = random.select_action(&state_tensor, &legal_mask, &legal_actions);
        let _ = engine.apply_action(action);
    }

    c.bench_function("engine_fork", |b| b.iter(|| black_box(engine.fork())));
}

/// Benchmark MCTS with different simulation counts.
fn bench_mcts_simulations(c: &mut Criterion) {
    let card_db = load_card_db();
    let deck_registry = load_decks();
    let deck = deck_registry
        .get("architect_fortify")
        .expect("Deck should exist");

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck, deck, 42).unwrap();

    // Advance to a state with at least 5 legal actions so MCTS actually runs
    advance_to_decision_point(&mut engine, 5, 42);

    let mut group = c.benchmark_group("mcts_simulations");
    group.sample_size(20); // Fewer samples for slower benchmarks

    for sims in [50, 100, 200].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(sims), sims, |b, &sims| {
            let config = MctsConfig {
                simulations: sims,
                exploration: 1.414,
                max_rollout_depth: 50,
                parallel_trees: 1,
                leaf_rollouts: 1,
            };

            b.iter(|| {
                let mut mcts = MctsBot::with_config(&card_db, config.clone(), 42);
                black_box(mcts.select_action_with_engine(&engine))
            })
        });
    }
    group.finish();
}

/// Benchmark throughput: games per second with random bots.
fn bench_games_per_second(c: &mut Criterion) {
    let card_db = load_card_db();
    let deck_registry = load_decks();
    let deck = deck_registry
        .get("architect_fortify")
        .expect("Deck should exist");

    let mut group = c.benchmark_group("games_per_second");
    group.throughput(criterion::Throughput::Elements(10));
    group.sample_size(30);

    group.bench_function("random_10_games", |b| {
        b.iter(|| {
            for seed in 0..10u64 {
                let mut engine = GameEngine::new(&card_db);
                engine.start_game(deck, deck, seed).unwrap();

                let mut random1 = RandomBot::new(seed);
                let mut random2 = RandomBot::new(seed + 1000);

                let mut actions = 0;
                while !engine.is_game_over() && actions < 500 {
                    let state_tensor = engine.get_state_tensor();
                    let legal_mask = engine.get_legal_action_mask();
                    let legal_actions = engine.get_legal_actions();

                    let action = if engine.current_player() == cardgame::types::PlayerId::PLAYER_ONE
                    {
                        random1.select_action(&state_tensor, &legal_mask, &legal_actions)
                    } else {
                        random2.select_action(&state_tensor, &legal_mask, &legal_actions)
                    };

                    let _ = engine.apply_action(action);
                    actions += 1;
                }
            }
        })
    });

    group.finish();
}

/// Benchmark state cloning performance at different game stages.
/// Critical for MCTS which clones state thousands of times per decision.
fn bench_state_cloning(c: &mut Criterion) {
    let card_db = load_card_db();
    let deck_registry = load_decks();
    let deck = deck_registry
        .get("architect_fortify")
        .expect("Deck should exist");

    let mut group = c.benchmark_group("state_cloning");

    // Early game state (turn 1-2)
    let mut early_engine = GameEngine::new(&card_db);
    early_engine.start_game(deck, deck, 42).unwrap();
    let mut random = RandomBot::new(42);
    for _ in 0..5 {
        if early_engine.is_game_over() {
            break;
        }
        let state_tensor = early_engine.get_state_tensor();
        let legal_mask = early_engine.get_legal_action_mask();
        let legal_actions = early_engine.get_legal_actions();
        let action = random.select_action(&state_tensor, &legal_mask, &legal_actions);
        let _ = early_engine.apply_action(action);
    }

    // Mid game state (turn 4-6)
    let mut mid_engine = GameEngine::new(&card_db);
    mid_engine.start_game(deck, deck, 43).unwrap();
    let mut random = RandomBot::new(43);
    for _ in 0..20 {
        if mid_engine.is_game_over() {
            break;
        }
        let state_tensor = mid_engine.get_state_tensor();
        let legal_mask = mid_engine.get_legal_action_mask();
        let legal_actions = mid_engine.get_legal_actions();
        let action = random.select_action(&state_tensor, &legal_mask, &legal_actions);
        let _ = mid_engine.apply_action(action);
    }

    // Late game state (turn 8+)
    let mut late_engine = GameEngine::new(&card_db);
    late_engine.start_game(deck, deck, 44).unwrap();
    let mut random = RandomBot::new(44);
    for _ in 0..40 {
        if late_engine.is_game_over() {
            break;
        }
        let state_tensor = late_engine.get_state_tensor();
        let legal_mask = late_engine.get_legal_action_mask();
        let legal_actions = late_engine.get_legal_actions();
        let action = random.select_action(&state_tensor, &legal_mask, &legal_actions);
        let _ = late_engine.apply_action(action);
    }

    group.bench_function("early_game", |b| {
        b.iter(|| black_box(early_engine.fork()))
    });

    group.bench_function("mid_game", |b| {
        b.iter(|| black_box(mid_engine.fork()))
    });

    group.bench_function("late_game", |b| {
        b.iter(|| black_box(late_engine.fork()))
    });

    group.finish();
}

/// Benchmark MCTS parallel scaling with different parallel_trees values.
/// Validates parallelization benefit for multi-core systems.
fn bench_mcts_parallel_scaling(c: &mut Criterion) {
    use criterion::BatchSize;

    let card_db = load_card_db();
    let deck_registry = load_decks();
    let deck = deck_registry
        .get("architect_fortify")
        .expect("Deck should exist");

    let mut base_engine = GameEngine::new(&card_db);
    base_engine.start_game(deck, deck, 42).unwrap();

    // Advance to a state with at least 5 legal actions so MCTS actually runs
    advance_to_decision_point(&mut base_engine, 5, 42);

    let mut group = c.benchmark_group("mcts_parallel_scaling");
    group.sample_size(10); // Fewer samples since these are slower

    // Fixed simulations budget, vary parallelization
    let total_simulations = 200;

    for parallel_trees in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(parallel_trees),
            parallel_trees,
            |b, &parallel_trees| {
                let config = MctsConfig {
                    simulations: total_simulations,
                    exploration: 1.414,
                    max_rollout_depth: 50,
                    parallel_trees,
                    leaf_rollouts: 1,
                };

                b.iter_batched(
                    || {
                        // Setup: create bot (not timed)
                        (
                            MctsBot::with_config(&card_db, config.clone(), 42),
                            base_engine.fork(),
                        )
                    },
                    |(mut mcts, engine): (MctsBot<'_>, GameEngine<'_>)| {
                        // Benchmark: actual MCTS decision (timed)
                        black_box(mcts.select_action_with_engine(&engine))
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_random_game,
    bench_greedy_game,
    bench_state_tensor,
    bench_legal_actions,
    bench_engine_fork,
    bench_mcts_simulations,
    bench_games_per_second,
    bench_state_cloning,
    bench_mcts_parallel_scaling,
);
criterion_main!(benches);
