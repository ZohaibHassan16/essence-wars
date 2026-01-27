//! Action execution module.
//!
//! This module provides the routing and context for action execution,
//! delegating to specialized handlers for each action type.

mod play_card;
mod attack;
mod ability;

pub use play_card::execute_play_card;
pub use attack::{execute_attack, execute_attack_with_tracers};
pub use ability::execute_use_ability;

use crate::core::cards::CardDatabase;
use crate::core::state::GameState;
use crate::core::tracing::EffectTracer;

use super::effect_queue::EffectQueue;

/// Context for action execution.
///
/// Similar to `EffectContext`, this bundles all the state needed by action
/// handlers, providing a clean interface and centralizing common operations.
pub struct ActionContext<'a> {
    /// The game state to modify
    pub state: &'a mut GameState,
    /// Card database for looking up card definitions
    pub card_db: &'a CardDatabase,
    /// Effect queue for processing triggered effects
    pub effect_queue: &'a mut EffectQueue,
    /// Optional effect tracer for debugging
    pub tracer: Option<&'a mut EffectTracer>,
}

impl<'a> ActionContext<'a> {
    /// Create a new action context.
    pub fn new(
        state: &'a mut GameState,
        card_db: &'a CardDatabase,
        effect_queue: &'a mut EffectQueue,
        tracer: Option<&'a mut EffectTracer>,
    ) -> Self {
        Self {
            state,
            card_db,
            effect_queue,
            tracer,
        }
    }

    /// Process all queued effects with optional tracer.
    pub fn process_effects(&mut self) {
        match self.tracer {
            Some(ref mut t) => {
                self.effect_queue
                    .process_all_with_tracer(self.state, self.card_db, Some(t));
            }
            None => {
                self.effect_queue.process_all(self.state, self.card_db);
            }
        }
    }

    /// Process effects and reborrow tracer for subsequent use.
    ///
    /// Use this when you need to call process_effects multiple times
    /// in a single function (e.g., OnAttack effects before combat).
    pub fn process_effects_reborrow(&mut self) {
        self.effect_queue.process_all_with_tracer(
            self.state,
            self.card_db,
            self.tracer.as_deref_mut(),
        );
    }
}
