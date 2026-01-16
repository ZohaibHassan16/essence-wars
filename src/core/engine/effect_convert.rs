//! Effect conversion and targeting functions.
//!
//! This module contains helper functions for resolving spell targets and
//! converting EffectDefinition to Effect enums with proper targeting.

use crate::core::cards::{AbilityDefinition, EffectDefinition};
use crate::core::effects::{Effect, EffectTarget, TargetingRule};
use crate::core::keywords::Keywords;
use crate::core::types::{PlayerId, Slot};

/// Resolve the target for a spell based on its targeting rule and the slot parameter.
///
/// Target encoding in slot parameter:
/// - For NoTarget spells: slot is ignored
/// - For creature-targeting spells: slot 0-4 indicates enemy creature slot, slot + 5 for friendly
/// - For TargetPlayer: slot 0 = enemy, slot 1 = self
/// - For TargetAny: slot 0-4 enemy creature, 5-9 friendly creature, 10 enemy player, 11 self
pub fn resolve_spell_target(
    targeting: &TargetingRule,
    slot: Slot,
    caster: PlayerId,
) -> Result<EffectTarget, String> {
    match targeting {
        TargetingRule::NoTarget => {
            Ok(EffectTarget::None)
        }
        TargetingRule::TargetCreature(_) => {
            // slot 0-4 = enemy creature, slot 5-9 = friendly creature
            if slot.0 < 5 {
                Ok(EffectTarget::Creature {
                    owner: caster.opponent(),
                    slot: Slot(slot.0),
                })
            } else if slot.0 < 10 {
                Ok(EffectTarget::Creature {
                    owner: caster,
                    slot: Slot(slot.0 - 5),
                })
            } else {
                Err("Invalid target slot for creature targeting".to_string())
            }
        }
        TargetingRule::TargetEnemyCreature => {
            // slot 0-4 = enemy creature slot
            if slot.0 < 5 {
                Ok(EffectTarget::Creature {
                    owner: caster.opponent(),
                    slot: Slot(slot.0),
                })
            } else {
                Err("Invalid target slot for enemy creature targeting".to_string())
            }
        }
        TargetingRule::TargetAllyCreature => {
            // slot 0-4 = friendly creature slot
            if slot.0 < 5 {
                Ok(EffectTarget::Creature {
                    owner: caster,
                    slot: Slot(slot.0),
                })
            } else {
                Err("Invalid target slot for ally creature targeting".to_string())
            }
        }
        TargetingRule::TargetPlayer => {
            // slot 0 = enemy, slot 1 = self
            if slot.0 == 0 {
                Ok(EffectTarget::Player(caster.opponent()))
            } else {
                Ok(EffectTarget::Player(caster))
            }
        }
        TargetingRule::TargetEnemyPlayer => {
            Ok(EffectTarget::Player(caster.opponent()))
        }
        TargetingRule::TargetAny => {
            // slot 0-4 enemy creature, 5-9 friendly creature, 10 enemy player, 11 self
            if slot.0 < 5 {
                Ok(EffectTarget::Creature {
                    owner: caster.opponent(),
                    slot: Slot(slot.0),
                })
            } else if slot.0 < 10 {
                Ok(EffectTarget::Creature {
                    owner: caster,
                    slot: Slot(slot.0 - 5),
                })
            } else if slot.0 == 10 {
                Ok(EffectTarget::Player(caster.opponent()))
            } else {
                Ok(EffectTarget::Player(caster))
            }
        }
        TargetingRule::TargetSlot => {
            // For summoning effects - just return the slot info
            if slot.0 < 5 {
                Ok(EffectTarget::Creature {
                    owner: caster,
                    slot: Slot(slot.0),
                })
            } else {
                Err("Invalid target slot".to_string())
            }
        }
    }
}

