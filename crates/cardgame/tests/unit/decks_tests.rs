//! Unit tests for decks module.

use cardgame::cards::CardDatabase;
use cardgame::decks::{DeckDefinition, DeckRegistry, Faction};
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
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");

    // Valid deck (using Argentum card IDs)
    let valid_deck = DeckDefinition {
        id: "valid".to_string(),
        name: "Valid".to_string(),
        description: String::new(),
        cards: vec![1000, 1000, 1001, 1001, 1002, 1002],
        tags: vec![],
    };
    assert!(valid_deck.validate(&card_db).is_ok());

    // Invalid deck (card 9999 doesn't exist)
    let invalid_deck = DeckDefinition {
        id: "invalid".to_string(),
        name: "Invalid".to_string(),
        description: String::new(),
        cards: vec![1000, 1000, 9999],
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

// === Faction Tests ===

#[test]
fn test_faction_enum() {
    // Test all_factions returns only true factions
    let factions = Faction::all_factions();
    assert_eq!(factions.len(), 3);
    assert!(factions.contains(&Faction::Argentum));
    assert!(factions.contains(&Faction::Symbiote));
    assert!(factions.contains(&Faction::Obsidion));
    assert!(!factions.contains(&Faction::Neutral));

    // Test all() includes Neutral
    let all = Faction::all();
    assert_eq!(all.len(), 4);
    assert!(all.contains(&Faction::Neutral));
}

#[test]
fn test_faction_is_true_faction() {
    assert!(Faction::Argentum.is_true_faction());
    assert!(Faction::Symbiote.is_true_faction());
    assert!(Faction::Obsidion.is_true_faction());
    assert!(!Faction::Neutral.is_true_faction());
}

#[test]
fn test_faction_parsing() {
    // Case-insensitive parsing
    assert_eq!("argentum".parse::<Faction>().unwrap(), Faction::Argentum);
    assert_eq!("ARGENTUM".parse::<Faction>().unwrap(), Faction::Argentum);
    assert_eq!("Argentum".parse::<Faction>().unwrap(), Faction::Argentum);

    assert_eq!("symbiote".parse::<Faction>().unwrap(), Faction::Symbiote);
    assert_eq!("obsidion".parse::<Faction>().unwrap(), Faction::Obsidion);

    // Neutral has multiple aliases
    assert_eq!("neutral".parse::<Faction>().unwrap(), Faction::Neutral);
    assert_eq!("freewalker".parse::<Faction>().unwrap(), Faction::Neutral);
    assert_eq!("free-walkers".parse::<Faction>().unwrap(), Faction::Neutral);

    // Invalid faction
    assert!("invalid".parse::<Faction>().is_err());
}

#[test]
fn test_faction_as_tag() {
    assert_eq!(Faction::Argentum.as_tag(), "argentum");
    assert_eq!(Faction::Symbiote.as_tag(), "symbiote");
    assert_eq!(Faction::Obsidion.as_tag(), "obsidion");
    assert_eq!(Faction::Neutral.as_tag(), "neutral");
}

#[test]
fn test_deck_faction_detection() {
    // Argentum deck
    let argentum_deck = DeckDefinition {
        id: "argentum_test".to_string(),
        name: "Argentum Test".to_string(),
        description: String::new(),
        cards: vec![1000, 1001, 1002],
        tags: vec!["control".to_string(), "argentum".to_string()],
    };
    assert_eq!(argentum_deck.faction(), Some(Faction::Argentum));
    assert!(argentum_deck.is_faction_deck());
    assert!(argentum_deck.is_faction(Faction::Argentum));
    assert!(!argentum_deck.is_faction(Faction::Symbiote));

    // Symbiote deck
    let symbiote_deck = DeckDefinition {
        id: "symbiote_test".to_string(),
        name: "Symbiote Test".to_string(),
        description: String::new(),
        cards: vec![2003, 2004, 2005],
        tags: vec!["aggro".to_string(), "symbiote".to_string()],
    };
    assert_eq!(symbiote_deck.faction(), Some(Faction::Symbiote));
    assert!(symbiote_deck.is_faction(Faction::Symbiote));

    // No faction tag
    let neutral_deck = DeckDefinition {
        id: "mixed_test".to_string(),
        name: "Mixed Test".to_string(),
        description: String::new(),
        cards: vec![4000, 4001, 4002],
        tags: vec!["aggro".to_string()],
    };
    assert_eq!(neutral_deck.faction(), None);
    assert!(!neutral_deck.is_faction_deck());
}

#[test]
fn test_deck_specialist_compatibility() {
    let argentum_deck = DeckDefinition {
        id: "architect_fortify".to_string(),
        name: "Argentum Control".to_string(),
        description: String::new(),
        cards: vec![1000, 1001, 1002],
        tags: vec!["argentum".to_string()],
    };

    // Argentum specialist can play Argentum decks
    assert!(argentum_deck.is_compatible_with_specialist(Faction::Argentum));

    // Symbiote specialist cannot play Argentum decks
    assert!(!argentum_deck.is_compatible_with_specialist(Faction::Symbiote));

    // Obsidion specialist cannot play Argentum decks
    assert!(!argentum_deck.is_compatible_with_specialist(Faction::Obsidion));
}

#[test]
fn test_registry_faction_filtering() {
    let mut registry = DeckRegistry::new();

    let argentum_deck = DeckDefinition {
        id: "argentum1".to_string(),
        name: "Argentum 1".to_string(),
        description: String::new(),
        cards: vec![1000],
        tags: vec!["argentum".to_string()],
    };

    let symbiote_deck = DeckDefinition {
        id: "symbiote1".to_string(),
        name: "Symbiote 1".to_string(),
        description: String::new(),
        cards: vec![2003],
        tags: vec!["symbiote".to_string()],
    };

    let neutral_deck = DeckDefinition {
        id: "neutral1".to_string(),
        name: "Neutral 1".to_string(),
        description: String::new(),
        cards: vec![4000],
        tags: vec!["aggro".to_string()],
    };

    registry.add(argentum_deck).unwrap();
    registry.add(symbiote_deck).unwrap();
    registry.add(neutral_deck).unwrap();

    // Test faction filtering
    let argentum_decks = registry.decks_for_faction(Faction::Argentum);
    assert_eq!(argentum_decks.len(), 1);
    assert_eq!(argentum_decks[0].id, "argentum1");

    let symbiote_decks = registry.decks_for_faction(Faction::Symbiote);
    assert_eq!(symbiote_decks.len(), 1);

    let obsidion_decks = registry.decks_for_faction(Faction::Obsidion);
    assert_eq!(obsidion_decks.len(), 0);

    // Test faction_decks and neutral_decks
    let faction_decks = registry.faction_decks();
    assert_eq!(faction_decks.len(), 2);

    let neutral_decks = registry.neutral_decks();
    assert_eq!(neutral_decks.len(), 1);
    assert_eq!(neutral_decks[0].id, "neutral1");

    // Test decks_for_specialist
    let specialist_decks = registry.decks_for_specialist(Faction::Argentum);
    assert_eq!(specialist_decks.len(), 1);
    assert_eq!(specialist_decks[0].id, "argentum1");
}

#[test]
fn test_real_deck_factions() {
    // Test that our actual faction decks are detected correctly
    let registry = DeckRegistry::load_from_directory(cardgame::data_dir().join("decks"))
        .expect("Failed to load decks");

    // Check architect_fortify exists and has correct faction
    if let Some(deck) = registry.get("architect_fortify") {
        assert_eq!(deck.faction(), Some(Faction::Argentum));
        assert!(deck.is_compatible_with_specialist(Faction::Argentum));
    }

    // Check broodmother_swarm exists and has correct faction
    if let Some(deck) = registry.get("broodmother_swarm") {
        assert_eq!(deck.faction(), Some(Faction::Symbiote));
        assert!(deck.is_compatible_with_specialist(Faction::Symbiote));
    }

    // Check archon_burst exists and has correct faction
    if let Some(deck) = registry.get("archon_burst") {
        assert_eq!(deck.faction(), Some(Faction::Obsidion));
        assert!(deck.is_compatible_with_specialist(Faction::Obsidion));
    }
}
