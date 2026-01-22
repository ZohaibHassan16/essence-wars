//! Game manager for tracking active games.

use cardgame::bots::{create_bot, AlphaBetaConfig, BotType, MctsConfig};
use cardgame::client_api::GameClient;
use cardgame::{CardDatabase, DeckRegistry, PlayerId};
use parking_lot::RwLock;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::serialization::*;

/// Represents an active game session
pub struct GameSession {
    pub id: String,
    pub client: GameClient,
    pub player_id: PlayerId,
    /// Bot type for the opponent (bot created on-demand due to lifetime constraints)
    pub opponent_bot_type: BotType,
    /// Seed for bot randomization
    pub bot_seed: u64,
    pub action_history: Vec<ActionInfo>,
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
        let card_db = CardDatabase::load_from_directory(&cards_path)
            .map_err(|e| format!("Failed to load card database: {}", e))?;

        // Load deck registry - takes only a path
        let deck_registry = DeckRegistry::load_from_directory(data_dir.join("decks"))
            .map_err(|e| format!("Failed to load deck registry: {}", e))?;

        Ok(Self {
            games: RwLock::new(HashMap::new()),
            card_db: Arc::new(card_db),
            deck_registry: Arc::new(deck_registry),
        })
    }

    /// List all available decks
    pub fn list_decks(&self) -> Vec<DeckInfo> {
        self.deck_registry
            .decks()
            .map(|deck| {
                let faction = deck.faction()
                    .map(|f| faction_to_string(f).to_string())
                    .unwrap_or_else(|| "neutral".to_string());

                DeckInfo {
                    id: deck.id.clone(),
                    name: deck.name.clone(),
                    description: deck.description.clone(),
                    faction,
                    card_count: deck.cards.len(),
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
                description: "Uses Monte Carlo Tree Search. Strongest opponent.".to_string(),
            },
        ]
    }

    /// Create a new game session
    pub fn new_game(&self, config: GameConfig) -> Result<GameStateDto, String> {
        // Get decks
        let player_deck = self
            .deck_registry
            .get(&config.player_deck_id)
            .ok_or_else(|| format!("Player deck not found: {}", config.player_deck_id))?;

        let opponent_deck = self
            .deck_registry
            .get(&config.opponent_deck_id)
            .ok_or_else(|| format!("Opponent deck not found: {}", config.opponent_deck_id))?;

        // Parse bot type
        let bot_type: BotType = config
            .opponent_bot_type
            .parse()
            .map_err(|e| format!("Unknown bot type: {}", e))?;

        // Create game client
        let mut client = GameClient::new(self.card_db.clone());

        // Determine who goes first
        let player_first = config.player_goes_first.unwrap_or(true);

        // Convert deck cards to CardId vec
        let deck1_cards = player_deck.to_card_ids();
        let deck2_cards = opponent_deck.to_card_ids();

        // Start game with random seed
        let mut rng = rand::thread_rng();
        let game_seed = rng.gen::<u64>();
        let bot_seed = rng.gen::<u64>();

        if player_first {
            client.start_game(deck1_cards, deck2_cards, game_seed);
        } else {
            client.start_game(deck2_cards, deck1_cards, game_seed);
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
            id: game_id.clone(),
            client,
            player_id,
            opponent_bot_type: bot_type,
            bot_seed,
            action_history: Vec::new(),
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
                    },
                    is_game_over: false,
                    winner: None,
                    game_over_reason: None,
                };
            }
        };

        let player_state = &state.players[player_id.index()];
        let opponent_state = &state.players[player_id.opponent().index()];

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
