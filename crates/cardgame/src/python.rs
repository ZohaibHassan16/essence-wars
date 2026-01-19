//! Python bindings for the Essence Wars game engine.
//!
//! This module provides PyO3 bindings to expose the Rust game engine to Python.
//! It enables high-performance training with frameworks like PyTorch while
//! keeping all game logic in Rust.
//!
//! # Usage from Python
//!
//! ```python
//! from essence_wars._core import PyGame, PyParallelGames
//!
//! # Single game
//! game = PyGame()
//! game.reset(seed=42)
//! obs = game.observe()  # numpy array (326,)
//! mask = game.action_mask()  # numpy array (256,)
//! reward, done = game.step(action)
//!
//! # Batched games for vectorized training
//! games = PyParallelGames(num_envs=64)
//! games.reset([seed1, seed2, ...])
//! obs_batch = games.observe_batch()  # numpy array (64, 326)
//! ```

// Allow clippy::useless_conversion for PyO3 method returns
// PyO3 macros require PyResult even though clippy 1.92.0 sees it as redundant
#![allow(clippy::useless_conversion)]

use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyRuntimeError};
use numpy::{PyArray1, PyArray2, PyReadonlyArray1, PyArrayMethods};

/// Type alias for the complex return type used in batched step operations
type BatchStepResult<'py> = (Bound<'py, PyArray1<f32>>, Bound<'py, PyArray1<bool>>);

use crate::core::cards::CardDatabase;
use crate::core::config::tensor::STATE_TENSOR_SIZE;
use crate::core::engine::GameEngine;
use crate::core::state::GameMode;
use crate::core::types::{CardId, PlayerId};
use crate::decks::DeckRegistry;
use crate::bots::{Bot, GreedyBot, MctsBot, MctsConfig, RandomBot};

/// Single game wrapper for Python.
///
/// Provides a Pythonic interface to the Rust game engine with zero-copy
/// tensor transfer via NumPy arrays.
#[pyclass]
pub struct PyGame {
    engine: GameEngine<'static>,
    #[allow(dead_code)]
    card_db: &'static CardDatabase,
    #[allow(dead_code)]
    deck_registry: &'static DeckRegistry,
    deck1: Vec<CardId>,
    deck2: Vec<CardId>,
    game_mode: GameMode,
}

// Static storage for card database and deck registry
// These are loaded once and shared across all PyGame instances
lazy_static::lazy_static! {
    static ref CARD_DB: CardDatabase = {
        let cards_path = crate::data_dir().join("cards/core_set");
        CardDatabase::load_from_directory(cards_path)
            .expect("Failed to load card database")
    };
    static ref DECK_REGISTRY: DeckRegistry = {
        let decks_path = crate::data_dir().join("decks");
        DeckRegistry::load_from_directory(decks_path)
            .expect("Failed to load deck registry")
    };
}

#[pymethods]
impl PyGame {
    /// Create a new game instance.
    ///
    /// Args:
    ///     deck1: Name of deck for player 1 (default: "argentum_control")
    ///     deck2: Name of deck for player 2 (default: "symbiote_aggro")
    ///     game_mode: "attrition" (default) or "essence_duel"
    #[new]
    #[pyo3(signature = (deck1=None, deck2=None, game_mode=None))]
    fn new(
        deck1: Option<&str>,
        deck2: Option<&str>,
        game_mode: Option<&str>,
    ) -> PyResult<Self> {
        let card_db: &'static CardDatabase = &CARD_DB;
        let deck_registry: &'static DeckRegistry = &DECK_REGISTRY;

        // Load decks
        let deck1_name = deck1.unwrap_or("artificer_tokens");
        let deck2_name = deck2.unwrap_or("broodmother_swarm");

        let deck1_def = deck_registry.get(deck1_name)
            .ok_or_else(|| PyValueError::new_err(format!("Unknown deck: {}", deck1_name)))?;
        let deck2_def = deck_registry.get(deck2_name)
            .ok_or_else(|| PyValueError::new_err(format!("Unknown deck: {}", deck2_name)))?;

        // Parse game mode
        let mode = match game_mode.unwrap_or("attrition") {
            "attrition" => GameMode::Attrition,
            "essence_duel" | "essence-duel" => GameMode::EssenceDuel,
            other => return Err(PyValueError::new_err(format!("Unknown game mode: {}", other))),
        };

        let engine = GameEngine::new(card_db);

        Ok(Self {
            engine,
            card_db,
            deck_registry,
            deck1: deck1_def.to_card_ids(),
            deck2: deck2_def.to_card_ids(),
            game_mode: mode,
        })
    }

