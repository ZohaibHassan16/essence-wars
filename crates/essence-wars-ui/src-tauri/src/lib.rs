//! Essence Wars Desktop Client
//!
//! A Tauri-based desktop client for the Essence Wars card game.

mod commands;
mod screenshot_server;
mod state;

pub mod ai;

use commands::*;
use screenshot_server::ScreenshotState;
use state::{GameManager, ReplayManager};
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize game manager
    let game_manager = GameManager::new().expect("Failed to initialize game manager");

    // Initialize replay manager (uses same card_db and deck_registry)
    let replay_manager = ReplayManager::new(
        game_manager.card_db(),
        game_manager.deck_registry(),
    )
    .expect("Failed to initialize replay manager");

    // Initialize screenshot state for HTTP server
    let screenshot_state = Arc::new(ScreenshotState::new());
    let screenshot_state_for_server = screenshot_state.clone();

    // Start screenshot HTTP server in background thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            let port = screenshot_server::DEFAULT_PORT;
            if let Err(e) = screenshot_server::start_server(screenshot_state_for_server, port).await
            {
                eprintln!("Screenshot server error: {}", e);
            }
        });
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(game_manager)
        .manage(replay_manager)
        .manage(screenshot_state)
        .setup(|app| {
            // Register the main window title with screenshot state
            // The window title is "Essence Wars" as configured in tauri.conf.json
            let state: tauri::State<'_, Arc<ScreenshotState>> = app.state();
            state.set_window_title("Essence Wars".to_string());
            eprintln!("Screenshot server: registered window 'Essence Wars'");
            Ok(())
        })
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
            // Spectator mode
            compute_spectator_match,
            // Replay mode
            save_replay,
            save_spectator_replay,
            list_replays,
            load_replay,
            delete_replay,
            // MCP sync mode
            get_mcp_synced_state,
            has_mcp_synced_state,
            clear_mcp_synced_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
