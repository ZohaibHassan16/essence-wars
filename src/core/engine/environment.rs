//! GameEnvironment trait for generic AI implementations.
//!
//! This trait provides a generic interface that AI algorithms (like MCTS or
//! reinforcement learning) can use to interact with any game, not just this
//! card game. This enables code reuse across different game implementations.

use crate::core::actions::Action;
use crate::core::legal::legal_actions;
use crate::core::state::GameState;
use crate::core::types::PlayerId;

use super::game_engine::GameEngine;

/// Trait for game environments (useful for generic AI implementations).
///
/// This trait provides a generic interface that AI algorithms (like MCTS or
/// reinforcement learning) can use to interact with any game, not just this
/// card game. This enables code reuse across different game implementations.
pub trait GameEnvironment {
    /// The state type for this game
    type State;
    /// The action type for this game
    type Action;

    /// Get a reference to the current state
    fn get_state(&self) -> &Self::State;

    /// Get all legal actions for the current state
    fn get_legal_actions(&self) -> Vec<Self::Action>;

    /// Apply an action to the current state
    fn apply_action(&mut self, action: Self::Action) -> Result<(), String>;

    /// Check if the game is in a terminal state
    fn is_terminal(&self) -> bool;

    /// Get the reward for a specific player
    fn get_reward(&self, player: u8) -> f32;

    /// Clone the environment for search (MCTS tree expansion)
    fn clone_for_search(&self) -> Self where Self: Sized;
}

impl<'a> GameEnvironment for GameEngine<'a> {
    type State = GameState;
    type Action = Action;

    fn get_state(&self) -> &Self::State {
        &self.state
    }

    fn get_legal_actions(&self) -> Vec<Self::Action> {
        legal_actions(&self.state, self.card_db()).to_vec()
    }

    fn apply_action(&mut self, action: Self::Action) -> Result<(), String> {
        GameEngine::apply_action(self, action)
    }

    fn is_terminal(&self) -> bool {
        self.is_game_over()
    }

    fn get_reward(&self, player: u8) -> f32 {
        GameEngine::get_reward(self, PlayerId(player))
    }

    fn clone_for_search(&self) -> Self {
        self.fork()
    }
}
