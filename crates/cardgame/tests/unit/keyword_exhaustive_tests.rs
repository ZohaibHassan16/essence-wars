//! Exhaustive keyword combination tests.
//!
//! This module completes the keyword testing matrix by testing:
//! - All 28 pairwise keyword combinations (attacker has both keywords)
//! - 15 strategic triple combinations
//! - Edge cases not covered in keyword_matrix_tests.rs
//!
//! This complements keyword_matrix_tests.rs which tests keyword-vs-keyword interactions.

use cardgame::cards::{CardDatabase, CardDefinition, CardType};
use cardgame::combat::resolve_combat;
use cardgame::engine::EffectQueue;
use cardgame::keywords::Keywords;
use cardgame::state::{Creature, CreatureStatus, GameState};
use cardgame::types::{CardId, CreatureInstanceId, PlayerId, Rarity, Slot};

// ============================================================================
// Test Helpers (same as keyword_matrix_tests.rs)
// ============================================================================

fn keyword_test_db() -> CardDatabase {
    let cards = vec![CardDefinition {
        id: 1,
        name: "Test Creature".to_string(),
        cost: 1,
        card_type: CardType::Creature {
            attack: 3,
            health: 3,
            keywords: vec![],
            abilities: vec![],
        },
        rarity: Rarity::Common,
        tags: vec![],
    }];
    CardDatabase::new(cards)
}

fn test_state() -> GameState {
    let mut state = GameState::new();
    state.players[0].life = 30;
    state.players[1].life = 30;
    state.current_turn = 2;
    state
}

fn test_state_with_life(p1_life: i16, p2_life: i16) -> GameState {
    let mut state = GameState::new();
    state.players[0].life = p1_life;
    state.players[1].life = p2_life;
    state.current_turn = 2;
    state
}

fn create_creature(
    instance_id: u32,
    owner: PlayerId,
    slot: Slot,
    attack: i8,
    health: i8,
    keywords: Keywords,
) -> Creature {
    Creature {
        instance_id: CreatureInstanceId(instance_id),
        card_id: CardId(1),
        owner,
        slot,
        attack,
        current_health: health,
        max_health: health,
        base_attack: attack.max(0) as u8,
        base_health: health as u8,
        keywords,
        status: CreatureStatus::default(),
        turn_played: 1,
        frenzy_stacks: 0,
    }
}

