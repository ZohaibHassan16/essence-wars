//! Game manager for tracking active games.

use cardgame::bots::{create_bot, AlphaBetaConfig, BotType, GreedyBot, MctsConfig};
use cardgame::client_api::GameClient;
use cardgame::execution::MAX_ACTIONS_PER_GAME;
use cardgame::{CardDatabase, DeckDefinition, DeckRegistry, PlayerId};
use std::time::Instant;
use parking_lot::RwLock;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::serialization::*;
use super::spectator::*;

/// Represents an active game session
pub struct GameSession {
    pub client: GameClient,
    pub player_id: PlayerId,
    /// Bot type for the opponent (bot created on-demand due to lifetime constraints)
    pub opponent_bot_type: BotType,
    /// Seed for bot randomization
    pub bot_seed: u64,
    /// History of action infos for display
    pub action_history: Vec<ActionInfo>,
    /// History of action indices for replay-based undo
    pub action_indices: Vec<u8>,
    /// Original config for replaying the game
    pub original_config: GameConfig,
    /// Game seed used when starting
    pub game_seed: u64,
}

// GameSession is Send because all its fields are Send
unsafe impl Send for GameSession {}

/// Thread-safe game manager
pub struct GameManager {
    games: RwLock<HashMap<String, GameSession>>,
    card_db: Arc<CardDatabase>,
    deck_registry: Arc<DeckRegistry>,
}

// Explicitly implement Send + Sync for GameManager
unsafe impl Send for GameManager {}
unsafe impl Sync for GameManager {}

impl GameManager {
    pub fn new() -> Result<Self, String> {
        // Get the data directory
        let data_dir = cardgame::data_dir();

        // Load card database from directory
        let cards_path = data_dir.join("cards/core_set");
        let commanders_path = data_dir.join("commanders");
        let card_db = CardDatabase::load_from_directory(&cards_path)
            .map_err(|e| format!("Failed to load card database: {}", e))?
            .with_commanders(
                CardDatabase::load_commanders_from_directory(&commanders_path)
                    .map_err(|e| format!("Failed to load commanders: {}", e))?,
            );

        // Load deck registry - takes only a path
        let deck_registry = DeckRegistry::load_from_directory(data_dir.join("decks"))
            .map_err(|e| format!("Failed to load deck registry: {}", e))?;

        Ok(Self {
            games: RwLock::new(HashMap::new()),
            card_db: Arc::new(card_db),
            deck_registry: Arc::new(deck_registry),
        })
    }

    /// Get a clone of the card database Arc (for async operations)
    pub fn card_db(&self) -> Arc<CardDatabase> {
        self.card_db.clone()
    }

    /// Get a clone of the deck registry Arc (for async operations)
    pub fn deck_registry(&self) -> Arc<DeckRegistry> {
        self.deck_registry.clone()
    }

