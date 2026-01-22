//! Game session management for the MCP server.
//!
//! Provides a single-active-session model for managing games.

use std::sync::Arc;

use cardgame::bots::{create_bot, AlphaBetaConfig, BotType, MctsConfig};
use cardgame::client_api::GameClient;
use cardgame::{Action, CardDatabase, DeckRegistry, PlayerId};
use rand::Rng;

/// A game session representing an active game.
pub struct GameSession {
    /// Unique identifier for this game session
    pub game_id: String,
    /// The game client wrapping the engine
    pub client: GameClient,
    /// The player's ID (always Player 1 from human perspective)
    pub player_id: PlayerId,
    /// Bot type for the opponent
    pub opponent_bot_type: BotType,
    /// Seed for bot randomization
    pub bot_seed: u64,
    /// Game seed used when starting
    pub game_seed: u64,
    /// Player deck ID for display
    pub player_deck_id: String,
    /// Opponent deck ID for display
    pub opponent_deck_id: String,
}

// GameSession is Send because all its fields are Send
unsafe impl Send for GameSession {}

/// Session manager for MCP server.
///
/// Supports a single active session at a time (simplifies the MCP interface).
pub struct SessionManager {
    /// Card database (shared)
    card_db: Arc<CardDatabase>,
    /// Deck registry (shared)
    deck_registry: Arc<DeckRegistry>,
    /// Current active session (if any)
    active_session: Option<GameSession>,
}

// Explicitly implement Send + Sync for SessionManager
unsafe impl Send for SessionManager {}
unsafe impl Sync for SessionManager {}

impl SessionManager {
    /// Create a new session manager.
    pub fn new() -> anyhow::Result<Self> {
        // Get the data directory
        let data_dir = cardgame::data_dir();

        // Load card database from directory
        let cards_path = data_dir.join("cards/core_set");
        let card_db = CardDatabase::load_from_directory(&cards_path)
            .map_err(|e| anyhow::anyhow!("Failed to load card database: {}", e))?;

        // Load deck registry
        let deck_registry = DeckRegistry::load_from_directory(data_dir.join("decks"))
            .map_err(|e| anyhow::anyhow!("Failed to load deck registry: {}", e))?;

        Ok(Self {
            card_db: Arc::new(card_db),
            deck_registry: Arc::new(deck_registry),
            active_session: None,
        })
    }

    /// Get the card database.
    pub fn card_db(&self) -> &Arc<CardDatabase> {
        &self.card_db
    }

    /// Get the deck registry.
    pub fn deck_registry(&self) -> &Arc<DeckRegistry> {
        &self.deck_registry
    }

    /// Check if there's an active session.
    pub fn has_active_session(&self) -> bool {
        self.active_session.is_some()
    }

    /// Get the active session (if any).
    pub fn active_session(&self) -> Option<&GameSession> {
        self.active_session.as_ref()
    }

    /// Get the active session mutably (if any).
    pub fn active_session_mut(&mut self) -> Option<&mut GameSession> {
        self.active_session.as_mut()
    }

    /// Start a new game session.
    ///
    /// Returns an error message if something goes wrong.
    pub fn start_game(
        &mut self,
        player_deck_id: &str,
        opponent_deck_id: &str,
        bot_type_str: &str,
        seed: Option<u64>,
    ) -> Result<(), String> {
        // End existing session if any
        self.active_session = None;

        // Get decks
        let player_deck = self
            .deck_registry
            .get(player_deck_id)
            .ok_or_else(|| format!("Player deck not found: {}", player_deck_id))?;

        let opponent_deck = self
            .deck_registry
            .get(opponent_deck_id)
            .ok_or_else(|| format!("Opponent deck not found: {}", opponent_deck_id))?;

        // Parse bot type
        let bot_type: BotType = bot_type_str
            .parse()
            .map_err(|e| format!("Unknown bot type: {}", e))?;

        // Create game client
        let mut client = GameClient::new(self.card_db.clone());

        // Convert deck cards to CardId vec
        let deck1_cards = player_deck.to_card_ids();
        let deck2_cards = opponent_deck.to_card_ids();

        // Generate seeds
        let mut rng = rand::thread_rng();
        let game_seed = seed.unwrap_or_else(|| rng.gen::<u64>());
        let bot_seed = rng.gen::<u64>();

        // Start game (player is always Player 1)
        client.start_game(deck1_cards, deck2_cards, game_seed);

        // Generate unique game ID
        let game_id = uuid::Uuid::new_v4().to_string();

        // Create session
        let session = GameSession {
            game_id,
            client,
            player_id: PlayerId::PLAYER_ONE,
            opponent_bot_type: bot_type,
            bot_seed,
            game_seed,
            player_deck_id: player_deck_id.to_string(),
            opponent_deck_id: opponent_deck_id.to_string(),
        };

        self.active_session = Some(session);
        Ok(())
    }

    /// Apply an action to the current game.
    pub fn apply_action(&mut self, action_index: u8) -> Result<(), String> {
        let session = self
            .active_session
            .as_mut()
            .ok_or_else(|| "No active game. Use start_game first.".to_string())?;

        session
            .client
            .apply_action_by_index(action_index)?;

        Ok(())
    }

    /// Get AI action for the opponent.
    pub fn get_ai_action(&self) -> Result<Action, String> {
        let session = self
            .active_session
            .as_ref()
            .ok_or_else(|| "No active game.".to_string())?;

        // Create bot on-demand
        let mcts_config = MctsConfig::default();
        let alphabeta_config = AlphaBetaConfig::default();
        let mut bot = create_bot(
            &self.card_db,
            &session.opponent_bot_type,
            None,
            &mcts_config,
            &alphabeta_config,
            session.bot_seed,
        );

        session
            .client
            .select_bot_action(&mut *bot)
            .ok_or_else(|| "Game is over or not started".to_string())
    }

    /// Run the AI's turn (apply actions until it's player's turn or game ends).
    pub fn run_ai_turn(&mut self) -> Result<Vec<Action>, String> {
        let mut actions = Vec::new();

        loop {
            let (is_game_over, is_player_turn) = {
                let session = self
                    .active_session
                    .as_ref()
                    .ok_or_else(|| "No active game.".to_string())?;

                let is_over = session.client.is_game_over();
                let current = session.client.current_player();
                let is_player = current == Some(session.player_id);

                (is_over, is_player)
            };

            if is_game_over || is_player_turn {
                break;
            }

            // Get and apply AI action
            let action = self.get_ai_action()?;
            let action_index = action.to_index();
            actions.push(action);

            self.apply_action(action_index)?;
        }

        Ok(actions)
    }

    /// End the current game and return the session.
    pub fn end_game(&mut self) -> Option<GameSession> {
        self.active_session.take()
    }
}
