//! WebAssembly bindings for Essence Wars game engine.
//!
//! This module provides a complete WASM interface for the game engine,
//! allowing web clients to play the game without a backend server.
//!
//! All functions accept and return JSON strings for simplicity and
//! compatibility with JavaScript.

use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use cardgame::bots::{create_bot, AlphaBetaConfig, BotType, GreedyBot, MctsConfig};
use cardgame::cards::CardDatabase;
use cardgame::client_api::GameClient;
use cardgame::decks::DeckRegistry;
use cardgame::embedded_data::{load_embedded_cards_with_commanders, load_embedded_decks};
use cardgame::{Action, CardId, PlayerId};

mod types;
mod spectator;
mod deck_builder;

pub use types::*;
pub use spectator::*;
pub use deck_builder::*;

/// Set up console error panic hook for better error messages in browser.
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Represents an active game session in WASM.
struct GameSession {
    client: GameClient,
    player_id: PlayerId,
    opponent_bot_type: BotType,
    bot_seed: u64,
    action_indices: Vec<u8>,
    original_config: GameConfig,
    game_seed: u64,
}

/// Main game manager for WASM environment.
///
/// This struct holds all game state and provides the public API
/// for JavaScript to interact with the game engine.
#[wasm_bindgen]
pub struct WasmGameManager {
    card_db: Arc<CardDatabase>,
    deck_registry: Arc<DeckRegistry>,
    games: HashMap<String, GameSession>,
    next_id: u32,
}

#[wasm_bindgen]
impl WasmGameManager {
    /// Create a new WasmGameManager.
    ///
    /// This loads all card and deck data from embedded resources.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmGameManager, JsError> {
        let card_db = load_embedded_cards_with_commanders()
            .map_err(|e| JsError::new(&format!("Failed to load cards: {}", e)))?;
        let deck_registry = load_embedded_decks()
            .map_err(|e| JsError::new(&format!("Failed to load decks: {}", e)))?;