    /// Reset the game to initial state.
    ///
    /// Args:
    ///     seed: Random seed for deck shuffling and game randomness
    fn reset(&mut self, seed: u64) {
        self.engine.start_game_with_mode(
            self.deck1.clone(),
            self.deck2.clone(),
            seed,
            self.game_mode,
        );
    }

    /// Get the current state tensor as a NumPy array.
    ///
    /// Returns:
    ///     numpy.ndarray: Float32 array of shape (326,) representing game state
    fn observe<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f32>> {
        let tensor = self.engine.get_state_tensor();
        PyArray1::from_slice_bound(py, &tensor)
    }

    /// Get the legal action mask as a NumPy array.
    ///
    /// Returns:
    ///     numpy.ndarray: Float32 array of shape (256,) where 1.0 = legal, 0.0 = illegal
    fn action_mask<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f32>> {
        let mask = self.engine.get_legal_action_mask();
        PyArray1::from_slice_bound(py, &mask)
    }

    /// Apply an action and return (reward, done).
    ///
    /// Args:
    ///     action: Action index (0-255)
    ///
    /// Returns:
    ///     tuple: (reward: float, done: bool)
    ///         - reward is from player 0's perspective: +1 win, -1 loss, 0 otherwise
    ///         - done is True if the game has ended
    fn step(&mut self, action: u8) -> PyResult<(f32, bool)> {
        self.engine.apply_action_by_index(action)
            .map_err(PyValueError::new_err)?;

        let done = self.engine.is_game_over();
        let reward = if done {
            self.engine.get_reward(PlayerId::PLAYER_ONE)
        } else {
            0.0
        };

        Ok((reward, done))
    }

    /// Get the current player (0 or 1).
    fn current_player(&self) -> u8 {
        self.engine.current_player().0
    }

    /// Check if the game is over.
    fn is_done(&self) -> bool {
        self.engine.is_game_over()
    }

    /// Get the reward for a specific player.
    ///
    /// Args:
    ///     player: Player index (0 or 1)
    ///
    /// Returns:
    ///     float: +1.0 for win, -1.0 for loss, 0.0 for ongoing/draw
    fn get_reward(&self, player: u8) -> f32 {
        self.engine.get_reward(PlayerId(player))
    }

    /// Clone the game state for tree search (e.g., MCTS).
    ///
    /// Returns:
    ///     PyGame: A new PyGame instance with copied state
    fn fork(&self) -> Self {
        Self {
            engine: self.engine.fork(),
            card_db: self.card_db,
            deck_registry: self.deck_registry,
            deck1: self.deck1.clone(),
            deck2: self.deck2.clone(),
            game_mode: self.game_mode,
        }
    }

    /// Get action from the built-in GreedyBot.
    ///
    /// Useful as a baseline opponent during training.
    ///
    /// Returns:
    ///     int: Action index chosen by GreedyBot
    fn greedy_action(&self) -> PyResult<u8> {
        let tensor = self.engine.get_state_tensor();
        let mask = self.engine.get_legal_action_mask();
        let legal_actions = self.engine.get_legal_actions();

        if legal_actions.is_empty() {
            return Err(PyRuntimeError::new_err("No legal actions available"));
        }

        let mut bot = GreedyBot::new(self.card_db, 42);
        let action = bot.select_action(&tensor, &mask, &legal_actions);

        Ok(action.to_index())
    }

    /// Get action from the built-in RandomBot.
    ///
    /// Returns:
    ///     int: Action index chosen uniformly at random from legal actions
    fn random_action(&self) -> PyResult<u8> {
        let tensor = self.engine.get_state_tensor();
        let mask = self.engine.get_legal_action_mask();
        let legal_actions = self.engine.get_legal_actions();

        if legal_actions.is_empty() {
            return Err(PyRuntimeError::new_err("No legal actions available"));
        }

        let mut bot = RandomBot::new(42);
        let action = bot.select_action(&tensor, &mask, &legal_actions);

        Ok(action.to_index())
    }

