//! Game bridge - wraps the cardgame engine for Bevy integration.

use std::sync::Arc;

use bevy::prelude::*;
use cardgame::cards::CardDatabase;
use cardgame::client_api::{GameClient, GameEvent};
use cardgame::bots::IntrospectionConfig;
use cardgame::decks::DeckRegistry;
use cardgame::bots::{BotDecision, MctsTreeSnapshot};

/// Resource that bridges the cardgame engine to Bevy.
#[derive(Resource)]
pub struct GameBridge {
    /// The game client wrapping the engine
    pub client: Option<GameClient>,
    /// Shared card database
    pub card_db: Arc<CardDatabase>,
    /// Shared deck registry
    pub deck_registry: Arc<DeckRegistry>,
    /// Last AI decision for glassbox visualization
    pub last_decision: Option<BotDecision>,
    /// AI introspection configuration (used for future AI visualization)
    #[allow(dead_code)]
    pub ai_config: IntrospectionConfig,
}

impl GameBridge {
    /// Create a new GameBridge, loading game data.
    pub fn new() -> Self {
        let data_dir = cardgame::data_dir();

        let card_db = CardDatabase::load_from_directory(data_dir.join("cards/core_set"))
            .expect("Failed to load card database");

        let deck_registry = DeckRegistry::load_from_directory(data_dir.join("decks"))
            .expect("Failed to load deck registry");

        Self {
            client: None,
            card_db: Arc::new(card_db),
            deck_registry: Arc::new(deck_registry),
            last_decision: None,
            ai_config: IntrospectionConfig::full(),
        }
    }

    /// Start a new game with the specified decks.
    pub fn start_game(&mut self, deck1_id: &str, deck2_id: &str, seed: u64) -> Result<(), String> {
        let deck1 = self.deck_registry.get(deck1_id)
            .ok_or_else(|| format!("Deck not found: {}", deck1_id))?;
        let deck2 = self.deck_registry.get(deck2_id)
            .ok_or_else(|| format!("Deck not found: {}", deck2_id))?;

        let mut client = GameClient::new(self.card_db.clone());
        let deck1_cards: Vec<cardgame::types::CardId> = deck1.cards.iter().map(|&id| cardgame::types::CardId(id)).collect();
        let deck2_cards: Vec<cardgame::types::CardId> = deck2.cards.iter().map(|&id| cardgame::types::CardId(id)).collect();
        client.start_game(deck1_cards, deck2_cards, seed);

        self.client = Some(client);
        self.last_decision = None;

        Ok(())
    }

    /// Check if a game is currently active.
    #[allow(dead_code)]
    pub fn is_game_active(&self) -> bool {
        self.client.as_ref().map(|c| !c.is_game_over()).unwrap_or(false)
    }

    /// Get current game events.
    #[allow(dead_code)]
    pub fn drain_events(&mut self) -> Vec<GameEvent> {
        self.client.as_mut()
            .map(|c| c.drain_events().into_iter().collect())
            .unwrap_or_default()
    }

    /// Get the MCTS snapshot from the last decision for visualization.
    pub fn get_mcts_snapshot(&self) -> Option<&MctsTreeSnapshot> {
        self.last_decision.as_ref()
            .and_then(|d| d.mcts_snapshot.as_ref())
    }

    /// Store a bot decision for visualization.
    #[allow(dead_code)]
    pub fn store_decision(&mut self, decision: BotDecision) {
        self.last_decision = Some(decision);
    }
}

impl Default for GameBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Event emitted when a game event occurs in the cardgame engine.
#[allow(dead_code)]
#[derive(Event, Clone)]
pub struct BevyGameEvent(pub GameEvent);
