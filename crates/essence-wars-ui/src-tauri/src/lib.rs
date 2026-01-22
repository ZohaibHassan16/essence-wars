//! Essence Wars Desktop Client
//!
//! A Tauri-based desktop client for the Essence Wars card game.

mod commands;
mod state;

pub mod ai;

use commands::*;
use state::GameManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize game manager
    let game_manager = GameManager::new().expect("Failed to initialize game manager");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(game_manager)
        .invoke_handler(tauri::generate_handler![
            list_decks,
            list_bots,
            new_game,
            get_game_state,
            get_legal_actions,
            apply_action,
            get_ai_move,
            get_ai_hint,
            undo_action,
            can_undo,
            end_game,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
