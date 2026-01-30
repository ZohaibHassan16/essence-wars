//! Unit tests for token activated abilities (UseAbility action).
//!
//! Tests verify that token abilities like Volatile Overload, Fungal Rot,
//! and Soul Siphon work correctly.

use cardgame::actions::{Action, Target};
use cardgame::cards::CardDatabase;
use cardgame::engine::GameEngine;
use cardgame::state::GameMode;
use cardgame::types::{CardId, Slot};

/// Helper function to load the full card database with commanders
fn load_test_db() -> CardDatabase {
    CardDatabase::load_with_commanders(
        cardgame::data_dir().join("cards/core_set"),
        cardgame::data_dir().join("commanders"),
    )
    .expect("Failed to load cards and commanders")
}

/// Helper to set up a game with specific commanders and advance to turn 2
/// so both players have enough essence to play 2-cost creatures.
fn setup_game_with_commanders(
    engine: &mut GameEngine,
    commander1_id: u16,
    commander2_id: u16,
    seed: u64,
) {
    // Use minimal decks with Brass Sentinel (cost 2)
    let deck1: Vec<CardId> = vec![CardId(1000); 30];
    let deck2: Vec<CardId> = vec![CardId(1000); 30];

    engine.start_game_raw(
        deck1,
        deck2,
        CardId(commander1_id),
        CardId(commander2_id),
        seed,
        GameMode::default(),
    ).unwrap();

    // Advance to turn 2 so P1 has 2 essence (enough for 2-cost cards)
    engine.apply_action(Action::EndTurn).expect("P1 should end turn");
    engine.apply_action(Action::EndTurn).expect("P2 should end turn");
}

// =============================================================================
// FUNGAL ROT TESTS (Plague Sovereign's Sporeling token)
// =============================================================================

#[test]
fn test_fungal_rot_debuffs_enemy_creature() {
    // Plague Sovereign (5005): Summons 1/1 Sporeling with Fungal Rot
    // Fungal Rot: Pay 1 essence, sacrifice: Give target enemy creature -1/-1
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // P1 has Plague Sovereign (spawns Sporelings)
    // P2 has Siege Marshal Vex (+2 attack, no Ward - so debuff can apply)
    setup_game_with_commanders(&mut engine, 5005, 5002, 42);

    // After setup at turn 2, P1 has 2 Sporelings (turn 1 start + turn 2 start)
    let p1_creatures_before = engine.state.players[0].creatures.len();
    assert!(p1_creatures_before >= 1, "P1 should have at least 1 Sporeling");

    // End P1 turn, P2 plays a creature (Brass Sentinel 2/5 Guard -> 4/5 with Vex +2 attack)
    engine.apply_action(Action::EndTurn).expect("P1 end turn");

    // P2 plays a creature
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P2 plays creature");

    // Check P2's creature initial stats (4/5 with Vex attack buff)
    let p2_creature = engine.state.players[1].get_creature(Slot(0)).expect("P2 should have creature");
    let initial_attack = p2_creature.attack;
    let initial_health = p2_creature.current_health;

    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 3: P1 now has 3 essence. Use Fungal Rot on P2's creature.
    // Find the UseAbility action for slot 0 (first Sporeling), targeting enemy slot 0
    let legal = engine.get_legal_actions();

    // UseAbility actions have index >= 75 and < 254
    let use_ability_action = legal.iter().find(|a| {
        matches!(a, Action::UseAbility { slot, target, .. }
            if *slot == Slot(0) && matches!(target, Target::EnemySlot(Slot(0))))
    });

    assert!(use_ability_action.is_some(), "Should have UseAbility action available for Sporeling targeting enemy");

    engine.apply_action(use_ability_action.unwrap().clone()).expect("Fungal Rot should work");

    // Verify: Sporeling died (sacrificed)
    assert!(
        engine.state.players[0].get_creature(Slot(0)).is_none(),
        "Sporeling should have been sacrificed"
    );

    // Verify: P2's creature has -1/-1
    let p2_creature = engine.state.players[1].get_creature(Slot(0)).expect("P2's creature should still exist");
    assert_eq!(
        p2_creature.attack, initial_attack - 1,
        "P2's creature should have -1 attack from Fungal Rot"
    );
    assert_eq!(
        p2_creature.current_health, initial_health - 1,
        "P2's creature should have -1 health from Fungal Rot"
    );
}

