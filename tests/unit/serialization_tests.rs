//! Round-trip serialization tests for game state types.
//!
//! These tests verify that all game state types can be serialized to JSON
//! and deserialized back without loss of information.

use cardgame::state::{
    CardInstance, Creature, CreatureStatus, GameMode, GamePhase, GameResult, GameState,
    PlayerState, Support, WinReason,
};
use cardgame::actions::{Action, Target};
use cardgame::types::{CardId, CreatureInstanceId, PlayerId, Slot};
use cardgame::keywords::Keywords;

/// Helper to test JSON round-trip serialization
fn test_json_roundtrip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
{
    let json = serde_json::to_string(value).expect("Failed to serialize to JSON");
    serde_json::from_str(&json).expect("Failed to deserialize from JSON")
}

/// Helper to test pretty JSON round-trip serialization
fn test_json_pretty_roundtrip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
{
    let json = serde_json::to_string_pretty(value).expect("Failed to serialize to pretty JSON");
    serde_json::from_str(&json).expect("Failed to deserialize from pretty JSON")
}

// =============================================================================
// CORE TYPES
// =============================================================================

#[test]
fn test_card_id_roundtrip() {
    let id = CardId(1234);
    let roundtrip = test_json_roundtrip(&id);
    assert_eq!(id, roundtrip);
}

#[test]
fn test_player_id_roundtrip() {
    let p1 = PlayerId::PLAYER_ONE;
    let p2 = PlayerId::PLAYER_TWO;
    assert_eq!(p1, test_json_roundtrip(&p1));
    assert_eq!(p2, test_json_roundtrip(&p2));
}

#[test]
fn test_slot_roundtrip() {
    for i in 0..5 {
        let slot = Slot(i);
        assert_eq!(slot, test_json_roundtrip(&slot));
    }
}

#[test]
fn test_creature_instance_id_roundtrip() {
    let id = CreatureInstanceId(42);
    let roundtrip = test_json_roundtrip(&id);
    assert_eq!(id, roundtrip);
}

#[test]
fn test_keywords_roundtrip() {
    // Empty keywords
    let empty = Keywords::none();
    assert_eq!(empty, test_json_roundtrip(&empty));

    // Single keyword
    let rush = Keywords::none().with_rush();
    assert_eq!(rush, test_json_roundtrip(&rush));

    // Multiple keywords
    let multi = Keywords::none().with_rush().with_guard().with_lifesteal();
    assert_eq!(multi, test_json_roundtrip(&multi));

    // All keywords
    let all = Keywords::all();
    assert_eq!(all, test_json_roundtrip(&all));
}

// =============================================================================
// ACTIONS
// =============================================================================

#[test]
fn test_target_roundtrip() {
    let targets = vec![
        Target::NoTarget,
        Target::EnemySlot(Slot(0)),
        Target::EnemySlot(Slot(4)),
        Target::Self_,
    ];

    for target in targets {
        assert_eq!(target, test_json_roundtrip(&target));
    }
}

#[test]
fn test_action_play_card_roundtrip() {
    let action = Action::PlayCard {
        hand_index: 3,
        slot: Slot(2),
    };
    assert_eq!(action, test_json_roundtrip(&action));
}

#[test]
fn test_action_attack_roundtrip() {
    let action = Action::Attack {
        attacker: Slot(1),
        defender: Slot(3),
    };
    assert_eq!(action, test_json_roundtrip(&action));
}

#[test]
fn test_action_use_ability_roundtrip() {
    let action = Action::UseAbility {
        slot: Slot(0),
        ability_index: 1,
        target: Target::EnemySlot(Slot(2)),
    };
    assert_eq!(action, test_json_roundtrip(&action));
}

#[test]
fn test_action_end_turn_roundtrip() {
    let action = Action::EndTurn;
    assert_eq!(action, test_json_roundtrip(&action));
}

// =============================================================================
// STATE TYPES
// =============================================================================

#[test]
fn test_creature_status_roundtrip() {
    let status = CreatureStatus(CreatureStatus::EXHAUSTED | CreatureStatus::SILENCED);
    assert_eq!(status, test_json_roundtrip(&status));
}