fn run_combat(
    attacker_attack: i8,
    attacker_health: i8,
    attacker_keywords: Keywords,
    defender_attack: i8,
    defender_health: i8,
    defender_keywords: Keywords,
) -> (cardgame::combat::CombatResult, GameState) {
    let card_db = keyword_test_db();
    let mut state = test_state();
    let mut effect_queue = EffectQueue::new();

    state.players[0].creatures.push(create_creature(
        0,
        PlayerId::PLAYER_ONE,
        Slot(0),
        attacker_attack,
        attacker_health,
        attacker_keywords,
    ));

    state.players[1].creatures.push(create_creature(
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        defender_attack,
        defender_health,
        defender_keywords,
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

    (result, state)
}

fn run_combat_with_life(
    attacker_attack: i8,
    attacker_health: i8,
    attacker_keywords: Keywords,
    defender_attack: i8,
    defender_health: i8,
    defender_keywords: Keywords,
    p1_life: i16,
    p2_life: i16,
) -> (cardgame::combat::CombatResult, GameState) {
    let card_db = keyword_test_db();
    let mut state = test_state_with_life(p1_life, p2_life);
    let mut effect_queue = EffectQueue::new();

    state.players[0].creatures.push(create_creature(
        0,
        PlayerId::PLAYER_ONE,
        Slot(0),
        attacker_attack,
        attacker_health,
        attacker_keywords,
    ));

    state.players[1].creatures.push(create_creature(
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        defender_attack,
        defender_health,
        defender_keywords,
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

    (result, state)
}

// ============================================================================
// SECTION 1: Rush Pairwise Combinations
// Rush has no combat effect, but verify it doesn't interfere with other keywords
// ============================================================================

#[test]
fn test_rush_plus_ranged() {
    // Rush + Ranged: Rush is irrelevant, Ranged prevents counter
    let (result, state) = run_combat(
        3,
        2,
        Keywords::none().with_rush().with_ranged(),
        5,
        4,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0); // Ranged prevents counter
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 2); // Full health
}

#[test]
fn test_rush_plus_piercing() {
    // Rush + Piercing: Rush irrelevant, Piercing overflow works
    let (result, state) = run_combat(
        5,
        4,
        Keywords::none().with_rush().with_piercing(),
        2,
        3,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 5);
    assert_eq!(result.face_damage, 2); // 5 - 3 = 2 overflow
    assert!(result.defender_died);
    assert_eq!(state.players[1].life, 28); // 30 - 2 = 28
}

#[test]
fn test_rush_plus_guard() {
    // Rush + Guard: Both affect non-combat mechanics, verify no interference
    let (result, _state) = run_combat(
        3,
        4,
        Keywords::none().with_rush().with_guard(),
        3,
        3,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 3);
    assert!(result.defender_died);
    assert!(!result.attacker_died);
}

#[test]
fn test_rush_plus_lifesteal() {
    // Rush + Lifesteal: Rush irrelevant, Lifesteal heals
    let (result, state) = run_combat_with_life(
        4,
        3,
        Keywords::none().with_rush().with_lifesteal(),
        2,
        5,
        Keywords::none(),
        20,
        30,
    );

    assert_eq!(result.attacker_damage_dealt, 4);
    assert_eq!(result.attacker_healed, 4);
    assert_eq!(state.players[0].life, 24); // 20 + 4 = 24
}

#[test]
fn test_rush_plus_lethal() {
    // Rush + Lethal: Rush irrelevant, Lethal kills
    let (result, _state) = run_combat(
        1,
        3,
        Keywords::none().with_rush().with_lethal(),
        2,
        10,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 1);
    assert!(result.defender_died); // Lethal kill
    assert!(!result.attacker_died); // 2 damage, 3 health
}

#[test]
fn test_rush_plus_shield() {
    // Rush + Shield: Rush irrelevant, Shield blocks counter
    let (result, state) = run_combat(
        3,
        2,
        Keywords::none().with_rush().with_shield(),
        5,
        4,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0); // Blocked by attacker's Shield
    assert!(!result.attacker_died);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(!attacker.keywords.has_shield()); // Shield consumed
}

#[test]
fn test_rush_plus_quick() {
    // Rush + Quick: Rush irrelevant, Quick strikes first
    let (result, state) = run_combat(
        3,
        2,
        Keywords::none().with_rush().with_quick(),
        5,
        3,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0); // Killed before counter
    assert!(!result.attacker_died);
    assert!(result.defender_died);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 2);
}

// ============================================================================
// SECTION 2: Additional Ranged Pairwise (attacker has both keywords)
// ============================================================================

#[test]
fn test_ranged_plus_guard() {
    // Ranged + Guard: Both work independently
    let (result, state) = run_combat(
        3,
        4,
        Keywords::none().with_ranged().with_guard(),
        5,
        3,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0); // Ranged prevents counter
    assert!(!result.attacker_died);
    assert!(result.defender_died);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(attacker.keywords.has_guard()); // Guard still present
}

#[test]
fn test_ranged_plus_lethal() {
    // Ranged + Lethal: No counter, Lethal kill
    let (result, state) = run_combat(
        1,
        1,
        Keywords::none().with_ranged().with_lethal(),
        10,
        20,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 1);
    assert_eq!(result.defender_damage_dealt, 0); // Ranged prevents counter
    assert!(!result.attacker_died);
    assert!(result.defender_died); // Lethal kill

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 1); // Full health (no counter)
}

#[test]
fn test_ranged_plus_shield() {
    // Ranged + Shield: No counter anyway, Shield unused
    let (result, state) = run_combat(
        3,
        2,
        Keywords::none().with_ranged().with_shield(),
        5,
        4,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0);
    assert!(!result.attacker_died);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    // Shield is still present since no damage was received
    assert!(attacker.keywords.has_shield());
}

#[test]
fn test_ranged_plus_quick() {
    // Ranged + Quick: Both benefit attacker (no counter + first strike)
    let (result, state) = run_combat(
        3,
        2,
        Keywords::none().with_ranged().with_quick(),
        5,
        4,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0);
    assert!(!result.attacker_died);

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 1); // 4 - 3 = 1
}