    /// Get read access to games (for replay saving)
    pub fn games_read(&self) -> parking_lot::RwLockReadGuard<'_, HashMap<String, GameSession>> {
        self.games.read()
    }

    /// List all available decks
    pub fn list_decks(&self) -> Vec<DeckInfo> {
        self.deck_registry
            .decks()
            .map(|deck| {
                let faction = deck.faction()
                    .map(|f| faction_to_string(f).to_string())
                    .unwrap_or_else(|| "neutral".to_string());

                // Look up commander information
                let commander = self.card_db
                    .get_commander(deck.commander_id())
                    .map(CommanderDto::from_commander_def);

                DeckInfo {
                    id: deck.id.clone(),
                    name: deck.name.clone(),
                    description: deck.description.clone(),
                    playstyle: deck.playstyle.clone(),
                    faction,
                    card_count: deck.cards.len(),
                    commander,
                }
            })
            .collect()
    }

    /// List available bot types
    pub fn list_bots(&self) -> Vec<BotInfo> {
        vec![
            BotInfo {
                id: "random".to_string(),
                name: "Random Bot".to_string(),
                description: "Makes completely random moves. Good for testing.".to_string(),
            },
            BotInfo {
                id: "greedy".to_string(),
                name: "Greedy Bot".to_string(),
                description: "Evaluates each move and picks the best. Moderate difficulty."
                    .to_string(),
            },
            BotInfo {
                id: "mcts".to_string(),
                name: "MCTS Bot".to_string(),
                description: "Uses Monte Carlo Tree Search. Configurable strength.".to_string(),
            },
            BotInfo {
                id: "alphabeta".to_string(),
                name: "Alpha-Beta Bot".to_string(),
                description: "Uses minimax search with pruning. Configurable depth.".to_string(),
            },
        ]
    }

    /// Resolve a deck ID to a DeckDefinition, supporting both built-in and custom decks.
    /// Custom deck IDs are prefixed with "custom:".
    fn resolve_deck(
        &self,
        deck_id: &str,
        custom_deck_manager: Option<&super::CustomDeckManager>,
    ) -> Result<DeckDefinition, String> {
        if let Some(custom_id) = deck_id.strip_prefix("custom:") {
            // Load custom deck
            let manager = custom_deck_manager
                .ok_or_else(|| "Custom deck manager not available".to_string())?;

            let custom_deck = manager
                .load_deck(custom_id)
                .map_err(|e| format!("Failed to load custom deck '{}': {}", custom_id, e))?;

            // Convert CustomDeck to DeckDefinition
            manager
                .to_deck_definition(&custom_deck)
                .map_err(|e| format!("Failed to convert custom deck '{}': {}", custom_id, e))
        } else {
            // Built-in deck
            self.deck_registry
                .get(deck_id)
                .cloned()
                .ok_or_else(|| format!("Deck not found: {}", deck_id))
        }
    }

    /// Create a new game session (built-in decks only, for backwards compatibility)
    pub fn new_game(&self, config: GameConfig) -> Result<GameStateDto, String> {
        self.new_game_with_custom_decks(config, None)
    }

    /// Create a new game session with custom deck support
    pub fn new_game_with_custom_decks(
        &self,
        config: GameConfig,
        custom_deck_manager: Option<&super::CustomDeckManager>,
    ) -> Result<GameStateDto, String> {
        // Get decks (supports both built-in and custom: prefixed deck IDs)
        let player_deck = self.resolve_deck(&config.player_deck_id, custom_deck_manager)?;
        let opponent_deck = self.resolve_deck(&config.opponent_deck_id, custom_deck_manager)?;

        // Parse bot type
        let bot_type: BotType = config
            .opponent_bot_type
            .parse()
            .map_err(|e| format!("Unknown bot type: {}", e))?;

        // Create game client
        let mut client = GameClient::new(self.card_db.clone());

        // Determine who goes first
        let player_first = config.player_goes_first.unwrap_or(true);

        // Use provided seed or generate random
        let mut rng = rand::thread_rng();
        let game_seed = config.seed.unwrap_or_else(|| rng.gen::<u64>());
        let bot_seed = rng.gen::<u64>();

        // Start game (deck definitions include commanders)
        if player_first {
            client.start_game(&player_deck, &opponent_deck, game_seed);
        } else {
            client.start_game(&opponent_deck, &player_deck, game_seed);
        }

        let game_id = Uuid::new_v4().to_string();
        let player_id = if player_first {
            PlayerId::PLAYER_ONE
        } else {
            PlayerId::PLAYER_TWO
        };

        // Convert to DTO before creating session
        let state_dto = self.client_to_dto(&client, &game_id, player_id);

        let session = GameSession {
            client,
            player_id,
            opponent_bot_type: bot_type,
            bot_seed,
            action_history: Vec::new(),
            action_indices: Vec::new(),
            original_config: config,
            game_seed,
        };

        // Store session
        self.games.write().insert(game_id.clone(), session);

        Ok(state_dto)
    }

    /// Get current game state
    pub fn get_game_state(&self, game_id: &str) -> Result<GameStateDto, String> {
        let games = self.games.read();
        let session = games
            .get(game_id)
            .ok_or_else(|| format!("Game not found: {}", game_id))?;

        Ok(self.client_to_dto(&session.client, game_id, session.player_id))
    }

    /// Get legal actions for the current player
    pub fn get_legal_actions(&self, game_id: &str) -> Result<Vec<ActionInfo>, String> {
        let games = self.games.read();
        let session = games
            .get(game_id)
            .ok_or_else(|| format!("Game not found: {}", game_id))?;

        let actions = session.client.get_legal_actions();
        let action_infos: Vec<ActionInfo> = actions
            .iter()
            .map(|action| action_to_info(action, action.to_index()))
            .collect();

        Ok(action_infos)
    }

    /// Apply a player action
    pub fn apply_action(&self, game_id: &str, action_index: u8) -> Result<GameStateUpdate, String> {
        let mut games = self.games.write();
        let session = games
            .get_mut(game_id)
            .ok_or_else(|| format!("Game not found: {}", game_id))?;

        // Apply action and get events
        let events = session
            .client
            .apply_action_by_index(action_index)
            .map_err(|e| e.to_string())?;

        // Store action index for replay-based undo
        session.action_indices.push(action_index);

        let action = cardgame::Action::from_index(action_index)
            .ok_or_else(|| format!("Invalid action index: {}", action_index))?;

        let action_info = action_to_info(&action, action_index);
        session.action_history.push(action_info.clone());

        let state_dto = self.client_to_dto(&session.client, game_id, session.player_id);
        let event_dtos: Vec<GameEventDto> = events.iter().map(game_event_to_dto).collect();

        Ok(GameStateUpdate {
            state: state_dto,
            last_action: Some(action_info),
            events: event_dtos,
        })
    }

    /// Get AI move for the opponent
    pub fn get_ai_move(&self, game_id: &str) -> Result<ActionInfo, String> {
        let games = self.games.read();
        let session = games
            .get(game_id)
            .ok_or_else(|| format!("Game not found: {}", game_id))?;

        // Create bot on-demand (bots have lifetime tied to card_db)
        let mcts_config = MctsConfig::default();
        let alphabeta_config = AlphaBetaConfig::default();
        let mut bot = create_bot(
            &self.card_db,
            &session.opponent_bot_type,
            None, // No custom weights
            &mcts_config,
            &alphabeta_config,
            session.bot_seed,
        );

        // Use the bot to select an action
        let action = session
            .client
            .select_bot_action(&mut *bot)
            .ok_or_else(|| "Game is over or not started".to_string())?;

        let action_index = action.to_index();
        Ok(action_to_info(&action, action_index))
    }

    /// Get AI hint with recommended action and alternatives
    pub fn get_ai_hint(&self, game_id: &str) -> Result<AiHintResponse, String> {
        let start = Instant::now();
        let games = self.games.read();
        let session = games
            .get(game_id)
            .ok_or_else(|| format!("Game not found: {}", game_id))?;

        // Get legal actions
        let legal_actions = session.client.get_legal_actions();
        if legal_actions.is_empty() {
            return Err("No legal actions available".to_string());
        }

        // Use GreedyBot to get a recommendation
        let mut greedy_bot = GreedyBot::new(&self.card_db, 42);

        // Get the AI's recommendation using the client's built-in hint system
        let best_action = session
            .client
            .get_ai_hint(&mut greedy_bot)
            .ok_or_else(|| "Failed to get AI hint".to_string())?;

        let thinking_time_ms = start.elapsed().as_millis() as u64;
        let recommended = action_to_info(&best_action, best_action.to_index());

        // For alternatives, just show other legal actions (without detailed scoring for now)
        let alternatives: Vec<AlternativeAction> = legal_actions
            .iter()
            .filter(|&&a| a != best_action)
            .take(4)
            .map(|action| {
                AlternativeAction {
                    action: action_to_info(action, action.to_index()),
                    score: 0.0,
                    score_delta: 0.0,
                }
            })
            .collect();

        Ok(AiHintResponse {
            recommended_action: recommended,
            score: 0.0,  // Would need engine access for actual score
            alternatives,
            thinking_time_ms,
        })
    }

    /// Undo the last action (dev mode feature)
    /// This replays the game from the beginning without the last action.
    pub fn undo_action(&self, game_id: &str) -> Result<GameStateDto, String> {
        let mut games = self.games.write();
        let session = games
            .get_mut(game_id)
            .ok_or_else(|| format!("Game not found: {}", game_id))?;

        // Check if there are actions to undo
        if session.action_indices.is_empty() {
            return Err("No actions to undo".to_string());
        }

        // Get decks
        let player_deck = self
            .deck_registry
            .get(&session.original_config.player_deck_id)
            .ok_or_else(|| "Player deck not found".to_string())?;
        let opponent_deck = self
            .deck_registry
            .get(&session.original_config.opponent_deck_id)
            .ok_or_else(|| "Opponent deck not found".to_string())?;

        let player_first = session.original_config.player_goes_first.unwrap_or(true);

        // Create new client and start game with same seed
        let mut new_client = GameClient::new(self.card_db.clone());
        if player_first {
            new_client.start_game(player_deck, opponent_deck, session.game_seed);
        } else {
            new_client.start_game(opponent_deck, player_deck, session.game_seed);
        }

        // Remove the last action
        session.action_indices.pop();
        session.action_history.pop();

        // Replay all actions except the last one
        for &action_index in &session.action_indices {
            let _ = new_client.apply_action_by_index(action_index);
        }

        // Replace the client with the replayed state
        session.client = new_client;

        Ok(self.client_to_dto(&session.client, game_id, session.player_id))
    }

    /// Check if undo is available
    pub fn can_undo(&self, game_id: &str) -> Result<bool, String> {
        let games = self.games.read();
        let session = games
            .get(game_id)
            .ok_or_else(|| format!("Game not found: {}", game_id))?;

        Ok(!session.action_indices.is_empty())
    }

    /// End the game and clean up
    pub fn end_game(&self, game_id: &str) -> Result<GameResultDto, String> {
        let mut games = self.games.write();
        let session = games
            .remove(game_id)
            .ok_or_else(|| format!("Game not found: {}", game_id))?;

        let state = session.client.get_state();
        let (player_life, opponent_life) = state
            .map(|s| {
                (
                    s.players[session.player_id.index()].life,
                    s.players[session.player_id.opponent().index()].life,
                )
            })
            .unwrap_or((0, 0));

        let result = session.client.get_result();
        let winner = result.and_then(|r| match r {
            cardgame::core::state::GameResult::Win { winner, .. } => {
                if winner == session.player_id {
                    Some(1)
                } else {
                    Some(2)
                }
            }
            cardgame::core::state::GameResult::Draw => None,
        });

        let reason = result
            .map(|r| match r {
                cardgame::core::state::GameResult::Win { reason, .. } => format!("{:?}", reason),
                cardgame::core::state::GameResult::Draw => "draw".to_string(),
            })
            .unwrap_or_else(|| "forfeit".to_string());

        Ok(GameResultDto {
            winner,
            reason,
            final_turn: session.client.turn_number(),
            player_final_life: player_life,
            opponent_final_life: opponent_life,
        })
    }

    // NOTE: Spectator match computation moved to SpectatorComputer for async support

    /// Convert GameClient state to DTO
    fn client_to_dto(&self, client: &GameClient, game_id: &str, player_id: PlayerId) -> GameStateDto {
        let state = match client.get_state() {
            Some(s) => s,
            None => {
                return GameStateDto {
                    id: game_id.to_string(),
                    turn: 0,
                    phase: "not_started".to_string(),
                    active_player: 0,
                    player: PlayerStateDto {
                        life: 0,
                        max_life: 30,
                        essence: 0,
                        max_essence: 0,
                        action_points: 0,
                        deck_count: 0,
                        hand: Vec::new(),
                        creatures: vec![None; 5],
                        supports: vec![None; 2],
                        commander: None,
                    },
                    opponent: PlayerStateDto {
                        life: 0,
                        max_life: 30,
                        essence: 0,
                        max_essence: 0,
                        action_points: 0,
                        deck_count: 0,
                        hand: Vec::new(),
                        creatures: vec![None; 5],
                        supports: vec![None; 2],
                        commander: None,
                    },
                    is_game_over: false,
                    winner: None,
                    game_over_reason: None,
                };
            }
        };

        let player_state = &state.players[player_id.index()];
        let opponent_state = &state.players[player_id.opponent().index()];

        // Get commander DTOs
        let player_commander = commander_to_dto(state.get_commander(player_id), &self.card_db);
        let opponent_commander = commander_to_dto(state.get_commander(player_id.opponent()), &self.card_db);

        // Convert player hand (visible)
        let player_hand: Vec<CardDto> = player_state
            .hand
            .iter()
            .filter_map(|card_inst| {
                match self.card_db.get(card_inst.card_id) {
                    Some(card) => Some(CardDto::from_card_def(card)),
                    None => {
                        eprintln!("Warning: Card ID {} not found in database", card_inst.card_id.0);
                        None
                    }
                }
            })
            .collect();

        // Convert opponent hand (hidden)
        let opponent_hand: Vec<CardDto> = (0..opponent_state.hand.len())
            .map(|_| CardDto::hidden())
            .collect();

        // Convert creatures to slot-indexed arrays
        let player_creatures = self.creatures_to_slots(player_state, state.current_turn);
        let opponent_creatures = self.creatures_to_slots(opponent_state, state.current_turn);

        // Convert supports to slot-indexed arrays
        let player_supports = self.supports_to_slots(player_state);
        let opponent_supports = self.supports_to_slots(opponent_state);

        let active_player = if state.active_player == player_id { 1 } else { 2 };

        let result = client.get_result();
        let winner = result.and_then(|r| match r {
            cardgame::core::state::GameResult::Win { winner, .. } => {
                if winner == player_id {
                    Some(1)
                } else {
                    Some(2)
                }
            }
            cardgame::core::state::GameResult::Draw => None,
        });

        let game_over_reason = result.map(|r| match r {
            cardgame::core::state::GameResult::Win { reason, .. } => format!("{:?}", reason),
            cardgame::core::state::GameResult::Draw => "draw".to_string(),
        });

        GameStateDto {
            id: game_id.to_string(),
            turn: state.current_turn,
            phase: format!("{:?}", state.phase),
            active_player,
            player: PlayerStateDto {
                life: player_state.life,
                max_life: 30,
                essence: player_state.current_essence,
                max_essence: player_state.max_essence,
                action_points: player_state.action_points,
                deck_count: player_state.deck.len(),
                hand: player_hand,
                creatures: player_creatures,
                supports: player_supports,
                commander: player_commander,
            },
            opponent: PlayerStateDto {
                life: opponent_state.life,
                max_life: 30,
                essence: opponent_state.current_essence,
                max_essence: opponent_state.max_essence,
                action_points: opponent_state.action_points,
                deck_count: opponent_state.deck.len(),
                hand: opponent_hand,
                creatures: opponent_creatures,
                supports: opponent_supports,
                commander: opponent_commander,
            },
            is_game_over: client.is_game_over(),
            winner,
            game_over_reason,
        }
    }

    /// Convert creature ArrayVec to slot-indexed array
    fn creatures_to_slots(
        &self,
        creatures: &cardgame::core::state::PlayerState,
        current_turn: u16,
    ) -> Vec<Option<CreatureDto>> {
        let mut slots: Vec<Option<CreatureDto>> = vec![None; 5];
        for creature in creatures.creatures.iter() {
            // CardId(0) is a sentinel for token creatures (not in database)
            if creature.card_id.0 == 0 {
                let dto = CreatureDto::from_token(creature, current_turn);
                slots[creature.slot.0 as usize] = Some(dto);
            } else {
                match self.card_db.get(creature.card_id) {
                    Some(card) => {
                        let dto = CreatureDto::from_creature(creature, card, current_turn);
                        slots[creature.slot.0 as usize] = Some(dto);
                    }
                    None => {
                        eprintln!("Warning: Creature card ID {} not found in database", creature.card_id.0);
                    }
                }
            }
        }
        slots
    }

    /// Convert support ArrayVec to slot-indexed array
    fn supports_to_slots(
        &self,
        player_state: &cardgame::core::state::PlayerState,
    ) -> Vec<Option<SupportDto>> {
        let mut slots: Vec<Option<SupportDto>> = vec![None; 2];
        for support in player_state.supports.iter() {
            match self.card_db.get(support.card_id) {
                Some(card) => {
                    let dto = SupportDto::from_support(support, card);
                    slots[support.slot.0 as usize] = Some(dto);
                }
                None => {
                    eprintln!("Warning: Support card ID {} not found in database", support.card_id.0);
                }
            }
        }
        slots
    }
}

