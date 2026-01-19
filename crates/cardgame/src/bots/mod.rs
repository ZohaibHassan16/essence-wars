//! Bot implementations for AI players.
//!
//! This module provides the `Bot` trait and various bot implementations
//! that can play the game. The engine core has no knowledge of bots -
//! bots interact only through the public GameEnvironment interface.

mod random;
mod greedy;
mod mcts;
pub mod weights;
pub mod factory;
pub mod introspection;

pub use random::RandomBot;
pub use greedy::GreedyBot;
pub use mcts::{MctsBot, MctsConfig, MctsNode};
pub use weights::{BotWeights, GreedyWeights, WeightSet};
pub use factory::{BotType, BotTypeParseError, create_bot, resolve_weights, resolve_weights_verbose, WeightResolutionError};
pub use introspection::{
    BotDecision, IntrospectionConfig, MctsNodeStats, MctsTreeSnapshot, PolicyOutput, PolicySource,
};

use crate::actions::Action;
use crate::engine::GameEngine;
use crate::tensor::STATE_TENSOR_SIZE;

/// Trait for bot implementations.
///
/// Bots receive the same information a neural network would:
/// - State tensor (326 floats)
/// - Legal action mask (256 floats)
/// - List of legal actions (for convenience)
///
/// The `Send` bound enables parallel game execution.
pub trait Bot: Send {
    /// Returns the bot's name for display purposes.
    fn name(&self) -> &str;

    /// Select an action given the current game state.
    ///
    /// # Arguments
    /// * `state_tensor` - 326-float representation of game state
    /// * `legal_mask` - 256-float mask (1.0 = legal, 0.0 = illegal)
    /// * `legal_actions` - List of legal actions (convenience, derived from mask)
    ///
    /// # Returns
    /// The action to take. Must be one of the legal actions.
    fn select_action(
        &mut self,
        state_tensor: &[f32; STATE_TENSOR_SIZE],
        legal_mask: &[f32; 256],
        legal_actions: &[Action],
    ) -> Action;

    /// Select an action with access to the game engine.
    ///
    /// This method allows bots that require engine access (like MCTS) to
    /// perform full search. The default implementation falls back to
    /// `select_action()` using state information from the engine.
    ///
    /// Bots that need engine access should override this method.
    fn select_action_with_engine(&mut self, engine: &GameEngine) -> Action {
        let state_tensor = engine.get_state_tensor();
        let legal_mask = engine.get_legal_action_mask();
        let legal_actions = engine.get_legal_actions();
        self.select_action(&state_tensor, &legal_mask, &legal_actions)
    }

    /// Returns true if this bot requires engine access for full functionality.
    ///
    /// Bots like MCTS that need to simulate future states should return true.
    /// This allows callers to use `select_action_with_engine()` when appropriate.
    fn requires_engine(&self) -> bool {
        false
    }

    /// Reset internal state between games.
    ///
    /// Called before each new game starts. Bots should clear any
    /// game-specific state (but may retain learned parameters).
    fn reset(&mut self);

    /// Clone the bot into a boxed trait object.
    ///
    /// Required for parallel game execution where each thread needs its own bot.
    fn clone_box(&self) -> Box<dyn Bot>;
}

impl Clone for Box<dyn Bot> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Extended bot trait with introspection capabilities.
///
/// Bots implementing this trait can expose their decision-making process
/// for visualization in Glassbox mode.
pub trait AnalyzableBot: Bot {
    /// Select an action and return introspection data.
    ///
    /// This method allows examining the bot's decision-making process
    /// including policy outputs, value estimates, and search tree snapshots.
    fn select_action_with_introspection(
        &mut self,
        engine: &GameEngine,
        config: &IntrospectionConfig,
    ) -> (Action, Option<BotDecision>);

    /// Get the last decision made (without re-computing).
    fn last_decision(&self) -> Option<&BotDecision>;

    /// Get the policy source type.
    fn policy_source(&self) -> PolicySource;
}