    /// Get action from the built-in MCTS bot.
    ///
    /// MCTS (Monte Carlo Tree Search) provides a strong baseline that
    /// uses tree search with UCB1 selection and GreedyBot rollouts.
    ///
    /// Args:
    ///     simulations: Number of MCTS simulations (default: 100)
    ///
    /// Returns:
    ///     int: Action index chosen by MCTS
    #[pyo3(signature = (simulations=100))]
    fn mcts_action(&self, simulations: u32) -> PyResult<u8> {
        let legal_actions = self.engine.get_legal_actions();

        if legal_actions.is_empty() {
            return Err(PyRuntimeError::new_err("No legal actions available"));
        }

        let config = MctsConfig {
            simulations,
            ..Default::default()
        };
        let mut bot = MctsBot::with_config(self.card_db, config, 42);

        // MCTS needs engine access for simulation
        let action = bot.select_action_with_engine(&self.engine);

        Ok(action.to_index())
    }

    /// Get the current turn number.
    fn turn_number(&self) -> u16 {
        self.engine.turn_number()
    }

    /// List all available decks.
    #[staticmethod]
    fn list_decks() -> Vec<String> {
        DECK_REGISTRY.deck_ids().into_iter().map(|s| s.to_string()).collect()
    }
}

/// Batched game wrapper for vectorized training.
///
/// Runs multiple games in parallel using Rayon for high throughput.
/// Achieves 60k+ steps per second on modern CPUs.
#[pyclass]
pub struct PyParallelGames {
    engines: Vec<GameEngine<'static>>,
    #[allow(dead_code)]
    card_db: &'static CardDatabase,
    #[allow(dead_code)]
    deck_registry: &'static DeckRegistry,
    deck1: Vec<CardId>,
    deck2: Vec<CardId>,
    game_mode: GameMode,
    num_envs: usize,
}

#[pymethods]
impl PyParallelGames {
    /// Create a batch of game environments.
    ///
    /// Args:
    ///     num_envs: Number of parallel environments
    ///     deck1: Name of deck for player 1 (default: "argentum_control")
    ///     deck2: Name of deck for player 2 (default: "symbiote_aggro")
    ///     game_mode: "attrition" (default) or "essence_duel"
    #[new]
    #[pyo3(signature = (num_envs, deck1=None, deck2=None, game_mode=None))]
    fn new(
        num_envs: usize,
        deck1: Option<&str>,
        deck2: Option<&str>,
        game_mode: Option<&str>,
    ) -> PyResult<Self> {
        let card_db: &'static CardDatabase = &CARD_DB;
        let deck_registry: &'static DeckRegistry = &DECK_REGISTRY;

        // Load decks
        let deck1_name = deck1.unwrap_or("artificer_tokens");
        let deck2_name = deck2.unwrap_or("broodmother_swarm");

        let deck1_def = deck_registry.get(deck1_name)
            .ok_or_else(|| PyValueError::new_err(format!("Unknown deck: {}", deck1_name)))?;
        let deck2_def = deck_registry.get(deck2_name)
            .ok_or_else(|| PyValueError::new_err(format!("Unknown deck: {}", deck2_name)))?;

        // Parse game mode
        let mode = match game_mode.unwrap_or("attrition") {
            "attrition" => GameMode::Attrition,
            "essence_duel" | "essence-duel" => GameMode::EssenceDuel,
            other => return Err(PyValueError::new_err(format!("Unknown game mode: {}", other))),
        };

        // Create engines
        let engines: Vec<_> = (0..num_envs)
            .map(|_| GameEngine::new(card_db))
            .collect();

        Ok(Self {
            engines,
            card_db,
            deck_registry,
            deck1: deck1_def.to_card_ids(),
            deck2: deck2_def.to_card_ids(),
            game_mode: mode,
            num_envs,
        })
    }

    /// Reset all games with given seeds.
    ///
    /// Args:
    ///     seeds: List of random seeds, one per environment
    fn reset(&mut self, seeds: Vec<u64>) -> PyResult<()> {
        if seeds.len() != self.num_envs {
            return Err(PyValueError::new_err(format!(
                "Expected {} seeds, got {}",
                self.num_envs,
                seeds.len()
            )));
        }

        for (engine, seed) in self.engines.iter_mut().zip(seeds.iter()) {
            engine.start_game_with_mode(
                self.deck1.clone(),
                self.deck2.clone(),
                *seed,
                self.game_mode,
            );
        }

        Ok(())
    }