impl Default for GameManager {
    fn default() -> Self {
        Self::new().expect("Failed to create game manager")
    }
}

/// A clonable struct for computing spectator matches in background threads.
/// This is separate from GameManager to allow moving into spawn_blocking.
#[derive(Clone)]
pub struct SpectatorComputer {
    card_db: Arc<CardDatabase>,
    deck_registry: Arc<DeckRegistry>,
    custom_deck_manager: Option<Arc<super::CustomDeckManager>>,
}

impl SpectatorComputer {
    /// Create a new SpectatorComputer from a GameManager
    pub fn from_manager(manager: &GameManager) -> Self {
        Self {
            card_db: manager.card_db(),
            deck_registry: manager.deck_registry(),
            custom_deck_manager: None,
        }
    }

    /// Create a new SpectatorComputer with custom deck support
    pub fn from_manager_with_custom_decks(
        manager: &GameManager,
        custom_deck_manager: Arc<super::CustomDeckManager>,
    ) -> Self {
        Self {
            card_db: manager.card_db(),
            deck_registry: manager.deck_registry(),
            custom_deck_manager: Some(custom_deck_manager),
        }
    }

    /// Resolve a deck ID to a DeckDefinition, supporting both built-in and custom decks.
    /// Custom deck IDs are prefixed with "custom:".
    fn resolve_deck(&self, deck_id: &str) -> Result<DeckDefinition, String> {
        if let Some(custom_id) = deck_id.strip_prefix("custom:") {
            // Load custom deck
            let manager = self
                .custom_deck_manager
                .as_ref()
                .ok_or_else(|| "Custom deck manager not available".to_string())?;

            let custom_deck = manager
                .load_deck(custom_id)
                .map_err(|e| format!("Failed to load custom deck '{}': {}", custom_id, e))?;

            // Convert CustomDeck to DeckDefinition
            manager
                .to_deck_definition(&custom_deck)
                .map_err(|e| format!("Failed to convert custom deck '{}': {}", custom_id, e))
        } else {
            // Built-in deck
            self.deck_registry
                .get(deck_id)
                .cloned()
                .ok_or_else(|| format!("Deck not found: {}", deck_id))
        }
    }

