//! Tracing infrastructure for debugging combat and effect resolution.
//!
//! This module provides:
//! - [`CombatTracer`]: Step-by-step combat keyword resolution tracing
//! - [`EffectTracer`]: Effect queue processing tracing
//!
//! These tracers are optional and can be enabled via CLI flags for debugging.

mod combat_tracer;
mod effect_tracer;

// Re-export all public items
pub use combat_tracer::{CombatPhase, CombatStep, CombatTrace, CombatTracer};
pub use effect_tracer::{EffectEvent, EffectEventType, EffectTracer};
