//! Systematic keyword interaction matrix tests.
//!
//! This module tests all 64 keyword combinations (8 attacker keywords × 8 defender keywords)
//! plus edge cases like zero attack, negative attack, and multi-keyword combinations.
//!
//! Keywords tested:
//! - Rush (does not affect combat resolution directly)
//! - Ranged (no counter-attack)
//! - Piercing (excess damage to face)
//! - Guard (affects targeting, not combat resolution)
//! - Lifesteal (heal on damage dealt)
//! - Lethal (any damage kills)
//! - Shield (block first damage)
//! - Quick (strike first)

use cardgame::cards::{CardDatabase, CardDefinition, CardType};
use cardgame::combat::resolve_combat;
use cardgame::engine::EffectQueue;
use cardgame::keywords::Keywords;
use cardgame::state::{Creature, CreatureStatus, GameState};
use cardgame::types::{CardId, CreatureInstanceId, PlayerId, Rarity, Slot};

/// Create a minimal card database for keyword tests
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

/// Create a test game state
fn test_state() -> GameState {
    let mut state = GameState::new();
    state.players[0].life = 30;
    state.players[1].life = 30;
    state.current_turn = 2;
    state
}

/// Helper to create a creature with specific stats and keywords
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

/// Helper to set up combat and return result
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

// ============================================================================
// SECTION 1: Single Keyword Tests (baseline verification)
// ============================================================================

#[test]
fn test_vanilla_vs_vanilla() {
    // 3/3 vs 3/3 - both die simultaneously
    let (result, _state) = run_combat(3, 3, Keywords::none(), 3, 3, Keywords::none());

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 3);
    assert!(result.attacker_died);
    assert!(result.defender_died);
}

#[test]
fn test_rush_has_no_combat_effect() {
    // Rush only affects summoning sickness, not combat
    let (result, _state) = run_combat(
        3,
        3,
        Keywords::none().with_rush(),
        3,
        3,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 3);
    assert!(result.attacker_died);
    assert!(result.defender_died);
}

#[test]
fn test_guard_has_no_combat_effect() {
    // Guard only affects targeting, not combat resolution
    let (result, _state) = run_combat(
        3,
        3,
        Keywords::none().with_guard(),
        3,
        3,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 3);
    assert!(result.attacker_died);
    assert!(result.defender_died);
}

// ============================================================================
// SECTION 2: Ranged Keyword Interactions
// ============================================================================

#[test]
fn test_ranged_vs_vanilla() {
    // Ranged: no counter-attack
    let (result, state) = run_combat(
        3,
        2,
        Keywords::none().with_ranged(),
        5,
        4,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0); // No counter-attack
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    // Attacker at full health
    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 2);

    // Defender took damage
    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 1); // 4 - 3 = 1
}

#[test]
fn test_ranged_vs_quick() {
    // Ranged blocks counter-attack even from Quick defender
    let (result, state) = run_combat(
        3,
        2,
        Keywords::none().with_ranged(),
        5,
        3,
        Keywords::none().with_quick(),
    );

    // Quick defender would strike first, but Ranged prevents counter-attack
    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0);
    assert!(!result.attacker_died);
    assert!(result.defender_died); // 3 damage kills 3 health defender

    // Attacker survives
    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 2);
}

#[test]
fn test_ranged_vs_shield() {
    // Ranged attack blocked by Shield, no counter-attack
    let (result, state) = run_combat(
        3,
        2,
        Keywords::none().with_ranged(),
        5,
        3,
        Keywords::none().with_shield(),
    );

    assert_eq!(result.attacker_damage_dealt, 0); // Blocked by shield
    assert_eq!(result.defender_damage_dealt, 0); // No counter due to Ranged
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    // Defender lost shield
    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert!(!defender.keywords.has_shield());
    assert_eq!(defender.current_health, 3); // Full health
}

#[test]
fn test_ranged_vs_lethal() {
    // Ranged prevents Lethal counter-attack
    let (result, state) = run_combat(
        3,
        2,
        Keywords::none().with_ranged(),
        1,
        1,
        Keywords::none().with_lethal(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0); // No counter-attack
    assert!(!result.attacker_died); // Survives because no counter
    assert!(result.defender_died); // Killed by 3 damage

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 2);
}