    /// Compute a complete spectator match between two AI players.
    pub fn compute_match(&self, config: SpectatorConfig) -> Result<SpectatorMatch, String> {
        // Get decks (supports both built-in and custom: prefixed deck IDs)
        let deck1 = self.resolve_deck(&config.player1_deck_id)?;
        let deck2 = self.resolve_deck(&config.player2_deck_id)?;

        // Parse bot types
        let bot1_type: BotType = config
            .player1_bot_type
            .parse()
            .map_err(|e| format!("Invalid bot type for player 1: {}", e))?;

        let bot2_type: BotType = config
            .player2_bot_type
            .parse()
            .map_err(|e| format!("Invalid bot type for player 2: {}", e))?;

        // Get bot display names
        let bot1_name = Self::bot_display_name(&bot1_type);
        let bot2_name = Self::bot_display_name(&bot2_type);

        // Create game client
        let mut client = GameClient::new(self.card_db.clone());

        // Use provided seed or generate random
        let mut rng = rand::thread_rng();
        let game_seed = config.seed.unwrap_or_else(|| rng.gen::<u64>());
        let bot1_seed = rng.gen::<u64>();
        let bot2_seed = rng.gen::<u64>();

        // Start game (deck definitions include commanders, player 1 always goes first in spectator mode)
        client.start_game(&deck1, &deck2, game_seed);

        let match_id = Uuid::new_v4().to_string();

        // Capture initial state (from player 1's perspective for consistency)
        let initial_state = self.client_to_spectator_dto(&client, &match_id);

        // Storage for actions
        let mut actions: Vec<SpectatorAction> = Vec::new();

        // Bot configurations - use configured values
        let mcts_config = MctsConfig {
            simulations: config.mcts_simulations,
            ..MctsConfig::default()
        };
        let alphabeta_config = AlphaBetaConfig {
            max_depth: config.alphabeta_depth,
            ..AlphaBetaConfig::default()
        };

        // Play game to completion with safety limit to prevent infinite loops
        let mut action_count = 0;
        while !client.is_game_over() && action_count < MAX_ACTIONS_PER_GAME {
            let current_player = client
                .current_player()
                .ok_or_else(|| "No active player".to_string())?;
            let player_num = current_player.0 + 1; // Convert 0-indexed to 1-indexed (1 or 2)
            let turn = client.turn_number();

            // Determine which bot to use
            let (bot_type, bot_seed) = if current_player == PlayerId::PLAYER_ONE {
                (&bot1_type, bot1_seed)
            } else {
                (&bot2_type, bot2_seed)
            };

            // Create bot and get action
            let start = Instant::now();
            let mut bot = create_bot(
                &self.card_db,
                bot_type,
                None, // No custom weights
                &mcts_config,
                &alphabeta_config,
                bot_seed,
            );

            let action = client
                .select_bot_action(&mut *bot)
                .ok_or_else(|| "Game ended unexpectedly".to_string())?;

            let thinking_time_ms = start.elapsed().as_millis() as u64;

            // Apply action and capture events
            let action_index = action.to_index();
            let events = client
                .apply_action_by_index(action_index)
                .map_err(|e| e.to_string())?;

            // Convert action and events to DTOs
            let action_info = action_to_info(&action, action_index);
            let event_dtos: Vec<GameEventDto> = events.iter().map(game_event_to_dto).collect();
            let state_after = self.client_to_spectator_dto(&client, &match_id);

            // Store action (no MCTS thinking data for now - can be added later)
            actions.push(SpectatorAction {
                turn,
                player: player_num,
                action: action_info,
                state_after,
                events: event_dtos,
                thinking: None, // TODO: Add MCTS introspection when available
                thinking_time_ms,
            });

            action_count += 1;
        }

        // Log warning if action limit was hit without game completion
        if action_count >= MAX_ACTIONS_PER_GAME && !client.is_game_over() {
            eprintln!(
                "Warning: Spectator game exceeded {} action limit without completion (seed: {:?})",
                MAX_ACTIONS_PER_GAME,
                config.seed
            );
        }

        // Get final result
        let result = client.get_result();
        let final_state = client.get_state();

        let (winner, reason) = match result {
            Some(cardgame::core::state::GameResult::Win { winner, reason }) => {
                // Convert from 0-indexed to 1-indexed (1 or 2)
                (Some(winner.0 + 1), format!("{:?}", reason))
            }
            Some(cardgame::core::state::GameResult::Draw) => (None, "TurnLimit".to_string()),
            None => (None, "Unknown".to_string()),
        };

        let (p1_life, p2_life) = final_state
            .map(|s| (s.players[0].life, s.players[1].life))
            .unwrap_or((0, 0));

        let spectator_result = SpectatorResult {
            winner,
            reason,
            player1_final_life: p1_life,
            player2_final_life: p2_life,
        };

        Ok(SpectatorMatch {
            id: match_id,
            config,
            initial_state,
            actions,
            result: spectator_result,
            total_turns: client.turn_number(),
            player1_deck_name: deck1.name.clone(),
            player2_deck_name: deck2.name.clone(),
            player1_bot_name: bot1_name,
            player2_bot_name: bot2_name,
        })
    }

