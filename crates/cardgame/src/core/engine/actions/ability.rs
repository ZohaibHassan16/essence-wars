//! Ability action execution.
//!
//! Handles creature ability usage including validation and effect processing.
//! Supports both token abilities (stored on creature) and card-based abilities.

use crate::core::actions::Target;
use crate::core::cards::CardType;
use crate::core::effects::{Effect, EffectSource, EffectTarget, TokenEffect, Trigger};
use crate::core::engine::effect_convert::effect_def_to_effect_with_target;
use crate::core::types::Slot;

use super::ActionContext;

/// Execute a UseAbility action.
///
/// Uses a creature's ability at the specified slot with the given target.
/// The ability is identified by ability_index (0-based).
///
/// Supports:
/// 1. Token abilities (stored in creature.token_data)
/// 2. Card-based activated abilities (looked up via card_db, Trigger::Activated)
///
/// # Errors
/// - Returns error if no creature exists at the slot
/// - Returns error if creature is silenced
/// - Returns error if creature is exhausted
/// - Returns error if ability_index is out of bounds
/// - Returns error if insufficient essence
pub fn execute_use_ability(
    ctx: &mut ActionContext,
    slot: Slot,
    ability_index: u8,
    target: Target,
) -> Result<(), String> {
    let current_player = ctx.state.active_player;

    // Get creature at slot
    let creature = ctx.state.players[current_player.index()]
        .get_creature(slot)
        .ok_or("No creature at slot")?;

    // Silenced creatures can't use abilities
    if creature.status.is_silenced() {
        return Err("Creature is silenced".to_string());
    }

    // Exhausted creatures can't use activated abilities
    if creature.status.is_exhausted() {
        return Err("Creature is exhausted".to_string());
    }

    // Check for token abilities first (clone to avoid borrow issues with ctx mutation)
    if let Some(token_data) = creature.token_data.clone() {
        return execute_token_ability(ctx, slot, ability_index, target, &token_data.abilities);
    }

    // Fall back to card-based abilities
    execute_card_ability(ctx, slot, ability_index, target)
}

/// Execute a token ability (stored directly on the creature).
fn execute_token_ability(
    ctx: &mut ActionContext,
    slot: Slot,
    ability_index: u8,
    target: Target,
    abilities: &[crate::core::effects::TokenAbility],
) -> Result<(), String> {
    let current_player = ctx.state.active_player;

    // Get the specific ability
    let ability = abilities.get(ability_index as usize)
        .ok_or("Invalid ability index")?;

    // Check and deduct essence cost
    let player_state = &mut ctx.state.players[current_player.index()];
    if ability.essence_cost > player_state.current_essence {
        return Err("Not enough essence".to_string());
    }
    player_state.current_essence -= ability.essence_cost;

    // Exhaust the creature (tap it)
    if let Some(creature) = player_state.get_creature_mut(slot) {
        creature.status.set_exhausted(true);
    }

    // Convert target to EffectTarget
    // For token abilities:
    // - NoTarget = enemy commander (face damage)
    // - EnemySlot = specific enemy creature
    let effect_target = match target {
        Target::NoTarget => EffectTarget::Player(current_player.opponent()),
        Target::EnemySlot(s) => EffectTarget::Creature {
            owner: current_player.opponent(),
            slot: s,
        },
        Target::Self_ => EffectTarget::Creature {
            owner: current_player,
            slot,
        },
    };

    // Queue effects
    let source = EffectSource::Creature { owner: current_player, slot };

    for token_effect in &ability.effects {
        match token_effect {
            TokenEffect::DestroySelf => {
                ctx.effect_queue.push(
                    Effect::DestroySelf { owner: current_player, slot },
                    source,
                );
            }
            TokenEffect::Damage { amount } => {
                ctx.effect_queue.push(
                    Effect::Damage {
                        target: effect_target,
                        amount: *amount,
                        filter: None,
                    },
                    source,
                );
            }
            TokenEffect::Debuff { attack, health } => {
                // Debuff uses BuffStats with negative values on the target creature
                ctx.effect_queue.push(
                    Effect::BuffStats {
                        target: effect_target,
                        attack: *attack,
                        health: *health,
                        filter: None,
                    },
                    source,
                );
            }
            TokenEffect::HealSelf { amount } => {
                // Heal the ability owner's commander directly (capped at 30)
                let owner_idx = current_player.index();
                ctx.state.players[owner_idx].life =
                    (ctx.state.players[owner_idx].life + *amount as i16).min(30);
            }
        }
    }

    // Process all effects
    ctx.process_effects();

    Ok(())
}

/// Execute a card-based ability (looked up via card database).
fn execute_card_ability(
    ctx: &mut ActionContext,
    slot: Slot,
    ability_index: u8,
    target: Target,
) -> Result<(), String> {
    let current_player = ctx.state.active_player;

    // Get creature at slot
    let creature = ctx.state.players[current_player.index()]
        .get_creature(slot)
        .ok_or("No creature at slot")?;

    let card_id = creature.card_id;

    // Get card definition
    let card_def = ctx.card_db.get(card_id)
        .ok_or("Card not found")?;

    // Get abilities from card type
    let abilities = match &card_def.card_type {
        CardType::Creature { abilities, .. } => abilities,
        _ => return Err("Not a creature card".to_string()),
    };

    // Get the specific ability
    let ability = abilities.get(ability_index as usize)
        .ok_or("Invalid ability index")?;

    // Verify this is an Activated ability
    if ability.trigger != Trigger::Activated {
        return Err("Not an activated ability".to_string());
    }

    // Check and deduct essence cost
    let player_state = &mut ctx.state.players[current_player.index()];
    if ability.essence_cost > player_state.current_essence {
        return Err("Not enough essence".to_string());
    }
    player_state.current_essence -= ability.essence_cost;

    // Exhaust the creature (tap it)
    if let Some(creature) = player_state.get_creature_mut(slot) {
        creature.status.set_exhausted(true);
    }

    // Convert target to EffectTarget
    let effect_target = match target {
        Target::NoTarget => EffectTarget::None,
        Target::EnemySlot(s) => EffectTarget::Creature {
            owner: current_player.opponent(),
            slot: s,
        },
        Target::Self_ => EffectTarget::Creature {
            owner: current_player,
            slot,
        },
    };

    // Queue ability effects
    let source = EffectSource::Creature { owner: current_player, slot };

    // Clone effects to avoid borrow issues
    let effects = ability.effects.clone();
    for effect_def in &effects {
        // Convert EffectDefinition to Effect with the resolved target
        if let Some(effect) = effect_def_to_effect_with_target(effect_def, effect_target, current_player) {
            ctx.effect_queue.push(effect, source);
        }
    }

    // Process all effects
    ctx.process_effects();

    Ok(())
}
