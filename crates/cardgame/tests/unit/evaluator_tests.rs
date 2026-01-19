//! Unit tests for tuning evaluator.

use cardgame::bots::GreedyWeights;
use cardgame::cards::CardDatabase;
use cardgame::tuning::{Evaluator, EvaluatorConfig, TuningMode};

#[test]
fn test_evaluate_vs_random() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    let config = EvaluatorConfig {
        games_per_eval: 10,
        mode: TuningMode::VsRandom,
        seed: 42,
        max_actions: 500,
        parallel: false, // Sequential for test stability
        mcts_sims: 100,
    };

    let mut evaluator = Evaluator::new(&card_db, config);

    // Use default weights
    let weights = GreedyWeights::default();
    let result = evaluator.evaluate(&weights.to_vec().iter().map(|&x| x as f64).collect::<Vec<_>>());

    // Should win most games against random
    assert!(result.win_rate > 0.5, "Should beat random, got {:.2}%", result.win_rate * 100.0);
    assert!(result.games == 10);
}

#[test]
fn test_evaluate_vs_greedy() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    let config = EvaluatorConfig {
        games_per_eval: 10,
        mode: TuningMode::VsGreedy,
        seed: 42,
        max_actions: 500,
        parallel: false,
        mcts_sims: 100,
    };

    let mut evaluator = Evaluator::new(&card_db, config);

    // Default vs default should be close to 50%
    let weights = GreedyWeights::default();
    let result = evaluator.evaluate(&weights.to_vec().iter().map(|&x| x as f64).collect::<Vec<_>>());

    // Mirror match has first-player advantage (typically 60-90% for P1)
    // We're testing that the game completes and returns a reasonable result
    assert!(result.win_rate >= 0.1 && result.win_rate <= 0.95,
            "Mirror match win rate out of expected range, got {:.2}%", result.win_rate * 100.0);
}

#[test]
fn test_bad_weights_lose() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    let config = EvaluatorConfig {
        games_per_eval: 10,
        mode: TuningMode::VsGreedy,
        seed: 42,
        max_actions: 500,
        parallel: false,
        mcts_sims: 100,
    };

    let mut evaluator = Evaluator::new(&card_db, config);

    // Terrible weights - negative for good things, positive for bad
    let bad_weights = GreedyWeights {
        own_life: -1.0,           // Penalize own life
        enemy_life_damage: -1.0,  // Penalize damaging enemy
        own_creature_attack: -1.0,
        own_creature_health: -1.0,
        enemy_creature_attack: 1.0,
        enemy_creature_health: 1.0,
        creature_count: -1.0,
        board_advantage: -1.0,
        cards_in_hand: -1.0,
        action_points: 0.0,
        keyword_guard: -1.0,
        keyword_lethal: -1.0,
        keyword_lifesteal: -1.0,
        keyword_rush: -1.0,
        keyword_ranged: -1.0,
        keyword_piercing: -1.0,
        keyword_shield: -1.0,
        keyword_quick: -1.0,
        keyword_ephemeral: 1.0,   // Reward ephemeral (bad!)
        keyword_regenerate: -1.0, // Penalize regenerate
        keyword_stealth: -1.0,    // Penalize stealth
        keyword_charge: -1.0,     // Penalize charge
        keyword_frenzy: -1.0,     // Penalize frenzy
        keyword_volatile: -1.0,   // Penalize volatile
        keyword_fortify: -1.0,    // Penalize fortify
        keyword_ward: -1.0,       // Penalize ward
        win_bonus: -1000.0,       // Penalize winning!
        lose_penalty: 1000.0,     // Reward losing!
    };

    let result = evaluator.evaluate(&bad_weights.to_vec().iter().map(|&x| x as f64).collect::<Vec<_>>());

    // Should lose most games with these terrible weights
    assert!(result.win_rate < 0.5, "Bad weights should lose, got {:.2}%", result.win_rate * 100.0);
}