    /// Get display name for a bot type
    fn bot_display_name(bot_type: &BotType) -> String {
        match bot_type {
            BotType::Random => "Random Bot".to_string(),
            BotType::Greedy => "Greedy Bot".to_string(),
            BotType::Mcts => "MCTS Bot".to_string(),
            BotType::AlphaBeta => "Alpha-Beta Bot".to_string(),
            BotType::AgentSpecialist(faction) => format!("Agent ({:?})", faction),
            BotType::AgentGeneralist => "Agent (Generalist)".to_string(),
        }
    }

    /// Convert GameClient state to DTO for spectator mode (both hands visible)
    fn client_to_spectator_dto(&self, client: &GameClient, game_id: &str) -> GameStateDto {
        let state = match client.get_state() {
            Some(s) => s,
            None => {
                return GameStateDto {
                    id: game_id.to_string(),
                    turn: 0,
                    phase: "not_started".to_string(),
                    active_player: 0,
                    player: PlayerStateDto {
                        life: 0,
                        max_life: 30,
                        essence: 0,
                        max_essence: 0,
                        action_points: 0,
                        deck_count: 0,
                        hand: Vec::new(),
                        creatures: vec![None; 5],
                        supports: vec![None; 2],
                        commander: None,
                    },
                    opponent: PlayerStateDto {
                        life: 0,
                        max_life: 30,
                        essence: 0,
                        max_essence: 0,
                        action_points: 0,
                        deck_count: 0,
                        hand: Vec::new(),
                        creatures: vec![None; 5],
                        supports: vec![None; 2],
                        commander: None,
                    },
                    is_game_over: false,
                    winner: None,
                    game_over_reason: None,
                };
            }
        };

        let player1_state = &state.players[0];
        let player2_state = &state.players[1];

        // Get commander DTOs
        let player1_commander = commander_to_dto(state.get_commander(PlayerId::PLAYER_ONE), &self.card_db);
        let player2_commander = commander_to_dto(state.get_commander(PlayerId::PLAYER_TWO), &self.card_db);

        // Convert both hands (visible in spectator mode)
        let player1_hand: Vec<CardDto> = player1_state
            .hand
            .iter()
            .filter_map(|card_inst| {
                self.card_db
                    .get(card_inst.card_id)
                    .map(CardDto::from_card_def)
            })
            .collect();

        let player2_hand: Vec<CardDto> = player2_state
            .hand
            .iter()
            .filter_map(|card_inst| {
                self.card_db
                    .get(card_inst.card_id)
                    .map(CardDto::from_card_def)
            })
            .collect();

        // Convert creatures to slot-indexed arrays
        let player1_creatures = self.creatures_to_slots(player1_state, state.current_turn);
        let player2_creatures = self.creatures_to_slots(player2_state, state.current_turn);

        // Convert supports to slot-indexed arrays
        let player1_supports = self.supports_to_slots(player1_state);
        let player2_supports = self.supports_to_slots(player2_state);

        let active_player = state.active_player.0;

        let result = client.get_result();
        let winner = result.and_then(|r| match r {
            cardgame::core::state::GameResult::Win { winner, .. } => Some(winner.0),
            cardgame::core::state::GameResult::Draw => None,
        });

        let game_over_reason = result.map(|r| match r {
            cardgame::core::state::GameResult::Win { reason, .. } => format!("{:?}", reason),
            cardgame::core::state::GameResult::Draw => "draw".to_string(),
        });

        GameStateDto {
            id: game_id.to_string(),
            turn: state.current_turn,
            phase: format!("{:?}", state.phase),
            active_player,
            player: PlayerStateDto {
                life: player1_state.life,
                max_life: 30,
                essence: player1_state.current_essence,
                max_essence: player1_state.max_essence,
                action_points: player1_state.action_points,
                deck_count: player1_state.deck.len(),
                hand: player1_hand,
                creatures: player1_creatures,
                supports: player1_supports,
                commander: player1_commander,
            },
            opponent: PlayerStateDto {
                life: player2_state.life,
                max_life: 30,
                essence: player2_state.current_essence,
                max_essence: player2_state.max_essence,
                action_points: player2_state.action_points,
                deck_count: player2_state.deck.len(),
                hand: player2_hand,
                creatures: player2_creatures,
                supports: player2_supports,
                commander: player2_commander,
            },
            is_game_over: client.is_game_over(),
            winner,
            game_over_reason,
        }
    }

