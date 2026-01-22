//! Tauri IPC commands for replay functionality.

use tauri::State;

use crate::state::{GameManager, ReplayInfo, ReplayManager, SpectatorMatch};

/// Save a replay from a completed game session.
///
/// Returns the path to the saved replay file.
#[tauri::command]
pub fn save_replay(
    game_id: String,
    name: Option<String>,
    game_manager: State<'_, GameManager>,
    replay_manager: State<'_, ReplayManager>,
) -> Result<String, String> {
    // Get the game session
    let games = game_manager.games_read();
    let session = games
        .get(&game_id)
        .ok_or_else(|| format!("Game not found: {}", game_id))?;

    // Save the replay
    replay_manager.save_from_session(session, name.as_deref())
}

/// List all saved replays.
#[tauri::command]
pub fn list_replays(replay_manager: State<'_, ReplayManager>) -> Result<Vec<ReplayInfo>, String> {
    replay_manager.list_replays()
}

/// Load a replay file for playback.
#[tauri::command]
pub fn load_replay(
    path: String,
    replay_manager: State<'_, ReplayManager>,
) -> Result<SpectatorMatch, String> {
    replay_manager.load_replay(&path)
}

/// Delete a replay file.
#[tauri::command]
pub fn delete_replay(path: String, replay_manager: State<'_, ReplayManager>) -> Result<(), String> {
    replay_manager.delete_replay(&path)
}

/// Save a spectator match (AI vs AI) as a replay.
///
/// Returns the path to the saved replay file.
#[tauri::command]
pub fn save_spectator_replay(
    spectator_match: SpectatorMatch,
    name: Option<String>,
    replay_manager: State<'_, ReplayManager>,
) -> Result<String, String> {
    replay_manager.save_spectator_match(&spectator_match, name.as_deref())
}
