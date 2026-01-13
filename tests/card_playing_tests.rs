//! Card playing logic tests.
//!
//! Tests for playing creatures, spells, and supports, including effect triggers and targeting.

mod common;

use cardgame::actions::Action;
use cardgame::effects::{EffectTarget, TargetingRule};
use cardgame::engine::{resolve_spell_target, GameEngine};
use cardgame::keywords::Keywords;
use cardgame::state::CardInstance;
use cardgame::types::{CardId, PlayerId, Slot};
use common::*;

#[test]
fn test_play_creature_placed_on_board_with_correct_stats() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 3;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 5); // Enough essence for cost-2 card
    engine.state.players[0].hand.push(CardInstance::new(CardId(1))); // Test Creature (2/3)

    // Play the creature at slot 2
    let result = engine.execute_play_card(0, Slot(2));
    assert!(result.is_ok(), "Play card should succeed");

    // Verify creature is on board with correct stats
    let creature = engine.state.get_creature(PlayerId::PLAYER_ONE, Slot(2));
    assert!(creature.is_some(), "Creature should be on board");

    let creature = creature.unwrap();
    assert_eq!(creature.attack, 2, "Attack should be 2");
    assert_eq!(creature.current_health, 3, "Health should be 3");
    assert_eq!(creature.max_health, 3, "Max health should be 3");
    assert_eq!(creature.card_id, CardId(1), "Card ID should match");
    assert_eq!(creature.owner, PlayerId::PLAYER_ONE, "Owner should be player one");
    assert_eq!(creature.slot, Slot(2), "Slot should be 2");
}

#[test]
fn test_play_creature_with_rush_can_attack_immediately() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 3;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 5);
    engine.state.players[0].hand.push(CardInstance::new(CardId(2))); // Rush Creature

    // Play the rush creature
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify creature can attack (has Rush, so ignores summoning sickness)
    let creature = engine.state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert!(creature.keywords.has_rush(), "Creature should have Rush keyword");
    assert!(creature.can_attack(engine.state.current_turn), "Rush creature should be able to attack immediately");
}

#[test]
fn test_play_creature_without_rush_has_summoning_sickness() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 3;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 5);
    engine.state.players[0].hand.push(CardInstance::new(CardId(1))); // Test Creature (no Rush)

    // Play the creature
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify creature cannot attack (summoning sickness)
    let creature = engine.state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert!(!creature.keywords.has_rush(), "Creature should not have Rush");
    assert!(!creature.can_attack(engine.state.current_turn), "Non-Rush creature should have summoning sickness");
}

#[test]
fn test_play_creature_with_onplay_effect_triggers() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 3;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 5);
    engine.state.players[0].hand.push(CardInstance::new(CardId(3))); // Draw Creature (OnPlay: draw 1)

    // Add cards to deck for drawing
    engine.state.players[0].deck.push(CardInstance::new(CardId(1)));
    engine.state.players[0].deck.push(CardInstance::new(CardId(1)));

    let initial_hand_size = engine.state.players[0].hand.len();
    let initial_deck_size = engine.state.players[0].deck.len();

    // Play the creature (will trigger OnPlay draw effect)
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify OnPlay effect triggered (drew 1 card)
    // Hand: removed 1 (played), gained 1 (drawn) = same size
    // Deck: removed 1 (drawn) = initial - 1
    assert_eq!(
        engine.state.players[0].hand.len(),
        initial_hand_size, // -1 played, +1 drawn
        "Hand size should be same after playing draw creature"
    );
    assert_eq!(
        engine.state.players[0].deck.len(),
        initial_deck_size - 1,
        "Deck should have one less card after draw"
    );
}

