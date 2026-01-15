//! Unit tests for combat resolution.

use cardgame::cards::{CardDatabase, CardDefinition, CardType};
use cardgame::combat::resolve_combat;
use cardgame::engine::EffectQueue;
use cardgame::keywords::Keywords;
use cardgame::state::{Creature, CreatureStatus, GameState};
use cardgame::types::{CardId, CreatureInstanceId, PlayerId, Rarity, Slot};

/// Create a minimal test card database
fn test_card_db() -> CardDatabase {
    let cards = vec![
        // Basic creature (no keywords)
        CardDefinition {
            id: 1,
            name: "Basic Creature".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 3,
                health: 4,
                keywords: vec![],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Creature with Quick
        CardDefinition {
            id: 2,
            name: "Quick Creature".to_string(),
            cost: 3,
            card_type: CardType::Creature {
                attack: 2,
                health: 2,
                keywords: vec!["Quick".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Creature with Shield
        CardDefinition {
            id: 3,
            name: "Shielded Creature".to_string(),
            cost: 3,
            card_type: CardType::Creature {
                attack: 2,
                health: 3,
                keywords: vec!["Shield".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Creature with Ranged
        CardDefinition {
            id: 4,
            name: "Ranged Creature".to_string(),
            cost: 3,
            card_type: CardType::Creature {
                attack: 4,
                health: 2,
                keywords: vec!["Ranged".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Creature with Piercing
        CardDefinition {
            id: 5,
            name: "Piercing Creature".to_string(),
            cost: 3,
            card_type: CardType::Creature {
                attack: 5,
                health: 3,
                keywords: vec!["Piercing".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Creature with Lethal
        CardDefinition {
            id: 6,
            name: "Lethal Creature".to_string(),
            cost: 3,
            card_type: CardType::Creature {
                attack: 1,
                health: 1,
                keywords: vec!["Lethal".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Creature with Lifesteal
        CardDefinition {
            id: 7,
            name: "Lifesteal Creature".to_string(),
            cost: 3,
            card_type: CardType::Creature {
                attack: 4,
                health: 4,
                keywords: vec!["Lifesteal".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Creature with Quick + Lethal
        CardDefinition {
            id: 8,
            name: "Quick Lethal Creature".to_string(),
            cost: 5,
            card_type: CardType::Creature {
                attack: 1,
                health: 1,
                keywords: vec!["Quick".to_string(), "Lethal".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Rare,
            tags: vec![],
        },
    ];
    CardDatabase::new(cards)
}

/// Create a test game state with two empty player boards
fn test_game_state() -> GameState {
    let mut state = GameState::new();
    state.players[0].life = 30;
    state.players[1].life = 30;
    state.current_turn = 2; // Turn 2 so creatures can attack (no summoning sickness)
    state
}

/// Helper to create a creature for testing
fn create_test_creature(
    instance_id: u32,
    card_id: u16,
    owner: PlayerId,
    slot: Slot,
    attack: i8,
    health: i8,
    keywords: Keywords,
) -> Creature {
    Creature {
        instance_id: CreatureInstanceId(instance_id),
        card_id: CardId(card_id),
        owner,
        slot,
        attack,
        current_health: health,
        max_health: health,
        base_attack: attack as u8,
        base_health: health as u8,
        keywords,
        status: CreatureStatus::default(),
        turn_played: 1, // Played last turn, so no summoning sickness
        frenzy_stacks: 0,
    }
}

#[test]
fn test_basic_combat() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 3/4 creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        1,
        PlayerId::PLAYER_ONE,
        Slot(0),
        3,
        4,
        Keywords::none(),
    ));

    // Player 2 has a 2/3 creature in slot 0
    state.players[1].creatures.push(create_test_creature(
        1,
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        3,
        Keywords::none(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Attacker deals 3 damage, defender deals 2 damage
    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 2);

    // Defender dies (3/3 -> 3/0), attacker survives (3/4 -> 3/2)
    assert!(!result.attacker_died);
    assert!(result.defender_died);

    // Verify state
    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 2);
    assert!(state.players[1].get_creature(Slot(0)).is_none()); // Defender removed
}

#[test]
fn test_face_damage() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 3/4 creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        1,
        PlayerId::PLAYER_ONE,
        Slot(0),
        3,
        4,
        Keywords::none(),
    ));

    // No defender in slot 0

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Face damage dealt
    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.face_damage, 3);
    assert_eq!(result.defender_damage_dealt, 0);
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    // Player 2's life reduced
    assert_eq!(state.players[1].life, 27);
}

#[test]
fn test_quick_kills_before_counter() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 2/2 Quick creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        2,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        2,
        Keywords::none().with_quick(),
    ));

    // Player 2 has a 5/2 creature in slot 0 (would kill attacker if it could counter)
    state.players[1].creatures.push(create_test_creature(
        1,
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        5,
        2,
        Keywords::none(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Quick creature kills defender before counter-attack
    assert!(!result.attacker_died);
    assert!(result.defender_died);
    assert_eq!(result.defender_damage_dealt, 0); // No counter-attack

    // Attacker still at full health
    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 2);
}

#[test]
fn test_shield_absorbs_damage() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 3/4 creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        1,
        PlayerId::PLAYER_ONE,
        Slot(0),
        3,
        4,
        Keywords::none(),
    ));

    // Player 2 has a 2/3 shielded creature in slot 0
    state.players[1].creatures.push(create_test_creature(
        1,
        3,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        3,
        Keywords::none().with_shield(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Shield absorbed damage, defender survives
    assert_eq!(result.attacker_damage_dealt, 0); // Blocked by shield
    assert!(!result.defender_died);

    // Defender lost shield but kept health
    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 3);
    assert!(!defender.keywords.has_shield());

    // Attacker took counter-attack damage
    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 2); // 4 - 2 = 2
}

#[test]
fn test_ranged_no_counter_attack() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 4/2 ranged creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        4,
        PlayerId::PLAYER_ONE,
        Slot(0),
        4,
        2,
        Keywords::none().with_ranged(),
    ));

    // Player 2 has a 3/5 creature in slot 0
    state.players[1].creatures.push(create_test_creature(
        1,
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        3,
        5,
        Keywords::none(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Ranged creature deals damage but takes none
    assert_eq!(result.attacker_damage_dealt, 4);
    assert_eq!(result.defender_damage_dealt, 0);
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    // Attacker still at full health
    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 2);

    // Defender took damage
    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 1); // 5 - 4 = 1
}

#[test]
fn test_piercing_overflow() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 5/3 piercing creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        5,
        PlayerId::PLAYER_ONE,
        Slot(0),
        5,
        3,
        Keywords::none().with_piercing(),
    ));

    // Player 2 has a 1/2 creature in slot 0
    state.players[1].creatures.push(create_test_creature(
        1,
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        1,
        2,
        Keywords::none(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // 5 attack - 2 health = 3 overflow damage
    assert_eq!(result.face_damage, 3);
    assert!(result.defender_died);

    // Player 2's life reduced by overflow
    assert_eq!(state.players[1].life, 27); // 30 - 3 = 27
}

#[test]
fn test_lethal_kills_regardless_of_health() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 1/1 lethal creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        6,
        PlayerId::PLAYER_ONE,
        Slot(0),
        1,
        1,
        Keywords::none().with_lethal(),
    ));

    // Player 2 has a 1/10 creature in slot 0 (very high health)
    state.players[1].creatures.push(create_test_creature(
        1,
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        1,
        10,
        Keywords::none(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Lethal killed the high-health defender
    assert!(result.defender_died);
    // Both die (simultaneous damage)
    assert!(result.attacker_died);
}

#[test]
fn test_lifesteal_heals() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Set player 1's life to 20
    state.players[0].life = 20;

    // Player 1 has a 4/4 lifesteal creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        7,
        PlayerId::PLAYER_ONE,
        Slot(0),
        4,
        4,
        Keywords::none().with_lifesteal(),
    ));

    // Player 2 has a 2/3 creature in slot 0
    state.players[1].creatures.push(create_test_creature(
        1,
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        3,
        Keywords::none(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Lifesteal heals for damage dealt (3, the defender's health)
    assert_eq!(result.attacker_healed, 3);

    // Player 1's life increased
    assert_eq!(state.players[0].life, 23); // 20 + 3 = 23
}

#[test]
fn test_quick_plus_lethal_combo() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 1/1 quick+lethal creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        8,
        PlayerId::PLAYER_ONE,
        Slot(0),
        1,
        1,
        Keywords::none().with_quick().with_lethal(),
    ));

    // Player 2 has a 10/10 creature in slot 0
    state.players[1].creatures.push(create_test_creature(
        1,
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        10,
        10,
        Keywords::none(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Quick + Lethal kills the 10/10 before it can counter-attack
    assert!(!result.attacker_died);
    assert!(result.defender_died);
    assert_eq!(result.defender_damage_dealt, 0);

    // Attacker survives at full health
    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 1);
}

#[test]
fn test_shield_vs_piercing() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 5/3 piercing creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        5,
        PlayerId::PLAYER_ONE,
        Slot(0),
        5,
        3,
        Keywords::none().with_piercing(),
    ));

    // Player 2 has a 2/3 shielded creature in slot 0
    state.players[1].creatures.push(create_test_creature(
        1,
        3,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        3,
        Keywords::none().with_shield(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Shield absorbs damage, no piercing overflow
    assert_eq!(result.attacker_damage_dealt, 0);
    assert_eq!(result.face_damage, 0);
    assert!(!result.defender_died);

    // Defender lost shield but took no damage
    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 3);
    assert!(!defender.keywords.has_shield());
}

#[test]
fn test_shield_vs_lethal() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 1/1 lethal creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        6,
        PlayerId::PLAYER_ONE,
        Slot(0),
        1,
        1,
        Keywords::none().with_lethal(),
    ));

    // Player 2 has a 2/3 shielded creature in slot 0
    state.players[1].creatures.push(create_test_creature(
        1,
        3,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        3,
        Keywords::none().with_shield(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Shield absorbs damage, lethal doesn't trigger
    assert_eq!(result.attacker_damage_dealt, 0);
    assert!(!result.defender_died);

    // Defender lost shield but survived
    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 3);
    assert!(!defender.keywords.has_shield());

    // Attacker died from counter-attack
    assert!(result.attacker_died);
}

#[test]
fn test_both_have_shield() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 3/3 shielded creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        3,
        PlayerId::PLAYER_ONE,
        Slot(0),
        3,
        3,
        Keywords::none().with_shield(),
    ));

    // Player 2 has a 3/3 shielded creature in slot 0
    state.players[1].creatures.push(create_test_creature(
        1,
        3,
        PlayerId::PLAYER_TWO,
        Slot(0),
        3,
        3,
        Keywords::none().with_shield(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Both shields absorbed damage
    assert_eq!(result.attacker_damage_dealt, 0);
    assert_eq!(result.defender_damage_dealt, 0);
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    // Both lost shields but survived at full health
    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 3);
    assert!(!attacker.keywords.has_shield());

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 3);
    assert!(!defender.keywords.has_shield());
}

#[test]
fn test_shield_vs_lifesteal() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Set player 1's life to 20
    state.players[0].life = 20;

    // Player 1 has a 4/4 lifesteal creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        7,
        PlayerId::PLAYER_ONE,
        Slot(0),
        4,
        4,
        Keywords::none().with_lifesteal(),
    ));

    // Player 2 has a 2/3 shielded creature in slot 0
    state.players[1].creatures.push(create_test_creature(
        1,
        3,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        3,
        Keywords::none().with_shield(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Shield blocked damage, no lifesteal healing
    assert_eq!(result.attacker_damage_dealt, 0);
    assert_eq!(result.attacker_healed, 0);

    // Player 1's life unchanged (counter-attack damages creature, not player)
    assert_eq!(state.players[0].life, 20);

    // Attacker took counter-attack damage (2 damage from defender)
    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 2); // 4 - 2 = 2
}

#[test]
fn test_lifesteal_face_attack() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Set player 1's life to 20
    state.players[0].life = 20;

    // Player 1 has a 4/4 lifesteal creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        7,
        PlayerId::PLAYER_ONE,
        Slot(0),
        4,
        4,
        Keywords::none().with_lifesteal(),
    ));

    // No defender

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Lifesteal heals for face damage
    assert_eq!(result.attacker_healed, 4);

    // Player 1's life increased
    assert_eq!(state.players[0].life, 24); // 20 + 4 = 24
}

#[test]
fn test_defender_has_quick() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 3/3 creature in slot 0 (no quick)
    state.players[0].creatures.push(create_test_creature(
        0,
        1,
        PlayerId::PLAYER_ONE,
        Slot(0),
        3,
        3,
        Keywords::none(),
    ));

    // Player 2 has a 3/3 quick creature in slot 0
    state.players[1].creatures.push(create_test_creature(
        1,
        2,
        PlayerId::PLAYER_TWO,
        Slot(0),
        3,
        3,
        Keywords::none().with_quick(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Defender has Quick, so they strike first
    // Attacker dies before dealing damage
    assert!(result.attacker_died);
    assert!(!result.defender_died);
    assert_eq!(result.attacker_damage_dealt, 0);
    assert_eq!(result.defender_damage_dealt, 3);

    // Defender survives at full health
    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 3);
}

#[test]
fn test_both_have_quick() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 2/4 quick creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        2,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        4,
        Keywords::none().with_quick(),
    ));

    // Player 2 has a 2/4 quick creature in slot 0
    state.players[1].creatures.push(create_test_creature(
        1,
        2,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
        4,
        Keywords::none().with_quick(),
    ));

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Both have Quick = simultaneous damage
    // Both deal 2 damage, both survive
    assert_eq!(result.attacker_damage_dealt, 2);
    assert_eq!(result.defender_damage_dealt, 2);
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    // Both at 2 health
    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 2);
    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 2);
}

#[test]
fn test_attacker_exhausted_after_combat() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Player 1 has a 3/4 creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        1,
        PlayerId::PLAYER_ONE,
        Slot(0),
        3,
        4,
        Keywords::none(),
    ));

    // No defender (face attack)

    let _result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Attacker should be exhausted
    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(attacker.status.is_exhausted());
}

#[test]
fn test_game_over_on_lethal_face_damage() {
    let card_db = test_card_db();
    let mut state = test_game_state();
    let mut effect_queue = EffectQueue::new();

    // Set player 2's life to 3
    state.players[1].life = 3;

    // Player 1 has a 5/4 creature in slot 0
    state.players[0].creatures.push(create_test_creature(
        0,
        1,
        PlayerId::PLAYER_ONE,
        Slot(0),
        5,
        4,
        Keywords::none(),
    ));

    // No defender

    let _result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    // Game should be over
    assert!(state.is_terminal());
    assert!(matches!(
        state.result,
        Some(cardgame::state::GameResult::Win {
            winner: PlayerId::PLAYER_ONE,
            ..
        })
    ));
}