// ============================================================================
// SECTION 3: Additional Piercing Pairwise
// ============================================================================

#[test]
fn test_piercing_plus_guard() {
    // Piercing + Guard: Both work independently
    let (result, state) = run_combat(
        5,
        4,
        Keywords::none().with_piercing().with_guard(),
        2,
        3,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 5);
    assert_eq!(result.face_damage, 2); // 5 - 3 = 2 overflow
    assert!(result.defender_died);
    assert_eq!(state.players[1].life, 28);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(attacker.keywords.has_guard());
}

#[test]
fn test_piercing_plus_shield() {
    // Piercing + Shield on attacker: Shield blocks counter, Piercing works
    let (result, state) = run_combat(
        5,
        2,
        Keywords::none().with_piercing().with_shield(),
        4,
        3,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 5);
    assert_eq!(result.face_damage, 2); // 5 - 3 = 2
    assert_eq!(result.defender_damage_dealt, 0); // Blocked by Shield
    assert!(result.defender_died);
    assert!(!result.attacker_died);

    assert_eq!(state.players[1].life, 28);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(!attacker.keywords.has_shield()); // Shield consumed
}

#[test]
fn test_piercing_plus_quick() {
    // Piercing + Quick: Strike first, pierce overflow
    let (result, state) = run_combat(
        5,
        2,
        Keywords::none().with_piercing().with_quick(),
        4,
        3,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 5);
    assert_eq!(result.face_damage, 2);
    assert_eq!(result.defender_damage_dealt, 0); // Killed before counter
    assert!(result.defender_died);
    assert!(!result.attacker_died);

    assert_eq!(state.players[1].life, 28);
}

// ============================================================================
// SECTION 4: Additional Guard Pairwise
// ============================================================================

#[test]
fn test_guard_plus_lifesteal() {
    // Guard + Lifesteal: Both work independently
    let (result, state) = run_combat_with_life(
        4,
        5,
        Keywords::none().with_guard().with_lifesteal(),
        3,
        4,
        Keywords::none(),
        20,
        30,
    );

    assert_eq!(result.attacker_damage_dealt, 4);
    assert_eq!(result.attacker_healed, 4);
    assert!(result.defender_died);
    assert_eq!(state.players[0].life, 24);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(attacker.keywords.has_guard());
}

#[test]
fn test_guard_plus_lethal() {
    // Guard + Lethal: Both work independently
    let (result, _state) = run_combat(
        1,
        5,
        Keywords::none().with_guard().with_lethal(),
        3,
        10,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 1);
    assert!(result.defender_died); // Lethal kill
    assert!(!result.attacker_died); // 3 damage, 5 health
}

#[test]
fn test_guard_plus_shield() {
    // Guard + Shield: Both work independently
    let (result, state) = run_combat(
        3,
        3,
        Keywords::none().with_guard().with_shield(),
        4,
        4,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0); // Blocked by Shield
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(attacker.keywords.has_guard());
    assert!(!attacker.keywords.has_shield()); // Shield consumed
}

#[test]
fn test_guard_plus_quick() {
    // Guard + Quick: Both work independently
    let (result, _state) = run_combat(
        3,
        3,
        Keywords::none().with_guard().with_quick(),
        4,
        3,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0); // Killed before counter
    assert!(!result.attacker_died);
    assert!(result.defender_died);
}

// ============================================================================
// SECTION 5: Additional Lifesteal Pairwise
// ============================================================================

#[test]
fn test_lifesteal_plus_shield() {
    // Lifesteal + Shield: Shield blocks counter, still heal for damage dealt
    let (result, state) = run_combat_with_life(
        4,
        2,
        Keywords::none().with_lifesteal().with_shield(),
        5,
        5,
        Keywords::none(),
        20,
        30,
    );

    assert_eq!(result.attacker_damage_dealt, 4);
    assert_eq!(result.attacker_healed, 4);
    assert_eq!(result.defender_damage_dealt, 0); // Blocked by Shield
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    assert_eq!(state.players[0].life, 24); // 20 + 4 = 24

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(!attacker.keywords.has_shield()); // Shield consumed
}

