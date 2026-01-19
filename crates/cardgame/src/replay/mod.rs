//! Replay system for game recording and playback.
//!
//! This module provides functionality to:
//! - Record games as replays with full action history
//! - Save and load replays from files (JSON or compressed)
//! - Step through replays for visualization
//! - Validate replay integrity
//!
//! # Example
//!
//! ## Recording a game
//!
//! ```ignore
//! use cardgame::replay::{GameReplay, PlayerConfig, ReplayBuilder};
//!
//! // Create a replay builder
//! let mut builder = ReplayBuilder::new(
//!     seed,
//!     GameMode::Attrition,
//!     PlayerConfig { name: "Player 1".into(), player_type: "human".into(), deck: deck1, deck_name: None },
//!     PlayerConfig { name: "Bot".into(), player_type: "mcts".into(), deck: deck2, deck_name: None },
//! );
//!
//! // Record actions as they happen
//! builder.record_action(turn, action, Some(thinking_time_us));
//!
//! // Finalize and save
//! let replay = builder.finalize(result, total_turns, final_life);
//! replay::save(&replay, "game.replay.json")?;
//! ```
//!
//! ## Playing back a replay
//!
//! ```ignore
//! use cardgame::replay::{load, ReplayIterator};
//!
//! // Load replay
//! let replay = replay::load("game.replay.json")?;
//!
//! // Create iterator
//! let mut iter = ReplayIterator::new(&replay, &card_db);
//!
//! // Step through actions
//! while let Some(step) = iter.step()? {
//!     println!("Turn {}: {:?}", step.turn, step.action);
//!     let state = iter.current_state().unwrap();
//!     // Update visualization...
//! }
//! ```
//!
//! # File Formats
//!
//! - `.replay.json` - Plain JSON, human-readable (~5-10KB per game)
//! - `.replay.json.gz` - Compressed JSON (~1-2KB per game)

mod types;
mod iterator;
mod io;

pub use types::{
    GameReplay, GameResultExport, PlayerConfig, ReplayAction, ReplayAnalytics, ReplayBuilder,
    ReplayHeader,
};
pub use iterator::{replay_to_end, validate_replay, ReplayError, ReplayIterator, ReplayStep};
pub use io::{
    estimate_sizes, from_compressed_bytes, from_json_string, load, load_compressed, load_json,
    save, save_compressed, save_json, to_compressed_bytes, to_json_compact, to_json_string,
    ReplayFormat, ReplayIoError,
};
