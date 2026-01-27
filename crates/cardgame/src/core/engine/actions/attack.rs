//! Attack action execution.
//!
//! Handles creature attacks including validation, OnAttack triggers,
//! and delegation to the combat module.

use crate::core::combat;
use crate::core::tracing::CombatTracer;
use crate::core::types::Slot;

use super::ActionContext;

/// Execute an Attack action.
///
/// Delegates to the combat module for full keyword resolution including:
/// Quick, Ranged, Shield, Piercing, Lethal, and Lifesteal.
pub fn execute_attack(ctx: &mut ActionContext, attacker_slot: Slot, defender_slot: Slot) -> Result<(), String> {
    execute_attack_internal(ctx, attacker_slot, defender_slot, None)
}

/// Execute an attack action with optional combat tracer for debugging.
pub fn execute_attack_with_tracers(
    ctx: &mut ActionContext,
    attacker_slot: Slot,
    defender_slot: Slot,
    combat_tracer: Option<&mut CombatTracer>,
) -> Result<(), String> {
    execute_attack_internal(ctx, attacker_slot, defender_slot, combat_tracer)
}

/// Internal attack execution with optional combat tracer.
fn execute_attack_internal(
    ctx: &mut ActionContext,
    attacker_slot: Slot,
    defender_slot: Slot,
    combat_tracer: Option<&mut CombatTracer>,
) -> Result<(), String> {
    let current_player = ctx.state.active_player;

    // Validate attacker exists
    let attacker = ctx
        .state
        .players[current_player.index()]
        .get_creature(attacker_slot)
        .ok_or_else(|| "No creature at attacker slot".to_string())?;

    // Validate attacker can attack
    if !attacker.can_attack(ctx.state.current_turn) {
        return Err("Creature cannot attack".to_string());
    }

    // Check for commander OnAttack triggers (before combat resolution)
    let attack_effects =
        crate::core::engine::passive::collect_commander_attack_effects(ctx.state, current_player, ctx.card_db);
    for (effect, source) in attack_effects {
        ctx.effect_queue.push(effect, source);
    }

    // Process OnAttack effects before combat (e.g., Alpha of the Hunt's buff)
    ctx.process_effects_reborrow();

    // Delegate to combat module for full keyword resolution
    let _result = combat::resolve_combat(
        ctx.state,
        ctx.card_db,
        ctx.effect_queue,
        current_player,
        attacker_slot,
        defender_slot,
        combat_tracer,
    );

    // Process any triggered effects from combat
    ctx.process_effects();

    Ok(())
}
