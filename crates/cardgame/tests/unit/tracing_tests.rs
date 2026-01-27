//! Unit tests for tracing module.

use cardgame::core::effects::{Effect, EffectSource, EffectTarget};
use cardgame::core::keywords::Keywords;
use cardgame::core::tracing::{
    CombatPhase, CombatStep, CombatTrace, CombatTracer,
    EffectEvent, EffectEventType, EffectTracer,
};
use cardgame::core::types::{PlayerId, Slot};

// ============================================================================
// Combat Tracer Tests
// ============================================================================

#[test]
fn test_combat_trace_creature_combat() {
    let mut trace = CombatTrace::new_creature_combat(
        PlayerId::PLAYER_ONE,
        Slot(2),
        3,
        4,
        Keywords::none().with_quick(),
        PlayerId::PLAYER_TWO,
        Slot(1),
        2,
        3,
        Keywords::none().with_shield(),
    );

    trace.log_quick_check(true, false);
    trace.log_quick_strike("Attacker", 3, false);
    trace.log_shield_check("Defender", true);
    trace.log_shield_absorption("Defender", 3);
    trace.log_complete(false, false);

    // 1 setup + 5 logged steps = 6 total
    assert_eq!(trace.steps.len(), 6);
    assert!(!trace.attacker_died);
    assert!(!trace.defender_died);

    let formatted = trace.format();
    assert!(formatted.contains("COMBAT TRACE"));
    assert!(formatted.contains("Quick"));
}

#[test]
fn test_combat_trace_face_attack() {
    let mut trace = CombatTrace::new_face_attack(
        PlayerId::PLAYER_ONE,
        Slot(0),
        5,
        3,
        Keywords::none().with_lifesteal(),
        Slot(2),
    );

    trace.log_face_damage(5);
    trace.log_lifesteal(5);
    trace.log_complete(false, false);

    assert!(trace.is_face_attack);
    assert_eq!(trace.face_damage, 5);

    let formatted = trace.format();
    assert!(formatted.contains("Face Attack"));
}

#[test]
fn test_combat_tracer() {
    let mut tracer = CombatTracer::new(true);

    let trace1 = CombatTrace::new_face_attack(
        PlayerId::PLAYER_ONE,
        Slot(0),
        3,
        3,
        Keywords::none(),
        Slot(0),
    );
    tracer.add_trace(trace1);

    let trace2 = CombatTrace::new_creature_combat(
        PlayerId::PLAYER_ONE,
        Slot(1),
        2,
        2,
        Keywords::none(),
        PlayerId::PLAYER_TWO,
        Slot(1),
        2,
        2,
        Keywords::none(),
    );
    tracer.add_trace(trace2);

    assert_eq!(tracer.traces().len(), 2);

    let formatted = tracer.format_all();
    assert!(formatted.contains("COMBAT TRACE"));
}

#[test]
fn test_combat_tracer_disabled() {
    let mut tracer = CombatTracer::new(false);

    let trace = CombatTrace::new_face_attack(
        PlayerId::PLAYER_ONE,
        Slot(0),
        3,
        3,
        Keywords::none(),
        Slot(0),
    );
    tracer.add_trace(trace);

    // Should be empty when disabled
    assert!(tracer.traces().is_empty());
}

#[test]
fn test_combat_step_builder() {
    let step = CombatStep::new(CombatPhase::DamageCalculation, "Test damage")
        .with_damage(5)
        .with_keywords(Keywords::none().with_piercing(), Some(Keywords::none().with_shield()))
        .with_death("Defender");

    assert_eq!(step.phase, CombatPhase::DamageCalculation);
    assert_eq!(step.damage_dealt, 5);
    assert!(step.attacker_keywords.has_piercing());
    assert!(step.defender_keywords.unwrap().has_shield());
    assert_eq!(step.death, Some("Defender".to_string()));
}

#[test]
fn test_combat_phase_display() {
    assert_eq!(CombatPhase::Setup.to_string(), "SETUP");
    assert_eq!(CombatPhase::QuickStrike.to_string(), "QUICK_STRIKE");
    assert_eq!(CombatPhase::LethalTrigger.to_string(), "LETHAL_KILL");
    assert_eq!(CombatPhase::Complete.to_string(), "COMPLETE");
}

// ============================================================================
// Effect Tracer Tests
// ============================================================================

