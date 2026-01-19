//! Replay iteration for playback.
//!
//! This module provides ReplayIterator for stepping through replays
//! and reconstructing game states.

use crate::core::actions::Action;
use crate::core::cards::CardDatabase;
use crate::core::engine::GameEngine;
use crate::core::state::GameState;
use crate::replay::types::GameReplay;

/// Error type for replay operations.
#[derive(Debug, Clone)]
pub enum ReplayError {
    /// Tried to step past the end of the replay.
    EndOfReplay,
    /// Failed to apply an action from the replay.
    ActionFailed(String),
    /// Seek position is out of bounds.
    SeekOutOfBounds(usize),
    /// Replay is incomplete or corrupted.
    InvalidReplay(String),
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplayError::EndOfReplay => write!(f, "End of replay reached"),
            ReplayError::ActionFailed(msg) => write!(f, "Action failed: {}", msg),
            ReplayError::SeekOutOfBounds(pos) => write!(f, "Seek position {} out of bounds", pos),
            ReplayError::InvalidReplay(msg) => write!(f, "Invalid replay: {}", msg),
        }
    }
}

impl std::error::Error for ReplayError {}

/// A single step in replay playback.
#[derive(Debug, Clone)]
pub struct ReplayStep {
    /// Index of the action in the replay.
    pub action_index: usize,
    /// The action that was applied.
    pub action: Action,
    /// Turn number when the action was taken.
    pub turn: u16,
    /// Whether this is the last action in the replay.
    pub is_final: bool,
}

/// Iterator for stepping through a replay.
///
/// ReplayIterator allows stepping through a replay action by action,
/// reconstructing the game state at each step.
pub struct ReplayIterator<'a> {
    replay: &'a GameReplay,
    card_db: &'a CardDatabase,
    engine: Option<GameEngine<'a>>,
    current_index: usize,
    initialized: bool,
}

impl<'a> ReplayIterator<'a> {
    /// Create a new replay iterator.
    pub fn new(replay: &'a GameReplay, card_db: &'a CardDatabase) -> Self {
        Self {
            replay,
            card_db,
            engine: None,
            current_index: 0,
            initialized: false,
        }
    }

    /// Initialize the game state from the replay.
    fn initialize(&mut self) -> Result<(), ReplayError> {
        if self.initialized {
            return Ok(());
        }

        let mut engine = GameEngine::new(self.card_db);
        engine.start_game_with_mode(
            self.replay.player1.deck.clone(),
            self.replay.player2.deck.clone(),
            self.replay.header.seed,
            self.replay.header.mode,
        );

        self.engine = Some(engine);
        self.initialized = true;
        Ok(())
    }

    /// Step to the next action in the replay.
    pub fn step(&mut self) -> Result<Option<ReplayStep>, ReplayError> {
        self.initialize()?;

        if self.current_index >= self.replay.actions.len() {
            return Ok(None);
        }

        let replay_action = &self.replay.actions[self.current_index];
        let action = replay_action.action;
        let turn = replay_action.turn;

        // Apply the action to the engine
        let engine = self.engine.as_mut().unwrap();
        engine.apply_action(action)
            .map_err(ReplayError::ActionFailed)?;

        let is_final = self.current_index == self.replay.actions.len() - 1;
        let step = ReplayStep {
            action_index: self.current_index,
            action,
            turn,
            is_final,
        };

        self.current_index += 1;
        Ok(Some(step))
    }

    /// Step forward by N actions.
    pub fn step_n(&mut self, n: usize) -> Result<Vec<ReplayStep>, ReplayError> {
        let mut steps = Vec::new();
        for _ in 0..n {
            match self.step()? {
                Some(step) => steps.push(step),
                None => break,
            }
        }
        Ok(steps)
    }

    /// Seek to a specific action index.
    ///
    /// This will replay from the beginning (or nearest snapshot) to reach the target.
    pub fn seek(&mut self, target_index: usize) -> Result<(), ReplayError> {
        if target_index >= self.replay.actions.len() {
            return Err(ReplayError::SeekOutOfBounds(target_index));
        }

        // If we need to go backwards, or haven't started, reset
        if target_index < self.current_index || !self.initialized {
            self.reset();
            self.initialize()?;
        }

        // Step forward to target
        while self.current_index < target_index {
            self.step()?;
        }

        Ok(())
    }

    /// Reset to the beginning of the replay.
    pub fn reset(&mut self) {
        self.engine = None;
        self.current_index = 0;
        self.initialized = false;
    }

    /// Get the current game state.
    pub fn current_state(&self) -> Option<&GameState> {
        self.engine.as_ref().map(|e| &e.state)
    }

    /// Get the current action index.
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Get the total number of actions.
    pub fn total_actions(&self) -> usize {
        self.replay.actions.len()
    }

    /// Check if replay is at the end.
    pub fn is_at_end(&self) -> bool {
        self.current_index >= self.replay.actions.len()
    }

    /// Get the replay being iterated.
    pub fn replay(&self) -> &GameReplay {
        self.replay
    }
}

/// Play through an entire replay and return the final state.
pub fn replay_to_end(
    replay: &GameReplay,
    card_db: &CardDatabase,
) -> Result<GameState, ReplayError> {
    let mut engine = GameEngine::new(card_db);
    engine.start_game_with_mode(
        replay.player1.deck.clone(),
        replay.player2.deck.clone(),
        replay.header.seed,
        replay.header.mode,
    );

    for replay_action in &replay.actions {
        engine.apply_action(replay_action.action)
            .map_err(ReplayError::ActionFailed)?;
    }

    Ok(engine.state)
}

/// Validate that a replay is consistent (can be replayed without errors).
pub fn validate_replay(replay: &GameReplay, card_db: &CardDatabase) -> Result<(), ReplayError> {
    let _final_state = replay_to_end(replay, card_db)?;
    Ok(())
}