#[test]
fn test_creature_roundtrip() {
    let creature = Creature {
        instance_id: CreatureInstanceId(1),
        card_id: CardId(1000),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(2),
        attack: 3,
        current_health: 4,
        max_health: 5,
        base_attack: 3,
        base_health: 5,
        keywords: Keywords::none().with_guard().with_rush(),
        status: CreatureStatus(CreatureStatus::EXHAUSTED),
        turn_played: 3,
        frenzy_stacks: 0,
    };
    let roundtrip = test_json_roundtrip(&creature);
    assert_eq!(creature.instance_id, roundtrip.instance_id);
    assert_eq!(creature.card_id, roundtrip.card_id);
    assert_eq!(creature.owner, roundtrip.owner);
    assert_eq!(creature.slot, roundtrip.slot);
    assert_eq!(creature.attack, roundtrip.attack);
    assert_eq!(creature.current_health, roundtrip.current_health);
    assert_eq!(creature.max_health, roundtrip.max_health);
    assert_eq!(creature.keywords, roundtrip.keywords);
    assert_eq!(creature.status, roundtrip.status);
    assert_eq!(creature.turn_played, roundtrip.turn_played);
}

#[test]
fn test_support_roundtrip() {
    let support = Support {
        card_id: CardId(2000),
        owner: PlayerId::PLAYER_TWO,
        slot: Slot(1),
        current_durability: 3,
    };
    let roundtrip = test_json_roundtrip(&support);
    assert_eq!(support.card_id, roundtrip.card_id);
    assert_eq!(support.owner, roundtrip.owner);
    assert_eq!(support.slot, roundtrip.slot);
    assert_eq!(support.current_durability, roundtrip.current_durability);
}

#[test]
fn test_card_instance_roundtrip() {
    let card = CardInstance::new(CardId(1234));
    assert_eq!(card, test_json_roundtrip(&card));
}

#[test]
fn test_game_phase_roundtrip() {
    assert_eq!(GamePhase::Main, test_json_roundtrip(&GamePhase::Main));
    assert_eq!(GamePhase::Ended, test_json_roundtrip(&GamePhase::Ended));
}

#[test]
fn test_win_reason_roundtrip() {
    let reasons = vec![
        WinReason::LifeReachedZero,
        WinReason::TurnLimitHigherLife,
        WinReason::VictoryPointsReached,
        WinReason::Concession,
    ];
    for reason in reasons {
        assert_eq!(reason, test_json_roundtrip(&reason));
    }
}

#[test]
fn test_game_mode_roundtrip() {
    assert_eq!(GameMode::Attrition, test_json_roundtrip(&GameMode::Attrition));
    assert_eq!(GameMode::EssenceDuel, test_json_roundtrip(&GameMode::EssenceDuel));
}

#[test]
fn test_game_result_roundtrip() {
    let win = GameResult::Win {
        winner: PlayerId::PLAYER_ONE,
        reason: WinReason::LifeReachedZero,
    };
    let draw = GameResult::Draw;

    assert_eq!(win, test_json_roundtrip(&win));
    assert_eq!(draw, test_json_roundtrip(&draw));
}

#[test]
fn test_player_state_roundtrip() {
    let mut state = PlayerState::new();
    state.life = 25;
    state.max_essence = 5;
    state.current_essence = 3;
    state.action_points = 2;
    state.hand.push(CardInstance::new(CardId(1000)));
    state.hand.push(CardInstance::new(CardId(1001)));
    state.deck.push(CardInstance::new(CardId(1002)));

    let roundtrip = test_json_roundtrip(&state);
    assert_eq!(state.life, roundtrip.life);
    assert_eq!(state.max_essence, roundtrip.max_essence);
    assert_eq!(state.current_essence, roundtrip.current_essence);
    assert_eq!(state.action_points, roundtrip.action_points);
    assert_eq!(state.hand.len(), roundtrip.hand.len());
    assert_eq!(state.deck.len(), roundtrip.deck.len());
}

#[test]
fn test_game_state_roundtrip() {
    let mut state = GameState::new();
    state.current_turn = 5;
    state.active_player = PlayerId::PLAYER_TWO;
    state.phase = GamePhase::Main;
    state.rng_state = 12345;
    state.game_mode = GameMode::Attrition;

    // Add some cards to hands
    state.players[0].hand.push(CardInstance::new(CardId(1000)));
    state.players[1].hand.push(CardInstance::new(CardId(2000)));

    let roundtrip = test_json_roundtrip(&state);
    assert_eq!(state.current_turn, roundtrip.current_turn);
    assert_eq!(state.active_player, roundtrip.active_player);
    assert_eq!(state.phase, roundtrip.phase);
    assert_eq!(state.rng_state, roundtrip.rng_state);
    assert_eq!(state.game_mode, roundtrip.game_mode);
    assert_eq!(state.players[0].hand.len(), roundtrip.players[0].hand.len());
    assert_eq!(state.players[1].hand.len(), roundtrip.players[1].hand.len());
}