/// Convert an EffectDefinition to an Effect enum with a specific target.
/// Used primarily for spell effects where the target is resolved beforehand.
pub fn effect_def_to_effect_with_target(
    def: &EffectDefinition,
    target: EffectTarget,
    source_player: PlayerId,
) -> Option<Effect> {
    match def {
        EffectDefinition::Damage { amount, filter } => {
            Some(Effect::Damage {
                target,
                amount: *amount,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Heal { amount, filter } => {
            Some(Effect::Heal {
                target,
                amount: *amount,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Draw { count } => {
            // Draw always affects the caster
            Some(Effect::Draw {
                player: source_player,
                count: *count,
            })
        }
        EffectDefinition::BuffStats { attack, health, filter } => {
            Some(Effect::BuffStats {
                target,
                attack: *attack,
                health: *health,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Destroy { filter } => {
            Some(Effect::Destroy { target, filter: filter.clone() })
        }
        EffectDefinition::GrantKeyword { keyword, filter } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::GrantKeyword {
                target,
                keyword: kw.0,
                filter: filter.clone(),
            })
        }
        EffectDefinition::RemoveKeyword { keyword, filter } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::RemoveKeyword {
                target,
                keyword: kw.0,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Silence { filter } => {
            Some(Effect::Silence { target, filter: filter.clone() })
        }
        EffectDefinition::GainEssence { amount } => {
            Some(Effect::GainEssence {
                player: source_player,
                amount: *amount,
            })
        }
        EffectDefinition::RefreshCreature => {
            Some(Effect::RefreshCreature { target })
        }
        EffectDefinition::Bounce { filter } => {
            Some(Effect::Bounce { target, filter: filter.clone() })
        }
        EffectDefinition::SummonToken { token } => {
            // Summon a token creature for the caster
            Some(Effect::SummonToken {
                owner: source_player,
                token: token.to_token_definition(),
                slot: None, // Will use first available slot
            })
        }
        EffectDefinition::Transform { into } => {
            // Transform the target creature into a token
            Some(Effect::Transform {
                target,
                into: into.to_token_definition(),
            })
        }
        EffectDefinition::Copy => {
            // Copy the target creature for the caster
            Some(Effect::Copy {
                target,
                owner: source_player,
            })
        }
    }
}

/// Convert an EffectDefinition to an Effect for triggered abilities.
/// The target is inferred based on the effect type and trigger context.
pub fn effect_def_to_triggered_effect(
    def: &EffectDefinition,
    source_owner: PlayerId,
    source_slot: Slot,
    ability: &AbilityDefinition,
) -> Option<Effect> {
    // Determine the target based on the ability's targeting rule and effect type
    let default_target = EffectTarget::Creature {
        owner: source_owner,
        slot: source_slot,
    };

    match def {
        EffectDefinition::Damage { amount, filter } => {
            // For damage, check the targeting rule to determine who gets hit
            let target = match &ability.targeting {
                TargetingRule::NoTarget => {
                    // Default to all enemy creatures for AoE damage
                    EffectTarget::AllEnemyCreatures(source_owner)
                }
                TargetingRule::TargetEnemyCreature => {
                    // This should be resolved at cast time, but for triggers,
                    // we default to all enemy creatures
                    EffectTarget::AllEnemyCreatures(source_owner)
                }
                TargetingRule::TargetAllyCreature => {
                    EffectTarget::AllAllyCreatures(source_owner)
                }
                TargetingRule::TargetEnemyPlayer => {
                    EffectTarget::Player(source_owner.opponent())
                }
                _ => EffectTarget::AllEnemyCreatures(source_owner),
            };
            Some(Effect::Damage {
                target,
                amount: *amount,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Heal { amount, filter } => {
            // Heal typically targets self or allies
            let target = match &ability.targeting {
                TargetingRule::TargetAllyCreature | TargetingRule::NoTarget => {
                    default_target
                }
                TargetingRule::TargetPlayer => {
                    EffectTarget::Player(source_owner)
                }
                _ => default_target,
            };
            Some(Effect::Heal {
                target,
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
            // Buff typically targets self
            Some(Effect::BuffStats {
                target: default_target,
                attack: *attack,
                health: *health,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Destroy { filter: _ } => {
            // Destroy needs a specific target - this should be resolved differently
            // For now, return None as it needs targeting
            None
        }
        EffectDefinition::GrantKeyword { keyword, filter } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::GrantKeyword {
                target: default_target,
                keyword: kw.0,
                filter: filter.clone(),
            })
        }
        EffectDefinition::RemoveKeyword { keyword, filter } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::RemoveKeyword {
                target: default_target,
                keyword: kw.0,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Silence { filter } => {
            Some(Effect::Silence {
                target: default_target,
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
                target: default_target,
            })
        }
        EffectDefinition::Bounce { filter } => {
            // Bounce typically targets enemy creatures
            let target = match &ability.targeting {
                TargetingRule::TargetEnemyCreature => {
                    EffectTarget::AllEnemyCreatures(source_owner)
                }
                TargetingRule::TargetAllyCreature => {
                    EffectTarget::AllAllyCreatures(source_owner)
                }
                _ => EffectTarget::AllEnemyCreatures(source_owner),
            };
            Some(Effect::Bounce {
                target,
                filter: filter.clone(),
            })
        }
        EffectDefinition::SummonToken { token } => {
            // Summon a token creature for the ability owner
            Some(Effect::SummonToken {
                owner: source_owner,
                token: token.to_token_definition(),
                slot: None, // Will use first available slot
            })
        }
        EffectDefinition::Transform { into } => {
            // Transform needs a specific target - typically used with targeting
            // For triggered abilities without targeting, this doesn't make sense
            // Return None to indicate it needs proper targeting resolution
            let _ = into; // Suppress unused warning
            None
        }
        EffectDefinition::Copy => {
            // Copy needs a specific target - typically used with targeting
            // For triggered abilities without targeting, this doesn't make sense
            None
        }
    }
}