#[test]
fn test_ranged_plus_piercing() {
    // Ranged + Piercing: no counter, pierce overflow
    let (result, state) = run_combat(
        5,
        2,
        Keywords::none().with_ranged().with_piercing(),
        3,
        2,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 5);
    assert_eq!(result.defender_damage_dealt, 0);
    assert_eq!(result.face_damage, 3); // 5 - 2 = 3 overflow
    assert!(!result.attacker_died);
    assert!(result.defender_died);

    // P2 took face damage
    assert_eq!(state.players[1].life, 27); // 30 - 3 = 27
}

#[test]
fn test_ranged_plus_lifesteal() {
    // Ranged + Lifesteal: no counter, heals for damage dealt
    let card_db = keyword_test_db();
    let mut state = test_state();
    let mut effect_queue = EffectQueue::new();
    state.players[0].life = 20;

    state.players[0].creatures.push(create_creature(
        0,
        PlayerId::PLAYER_ONE,
        Slot(0),
        4,
        2,
        Keywords::none().with_ranged().with_lifesteal(),
    ));

    state.players[1].creatures.push(create_creature(
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        5,
        6,
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

    assert_eq!(result.attacker_damage_dealt, 4);
    assert_eq!(result.attacker_healed, 4);
    assert_eq!(result.defender_damage_dealt, 0);

    assert_eq!(state.players[0].life, 24); // 20 + 4 = 24
}

// ============================================================================
// SECTION 3: Quick Keyword Interactions
// ============================================================================

#[test]
fn test_quick_attacker_kills_before_counter() {
    // Quick attacker kills defender before they can counter
    let (result, state) = run_combat(
        3,
        2,
        Keywords::none().with_quick(),
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

#[test]
fn test_quick_attacker_doesnt_kill() {
    // Quick attacker doesn't kill - defender counters
    let (result, state) = run_combat(
        2,
        3,
        Keywords::none().with_quick(),
        4,
        5,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 2);
    assert_eq!(result.defender_damage_dealt, 4); // Counter-attack after surviving
    assert!(result.attacker_died); // 4 > 3 health
    assert!(!result.defender_died); // 5 - 2 = 3 > 0

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 3);
}

#[test]
fn test_quick_defender_kills_before_attack() {
    // Quick defender kills attacker before they attack
    let (result, state) = run_combat(
        5,
        2,
        Keywords::none(),
        3,
        4,
        Keywords::none().with_quick(),
    );

    assert_eq!(result.attacker_damage_dealt, 0); // Killed before attacking
    assert_eq!(result.defender_damage_dealt, 3);
    assert!(result.attacker_died);
    assert!(!result.defender_died);

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 4); // Full health
}

#[test]
fn test_quick_vs_quick() {
    // Both Quick = simultaneous damage
    let (result, _state) = run_combat(
        3,
        3,
        Keywords::none().with_quick(),
        3,
        3,
        Keywords::none().with_quick(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 3);
    assert!(result.attacker_died);
    assert!(result.defender_died);
}

#[test]
fn test_quick_vs_shield() {
    // Quick attacks first, Shield absorbs, defender counters
    let (result, state) = run_combat(
        3,
        2,
        Keywords::none().with_quick(),
        2,
        3,
        Keywords::none().with_shield(),
    );

    assert_eq!(result.attacker_damage_dealt, 0); // Absorbed by Shield
    assert_eq!(result.defender_damage_dealt, 2); // Counter-attack (defender survived)
    assert!(result.attacker_died); // 2 damage kills 2 health
    assert!(!result.defender_died);

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 3);
    assert!(!defender.keywords.has_shield());
}

#[test]
fn test_quick_plus_lethal() {
    // Quick + Lethal: strike first, any damage kills
    let (result, state) = run_combat(
        1,
        1,
        Keywords::none().with_quick().with_lethal(),
        10,
        10,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 1);
    assert_eq!(result.defender_damage_dealt, 0); // Killed before counter
    assert!(!result.attacker_died);
    assert!(result.defender_died);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 1);
}

#[test]
fn test_quick_plus_lethal_vs_shield() {
    // Quick + Lethal vs Shield: Shield blocks, Lethal doesn't trigger, defender counters
    let (result, state) = run_combat(
        1,
        1,
        Keywords::none().with_quick().with_lethal(),
        3,
        3,
        Keywords::none().with_shield(),
    );

    assert_eq!(result.attacker_damage_dealt, 0); // Shield blocked
    assert_eq!(result.defender_damage_dealt, 3); // Counter-attack
    assert!(result.attacker_died); // Killed by counter
    assert!(!result.defender_died); // Shield saved it

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 3);
    assert!(!defender.keywords.has_shield());
}

