//! Unit tests for core game engine modules.
//!
//! Tests are organized in subdirectory matching src/core/ structure.

#[path = "unit/types_tests.rs"]
mod types_tests;
#[path = "unit/keywords_tests.rs"]
mod keywords_tests;
#[path = "unit/config_tests.rs"]
mod config_tests;
#[path = "unit/effects_tests.rs"]
mod effects_tests;
#[path = "unit/state_tests.rs"]
mod state_tests;
#[path = "unit/cards_tests.rs"]
mod cards_tests;
#[path = "unit/actions_tests.rs"]
mod actions_tests;
#[path = "unit/legal_tests.rs"]
mod legal_tests;
#[path = "unit/combat_tests.rs"]
mod combat_tests;
#[path = "unit/keyword_matrix_tests.rs"]
mod keyword_matrix_tests;
#[path = "unit/keyword_exhaustive_tests.rs"]
mod keyword_exhaustive_tests;
#[path = "unit/tensor_tests.rs"]
mod tensor_tests;
#[path = "unit/decks_tests.rs"]
mod decks_tests;
#[path = "unit/cmaes_tests.rs"]
mod cmaes_tests;
#[path = "unit/evaluator_tests.rs"]
mod evaluator_tests;
#[path = "unit/greedy_tests.rs"]
mod greedy_tests;
#[path = "unit/mcts_tests.rs"]
mod mcts_tests;
#[path = "unit/random_tests.rs"]
mod random_tests;
#[path = "unit/weights_tests.rs"]
mod weights_tests;
#[path = "unit/logger_tests.rs"]
mod logger_tests;
#[path = "unit/runner_tests.rs"]
mod runner_tests;
#[path = "unit/stats_tests.rs"]
mod stats_tests;
#[path = "unit/resource_system_tests.rs"]
mod resource_system_tests;
#[path = "unit/game_mode_tests.rs"]
mod game_mode_tests;
#[path = "unit/serialization_tests.rs"]
mod serialization_tests;
#[path = "unit/client_api_tests.rs"]
mod client_api_tests;