#[test]
fn test_play_spell_with_notarget_effects_apply() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 3;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);
    engine.state.players[0].hand.push(CardInstance::new(CardId(5))); // Draw Spell (draw 2)

    // Add cards to deck for drawing
    for _ in 0..5 {
        engine.state.players[0].deck.push(CardInstance::new(CardId(1)));
    }

    let initial_hand_size = engine.state.players[0].hand.len();
    let initial_deck_size = engine.state.players[0].deck.len();

    // Play the spell
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify spell effects applied (drew 2 cards)
    // Hand: removed 1 (played spell), gained 2 (drawn) = +1
    // Deck: removed 2 (drawn) = -2
    assert_eq!(
        engine.state.players[0].hand.len(),
        initial_hand_size + 1, // -1 played spell, +2 drawn
        "Hand size should increase by 1 after playing draw spell"
    );
    assert_eq!(
        engine.state.players[0].deck.len(),
        initial_deck_size - 2,
        "Deck should have 2 less cards after draw"
    );
}

#[test]
fn test_play_spell_targeting_enemy_creature() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 3;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);
    engine.state.players[0].hand.push(CardInstance::new(CardId(4))); // Damage Spell (3 damage)

    // Create an enemy creature with 5 health
    create_test_creature(
        &mut engine.state,
        PlayerId::PLAYER_TWO,
        Slot(2),
        2,
        5,
        Keywords::none(),
    );

    // Play the spell targeting enemy slot 2
    engine.execute_play_card(0, Slot(2)).unwrap();

    // Verify damage was dealt
    let creature = engine.state.get_creature(PlayerId::PLAYER_TWO, Slot(2)).unwrap();
    assert_eq!(creature.current_health, 2, "Creature should have taken 3 damage (5 - 3 = 2)");
}

#[test]
fn test_play_spell_targeting_ally_creature() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 5;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);
    engine.state.players[0].hand.push(CardInstance::new(CardId(8))); // Buff Spell (+2/+2)

    // Create a friendly creature
    create_test_creature(
        &mut engine.state,
        PlayerId::PLAYER_ONE,
        Slot(0),
        2,
        3,
        Keywords::none(),
    );

    // Play the buff spell targeting ally slot 0 (for ally targeting, slot 0-4 = ally)
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify buff was applied
    let creature = engine.state.get_creature(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert_eq!(creature.attack, 4, "Attack should be buffed to 4 (2 + 2)");
    assert_eq!(creature.current_health, 5, "Health should be buffed to 5 (3 + 2)");
}

#[test]
fn test_play_support_placed_in_support_slot() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 5;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);
    engine.state.players[0].hand.push(CardInstance::new(CardId(6))); // Test Support

    // Play the support at slot 0
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify support is on board
    let support = engine.state.get_support(PlayerId::PLAYER_ONE, Slot(0));
    assert!(support.is_some(), "Support should be on board");

    let support = support.unwrap();
    assert_eq!(support.card_id, CardId(6), "Card ID should match");
    assert_eq!(support.current_durability, 2, "Durability should be 2");
}

#[test]
fn test_play_support_with_onplay_effect() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 5;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);
    engine.state.players[0].hand.push(CardInstance::new(CardId(7))); // Draw Support (OnPlay: draw 1)

    // Add cards to deck for drawing
    for _ in 0..3 {
        engine.state.players[0].deck.push(CardInstance::new(CardId(1)));
    }

    let initial_hand_size = engine.state.players[0].hand.len();

    // Play the support
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify OnPlay effect triggered
    // Hand: removed 1 (played support), gained 1 (drawn) = same size
    assert_eq!(
        engine.state.players[0].hand.len(),
        initial_hand_size,
        "Hand size should be same after playing support with draw"
    );
}

#[test]
fn test_ap_correctly_deducted() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 5;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);
    engine.state.players[0].hand.push(CardInstance::new(CardId(1))); // Cost 2 creature

    // Play the creature
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify AP was deducted (1 AP per action, not card cost)
    assert_eq!(
        engine.state.players[0].action_points,
        4, // 5 - 1 = 4 (1 AP per action)
        "AP should be 4 after playing a card (1 AP per action)"
    );

    // Verify Essence was deducted (card's cost)
    assert_eq!(
        engine.state.players[0].current_essence,
        8, // 10 - 2 = 8 (card cost)
        "Essence should be 8 after playing cost 2 card"
    );
}

