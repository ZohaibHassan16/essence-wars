//! Face attack resolution.
//!
//! Handles attacks against empty slots that deal damage directly to the enemy player.

use crate::core::cards::CardDatabase;
use crate::core::effects::Trigger;
use crate::core::engine::EffectQueue;
use crate::core::keywords::Keywords;
use crate::core::state::GameState;
use crate::core::tracing::{CombatTrace, CombatTracer};
use crate::core::types::{PlayerId, Slot};

use super::death::check_game_over;
use super::triggers::trigger_creature_ability;
use super::CombatResult;

/// Resolve a face attack (attacking an empty slot).
///
/// Face attacks deal damage directly to the enemy player's life total.
/// Lifesteal still applies, healing the attacker's controller.
#[allow(clippy::too_many_arguments)]
pub fn resolve_face_attack(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    attacker_player: PlayerId,
    attacker_slot: Slot,
    defender_player: PlayerId,
    defender_slot: Slot,
    attacker_attack: u8,
    attacker_keywords: Keywords,
    tracer: Option<&mut CombatTracer>,
) -> CombatResult {
    // Get attacker health for trace
    let attacker_health = state.players[attacker_player.index()]
        .get_creature(attacker_slot)
        .map(|c| c.current_health)
        .unwrap_or(0);

    // Create trace if enabled
    let mut trace = tracer.as_ref().and_then(|t| {
        if t.is_enabled() {
            Some(CombatTrace::new_face_attack(
                attacker_player,
                attacker_slot,
                attacker_attack as i8,
                attacker_health,
                attacker_keywords,
                defender_slot,
            ))
        } else {
            None
        }
    });

    let damage = attacker_attack;

    // Deal face damage
    state.players[defender_player.index()].life =
        state.players[defender_player.index()].life.saturating_sub(damage as i16);

    // Track total damage dealt
    state.players[attacker_player.index()].total_damage_dealt += damage as u16;

    // Log face damage
    if let Some(ref mut t) = trace {
        t.log_face_damage(damage);
    }

    // Apply Lifesteal
    let healed = apply_lifesteal(state, attacker_player, attacker_keywords, damage, &mut trace);

    // Mark attacker as having attacked
    mark_attacked(state, attacker_player, attacker_slot);

    // Trigger OnAttack effect (if creature has one)
    trigger_creature_ability(state, card_db, effect_queue, attacker_player, attacker_slot, Trigger::OnAttack);

    // Check for game over
    check_game_over(state);

    // Log completion
    if let Some(ref mut t) = trace {
        t.log_complete(false, false);
    }

    // Add trace to tracer
    if let (Some(tracer), Some(trace)) = (tracer, trace) {
        tracer.add_trace(trace);
    }

    CombatResult {
        attacker_damage_dealt: damage,
        defender_damage_dealt: 0,
        attacker_died: false,
        defender_died: false,
        face_damage: damage,
        attacker_healed: healed,
    }
}

/// Apply Lifesteal healing to the attacker's controller.
fn apply_lifesteal(
    state: &mut GameState,
    attacker_player: PlayerId,
    attacker_keywords: Keywords,
    damage: u8,
    trace: &mut Option<CombatTrace>,
) -> u8 {
    if attacker_keywords.has_lifesteal() && damage > 0 {
        // Heal attacker's controller, capped at 30 (max life per DESIGN.md)
        let current_life = state.players[attacker_player.index()].life;
        let new_life = (current_life + damage as i16).min(30);
        let actual_heal = (new_life - current_life) as u8;
        state.players[attacker_player.index()].life = new_life;

        if let Some(ref mut t) = trace {
            t.log_lifesteal(actual_heal);
        }

        actual_heal
    } else {
        0
    }
}

/// Mark a creature as having attacked this turn.
/// Also increments frenzy_stacks if the creature has Frenzy keyword.
pub fn mark_attacked(state: &mut GameState, player: PlayerId, slot: Slot) {
    if let Some(creature) = state.players[player.index()].get_creature_mut(slot) {
        creature.status.set_exhausted(true);

        // FRENZY: Gain +1 attack stack for subsequent attacks this turn
        if creature.keywords.has_frenzy() {
            creature.frenzy_stacks = creature.frenzy_stacks.saturating_add(1);
        }
    }
}
