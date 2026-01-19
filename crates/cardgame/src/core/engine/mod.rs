//! Module for the main game engine logic.
//!
//! The engine orchestrates game flow, processes actions, and manages
//! the game loop including turn structure and priority.
//!
//! This module also includes the Effect Queue System for processing game effects
//! without recursion, enabling clean triggered ability handling.

mod effect_queue;
mod effect_convert;
mod passive;
mod game_engine;
mod environment;

// Re-export public API
pub use effect_queue::EffectQueue;
pub use effect_convert::{
    resolve_spell_target,
    effect_def_to_effect_with_target,
    effect_def_to_triggered_effect,
};
pub use passive::support_effect_def_to_effect;
pub use game_engine::GameEngine;
pub use environment::GameEnvironment;

/// Seeded shuffle using Linear Congruential Generator for deterministic results.
/// Uses the same constants as PCG for good statistical properties.
pub fn seeded_shuffle<T>(items: &mut [T], seed: u64) {
    let mut rng = seed;
    for i in (1..items.len()).rev() {
        // LCG: next = (a * current + c) mod m
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (rng as usize) % (i + 1);
        items.swap(i, j);
    }
}