#[test]
fn test_game_state_with_result_roundtrip() {
    let mut state = GameState::new();
    state.result = Some(GameResult::Win {
        winner: PlayerId::PLAYER_ONE,
        reason: WinReason::LifeReachedZero,
    });
    state.phase = GamePhase::Ended;

    let roundtrip = test_json_roundtrip(&state);
    assert_eq!(state.result, roundtrip.result);
    assert_eq!(state.phase, roundtrip.phase);
}

// =============================================================================
// COMPLEX STATE TESTS
// =============================================================================

#[test]
fn test_game_state_with_creatures_roundtrip() {
    let mut state = GameState::new();

    // Add a creature to player 1
    state.players[0].creatures.push(Creature {
        instance_id: CreatureInstanceId(0),
        card_id: CardId(1000),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        attack: 2,
        current_health: 4,
        max_health: 4,
        base_attack: 2,
        base_health: 4,
        keywords: Keywords::none().with_guard(),
        status: CreatureStatus::default(),
        turn_played: 1,
        frenzy_stacks: 0,
    });

    // Add a creature to player 2
    state.players[1].creatures.push(Creature {
        instance_id: CreatureInstanceId(1),
        card_id: CardId(2000),
        owner: PlayerId::PLAYER_TWO,
        slot: Slot(2),
        attack: 3,
        current_health: 2,
        max_health: 3,
        base_attack: 3,
        base_health: 3,
        keywords: Keywords::none().with_rush().with_lethal(),
        status: CreatureStatus(CreatureStatus::EXHAUSTED),
        turn_played: 2,
        frenzy_stacks: 1,
    });

    let roundtrip = test_json_roundtrip(&state);
    assert_eq!(state.players[0].creatures.len(), roundtrip.players[0].creatures.len());
    assert_eq!(state.players[1].creatures.len(), roundtrip.players[1].creatures.len());

    let c1 = &state.players[0].creatures[0];
    let c1_rt = &roundtrip.players[0].creatures[0];
    assert_eq!(c1.instance_id, c1_rt.instance_id);
    assert_eq!(c1.keywords, c1_rt.keywords);

    let c2 = &state.players[1].creatures[0];
    let c2_rt = &roundtrip.players[1].creatures[0];
    assert_eq!(c2.instance_id, c2_rt.instance_id);
    assert_eq!(c2.keywords, c2_rt.keywords);
    assert_eq!(c2.frenzy_stacks, c2_rt.frenzy_stacks);
}

#[test]
fn test_game_state_with_supports_roundtrip() {
    let mut state = GameState::new();

    state.players[0].supports.push(Support {
        card_id: CardId(1013),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        current_durability: 3,
    });

    let roundtrip = test_json_roundtrip(&state);
    assert_eq!(state.players[0].supports.len(), roundtrip.players[0].supports.len());

    let s1 = &state.players[0].supports[0];
    let s1_rt = &roundtrip.players[0].supports[0];
    assert_eq!(s1.card_id, s1_rt.card_id);
    assert_eq!(s1.current_durability, s1_rt.current_durability);
}

// =============================================================================
// PRETTY JSON TESTS (for human-readable output)
// =============================================================================

#[test]
fn test_game_state_pretty_json() {
    let mut state = GameState::new();
    state.current_turn = 3;
    state.players[0].life = 28;
    state.players[1].life = 25;
    state.players[0].hand.push(CardInstance::new(CardId(1000)));

    let roundtrip = test_json_pretty_roundtrip(&state);
    assert_eq!(state.current_turn, roundtrip.current_turn);
    assert_eq!(state.players[0].life, roundtrip.players[0].life);
    assert_eq!(state.players[1].life, roundtrip.players[1].life);
}

#[test]
fn test_action_list_json() {
    let actions = vec![
        Action::PlayCard { hand_index: 0, slot: Slot(2) },
        Action::Attack { attacker: Slot(0), defender: Slot(1) },
        Action::EndTurn,
    ];

    let json = serde_json::to_string_pretty(&actions).unwrap();
    let roundtrip: Vec<Action> = serde_json::from_str(&json).unwrap();
    assert_eq!(actions, roundtrip);
}
