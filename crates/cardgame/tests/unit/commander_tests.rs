//! Unit tests for commander passive abilities.
//!
//! Tests verify that commander passive abilities correctly modify creature
//! stats and keywords when creatures are played.

use cardgame::actions::Action;
use cardgame::cards::CardDatabase;
use cardgame::engine::GameEngine;
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
    // Use minimal decks with just a few creatures
    // Brass Sentinel (1000) - 2/4 Guard creature, costs 2 essence
    let deck1: Vec<CardId> = vec![CardId(1000); 30];
    let deck2: Vec<CardId> = vec![CardId(1000); 30];

    engine.start_game_with_commanders(
        deck1,
        deck2,
        CardId(commander1_id),
        CardId(commander2_id),
        seed,
    );

    // Advance to turn 2 so P1 has 2 essence (enough to play Brass Sentinel)
    // Turn 1: P1 has 1 essence, P1 ends turn
    engine.apply_action(Action::EndTurn).expect("P1 should end turn");
    // Turn 1: P2 has 1 essence (or 2 for FPA compensation), P2 ends turn
    engine.apply_action(Action::EndTurn).expect("P2 should end turn");
    // Now it's turn 2: P1 has 2 essence
}

/// Helper to play a creature at a specific slot
fn play_creature_at_slot(engine: &mut GameEngine, hand_index: u8, slot: Slot) {
    let action = Action::PlayCard { hand_index, slot };
    engine.apply_action(action).expect("Failed to play creature");
}

// =============================================================================
// KEYWORD GRANTING PASSIVES
// =============================================================================

#[test]
fn test_sanctum_healer_grants_regenerate() {
    // The Sanctum Healer (5001): Your creatures have Regenerate
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game with Sanctum Healer as P1's commander
    setup_game_with_commanders(&mut engine, 5001, 5000, 42);

    // Play a creature for P1
    play_creature_at_slot(&mut engine, 0, Slot(0));

    // Check the creature has Regenerate
    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");

    assert!(
        creature.keywords.has_regenerate(),
        "Creature should have Regenerate from Sanctum Healer passive"
    );
}

#[test]
fn test_grand_architect_grants_fortify() {
    // The Grand Architect (5003): Your creatures have Fortify
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5003, 5000, 42);

    play_creature_at_slot(&mut engine, 0, Slot(0));

    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");

    assert!(
        creature.keywords.has_fortify(),
        "Creature should have Fortify from Grand Architect passive"
    );
}

#[test]
fn test_eternal_grove_grants_regenerate() {
    // The Eternal Grove (5007): Your creatures have Regenerate
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5007, 5000, 42);

    play_creature_at_slot(&mut engine, 0, Slot(0));

    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");

    assert!(
        creature.keywords.has_regenerate(),
        "Creature should have Regenerate from Eternal Grove passive"
    );
}

#[test]
fn test_blood_sovereign_grants_lifesteal() {
    // The Blood Sovereign (5008): Your creatures have Lifesteal
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5008, 5000, 42);

    play_creature_at_slot(&mut engine, 0, Slot(0));

    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");

    assert!(
        creature.keywords.has_lifesteal(),
        "Creature should have Lifesteal from Blood Sovereign passive"
    );
}

#[test]
fn test_shadow_weaver_grants_stealth() {
    // The Shadow Weaver (5010): Your creatures have Stealth
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5010, 5000, 42);

    play_creature_at_slot(&mut engine, 0, Slot(0));

    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");

    assert!(
        creature.keywords.has_stealth(),
        "Creature should have Stealth from Shadow Weaver passive"
    );
}

#[test]
fn test_void_archon_grants_quick() {
    // Void Archon (5011): Your creatures have Quick
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5011, 5000, 42);

    play_creature_at_slot(&mut engine, 0, Slot(0));

    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");

    assert!(
        creature.keywords.has_quick(),
        "Creature should have Quick from Void Archon passive"
    );
}

// =============================================================================
// STAT BUFF PASSIVES
// =============================================================================

#[test]
fn test_siege_marshal_vex_grants_attack_bonus() {
    // Siege Marshal Vex (5002): Your creatures have +1 Attack
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5002, 5000, 42);

    // Get base attack of Brass Sentinel (should be 2)
    let brass_sentinel = card_db.get(CardId(1000)).expect("Card should exist");
    let base_attack = brass_sentinel.attack().expect("Should have attack");

    play_creature_at_slot(&mut engine, 0, Slot(0));

    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");

    assert_eq!(
        creature.attack,
        base_attack as i8 + 1,
        "Creature should have +1 Attack from Siege Marshal Vex passive (base {} + 1 = {})",
        base_attack,
        base_attack + 1
    );
}

#[test]
fn test_alpha_of_the_hunt_grants_attack_bonus() {
    // Alpha of the Hunt (5006): Your creatures have +1 Attack
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5006, 5000, 42);

    let brass_sentinel = card_db.get(CardId(1000)).expect("Card should exist");
    let base_attack = brass_sentinel.attack().expect("Should have attack");

    play_creature_at_slot(&mut engine, 0, Slot(0));

    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");

    assert_eq!(
        creature.attack,
        base_attack as i8 + 1,
        "Creature should have +1 Attack from Alpha of the Hunt passive"
    );
}