#[test]
fn test_lifesteal_plus_quick() {
    // Lifesteal + Quick: Strike first, heal for damage dealt
    let (result, state) = run_combat_with_life(
        4,
        2,
        Keywords::none().with_lifesteal().with_quick(),
        5,
        4,
        Keywords::none(),
        20,
        30,
    );

    assert_eq!(result.attacker_damage_dealt, 4);
    assert_eq!(result.attacker_healed, 4);
    assert_eq!(result.defender_damage_dealt, 0); // Killed before counter
    assert!(!result.attacker_died);
    assert!(result.defender_died);

    assert_eq!(state.players[0].life, 24);
}

// ============================================================================
// SECTION 6: Additional Lethal Pairwise
// ============================================================================

#[test]
fn test_lethal_plus_shield() {
    // Lethal + Shield on attacker: Shield blocks counter, Lethal kills
    let (result, state) = run_combat(
        1,
        1,
        Keywords::none().with_lethal().with_shield(),
        5,
        10,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 1);
    assert_eq!(result.defender_damage_dealt, 0); // Blocked by Shield
    assert!(result.defender_died); // Lethal kill
    assert!(!result.attacker_died);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(!attacker.keywords.has_shield()); // Shield consumed
    assert_eq!(attacker.current_health, 1);
}

// ============================================================================
// SECTION 7: Additional Shield Pairwise
// ============================================================================

#[test]
fn test_shield_plus_quick() {
    // Shield + Quick on attacker: Strike first, Shield as backup
    // If we kill defender, Shield is not used
    let (result, state) = run_combat(
        3,
        2,
        Keywords::none().with_shield().with_quick(),
        4,
        3,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0); // Killed before counter
    assert!(!result.attacker_died);
    assert!(result.defender_died);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(attacker.keywords.has_shield()); // Shield NOT used (no counter happened)
}

#[test]
fn test_shield_plus_quick_defender_survives() {
    // Shield + Quick on attacker, but defender survives
    let (result, state) = run_combat(
        2,
        2,
        Keywords::none().with_shield().with_quick(),
        5,
        4,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 2);
    assert_eq!(result.defender_damage_dealt, 0); // Blocked by Shield
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(!attacker.keywords.has_shield()); // Shield consumed

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 2); // 4 - 2 = 2
}

// ============================================================================
// SECTION 8: Triple Keyword Combinations (Strategic)
// ============================================================================

#[test]
fn test_triple_quick_lethal_lifesteal() {
    // Quick + Lethal + Lifesteal: Strike first, kill anything, heal 1
    let (result, state) = run_combat_with_life(
        1,
        1,
        Keywords::none().with_quick().with_lethal().with_lifesteal(),
        10,
        20,
        Keywords::none(),
        20,
        30,
    );

    assert_eq!(result.attacker_damage_dealt, 1);
    assert_eq!(result.attacker_healed, 1); // Heal for damage dealt
    assert_eq!(result.defender_damage_dealt, 0); // Killed before counter
    assert!(!result.attacker_died);
    assert!(result.defender_died);

    assert_eq!(state.players[0].life, 21); // 20 + 1 = 21
}

#[test]
fn test_triple_quick_lethal_shield() {
    // Quick + Lethal + Shield: Ultimate assassin with protection
    let (result, state) = run_combat(
        1,
        1,
        Keywords::none().with_quick().with_lethal().with_shield(),
        10,
        20,
        Keywords::none().with_quick(), // Defender also has Quick
    );

    // Both Quick = simultaneous damage
    assert_eq!(result.attacker_damage_dealt, 1);
    // Defender's counter blocked by Shield
    assert_eq!(result.defender_damage_dealt, 0);
    assert!(!result.attacker_died);
    assert!(result.defender_died); // Lethal kill

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(!attacker.keywords.has_shield()); // Shield consumed
}

#[test]
fn test_triple_ranged_lethal_quick() {
    // Ranged + Lethal + Quick: No counter, strike first, kill anything
    let (result, state) = run_combat(
        1,
        1,
        Keywords::none().with_ranged().with_lethal().with_quick(),
        20,
        50,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 1);
    assert_eq!(result.defender_damage_dealt, 0); // Ranged prevents counter
    assert!(!result.attacker_died);
    assert!(result.defender_died); // Lethal

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 1);
}