        Ok(Self {
            card_db: Arc::new(card_db),
            deck_registry: Arc::new(deck_registry),
            games: HashMap::new(),
            next_id: 1,
        })
    }

    // =========================================================================
    // Game Setup
    // =========================================================================

    /// List all available decks.
    ///
    /// Returns a JSON array of DeckInfo objects.
    #[wasm_bindgen]
    pub fn list_decks(&self) -> String {
        let decks: Vec<DeckInfo> = self
            .deck_registry
            .decks()
            .map(|deck| {
                let faction = deck
                    .faction()
                    .map(|f| deck_faction_to_string(f).to_string())
                    .unwrap_or_else(|| "neutral".to_string());

                let commander = self
                    .card_db
                    .get_commander(deck.commander_id())
                    .map(CommanderDto::from_commander);

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
            .collect();

        serde_json::to_string(&decks).unwrap_or_else(|_| "[]".to_string())
    }

    /// List available bot types.
    ///
    /// Returns a JSON array of BotInfo objects.
    #[wasm_bindgen]
    pub fn list_bots(&self) -> String {
        let bots = vec![
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
                description: "Uses Monte Carlo Tree Search. Strong but slower on web.".to_string(),
            },
            BotInfo {
                id: "alphabeta".to_string(),
                name: "Alpha-Beta Bot".to_string(),
                description: "Uses minimax search with pruning. Strong but slower on web."
                    .to_string(),
            },
        ];

        serde_json::to_string(&bots).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get all cards in a deck.
    ///
    /// Returns a JSON array of CardDto objects.
    #[wasm_bindgen]
    pub fn get_deck_cards(&self, deck_id: &str) -> Result<String, JsError> {
        let deck = self
            .deck_registry
            .get(deck_id)
            .ok_or_else(|| JsError::new(&format!("Deck not found: {}", deck_id)))?;

        let cards: Vec<CardDto> = deck
            .cards
            .iter()
            .filter_map(|&card_id| {
                self.card_db
                    .get(CardId(card_id))
                    .map(CardDto::from_card)
            })
            .collect();

        serde_json::to_string(&cards)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    // =========================================================================
    // Game Session
    // =========================================================================

    /// Start a new game.
    ///
    /// Takes a JSON GameConfig and returns a JSON GameStateDto.
    #[wasm_bindgen]
    pub fn new_game(&mut self, config_json: &str) -> Result<String, JsError> {
        let config: GameConfig = serde_json::from_str(config_json)
            .map_err(|e| JsError::new(&format!("Invalid config: {}", e)))?;

        // Get decks
        let player_deck = self
            .deck_registry
            .get(&config.player_deck_id)
            .ok_or_else(|| JsError::new(&format!("Player deck not found: {}", config.player_deck_id)))?
            .clone();

        let opponent_deck = self
            .deck_registry
            .get(&config.opponent_deck_id)
            .ok_or_else(|| JsError::new(&format!("Opponent deck not found: {}", config.opponent_deck_id)))?
            .clone();

        // Parse bot type
        let bot_type: BotType = config
            .opponent_bot_type
            .parse()
            .map_err(|e| JsError::new(&format!("Unknown bot type: {:?}", e)))?;

        // Create game client
        let mut client = GameClient::new(self.card_db.clone());

        // Determine who goes first
        let player_first = config.player_goes_first.unwrap_or(true);

        // Generate seeds
        let game_seed = config.seed.unwrap_or_else(|| {
            use rand::Rng;
            rand::thread_rng().gen()
        });
        let bot_seed = {
            use rand::Rng;
            rand::thread_rng().gen()
        };

        // Start game
        if player_first {
            client.start_game(&player_deck, &opponent_deck, game_seed);
        } else {
            client.start_game(&opponent_deck, &player_deck, game_seed);
        }

        // Generate game ID
        let game_id = format!("game_{}", self.next_id);
        self.next_id += 1;

        let player_id = if player_first {
            PlayerId::PLAYER_ONE
        } else {
            PlayerId::PLAYER_TWO
        };

        // Convert to DTO before storing
        let state_dto = self.client_to_dto(&client, &game_id, player_id);

        // Store session
        let session = GameSession {
            client,
            player_id,
            opponent_bot_type: bot_type,
            bot_seed,
            action_indices: Vec::new(),
            original_config: config,
            game_seed,
        };
        self.games.insert(game_id.clone(), session);

        serde_json::to_string(&state_dto)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    /// Get current game state.
    ///
    /// Returns a JSON GameStateDto.
    #[wasm_bindgen]
    pub fn get_game_state(&self, game_id: &str) -> Result<String, JsError> {
        let session = self
            .games
            .get(game_id)
            .ok_or_else(|| JsError::new(&format!("Game not found: {}", game_id)))?;

        let state_dto = self.client_to_dto(&session.client, game_id, session.player_id);

        serde_json::to_string(&state_dto)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    /// Get legal actions for the current player.
    ///
    /// Returns a JSON array of ActionInfo objects.
    #[wasm_bindgen]
    pub fn get_legal_actions(&self, game_id: &str) -> Result<String, JsError> {
        let session = self
            .games
            .get(game_id)
            .ok_or_else(|| JsError::new(&format!("Game not found: {}", game_id)))?;

        let actions = session.client.get_legal_actions();
        let action_infos: Vec<ActionInfo> = actions
            .iter()
            .map(|action| action_to_info(action, action.to_index()))
            .collect();

        serde_json::to_string(&action_infos)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    /// Apply a player action.
    ///
    /// Returns a JSON GameStateUpdate with the new state and events.
    #[wasm_bindgen]
    pub fn apply_action(&mut self, game_id: &str, action_index: u8) -> Result<String, JsError> {
        // First, apply the action and collect needed data
        let (events, player_id) = {
            let session = self
                .games
                .get_mut(game_id)
                .ok_or_else(|| JsError::new(&format!("Game not found: {}", game_id)))?;

            // Apply action and get events
            let events = session
                .client
                .apply_action_by_index(action_index)
                .map_err(|e| JsError::new(&format!("Action error: {}", e)))?;

            // Store action index for undo
            session.action_indices.push(action_index);

            (events, session.player_id)
        };

        // Now we can borrow self immutably for client_to_dto
        let session = self
            .games
            .get(game_id)
            .ok_or_else(|| JsError::new(&format!("Game not found: {}", game_id)))?;

        let action = Action::from_index(action_index)
            .ok_or_else(|| JsError::new(&format!("Invalid action index: {}", action_index)))?;

        let action_info = action_to_info(&action, action_index);
        let state_dto = self.client_to_dto(&session.client, game_id, player_id);
        let event_dtos: Vec<GameEventDto> = events.iter().map(game_event_to_dto).collect();

        let update = GameStateUpdate {
            state: state_dto,
            last_action: Some(action_info),
            events: event_dtos,
        };

        serde_json::to_string(&update)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    /// Get AI's chosen move.
    ///
    /// Returns a JSON ActionInfo object.
    #[wasm_bindgen]
    pub fn get_ai_move(&self, game_id: &str) -> Result<String, JsError> {
        let session = self
            .games
            .get(game_id)
            .ok_or_else(|| JsError::new(&format!("Game not found: {}", game_id)))?;

        // Use reduced settings for web performance
        let mcts_config = MctsConfig {
            simulations: 200, // Reduced from 1000 for web
            ..MctsConfig::default()
        };
        let alphabeta_config = AlphaBetaConfig {
            max_depth: 4, // Reduced from 6 for web
            ..AlphaBetaConfig::default()
        };

        let mut bot = create_bot(
            &self.card_db,
            &session.opponent_bot_type,
            None,
            &mcts_config,
            &alphabeta_config,
            session.bot_seed,
        );

        let action = session
            .client
            .select_bot_action(&mut *bot)
            .ok_or_else(|| JsError::new("Game is over or not started"))?;

        let action_info = action_to_info(&action, action.to_index());

        serde_json::to_string(&action_info)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    /// Get AI hint with recommended action.
    ///
    /// Returns a JSON AiHintResponse.
    #[wasm_bindgen]
    pub fn get_ai_hint(&self, game_id: &str) -> Result<String, JsError> {
        let session = self
            .games
            .get(game_id)
            .ok_or_else(|| JsError::new(&format!("Game not found: {}", game_id)))?;

        let legal_actions = session.client.get_legal_actions();
        if legal_actions.is_empty() {
            return Err(JsError::new("No legal actions available"));
        }

        // Use GreedyBot for fast hints
        let mut greedy_bot = GreedyBot::new(&self.card_db, 42);

        let best_action = session
            .client
            .get_ai_hint(&mut greedy_bot)
            .ok_or_else(|| JsError::new("Failed to get AI hint"))?;

        let recommended = action_to_info(&best_action, best_action.to_index());

        let alternatives: Vec<AlternativeAction> = legal_actions
            .iter()
            .filter(|&&a| a != best_action)
            .take(4)
            .map(|action| AlternativeAction {
                action: action_to_info(action, action.to_index()),
                score: 0.0,
                score_delta: 0.0,
            })
            .collect();

        let response = AiHintResponse {
            recommended_action: recommended,
            score: 0.0,
            alternatives,
            thinking_time_ms: 0,
        };

        serde_json::to_string(&response)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    /// Undo the last action.
    ///
    /// Returns a JSON GameStateDto.
    #[wasm_bindgen]
    pub fn undo_action(&mut self, game_id: &str) -> Result<String, JsError> {
        // First, collect the data we need and perform the undo
        let player_id = {
            let session = self
                .games
                .get_mut(game_id)
                .ok_or_else(|| JsError::new(&format!("Game not found: {}", game_id)))?;

            if session.action_indices.is_empty() {
                return Err(JsError::new("No actions to undo"));
            }

            // Get decks
            let player_deck = self
                .deck_registry
                .get(&session.original_config.player_deck_id)
                .ok_or_else(|| JsError::new("Player deck not found"))?;
            let opponent_deck = self
                .deck_registry
                .get(&session.original_config.opponent_deck_id)
                .ok_or_else(|| JsError::new("Opponent deck not found"))?;

            let player_first = session.original_config.player_goes_first.unwrap_or(true);

            // Create new client and replay
            let mut new_client = GameClient::new(self.card_db.clone());
            if player_first {
                new_client.start_game(player_deck, opponent_deck, session.game_seed);
            } else {
                new_client.start_game(opponent_deck, player_deck, session.game_seed);
            }

            // Remove last action
            session.action_indices.pop();

            // Replay remaining actions
            for &action_index in &session.action_indices {
                let _ = new_client.apply_action_by_index(action_index);
            }

            session.client = new_client;
            session.player_id
        };

        // Now borrow immutably for client_to_dto
        let session = self
            .games
            .get(game_id)
            .ok_or_else(|| JsError::new(&format!("Game not found: {}", game_id)))?;

        let state_dto = self.client_to_dto(&session.client, game_id, player_id);

        serde_json::to_string(&state_dto)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    /// Check if undo is available.
    #[wasm_bindgen]
    pub fn can_undo(&self, game_id: &str) -> Result<bool, JsError> {
        let session = self
            .games
            .get(game_id)
            .ok_or_else(|| JsError::new(&format!("Game not found: {}", game_id)))?;

        Ok(!session.action_indices.is_empty())
    }

    /// End the game and get results.
    ///
    /// Returns a JSON GameResultDto.
    #[wasm_bindgen]
    pub fn end_game(&mut self, game_id: &str) -> Result<String, JsError> {
        let session = self
            .games
            .remove(game_id)
            .ok_or_else(|| JsError::new(&format!("Game not found: {}", game_id)))?;

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

        let result_dto = GameResultDto {
            winner,
            reason,
            final_turn: session.client.turn_number(),
            player_final_life: player_life,
            opponent_final_life: opponent_life,
        };

        serde_json::to_string(&result_dto)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    // =========================================================================
    // Private Helpers
    // =========================================================================

    /// Convert GameClient state to DTO.
    fn client_to_dto(&self, client: &GameClient, game_id: &str, player_id: PlayerId) -> GameStateDto {
        let state = match client.get_state() {
            Some(s) => s,
            None => return GameStateDto::empty(game_id),
        };

        let player_state = &state.players[player_id.index()];
        let opponent_state = &state.players[player_id.opponent().index()];

        let player_commander = state
            .get_commander(player_id)
            .and_then(|cmd_id| self.card_db.get_commander(cmd_id))
            .map(CommanderDto::from_commander);

        let opponent_commander = state
            .get_commander(player_id.opponent())
            .and_then(|cmd_id| self.card_db.get_commander(cmd_id))
            .map(CommanderDto::from_commander);

        // Convert player hand (visible)
        let player_hand: Vec<CardDto> = player_state
            .hand
            .iter()
            .filter_map(|card_inst| {
                self.card_db
                    .get(card_inst.card_id)
                    .map(CardDto::from_card)
            })
            .collect();

        // Convert opponent hand (hidden)
        let opponent_hand: Vec<CardDto> = (0..opponent_state.hand.len())
            .map(|_| CardDto::hidden())
            .collect();

        // Convert creatures
        let player_creatures = self.creatures_to_slots(player_state, state.current_turn);
        let opponent_creatures = self.creatures_to_slots(opponent_state, state.current_turn);

        // Convert supports
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
                essence_extracted: player_state.total_damage_dealt,
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
                essence_extracted: opponent_state.total_damage_dealt,
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

    /// Convert creatures to slot-indexed array.
    fn creatures_to_slots(
        &self,
        player_state: &cardgame::core::state::PlayerState,
        current_turn: u16,
    ) -> Vec<Option<CreatureDto>> {
        let mut slots: Vec<Option<CreatureDto>> = vec![None; 5];
        for creature in player_state.creatures.iter() {
            if creature.card_id.0 == 0 {
                // Token creature
                let dto = CreatureDto::from_token(creature, current_turn);
                slots[creature.slot.0 as usize] = Some(dto);
            } else if let Some(card) = self.card_db.get(creature.card_id) {
                let dto = CreatureDto::from_creature(creature, card, current_turn);
                slots[creature.slot.0 as usize] = Some(dto);
            }
        }
        slots
    }

    /// Convert supports to slot-indexed array.
    fn supports_to_slots(
        &self,
        player_state: &cardgame::core::state::PlayerState,
    ) -> Vec<Option<SupportDto>> {
        let mut slots: Vec<Option<SupportDto>> = vec![None; 2];
        for support in player_state.supports.iter() {
            if let Some(card) = self.card_db.get(support.card_id) {
                let dto = SupportDto::from_support(support, card);
                slots[support.slot.0 as usize] = Some(dto);
            }
        }
        slots
    }
}

impl Default for WasmGameManager {
    fn default() -> Self {
        Self::new().expect("Failed to create WasmGameManager")
    }
}
