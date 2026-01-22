//! Spectator mode Tauri commands.
//!
//! Commands for computing and managing AI vs AI spectator matches.

use crate::state::{GameManager, SpectatorComputer, SpectatorConfig, SpectatorMatch};
use tauri::State;

/// Compute a complete spectator match between two AI players.
///
/// This runs the entire match to completion and returns all actions
/// with their associated game states and events for playback.
///
/// The computation runs in a background thread to avoid blocking the UI.
#[tauri::command]
pub async fn compute_spectator_match(
    config: SpectatorConfig,
    game_manager: State<'_, GameManager>,
) -> Result<SpectatorMatch, String> {
    // Create a clonable computer from the manager
    let computer = SpectatorComputer::from_manager(game_manager.inner());

    // Run the CPU-intensive game simulation in a blocking thread
    tokio::task::spawn_blocking(move || computer.compute_match(config))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}
