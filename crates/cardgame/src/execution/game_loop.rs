//! Unified game loop abstraction for running games between bots.
//!
//! This module provides a single, reusable implementation of the game loop
//! pattern that is used throughout the codebase. It ensures:
//!
//! - **Safety**: Action limit is built-in and cannot be forgotten
//! - **Flexibility**: Callbacks allow customization without code duplication
//! - **Consistency**: All game loops behave identically
//!
//! # Usage
//!
//! ## Basic game loop (no callbacks)
//!
//! ```ignore
//! use cardgame::execution::game_loop::{run_game_loop, GameLoopConfig, NoOpCallback};
//!
//! let config = GameLoopConfig::new(seed);
//! let result = run_game_loop(
//!     &mut engine,
//!     &mut bot1,
//!     &mut bot2,
//!     &config,
//!     &mut NoOpCallback,
//! );
//! ```
//!
//! ## With per-action callback
//!
//! ```ignore
//! struct MyCallback { actions: Vec<Action> }
//!
//! impl GameLoopCallback for MyCallback {
//!     fn on_action(&mut self, ctx: &ActionContext) -> bool {
//!         self.actions.push(ctx.action);
//!         true // continue
//!     }
//! }
//!
//! let mut callback = MyCallback { actions: vec![] };
//! run_game_loop(&mut engine, &mut bot1, &mut bot2, &config, &mut callback);
//! ```
//!
//! ## With GameClient (for event capture)
//!
//! ```ignore
//! use cardgame::execution::game_loop::{run_game_loop_with_client, ClientGameLoopCallback};
//!
//! struct EventCallback { events: Vec<GameEvent> }
//!
//! impl ClientGameLoopCallback for EventCallback {
//!     fn on_action(&mut self, ctx: &ClientActionContext) -> bool {
//!         self.events.extend_from_slice(ctx.events);
//!         true
//!     }
//! }
//!
//! run_game_loop_with_client(&mut client, &mut bot1, &mut bot2, &config, &mut callback);
//! ```

use std::time::{Duration, Instant};

use crate::actions::Action;
use crate::bots::Bot;
use crate::client_api::{GameClient, GameEvent};
use crate::engine::GameEngine;
use crate::types::PlayerId;

// ============================================================================
// Constants
// ============================================================================

// Re-export safety limit from core config
pub use crate::core::config::game::MAX_ACTIONS_PER_GAME;

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for running a game loop.
#[derive(Clone, Debug)]
pub struct GameLoopConfig {
    /// Maximum actions before aborting (default: MAX_ACTIONS_PER_GAME).
    ///
    /// This is a safety limit to prevent infinite loops. It should not
    /// be disabled - use a high value if you need more actions.
    pub max_actions: usize,

    /// Seed used for this game (for logging on limit exceeded).
    pub seed: u64,
}

impl Default for GameLoopConfig {
    fn default() -> Self {
        Self {
            max_actions: MAX_ACTIONS_PER_GAME,
            seed: 0,
        }
    }
}

impl GameLoopConfig {
    /// Create a new config with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            ..Default::default()
        }
    }

    /// Override the max_actions limit.
    ///
    /// The value is clamped to 1..=10000 to prevent misuse.
    pub fn with_max_actions(mut self, max: usize) -> Self {
        self.max_actions = max.clamp(1, 10_000);
        self
    }
}

// ============================================================================
// Results
// ============================================================================

/// Result of running a game loop.
#[derive(Clone, Debug, PartialEq)]
pub enum GameLoopResult {
    /// Game completed normally (win, draw, or turn limit).
    Completed {
        /// Winner of the game (None if draw).
        winner: Option<PlayerId>,
        /// Number of turns played.
        turns: u32,
        /// Number of actions executed.
        actions: usize,
    },

    /// Game exceeded action limit without completing.
    ///
    /// This indicates a bug or degenerate game state.
    ActionLimitExceeded {
        /// Number of actions executed before limit was hit.
        actions: usize,
        /// Seed for debugging/reproduction.
        seed: u64,
    },
}

impl GameLoopResult {
    /// Returns true if the game completed normally.
    pub fn is_completed(&self) -> bool {
        matches!(self, GameLoopResult::Completed { .. })
    }

    /// Returns the winner, if any.
    pub fn winner(&self) -> Option<PlayerId> {
        match self {
            GameLoopResult::Completed { winner, .. } => *winner,
            GameLoopResult::ActionLimitExceeded { .. } => None,
        }
    }

    /// Returns the number of actions executed.
    pub fn actions(&self) -> usize {
        match self {
            GameLoopResult::Completed { actions, .. } => *actions,
            GameLoopResult::ActionLimitExceeded { actions, .. } => *actions,
        }
    }
}

// ============================================================================
// GameEngine Callback Types
// ============================================================================

