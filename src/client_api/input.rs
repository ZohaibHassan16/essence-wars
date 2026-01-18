//! Player input trait for unified human/bot handling.
//!
//! The PlayerInput trait provides a common interface for both human players
//! and AI bots, enabling the GameClient to handle them uniformly.

use crate::bots::Bot;
use crate::core::actions::Action;
use crate::core::engine::GameEngine;
use crate::tensor::{legal_mask_to_tensor, state_to_tensor};

/// A unified interface for human and bot players.
///
/// This trait allows GameClient to work with any source of player input,
/// whether it's a human making choices through a UI or an AI bot.
pub trait PlayerInput: Send {
    /// Get the name of this player/bot.
    fn name(&self) -> &str;

    /// Select an action given the current game state and legal actions.
    ///
    /// # Arguments
    /// * `engine` - The game engine (provides state and card database)
    /// * `legal_actions` - List of currently legal actions
    ///
    /// # Returns
    /// The selected action from the legal_actions list.
    fn select_action(&mut self, engine: &GameEngine, legal_actions: &[Action]) -> Action;

    /// Reset the player input state (called between games).
    fn reset(&mut self);

    /// Check if this is a human player (requires async input).
    fn is_human(&self) -> bool {
        false
    }
}

/// Wrapper to use any Bot as a PlayerInput.
pub struct BotInput {
    bot: Box<dyn Bot>,
}

impl BotInput {
    /// Create a new BotInput wrapping the given bot.
    pub fn new(bot: Box<dyn Bot>) -> Self {
        Self { bot }
    }

    /// Get a reference to the underlying bot.
    pub fn bot(&self) -> &dyn Bot {
        self.bot.as_ref()
    }

    /// Get a mutable reference to the underlying bot.
    pub fn bot_mut(&mut self) -> &mut dyn Bot {
        self.bot.as_mut()
    }
}

impl PlayerInput for BotInput {
    fn name(&self) -> &str {
        self.bot.name()
    }

    fn select_action(&mut self, engine: &GameEngine, legal_actions: &[Action]) -> Action {
        let state_tensor = state_to_tensor(&engine.state);
        let legal_mask = legal_mask_to_tensor(&crate::core::legal::legal_action_mask(
            &engine.state,
            engine.card_db(),
        ));

        self.bot.select_action(&state_tensor, &legal_mask, legal_actions)
    }

    fn reset(&mut self) {
        self.bot.reset();
    }
}

/// A placeholder for human player input.
///
/// In a real application, this would be connected to a UI.
/// For now, it provides a structure for future implementation.
pub struct HumanInput {
    name: String,
    pending_action: Option<Action>,
}

impl HumanInput {
    /// Create a new human input with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pending_action: None,
        }
    }

    /// Set the action that should be returned on the next select_action call.
    ///
    /// This is used by the UI to provide the human's choice.
    pub fn set_action(&mut self, action: Action) {
        self.pending_action = Some(action);
    }

    /// Check if an action is pending.
    pub fn has_pending_action(&self) -> bool {
        self.pending_action.is_some()
    }

    /// Clear any pending action.
    pub fn clear_action(&mut self) {
        self.pending_action = None;
    }
}

impl PlayerInput for HumanInput {
    fn name(&self) -> &str {
        &self.name
    }

    fn select_action(&mut self, _engine: &GameEngine, legal_actions: &[Action]) -> Action {
        // In a real implementation, this would wait for UI input.
        // For now, return the pending action or EndTurn as fallback.
        self.pending_action.take().unwrap_or_else(|| {
            // Fallback: find EndTurn in legal actions
            legal_actions
                .iter()
                .find(|a| matches!(a, Action::EndTurn))
                .copied()
                .unwrap_or(legal_actions[0])
        })
    }

    fn reset(&mut self) {
        self.pending_action = None;
    }

    fn is_human(&self) -> bool {
        true
    }
}

/// A simple action queue for scripted sequences (useful for testing/replays).
pub struct ScriptedInput {
    name: String,
    actions: Vec<Action>,
    index: usize,
}

impl ScriptedInput {
    /// Create a new scripted input with predefined actions.
    pub fn new(name: impl Into<String>, actions: Vec<Action>) -> Self {
        Self {
            name: name.into(),
            actions,
            index: 0,
        }
    }

    /// Get remaining actions count.
    pub fn remaining(&self) -> usize {
        self.actions.len().saturating_sub(self.index)
    }
}

impl PlayerInput for ScriptedInput {
    fn name(&self) -> &str {
        &self.name
    }

    fn select_action(&mut self, _engine: &GameEngine, legal_actions: &[Action]) -> Action {
        if self.index < self.actions.len() {
            let action = self.actions[self.index];
            self.index += 1;

            // Verify action is legal
            if legal_actions.contains(&action) {
                return action;
            }
        }

        // Fallback: EndTurn or first legal action
        legal_actions
            .iter()
            .find(|a| matches!(a, Action::EndTurn))
            .copied()
            .unwrap_or(legal_actions[0])
    }

    fn reset(&mut self) {
        self.index = 0;
    }
}
