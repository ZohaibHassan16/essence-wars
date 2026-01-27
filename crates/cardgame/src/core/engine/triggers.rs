//! Trigger checking and death processing utilities.
//!
//! This module contains functions for checking and queuing triggered abilities
//! from creatures, as well as processing creature deaths.

use crate::core::cards::{CardDatabase, EffectDefinition};
use crate::core::effects::{Effect, EffectSource, EffectTarget, Trigger};
use crate::core::keywords::Keywords;
use crate::core::state::GameState;
use crate::core::types::{PlayerId, Slot};

use super::effect_context::EffectContext;
use super::passive::{
    collect_commander_ally_death_effects,
    collect_commander_any_death_effects,
    collect_commander_enemy_death_effects,
};

/// Check for and queue triggered abilities on a creature.
///
/// This function checks if a creature has any abilities that trigger
/// on the given trigger type, and if so, queues them for processing.
pub fn check_creature_triggers(
    ctx: &mut EffectContext,
    trigger: Trigger,
    owner: PlayerId,
    slot: Slot,
) {
    let Some(creature) = ctx.state.players[owner.index()].get_creature(slot) else {
        return;
    };

    // Silenced creatures don't trigger abilities
    if creature.status.is_silenced() {
        return;
    }

    let card_id = creature.card_id;
    let Some(card_def) = ctx.card_db.get(card_id) else {
        return;
    };

    let Some(abilities) = card_def.creature_abilities() else {
        return;
    };

    for ability in abilities {
        if ability.trigger == trigger {
            // Convert ability effects to Effect enum and queue them
            let source = EffectSource::Creature { owner, slot };
            for effect_def in &ability.effects {
                if let Some(effect) = effect_def_to_effect(effect_def, owner, slot) {
                    ctx.push_effect(effect, source);
                }
            }
        }
    }
}

/// Convert an EffectDefinition to an Effect enum.
///
/// This handles the conversion from card definition effects to runtime effects,
/// inferring appropriate targets for triggered abilities.
pub fn effect_def_to_effect(
    def: &EffectDefinition,
    source_owner: PlayerId,
    source_slot: Slot,
) -> Option<Effect> {
    match def {
        EffectDefinition::Damage { amount, filter } => {
            // Default to targeting all enemy creatures for triggered abilities
            Some(Effect::Damage {
                target: EffectTarget::AllEnemyCreatures(source_owner),
                amount: *amount,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Heal { amount, filter } => {
            Some(Effect::Heal {
                target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                amount: *amount,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Draw { count } => {
            Some(Effect::Draw {
                player: source_owner,
                count: *count,
            })
        }
        EffectDefinition::BuffStats { attack, health, filter } => {
            Some(Effect::BuffStats {
                target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                attack: *attack,
                health: *health,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Destroy { filter: _ } => {
            // Would need targeting info from ability definition
            None
        }
        EffectDefinition::GrantKeyword { keyword, filter } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::GrantKeyword {
                target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                keyword: kw.0,
                filter: filter.clone(),
            })
        }
        EffectDefinition::RemoveKeyword { keyword, filter } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::RemoveKeyword {
                target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                keyword: kw.0,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Silence { filter } => {
            Some(Effect::Silence {
                target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                filter: filter.clone(),
            })
        }
        EffectDefinition::GainEssence { amount } => {
            Some(Effect::GainEssence {
                player: source_owner,
                amount: *amount,
            })
        }
        EffectDefinition::RefreshCreature => {
            Some(Effect::RefreshCreature {
                target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
            })
        }
        EffectDefinition::Bounce { filter } => {
            // For triggered abilities, bounce targets enemy creatures
            Some(Effect::Bounce {
                target: EffectTarget::AllEnemyCreatures(source_owner),
                filter: filter.clone(),
            })
        }
        EffectDefinition::SummonToken { token } => {
            Some(Effect::SummonToken {
                owner: source_owner,
                token: token.to_token_definition(),
                slot: None,
            })
        }
        EffectDefinition::Transform { .. } => {
            // Transform needs specific targeting - not supported in this context
            None
        }
        EffectDefinition::Copy => {
            // Copy needs specific targeting - not supported in this context
            None
        }
    }
}

/// Process all pending deaths.
///
/// This function handles death processing including:
/// - Queuing OnDeath triggers from dying creatures
/// - Queuing OnAllyDeath triggers from allied creatures
/// - Queuing commander death triggers
/// - Removing dead creatures from the board
pub fn process_deaths(
    pending_deaths: &mut Vec<(PlayerId, Slot)>,
    state: &mut GameState,
    card_db: &CardDatabase,
    queue_effect: &mut impl FnMut(Effect, EffectSource),
) {
    // Process deaths in the order they occurred
    while let Some((owner, slot)) = pending_deaths.pop() {
        // Check creature still exists
        let creature_info = state.players[owner.index()]
            .get_creature(slot)
            .map(|c| (c.card_id, c.status.is_silenced()));

        if let Some((card_id, is_silenced)) = creature_info {
            // Queue OnDeath triggers before removing
            if !is_silenced {
                if let Some(card_def) = card_db.get(card_id) {
                    if let Some(abilities) = card_def.creature_abilities() {
                        for ability in abilities {
                            if ability.trigger == Trigger::OnDeath {
                                let source = EffectSource::Creature { owner, slot };
                                for effect_def in &ability.effects {
                                    if let Some(effect) = effect_def_to_effect(
                                        effect_def,
                                        owner,
                                        slot,
                                    ) {
                                        queue_effect(effect, source);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Queue OnAllyDeath triggers for other friendly creatures
            let ally_creatures: Vec<Slot> = state.players[owner.index()]
                .creatures.iter()
                .filter(|c| c.slot != slot && !c.status.is_silenced())
                .map(|c| c.slot)
                .collect();

            for ally_slot in ally_creatures {
                if let Some(ally) = state.players[owner.index()].get_creature(ally_slot) {
                    if let Some(card_def) = card_db.get(ally.card_id) {
                        if let Some(abilities) = card_def.creature_abilities() {
                            for ability in abilities {
                                if ability.trigger == Trigger::OnAllyDeath {
                                    let source = EffectSource::Creature {
                                        owner,
                                        slot: ally_slot
                                    };
                                    for effect_def in &ability.effects {
                                        if let Some(effect) = effect_def_to_effect(
                                            effect_def,
                                            owner,
                                            ally_slot,
                                        ) {
                                            queue_effect(effect, source);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Queue OnAllyDeath commander trigger (e.g., Plague Sovereign)
            for (effect, source) in collect_commander_ally_death_effects(state, owner, card_db) {
                queue_effect(effect, source);
            }

            // Queue OnEnemyDeath commander trigger
            // This triggers for the opponent when one of your creatures dies
            for (effect, source) in collect_commander_enemy_death_effects(state, owner, card_db) {
                queue_effect(effect, source);
            }

            // Queue OnAnyDeath commander triggers for both players
            for (effect, source) in collect_commander_any_death_effects(state, owner, card_db) {
                queue_effect(effect, source);
            }
            for (effect, source) in collect_commander_any_death_effects(state, owner.opponent(), card_db) {
                queue_effect(effect, source);
            }

            // Remove the creature from the board
            state.players[owner.index()].creatures.retain(|c| c.slot != slot);
        }
    }
}