/// Context provided to callbacks for each action (GameEngine variant).
pub struct ActionContext<'a, 'b> {
    /// Current turn number.
    pub turn: u16,
    /// Current player.
    pub player: PlayerId,
    /// Action that was selected (before application).
    pub action: Action,
    /// Index of this action (0-based).
    pub action_index: usize,
    /// Time spent selecting the action.
    pub thinking_time: Duration,
    /// Reference to the game engine (state before action application).
    pub engine: &'a GameEngine<'b>,
}

/// Callback trait for per-action processing with GameEngine.
///
/// Implement this trait to receive notifications during the game loop.
/// The default implementations are no-ops.
pub trait GameLoopCallback {
    /// Called after an action is selected but before it's applied.
    ///
    /// Return `false` to abort the game loop early.
    fn on_action(&mut self, _ctx: &ActionContext) -> bool {
        true
    }

    /// Called when the game loop completes.
    fn on_complete(&mut self, _result: &GameLoopResult) {}
}

/// No-op callback for when you don't need per-action processing.
///
/// This is zero-overhead - the compiler will inline and eliminate all calls.
pub struct NoOpCallback;

impl GameLoopCallback for NoOpCallback {}

// ============================================================================
// GameClient Callback Types
// ============================================================================

/// Context provided to callbacks for each action (GameClient variant).
///
/// This includes events captured from the action application.
pub struct ClientActionContext<'a> {
    /// Current turn number.
    pub turn: u16,
    /// Current player.
    pub player: PlayerId,
    /// Action that was applied.
    pub action: Action,
    /// Index of this action (0-based).
    pub action_index: usize,
    /// Time spent selecting the action.
    pub thinking_time: Duration,
    /// Events generated by this action.
    pub events: &'a [GameEvent],
    /// Reference to the game client (state after action application).
    pub client: &'a GameClient,
}

/// Callback trait for per-action processing with GameClient.
///
/// This variant provides access to game events for UI/replay purposes.
pub trait ClientGameLoopCallback {
    /// Called after an action is applied.
    ///
    /// Return `false` to abort the game loop early.
    fn on_action(&mut self, _ctx: &ClientActionContext) -> bool {
        true
    }

    /// Called when the game loop completes.
    fn on_complete(&mut self, _result: &GameLoopResult) {}
}

/// No-op callback for GameClient variant.
pub struct NoOpClientCallback;

impl ClientGameLoopCallback for NoOpClientCallback {}

// ============================================================================
// Core Game Loop (GameEngine)
// ============================================================================

/// Run a complete game between two bots using GameEngine.
///
/// This is the primary function for running bot-vs-bot games. The safety
/// limit is built-in and cannot be disabled.
///
/// # Arguments
///
/// * `engine` - Initialized game engine (game must be started)
/// * `bot1` - Bot for Player 1
/// * `bot2` - Bot for Player 2
/// * `config` - Configuration including max_actions safety limit
/// * `callback` - Callback for per-action processing
///
/// # Returns
///
/// `GameLoopResult` indicating how the game ended.
///
/// # Example
///
/// ```ignore
/// let config = GameLoopConfig::new(seed);
/// let result = run_game_loop(
///     &mut engine,
///     &mut bot1,
///     &mut bot2,
///     &config,
///     &mut NoOpCallback,
/// );
/// match result {
///     GameLoopResult::Completed { winner, turns, actions } => {
///         println!("Game completed in {} turns ({} actions)", turns, actions);
///     }
///     GameLoopResult::ActionLimitExceeded { actions, seed } => {
///         eprintln!("WARNING: Game hit action limit at {} actions (seed: {})", actions, seed);
///     }
/// }
/// ```
pub fn run_game_loop(
    engine: &mut GameEngine,
    bot1: &mut dyn Bot,
    bot2: &mut dyn Bot,
    config: &GameLoopConfig,
    callback: &mut impl GameLoopCallback,
) -> GameLoopResult {
    let mut action_count = 0;

    while !engine.is_game_over() && action_count < config.max_actions {
        let start = Instant::now();
        let current_player = engine.current_player();
        let turn = engine.turn_number();

        // Select action using appropriate bot
        let action = if current_player == PlayerId::PLAYER_ONE {
            bot1.select_action_with_engine(engine)
        } else {
            bot2.select_action_with_engine(engine)
        };

        let thinking_time = start.elapsed();

        // Create context for callback
        let ctx = ActionContext {
            turn,
            player: current_player,
            action,
            action_index: action_count,
            thinking_time,
            engine,
        };

        // Allow callback to process and potentially abort
        if !callback.on_action(&ctx) {
            break;
        }

        // Apply action
        if engine.apply_action(action).is_err() {
            break;
        }

        action_count += 1;
    }

    // Determine result
    let result = if action_count >= config.max_actions && !engine.is_game_over() {
        log::warn!(
            "Game exceeded {} action limit without completion (seed: {})",
            config.max_actions,
            config.seed
        );
        GameLoopResult::ActionLimitExceeded {
            actions: action_count,
            seed: config.seed,
        }
    } else {
        GameLoopResult::Completed {
            winner: engine.winner(),
            turns: engine.turn_number() as u32,
            actions: action_count,
        }
    };

    callback.on_complete(&result);
    result
}

