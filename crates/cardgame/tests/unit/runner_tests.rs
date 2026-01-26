//! Unit tests for arena runner.

use cardgame::arena::GameRunner;
use cardgame::bots::RandomBot;
use cardgame::cards::CardDatabase;
use cardgame::decks::DeckDefinition;

/// Default commander for tests (The High Artificer).
const DEFAULT_COMMANDER: u16 = 5000;

fn test_deck() -> DeckDefinition {
    // Simple deck with starter set cards
    let valid_ids = [1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 15, 32, 33, 40];
    let cards: Vec<u16> = (0..20)
        .map(|i| valid_ids[i % valid_ids.len()] as u16)
        .collect();

    DeckDefinition {
        id: "test_deck".to_string(),
        name: "Test Deck".to_string(),
        description: String::new(),
        playstyle: String::new(),
        commander: DEFAULT_COMMANDER,
        cards,
        tags: vec![],
    }
}

#[test]
fn test_run_single_game() {
    let card_db = CardDatabase::load_with_commanders(
        cardgame::data_dir().join("cards/core_set"),
        cardgame::data_dir().join("commanders"),
    )
        .expect("Failed to load cards");

    let mut runner = GameRunner::new(&card_db);
    let mut bot1 = RandomBot::new(42);
    let mut bot2 = RandomBot::new(43);

    let deck = test_deck();
    let result = runner.run_game(
        &mut bot1,
        &mut bot2,
        &deck,
        &deck,
        12345,
    );

    assert!(result.turns > 0);
    assert_eq!(result.seed, 12345);
    assert!(!result.actions.is_empty());
}

#[test]
fn test_game_determinism() {
    let card_db = CardDatabase::load_with_commanders(
        cardgame::data_dir().join("cards/core_set"),
        cardgame::data_dir().join("commanders"),
    )
        .expect("Failed to load cards");

    let mut runner = GameRunner::new(&card_db);
    let deck = test_deck();

    // Run same game twice
    let mut bot1a = RandomBot::new(100);
    let mut bot2a = RandomBot::new(200);
    let result1 = runner.run_game(
        &mut bot1a,
        &mut bot2a,
        &deck,
        &deck,
        12345,
    );

    let mut bot1b = RandomBot::new(100);
    let mut bot2b = RandomBot::new(200);
    let result2 = runner.run_game(
        &mut bot1b,
        &mut bot2b,
        &deck,
        &deck,
        12345,
    );

    assert_eq!(result1.winner, result2.winner);
    assert_eq!(result1.turns, result2.turns);
    assert_eq!(result1.actions.len(), result2.actions.len());
}

#[test]
fn test_run_match() {
    let card_db = CardDatabase::load_with_commanders(
        cardgame::data_dir().join("cards/core_set"),
        cardgame::data_dir().join("commanders"),
    )
        .expect("Failed to load cards");

    let mut runner = GameRunner::new(&card_db);
    let mut bot1 = RandomBot::new(42);
    let mut bot2 = RandomBot::new(43);
    let deck = test_deck();

    let stats = runner.run_match(
        &mut bot1,
        &mut bot2,
        &deck,
        &deck,
        10,
        1000,
    );

    assert_eq!(stats.overall.games, 10);
    assert_eq!(
        stats.overall.bot1_wins + stats.overall.bot2_wins + stats.overall.draws,
        10
    );
}
