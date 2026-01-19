//! Unit tests for config module.

use cardgame::config::{balance, GameConfig};

#[test]
fn test_vanilla_stats_formula() {
    assert_eq!(balance::vanilla_stats(1), 3); // 1-cost = 3 stats
    assert_eq!(balance::vanilla_stats(3), 7); // 3-cost = 7 stats
    assert_eq!(balance::vanilla_stats(5), 11); // 5-cost = 11 stats
}

#[test]
fn test_default_config() {
    let config = GameConfig::default();
    assert_eq!(config.deck_size, 20);
    assert_eq!(config.turn_limit, 30);
}

#[test]
fn test_standard_config() {
    let config = GameConfig::standard();
    assert_eq!(config.deck_size, 30);
}