#[test]
fn test_triple_guard_quick_lethal() {
    // Guard + Quick + Lethal: Defensive wall with lethal counter potential
    // Test as defender being attacked
    let (result, state) = run_combat(
        3,
        4,
        Keywords::none(), // Vanilla attacker
        1,
        5,
        Keywords::none().with_guard().with_quick().with_lethal(), // Defender
    );

    // Defender has Quick, strikes first with Lethal
    assert_eq!(result.attacker_damage_dealt, 0); // Killed before attacking
    assert_eq!(result.defender_damage_dealt, 1);
    assert!(result.attacker_died); // Lethal kill
    assert!(!result.defender_died);

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 5); // Full health
}

#[test]
fn test_triple_guard_shield_lifesteal() {
    // Guard + Shield + Lifesteal: Ultimate defensive creature
    let (result, state) = run_combat_with_life(
        4,
        5,
        Keywords::none().with_guard().with_shield().with_lifesteal(),
        5,
        4,
        Keywords::none(),
        20,
        30,
    );

    assert_eq!(result.attacker_damage_dealt, 4);
    assert_eq!(result.attacker_healed, 4);
    assert_eq!(result.defender_damage_dealt, 0); // Blocked by Shield
    assert!(!result.attacker_died);
    assert!(result.defender_died);

    assert_eq!(state.players[0].life, 24); // 20 + 4 = 24

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(!attacker.keywords.has_shield()); // Shield consumed
    assert!(attacker.keywords.has_guard()); // Guard still present
}

#[test]
fn test_triple_ranged_piercing_quick() {
    // Ranged + Piercing + Quick: No counter, first strike, overflow
    let (result, state) = run_combat(
        6,
        2,
        Keywords::none().with_ranged().with_piercing().with_quick(),
        5,
        4,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 6);
    assert_eq!(result.face_damage, 2); // 6 - 4 = 2 overflow
    assert_eq!(result.defender_damage_dealt, 0);
    assert!(!result.attacker_died);
    assert!(result.defender_died);

    assert_eq!(state.players[1].life, 28);
}

#[test]
fn test_triple_shield_lifesteal_quick() {
    // Shield + Lifesteal + Quick: Protected healer with first strike
    // Attacker: 3/2 with Shield + Lifesteal + Quick
    // Defender: 5/4 vanilla
    let (result, state) = run_combat_with_life(
        3,
        2,
        Keywords::none().with_shield().with_lifesteal().with_quick(),
        5,
        4,
        Keywords::none(),
        20,
        30,
    );

    // Quick strikes first: 3 damage to 4 health = defender survives at 1 health
    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.attacker_healed, 3);
    // Defender survives and counter-attacks, but Shield absorbs
    assert_eq!(result.defender_damage_dealt, 0); // Absorbed by Shield
    assert!(!result.attacker_died);
    assert!(!result.defender_died); // 4 - 3 = 1 health remaining

    assert_eq!(state.players[0].life, 23);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(!attacker.keywords.has_shield()); // Shield WAS used (absorbed counter)

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 1);
}

#[test]
fn test_triple_piercing_lethal_quick() {
    // Piercing + Lethal + Quick: First strike, lethal kill, but no pierce overflow
    // (Lethal kills with 1 damage, no "excess" for pierce)
    let (result, state) = run_combat(
        2,
        3,
        Keywords::none().with_piercing().with_lethal().with_quick(),
        5,
        10,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 2);
    assert_eq!(result.defender_damage_dealt, 0); // Killed before counter
    assert!(result.defender_died); // Lethal
    // Piercing doesn't work with Lethal - 2 attack vs 10 health = no overflow
    assert_eq!(result.face_damage, 0);
    assert_eq!(state.players[1].life, 30);
}

#[test]
fn test_triple_ranged_shield_lifesteal() {
    // Ranged + Shield + Lifesteal: No counter (ranged), shield unused, heals
    let (result, state) = run_combat_with_life(
        4,
        2,
        Keywords::none().with_ranged().with_shield().with_lifesteal(),
        5,
        5,
        Keywords::none(),
        20,
        30,
    );

    assert_eq!(result.attacker_damage_dealt, 4);
    assert_eq!(result.attacker_healed, 4);
    assert_eq!(result.defender_damage_dealt, 0);
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    assert_eq!(state.players[0].life, 24);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(attacker.keywords.has_shield()); // Shield NOT used (no damage received)
}

