//! Unit tests for arena logger.

use cardgame::actions::Action;
use cardgame::arena::{
    ActionLogger, ActionRecord, CreatureSnapshot, LoggerCombatTrace, LogOutput, StateSnapshot,
};
use cardgame::combat::CombatResult;
use cardgame::keywords::Keywords;
use cardgame::state::GameState;
use cardgame::types::{CardId, PlayerId, Slot};

#[test]
fn test_logger_memory_output() {
    let mut logger = ActionLogger::new(LogOutput::Memory(Vec::new()), false);

    logger.log_game_start(12345, "Bot1", "Bot2").unwrap();

    let record = ActionRecord {
        turn: 1,
        player: PlayerId::PLAYER_ONE,
        action: Action::EndTurn,
        thinking_time_us: 100,
        state_snapshot: None,
    };
    logger.log_action(&record).unwrap();
    logger.log_game_end(Some(PlayerId::PLAYER_ONE), 10, 15, 0).unwrap();

    if let LogOutput::Memory(lines) = &logger.output {
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("12345"));
        assert!(lines[1].contains("EndTurn"));
        assert!(lines[2].contains("Player 1 wins"));
    }
}

#[test]
fn test_state_snapshot() {
    let mut state = GameState::new();
    state.players[0].life = 25;
    state.players[1].life = 30;
    state.players[0].action_points = 3;

    let snapshot = StateSnapshot::from_state(&state);

    assert_eq!(snapshot.p1_life, 25);
    assert_eq!(snapshot.p2_life, 30);
    assert_eq!(snapshot.p1_ap, 3);
}

#[test]
fn test_creature_snapshot() {
    let snapshot = CreatureSnapshot {
        slot: Slot(2),
        card_id: CardId(5),
        owner: PlayerId::PLAYER_ONE,
        attack: 3,
        health: 4,
        max_health: 5,
        keywords: Keywords::none().with_ranged().with_lethal(),
        exhausted: false,
        silenced: false,
    };

    assert_eq!(snapshot.slot.0, 2);
    assert_eq!(snapshot.attack, 3);
    assert_eq!(snapshot.health, 4);
    assert!(snapshot.keywords.has_ranged());
    assert!(snapshot.keywords.has_lethal());
    assert!(!snapshot.keywords.has_shield());

    // Test display format
    let display = format!("{}", snapshot);
    assert!(display.contains("P1"));
    assert!(display.contains("slot=2"));
    assert!(display.contains("3/4"));
    assert!(display.contains("Ranged"));
    assert!(display.contains("Lethal"));
}

#[test]
fn test_combat_trace_format() {
    let attacker = CreatureSnapshot {
        slot: Slot(1),
        card_id: CardId(10),
        owner: PlayerId::PLAYER_ONE,
        attack: 4,
        health: 3,
        max_health: 3,
        keywords: Keywords::none().with_piercing(),
        exhausted: false,
        silenced: false,
    };

    let defender = CreatureSnapshot {
        slot: Slot(2),
        card_id: CardId(20),
        owner: PlayerId::PLAYER_TWO,
        attack: 2,
        health: 2,
        max_health: 2,
        keywords: Keywords::none(),
        exhausted: false,
        silenced: false,
    };

    let result = CombatResult {
        attacker_damage_dealt: 4,
        defender_damage_dealt: 2,
        attacker_died: false,
        defender_died: true,
        face_damage: 2, // Piercing overflow
        attacker_healed: 0,
    };

    let trace = LoggerCombatTrace::new(5, attacker, Some(defender), Slot(2), result);
    let formatted = trace.format();

    assert!(formatted.contains("COMBAT (Turn 5)"));
    assert!(formatted.contains("Attacker:"));
    assert!(formatted.contains("Defender:"));
    assert!(formatted.contains("Attacker dealt 4 damage"));
    assert!(formatted.contains("(LETHAL)"));
    assert!(formatted.contains("Face damage: 2"));
    assert!(formatted.contains("Defender DIED"));
}

#[test]
fn test_combat_trace_face_attack() {
    let attacker = CreatureSnapshot {
        slot: Slot(0),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        attack: 5,
        health: 3,
        max_health: 3,
        keywords: Keywords::none(),
        exhausted: false,
        silenced: false,
    };

    let result = CombatResult {
        attacker_damage_dealt: 5,
        defender_damage_dealt: 0,
        attacker_died: false,
        defender_died: false,
        face_damage: 5,
        attacker_healed: 0,
    };

    let trace = LoggerCombatTrace::new(3, attacker, None, Slot(2), result);
    let formatted = trace.format();

    assert!(formatted.contains("Face attack"));
    assert!(formatted.contains("slot 2"));
    assert!(formatted.contains("Face damage: 5"));
}

#[test]
fn test_logger_with_combat() {
    let mut logger = ActionLogger::new(LogOutput::Memory(Vec::new()), true);
    logger.log_game_start(12345, "Bot1", "Bot2").unwrap();

    let attacker = CreatureSnapshot {
        slot: Slot(0),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        attack: 2,
        health: 3,
        max_health: 3,
        keywords: Keywords::none(),
        exhausted: false,
        silenced: false,
    };

    let result = CombatResult {
        attacker_damage_dealt: 2,
        defender_damage_dealt: 0,
        attacker_died: false,
        defender_died: false,
        face_damage: 2,
        attacker_healed: 0,
    };

    let trace = LoggerCombatTrace::new(1, attacker, None, Slot(0), result);
    logger.log_combat(&trace).unwrap();

    assert_eq!(logger.combat_count(), 1);

    logger.log_game_end(Some(PlayerId::PLAYER_ONE), 10, 28, 0).unwrap();

    if let LogOutput::Memory(lines) = &logger.output {
        assert!(lines.len() >= 3);
        // Check that game end includes combat count
        let end_line = lines.last().unwrap();
        assert!(end_line.contains("Combats: 1"));
    }
}
