//! Combat trigger handling.
//!
//! Provides a unified function for triggering creature abilities during combat.

use crate::core::cards::CardDatabase;
use crate::core::effects::{EffectSource, Trigger};
use crate::core::engine::EffectQueue;
use crate::core::state::GameState;
use crate::core::types::{PlayerId, Slot};

/// Trigger a creature's ability of a specific type.
///
/// This is the unified handler for all creature-triggered abilities during combat:
/// - OnAttack
/// - OnDealDamage
/// - OnTakeDamage
/// - OnKill
/// - OnDeath
/// - OnAllyDeath
///
/// # Arguments
/// * `state` - The current game state
/// * `card_db` - Card database for looking up card definitions
/// * `effect_queue` - Effect queue for queuing triggered effects
/// * `player` - The player who owns the creature
/// * `slot` - The slot of the creature
/// * `trigger` - The trigger type to check for
pub fn trigger_creature_ability(
    state: &GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    player: PlayerId,
    slot: Slot,
    trigger: Trigger,
) {
    let creature = match state.players[player.index()].get_creature(slot) {
        Some(c) => c,
        None => return,
    };

    // Don't trigger if silenced
    if creature.status.is_silenced() {
        return;
    }

    let card_id = creature.card_id;
    let card_def = match card_db.get(card_id) {
        Some(c) => c,
        None => return,
    };

    let abilities = match card_def.creature_abilities() {
        Some(a) => a,
        None => return,
    };

    for ability in abilities {
        if ability.trigger == trigger {
            let source = EffectSource::Creature { owner: player, slot };
            for effect_def in &ability.effects {
                if let Some(effect) = crate::engine::effect_def_to_triggered_effect(
                    effect_def,
                    player,
                    slot,
                    ability,
                ) {
                    effect_queue.push(effect, source);
                }
            }
        }
    }
}

/// Trigger OnAllyDeath for all friendly creatures except the one that died.
///
/// This iterates through all creatures belonging to the same player and triggers
/// their OnAllyDeath abilities.
pub fn trigger_ally_death_for_allies(
    state: &GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    player: PlayerId,
    dead_slot: Slot,
) {
    // Collect ally info to avoid borrow issues
    let ally_creatures: Vec<(Slot, crate::types::CardId, bool)> = state.players[player.index()]
        .creatures
        .iter()
        .filter(|c| c.slot != dead_slot)
        .map(|c| (c.slot, c.card_id, c.status.is_silenced()))
        .collect();

    for (ally_slot, ally_card_id, ally_silenced) in ally_creatures {
        if ally_silenced {
            continue;
        }

        if let Some(card_def) = card_db.get(ally_card_id) {
            if let Some(abilities) = card_def.creature_abilities() {
                for ability in abilities {
                    if ability.trigger == Trigger::OnAllyDeath {
                        let source = EffectSource::Creature { owner: player, slot: ally_slot };
                        for effect_def in &ability.effects {
                            if let Some(effect) = crate::engine::effect_def_to_triggered_effect(
                                effect_def,
                                player,
                                ally_slot,
                                ability,
                            ) {
                                effect_queue.push(effect, source);
                            }
                        }
                    }
                }
            }
        }
    }
}
