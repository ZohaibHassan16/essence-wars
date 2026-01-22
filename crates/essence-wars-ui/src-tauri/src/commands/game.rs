//! Game-related Tauri commands.

use crate::state::{
    ActionInfo, BotInfo, DeckInfo, GameConfig, GameManager, GameResultDto, GameStateDto,
    GameStateUpdate,
};
use tauri::State;

/// List all available decks
#[tauri::command]
pub fn list_decks(game_manager: State<'_, GameManager>) -> Vec<DeckInfo> {
    game_manager.list_decks()
}

/// List available bot types
#[tauri::command]
pub fn list_bots(game_manager: State<'_, GameManager>) -> Vec<BotInfo> {
    game_manager.list_bots()
}

/// Start a new game
#[tauri::command]
pub fn new_game(
    config: GameConfig,
    game_manager: State<'_, GameManager>,
) -> Result<GameStateDto, String> {
    game_manager.new_game(config)
}

/// Get current game state
#[tauri::command]
pub fn get_game_state(
    game_id: String,
    game_manager: State<'_, GameManager>,
) -> Result<GameStateDto, String> {
    game_manager.get_game_state(&game_id)
}

/// Get legal actions for the current player
#[tauri::command]
pub fn get_legal_actions(
    game_id: String,
    game_manager: State<'_, GameManager>,
) -> Result<Vec<ActionInfo>, String> {
    game_manager.get_legal_actions(&game_id)
}

/// Apply a player action
#[tauri::command]
pub fn apply_action(
    game_id: String,
    action_index: u8,
    game_manager: State<'_, GameManager>,
) -> Result<GameStateUpdate, String> {
    game_manager.apply_action(&game_id, action_index)
}

/// Get AI's chosen move
#[tauri::command]
pub fn get_ai_move(
    game_id: String,
    game_manager: State<'_, GameManager>,
) -> Result<ActionInfo, String> {
    game_manager.get_ai_move(&game_id)
}

/// End the game and get results
#[tauri::command]
pub fn end_game(
    game_id: String,
    game_manager: State<'_, GameManager>,
) -> Result<GameResultDto, String> {
    game_manager.end_game(&game_id)
}
