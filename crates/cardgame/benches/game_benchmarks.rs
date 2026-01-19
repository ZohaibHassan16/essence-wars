//! Benchmarks for game engine performance.
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

use cardgame::bots::{Bot, GreedyBot, MctsBot, MctsConfig, RandomBot};
use cardgame::cards::CardDatabase;
use cardgame::engine::GameEngine;
use cardgame::types::CardId;

fn test_deck() -> Vec<CardId> {
    vec![
        CardId(1), CardId(1),   // Eager Recruit x2
        CardId(3), CardId(3),   // Nimble Scout x2
        CardId(6), CardId(6),   // Frontier Ranger x2
        CardId(8), CardId(8),   // Shielded Squire x2
        CardId(11), CardId(11), // Centaur Charger x2
        CardId(12), CardId(12), // Blade Dancer x2
        CardId(16), CardId(16), // Piercing Striker x2
        CardId(20), CardId(20), // Siege Breaker x2
        CardId(34), CardId(34), // Lightning Bolt x2
    ]
}

/// Benchmark single game with random bots.
fn bench_random_game(c: &mut Criterion) {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    let deck = test_deck();

    c.bench_function("random_game", |b| {
        b.iter(|| {
            let mut engine = GameEngine::new(&card_db);
            engine.start_game(deck.clone(), deck.clone(), 42);

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
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    let deck = test_deck();

    c.bench_function("greedy_game", |b| {
        b.iter(|| {
            let mut engine = GameEngine::new(&card_db);
            engine.start_game(deck.clone(), deck.clone(), 42);

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
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    let deck = test_deck();

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck.clone(), deck.clone(), 42);

    // Play a few turns to get a more complex state
    let mut random = RandomBot::new(42);
    for _ in 0..20 {
        if engine.is_game_over() { break; }
        let state_tensor = engine.get_state_tensor();
        let legal_mask = engine.get_legal_action_mask();
        let legal_actions = engine.get_legal_actions();
        let action = random.select_action(&state_tensor, &legal_mask, &legal_actions);
        let _ = engine.apply_action(action);
    }

    c.bench_function("state_tensor", |b| {
        b.iter(|| {
            black_box(engine.get_state_tensor())
        })
    });
}

/// Benchmark legal action generation.
fn bench_legal_actions(c: &mut Criterion) {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    let deck = test_deck();

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck.clone(), deck.clone(), 42);

    // Play a few turns to get a more complex state
    let mut random = RandomBot::new(42);
    for _ in 0..20 {
        if engine.is_game_over() { break; }
        let state_tensor = engine.get_state_tensor();
        let legal_mask = engine.get_legal_action_mask();
        let legal_actions = engine.get_legal_actions();
        let action = random.select_action(&state_tensor, &legal_mask, &legal_actions);
        let _ = engine.apply_action(action);
    }

    c.bench_function("legal_actions", |b| {
        b.iter(|| {
            black_box(engine.get_legal_actions())
        })
    });
}

/// Benchmark engine fork operation.
fn bench_engine_fork(c: &mut Criterion) {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    let deck = test_deck();

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck.clone(), deck.clone(), 42);

    // Play a few turns to get a more complex state
    let mut random = RandomBot::new(42);
    for _ in 0..20 {
        if engine.is_game_over() { break; }
        let state_tensor = engine.get_state_tensor();
        let legal_mask = engine.get_legal_action_mask();
        let legal_actions = engine.get_legal_actions();
        let action = random.select_action(&state_tensor, &legal_mask, &legal_actions);
        let _ = engine.apply_action(action);
    }

    c.bench_function("engine_fork", |b| {
        b.iter(|| {
            black_box(engine.fork())
        })
    });
}

/// Benchmark MCTS with different simulation counts.
fn bench_mcts_simulations(c: &mut Criterion) {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    let deck = test_deck();

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck.clone(), deck.clone(), 42);

    // Play a few turns to get to an interesting decision point
    let mut random = RandomBot::new(42);
    for _ in 0..10 {
        if engine.is_game_over() { break; }
        let state_tensor = engine.get_state_tensor();
        let legal_mask = engine.get_legal_action_mask();
        let legal_actions = engine.get_legal_actions();
        let action = random.select_action(&state_tensor, &legal_mask, &legal_actions);
        let _ = engine.apply_action(action);
    }

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
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    let deck = test_deck();

    let mut group = c.benchmark_group("games_per_second");
    group.throughput(criterion::Throughput::Elements(10));
    group.sample_size(30);

    group.bench_function("random_10_games", |b| {
        b.iter(|| {
            for seed in 0..10u64 {
                let mut engine = GameEngine::new(&card_db);
                engine.start_game(deck.clone(), deck.clone(), seed);

                let mut random1 = RandomBot::new(seed);
                let mut random2 = RandomBot::new(seed + 1000);

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

                    let _ = engine.apply_action(action);
                    actions += 1;
                }
            }
        })
    });

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
);
criterion_main!(benches);
