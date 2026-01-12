//! Card playing logic tests.
//!
//! Tests for playing creatures, spells, and supports, including effect triggers and targeting.

mod common;

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
    engine.state.players[0].hand.push(CardInstance::new(CardId(1))); // Cost 2 creature

    // Play the creature
    engine.execute_play_card(0, Slot(0)).unwrap();

    // Verify AP was deducted
    assert_eq!(
        engine.state.players[0].action_points,
        3, // 5 - 2 = 3
        "AP should be 3 after playing cost 2 card"
    );
}

#[test]
fn test_invalid_play_not_enough_ap_fails() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state with not enough AP
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 1; // Only 1 AP
    engine.state.players[0].hand.push(CardInstance::new(CardId(1))); // Cost 2 creature

    // Attempt to play should fail
    let result = engine.execute_play_card(0, Slot(0));
    assert!(result.is_err(), "Play should fail with not enough AP");
    assert!(result.unwrap_err().contains("Not enough AP"));
}

#[test]
fn test_card_removed_from_hand_on_play() {
    let card_db = card_playing_test_db();
    let mut engine = GameEngine::new(&card_db);

    // Set up game state
    engine.state.current_turn = 1;
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].action_points = 5;
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
