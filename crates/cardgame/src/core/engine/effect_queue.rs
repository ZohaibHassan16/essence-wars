//! Effect queue for processing game effects in FIFO order.
//!
//! When an effect triggers another effect, the new effect goes to the back
//! of the queue. This avoids recursion and makes resolution predictable.

use std::collections::VecDeque;

use crate::core::cards::CardDatabase;
use crate::core::effects::{Effect, EffectResult, EffectSource, EffectTarget, PendingEffect};
use crate::core::keywords::Keywords;
use crate::core::state::GameState;
use crate::core::tracing::EffectTracer;
use crate::core::types::{PlayerId, Slot};

use super::effect_context::EffectContext;
use super::handlers::create_handler;
use super::triggers::process_deaths;

/// Effect queue for processing game effects in FIFO order.
///
/// When an effect triggers another effect, the new effect goes to the back
/// of the queue. This avoids recursion and makes resolution predictable.
#[derive(Debug, Default)]
pub struct EffectQueue {
    queue: VecDeque<PendingEffect>,
    /// Creatures marked for death (processed after each effect)
    pending_deaths: Vec<(PlayerId, Slot)>,
    /// Accumulated result from effect resolution (for conditional triggers)
    accumulated_result: EffectResult,
}

impl EffectQueue {
    /// Create a new empty effect queue
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            pending_deaths: Vec::new(),
            accumulated_result: EffectResult::none(),
        }
    }

    /// Reset the accumulated result (call before processing a new spell/ability)
    pub fn reset_accumulated_result(&mut self) {
        self.accumulated_result = EffectResult::none();
    }

    /// Get the accumulated result from effect resolution
    pub fn accumulated_result(&self) -> &EffectResult {
        &self.accumulated_result
    }

    /// Add an effect to the back of the queue
    pub fn push(&mut self, effect: Effect, source: EffectSource) {
        self.queue.push_back(PendingEffect::new(effect, source));
    }

    /// Add a pending effect to the back of the queue
    pub fn push_pending(&mut self, pending: PendingEffect) {
        self.queue.push_back(pending);
    }

    /// Add multiple effects to the queue
    pub fn push_all(&mut self, effects: impl IntoIterator<Item = PendingEffect>) {
        self.queue.extend(effects);
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get the number of effects in the queue
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Process all effects in the queue until empty
    pub fn process_all(
        &mut self,
        state: &mut GameState,
        card_db: &CardDatabase,
    ) {
        self.process_all_with_tracer(state, card_db, None);
    }

    /// Process all effects in the queue with optional tracer
    pub fn process_all_with_tracer(
        &mut self,
        state: &mut GameState,
        card_db: &CardDatabase,
        mut tracer: Option<&mut EffectTracer>,
    ) {
        let initial_queue_size = self.queue.len();
        let mut effects_processed = 0;

        // Log queue start if tracer enabled
        if let Some(ref mut t) = tracer {
            t.log_queue_start(initial_queue_size);
        }

        while let Some(pending) = self.queue.pop_front() {
            // Log effect start
            if let Some(ref mut t) = tracer {
                t.log_effect_start(&pending.effect, self.queue.len());
            }

            // Clone effect for logging completion
            let effect_for_log = pending.effect.clone();

            self.resolve_effect(pending, state, card_db);
            effects_processed += 1;

            // Log effect complete
            if let Some(ref mut t) = tracer {
                t.log_effect_complete(&effect_for_log, vec![]);
            }

            // Process any pending deaths after each effect
            process_deaths(
                &mut self.pending_deaths,
                state,
                card_db,
                &mut |effect, source| self.queue.push_back(PendingEffect::new(effect, source)),
            );

            // Check for game over conditions
            if state.is_terminal() {
                // Clear remaining effects if game is over
                self.queue.clear();
                break;
            }
        }

        // Log queue complete
        if let Some(t) = tracer {
            t.log_queue_complete(effects_processed);
        }
    }

    /// Check if a creature has Ward and consume it.
    /// Returns true if Ward was consumed (effect should be blocked).
    /// Ward only blocks single-target effects, not AoE effects.
    fn check_and_consume_ward(state: &mut GameState, owner: PlayerId, slot: Slot) -> bool {
        if let Some(creature) = state.players[owner.index()].get_creature_mut(slot) {
            if creature.keywords.has_ward() {
                creature.keywords.remove(Keywords::WARD);
                return true; // Ward consumed, block the effect
            }
        }
        false
    }

    /// Extract single-target creature info from an effect for Ward checking.
    fn get_single_target_creature(effect: &Effect) -> Option<(PlayerId, Slot)> {
        match effect {
            Effect::Damage { target: EffectTarget::Creature { owner, slot }, .. } |
            Effect::Heal { target: EffectTarget::Creature { owner, slot }, .. } |
            Effect::BuffStats { target: EffectTarget::Creature { owner, slot }, .. } |
            Effect::SetStats { target: EffectTarget::Creature { owner, slot }, .. } |
            Effect::Destroy { target: EffectTarget::Creature { owner, slot }, .. } |
            Effect::GrantKeyword { target: EffectTarget::Creature { owner, slot }, .. } |
            Effect::RemoveKeyword { target: EffectTarget::Creature { owner, slot }, .. } |
            Effect::Silence { target: EffectTarget::Creature { owner, slot }, .. } |
            Effect::Bounce { target: EffectTarget::Creature { owner, slot }, .. } |
            Effect::Transform { target: EffectTarget::Creature { owner, slot }, .. } => {
                Some((*owner, *slot))
            }
            _ => None,
        }
    }

    /// Resolve a single effect using the handler strategy pattern.
    fn resolve_effect(
        &mut self,
        pending: PendingEffect,
        state: &mut GameState,
        card_db: &CardDatabase,
    ) {
        let source_player = match pending.source {
            EffectSource::Card(_) => state.active_player,
            EffectSource::Creature { owner, .. } => owner,
            EffectSource::Support { owner, .. } => owner,
            EffectSource::Commander { owner } => owner,
            EffectSource::System => state.active_player,
        };

        // Create the appropriate handler for this effect type
        let handler = create_handler(&pending.effect, source_player);

        // Check Ward for single-target effects that are blocked by Ward
        if handler.blocked_by_ward() {
            if let Some((owner, slot)) = Self::get_single_target_creature(&pending.effect) {
                if Self::check_and_consume_ward(state, owner, slot) {
                    return; // Effect blocked by Ward
                }
            }
        }

        // Create context and apply the effect
        let mut ctx = EffectContext::new(
            state,
            card_db,
            &mut self.pending_deaths,
            &mut self.accumulated_result,
            &mut self.queue,
        );

        handler.apply(&mut ctx);
    }
}
