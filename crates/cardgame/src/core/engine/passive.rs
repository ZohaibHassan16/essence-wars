//! Passive effect helpers for supports and commanders.
//!
//! This module contains helper functions for applying and removing passive
//! effects from supports and commanders to creatures. Supports can grant attack
//! bonuses, health bonuses, or keywords to friendly creatures while they are in
//! play. Commander passives are always active and cannot be removed.

use crate::core::cards::{
    AbilityDefinition, CardDatabase, CardType, CommanderAbility, CommanderPassiveEffect,
    EffectDefinition, PassiveModifier,
};
use crate::core::effects::{Effect, EffectTarget, TargetingRule};
use crate::core::keywords::Keywords;
use crate::core::state::{Creature, Support};
use crate::core::types::{CardId, PlayerId};

/// Apply a single passive modifier to a creature.
pub(super) fn apply_passive_to_creature(creature: &mut Creature, modifier: &PassiveModifier) {
    match modifier {
        PassiveModifier::AttackBonus(amount) => {
            creature.attack = creature.attack.saturating_add(*amount);
        }
        PassiveModifier::HealthBonus(amount) => {
            creature.current_health = creature.current_health.saturating_add(*amount);
            creature.max_health = creature.max_health.saturating_add(*amount);
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
            creature.attack = creature.attack.saturating_sub(*amount);
        }
        PassiveModifier::HealthBonus(amount) => {
            creature.current_health = creature.current_health.saturating_sub(*amount);
            creature.max_health = creature.max_health.saturating_sub(*amount);
            // Ensure health and max_health don't go below 1 from passive removal
            // (damage should kill, not passive loss)
            if creature.current_health < 1 {
                creature.current_health = 1;
            }
            if creature.max_health < 1 {
                creature.max_health = 1;
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
        EffectDefinition::Damage { amount, filter } => {
            let target = match &ability.targeting {
                TargetingRule::NoTarget => EffectTarget::AllEnemyCreatures(source_owner),
                TargetingRule::TargetEnemyCreature => EffectTarget::AllEnemyCreatures(source_owner),
                TargetingRule::TargetEnemyPlayer => EffectTarget::Player(source_owner.opponent()),
                _ => EffectTarget::AllEnemyCreatures(source_owner),
            };
            Some(Effect::Damage { target, amount: *amount, filter: filter.clone() })
        }
        EffectDefinition::Heal { amount, filter } => {
            // For supports, NoTarget heals should heal the player
            let target = match &ability.targeting {
                TargetingRule::NoTarget => EffectTarget::Player(source_owner),
                TargetingRule::TargetPlayer => EffectTarget::Player(source_owner),
                TargetingRule::TargetAllyCreature => EffectTarget::AllAllyCreatures(source_owner),
                _ => EffectTarget::Player(source_owner),
            };
            Some(Effect::Heal { target, amount: *amount, filter: filter.clone() })
        }
        EffectDefinition::Draw { count } => {
            Some(Effect::Draw { player: source_owner, count: *count })
        }
        EffectDefinition::BuffStats { attack, health, filter } => {
            // Buff all friendly creatures
            Some(Effect::BuffStats {
                target: EffectTarget::AllAllyCreatures(source_owner),
                attack: *attack,
                health: *health,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Destroy { filter: _ } => None, // Needs specific targeting
        EffectDefinition::GrantKeyword { keyword, filter } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::GrantKeyword {
                target: EffectTarget::AllAllyCreatures(source_owner),
                keyword: kw.0,
                filter: filter.clone(),
            })
        }
        EffectDefinition::RemoveKeyword { keyword, filter } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::RemoveKeyword {
                target: EffectTarget::AllEnemyCreatures(source_owner),
                keyword: kw.0,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Silence { filter: _ } => None, // Needs specific targeting
        EffectDefinition::GainEssence { amount } => {
            Some(Effect::GainEssence { player: source_owner, amount: *amount })
        }
        EffectDefinition::RefreshCreature => {
            Some(Effect::RefreshCreature {
                target: EffectTarget::AllAllyCreatures(source_owner),
            })
        }
        EffectDefinition::Bounce { filter } => {
            // Bounce typically targets enemy creatures
            let target = match &ability.targeting {
                TargetingRule::TargetAllyCreature => EffectTarget::AllAllyCreatures(source_owner),
                _ => EffectTarget::AllEnemyCreatures(source_owner),
            };
            Some(Effect::Bounce { target, filter: filter.clone() })
        }
        EffectDefinition::SummonToken { token } => {
            Some(Effect::SummonToken {
                owner: source_owner,
                token: token.to_token_definition(),
                slot: None,
            })
        }
        EffectDefinition::Transform { .. } => None, // Needs specific targeting
        EffectDefinition::Copy => None, // Needs specific targeting
    }
}

// =============================================================================
// COMMANDER PASSIVE EFFECTS
// =============================================================================

/// Apply a commander's passive effect to a creature.
///
/// Commander passives work like support passives but are always active and
/// cannot be removed. They affect all friendly creatures.
pub(super) fn apply_commander_passive_to_creature(
    creature: &mut Creature,
    effect: &CommanderPassiveEffect,
) {
    match effect {
        CommanderPassiveEffect::GrantKeyword { keyword } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            creature.keywords.add(kw.0);
        }
        CommanderPassiveEffect::BuffStats { attack, health } => {
            creature.attack = creature.attack.saturating_add(*attack);
            if *health != 0 {
                creature.current_health = creature.current_health.saturating_add(*health);
                creature.max_health = creature.max_health.saturating_add(*health);
            }
        }
    }
}

/// Apply a player's commander passive to a newly placed creature.
///
/// Called when a creature enters play. Looks up the player's commander
/// and applies its passive ability if it has one.
pub(super) fn apply_commander_passive_to_new_creature(
    creature: &mut Creature,
    commander_id: Option<CardId>,
    card_db: &CardDatabase,
) {
    if let Some(cmd_id) = commander_id {
        if let Some(commander) = card_db.get_commander(cmd_id) {
            if let CommanderAbility::Passive { passive_ability } = &commander.ability {
                apply_commander_passive_to_creature(creature, &passive_ability.effect);
            }
        }
    }
}

/// Apply a commander's passive effect to all existing creatures.
///
/// Called at game start to apply commander passives to any creatures
/// that may already exist (though typically the board is empty at start).
pub(super) fn apply_commander_passive_to_all_creatures(
    commander_id: Option<CardId>,
    creatures: &mut [Creature],
    card_db: &CardDatabase,
) {
    if let Some(cmd_id) = commander_id {
        if let Some(commander) = card_db.get_commander(cmd_id) {
            if let CommanderAbility::Passive { passive_ability } = &commander.ability {
                for creature in creatures {
                    apply_commander_passive_to_creature(creature, &passive_ability.effect);
                }
            }
        }
    }
}
