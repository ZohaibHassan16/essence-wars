//! Tests for game mode system (Attrition vs Essence Duel).

use cardgame::cards::CardDatabase;
use cardgame::core::config::game;
use cardgame::core::state::{GameMode, GameResult, WinReason};
use cardgame::engine::GameEngine;
use cardgame::types::CardId;

fn load_test_db() -> CardDatabase {
    CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load card database")
}

fn get_test_deck() -> Vec<CardId> {
    // Simple test deck with low-cost creatures
    vec![
        CardId(2000), CardId(2000), // Spore Crawler x2
        CardId(2001), CardId(2001), // Feral Striker x2
        CardId(2002), CardId(2002), // Venomfang x2
        CardId(2003), CardId(2003), // Swarm Runner x2
        CardId(2004), CardId(2004), // Hive Warrior x2
        CardId(2005), CardId(2005), // Brood Guardian x2
        CardId(2006), CardId(2006), // Evolution Master x2
        CardId(2007), CardId(2007), // Apex Predator x2
        CardId(2008), CardId(2008), // Swarm Surge x2
        CardId(2009), CardId(2009), // Feeding Frenzy x2
        CardId(4000), CardId(4000), // Hedge Wizard x2
        CardId(4001), CardId(4001), // Street Rat x2
        CardId(4002), CardId(4002), // Wandering Merchant x2
        CardId(4003), CardId(4003), // Caravan Guard x2
        CardId(4004), CardId(4004), // Free-Walker Scout x2
    ]
}

#[test]
fn test_game_mode_default_is_attrition() {
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);
    let deck = get_test_deck();

    engine.start_game(deck.clone(), deck.clone(), 12345);

    assert_eq!(engine.state.game_mode, GameMode::Attrition);
}

#[test]
fn test_start_game_with_mode_sets_mode() {
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);
    let deck = get_test_deck();

    engine.start_game_with_mode(deck.clone(), deck.clone(), 12345, GameMode::EssenceDuel);

    assert_eq!(engine.state.game_mode, GameMode::EssenceDuel);
}

#[test]
fn test_essence_duel_vp_threshold_is_50() {
    assert_eq!(game::VICTORY_POINTS_THRESHOLD, 50);
}

#[test]
fn test_essence_duel_vp_victory_at_threshold() {
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);
    let deck = get_test_deck();

    engine.start_game_with_mode(deck.clone(), deck.clone(), 12345, GameMode::EssenceDuel);

    // Simulate player 1 dealing exactly 50 face damage
    engine.state.players[0].total_damage_dealt = 50;
    engine.check_victory_points_victory();

    assert!(engine.is_game_over());
    match engine.state.result {
        Some(GameResult::Win { winner, reason }) => {
            assert_eq!(winner.index(), 0); // Player 1 wins
            assert_eq!(reason, WinReason::VictoryPointsReached);
        }
        _ => panic!("Expected VP victory for player 1"),
    }
}

#[test]
fn test_essence_duel_vp_victory_above_threshold() {
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);
    let deck = get_test_deck();

    engine.start_game_with_mode(deck.clone(), deck.clone(), 12345, GameMode::EssenceDuel);

    // Simulate player 2 dealing more than 50 face damage
    engine.state.players[1].total_damage_dealt = 55;
    engine.check_victory_points_victory();

    assert!(engine.is_game_over());
    match engine.state.result {
        Some(GameResult::Win { winner, reason }) => {
            assert_eq!(winner.index(), 1); // Player 2 wins
            assert_eq!(reason, WinReason::VictoryPointsReached);
        }
        _ => panic!("Expected VP victory for player 2"),
    }
}

#[test]
fn test_essence_duel_no_victory_below_threshold() {
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);
    let deck = get_test_deck();

    engine.start_game_with_mode(deck.clone(), deck.clone(), 12345, GameMode::EssenceDuel);

    // Set damage just below threshold
    engine.state.players[0].total_damage_dealt = 49;
    engine.state.players[1].total_damage_dealt = 49;
    engine.check_victory_points_victory();

    assert!(!engine.is_game_over());
}

#[test]
fn test_attrition_ignores_vp() {
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);
    let deck = get_test_deck();

    // Use default Attrition mode
    engine.start_game(deck.clone(), deck.clone(), 12345);

    // Even with 100 damage dealt, game doesn't end via VP in Attrition mode
    engine.state.players[0].total_damage_dealt = 100;
    engine.check_victory_points_victory();

    assert!(!engine.is_game_over());
}

#[test]
fn test_life_victory_takes_precedence_in_essence_duel() {
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);
    let deck = get_test_deck();

    engine.start_game_with_mode(deck.clone(), deck.clone(), 12345, GameMode::EssenceDuel);

    // Reduce player 2's life to 0
    engine.state.players[1].life = 0;
    engine.check_life_victory();

    assert!(engine.is_game_over());
    match engine.state.result {
        Some(GameResult::Win { winner, reason }) => {
            assert_eq!(winner.index(), 0); // Player 1 wins
            assert_eq!(reason, WinReason::LifeReachedZero);
        }
        _ => panic!("Expected life-based victory"),
    }
}

#[test]
fn test_vp_check_doesnt_override_existing_result() {
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);
    let deck = get_test_deck();

    engine.start_game_with_mode(deck.clone(), deck.clone(), 12345, GameMode::EssenceDuel);

    // Player 2 has life victory first
    engine.state.players[1].life = 0;
    engine.check_life_victory();

    // Now player 2 also has VP (shouldn't override)
    engine.state.players[1].total_damage_dealt = 50;
    engine.check_victory_points_victory();

    // Life victory should still be the reason
    match engine.state.result {
        Some(GameResult::Win { reason, .. }) => {
            assert_eq!(reason, WinReason::LifeReachedZero);
        }
        _ => panic!("Expected life-based victory"),
    }
}

#[test]
fn test_game_mode_preserved_across_turns() {
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);
    let deck = get_test_deck();

    engine.start_game_with_mode(deck.clone(), deck.clone(), 12345, GameMode::EssenceDuel);

    // End turn a few times
    for _ in 0..3 {
        engine.state.active_player_state_mut().action_points = 0;
        let _ = engine.apply_action(cardgame::actions::Action::EndTurn);
    }

    // Mode should still be Essence Duel
    assert_eq!(engine.state.game_mode, GameMode::EssenceDuel);
}
