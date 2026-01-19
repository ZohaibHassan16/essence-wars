//! Unit tests for state module.

use cardgame::keywords::Keywords;
use cardgame::state::{Creature, CreatureStatus, GameState, PlayerState};
use cardgame::types::{CardId, CreatureInstanceId, PlayerId, Slot};

#[test]
fn test_creature_status() {
    let mut status = CreatureStatus::default();
    assert!(!status.is_exhausted());
    assert!(!status.is_silenced());

    status.set_exhausted(true);
    assert!(status.is_exhausted());
    assert!(!status.is_silenced());

    status.set_silenced(true);
    assert!(status.is_exhausted());
    assert!(status.is_silenced());

    status.set_exhausted(false);
    assert!(!status.is_exhausted());
    assert!(status.is_silenced());
}

#[test]
fn test_player_state_creature_slots() {
    let mut player = PlayerState::new();

    // All slots should be empty initially
    assert!(player.find_empty_creature_slot().is_some());
    assert_eq!(player.find_empty_creature_slot(), Some(Slot(0)));

    // Add a creature to slot 0
    player.creatures.push(Creature {
        instance_id: CreatureInstanceId(0),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        attack: 2,
        current_health: 3,
        max_health: 3,
        base_attack: 2,
        base_health: 3,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 1,
        frenzy_stacks: 0,
    });

    // Next empty slot should be 1
    assert_eq!(player.find_empty_creature_slot(), Some(Slot(1)));
    assert!(player.get_creature(Slot(0)).is_some());
    assert!(player.get_creature(Slot(1)).is_none());
}

#[test]
fn test_game_state_helpers() {
    let mut state = GameState::new();

    assert!(!state.is_terminal());
    assert_eq!(state.active_player, PlayerId::PLAYER_ONE);

    // Test creature ID generation
    let id1 = state.next_creature_instance_id();
    let id2 = state.next_creature_instance_id();
    assert_eq!(id1, CreatureInstanceId(0));
    assert_eq!(id2, CreatureInstanceId(1));
}

#[test]
fn test_creature_can_attack() {
    let mut creature = Creature {
        instance_id: CreatureInstanceId(0),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        attack: 2,
        current_health: 3,
        max_health: 3,
        base_attack: 2,
        base_health: 3,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 1,
        frenzy_stacks: 0,
    };

    // Can attack on turn 2 (no summoning sickness)
    assert!(creature.can_attack(2));

    // Cannot attack on turn 1 (summoning sickness)
    assert!(!creature.can_attack(1));

    // Can attack on turn 1 with Rush
    creature.keywords = Keywords::none().with_rush();
    assert!(creature.can_attack(1));

    // Cannot attack when exhausted
    creature.status.set_exhausted(true);
    assert!(!creature.can_attack(1));
    assert!(!creature.can_attack(2));
}

#[test]
fn test_game_state_size() {
    // GameState should be reasonably small for fast cloning
    let size = std::mem::size_of::<GameState>();
    println!("GameState size: {} bytes", size);
    // Should be under 2KB for efficient MCTS cloning
    assert!(size < 2048, "GameState too large: {} bytes", size);
}

#[test]
fn test_debug_validate_valid_state() {
    // A fresh game state should pass validation
    let state = GameState::new();
    state.debug_validate(); // Should not panic
}

#[test]
fn test_debug_validate_with_creatures() {
    let mut state = GameState::new();

    // Add a valid creature
    state.players[0].creatures.push(Creature {
        instance_id: CreatureInstanceId(0),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        attack: 2,
        current_health: 3,
        max_health: 3,
        base_attack: 2,
        base_health: 3,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 1,
        frenzy_stacks: 0,
    });

    state.debug_validate(); // Should not panic
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "Creature owner")]
fn test_debug_validate_catches_wrong_owner() {
    let mut state = GameState::new();

    // Add a creature with wrong owner (stored in P1's list but owner is P2)
    state.players[0].creatures.push(Creature {
        instance_id: CreatureInstanceId(0),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_TWO, // Wrong owner!
        slot: Slot(0),
        attack: 2,
        current_health: 3,
        max_health: 3,
        base_attack: 2,
        base_health: 3,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 1,
        frenzy_stacks: 0,
    });

    state.debug_validate(); // Should panic
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "invalid health")]
fn test_debug_validate_catches_dead_creature() {
    let mut state = GameState::new();

    // Add a creature with invalid negative health (0 is temporarily allowed for death processing)
    state.players[0].creatures.push(Creature {
        instance_id: CreatureInstanceId(0),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        attack: 2,
        current_health: -1, // Invalid negative health!
        max_health: 3,
        base_attack: 2,
        base_health: 3,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 1,
        frenzy_stacks: 0,
    });

    state.debug_validate(); // Should panic
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "Duplicate creature")]
fn test_debug_validate_catches_duplicate_slot() {
    let mut state = GameState::new();

    // Add two creatures in the same slot
    state.players[0].creatures.push(Creature {
        instance_id: CreatureInstanceId(0),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        attack: 2,
        current_health: 3,
        max_health: 3,
        base_attack: 2,
        base_health: 3,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 1,
        frenzy_stacks: 0,
    });
    state.players[0].creatures.push(Creature {
        instance_id: CreatureInstanceId(1),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0), // Same slot!
        attack: 2,
        current_health: 3,
        max_health: 3,
        base_attack: 2,
        base_health: 3,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 1,
        frenzy_stacks: 0,
    });

    state.debug_validate(); // Should panic
}
