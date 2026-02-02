//! Module for the main game engine logic.
//!
//! The engine orchestrates game flow, processes actions, and manages
//! the game loop including turn structure and priority.
//!
//! This module also includes the Effect Queue System for processing game effects
//! without recursion, enabling clean triggered ability handling.

mod effect_queue;
mod effect_convert;
mod effect_context;
mod handlers;
mod triggers;
mod passive;
mod game_engine;
mod environment;
mod actions;
mod init;
mod victory;
mod turn;

// Re-export public API
pub use effect_queue::EffectQueue;
pub use effect_convert::{
    resolve_spell_target,
    effect_def_to_effect_with_target,
    effect_def_to_triggered_effect,
};
pub use effect_context::{EffectContext, TargetResolver, ResolvedTargets};
pub use handlers::{EffectHandler, create_handler};
pub use triggers::{check_creature_triggers, effect_def_to_effect, process_deaths};
pub use passive::{
    support_effect_def_to_effect,
    collect_commander_ally_death_effects,
    collect_commander_enemy_death_effects,
    collect_commander_any_death_effects,
    collect_commander_kill_effects,
    resolve_commander_passive,
    apply_commander_passive_from_cache,
};
pub use game_engine::GameEngine;
pub use environment::GameEnvironment;
pub use actions::ActionContext;
pub use init::{initialize_game, initialize_game_raw, draw_card, GameInitError};
pub use victory::{check_victory_conditions, check_turn_limit_victory, check_life_victory, check_victory_points_victory};
pub use turn::{start_turn, end_turn};

/// Seeded shuffle using Linear Congruential Generator for deterministic results.
///
/// Uses PCG (Permuted Congruential Generator) constants for excellent statistical properties:
/// - Multiplier: 6364136223846793005 (from PCG paper by Melissa O'Neill)
/// - Increment: 1442695040888963407 (from PCG paper)
///
/// These constants ensure full-period generation and good distribution across the u64 range.
/// The Fisher-Yates shuffle algorithm ensures uniform permutation probability.
pub fn seeded_shuffle<T>(items: &mut [T], seed: u64) {
    let mut rng = seed;
    for i in (1..items.len()).rev() {
        // LCG step: next = (multiplier * current + increment) mod 2^64
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (rng as usize) % (i + 1);
        items.swap(i, j);
    }
}
