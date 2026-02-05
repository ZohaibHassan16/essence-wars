//! Spectator mode Tauri commands.
//!
//! Commands for computing and managing AI vs AI spectator matches.

use std::sync::Arc;
use crate::state::{CustomDeckManager, GameManager, SpectatorComputer, SpectatorConfig, SpectatorMatch};
use tauri::State;

/// Compute a complete spectator match between two AI players.
///
/// This runs the entire match to completion and returns all actions
/// with their associated game states and events for playback.
///
/// The computation runs in a background thread to avoid blocking the UI.
/// Supports custom decks with "custom:" prefix in deck IDs.
#[tauri::command]
pub async fn compute_spectator_match(
    config: SpectatorConfig,
    game_manager: State<'_, GameManager>,
    custom_deck_manager: State<'_, CustomDeckManager>,
) -> Result<SpectatorMatch, String> {
    // Clone the custom deck manager so it can be moved into the blocking task
    let custom_manager = Arc::new(custom_deck_manager.inner().clone());
    let computer = SpectatorComputer::new(game_manager.inner(), Some(custom_manager));

    // Run the CPU-intensive game simulation in a blocking thread
    tokio::task::spawn_blocking(move || computer.compute_match(config))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}
