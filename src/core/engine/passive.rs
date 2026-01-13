//! Support passive effect helpers.
//!
//! This module contains helper functions for applying and removing passive
//! effects from supports to creatures. Supports can grant attack bonuses,
//! health bonuses, or keywords to friendly creatures while they are in play.

use crate::core::cards::{AbilityDefinition, CardDatabase, CardType, EffectDefinition, PassiveModifier};
use crate::core::effects::{Effect, EffectTarget, TargetingRule};
use crate::core::keywords::Keywords;
use crate::core::state::{Creature, Support};
use crate::core::types::{CardId, PlayerId};

/// Apply a single passive modifier to a creature.
pub(super) fn apply_passive_to_creature(creature: &mut Creature, modifier: &PassiveModifier) {
    match modifier {
        PassiveModifier::AttackBonus(amount) => {
            creature.attack += *amount as i8;
        }
        PassiveModifier::HealthBonus(amount) => {
            creature.current_health += *amount as i8;
            creature.max_health += *amount as i8;
        }
        PassiveModifier::GrantKeyword(keyword_name) => {
            let kw = Keywords::from_names(&[keyword_name.as_str()]);
            creature.keywords.add(kw.0);
        }
    }
}

/// Remove a single passive modifier from a creature.
pub(super) fn remove_passive_from_creature(creature: &mut Creature, modifier: &PassiveModifier) {
    match modifier {
        PassiveModifier::AttackBonus(amount) => {
            creature.attack -= *amount as i8;
        }
        PassiveModifier::HealthBonus(amount) => {
            creature.current_health -= *amount as i8;
            creature.max_health -= *amount as i8;
            // Ensure health doesn't go below 1 from passive removal
            // (damage should kill, not passive loss)
            if creature.current_health < 1 {
                creature.current_health = 1;
            }
        }
        PassiveModifier::GrantKeyword(keyword_name) => {
            let kw = Keywords::from_names(&[keyword_name.as_str()]);
            creature.keywords.remove(kw.0);
        }
    }
}

/// Apply all passive effects from a player's supports to a specific creature.
pub(super) fn apply_all_support_passives_to_creature(
    creature: &mut Creature,
    supports: &[Support],
    card_db: &CardDatabase,
) {
    for support in supports {
        if let Some(card_def) = card_db.get(support.card_id) {
            if let CardType::Support { passive_effects, .. } = &card_def.card_type {
                for passive in passive_effects {
                    apply_passive_to_creature(creature, &passive.modifier);
                }
            }
        }
    }
}

/// Apply passive effects from a newly placed support to all existing creatures.
pub(super) fn apply_support_passives_to_all_creatures(
    support_card_id: CardId,
    creatures: &mut [Creature],
    card_db: &CardDatabase,
) {
    if let Some(card_def) = card_db.get(support_card_id) {
        if let CardType::Support { passive_effects, .. } = &card_def.card_type {
            for creature in creatures {
                for passive in passive_effects {
                    apply_passive_to_creature(creature, &passive.modifier);
                }
            }
        }
    }
}

/// Remove passive effects from a support being removed from all creatures.
pub(super) fn remove_support_passives_from_all_creatures(
    support_card_id: CardId,
    creatures: &mut [Creature],
    card_db: &CardDatabase,
) {
    if let Some(card_def) = card_db.get(support_card_id) {
        if let CardType::Support { passive_effects, .. } = &card_def.card_type {
            for creature in creatures {
                for passive in passive_effects {
                    remove_passive_from_creature(creature, &passive.modifier);
                }
            }
        }
    }
}

/// Convert an EffectDefinition to an Effect for support-triggered abilities.
/// This handles supports differently from creatures - e.g., NoTarget heals target the player.
pub fn support_effect_def_to_effect(
    def: &EffectDefinition,
    source_owner: PlayerId,
    ability: &AbilityDefinition,
) -> Option<Effect> {
    match def {
        EffectDefinition::Damage { amount } => {
            let target = match &ability.targeting {
                TargetingRule::NoTarget => EffectTarget::AllEnemyCreatures(source_owner),
                TargetingRule::TargetEnemyCreature => EffectTarget::AllEnemyCreatures(source_owner),
                TargetingRule::TargetEnemyPlayer => EffectTarget::Player(source_owner.opponent()),
                _ => EffectTarget::AllEnemyCreatures(source_owner),
            };
            Some(Effect::Damage { target, amount: *amount })
        }
        EffectDefinition::Heal { amount } => {
            // For supports, NoTarget heals should heal the player
            let target = match &ability.targeting {
                TargetingRule::NoTarget => EffectTarget::Player(source_owner),
                TargetingRule::TargetPlayer => EffectTarget::Player(source_owner),
                TargetingRule::TargetAllyCreature => EffectTarget::AllAllyCreatures(source_owner),
                _ => EffectTarget::Player(source_owner),
            };
            Some(Effect::Heal { target, amount: *amount })
        }
        EffectDefinition::Draw { count } => {
            Some(Effect::Draw { player: source_owner, count: *count })
        }
        EffectDefinition::BuffStats { attack, health } => {
            // Buff all friendly creatures
            Some(Effect::BuffStats {
                target: EffectTarget::AllAllyCreatures(source_owner),
                attack: *attack,
                health: *health,
            })
        }
        EffectDefinition::Destroy => None, // Needs specific targeting
        EffectDefinition::GrantKeyword { keyword } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::GrantKeyword {
                target: EffectTarget::AllAllyCreatures(source_owner),
                keyword: kw.0,
            })
        }
        EffectDefinition::RemoveKeyword { keyword } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::RemoveKeyword {
                target: EffectTarget::AllEnemyCreatures(source_owner),
                keyword: kw.0,
            })
        }
        EffectDefinition::Silence => None, // Needs specific targeting
        EffectDefinition::GainEssence { amount } => {
            Some(Effect::GainEssence { player: source_owner, amount: *amount })
        }
        EffectDefinition::RefreshCreature => {
            Some(Effect::RefreshCreature {
                target: EffectTarget::AllAllyCreatures(source_owner),
            })
        }
    }
}
