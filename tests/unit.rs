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
