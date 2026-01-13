//! Unit tests for decks module.

use cardgame::cards::CardDatabase;
use cardgame::decks::{DeckDefinition, DeckRegistry};
use cardgame::types::CardId;

#[test]
fn test_deck_definition() {
    let deck = DeckDefinition {
        id: "test".to_string(),
        name: "Test Deck".to_string(),
        description: "A test deck".to_string(),
        cards: vec![1, 1, 2, 2, 3],
        tags: vec!["aggro".to_string()],
    };

    assert_eq!(deck.size(), 5);
    assert!(deck.has_tag("aggro"));
    assert!(deck.has_tag("AGGRO"));
    assert!(!deck.has_tag("control"));

    let card_ids = deck.to_card_ids();
    assert_eq!(card_ids.len(), 5);
    assert_eq!(card_ids[0], CardId(1));
}

#[test]
fn test_deck_validation() {
    let card_db = CardDatabase::load_from_directory("data/cards")
        .expect("Failed to load cards");

    // Valid deck
    let valid_deck = DeckDefinition {
        id: "valid".to_string(),
        name: "Valid".to_string(),
        description: String::new(),
        cards: vec![1, 1, 2, 2, 3, 3],
        tags: vec![],
    };
    assert!(valid_deck.validate(&card_db).is_ok());

    // Invalid deck (card 9999 doesn't exist)
    let invalid_deck = DeckDefinition {
        id: "invalid".to_string(),
        name: "Invalid".to_string(),
        description: String::new(),
        cards: vec![1, 1, 9999],
        tags: vec![],
    };
    assert!(invalid_deck.validate(&card_db).is_err());
}

#[test]
fn test_deck_registry() {
    let mut registry = DeckRegistry::new();

    let deck1 = DeckDefinition {
        id: "deck1".to_string(),
        name: "Deck 1".to_string(),
        description: String::new(),
        cards: vec![1, 2, 3],
        tags: vec!["aggro".to_string()],
    };

    let deck2 = DeckDefinition {
        id: "deck2".to_string(),
        name: "Deck 2".to_string(),
        description: String::new(),
        cards: vec![4, 5, 6],
        tags: vec!["control".to_string()],
    };

    registry.add(deck1).unwrap();
    registry.add(deck2).unwrap();

    assert_eq!(registry.len(), 2);
    assert!(registry.get("deck1").is_some());
    assert!(registry.get("deck2").is_some());
    assert!(registry.get("deck3").is_none());

    let aggro_decks = registry.decks_with_tag("aggro");
    assert_eq!(aggro_decks.len(), 1);
    assert_eq!(aggro_decks[0].id, "deck1");
}

#[test]
fn test_duplicate_id_rejected() {
    let mut registry = DeckRegistry::new();

    let deck1 = DeckDefinition {
        id: "same".to_string(),
        name: "Deck 1".to_string(),
        description: String::new(),
        cards: vec![1],
        tags: vec![],
    };

    let deck2 = DeckDefinition {
        id: "same".to_string(),
        name: "Deck 2".to_string(),
        description: String::new(),
        cards: vec![2],
        tags: vec![],
    };

    assert!(registry.add(deck1).is_ok());
    assert!(registry.add(deck2).is_err());
}
