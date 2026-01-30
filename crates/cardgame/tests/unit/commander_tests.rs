//! Unit tests for commander passive abilities.
//!
//! Tests verify that commander passive abilities correctly modify creature
//! stats and keywords when creatures are played.

use cardgame::actions::Action;
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
    // Use minimal decks with just a few creatures
    // Brass Sentinel (1000) - 2/5 Guard creature, costs 2 essence
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
fn test_sanctum_healer_grants_ward_and_health() {
    // The Sanctum Healer (5001): Your creatures have Ward and +0/+2
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game with Sanctum Healer as P1's commander
    setup_game_with_commanders(&mut engine, 5001, 5000, 42);

    // Play a creature for P1 (Brass Sentinel: 2/5 Guard)
    play_creature_at_slot(&mut engine, 0, Slot(0));

    // Check the creature has Ward and +0/+2 health buff
    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");

    // Verify Ward is granted
    assert!(
        creature.keywords.has_ward(),
        "Creature should have Ward from Sanctum Healer passive"
    );

    // Brass Sentinel base stats: 2/5, with +0/+2 should be 2/7
    assert_eq!(
        creature.attack, 2,
        "Creature attack should be 2 (unchanged)"
    );
    assert_eq!(
        creature.current_health, 7,
        "Creature health should be 7 (5 base + 2 from Sanctum Healer)"
    );
    assert_eq!(
        creature.max_health, 7,
        "Creature max health should be 7 (5 base + 2 from Sanctum Healer)"
    );
}

#[test]
fn test_grand_architect_grants_fortify_and_health() {
    // The Grand Architect (5003): Your creatures have Fortify and +0/+2
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

    // Brass Sentinel base health is 5, with +0/+2 should be 7
    assert_eq!(
        creature.current_health, 7,
        "Creature should have +2 health from Grand Architect passive"
    );
}

#[test]
fn test_eternal_grove_buffs_creatures_at_start_of_turn() {
    // The Eternal Grove (5007): At start of turn, give all creatures +1/+1
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5007, 5000, 42);

    // Play a creature (Brass Sentinel: 2/5)
    play_creature_at_slot(&mut engine, 0, Slot(0));

    // Check initial stats (no buff yet - trigger is at START of turn)
    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");
    assert_eq!(creature.attack, 2, "Initial attack should be base 2");
    assert_eq!(creature.current_health, 5, "Initial health should be base 5");

    // End P1's turn, then P2's turn -> back to P1's turn (triggers buff)
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Now at start of P1's turn, creatures should have +1/+1
    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");

    // Brass Sentinel base: 2/5, after one StartOfTurn buff: 3/6
    assert_eq!(
        creature.attack, 3,
        "Creature should have +1 attack from Eternal Grove trigger (base 2 + 1 = 3)"
    );
    assert_eq!(
        creature.current_health, 6,
        "Creature should have +1 health from Eternal Grove trigger (base 5 + 1 = 6)"
    );
}

#[test]
fn test_blood_sovereign_grants_lifesteal_and_health() {
    // The Blood Sovereign (5008): Your creatures have Lifesteal and +0/+1
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

    // Brass Sentinel base health is 5, with +0/+1 should be 6
    assert_eq!(
        creature.current_health, 6,
        "Creature should have +1 health from Blood Sovereign passive"
    );
}

#[test]
fn test_shadow_weaver_spawns_shades_with_stealth() {
    // The Shadow Weaver (5010): At start of turn, summon 1/1 Shade with Stealth
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5010, 5000, 42);

    // After setup at turn 2, P1 should have 2 Shades (turn 1 + turn 2 start)
    let p1_creatures = &engine.state.players[0].creatures;
    assert!(p1_creatures.len() >= 1, "P1 should have at least 1 Shade");

    // The Shade should have Stealth keyword
    let shade = &p1_creatures[0];
    assert!(
        shade.keywords.has_stealth(),
        "Shade token should have Stealth"
    );

    // Verify stats (1/1)
    assert_eq!(shade.attack, 1, "Shade should have 1 attack");
    assert_eq!(shade.current_health, 1, "Shade should have 1 health");
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
    // Siege Marshal Vex (5002): Your creatures have +2 Attack (buffed in v0.8.0)
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
        base_attack as i8 + 2,
        "Creature should have +2 Attack from Siege Marshal Vex passive (base {} + 2 = {})",
        base_attack,
        base_attack + 2
    );
}