#[test]
fn test_fungal_rot_can_kill_creature_at_1_health() {
    // Verify that debuff can kill a creature if it brings health to 0
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // P1 has Plague Sovereign (spawns Sporelings)
    // P2 has a passive commander

    // Use Eager Sellsword (4031) - 1 cost, 2/1 Rush for P2
    let deck1: Vec<CardId> = vec![CardId(1000); 30];
    let deck2: Vec<CardId> = vec![CardId(4031); 30]; // 2/1 creatures

    engine.start_game_raw(
        deck1,
        deck2,
        CardId(5005), // Plague Sovereign
        CardId(5002), // Siege Marshal Vex (+2 attack, no health buff)
        42,
        GameMode::default(),
    ).unwrap();

    // End P1 turn
    engine.apply_action(Action::EndTurn).expect("P1 end turn");

    // P2 plays Eager Sellsword (2/1 -> 4/1 with Vex +2 attack)
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P2 plays creature");

    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 3: P1 uses Fungal Rot on P2's 1-health creature
    let legal = engine.get_legal_actions();
    let use_ability_action = legal.iter().find(|a| {
        matches!(a, Action::UseAbility { slot, target, .. }
            if *slot == Slot(0) && matches!(target, Target::EnemySlot(Slot(0))))
    });

    assert!(use_ability_action.is_some(), "Should have UseAbility action");

    engine.apply_action(use_ability_action.unwrap().clone()).expect("Fungal Rot should work");

    // Verify: P2's creature died from -1 health debuff (1 - 1 = 0)
    assert!(
        engine.state.players[1].get_creature(Slot(0)).is_none(),
        "P2's creature should have died from Fungal Rot debuff (1 health - 1 = 0)"
    );
}

#[test]
fn test_fungal_rot_requires_essence() {
    // Verify that Fungal Rot requires 1 essence
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5005, 5001, 42);

    // Spend all essence first
    let player = &mut engine.state.players[0];
    player.current_essence = 0;

    // End P1 turn, P2 plays a creature
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P2 plays creature");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 3: P1 has 3 essence, but let's manually set to 0
    engine.state.players[0].current_essence = 0;

    // Check that no UseAbility actions are available (no essence)
    let legal = engine.get_legal_actions();
    let has_use_ability = legal.iter().any(|a| matches!(a, Action::UseAbility { .. }));

    assert!(
        !has_use_ability,
        "Should not have UseAbility actions when essence is 0"
    );
}

// =============================================================================
// SOUL SIPHON TESTS (Shadow Weaver's Shade token)
// =============================================================================

#[test]
fn test_soul_siphon_damages_and_heals() {
    // Shadow Weaver (5010): Summons 1/1 Shade with Stealth and Soul Siphon
    // Soul Siphon: Pay 1 essence, sacrifice: Deal 1 damage to enemy commander, heal your commander for 2
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5010, 5001, 42);

    // Record initial life totals
    let p2_life_before = engine.state.players[1].life;

    // Damage P1 a bit so heal has room to work
    engine.state.players[0].life = 25;
    let p1_life_before_ability = engine.state.players[0].life;

    // After setup at turn 2, P1 has Shades
    let shade_count = engine.state.players[0].creatures.len();
    assert!(shade_count >= 1, "P1 should have at least 1 Shade");

    // Verify Shade has Stealth
    let shade = engine.state.players[0].get_creature(Slot(0)).expect("Shade should exist");
    assert!(shade.keywords.has_stealth(), "Shade should have Stealth");

    // Use Soul Siphon (NoTarget ability - targets enemy commander automatically)
    let legal = engine.get_legal_actions();

    // Find UseAbility with NoTarget
    let use_ability_action = legal.iter().find(|a| {
        matches!(a, Action::UseAbility { slot, target, .. }
            if *slot == Slot(0) && matches!(target, Target::NoTarget))
    });

    assert!(use_ability_action.is_some(), "Should have UseAbility action for Soul Siphon (NoTarget)");

    engine.apply_action(use_ability_action.unwrap().clone()).expect("Soul Siphon should work");

    // Verify: Shade died (sacrificed)
    assert!(
        engine.state.players[0].get_creature(Slot(0)).is_none(),
        "Shade should have been sacrificed"
    );

    // Verify: P2 took 1 damage
    assert_eq!(
        engine.state.players[1].life,
        p2_life_before - 1,
        "P2 should have taken 1 damage from Soul Siphon"
    );

    // Verify: P1 healed for 2
    assert_eq!(
        engine.state.players[0].life,
        p1_life_before_ability + 2,
        "P1 should have healed for 2 from Soul Siphon"
    );
}