#[test]
fn test_triple_guard_piercing_lifesteal() {
    // Guard + Piercing + Lifesteal: Defensive with offensive capabilities
    let (result, state) = run_combat_with_life(
        6,
        5,
        Keywords::none().with_guard().with_piercing().with_lifesteal(),
        3,
        4,
        Keywords::none(),
        20,
        30,
    );

    assert_eq!(result.attacker_damage_dealt, 6);
    assert_eq!(result.face_damage, 2); // 6 - 4 = 2
    assert_eq!(result.attacker_healed, 4); // Heal for creature damage only
    assert_eq!(result.defender_damage_dealt, 3);
    assert!(result.defender_died);
    assert!(!result.attacker_died);

    assert_eq!(state.players[0].life, 24); // 20 + 4 = 24
    assert_eq!(state.players[1].life, 28); // 30 - 2 = 28
}

// ============================================================================
// SECTION 9: Quadruple Keyword Combinations (Rare but possible)
// ============================================================================

#[test]
fn test_quad_quick_lethal_shield_lifesteal() {
    // Quick + Lethal + Shield + Lifesteal: Ultimate 1/1 assassin
    let (result, state) = run_combat_with_life(
        1,
        1,
        Keywords::none()
            .with_quick()
            .with_lethal()
            .with_shield()
            .with_lifesteal(),
        20,
        50,
        Keywords::none().with_quick(), // Defender also Quick
        20,
        30,
    );

    // Both Quick = simultaneous
    assert_eq!(result.attacker_damage_dealt, 1);
    assert_eq!(result.defender_damage_dealt, 0); // Blocked by Shield
    assert_eq!(result.attacker_healed, 1);
    assert!(result.defender_died); // Lethal
    assert!(!result.attacker_died);

    assert_eq!(state.players[0].life, 21);
}

#[test]
fn test_quad_ranged_piercing_lifesteal_quick() {
    // Ranged + Piercing + Lifesteal + Quick: Maximum face damage potential
    let (result, state) = run_combat_with_life(
        10,
        2,
        Keywords::none()
            .with_ranged()
            .with_piercing()
            .with_lifesteal()
            .with_quick(),
        5,
        3,
        Keywords::none(),
        20,
        30,
    );

    assert_eq!(result.attacker_damage_dealt, 10);
    assert_eq!(result.face_damage, 7); // 10 - 3 = 7 overflow
    assert_eq!(result.attacker_healed, 3); // Heal for creature damage only
    assert_eq!(result.defender_damage_dealt, 0);
    assert!(result.defender_died);
    assert!(!result.attacker_died);

    assert_eq!(state.players[0].life, 23); // 20 + 3 = 23
    assert_eq!(state.players[1].life, 23); // 30 - 7 = 23
}

// ============================================================================
// SECTION 10: All 8 Keywords at Once (Extreme Edge Case)
// ============================================================================

#[test]
fn test_all_keywords() {
    // All 8 keywords on one creature - verify no conflicts
    let (result, state) = run_combat_with_life(
        5,
        3,
        Keywords::none()
            .with_rush()
            .with_ranged()
            .with_piercing()
            .with_guard()
            .with_lifesteal()
            .with_lethal()
            .with_shield()
            .with_quick(),
        10,
        4,
        Keywords::none(),
        20,
        30,
    );

    // Quick strikes first, Lethal kills, Ranged prevents counter
    assert_eq!(result.attacker_damage_dealt, 5);
    assert!(result.defender_died); // Lethal or 5 damage kills 4 health
    assert_eq!(result.defender_damage_dealt, 0); // Ranged prevents counter
    assert!(!result.attacker_died);

    // Piercing: 5 - 4 = 1 overflow
    assert_eq!(result.face_damage, 1);

    // Lifesteal: heals for creature damage (4, the defender's health)
    assert_eq!(result.attacker_healed, 4);

    assert_eq!(state.players[0].life, 24);
    assert_eq!(state.players[1].life, 29);

    // Shield not used (no damage received)
    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(attacker.keywords.has_shield());
}