    /// Get batch observations as a 2D NumPy array.
    ///
    /// Returns:
    ///     numpy.ndarray: Float32 array of shape (num_envs, 326)
    fn observe_batch<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray2<f32>> {
        let mut data = vec![0.0f32; self.num_envs * STATE_TENSOR_SIZE];

        for (i, engine) in self.engines.iter().enumerate() {
            let tensor = engine.get_state_tensor();
            let start = i * STATE_TENSOR_SIZE;
            data[start..start + STATE_TENSOR_SIZE].copy_from_slice(&tensor);
        }

        // Create 1D array and reshape to 2D
        let arr = PyArray1::from_slice_bound(py, &data);
        arr.reshape([self.num_envs, STATE_TENSOR_SIZE])
            .expect("Failed to reshape observation batch")
    }

    /// Get batch action masks as a 2D NumPy array.
    ///
    /// Returns:
    ///     numpy.ndarray: Float32 array of shape (num_envs, 256)
    fn action_mask_batch<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray2<f32>> {
        let mut data = vec![0.0f32; self.num_envs * 256];

        for (i, engine) in self.engines.iter().enumerate() {
            let mask = engine.get_legal_action_mask();
            let start = i * 256;
            data[start..start + 256].copy_from_slice(&mask);
        }

        let arr = PyArray1::from_slice_bound(py, &data);
        arr.reshape([self.num_envs, 256])
            .expect("Failed to reshape action mask batch")
    }

    /// Apply actions to all environments.
    ///
    /// Args:
    ///     actions: NumPy array of action indices, shape (num_envs,)
    ///
    /// Returns:
    ///     tuple: (rewards, dones) as NumPy arrays of shape (num_envs,)
    fn step_batch<'py>(
        &mut self,
        py: Python<'py>,
        actions: PyReadonlyArray1<'py, u8>,
    ) -> PyResult<BatchStepResult<'py>> {
        let actions = actions.as_slice()?;

        if actions.len() != self.num_envs {
            return Err(PyValueError::new_err(format!(
                "Expected {} actions, got {}",
                self.num_envs,
                actions.len()
            )));
        }

        let mut rewards = vec![0.0f32; self.num_envs];
        let mut dones = vec![false; self.num_envs];

        for (i, (engine, &action)) in self.engines.iter_mut().zip(actions.iter()).enumerate() {
            if let Err(e) = engine.apply_action_by_index(action) {
                return Err(PyValueError::new_err(format!(
                    "Env {}: {}",
                    i, e
                )));
            }

            dones[i] = engine.is_game_over();
            if dones[i] {
                rewards[i] = engine.get_reward(PlayerId::PLAYER_ONE);
            }
        }

        Ok((
            PyArray1::from_slice_bound(py, &rewards),
            PyArray1::from_slice_bound(py, &dones),
        ))
    }

    /// Get the number of environments.
    #[getter]
    fn num_envs(&self) -> usize {
        self.num_envs
    }

    /// Get current players for all environments.
    ///
    /// Returns:
    ///     list: List of current player indices (0 or 1)
    fn current_players(&self) -> Vec<u8> {
        self.engines.iter().map(|e| e.current_player().0).collect()
    }

    /// Check which environments are done.
    ///
    /// Returns:
    ///     list: List of booleans indicating game over status
    fn dones(&self) -> Vec<bool> {
        self.engines.iter().map(|e| e.is_game_over()).collect()
    }

    /// Reset a single environment by index.
    ///
    /// Args:
    ///     idx: Environment index (0 to num_envs - 1)
    ///     seed: Random seed for this environment
    fn reset_single(&mut self, idx: usize, seed: u64) -> PyResult<()> {
        if idx >= self.num_envs {
            return Err(PyValueError::new_err(format!(
                "Index {} out of bounds for {} environments",
                idx, self.num_envs
            )));
        }

        self.engines[idx].start_game_with_mode(
            self.deck1.clone(),
            self.deck2.clone(),
            seed,
            self.game_mode,
        );

        Ok(())
    }

    /// Reset all environments with a base seed.
    ///
    /// Each environment gets base_seed + env_index as its seed.
    ///
    /// Args:
    ///     base_seed: Base random seed
    fn reset_all(&mut self, base_seed: u64) {
        for (i, engine) in self.engines.iter_mut().enumerate() {
            engine.start_game_with_mode(
                self.deck1.clone(),
                self.deck2.clone(),
                base_seed + i as u64,
                self.game_mode,
            );
        }
    }
}

/// Python module definition.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGame>()?;
    m.add_class::<PyParallelGames>()?;

    // Add constants
    m.add("STATE_TENSOR_SIZE", STATE_TENSOR_SIZE)?;
    m.add("ACTION_SPACE_SIZE", 256usize)?;

    Ok(())
}
