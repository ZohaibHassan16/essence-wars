//! Passive and triggered effect helpers for supports and commanders.
//!
//! This module contains helper functions for applying and removing passive
//! effects from supports and commanders to creatures. Supports can grant attack
//! bonuses, health bonuses, or keywords to friendly creatures while they are in
//! play. Commander passives are always active and cannot be removed.
//!
//! This module also handles commander triggered abilities (StartOfTurn,
//! OnCreaturePlayed, OnAllyDeath, OnEnemyDeath).

use crate::core::cards::{
    AbilityDefinition, CardDatabase, CardType, CommanderAbility, CommanderPassiveEffect,
    CommanderTrigger, EffectDefinition, PassiveModifier,
};
use crate::core::effects::{Effect, EffectSource, EffectTarget, TargetingRule};
use crate::core::keywords::Keywords;
use crate::core::state::{Creature, GameState, ResolvedCommanderPassive, Support};
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
            creature.keywords.add(Keywords::parse_keyword_name(keyword_name));
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
            creature.keywords.remove(Keywords::parse_keyword_name(keyword_name));
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
            let kw_bits = Keywords::parse_keyword_name(keyword);
            Some(Effect::GrantKeyword {
                target: EffectTarget::AllAllyCreatures(source_owner),
                keyword: kw_bits,
                filter: filter.clone(),
            })
        }
        EffectDefinition::RemoveKeyword { keyword, filter } => {
            let kw_bits = Keywords::parse_keyword_name(keyword);
            Some(Effect::RemoveKeyword {
                target: EffectTarget::AllEnemyCreatures(source_owner),
                keyword: kw_bits,
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
            let kw_bits = Keywords::parse_keyword_name(keyword);
            creature.keywords.add(kw_bits);
        }
        CommanderPassiveEffect::BuffStats { attack, health } => {
            creature.attack = creature.attack.saturating_add(*attack);
            if *health != 0 {
                creature.current_health = creature.current_health.saturating_add(*health);
                creature.max_health = creature.max_health.saturating_add(*health);
            }
        }
        CommanderPassiveEffect::GrantKeywordAndBuff {
            keyword,
            attack,
            health,
        } => {
            // Grant keyword
            let kw_bits = Keywords::parse_keyword_name(keyword);
            creature.keywords.add(kw_bits);
            // Apply stat buff
            creature.attack = creature.attack.saturating_add(*attack);
            if *health != 0 {
                creature.current_health = creature.current_health.saturating_add(*health);
                creature.max_health = creature.max_health.saturating_add(*health);
            }
        }
        CommanderPassiveEffect::BuffStatsIfKeyword {
            required_keyword,
            attack,
            health,
        } => {
            // Only apply buff if creature has the required keyword
            let required_kw_bits = Keywords::parse_keyword_name(required_keyword);
            if creature.keywords.has(required_kw_bits) {
                creature.attack = creature.attack.saturating_add(*attack);
                if *health != 0 {
                    creature.current_health = creature.current_health.saturating_add(*health);
                    creature.max_health = creature.max_health.saturating_add(*health);
                }
            }
        }
        CommanderPassiveEffect::GrantKeywordIfKeyword {
            required_keyword,
            granted_keyword,
        } => {
            // Only grant keyword if creature has the required keyword
            let required_kw_bits = Keywords::parse_keyword_name(required_keyword);
            if creature.keywords.has(required_kw_bits) {
                let granted_kw_bits = Keywords::parse_keyword_name(granted_keyword);
                creature.keywords.add(granted_kw_bits);
            }
        }
    }
}

/// Apply a commander's passive effect to all existing creatures.
///
/// Called at game start to apply commander passives to any creatures
/// that may already exist (though typically the board is empty at start).
#[allow(dead_code)]
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

/// Resolve a commander's passive ability into a cached struct.
///
/// This is called once at game setup to pre-compute the passive effect,
/// avoiding CardDatabase lookups during gameplay.
pub fn resolve_commander_passive(
    commander_id: Option<CardId>,
    card_db: &CardDatabase,
) -> ResolvedCommanderPassive {
    let mut resolved = ResolvedCommanderPassive::default();

    if let Some(cmd_id) = commander_id {
        if let Some(commander) = card_db.get_commander(cmd_id) {
            if let CommanderAbility::Passive { passive_ability } = &commander.ability {
                match &passive_ability.effect {
                    CommanderPassiveEffect::GrantKeyword { keyword } => {
                        resolved.grant_keyword = Keywords::parse_keyword_name(keyword);
                    }
                    CommanderPassiveEffect::BuffStats { attack, health } => {
                        resolved.attack_bonus = *attack;
                        resolved.health_bonus = *health;
                    }
                    CommanderPassiveEffect::GrantKeywordAndBuff { keyword, attack, health } => {
                        resolved.grant_keyword = Keywords::parse_keyword_name(keyword);
                        resolved.attack_bonus = *attack;
                        resolved.health_bonus = *health;
                    }
                    CommanderPassiveEffect::BuffStatsIfKeyword { required_keyword, attack, health } => {
                        resolved.required_keyword = Keywords::parse_keyword_name(required_keyword);
                        resolved.attack_bonus = *attack;
                        resolved.health_bonus = *health;
                    }
                    CommanderPassiveEffect::GrantKeywordIfKeyword { required_keyword, granted_keyword } => {
                        resolved.required_keyword = Keywords::parse_keyword_name(required_keyword);
                        resolved.conditional_grant_keyword = Keywords::parse_keyword_name(granted_keyword);
                    }
                }
            }
        }
    }

    resolved
}

