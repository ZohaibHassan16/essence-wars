//! Unit tests for bot weights.

use cardgame::bots::{BotWeights, GreedyWeights, WeightSet};

#[test]
fn test_default_weights() {
    let weights = GreedyWeights::default();
    assert!(weights.own_life > 0.0);
    assert!(weights.win_bonus > 0.0);
    assert!(weights.lose_penalty < 0.0);
}

#[test]
fn test_weight_vec_roundtrip() {
    let original = GreedyWeights::default();
    let vec = original.to_vec();
    let restored = GreedyWeights::from_vec(&vec).unwrap();

    assert!((original.own_life - restored.own_life).abs() < 0.001);
    assert!((original.win_bonus - restored.win_bonus).abs() < 0.001);
}

#[test]
fn test_bot_weights_deck_specific() {
    let mut weights = BotWeights::default();

    // Add specialist weights for aggressive deck
    let mut aggressive = WeightSet::default();
    aggressive.greedy.enemy_life_damage = 3.0; // More aggressive
    weights.deck_specific.insert("symbiote_aggro".to_string(), aggressive);

    // Default should be different
    assert!((weights.for_deck("unknown").greedy.enemy_life_damage - 2.0).abs() < 0.001);
    // Specialist should override
    assert!((weights.for_deck("symbiote_aggro").greedy.enemy_life_damage - 3.0).abs() < 0.001);
}

#[test]
fn test_param_count() {
    let weights = GreedyWeights::default();
    assert_eq!(weights.to_vec().len(), GreedyWeights::PARAM_COUNT);
    assert_eq!(GreedyWeights::bounds().len(), GreedyWeights::PARAM_COUNT);
}
