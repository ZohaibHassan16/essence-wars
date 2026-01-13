//! Bot implementations for AI players.
//!
//! This module provides the `Bot` trait and various bot implementations
//! that can play the game. The engine core has no knowledge of bots -
//! bots interact only through the public GameEnvironment interface.

mod random;
mod greedy;
mod mcts;
pub mod weights;

pub use random::RandomBot;
pub use greedy::GreedyBot;
pub use mcts::{MctsBot, MctsConfig, MctsNode};
pub use weights::{BotWeights, GreedyWeights, WeightSet};

use crate::actions::Action;

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
