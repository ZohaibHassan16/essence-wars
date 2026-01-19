//! Replay types for game recording and playback.
//!
//! This module defines the data structures for storing game replays,
//! including all actions taken and optional analytics data.

use serde::{Deserialize, Serialize};
use crate::core::actions::Action;
use crate::core::state::{GameMode, GameResult, GameState};
use crate::core::types::CardId;

/// Header information for a replay file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayHeader {
    /// Engine version that created the replay.
    pub engine_version: String,
    /// Git commit hash (if available).
    pub git_hash: Option<String>,
    /// Random seed used for the game.
    pub seed: u64,
    /// Timestamp when the game was played (Unix timestamp).
    pub timestamp: u64,
    /// Game mode.
    pub mode: GameMode,
}

impl ReplayHeader {
    /// Create a new header with current engine version.
    pub fn new(seed: u64, mode: GameMode) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        Self {
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            git_hash: option_env!("GIT_HASH").map(String::from),
            seed,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            mode,
        }
    }
}

/// Configuration for a player in the replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    /// Player name or identifier.
    pub name: String,
    /// Type of player (e.g., "human", "mcts", "greedy").
    pub player_type: String,
    /// Deck used by the player.
    pub deck: Vec<CardId>,
    /// Optional deck name/ID.
    pub deck_name: Option<String>,
}

/// A single action in the replay with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayAction {
    /// Turn number when action was taken.
    pub turn: u16,
    /// The action taken.
    pub action: Action,
    /// Thinking time in microseconds (for bots).
    pub thinking_time_us: Option<u64>,
}

/// Result data for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResultExport {
    /// Winner player index (0 or 1), or None for draw.
    pub winner: Option<u8>,
    /// Reason for the game ending.
    pub reason: String,
    /// Total turns played.
    pub total_turns: u16,
    /// Final life totals.
    pub final_life: [i16; 2],
}

impl From<Option<GameResult>> for GameResultExport {
    fn from(result: Option<GameResult>) -> Self {
        match result {
            Some(GameResult::Win { winner, reason }) => Self {
                winner: Some(winner.0),
                reason: format!("{:?}", reason),
                total_turns: 0, // Must be set separately
                final_life: [0, 0], // Must be set separately
            },
            Some(GameResult::Draw) | None => Self {
                winner: None,
                reason: "Draw".to_string(),
                total_turns: 0,
                final_life: [0, 0],
            },
        }
    }
}

/// Analytics data for AI decision visualization (optional).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplayAnalytics {
    /// Total thinking time in milliseconds.
    pub total_thinking_time_ms: u64,
    /// Number of MCTS simulations (if applicable).
    pub total_simulations: Option<u64>,
    /// Win rate estimates at each turn (for AI visualization).
    pub win_rate_history: Vec<f32>,
}

/// A complete game replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameReplay {
    /// Header with metadata.
    pub header: ReplayHeader,
    /// Player 1 configuration.
    pub player1: PlayerConfig,
    /// Player 2 configuration.
    pub player2: PlayerConfig,
    /// List of actions taken during the game.
    pub actions: Vec<ReplayAction>,
    /// Game result.
    pub result: GameResultExport,
    /// Optional state snapshots for fast seeking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshots: Option<Vec<GameState>>,
    /// Optional analytics data for AI visualization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics: Option<ReplayAnalytics>,
}

impl GameReplay {
    /// Create a new empty replay with the given configuration.
    pub fn new(
        seed: u64,
        mode: GameMode,
        player1: PlayerConfig,
        player2: PlayerConfig,
    ) -> Self {
        Self {
            header: ReplayHeader::new(seed, mode),
            player1,
            player2,
            actions: Vec::new(),
            result: GameResultExport {
                winner: None,
                reason: "In Progress".to_string(),
                total_turns: 0,
                final_life: [30, 30],
            },
            snapshots: None,
            analytics: None,
        }
    }

    /// Add an action to the replay.
    pub fn add_action(&mut self, turn: u16, action: Action, thinking_time_us: Option<u64>) {
        self.actions.push(ReplayAction {
            turn,
            action,
            thinking_time_us,
        });
    }

    /// Set the game result.
    pub fn set_result(&mut self, result: Option<GameResult>, total_turns: u16, final_life: [i16; 2]) {
        let mut export: GameResultExport = result.into();
        export.total_turns = total_turns;
        export.final_life = final_life;
        self.result = export;
    }

    /// Get the total number of actions.
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    /// Check if the game is complete.
    pub fn is_complete(&self) -> bool {
        self.result.reason != "In Progress"
    }
}

/// Builder for creating GameReplay instances.
pub struct ReplayBuilder {
    replay: GameReplay,
    include_snapshots: bool,
    snapshot_interval: usize,
    state_history: Vec<GameState>,
}

impl ReplayBuilder {
    /// Create a new replay builder.
    pub fn new(
        seed: u64,
        mode: GameMode,
        player1: PlayerConfig,
        player2: PlayerConfig,
    ) -> Self {
        Self {
            replay: GameReplay::new(seed, mode, player1, player2),
            include_snapshots: false,
            snapshot_interval: 5, // Every 5 turns
            state_history: Vec::new(),
        }
    }

    /// Enable state snapshots for fast seeking.
    pub fn with_snapshots(mut self, interval: usize) -> Self {
        self.include_snapshots = true;
        self.snapshot_interval = interval;
        self
    }

    /// Enable analytics tracking.
    pub fn with_analytics(mut self) -> Self {
        self.replay.analytics = Some(ReplayAnalytics::default());
        self
    }

    /// Record an action.
    pub fn record_action(&mut self, turn: u16, action: Action, thinking_time_us: Option<u64>) {
        self.replay.add_action(turn, action, thinking_time_us);
    }

    /// Record a state snapshot (if enabled).
    pub fn record_state(&mut self, state: &GameState) {
        if self.include_snapshots && self.state_history.len().is_multiple_of(self.snapshot_interval) {
            self.state_history.push(state.clone());
        }
    }

    /// Update analytics with thinking time.
    pub fn record_thinking_time(&mut self, time_ms: u64) {
        if let Some(ref mut analytics) = self.replay.analytics {
            analytics.total_thinking_time_ms += time_ms;
        }
    }

    /// Update analytics with win rate estimate.
    pub fn record_win_rate(&mut self, win_rate: f32) {
        if let Some(ref mut analytics) = self.replay.analytics {
            analytics.win_rate_history.push(win_rate);
        }
    }

    /// Finalize the replay with the game result.
    pub fn finalize(mut self, result: Option<GameResult>, total_turns: u16, final_life: [i16; 2]) -> GameReplay {
        self.replay.set_result(result, total_turns, final_life);

        if self.include_snapshots && !self.state_history.is_empty() {
            self.replay.snapshots = Some(self.state_history);
        }

        self.replay
    }
}
