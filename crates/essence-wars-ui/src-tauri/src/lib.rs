//! Essence Wars Desktop Client
//!
//! A Tauri-based desktop client for the Essence Wars card game.

mod commands;
mod screenshot_server;
mod state;

pub mod ai;

use commands::*;
use screenshot_server::ScreenshotState;
use state::{CustomDeckManager, GameManager, ReplayManager};
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set data directory for bundled resources
    #[cfg(not(debug_assertions))]
    {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // On Windows, Tauri bundles resources next to the exe
                let bundled_data = exe_dir.join("data");
                if bundled_data.exists() {
                    std::env::set_var("CARDGAME_DATA_DIR", bundled_data);
                }
            }
        }
    }

    // Initialize game manager
    let game_manager = GameManager::new().expect("Failed to initialize game manager");

    // Initialize replay manager (uses same card_db and deck_registry)
    let replay_manager = ReplayManager::new(
        game_manager.card_db(),
        game_manager.deck_registry(),
    )
    .expect("Failed to initialize replay manager");

    // Initialize custom deck manager (uses same card_db)
    let custom_deck_manager = CustomDeckManager::new(game_manager.card_db())
        .expect("Failed to initialize custom deck manager");

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
        .manage(custom_deck_manager)
        .manage(screenshot_state)
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
            // Deck builder
            list_all_cards,
            list_commanders,
            list_custom_decks,
            load_custom_deck,
            save_custom_deck,
            delete_custom_deck,
            validate_custom_deck,
            calculate_deck_playstyle,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
