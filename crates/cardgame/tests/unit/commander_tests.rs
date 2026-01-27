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
fn test_broodmother_summons_broodling_when_rush_creature_played() {
    // The Broodmother (5004): When you play a creature with Rush, summon a 1/1 Broodling with Rush
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Use Broodling (2003) - 1 cost, 2/2 Rush creature
    let deck1: Vec<CardId> = vec![CardId(2003); 30]; // Rush creatures
    let deck2: Vec<CardId> = vec![CardId(1000); 30]; // Brass Sentinels

    engine.start_game_raw(
        deck1,
        deck2,
        CardId(5004), // Broodmother
        CardId(5000), // High Artificer
        42,
        GameMode::default(),
    );

    // Turn 1: P1 has 1 essence, can play 1-cost Broodling
    // Play the Rush creature
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("Failed to play Rush creature");

    // Check P1 has 2 creatures: the Broodling played and the token summoned
    let p1_creatures = &engine.state.players[0].creatures;
    assert_eq!(
        p1_creatures.len(),
        2,
        "P1 should have 2 creatures: 1 played + 1 summoned Broodling token"
    );

    // Find the token (it should be the one with 1/1 stats)
    let token = p1_creatures.iter().find(|c| c.attack == 1 && c.current_health == 1);
    assert!(
        token.is_some(),
        "There should be a 1/1 Broodling token summoned by The Broodmother"
    );

    // Verify the token has Rush
    let token = token.unwrap();
    assert!(
        token.keywords.has_rush(),
        "Broodling token should have Rush keyword"
    );
}

#[test]
fn test_broodmother_summons_on_any_creature_played() {
    // Verify Broodmother triggers on ANY creature played (not just Rush)
    // This was buffed from Rush-only to any creature in v0.8.0
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Use Brass Sentinel (1000) - 2 cost, 2/4 Guard (no Rush)
    let deck1: Vec<CardId> = vec![CardId(1000); 30];
    let deck2: Vec<CardId> = vec![CardId(1000); 30];

    engine.start_game_raw(
        deck1,
        deck2,
        CardId(5004), // Broodmother
        CardId(5000), // High Artificer
        42,
        GameMode::default(),
    );

    // Advance to turn 2 so P1 has 2 essence
    engine.apply_action(Action::EndTurn).expect("P1 end turn");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Play the non-Rush creature
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("Failed to play creature");

    // Check P1 has 2 creatures (the played creature + Broodling token)
    let p1_creatures = &engine.state.players[0].creatures;
    assert_eq!(
        p1_creatures.len(),
        2,
        "P1 should have 2 creatures (played creature + Broodling token)"
    );

    // Verify the Broodling was summoned (1/1 with Rush)
    // Brass Sentinel is 2/4 with Guard, Broodling is 1/1 with Rush
    let broodling = p1_creatures.iter().find(|c| {
        c.attack == 1 && c.current_health == 1 && c.keywords.has_rush()
    });
    assert!(broodling.is_some(), "Broodling token should be summoned");
}

#[test]
fn test_plague_sovereign_deals_damage_on_ally_death() {
    // Plague Sovereign (5005): When one of your creatures dies, deal 1 damage to enemy commander
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Use low-health creatures that will die in combat
    // Eager Sellsword (4031) - 1 cost, 2/1 Rush
    let deck1: Vec<CardId> = vec![CardId(4031); 30]; // Rush creatures that will die easily
    let deck2: Vec<CardId> = vec![CardId(1000); 30]; // Brass Sentinel 2/4 Guard

    engine.start_game_raw(
        deck1,
        deck2,
        CardId(5005), // Plague Sovereign
        CardId(5001), // Sanctum Healer (passive, no tokens)
        42,
        GameMode::default(),
    );

    let initial_p2_health = engine.state.players[1].life;

    // Turn 1: P1 plays a 1-cost Rush creature (can attack immediately)
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P1 plays creature");

    // End turn
    engine.apply_action(Action::EndTurn).expect("P1 end turn");

    // P2 turn 1: P2 only has 2 essence (first-player adjustment), need turn 2 to play 2-cost
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 2: P1 can attack, but let's wait for P2 to have a creature
    engine.apply_action(Action::EndTurn).expect("P1 end turn");

    // P2 turn 2: now has 3 essence, can play Brass Sentinel (2 cost)
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P2 plays creature");

    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 3: P1 attacks with their 2/1 Rush creature against P2's 2/4 Guard
    // P1's creature will die (has 1 health, takes 2 damage from Sentinel)
    // Plague Sovereign should trigger, dealing 1 damage to P2

    // First, check P1's creature exists and can attack
    assert!(engine.state.players[0].get_creature(Slot(0)).is_some(), "P1 creature should exist");

    // Attack P2's Brass Sentinel
    let attack_action = Action::Attack {
        attacker: Slot(0),
        defender: Slot(0),
    };
    engine.apply_action(attack_action).expect("Attack should succeed");

    // P1's creature (2/1) should have died from Brass Sentinel's 2 attack
    assert!(
        engine.state.players[0].get_creature(Slot(0)).is_none(),
        "P1's creature should have died in combat"
    );

    // Check P2's health decreased by 2 from Plague Sovereign trigger (buffed from 1 to 2 in v0.8.0)
    let p2_health_after = engine.state.players[1].life;
    assert_eq!(
        p2_health_after,
        initial_p2_health - 2,
        "P2 should have taken 2 damage from Plague Sovereign trigger (was {}, now {})",
        initial_p2_health,
        p2_health_after
    );
}