#[test]
fn test_alpha_of_the_hunt_grants_attack_bonus() {
    // Alpha of the Hunt (5006): When a creature attacks, give all your creatures +1/+0
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    setup_game_with_commanders(&mut engine, 5006, 5000, 42);

    let brass_sentinel = card_db.get(CardId(1000)).expect("Card should exist");
    let base_attack = brass_sentinel.attack().expect("Should have attack");

    // P1 plays a creature
    play_creature_at_slot(&mut engine, 0, Slot(0));

    // Creature should have base attack (no passive bonus)
    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should exist");
    assert_eq!(
        creature.attack, base_attack as i8,
        "Creature should have base attack before any attack triggers"
    );

    // End P1 turn, P2 plays nothing and ends turn
    engine.apply_action(Action::EndTurn).expect("End turn");
    engine.apply_action(Action::EndTurn).expect("End turn");

    // Now P1's creature can attack (summoning sickness is gone)
    // Attack empty slot (face attack)
    engine
        .apply_action(Action::Attack {
            attacker: Slot(0),
            defender: Slot(0), // Empty slot = face attack
        })
        .expect("Attack should succeed");

    // After the attack, creature should have +1 Attack from OnAttack trigger
    let creature = engine.state.players[0]
        .get_creature(Slot(0))
        .expect("Creature should still exist");
    assert_eq!(
        creature.attack,
        base_attack as i8 + 1,
        "Creature should have +1 Attack after attacking (Alpha of the Hunt OnAttack trigger)"
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

    // P1 has Blood Sovereign (Lifesteal), P2 has Sanctum Healer (Regenerate passive, no tokens)
    setup_game_with_commanders(&mut engine, 5008, 5001, 42);

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

    // High Artificer summoned Brass Cogs at turn 1 start (slot 0) and turn 2 start (slot 1)
    // Play our creature at slot 2 instead
    play_creature_at_slot(&mut engine, 0, Slot(2));

    let creature = engine.state.players[0]
        .get_creature(Slot(2))
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

    // Brass Sentinel is 2/5, so with +2 attack (v0.8.0 Vex buff) it should be 4/5
    assert_eq!(creature.attack, 4, "Attack should be 2 (base) + 2 (passive) = 4");
    assert_eq!(creature.current_health, 5, "Health should be unchanged at 5");
    assert_eq!(creature.max_health, 5, "Max health should be unchanged at 5");
}

#[test]
fn test_commander_passive_preserved_after_combat() {
    // Verify that commander passives are properly applied and the keywords persist
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // P1 has Void Archon (Quick), P2 has Siege Marshal Vex (+2 Attack, buffed in v0.8.0)
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

    // P2's creature should have +2 attack (4 instead of 2, Vex buffed in v0.8.0)
    assert_eq!(
        p2_creature.attack, 4,
        "P2's creature should have +2 Attack from commander passive"
    );
}

// =============================================================================
// TRIGGERED ABILITY TESTS
// =============================================================================

#[test]
fn test_high_artificer_summons_brass_cog_on_turn_start() {
    // The High Artificer (5000): At start of turn, summon a 2/2 Brass Cog
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game with High Artificer as P1's commander
    setup_game_with_commanders(&mut engine, 5000, 5001, 42);

    // After setup, we're at turn 2. The High Artificer triggers at the START of
    // each of P1's turns, so:
    // - Turn 1 Start: Summons 1 Brass Cog (slot 0)
    // - Turn 2 Start: Summons 1 Brass Cog (slot 1)
    // Total: 2 Brass Cogs

    // Check P1 has 2 creatures (Brass Cogs)
    let p1_creatures = &engine.state.players[0].creatures;
    assert_eq!(
        p1_creatures.len(),
        2,
        "P1 should have 2 Brass Cogs (one from turn 1 start, one from turn 2 start)"
    );

    let brass_cog = &p1_creatures[0];
    // Brass Cog is now 2/2 after v0.8.0 balance buff
    assert_eq!(brass_cog.attack, 2, "Brass Cog should have 2 attack");
    assert_eq!(brass_cog.current_health, 2, "Brass Cog should have 2 health");
}

#[test]
fn test_high_artificer_summons_multiple_tokens_over_turns() {
    // Verify High Artificer summons a new Brass Cog each turn
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game with High Artificer as P1's commander
    setup_game_with_commanders(&mut engine, 5000, 5001, 42);

    // After setup, we're at turn 2 with 2 Brass Cogs (turn 1 start + turn 2 start)
    assert_eq!(engine.state.players[0].creatures.len(), 2, "Should have 2 tokens after turn 2 start");

    // End turns to get to turn 3
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Should now have 3 Brass Cogs (turn 1, turn 2, turn 3)
    assert_eq!(
        engine.state.players[0].creatures.len(),
        3,
        "P1 should have 3 Brass Cogs after turn 3 start"
    );

    // End turns to get to turn 4
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Should now have 4 Brass Cogs
    assert_eq!(
        engine.state.players[0].creatures.len(),
        4,
        "P1 should have 4 Brass Cogs after turn 4 start"
    );
}

#[test]
fn test_broodmother_grants_rush_to_all_creatures() {
    // The Broodmother (5004): Your creatures have Rush
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Use Brass Sentinel (1000) - 2 cost, 2/4 Guard (no Rush naturally)
    let deck1: Vec<CardId> = vec![CardId(1000); 30];
    let deck2: Vec<CardId> = vec![CardId(1000); 30];

    engine.start_game_raw(
        deck1,
        deck2,
        CardId(5004), // Broodmother
        CardId(5000), // High Artificer
        42,
        GameMode::default(),
    ).unwrap();

    // Advance to turn 2 so P1 has 2 essence for Brass Sentinel
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Play the creature (no Rush naturally)
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("Failed to play creature");

    // The creature should have Rush granted by The Broodmother
    let p1_creatures = &engine.state.players[0].creatures;
    assert_eq!(p1_creatures.len(), 1, "P1 should have 1 creature");

    let creature = &p1_creatures[0];
    assert!(
        creature.keywords.has_rush(),
        "Brass Sentinel should have Rush granted by The Broodmother"
    );
}

#[test]
fn test_broodmother_rush_does_not_affect_opponent() {
    // Verify The Broodmother's Rush only affects own creatures, not opponent's
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Use Brass Sentinel (1000) - 2 cost, 2/4 Guard (no Rush naturally)
    let deck1: Vec<CardId> = vec![CardId(1000); 30];
    let deck2: Vec<CardId> = vec![CardId(1000); 30];

    // Use two passive commanders to avoid token spawning complications
    engine.start_game_raw(
        deck1,
        deck2,
        CardId(5004), // Broodmother (P1) - grants Rush
        CardId(5001), // Sanctum Healer (P2) - grants +0/+3
        42,
        GameMode::default(),
    ).unwrap();

    // Advance to turn 2 so both players have 2 essence
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // P1 plays creature - should have Rush
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P1 plays creature");
    engine.apply_action(Action::EndTurn).expect("P1 end turn");

    // P2 plays creature - should NOT have Rush (P2 has Sanctum Healer, not Broodmother)
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P2 plays creature");

    // Verify P1's creature has Rush
    let p1_creature = &engine.state.players[0].creatures[0];
    assert!(
        p1_creature.keywords.has_rush(),
        "P1's creature should have Rush from The Broodmother"
    );

    // Verify P2's creature does NOT have Rush (but should have Ward and +2 health from Sanctum Healer)
    let p2_creature = &engine.state.players[1].creatures[0];
    assert!(
        !p2_creature.keywords.has_rush(),
        "P2's creature should NOT have Rush (wrong commander)"
    );
    assert!(
        p2_creature.keywords.has_ward(),
        "P2's creature should have Ward from Sanctum Healer"
    );
    // Brass Sentinel base is 2/5, with Sanctum Healer should be 2/7
    assert_eq!(
        p2_creature.current_health, 7,
        "P2's creature should have +2 health from Sanctum Healer"
    );
}

#[test]
fn test_plague_sovereign_spawns_sporelings() {
    // Plague Sovereign (5005): At start of turn, summon 1/1 Sporeling with Fungal Rot
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Use minimal decks
    let deck1: Vec<CardId> = vec![CardId(1000); 30];
    let deck2: Vec<CardId> = vec![CardId(1000); 30];

    engine.start_game_raw(
        deck1,
        deck2,
        CardId(5005), // Plague Sovereign
        CardId(5001), // Sanctum Healer
        42,
        GameMode::default(),
    ).unwrap();

    // Advance to turn 2
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // After turn 2 start, P1 should have 2 Sporelings (turn 1 + turn 2)
    let p1_creatures = &engine.state.players[0].creatures;
    assert_eq!(
        p1_creatures.len(),
        2,
        "P1 should have 2 Sporelings after turn 2 start"
    );

    // Verify Sporeling stats (1/1)
    let sporeling = &p1_creatures[0];
    assert_eq!(sporeling.attack, 1, "Sporeling should have 1 attack");
    assert_eq!(sporeling.current_health, 1, "Sporeling should have 1 health");

    // Verify Sporeling has token abilities (Fungal Rot)
    assert!(
        sporeling.token_abilities.is_some(),
        "Sporeling should have token abilities (Fungal Rot)"
    );
}

#[test]
fn test_deathmaster_grants_quick_to_lethal_creatures() {
    // The Deathmaster (5009): Your creatures with Lethal have Quick
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // P1 (Deathmaster) uses Lethal creatures
    // Contract Killer (4036): 3 cost, 3/2 Lethal - should gain Quick
    let deck1: Vec<CardId> = vec![CardId(4036); 30]; // Contract Killer 3/2 Lethal
    let deck2: Vec<CardId> = vec![CardId(4031); 30]; // Eager Sellsword 2/1 Rush

    engine.start_game_raw(
        deck1,
        deck2,
        CardId(5009), // The Deathmaster
        CardId(5001), // Sanctum Healer
        42,
        GameMode::default(),
    ).unwrap();

    // Turn 1: P1 ends turn (need 3 essence for Contract Killer)
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 2 P1: still need more essence
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 3 P1: plays Contract Killer (3 cost, 3/2 Lethal)
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P1 plays creature");

    // Verify the Lethal creature got Quick from Deathmaster
    let creature = engine.state.players[0].get_creature(Slot(0)).expect("Creature should exist");
    assert!(
        creature.keywords.has_quick(),
        "Contract Killer should have Quick from Deathmaster passive"
    );
    assert!(
        creature.keywords.has_lethal(),
        "Contract Killer should still have Lethal"
    );
    // Stats should be unchanged (no buff)
    assert_eq!(creature.attack, 3, "Contract Killer attack should be 3");
    assert_eq!(creature.current_health, 2, "Contract Killer health should be 2 (base)");
}

#[test]
fn test_deathmaster_does_not_grant_quick_to_non_lethal_creatures() {
    // Verify The Deathmaster only grants Quick to creatures with Lethal
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // P1 (Deathmaster) uses non-Lethal creatures
    // Brass Sentinel (1000): 2 cost, 2/5 Guard - should NOT get Quick
    let deck1: Vec<CardId> = vec![CardId(1000); 30]; // Brass Sentinel 2/5 Guard
    let deck2: Vec<CardId> = vec![CardId(4031); 30]; // Eager Sellsword 2/1 Rush

    engine.start_game_raw(
        deck1,
        deck2,
        CardId(5009), // The Deathmaster
        CardId(5001), // Sanctum Healer
        42,
        GameMode::default(),
    ).unwrap();

    // Turn 1: P1 ends turn (need 2 essence)
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 2 P1: plays Brass Sentinel (2 cost, 2/5 Guard - no Lethal)
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P1 plays creature");

    // Verify the non-Lethal creature did NOT get Quick
    let creature = engine.state.players[0].get_creature(Slot(0)).expect("Creature should exist");
    assert!(
        !creature.keywords.has_quick(),
        "Brass Sentinel should NOT have Quick (no Lethal = no Quick from Deathmaster)"
    );
    assert!(
        creature.keywords.has_guard(),
        "Brass Sentinel should still have Guard"
    );
}