#[test]
fn test_effect_tracer_basic() {
    let mut tracer = EffectTracer::new(true);

    tracer.log_queue_start(3);

    let effect = Effect::Damage {
        target: EffectTarget::Creature {
            owner: PlayerId::PLAYER_ONE,
            slot: Slot(0),
        },
        amount: 5,
        filter: None,
    };
    tracer.log_effect_queued(&effect, &EffectSource::System, 3);
    tracer.log_effect_start(&effect, 2);
    tracer.log_effect_complete(&effect, vec!["P1 Slot 0 health: 5 -> 0".to_string()]);

    tracer.log_death(PlayerId::PLAYER_ONE, 0, "Test Creature");
    tracer.log_queue_complete(1);

    assert_eq!(tracer.events.len(), 6);

    let formatted = tracer.format();
    assert!(formatted.contains("EFFECT QUEUE TRACE"));
    assert!(formatted.contains("Damage"));
    assert!(formatted.contains("DEATH"));
}

#[test]
fn test_effect_tracer_cascade() {
    let mut tracer = EffectTracer::new(true);

    tracer.log_queue_start(1);
    tracer.log_death(PlayerId::PLAYER_ONE, 0, "Creature A");
    tracer.log_death_trigger("OnDeath", PlayerId::PLAYER_ONE, 0);

    // Cascade depth should be 1
    assert_eq!(tracer.events.last().unwrap().cascade_depth, 1);

    tracer.log_death(PlayerId::PLAYER_TWO, 0, "Creature B");
    tracer.log_death_trigger("OnDeath", PlayerId::PLAYER_TWO, 0);

    // Cascade depth should be 2
    assert_eq!(tracer.events.last().unwrap().cascade_depth, 2);

    tracer.end_death_trigger();
    tracer.end_death_trigger();

    // After ending both triggers, new events should be at depth 0
    tracer.log_queue_complete(0);
    assert_eq!(tracer.events.last().unwrap().cascade_depth, 0);
}

#[test]
fn test_effect_tracer_disabled() {
    let mut tracer = EffectTracer::new(false);

    tracer.log_queue_start(3);
    tracer.log_death(PlayerId::PLAYER_ONE, 0, "Test");

    // Should be empty when disabled
    assert!(tracer.events.is_empty());
}

#[test]
fn test_effect_event_builder() {
    let event = EffectEvent::new(EffectEventType::Queued, "Test effect")
        .with_queue_depth(5)
        .with_cascade_depth(2)
        .with_state_change("health: 10 -> 5");

    assert_eq!(event.event_type, EffectEventType::Queued);
    assert_eq!(event.queue_depth, 5);
    assert_eq!(event.cascade_depth, 2);
    assert_eq!(event.state_changes.len(), 1);
    assert!(event.state_changes[0].contains("health"));
}

#[test]
fn test_effect_event_type_display() {
    assert_eq!(EffectEventType::Queued.to_string(), "QUEUED");
    assert_eq!(EffectEventType::ExecutionStart.to_string(), "EXEC_START");
    assert_eq!(EffectEventType::DeathDetected.to_string(), "DEATH");
    assert_eq!(EffectEventType::QueueComplete.to_string(), "QUEUE_DONE");
}

#[test]
fn test_effect_tracer_clear() {
    let mut tracer = EffectTracer::new(true);

    tracer.log_queue_start(1);
    tracer.log_death_trigger("OnDeath", PlayerId::PLAYER_ONE, 0);

    assert!(!tracer.events.is_empty());

    tracer.clear();

    assert!(tracer.events.is_empty());
}

#[test]
fn test_effect_tracer_various_effects() {
    let mut tracer = EffectTracer::new(true);
    tracer.log_queue_start(3);

    // Test different effect types
    let heal = Effect::Heal {
        target: EffectTarget::Creature {
            owner: PlayerId::PLAYER_ONE,
            slot: Slot(2),
        },
        amount: 3,
        filter: None,
    };
    tracer.log_effect_queued(&heal, &EffectSource::System, 3);

    let draw = Effect::Draw {
        player: PlayerId::PLAYER_ONE,
        count: 2,
    };
    tracer.log_effect_queued(&draw, &EffectSource::System, 2);

    let buff = Effect::BuffStats {
        target: EffectTarget::AllAllyCreatures(PlayerId::PLAYER_TWO),
        attack: 1,
        health: 1,
        filter: None,
    };
    tracer.log_effect_queued(&buff, &EffectSource::Commander { owner: PlayerId::PLAYER_TWO }, 1);

    tracer.log_queue_complete(3);

    let formatted = tracer.format();
    assert!(formatted.contains("Heal"));
    assert!(formatted.contains("Draw"));
    assert!(formatted.contains("BuffStats"));
    assert!(formatted.contains("Commander"));
}
