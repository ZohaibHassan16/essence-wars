//! Replay types for file storage and browser UI.

use serde::{Deserialize, Serialize};

/// Information about a saved replay for the browser UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayInfo {
    /// Full path to the replay file
    pub path: String,
    /// Just the filename
    pub filename: String,
    /// Unix timestamp when replay was saved
    pub timestamp: u64,
    /// Human-readable date string
    pub date_string: String,
    /// Player 1 deck display name
    pub player1_deck_name: String,
    /// Player 2 deck display name
    pub player2_deck_name: String,
    /// Player 1 type (e.g., "Human", "MCTS Bot")
    pub player1_type: String,
    /// Player 2 type (e.g., "Human", "MCTS Bot")
    pub player2_type: String,
    /// Winner (1 or 2, or None for draw)
    pub winner: Option<u8>,
    /// Total turns in the game
    pub total_turns: u16,
    /// Total number of actions
    pub total_actions: usize,
}

/// Metadata stored in replay files for quick loading without parsing the full match.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayMetadata {
    /// Engine version that created the replay
    pub engine_version: String,
    /// Unix timestamp when replay was saved
    pub timestamp: u64,
    /// Player 1 deck display name
    pub player1_deck_name: String,
    /// Player 2 deck display name
    pub player2_deck_name: String,
    /// Player 1 type description
    pub player1_type: String,
    /// Player 2 type description
    pub player2_type: String,
    /// Winner (1 or 2, or None for draw)
    pub winner: Option<u8>,
    /// Total turns
    pub total_turns: u16,
    /// Total actions
    pub total_actions: usize,
}

impl ReplayMetadata {
    pub fn new(
        player1_deck_name: String,
        player2_deck_name: String,
        player1_type: String,
        player2_type: String,
        winner: Option<u8>,
        total_turns: u16,
        total_actions: usize,
    ) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        Self {
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            player1_deck_name,
            player2_deck_name,
            player1_type,
            player2_type,
            winner,
            total_turns,
            total_actions,
        }
    }
}

/// Stored replay file format - wraps SpectatorMatch with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredReplay {
    /// Metadata for quick loading
    pub metadata: ReplayMetadata,
    /// The full spectator match data for playback
    pub match_data: super::SpectatorMatch,
}