/// Apply a cached commander passive to a creature.
///
/// This is the fast path used during gameplay - no CardDatabase lookup required.
#[inline]
pub fn apply_commander_passive_from_cache(
    creature: &mut Creature,
    passive: &ResolvedCommanderPassive,
) {
    if !passive.has_passive() {
        return;
    }

    // Unconditional keyword grant
    if passive.grant_keyword != 0 {
        creature.keywords.add(passive.grant_keyword);
    }

    // Stat buffs (conditional or unconditional based on required_keyword)
    let should_apply_stats = passive.required_keyword == 0
        || creature.keywords.has(passive.required_keyword);

    if should_apply_stats {
        creature.attack = creature.attack.saturating_add(passive.attack_bonus);
        if passive.health_bonus != 0 {
            creature.current_health = creature.current_health.saturating_add(passive.health_bonus);
            creature.max_health = creature.max_health.saturating_add(passive.health_bonus);
        }
    }

    // Conditional keyword grant (only if required_keyword is present)
    if passive.conditional_grant_keyword != 0
        && creature.keywords.has(passive.required_keyword)
    {
        creature.keywords.add(passive.conditional_grant_keyword);
    }
}

// =============================================================================
// COMMANDER TRIGGERED ABILITIES
// =============================================================================

/// Convert a commander's effect definition to an Effect.
///
/// This handles the targeting logic for commander abilities which typically
/// target all friendly or all enemy creatures, or the enemy player.
fn commander_effect_def_to_effect(
    def: &EffectDefinition,
    source_owner: PlayerId,
) -> Option<Effect> {
    match def {
        EffectDefinition::Damage { amount, filter } => {
            // Commander damage effects target the enemy player (commander)
            Some(Effect::Damage {
                target: EffectTarget::Player(source_owner.opponent()),
                amount: *amount,
                filter: filter.clone(),
            })
        }
        EffectDefinition::Heal { amount, filter } => {
            Some(Effect::Heal {
                target: EffectTarget::Player(source_owner),
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
                target: EffectTarget::AllAllyCreatures(source_owner),
                attack: *attack,
                health: *health,
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
        EffectDefinition::GrantKeyword { keyword, filter } => {
            let kw_bits = Keywords::parse_keyword_name(keyword);
            Some(Effect::GrantKeyword {
                target: EffectTarget::AllAllyCreatures(source_owner),
                keyword: kw_bits,
                filter: filter.clone(),
            })
        }
        EffectDefinition::GainEssence { amount } => {
            Some(Effect::GainEssence {
                player: source_owner,
                amount: *amount,
            })
        }
        // Effects that need specific targeting aren't supported for commanders yet
        _ => None,
    }
}

/// Check if a creature's keywords satisfy the commander's trigger condition.
fn check_commander_condition(
    condition: &Option<crate::core::cards::CommanderTriggerCondition>,
    creature_keywords: Keywords,
) -> bool {
    match condition {
        None => true, // No condition means always trigger
        Some(cond) => {
            if let Some(keyword_name) = &cond.has_keyword {
                let required_kw_bits = Keywords::parse_keyword_name(keyword_name);
                creature_keywords.has(required_kw_bits)
            } else {
                true
            }
        }
    }
}

/// Collect effects from a commander's triggered ability.
///
/// Returns a vector of (Effect, EffectSource) pairs to be pushed to the queue.
pub fn collect_commander_trigger_effects(
    commander_id: CardId,
    owner: PlayerId,
    trigger: CommanderTrigger,
    card_db: &CardDatabase,
    condition_check: impl Fn(&Option<crate::core::cards::CommanderTriggerCondition>) -> bool,
) -> Vec<(Effect, EffectSource)> {
    let mut effects = Vec::new();

    if let Some(commander) = card_db.get_commander(commander_id) {
        if let Some(triggered) = commander.triggered_ability() {
            if triggered.trigger == trigger && condition_check(&triggered.condition) {
                let source = EffectSource::Commander { owner };
                for effect_def in &triggered.effects {
                    if let Some(effect) = commander_effect_def_to_effect(effect_def, owner) {
                        effects.push((effect, source));
                    }
                }
            }
        }
    }

    effects
}

/// Collect StartOfTurn trigger effects for a player's commander.
pub fn collect_commander_start_of_turn_effects(
    state: &GameState,
    player: PlayerId,
    card_db: &CardDatabase,
) -> Vec<(Effect, EffectSource)> {
    if let Some(commander_id) = state.get_commander(player) {
        collect_commander_trigger_effects(
            commander_id,
            player,
            CommanderTrigger::StartOfTurn,
            card_db,
            |_| true, // No condition for StartOfTurn
        )
    } else {
        Vec::new()
    }
}

/// Collect OnCreaturePlayed trigger effects for a player's commander.
///
/// The `creature_keywords` parameter is used to check conditions (e.g., Rush for Broodmother).
pub fn collect_commander_creature_played_effects(
    state: &GameState,
    player: PlayerId,
    creature_keywords: Keywords,
    card_db: &CardDatabase,
) -> Vec<(Effect, EffectSource)> {
    if let Some(commander_id) = state.get_commander(player) {
        collect_commander_trigger_effects(
            commander_id,
            player,
            CommanderTrigger::OnCreaturePlayed,
            card_db,
            |condition| check_commander_condition(condition, creature_keywords),
        )
    } else {
        Vec::new()
    }
}

/// Collect OnAllyDeath trigger effects for a player's commander.
pub fn collect_commander_ally_death_effects(
    state: &GameState,
    player: PlayerId,
    card_db: &CardDatabase,
) -> Vec<(Effect, EffectSource)> {
    if let Some(commander_id) = state.get_commander(player) {
        collect_commander_trigger_effects(
            commander_id,
            player,
            CommanderTrigger::OnAllyDeath,
            card_db,
            |_| true, // No condition for OnAllyDeath
        )
    } else {
        Vec::new()
    }
}

/// Collect OnEnemyDeath trigger effects for a player's commander.
///
/// Called when an enemy creature dies - check the opponent's commander.
pub fn collect_commander_enemy_death_effects(
    state: &GameState,
    dying_creature_owner: PlayerId,
    card_db: &CardDatabase,
) -> Vec<(Effect, EffectSource)> {
    // When a creature dies, check the OPPONENT's commander for OnEnemyDeath triggers
    let opponent = dying_creature_owner.opponent();
    if let Some(commander_id) = state.get_commander(opponent) {
        collect_commander_trigger_effects(
            commander_id,
            opponent,
            CommanderTrigger::OnEnemyDeath,
            card_db,
            |_| true, // No condition for OnEnemyDeath
        )
    } else {
        Vec::new()
    }
}

/// Collect OnAttack trigger effects for a player's commander.
///
/// Called when a friendly creature attacks.
pub fn collect_commander_attack_effects(
    state: &GameState,
    attacker_owner: PlayerId,
    card_db: &CardDatabase,
) -> Vec<(Effect, EffectSource)> {
    if let Some(commander_id) = state.get_commander(attacker_owner) {
        collect_commander_trigger_effects(
            commander_id,
            attacker_owner,
            CommanderTrigger::OnAttack,
            card_db,
            |_| true, // No condition for OnAttack
        )
    } else {
        Vec::new()
    }
}

/// Collect OnAnyDeath trigger effects for a player's commander.
///
/// Called when any creature dies (ally or enemy).
pub fn collect_commander_any_death_effects(
    state: &GameState,
    commander_owner: PlayerId,
    card_db: &CardDatabase,
) -> Vec<(Effect, EffectSource)> {
    if let Some(commander_id) = state.get_commander(commander_owner) {
        collect_commander_trigger_effects(
            commander_id,
            commander_owner,
            CommanderTrigger::OnAnyDeath,
            card_db,
            |_| true, // No condition for OnAnyDeath
        )
    } else {
        Vec::new()
    }
}

/// Collect OnKill trigger effects for a player's commander.
///
/// Called when a friendly creature kills an enemy creature.
pub fn collect_commander_kill_effects(
    state: &GameState,
    killer_owner: PlayerId,
    card_db: &CardDatabase,
) -> Vec<(Effect, EffectSource)> {
    if let Some(commander_id) = state.get_commander(killer_owner) {
        collect_commander_trigger_effects(
            commander_id,
            killer_owner,
            CommanderTrigger::OnKill,
            card_db,
            |_| true, // No condition for OnKill
        )
    } else {
        Vec::new()
    }
}