#[test]
fn test_invalid_play_not_enough_ap_fails() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state with not enough AP (0 AP, but have Essence)
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 0; // No AP
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10); // Plenty of Essence
    engine.state.players[0].hand.push(CardInstance::new(CardId(1))); // Cost 2 creature

    // Attempt to play should fail due to no AP
    let result = engine.execute_play_card(0, Slot(0));
    assert!(result.is_err(), "Play should fail with not enough AP");
    assert!(result.unwrap_err().contains("Not enough AP"));
}

#[test]
fn test_invalid_play_not_enough_essence_fails() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state with enough AP but not enough Essence
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 3; // Plenty of AP
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 1); // Only 1 Essence
    engine.state.players[0].hand.push(CardInstance::new(CardId(1))); // Cost 2 creature

    // Attempt to play should fail due to insufficient Essence
    let result = engine.execute_play_card(0, Slot(0));
    assert!(result.is_err(), "Play should fail with not enough Essence");
    assert!(result.unwrap_err().contains("Not enough Essence"));
}

#[test]
fn test_card_removed_from_hand_on_play() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 5;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);
    engine.state.players[0].hand.push(CardInstance::new(CardId(1)));
    engine.state.players[0].hand.push(CardInstance::new(CardId(2)));

    assert_eq!(engine.state.players[0].hand.len(), 2, "Should start with 2 cards");

    // Play the first card
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify card was removed from hand
    assert_eq!(
        engine.state.players[0].hand.len(),
        1,
        "Hand should have 1 card after playing"
    );
}

#[test]
fn test_spell_kills_creature() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 3;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);
    engine.state.players[0].hand.push(CardInstance::new(CardId(4))); // Damage Spell (3 damage)

    // Create an enemy creature with only 2 health (less than damage)
    create_test_creature(
        &mut engine.state,
        PlayerId::PLAYER_TWO,
        Slot(1),
        2,
        2,
        Keywords::none(),
    );

    // Play the spell targeting enemy slot 1
    engine.execute_play_card(0, Slot(1)).unwrap();

    // Verify creature was killed
    let creature = engine.state.get_creature(PlayerId::PLAYER_TWO, Slot(1));
    assert!(creature.is_none(), "Creature should be dead");
}

#[test]
fn test_resolve_spell_target_no_target() {
    let target = resolve_spell_target(
        &TargetingRule::NoTarget,
        Slot(0),
        PlayerId::PLAYER_ONE,
    ).unwrap();

    assert_eq!(target, EffectTarget::None);
}

#[test]
fn test_resolve_spell_target_enemy_creature() {
    let target = resolve_spell_target(
        &TargetingRule::TargetEnemyCreature,
        Slot(2),
        PlayerId::PLAYER_ONE,
    ).unwrap();

    assert_eq!(
        target,
        EffectTarget::Creature {
            owner: PlayerId::PLAYER_TWO,
            slot: Slot(2),
        }
    );
}

#[test]
fn test_resolve_spell_target_ally_creature() {
    let target = resolve_spell_target(
        &TargetingRule::TargetAllyCreature,
        Slot(3),
        PlayerId::PLAYER_ONE,
    ).unwrap();

    assert_eq!(
        target,
        EffectTarget::Creature {
            owner: PlayerId::PLAYER_ONE,
            slot: Slot(3),
        }
    );
}

#[test]
fn test_resolve_spell_target_enemy_player() {
    let target = resolve_spell_target(
        &TargetingRule::TargetEnemyPlayer,
        Slot(0),
        PlayerId::PLAYER_ONE,
    ).unwrap();

    assert_eq!(target, EffectTarget::Player(PlayerId::PLAYER_TWO));
}

#[test]
fn test_resolve_spell_target_player_enemy() {
    let target = resolve_spell_target(
        &TargetingRule::TargetPlayer,
        Slot(0), // 0 = enemy
        PlayerId::PLAYER_ONE,
    ).unwrap();

    assert_eq!(target, EffectTarget::Player(PlayerId::PLAYER_TWO));
}

#[test]
fn test_resolve_spell_target_player_self() {
    let target = resolve_spell_target(
        &TargetingRule::TargetPlayer,
        Slot(1), // 1 = self
        PlayerId::PLAYER_ONE,
    ).unwrap();

    assert_eq!(target, EffectTarget::Player(PlayerId::PLAYER_ONE));
}

