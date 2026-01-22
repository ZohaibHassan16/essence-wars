//! MCP sync commands for displaying game state synced from MCP server.

use crate::screenshot_server::ScreenshotState;
use crate::state::GameStateDto;
use std::sync::Arc;
use tauri::State;

/// Response containing synced game state and metadata.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSyncedState {
    pub state: GameStateDto,
    pub age_ms: u64,
    pub timestamp: u64,
}

/// Get the current MCP-synced game state, if any.
///
/// Returns the game state that was pushed from the MCP server via /sync_state,
/// or null if no state has been synced.
#[tauri::command]
pub fn get_mcp_synced_state(
    screenshot_state: State<'_, Arc<ScreenshotState>>,
) -> Option<McpSyncedState> {
    screenshot_state.get_synced_state().map(|cached| McpSyncedState {
        state: cached.state,
        age_ms: cached.synced_at.elapsed().as_millis() as u64,
        timestamp: cached.timestamp,
    })
}

/// Check if there is an MCP-synced game state available.
#[tauri::command]
pub fn has_mcp_synced_state(
    screenshot_state: State<'_, Arc<ScreenshotState>>,
) -> bool {
    screenshot_state.get_synced_state().is_some()
}

/// Clear the MCP-synced game state.
#[tauri::command]
pub fn clear_mcp_synced_state(
    screenshot_state: State<'_, Arc<ScreenshotState>>,
) {
    screenshot_state.clear_synced_state();
}