// =============================================================================
// EDGE CASES AND BEHAVIOR
// =============================================================================

#[test]
fn test_commander_passive_does_not_affect_enemy() {
    // Verify that a commander's passive only affects the owning player's creatures
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // P1 has Blood Sovereign (Lifesteal), P2 has High Artificer (triggered, no passive)
    setup_game_with_commanders(&mut engine, 5008, 5000, 42);

    // P1 plays a creature
    play_creature_at_slot(&mut engine, 0, Slot(0));

    // End P1's turn
    engine.apply_action(Action::EndTurn).expect("Should end turn");

    // P2 plays a creature
    play_creature_at_slot(&mut engine, 0, Slot(0));

    // Check P2's creature does NOT have Lifesteal
    let p2_creature = engine.state.players[1]
        .get_creature(Slot(0))
        .expect("P2 creature should exist");

    assert!(
        !p2_creature.keywords.has_lifesteal(),
        "P2's creature should NOT have Lifesteal (Blood Sovereign is P1's commander)"
    );

    // But P1's creature should still have it
    let p1_creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("P1 creature should exist");

    assert!(
        p1_creature.keywords.has_lifesteal(),
        "P1's creature should have Lifesteal from their commander"
    );
}

#[test]
fn test_commander_passive_applies_to_multiple_creatures() {
    // Verify passive applies to all creatures, not just the first
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // P1 has Blood Sovereign (Lifesteal)
    setup_game_with_commanders(&mut engine, 5008, 5000, 42);

    // Play creatures across multiple turns (each Brass Sentinel costs 2 essence)
    // Turn 2: Play first creature
    play_creature_at_slot(&mut engine, 0, Slot(0));
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 3: Play second creature
    play_creature_at_slot(&mut engine, 0, Slot(1));
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 4: Play third creature
    play_creature_at_slot(&mut engine, 0, Slot(2));

    // All three should have Lifesteal
    for slot_num in 0..3 {
        let creature = engine.state.players[0]
            .get_creature(Slot(slot_num))
            .expect(&format!("Creature at slot {} should exist", slot_num));

        assert!(
            creature.keywords.has_lifesteal(),
            "Creature at slot {} should have Lifesteal from commander passive",
            slot_num
        );
    }
}

#[test]
fn test_triggered_commander_has_no_passive_effect() {
    // Verify that commanders with triggered abilities don't grant passive effects
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Both commanders have triggered abilities, not passives
    // High Artificer (5000) and Broodmother (5004)
    setup_game_with_commanders(&mut engine, 5000, 5004, 42);

    // P1 plays a creature
    play_creature_at_slot(&mut engine, 0, Slot(0));

    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");

    // Brass Sentinel has Guard keyword by default
    // Check that no extra keywords were added (beyond what the card has)
    let brass_sentinel = card_db.get(CardId(1000)).expect("Card should exist");
    let expected_keywords = brass_sentinel.keywords();

    assert_eq!(
        creature.keywords.0,
        expected_keywords.0,
        "Creature should only have its card's base keywords when commander has triggered ability"
    );
}

#[test]
fn test_commander_stat_buff_stacks_with_creature_base() {
    // Verify stat buff is additive to creature's base stats
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Siege Marshal Vex (5002) grants +1 Attack
    setup_game_with_commanders(&mut engine, 5002, 5000, 42);

    play_creature_at_slot(&mut engine, 0, Slot(0));

    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");

    // Brass Sentinel is 2/5, so with +1 attack it should be 3/5
    assert_eq!(creature.attack, 3, "Attack should be 2 (base) + 1 (passive) = 3");
    assert_eq!(creature.current_health, 5, "Health should be unchanged at 5");
    assert_eq!(creature.max_health, 5, "Max health should be unchanged at 5");
}

#[test]
fn test_commander_passive_preserved_after_combat() {
    // Verify that commander passives are properly applied and the keywords persist
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // P1 has Void Archon (Quick), P2 has Siege Marshal Vex (+1 Attack)
    setup_game_with_commanders(&mut engine, 5011, 5002, 42);

    // P1 plays a creature
    play_creature_at_slot(&mut engine, 0, Slot(0));

    // End turn so we're not in the same turn
    engine.apply_action(Action::EndTurn).expect("Should end turn");

    // P2 plays a creature
    play_creature_at_slot(&mut engine, 0, Slot(0));

    // End turn back to P1
    engine.apply_action(Action::EndTurn).expect("Should end turn");

    // Both creatures should still have their commander passives
    let p1_creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("P1 creature should exist");

    let p2_creature = engine.state.players[1]
        .get_creature(Slot(0))
        .expect("P2 creature should exist");

    assert!(
        p1_creature.keywords.has_quick(),
        "P1's creature should still have Quick after turn cycle"
    );

    // P2's creature should have +1 attack (3 instead of 2)
    assert_eq!(
        p2_creature.attack, 3,
        "P2's creature should have +1 Attack from commander passive"
    );
}
