//! Ability action execution.
//!
//! Handles creature ability usage including validation and effect processing.

use crate::core::actions::Target;
use crate::core::cards::CardType;
use crate::core::effects::{EffectSource, EffectTarget};
use crate::core::engine::effect_convert::effect_def_to_effect_with_target;
use crate::core::types::Slot;

use super::ActionContext;

/// Execute a UseAbility action.
///
/// Uses a creature's ability at the specified slot with the given target.
/// The ability is identified by ability_index (0-based).
///
/// # Errors
/// - Returns error if no creature exists at the slot
/// - Returns error if creature is silenced
/// - Returns error if ability_index is out of bounds
/// - Returns error if the card definition is not found
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

    // Convert target to EffectTarget
    let effect_target = match target {
        Target::NoTarget => EffectTarget::None,
        Target::EnemySlot(s) => EffectTarget::Creature {
            owner: current_player.opponent(),
            slot: s
        },
        Target::Self_ => EffectTarget::Creature {
            owner: current_player,
            slot
        },
    };

    // Queue ability effects
    let source = EffectSource::Creature { owner: current_player, slot };

    for effect_def in &ability.effects {
        // Convert EffectDefinition to Effect with the resolved target
        if let Some(effect) = effect_def_to_effect_with_target(effect_def, effect_target, current_player) {
            ctx.effect_queue.push(effect, source);
        }
    }

    // Process all effects
    ctx.process_effects();

    Ok(())
}
