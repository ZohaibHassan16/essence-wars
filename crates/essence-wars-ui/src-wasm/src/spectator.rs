//! Spectator mode for AI vs AI matches.
//!
//! Computes complete matches between two AI players for playback.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use wasm_bindgen::prelude::*;

use cardgame::bots::{create_bot, AlphaBetaConfig, BotType, MctsConfig};
use cardgame::cards::CardDatabase;
use cardgame::client_api::GameClient;
use cardgame::embedded_data::{load_embedded_cards_with_commanders, load_embedded_decks};
use cardgame::core::config::game::MAX_ACTIONS_PER_GAME;
use cardgame::PlayerId;

use crate::types::*;

// =============================================================================
// Spectator Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectatorConfig {
    pub player1_deck_id: String,
    pub player2_deck_id: String,
    pub player1_bot_type: String,
    pub player2_bot_type: String,
    pub seed: Option<u64>,
    #[serde(default = "default_mcts_sims")]
    pub mcts_simulations: u32,
    #[serde(default = "default_ab_depth")]
    pub alphabeta_depth: u32,
}

fn default_mcts_sims() -> u32 {
    200 // Reduced for web
}

fn default_ab_depth() -> u32 {
    4 // Reduced for web
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectatorMatch {
    pub id: String,
    pub config: SpectatorConfig,
    pub initial_state: GameStateDto,
    pub actions: Vec<SpectatorAction>,
    pub result: SpectatorResult,
    pub total_turns: u16,
    pub player1_deck_name: String,
    pub player2_deck_name: String,
    pub player1_bot_name: String,
    pub player2_bot_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectatorAction {
    pub turn: u16,
    pub player: u8,
    pub action: ActionInfo,
    pub state_after: GameStateDto,
    pub events: Vec<GameEventDto>,
    pub thinking_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectatorResult {
    pub winner: Option<u8>,
    pub reason: String,
    pub player1_final_life: i16,
    pub player2_final_life: i16,
}

// =============================================================================
// Spectator Functions
// =============================================================================

/// Compute a complete spectator match.
///
/// This is a standalone function (not on WasmGameManager) to allow
/// simpler calling from JavaScript.
#[wasm_bindgen]
pub fn compute_spectator_match(config_json: &str) -> Result<String, JsError> {
    let config: SpectatorConfig = serde_json::from_str(config_json)
        .map_err(|e| JsError::new(&format!("Invalid config: {}", e)))?;

    // Load game data
    let card_db = Arc::new(
        load_embedded_cards_with_commanders()
            .map_err(|e| JsError::new(&format!("Failed to load cards: {}", e)))?,
    );
    let deck_registry = Arc::new(
        load_embedded_decks()
            .map_err(|e| JsError::new(&format!("Failed to load decks: {}", e)))?,
    );

    // Get decks
    let deck1 = deck_registry
        .get(&config.player1_deck_id)
        .ok_or_else(|| JsError::new(&format!("Deck not found: {}", config.player1_deck_id)))?
        .clone();
    let deck2 = deck_registry
        .get(&config.player2_deck_id)
        .ok_or_else(|| JsError::new(&format!("Deck not found: {}", config.player2_deck_id)))?
        .clone();

    // Parse bot types
    let bot1_type: BotType = config
        .player1_bot_type
        .parse()
        .map_err(|e| JsError::new(&format!("Invalid bot type for player 1: {:?}", e)))?;
    let bot2_type: BotType = config
        .player2_bot_type
        .parse()
        .map_err(|e| JsError::new(&format!("Invalid bot type for player 2: {:?}", e)))?;

    // Get bot display names
    let bot1_name = bot_display_name(&bot1_type);
    let bot2_name = bot_display_name(&bot2_type);

    // Create game client
    let mut client = GameClient::new(card_db.clone());

    // Generate seeds
    let game_seed = config.seed.unwrap_or_else(|| {
        use rand::Rng;
        rand::thread_rng().gen()
    });
    let bot1_seed: u64 = {
        use rand::Rng;
        rand::thread_rng().gen()
    };
    let bot2_seed: u64 = {
        use rand::Rng;
        rand::thread_rng().gen()
    };

    // Start game
    client.start_game(&deck1, &deck2, game_seed);

    let match_id = format!("spectator_{}", game_seed);

    // Capture initial state
    let initial_state = client_to_spectator_dto(&client, &match_id, &card_db);

    // Storage for actions
    let mut actions: Vec<SpectatorAction> = Vec::new();

    // Bot configurations (reduced for web)
    let mcts_config = MctsConfig {
        simulations: config.mcts_simulations,
        ..MctsConfig::default()
    };
    let alphabeta_config = AlphaBetaConfig {
        max_depth: config.alphabeta_depth,
        ..AlphaBetaConfig::default()
    };

    // Play game to completion
    let mut action_count = 0;
    while !client.is_game_over() && action_count < MAX_ACTIONS_PER_GAME {
        let current_player = client
            .current_player()
            .ok_or_else(|| JsError::new("No active player"))?;
        let player_num = current_player.0 + 1;
        let turn = client.turn_number();

        // Determine which bot to use
        let (bot_type, bot_seed) = if current_player == PlayerId::PLAYER_ONE {
            (&bot1_type, bot1_seed)
        } else {
            (&bot2_type, bot2_seed)
        };

        // Create bot and get action
        let start = js_sys::Date::now();
        let mut bot = create_bot(
            &card_db,
            bot_type,
            None,
            &mcts_config,
            &alphabeta_config,
            bot_seed,
        );

        let action = client
            .select_bot_action(&mut *bot)
            .ok_or_else(|| JsError::new("Game ended unexpectedly"))?;

        let thinking_time_ms = (js_sys::Date::now() - start) as u64;

        // Apply action and capture events
        let action_index = action.to_index();
        let events = client
            .apply_action_by_index(action_index)
            .map_err(|e| JsError::new(&format!("Action error: {}", e)))?;

        // Convert to DTOs
        let action_info = action_to_info(&action, action_index);
        let event_dtos: Vec<GameEventDto> = events.iter().map(game_event_to_dto).collect();
        let state_after = client_to_spectator_dto(&client, &match_id, &card_db);

        actions.push(SpectatorAction {
            turn,
            player: player_num,
            action: action_info,
            state_after,
            events: event_dtos,
            thinking_time_ms,
        });

        action_count += 1;
    }

    // Get final result
    let result = client.get_result();
    let final_state = client.get_state();

    let (winner, reason) = match result {
        Some(cardgame::core::state::GameResult::Win { winner, reason }) => {
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

    let spectator_match = SpectatorMatch {
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
    };

    serde_json::to_string(&spectator_match)
        .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

// =============================================================================
// Helpers
// =============================================================================

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

fn client_to_spectator_dto(
    client: &GameClient,
    game_id: &str,
    card_db: &CardDatabase,
) -> GameStateDto {
    let state = match client.get_state() {
        Some(s) => s,
        None => return GameStateDto::empty(game_id),
    };

    let player1_state = &state.players[0];
    let player2_state = &state.players[1];

    let player1_commander = state
        .get_commander(PlayerId::PLAYER_ONE)
        .and_then(|cmd_id| card_db.get_commander(cmd_id))
        .map(CommanderDto::from_commander);

    let player2_commander = state
        .get_commander(PlayerId::PLAYER_TWO)
        .and_then(|cmd_id| card_db.get_commander(cmd_id))
        .map(CommanderDto::from_commander);

    // Both hands visible in spectator mode
    let player1_hand: Vec<CardDto> = player1_state
        .hand
        .iter()
        .filter_map(|card_inst| card_db.get(card_inst.card_id).map(CardDto::from_card))
        .collect();

    let player2_hand: Vec<CardDto> = player2_state
        .hand
        .iter()
        .filter_map(|card_inst| card_db.get(card_inst.card_id).map(CardDto::from_card))
        .collect();

    let player1_creatures = creatures_to_slots(player1_state, state.current_turn, card_db);
    let player2_creatures = creatures_to_slots(player2_state, state.current_turn, card_db);

    let player1_supports = supports_to_slots(player1_state, card_db);
    let player2_supports = supports_to_slots(player2_state, card_db);

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
            essence_extracted: player1_state.total_damage_dealt,
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
            essence_extracted: player2_state.total_damage_dealt,
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

fn creatures_to_slots(
    player_state: &cardgame::core::state::PlayerState,
    current_turn: u16,
    card_db: &CardDatabase,
) -> Vec<Option<CreatureDto>> {
    let mut slots: Vec<Option<CreatureDto>> = vec![None; 5];
    for creature in player_state.creatures.iter() {
        if creature.card_id.0 == 0 {
            let dto = CreatureDto::from_token(creature, current_turn);
            slots[creature.slot.0 as usize] = Some(dto);
        } else if let Some(card) = card_db.get(creature.card_id) {
            let dto = CreatureDto::from_creature(creature, card, current_turn);
            slots[creature.slot.0 as usize] = Some(dto);
        }
    }
    slots
}

fn supports_to_slots(
    player_state: &cardgame::core::state::PlayerState,
    card_db: &CardDatabase,
) -> Vec<Option<SupportDto>> {
    let mut slots: Vec<Option<SupportDto>> = vec![None; 2];
    for support in player_state.supports.iter() {
        if let Some(card) = card_db.get(support.card_id) {
            let dto = SupportDto::from_support(support, card);
            slots[support.slot.0 as usize] = Some(dto);
        }
    }
    slots
}