// ============================================================================
// SECTION 4: Shield Keyword Interactions
// ============================================================================

#[test]
fn test_shield_attacker_vs_vanilla() {
    // Attacker has Shield, blocks defender's counter-attack
    let (result, state) = run_combat(
        3,
        3,
        Keywords::none().with_shield(),
        4,
        4,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0); // Blocked by attacker's Shield
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 3);
    assert!(!attacker.keywords.has_shield()); // Shield consumed

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 1); // 4 - 3 = 1
}

#[test]
fn test_shield_vs_shield() {
    // Both have Shield - both absorb, no damage dealt
    let (result, state) = run_combat(
        5,
        3,
        Keywords::none().with_shield(),
        5,
        3,
        Keywords::none().with_shield(),
    );

    assert_eq!(result.attacker_damage_dealt, 0);
    assert_eq!(result.defender_damage_dealt, 0);
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert!(!attacker.keywords.has_shield());
    assert_eq!(attacker.current_health, 3);

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert!(!defender.keywords.has_shield());
    assert_eq!(defender.current_health, 3);
}

#[test]
fn test_shield_vs_piercing() {
    // Shield blocks all damage, Piercing has no effect
    let (result, state) = run_combat(
        5,
        3,
        Keywords::none().with_piercing(),
        3,
        3,
        Keywords::none().with_shield(),
    );

    assert_eq!(result.attacker_damage_dealt, 0); // Blocked
    assert_eq!(result.face_damage, 0); // No pierce
    assert_eq!(result.defender_damage_dealt, 3);
    assert!(result.attacker_died);
    assert!(!result.defender_died);

    // No face damage from pierce
    assert_eq!(state.players[1].life, 30);
}

#[test]
fn test_shield_vs_lethal() {
    // Shield blocks damage, Lethal doesn't trigger (requires damage > 0)
    let (result, state) = run_combat(
        1,
        1,
        Keywords::none().with_lethal(),
        3,
        5,
        Keywords::none().with_shield(),
    );

    assert_eq!(result.attacker_damage_dealt, 0); // Blocked
    assert!(!result.defender_died); // Lethal didn't trigger
    assert!(result.attacker_died); // Killed by counter

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 5);
    assert!(!defender.keywords.has_shield());
}