// =============================================================================
// SUPPORT CARD PASSIVE EFFECT TESTS
// =============================================================================

/// Test that attack bonus passive effect from support increases creature attack
#[test]
fn test_support_attack_bonus_passive_effect() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 10;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);

    // First play a creature (CardId 1: 2/3 stats)
    engine.state.players[0].hand.push(CardInstance::new(CardId(1)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify creature has base attack
    let creature = engine.state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(creature.attack, 2, "Creature should have base attack of 2");

    // Now play the attack bonus support (CardId 10: War Banner, +1 attack)
    engine.state.players[0].hand.push(CardInstance::new(CardId(10)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify support is on board
    let support = engine.state.get_support(PlayerId::PLAYER_ONE, Slot(0));
    assert!(support.is_some(), "Support should be on board");

    // Verify creature now has buffed attack
    let creature = engine.state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(
        creature.attack, 3,
        "Creature attack should be buffed to 3 (2 base + 1 from War Banner)"
    );
}

/// Test that health bonus passive effect from support increases creature health
#[test]
fn test_support_health_bonus_passive_effect() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 10;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);

    // First play a creature (CardId 1: 2/3 stats)
    engine.state.players[0].hand.push(CardInstance::new(CardId(1)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify creature has base health
    let creature = engine.state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(creature.current_health, 3, "Creature should have base health of 3");
    assert_eq!(creature.max_health, 3, "Creature should have max health of 3");

    // Now play the health bonus support (CardId 11: Barrier Shield, +2 health)
    engine.state.players[0].hand.push(CardInstance::new(CardId(11)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify creature now has buffed health
    let creature = engine.state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(
        creature.current_health, 5,
        "Creature current health should be buffed to 5 (3 base + 2 from Barrier Shield)"
    );
    assert_eq!(
        creature.max_health, 5,
        "Creature max health should be buffed to 5 (3 base + 2 from Barrier Shield)"
    );
}

/// Test that grant keyword passive effect gives creatures the keyword
#[test]
fn test_support_grant_keyword_passive_effect() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 10;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);

    // First play a creature without Rush (CardId 1: basic 2/3)
    engine.state.players[0].hand.push(CardInstance::new(CardId(1)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify creature does NOT have Rush
    let creature = engine.state.players[0].get_creature(Slot(0)).unwrap();
    assert!(
        !creature.keywords.has_rush(),
        "Creature should not have Rush before support is played"
    );

    // Now play the Rush grant support (CardId 12: Haste Totem, grants Rush)
    engine.state.players[0].hand.push(CardInstance::new(CardId(12)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify creature now has Rush keyword
    let creature = engine.state.players[0].get_creature(Slot(0)).unwrap();
    assert!(
        creature.keywords.has_rush(),
        "Creature should have Rush after Haste Totem is played"
    );
}

/// Test that passive effects apply to newly played creatures too
#[test]
fn test_support_passive_effect_applies_to_new_creatures() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 15;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);

    // First play the attack bonus support (CardId 10: War Banner, +1 attack)
    engine.state.players[0].hand.push(CardInstance::new(CardId(10)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Now play a creature AFTER the support is already on board
    engine.state.players[0].hand.push(CardInstance::new(CardId(1)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify new creature has buffed attack immediately
    let creature = engine.state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(
        creature.attack, 3,
        "Newly played creature should have buffed attack of 3 (2 base + 1 from War Banner)"
    );
}

/// Test that passive effects are removed when support is destroyed
#[test]
fn test_support_passive_effect_removed_when_support_destroyed() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 10;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);

    // Play a creature
    engine.state.players[0].hand.push(CardInstance::new(CardId(1)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Play the attack bonus support
    engine.state.players[0].hand.push(CardInstance::new(CardId(10)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify creature has buffed attack
    let creature = engine.state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(creature.attack, 3, "Creature should have buffed attack");

    // Remove the support properly (simulating destruction)
    engine.remove_support(PlayerId::PLAYER_ONE, Slot(0));

    // Verify creature's attack returns to base
    let creature = engine.state.players[0].get_creature(Slot(0)).unwrap();
    assert_eq!(
        creature.attack, 2,
        "Creature attack should return to base 2 after support is removed"
    );
}

// =============================================================================
// SUPPORT CARD TRIGGERED EFFECT TESTS
// =============================================================================

/// Test that StartOfTurn triggered effect fires at the start of owner's turn
#[test]
fn test_support_start_of_turn_triggered_effect() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 10;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);
    engine.state.players[0].life = 25; // Reduced life to see healing

    // Play the StartOfTurn heal support (CardId 13: Healing Shrine, heal 2)
    engine.state.players[0].hand.push(CardInstance::new(CardId(13)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify support is on board
    let support = engine.state.get_support(PlayerId::PLAYER_ONE, Slot(0));
    assert!(support.is_some(), "Support should be on board");

    // Life should still be 25 (no healing on play)
    assert_eq!(engine.state.players[0].life, 25, "Life should be 25 before turn start");

    // End turn (P1 -> P2)
    engine.apply_action(Action::EndTurn).unwrap();

    // End P2's turn (P2 -> P1)
    engine.apply_action(Action::EndTurn).unwrap();

    // Now it's P1's turn again - StartOfTurn should have triggered
    assert_eq!(
        engine.state.players[0].life, 27,
        "Life should be 27 after StartOfTurn heal (25 + 2)"
    );
}

/// Test that StartOfTurn only triggers for the support owner
#[test]
fn test_support_start_of_turn_only_triggers_for_owner() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 10;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);
    engine.state.players[0].life = 25;
    engine.state.players[1].life = 25;

    // P1 plays the heal support
    engine.state.players[0].hand.push(CardInstance::new(CardId(13)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // End P1's turn (P1 -> P2)
    engine.apply_action(Action::EndTurn).unwrap();

    // P2's turn starts - P1's support should NOT trigger
    assert_eq!(
        engine.state.players[0].life, 25,
        "P1's life should still be 25 (not their turn)"
    );
    assert_eq!(
        engine.state.players[1].life, 25,
        "P2's life should still be 25 (not their support)"
    );
}

// =============================================================================
// SUPPORT CARD DURABILITY TESTS
// =============================================================================

/// Test that support durability is tracked correctly
#[test]
fn test_support_durability_initial_value() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 10;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);

    // Play a support with durability 3 (CardId 10: War Banner)
    engine.state.players[0].hand.push(CardInstance::new(CardId(10)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify durability is set correctly
    let support = engine.state.get_support(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert_eq!(support.current_durability, 3, "Support should have 3 durability");
}

/// Test that support durability decrements each turn
#[test]
fn test_support_durability_decrements_each_turn() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 10;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);

    // Play a support with durability 3 (CardId 10: War Banner)
    engine.state.players[0].hand.push(CardInstance::new(CardId(10)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify initial durability
    let support = engine.state.get_support(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert_eq!(support.current_durability, 3, "Initial durability should be 3");

    // End turn twice (full round)
    engine.apply_action(Action::EndTurn).unwrap(); // P1 -> P2
    engine.apply_action(Action::EndTurn).unwrap(); // P2 -> P1

    // Durability should have decremented at end of P1's turn
    let support = engine.state.get_support(PlayerId::PLAYER_ONE, Slot(0)).unwrap();
    assert_eq!(
        support.current_durability, 2,
        "Durability should be 2 after one round"
    );
}

/// Test that support is removed when durability reaches 0
#[test]
fn test_support_removed_when_durability_depleted() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 10;
    setup_test_essence(&mut engine.state, PlayerId::PLAYER_ONE, 10);

    // Play a support with durability 2 (CardId 12: Haste Totem)
    engine.state.players[0].hand.push(CardInstance::new(CardId(12)));
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify support is on board
    assert!(
        engine.state.get_support(PlayerId::PLAYER_ONE, Slot(0)).is_some(),
        "Support should be on board initially"
    );

    // Complete 2 full rounds (4 end turns)
    for _ in 0..4 {
        engine.apply_action(Action::EndTurn).unwrap();
    }

    // Support should be removed after durability hits 0
    assert!(
        engine.state.get_support(PlayerId::PLAYER_ONE, Slot(0)).is_none(),
        "Support should be removed after durability depleted"
    );
}
