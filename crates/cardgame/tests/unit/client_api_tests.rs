//! Tests for the client_api module.

use std::sync::Arc;
use cardgame::cards::CardDatabase;
use cardgame::client_api::{GameClient, GameClientBuilder, GameEvent, StateSnapshot};
use cardgame::decks::DeckRegistry;
use cardgame::state::GameMode;
use cardgame::types::PlayerId;

/// Helper to load card database.
fn load_card_db() -> Arc<CardDatabase> {
    Arc::new(
        CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
            .expect("Failed to load cards")
    )
}

/// Helper to load deck registry.
fn load_decks() -> DeckRegistry {
    DeckRegistry::load_from_directory(cardgame::data_dir().join("decks"))
        .expect("Failed to load decks")
}

/// Helper to create a GameClient with loaded card database.
fn create_test_client() -> (GameClient, Vec<cardgame::types::CardId>, Vec<cardgame::types::CardId>) {
    let db = load_card_db();
    let registry = load_decks();

    let deck1 = registry.get("architect_fortify")
        .expect("Deck not found")
        .to_card_ids();
    let deck2 = registry.get("broodmother_swarm")
        .expect("Deck not found")
        .to_card_ids();

    let client = GameClient::new(db);
    (client, deck1, deck2)
}

#[test]
fn test_game_client_creation() {
    let (client, _, _) = create_test_client();
    assert!(client.get_state().is_none());
    assert!(client.is_game_over());
}

#[test]
fn test_game_client_start_game() {
    let db = load_card_db();
    let registry = load_decks();

    let deck1 = registry.get("architect_fortify")
        .expect("Deck not found")
        .to_card_ids();
    let deck2 = registry.get("broodmother_swarm")
        .expect("Deck not found")
        .to_card_ids();

    let mut client = GameClient::new(db);
    client.start_game(deck1, deck2, 12345);

    assert!(client.get_state().is_some());
    assert!(!client.is_game_over());
    assert_eq!(client.turn_number(), 1);
    assert_eq!(client.current_player(), Some(PlayerId::PLAYER_ONE));
}

#[test]
fn test_game_client_emits_game_started_event() {
    let db = load_card_db();
    let registry = load_decks();

    let deck1 = registry.get("architect_fortify")
        .expect("Deck not found")
        .to_card_ids();
    let deck2 = registry.get("broodmother_swarm")
        .expect("Deck not found")
        .to_card_ids();

    let mut client = GameClient::new(db);
    client.start_game(deck1, deck2, 12345);

    let events = client.drain_events();

    // Should have GameStarted and TurnStarted events
    assert!(events.iter().any(|e| matches!(e, GameEvent::GameStarted { .. })));
    assert!(events.iter().any(|e| matches!(e, GameEvent::TurnStarted { player: PlayerId(0), .. })));
}

#[test]
fn test_game_client_apply_action() {
    let db = load_card_db();
    let registry = load_decks();

    let deck1 = registry.get("architect_fortify")
        .expect("Deck not found")
        .to_card_ids();
    let deck2 = registry.get("broodmother_swarm")
        .expect("Deck not found")
        .to_card_ids();

    let mut client = GameClient::new(db);
    client.start_game(deck1, deck2, 12345);

    // Clear initial events
    client.drain_events();

    // Get legal actions and apply one
    let legal = client.get_legal_actions();
    assert!(!legal.is_empty());

    // Apply EndTurn action
    let end_turn = legal.iter().find(|a| matches!(a, cardgame::actions::Action::EndTurn));
    if let Some(action) = end_turn {
        let result = client.apply_action(*action);
        assert!(result.is_ok());
    }
}

#[test]
fn test_game_client_event_history() {
    let db = load_card_db();
    let registry = load_decks();

    let deck1 = registry.get("architect_fortify")
        .expect("Deck not found")
        .to_card_ids();
    let deck2 = registry.get("broodmother_swarm")
        .expect("Deck not found")
        .to_card_ids();

    let mut client = GameClientBuilder::new()
        .with_card_db(db)
        .keep_history(true)
        .build();

    client.start_game(deck1, deck2, 12345);

    // Apply a few actions
    for _ in 0..3 {
        let legal = client.get_legal_actions();
        if let Some(action) = legal.iter().find(|a| matches!(a, cardgame::actions::Action::EndTurn)) {
            let _ = client.apply_action(*action);
        }
    }

    // History should have accumulated events
    let history = client.event_history();
    assert!(!history.is_empty());
}

#[test]
fn test_state_snapshot_creation() {
    let db = load_card_db();
    let registry = load_decks();

    let deck1 = registry.get("architect_fortify")
        .expect("Deck not found")
        .to_card_ids();
    let deck2 = registry.get("broodmother_swarm")
        .expect("Deck not found")
        .to_card_ids();

    let mut client = GameClient::new(db);
    client.start_game(deck1, deck2, 12345);

    let state = client.get_state().unwrap();
    let snapshot = StateSnapshot::from_state(state);

    assert_eq!(snapshot.turn, 1);
    assert_eq!(snapshot.active_player, PlayerId::PLAYER_ONE);
    assert_eq!(snapshot.players[0].life, 30); // Starting life
    assert_eq!(snapshot.players[1].life, 30);
}

#[test]
fn test_game_event_serialization() {
    use cardgame::state::GameMode;

    let event = GameEvent::GameStarted {
        seed: 12345,
        mode: GameMode::Attrition,
        player1_deck_size: 30,
        player2_deck_size: 30,
    };

    // Should serialize to JSON
    let json = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(json.contains("GameStarted"));
    assert!(json.contains("12345"));

    // Should deserialize back
    let deserialized: GameEvent = serde_json::from_str(&json).expect("Failed to deserialize");
    if let GameEvent::GameStarted { seed, .. } = deserialized {
        assert_eq!(seed, 12345);
    } else {
        panic!("Wrong event type after deserialization");
    }
}

#[test]
fn test_game_event_affected_player() {
    use cardgame::types::Slot;
    use cardgame::keywords::Keywords;
    use cardgame::types::{CardId, CreatureInstanceId};

    let event = GameEvent::CreatureSpawned {
        player: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        card_id: CardId(1000),
        instance_id: CreatureInstanceId(1),
        attack: 2,
        health: 4,
        keywords: Keywords::none(),
    };

    assert_eq!(event.affected_player(), Some(PlayerId::PLAYER_ONE));

    let event2 = GameEvent::GameStarted {
        seed: 0,
        mode: GameMode::Attrition,
        player1_deck_size: 30,
        player2_deck_size: 30,
    };

    assert_eq!(event2.affected_player(), None);
}