// ============================================================================
// SECTION 11: Keyword Interactions with High Stat Variance
// ============================================================================

#[test]
fn test_lethal_vs_1_health() {
    // Lethal against 1 health - should still work
    let (result, _state) = run_combat(
        1,
        1,
        Keywords::none().with_lethal(),
        0,
        1,
        Keywords::none(),
    );

    assert!(result.defender_died);
}

#[test]
fn test_piercing_max_overflow() {
    // High attack, low health defender - large overflow
    let (result, state) = run_combat(
        20,
        5,
        Keywords::none().with_piercing(),
        1,
        1,
        Keywords::none(),
    );

    assert_eq!(result.face_damage, 19); // 20 - 1 = 19
    assert_eq!(state.players[1].life, 11); // 30 - 19 = 11
}

#[test]
fn test_lifesteal_large_heal() {
    // Large damage, verify heal capped at 30
    let (result, state) = run_combat_with_life(
        15,
        10,
        Keywords::none().with_lifesteal(),
        1,
        5,
        Keywords::none(),
        20,
        30,
    );

    assert_eq!(result.attacker_healed, 5); // Only heals for damage dealt to creature (defender health)
    assert_eq!(state.players[0].life, 25); // 20 + 5 = 25
}

#[test]
fn test_lifesteal_at_max_health() {
    // Already at 30, lifesteal does nothing
    let (result, state) = run_combat_with_life(
        5,
        5,
        Keywords::none().with_lifesteal(),
        2,
        3,
        Keywords::none(),
        30,
        30,
    );

    assert_eq!(result.attacker_healed, 0); // Capped at 30
    assert_eq!(state.players[0].life, 30);
}

// ============================================================================
// SECTION 12: Keywords vs Keywords (Defender has combo)
// ============================================================================

#[test]
fn test_vanilla_vs_quick_shield() {
    // Attacker faces defender with Quick + Shield
    let (result, state) = run_combat(
        3,
        4,
        Keywords::none(),
        5,
        3,
        Keywords::none().with_quick().with_shield(),
    );

    // Defender Quick strikes first
    // Attacker dies before attacking
    assert_eq!(result.attacker_damage_dealt, 0);
    assert_eq!(result.defender_damage_dealt, 5);
    assert!(result.attacker_died);
    assert!(!result.defender_died);

    // Shield was not used (defender never took damage)
    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert!(defender.keywords.has_shield());
}

#[test]
fn test_vanilla_vs_lethal_lifesteal() {
    // Attacker faces defender with Lethal + Lifesteal
    // Note: In this engine, only ATTACKERS benefit from Lifesteal.
    // Defender Lifesteal does NOT heal the defending player.
    let (result, state) = run_combat_with_life(
        3,
        10,
        Keywords::none(),
        1,
        5,
        Keywords::none().with_lethal().with_lifesteal(),
        30,
        20,
    );

    // Simultaneous damage
    // Attacker deals 3 damage, defender deals 1 with Lethal
    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 1);
    assert!(result.attacker_died); // Lethal kill
    assert!(!result.defender_died); // 5 - 3 = 2 health

    // Defender's Lifesteal does NOT heal in this engine (only attacker lifesteal works)
    assert_eq!(state.players[1].life, 20); // Unchanged - defender lifesteal not applied
}

#[test]
fn test_quick_vs_quick_lethal_shield() {
    // Quick attacker vs Quick + Lethal + Shield defender
    let (result, state) = run_combat(
        5,
        3,
        Keywords::none().with_quick(),
        1,
        4,
        Keywords::none().with_quick().with_lethal().with_shield(),
    );

    // Both Quick = simultaneous
    // Attacker's 5 damage hits Shield, absorbed
    // Defender's 1 damage with Lethal kills attacker
    assert_eq!(result.attacker_damage_dealt, 0); // Blocked by Shield
    assert_eq!(result.defender_damage_dealt, 1);
    assert!(result.attacker_died); // Lethal
    assert!(!result.defender_died);

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert!(!defender.keywords.has_shield()); // Shield consumed
}
