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
    /// Current deck 1 ID (for restart in multi-game mode)
    pub current_deck1: Option<String>,
    /// Current deck 2 ID (for restart in multi-game mode)
    pub current_deck2: Option<String>,
    /// Current seed (for restart in multi-game mode)
    pub current_seed: u64,
}

impl GameBridge {
    /// Create a new GameBridge, loading game data.
    /// On native builds, loads from filesystem. On WASM, uses embedded data.
    #[cfg(not(target_arch = "wasm32"))]
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
            current_deck1: None,
            current_deck2: None,
            current_seed: 42,
        }
    }

    /// WASM version: Create a new GameBridge using embedded game data.
    #[cfg(target_arch = "wasm32")]
    pub fn new() -> Self {
        use cardgame::embedded_data;

        let (card_db, deck_registry) = embedded_data::load_embedded_game_data()
            .expect("Failed to load embedded game data");

        Self {
            client: None,
            card_db: Arc::new(card_db),
            deck_registry: Arc::new(deck_registry),
            last_decision: None,
            ai_config: IntrospectionConfig::full(),
            current_deck1: None,
            current_deck2: None,
            current_seed: 42,
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

        // Store deck IDs and seed for restart capability
        self.current_deck1 = Some(deck1_id.to_string());
        self.current_deck2 = Some(deck2_id.to_string());
        self.current_seed = seed;

        Ok(())
    }

    /// Restart the current game with a new seed.
    /// Used for multi-game benchmark runs.
    pub fn restart_with_seed(&mut self, new_seed: u64) -> Result<(), String> {
        let deck1_id = self.current_deck1.clone()
            .ok_or_else(|| "No deck1 set for restart".to_string())?;
        let deck2_id = self.current_deck2.clone()
            .ok_or_else(|| "No deck2 set for restart".to_string())?;

        self.start_game(&deck1_id, &deck2_id, new_seed)
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