    /// Convert creatures to slot-indexed array
    fn creatures_to_slots(
        &self,
        player_state: &cardgame::core::state::PlayerState,
        current_turn: u16,
    ) -> Vec<Option<CreatureDto>> {
        let mut slots: Vec<Option<CreatureDto>> = vec![None; 5];
        for creature in player_state.creatures.iter() {
            // CardId(0) is a sentinel for token creatures (not in database)
            if creature.card_id.0 == 0 {
                let dto = CreatureDto::from_token(creature, current_turn);
                slots[creature.slot.0 as usize] = Some(dto);
            } else {
                match self.card_db.get(creature.card_id) {
                    Some(card) => {
                        let dto = CreatureDto::from_creature(creature, card, current_turn);
                        slots[creature.slot.0 as usize] = Some(dto);
                    }
                    None => {
                        eprintln!(
                            "Warning: Creature card ID {} not found in database",
                            creature.card_id.0
                        );
                    }
                }
            }
        }
        slots
    }

    /// Convert supports to slot-indexed array
    fn supports_to_slots(
        &self,
        player_state: &cardgame::core::state::PlayerState,
    ) -> Vec<Option<SupportDto>> {
        let mut slots: Vec<Option<SupportDto>> = vec![None; 2];
        for support in player_state.supports.iter() {
            match self.card_db.get(support.card_id) {
                Some(card) => {
                    let dto = SupportDto::from_support(support, card);
                    slots[support.slot.0 as usize] = Some(dto);
                }
                None => {
                    eprintln!(
                        "Warning: Support card ID {} not found in database",
                        support.card_id.0
                    );
                }
            }
        }
        slots
    }
}