// ============================================================================
// Core Game Loop (GameClient)
// ============================================================================

/// Run a complete game between two bots using GameClient.
///
/// This variant captures events for UI/replay purposes. Use this when you
/// need to track game events for animations or analysis.
///
/// # Arguments
///
/// * `client` - Initialized game client (game must be started)
/// * `bot1` - Bot for Player 1
/// * `bot2` - Bot for Player 2
/// * `config` - Configuration including max_actions safety limit
/// * `callback` - Callback for per-action processing (receives events)
///
/// # Returns
///
/// `GameLoopResult` indicating how the game ended.
pub fn run_game_loop_with_client(
    client: &mut GameClient,
    bot1: &mut dyn Bot,
    bot2: &mut dyn Bot,
    config: &GameLoopConfig,
    callback: &mut impl ClientGameLoopCallback,
) -> GameLoopResult {
    let mut action_count = 0;

    while !client.is_game_over() && action_count < config.max_actions {
        let start = Instant::now();

        // Get current player (returns None if game not active)
        let current_player = match client.current_player() {
            Some(p) => p,
            None => break,
        };

        let turn = client.turn_number();

        // Select action using appropriate bot
        let action = if current_player == PlayerId::PLAYER_ONE {
            match client.select_bot_action(bot1) {
                Some(a) => a,
                None => break,
            }
        } else {
            match client.select_bot_action(bot2) {
                Some(a) => a,
                None => break,
            }
        };

        let thinking_time = start.elapsed();

        // Apply action and capture events
        let events = match client.apply_action(action) {
            Ok(events) => events,
            Err(_) => break,
        };

        // Create context for callback
        let ctx = ClientActionContext {
            turn,
            player: current_player,
            action,
            action_index: action_count,
            thinking_time,
            events: &events,
            client,
        };

        // Allow callback to process and potentially abort
        if !callback.on_action(&ctx) {
            break;
        }

        action_count += 1;
    }

    // Determine result
    let result = if action_count >= config.max_actions && !client.is_game_over() {
        log::warn!(
            "Game exceeded {} action limit without completion (seed: {})",
            config.max_actions,
            config.seed
        );
        GameLoopResult::ActionLimitExceeded {
            actions: action_count,
            seed: config.seed,
        }
    } else {
        let winner = client.get_result().and_then(|r| match r {
            crate::core::state::GameResult::Win { winner, .. } => Some(winner),
            crate::core::state::GameResult::Draw => None,
        });
        GameLoopResult::Completed {
            winner,
            turns: client.turn_number() as u32,
            actions: action_count,
        }
    };

    callback.on_complete(&result);
    result
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Run a game loop and return just the winner.
///
/// This is a convenience wrapper for the common case where you only
/// need the game result.
pub fn run_game_to_completion(
    engine: &mut GameEngine,
    bot1: &mut dyn Bot,
    bot2: &mut dyn Bot,
    seed: u64,
) -> Option<PlayerId> {
    let config = GameLoopConfig::new(seed);
    run_game_loop(engine, bot1, bot2, &config, &mut NoOpCallback).winner()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = GameLoopConfig::default();
        assert_eq!(config.max_actions, MAX_ACTIONS_PER_GAME);
        assert_eq!(config.seed, 0);
    }

    #[test]
    fn test_config_with_seed() {
        let config = GameLoopConfig::new(12345);
        assert_eq!(config.seed, 12345);
        assert_eq!(config.max_actions, MAX_ACTIONS_PER_GAME);
    }

    #[test]
    fn test_config_max_actions_clamp() {
        // Too low - should clamp to 1
        let config = GameLoopConfig::default().with_max_actions(0);
        assert_eq!(config.max_actions, 1);

        // Too high - should clamp to 10000
        let config = GameLoopConfig::default().with_max_actions(100_000);
        assert_eq!(config.max_actions, 10_000);

        // Normal value
        let config = GameLoopConfig::default().with_max_actions(500);
        assert_eq!(config.max_actions, 500);
    }

    #[test]
    fn test_result_accessors() {
        let completed = GameLoopResult::Completed {
            winner: Some(PlayerId::PLAYER_ONE),
            turns: 15,
            actions: 42,
        };
        assert!(completed.is_completed());
        assert_eq!(completed.winner(), Some(PlayerId::PLAYER_ONE));
        assert_eq!(completed.actions(), 42);

        let exceeded = GameLoopResult::ActionLimitExceeded {
            actions: 1000,
            seed: 999,
        };
        assert!(!exceeded.is_completed());
        assert_eq!(exceeded.winner(), None);
        assert_eq!(exceeded.actions(), 1000);
    }
}