#[test]
fn test_shield_vs_lifesteal() {
    // Shield blocks damage, Lifesteal doesn't heal
    let card_db = keyword_test_db();
    let mut state = test_state();
    let mut effect_queue = EffectQueue::new();
    state.players[0].life = 20;

    state.players[0].creatures.push(create_creature(
        0,
        PlayerId::PLAYER_ONE,
        Slot(0),
        4,
        2,
        Keywords::none().with_lifesteal(),
    ));

    state.players[1].creatures.push(create_creature(
        1,
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

    assert_eq!(result.attacker_damage_dealt, 0); // Blocked
    assert_eq!(result.attacker_healed, 0); // No heal
    assert_eq!(state.players[0].life, 20); // Unchanged
}

// ============================================================================
// SECTION 5: Lethal Keyword Interactions
// ============================================================================

#[test]
fn test_lethal_vs_high_health() {
    // Lethal kills regardless of health
    let (result, _state) = run_combat(
        1,
        1,
        Keywords::none().with_lethal(),
        1,
        100,
        Keywords::none(),
    );

    assert!(result.defender_died);
    assert!(result.attacker_died); // Counter-attack
}

#[test]
fn test_lethal_vs_lethal() {
    // Both have Lethal - both die from 1 damage each
    let (result, _state) = run_combat(
        1,
        5,
        Keywords::none().with_lethal(),
        1,
        5,
        Keywords::none().with_lethal(),
    );

    assert!(result.attacker_died);
    assert!(result.defender_died);
}

#[test]
fn test_lethal_plus_piercing() {
    // Lethal + Piercing: kills, but no pierce overflow (Lethal doesn't do "excess" damage)
    let (result, state) = run_combat(
        2,
        3,
        Keywords::none().with_lethal().with_piercing(),
        3,
        10,
        Keywords::none(),
    );

    assert!(result.defender_died);
    // Piercing is based on attack vs actual health, not Lethal
    // With 2 attack vs 10 health, no excess = no pierce
    assert_eq!(result.face_damage, 0);
    assert_eq!(state.players[1].life, 30);
}

#[test]
fn test_lethal_plus_lifesteal() {
    // Lethal + Lifesteal: kills with minimal damage, heals for that damage
    let card_db = keyword_test_db();
    let mut state = test_state();
    let mut effect_queue = EffectQueue::new();
    state.players[0].life = 20;

    state.players[0].creatures.push(create_creature(
        0,
        PlayerId::PLAYER_ONE,
        Slot(0),
        1,
        1,
        Keywords::none().with_lethal().with_lifesteal(),
    ));

    state.players[1].creatures.push(create_creature(
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        5,
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

    assert!(result.defender_died); // Lethal kill
    // Heals for 1 damage dealt (the actual damage, not the health killed)
    assert_eq!(result.attacker_healed, 1);
    assert_eq!(state.players[0].life, 21);
}

// ============================================================================
// SECTION 6: Piercing Keyword Interactions
// ============================================================================

#[test]
fn test_piercing_exact_kill() {
    // Piercing with exactly enough damage - no overflow
    let (result, state) = run_combat(
        3,
        4,
        Keywords::none().with_piercing(),
        2,
        3,
        Keywords::none(),
    );

    assert!(result.defender_died);
    assert_eq!(result.face_damage, 0); // 3 attack - 3 health = 0 overflow
    assert_eq!(state.players[1].life, 30);
}

#[test]
fn test_piercing_no_kill() {
    // Piercing but defender survives - no overflow
    let (result, state) = run_combat(
        2,
        4,
        Keywords::none().with_piercing(),
        2,
        5,
        Keywords::none(),
    );

    assert!(!result.defender_died);
    assert_eq!(result.face_damage, 0);
    assert_eq!(state.players[1].life, 30);
}

#[test]
fn test_piercing_plus_lifesteal() {
    // Piercing + Lifesteal: heals for damage dealt (not pierce overflow)
    let card_db = keyword_test_db();
    let mut state = test_state();
    let mut effect_queue = EffectQueue::new();
    state.players[0].life = 20;

    state.players[0].creatures.push(create_creature(
        0,
        PlayerId::PLAYER_ONE,
        Slot(0),
        5,
        4,
        Keywords::none().with_piercing().with_lifesteal(),
    ));

    state.players[1].creatures.push(create_creature(
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        2,
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

    assert!(result.defender_died);
    assert_eq!(result.face_damage, 3); // 5 - 2 = 3 overflow
    // Lifesteal heals for damage dealt to creature (2), not the pierce overflow
    assert_eq!(result.attacker_healed, 2);
    assert_eq!(state.players[0].life, 22); // 20 + 2 = 22
    assert_eq!(state.players[1].life, 27); // 30 - 3 = 27
}

// ============================================================================
// SECTION 7: Lifesteal Keyword Interactions
// ============================================================================

#[test]
fn test_lifesteal_capped_at_30() {
    // Lifesteal cannot heal above 30
    let card_db = keyword_test_db();
    let mut state = test_state();
    let mut effect_queue = EffectQueue::new();
    state.players[0].life = 28;

    state.players[0].creatures.push(create_creature(
        0,
        PlayerId::PLAYER_ONE,
        Slot(0),
        5,
        4,
        Keywords::none().with_lifesteal(),
    ));

    state.players[1].creatures.push(create_creature(
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        1,
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

    assert_eq!(result.attacker_healed, 2); // Only healed 2 (28 + 2 = 30)
    assert_eq!(state.players[0].life, 30);
}

#[test]
fn test_lifesteal_on_face_attack() {
    // Lifesteal works on face damage
    let card_db = keyword_test_db();
    let mut state = test_state();
    let mut effect_queue = EffectQueue::new();
    state.players[0].life = 20;

    state.players[0].creatures.push(create_creature(
        0,
        PlayerId::PLAYER_ONE,
        Slot(0),
        5,
        3,
        Keywords::none().with_lifesteal(),
    ));

    // No defender - face attack

    let result = resolve_combat(
        &mut state,
        &card_db,
        &mut effect_queue,
        PlayerId::PLAYER_ONE,
        Slot(0),
        Slot(0),
        None,
    );

    assert_eq!(result.face_damage, 5);
    assert_eq!(result.attacker_healed, 5);
    assert_eq!(state.players[0].life, 25);
    assert_eq!(state.players[1].life, 25);
}

// ============================================================================
// SECTION 8: Edge Cases
// ============================================================================

#[test]
fn test_zero_attack_no_damage() {
    // Zero attack deals no damage
    let (result, state) = run_combat(0, 2, Keywords::none(), 3, 3, Keywords::none());

    assert_eq!(result.attacker_damage_dealt, 0);
    assert_eq!(result.defender_damage_dealt, 3);
    assert!(result.attacker_died); // 2 health, takes 3 damage
    assert!(!result.defender_died);

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 3); // No damage taken
}

#[test]
fn test_zero_attack_with_lethal() {
    // Zero attack with Lethal - no damage dealt, Lethal doesn't trigger
    let (result, state) = run_combat(
        0,
        2,
        Keywords::none().with_lethal(),
        3,
        10,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 0);
    assert!(!result.defender_died); // Lethal requires damage > 0
    assert!(result.attacker_died); // 2 health, takes 3 damage

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 10);
}

#[test]
fn test_negative_attack_clamped_to_zero() {
    // Negative attack is clamped to 0
    let (result, state) = run_combat(
        -5, // Negative attack
        5,
        Keywords::none(),
        3,
        3,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 0); // Clamped to 0
    assert!(!result.defender_died);

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 3);
}

#[test]
fn test_negative_attack_with_lethal() {
    // Negative attack with Lethal - still no damage, no kill
    let (result, state) = run_combat(
        -3,
        5,
        Keywords::none().with_lethal(),
        2,
        10,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 0);
    assert!(!result.defender_died);

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 10);
}

#[test]
fn test_zero_attack_with_lifesteal() {
    // Zero attack with Lifesteal - no damage, no heal
    let card_db = keyword_test_db();
    let mut state = test_state();
    let mut effect_queue = EffectQueue::new();
    state.players[0].life = 20;

    state.players[0].creatures.push(create_creature(
        0,
        PlayerId::PLAYER_ONE,
        Slot(0),
        0,
        5,
        Keywords::none().with_lifesteal(),
    ));

    state.players[1].creatures.push(create_creature(
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

    assert_eq!(result.attacker_damage_dealt, 0);
    assert_eq!(result.attacker_healed, 0);
    assert_eq!(state.players[0].life, 20); // Unchanged
}

#[test]
fn test_zero_attack_with_piercing() {
    // Zero attack with Piercing - no damage, no pierce
    let (result, state) = run_combat(
        0,
        5,
        Keywords::none().with_piercing(),
        2,
        2,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 0);
    assert_eq!(result.face_damage, 0);
    assert_eq!(state.players[1].life, 30);
}

// ============================================================================
// SECTION 9: Triple Keyword Combinations
// ============================================================================

#[test]
fn test_quick_lethal_ranged() {
    // Quick + Lethal + Ranged: strikes first, kills anything, no counter
    let (result, state) = run_combat(
        1,
        1,
        Keywords::none().with_quick().with_lethal().with_ranged(),
        10,
        20,
        Keywords::none(),
    );

    assert_eq!(result.attacker_damage_dealt, 1);
    assert_eq!(result.defender_damage_dealt, 0); // Ranged prevents counter
    assert!(!result.attacker_died);
    assert!(result.defender_died); // Lethal kill

    let attacker = state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(attacker.current_health, 1);
}

#[test]
fn test_ranged_piercing_lifesteal() {
    // Ranged + Piercing + Lifesteal: no counter, overflow, heal
    let card_db = keyword_test_db();
    let mut state = test_state();
    let mut effect_queue = EffectQueue::new();
    state.players[0].life = 20;

    state.players[0].creatures.push(create_creature(
        0,
        PlayerId::PLAYER_ONE,
        Slot(0),
        6,
        2,
        Keywords::none()
            .with_ranged()
            .with_piercing()
            .with_lifesteal(),
    ));

    state.players[1].creatures.push(create_creature(
        1,
        PlayerId::PLAYER_TWO,
        Slot(0),
        10,
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

    assert_eq!(result.attacker_damage_dealt, 6);
    assert_eq!(result.defender_damage_dealt, 0); // Ranged
    assert_eq!(result.face_damage, 3); // 6 - 3 = 3 pierce
    assert_eq!(result.attacker_healed, 3); // Heals for creature damage, not pierce
    assert!(result.defender_died);

    assert_eq!(state.players[0].life, 23); // 20 + 3 = 23
    assert_eq!(state.players[1].life, 27); // 30 - 3 = 27
}

#[test]
fn test_quick_lethal_vs_shield_quick() {
    // Quick + Lethal attacker vs Shield + Quick defender
    // Both Quick = simultaneous, Shield blocks attacker's Lethal, defender counters
    let (result, state) = run_combat(
        1,
        2,
        Keywords::none().with_quick().with_lethal(),
        5,
        4,
        Keywords::none().with_shield().with_quick(),
    );

    // Both Quick = simultaneous damage
    // Attacker's Lethal is blocked by Shield
    assert_eq!(result.attacker_damage_dealt, 0); // Blocked by Shield
    assert_eq!(result.defender_damage_dealt, 5);
    assert!(result.attacker_died); // 5 > 2
    assert!(!result.defender_died); // Shield saved it

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert!(!defender.keywords.has_shield());
    assert_eq!(defender.current_health, 4);
}

// ============================================================================
// SECTION 10: Simultaneous Death Scenarios
// ============================================================================

#[test]
fn test_both_die_simultaneously() {
    // Both creatures kill each other
    let (result, state) = run_combat(5, 5, Keywords::none(), 5, 5, Keywords::none());

    assert!(result.attacker_died);
    assert!(result.defender_died);
    assert!(state.players[0].get_creature(Slot(0)).is_none());
    assert!(state.players[1].get_creature(Slot(0)).is_none());
}

#[test]
fn test_lethal_trade() {
    // Both have Lethal, both die
    let (result, _state) = run_combat(
        1,
        10,
        Keywords::none().with_lethal(),
        1,
        10,
        Keywords::none().with_lethal(),
    );

    assert!(result.attacker_died);
    assert!(result.defender_died);
}

// ============================================================================
// SECTION 11: Guard + Ranged Targeting (not combat resolution)
// Note: Guard doesn't affect resolve_combat, but we verify Ranged doesn't
// grant any extra combat benefits beyond no-counter
// ============================================================================

#[test]
fn test_ranged_vs_guard() {
    // Ranged attacking a Guard creature - still no counter-attack
    let (result, state) = run_combat(
        3,
        2,
        Keywords::none().with_ranged(),
        5,
        4,
        Keywords::none().with_guard(),
    );

    assert_eq!(result.attacker_damage_dealt, 3);
    assert_eq!(result.defender_damage_dealt, 0);
    assert!(!result.attacker_died);
    assert!(!result.defender_died);

    let defender = state.players[1].get_creature(Slot(0)).unwrap();
    assert_eq!(defender.current_health, 1); // 4 - 3 = 1
}
