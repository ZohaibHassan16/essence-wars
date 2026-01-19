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
        filter: None,
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

// ============================================================================
// CreatureFilter::matches() Tests
// ============================================================================

use cardgame::keywords::Keywords;

#[test]
fn test_filter_any_matches_all() {
    let filter = CreatureFilter::any();

    // Should match any health and keywords
    assert!(filter.matches(1, 0));
    assert!(filter.matches(10, Keywords::RUSH | Keywords::GUARD));
    assert!(filter.matches(100, Keywords::LIFESTEAL));
}

#[test]
fn test_filter_max_health() {
    let filter = CreatureFilter::any().with_max_health(3);

    // Should match creatures with health <= 3
    assert!(filter.matches(1, 0));
    assert!(filter.matches(2, 0));
    assert!(filter.matches(3, 0));

    // Should not match creatures with health > 3
    assert!(!filter.matches(4, 0));
    assert!(!filter.matches(10, 0));
}

#[test]
fn test_filter_min_health() {
    let filter = CreatureFilter::any().with_min_health(5);

    // Should not match creatures with health < 5
    assert!(!filter.matches(1, 0));
    assert!(!filter.matches(4, 0));

    // Should match creatures with health >= 5
    assert!(filter.matches(5, 0));
    assert!(filter.matches(10, 0));
}

#[test]
fn test_filter_has_keyword() {
    let filter = CreatureFilter::any().with_keyword(Keywords::GUARD);

    // Should match creatures with Guard
    assert!(filter.matches(5, Keywords::GUARD));
    assert!(filter.matches(5, Keywords::GUARD | Keywords::RUSH)); // Has Guard among others

    // Should not match creatures without Guard
    assert!(!filter.matches(5, 0));
    assert!(!filter.matches(5, Keywords::RUSH));
    assert!(!filter.matches(5, Keywords::LIFESTEAL | Keywords::SHIELD));
}

#[test]
fn test_filter_lacks_keyword() {
    let filter = CreatureFilter::any().without_keyword(Keywords::SHIELD);

    // Should match creatures without Shield
    assert!(filter.matches(5, 0));
    assert!(filter.matches(5, Keywords::GUARD));
    assert!(filter.matches(5, Keywords::RUSH | Keywords::GUARD));

    // Should not match creatures with Shield
    assert!(!filter.matches(5, Keywords::SHIELD));
    assert!(!filter.matches(5, Keywords::SHIELD | Keywords::GUARD));
}

#[test]
fn test_filter_combined_conditions() {
    // Filter: max health 4, must have Rush, must NOT have Guard
    let filter = CreatureFilter::any()
        .with_max_health(4)
        .with_keyword(Keywords::RUSH)
        .without_keyword(Keywords::GUARD);

    // Matches: health <= 4, has Rush, no Guard
    assert!(filter.matches(3, Keywords::RUSH));
    assert!(filter.matches(4, Keywords::RUSH | Keywords::LETHAL));

    // Fails health check
    assert!(!filter.matches(5, Keywords::RUSH));

    // Fails has_keyword check (no Rush)
    assert!(!filter.matches(3, 0));
    assert!(!filter.matches(3, Keywords::GUARD));

    // Fails lacks_keyword check (has Guard)
    assert!(!filter.matches(3, Keywords::RUSH | Keywords::GUARD));
}

#[test]
fn test_filter_health_range() {
    // Filter: health between 3 and 5 (inclusive)
    let filter = CreatureFilter::any()
        .with_min_health(3)
        .with_max_health(5);

    assert!(!filter.matches(2, 0)); // Too low
    assert!(filter.matches(3, 0));  // Min boundary
    assert!(filter.matches(4, 0));  // Middle
    assert!(filter.matches(5, 0));  // Max boundary
    assert!(!filter.matches(6, 0)); // Too high
}