#[test]
fn test_soul_siphon_heal_caps_at_30() {
    // Verify heal doesn't exceed max life
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5010, 5001, 42);

    // Set P1 to 29 life (heal 2 would go to 31, but should cap at 30)
    engine.state.players[0].life = 29;

    // Use Soul Siphon
    let legal = engine.get_legal_actions();
    let use_ability_action = legal.iter().find(|a| {
        matches!(a, Action::UseAbility { slot, target, .. }
            if *slot == Slot(0) && matches!(target, Target::NoTarget))
    });

    assert!(use_ability_action.is_some(), "Should have UseAbility action");

    engine.apply_action(use_ability_action.unwrap().clone()).expect("Soul Siphon should work");

    // Verify: P1 healed but capped at 30
    assert_eq!(
        engine.state.players[0].life,
        30,
        "P1's life should be capped at 30 (was 29, healed 2, cap at 30)"
    );
}

#[test]
fn test_shade_has_stealth() {
    // Verify Shade token has Stealth keyword
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5010, 5001, 42);

    // Check that Shade has Stealth
    let shade = engine.state.players[0].get_creature(Slot(0)).expect("Shade should exist");
    assert!(
        shade.keywords.has_stealth(),
        "Shade should have Stealth keyword"
    );
}

// =============================================================================
// VOLATILE OVERLOAD TESTS (High Artificer's Brass Cog token)
// =============================================================================

#[test]
fn test_volatile_overload_damages_enemy_creature() {
    // High Artificer (5000): Summons 2/2 Brass Cog with Volatile Overload
    // Volatile Overload: Pay 1 essence, sacrifice: Deal 2 damage to any enemy
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Use Siege Marshal Vex (5002) - no Ward, so damage can apply
    setup_game_with_commanders(&mut engine, 5000, 5002, 42);

    // End P1 turn, P2 plays a creature
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P2 plays creature");

    // Get P2's creature health (Brass Sentinel 2/5 -> 4/5 with Vex +2 attack)
    let p2_creature = engine.state.players[1].get_creature(Slot(0)).expect("P2 should have creature");
    let initial_health = p2_creature.current_health;

    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 3: Use Volatile Overload targeting P2's creature
    let legal = engine.get_legal_actions();
    let use_ability_action = legal.iter().find(|a| {
        matches!(a, Action::UseAbility { slot, target, .. }
            if *slot == Slot(0) && matches!(target, Target::EnemySlot(Slot(0))))
    });

    assert!(use_ability_action.is_some(), "Should have UseAbility for Volatile Overload");

    engine.apply_action(use_ability_action.unwrap().clone()).expect("Volatile Overload should work");

    // Verify: Brass Cog died
    assert!(
        engine.state.players[0].get_creature(Slot(0)).is_none(),
        "Brass Cog should have been sacrificed"
    );

    // Verify: P2's creature took 2 damage
    let p2_creature = engine.state.players[1].get_creature(Slot(0)).expect("P2's creature should still exist");
    assert_eq!(
        p2_creature.current_health,
        initial_health - 2,
        "P2's creature should have taken 2 damage from Volatile Overload"
    );
}

#[test]
fn test_volatile_overload_damages_enemy_commander() {
    // Verify Volatile Overload can target enemy commander (NoTarget = commander)
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5000, 5001, 42);

    let p2_life_before = engine.state.players[1].life;

    // Use Volatile Overload with NoTarget (damages enemy commander)
    let legal = engine.get_legal_actions();
    let use_ability_action = legal.iter().find(|a| {
        matches!(a, Action::UseAbility { slot, target, .. }
            if *slot == Slot(0) && matches!(target, Target::NoTarget))
    });

    assert!(use_ability_action.is_some(), "Should have UseAbility for Volatile Overload (NoTarget)");

    engine.apply_action(use_ability_action.unwrap().clone()).expect("Volatile Overload should work");

    // Verify: P2 took 2 damage
    assert_eq!(
        engine.state.players[1].life,
        p2_life_before - 2,
        "P2 commander should have taken 2 damage from Volatile Overload"
    );
}