#[test]
fn test_shadow_emperor_kael_deals_damage_on_kill() {
    // Shadow Emperor Kael (5009): When one of your creatures kills an enemy, deal 1 damage to enemy commander
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // P1 (Kael) uses strong creatures that will kill enemies
    // P2 uses weaker creatures
    let deck1: Vec<CardId> = vec![CardId(1000); 30]; // Brass Sentinel 2/4 Guard (strong)
    let deck2: Vec<CardId> = vec![CardId(4031); 30]; // Eager Sellsword 2/1 Rush (weak)

    engine.start_game_raw(
        deck1,
        deck2,
        CardId(5009), // Shadow Emperor Kael
        CardId(5001), // Sanctum Healer
        42,
        GameMode::default(),
    );

    // Record P2's initial health
    let initial_p2_health = engine.state.players[1].life;

    // Turn 1: P1 ends turn (need essence for 2-cost creature)
    engine.apply_action(Action::EndTurn).expect("P1 end turn");

    // Turn 1 P2: plays Eager Sellsword (1 cost, 2/1 Rush)
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P2 plays creature");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 2 P1: plays Brass Sentinel (2 cost, 2/4 Guard)
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P1 plays creature");
    engine.apply_action(Action::EndTurn).expect("P1 end turn");

    // Turn 2 P2: End turn
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 3 P1: attack P2's Eager Sellsword
    // P1's 2/4 attacks P2's 2/1 - P2's creature dies, P1's creature survives with 2 HP
    let attack_action = Action::Attack {
        attacker: Slot(0),
        defender: Slot(0),
    };
    engine.apply_action(attack_action).expect("Attack should succeed");

    // P2's creature should have died (P1 killed it)
    assert!(
        engine.state.players[1].get_creature(Slot(0)).is_none(),
        "P2's creature should have died in combat"
    );

    // P2 should have taken 1 damage from Shadow Emperor Kael OnKill trigger
    let p2_health_after = engine.state.players[1].life;
    assert_eq!(
        p2_health_after,
        initial_p2_health - 1,
        "P2 should have taken 1 damage from Kael OnKill trigger (was {}, now {})",
        initial_p2_health,
        p2_health_after
    );
}

#[test]
fn test_shadow_emperor_kael_multiple_kills_multiple_damage() {
    // Verify Kael deals damage for each kill (in separate combats)
    let card_db = load_test_db();
    let mut engine = GameEngine::new(&card_db);

    // P1 (Kael) uses strong creatures that will kill enemies
    // P2 uses weaker creatures
    let deck1: Vec<CardId> = vec![CardId(1000); 30]; // Brass Sentinel 2/4 Guard (strong)
    let deck2: Vec<CardId> = vec![CardId(4031); 30]; // Eager Sellsword 2/1 Rush (weak)

    engine.start_game_raw(
        deck1,
        deck2,
        CardId(5009), // Shadow Emperor Kael
        CardId(5001), // Sanctum Healer
        42,
        GameMode::default(),
    );

    // Record P2's initial health
    let initial_p2_health = engine.state.players[1].life;

    // Turn 1: P1 ends turn
    engine.apply_action(Action::EndTurn).expect("P1 end turn");

    // Turn 1 P2: plays two Eager Sellswords (1 cost each, has 1 essence)
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P2 plays creature 1");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 2 P1: plays Brass Sentinel (2 cost)
    let action = Action::PlayCard { hand_index: 0, slot: Slot(0) };
    engine.apply_action(action).expect("P1 plays creature 1");
    engine.apply_action(Action::EndTurn).expect("P1 end turn");

    // Turn 2 P2: plays second Eager Sellsword
    let action = Action::PlayCard { hand_index: 0, slot: Slot(1) };
    engine.apply_action(action).expect("P2 plays creature 2");
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 3 P1: plays second Brass Sentinel
    let action = Action::PlayCard { hand_index: 0, slot: Slot(1) };
    engine.apply_action(action).expect("P1 plays creature 2");
    engine.apply_action(Action::EndTurn).expect("P1 end turn");

    // Turn 3 P2: End turn
    engine.apply_action(Action::EndTurn).expect("P2 end turn");

    // Turn 4 P1: attack and kill first enemy
    // P1's 2/4 attacks P2's 2/1 - P2's creature dies
    let attack_action = Action::Attack {
        attacker: Slot(0),
        defender: Slot(0),
    };
    engine.apply_action(attack_action).expect("P1 attack 1");

    // P2 should have taken 1 damage from Kael OnKill trigger
    let p2_health_after_first = engine.state.players[1].life;
    assert_eq!(
        p2_health_after_first,
        initial_p2_health - 1,
        "P2 should have taken 1 damage from first kill"
    );

    // P1 attacks and kills second enemy
    // P1's 2/4 attacks P2's 2/1 - P2's creature dies
    let attack_action = Action::Attack {
        attacker: Slot(1),
        defender: Slot(1),
    };
    engine.apply_action(attack_action).expect("P1 attack 2");

    // P2 should have taken another 1 damage from Kael OnKill trigger (total 2)
    let p2_health_after_second = engine.state.players[1].life;
    assert_eq!(
        p2_health_after_second,
        initial_p2_health - 2,
        "P2 should have taken 2 damage total from two kills"
    );
}
