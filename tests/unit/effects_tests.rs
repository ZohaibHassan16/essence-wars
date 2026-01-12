//! Unit tests for effects module.

use cardgame::effects::{
    CreatureFilter, Effect, EffectSource, EffectTarget, PendingEffect, TargetingRule, Trigger,
};
use cardgame::types::{PlayerId, Slot};

#[test]
fn test_effect_target_variants() {
    let target = EffectTarget::Creature {
        owner: PlayerId(0),
        slot: Slot(2),
    };

    match target {
        EffectTarget::Creature { owner, slot } => {
            assert_eq!(owner, PlayerId(0));
            assert_eq!(slot, Slot(2));
        }
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_pending_effect_creation() {
    let effect = Effect::Damage {
        target: EffectTarget::Player(PlayerId(1)),
        amount: 5,
    };
    let source = EffectSource::System;

    let pending = PendingEffect::new(effect.clone(), source);
    assert_eq!(pending.effect, effect);
    assert_eq!(pending.source, source);
}

#[test]
fn test_creature_filter_builder() {
    let filter = CreatureFilter::any()
        .with_max_health(4)
        .without_keyword(0b0000_1000); // Guard

    assert_eq!(filter.max_health, Some(4));
    assert_eq!(filter.lacks_keyword, Some(0b0000_1000));
    assert_eq!(filter.min_health, None);
}

#[test]
fn test_targeting_rule_default() {
    let rule: TargetingRule = Default::default();
    assert_eq!(rule, TargetingRule::NoTarget);
}

#[test]
fn test_trigger_variants() {
    // Just ensure all variants exist and are distinct
    let triggers = [
        Trigger::OnPlay,
        Trigger::OnAttack,
        Trigger::OnDealDamage,
        Trigger::OnTakeDamage,
        Trigger::OnKill,
        Trigger::OnDeath,
        Trigger::StartOfTurn,
        Trigger::EndOfTurn,
        Trigger::OnAllyPlayed,
        Trigger::OnAllyDeath,
    ];

    // Verify all are unique
    for (i, t1) in triggers.iter().enumerate() {
        for (j, t2) in triggers.iter().enumerate() {
            if i != j {
                assert_ne!(t1, t2);
            }
        }
    }
}
