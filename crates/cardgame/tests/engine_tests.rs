//! Core engine functionality tests.
//!
//! Tests for game setup, turn flow, win conditions, and basic gameplay mechanics.

mod common;

use cardgame::actions::Action;
use cardgame::config::{game, player};
use cardgame::engine::{seeded_shuffle, GameEngine};
use cardgame::state::{Creature, CreatureStatus, GameMode, GameResult, WinReason};
use cardgame::keywords::Keywords;
use cardgame::types::{CardId, PlayerId, Slot};
use common::*;

/// Default commander for tests (The High Artificer).
const DEFAULT_COMMANDER: cardgame::types::CardId = cardgame::types::CardId(5000);

#[test]
fn test_new_game_setup() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    let deck1 = simple_deck();
    let deck2 = simple_deck();

    engine.start_game_raw(deck1, deck2, DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Both players should start with 30 life
    assert_eq!(engine.state.players[0].life, 30);
    assert_eq!(engine.state.players[1].life, 30);

    // Player 1 should be active
    assert_eq!(engine.state.active_player, PlayerId::PLAYER_ONE);

    // Turn counter should be 1
    assert_eq!(engine.state.current_turn, 1);

    // Player 1 should have 3 AP (restored at turn start)
    assert_eq!(engine.state.players[0].action_points, player::AP_PER_TURN);

    // Player 2 should have 0 AP (not their turn yet)
    assert_eq!(engine.state.players[1].action_points, 0);

    // Each player draws STARTING_HAND_SIZE cards, then P1 draws 1 at turn start
    // P1: STARTING_HAND_SIZE + 1 turn start draw
    // P2: STARTING_HAND_SIZE + P2_BONUS_CARDS (FPA compensation)
    let starting_hand = player::STARTING_HAND_SIZE;
    let p2_bonus = player::P2_BONUS_CARDS;
    assert_eq!(engine.state.players[0].hand.len(), starting_hand + 1);
    assert_eq!(engine.state.players[1].hand.len(), starting_hand + p2_bonus);

    // Each player should have remaining cards in deck
    // simple_deck() returns 30 cards
    let deck_size = 30;
    assert_eq!(engine.state.players[0].deck.len(), deck_size - starting_hand - 1);
    assert_eq!(engine.state.players[1].deck.len(), deck_size - starting_hand - p2_bonus);

    // Game should not be terminal
    assert!(!engine.is_terminal());
    assert!(engine.winner().is_none());
}

#[test]
fn test_turn_start_ap_restored() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(simple_deck(), simple_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Spend some AP
    engine.state.players[0].action_points = 0;

    // End turn (switches to P2)
    engine.apply_action(Action::EndTurn).unwrap();

    // P2 should now have 3 AP
    assert_eq!(engine.state.players[1].action_points, player::AP_PER_TURN);

    // P1's AP should still be 0 (not their turn)
    assert_eq!(engine.state.players[0].action_points, 0);
}

#[test]
fn test_turn_start_card_drawn() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(simple_deck(), simple_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // P1 has 4 cards, P2 has 3 cards after game start
    let p1_hand_before = engine.state.players[0].hand.len();
    let p2_hand_before = engine.state.players[1].hand.len();

    // End P1's turn
    engine.apply_action(Action::EndTurn).unwrap();

    // P2 should have drawn a card (turn start)
    assert_eq!(engine.state.players[1].hand.len(), p2_hand_before + 1);

    // P1's hand should be unchanged
    assert_eq!(engine.state.players[0].hand.len(), p1_hand_before);
}

#[test]
fn test_turn_start_creatures_can_attack() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(simple_deck(), simple_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Add a creature to P1's board from previous turn
    let creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
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
        turn_played: 0, // Played on a previous turn
        frenzy_stacks: 0,
        token_abilities: None,
        token_name: None,
    };
    engine.state.players[0].creatures.push(creature);

    // Mark it as exhausted (attacked last turn)
    engine.state.players[0].creatures[0].status.set_exhausted(true);

    // End P1's turn, then P2's turn to get back to P1
    engine.apply_action(Action::EndTurn).unwrap();
    engine.apply_action(Action::EndTurn).unwrap();

    // Creature should no longer be exhausted
    assert!(!engine.state.players[0].creatures[0].status.is_exhausted());
}

#[test]
fn test_turn_end_player_switches() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(simple_deck(), simple_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    assert_eq!(engine.state.active_player, PlayerId::PLAYER_ONE);

    engine.apply_action(Action::EndTurn).unwrap();
    assert_eq!(engine.state.active_player, PlayerId::PLAYER_TWO);

    engine.apply_action(Action::EndTurn).unwrap();
    assert_eq!(engine.state.active_player, PlayerId::PLAYER_ONE);

    // Turn counter should have advanced
    assert_eq!(engine.state.current_turn, 3);
}

#[test]
fn test_win_by_damage() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(simple_deck(), simple_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Add a powerful creature to P1's board
    let creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
        card_id: CardId(3),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(2),
        attack: 30, // Enough to kill in one hit
        current_health: 5,
        max_health: 5,
        base_attack: 30,
        base_health: 5,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0, // Not summoning sick
        frenzy_stacks: 0,
        token_abilities: None,
        token_name: None,
    };
    engine.state.players[0].creatures.push(creature);

    // Attack P2's face (slot 2 can attack 1, 2, 3)
    engine
        .apply_action(Action::Attack {
            attacker: Slot(2),
            defender: Slot(2),
        })
        .unwrap();

    // P2 should have 0 or less life
    assert!(engine.state.players[1].life <= 0);

    // Game should be terminal
    assert!(engine.is_terminal());

    // P1 should be the winner
    assert_eq!(engine.winner(), Some(PlayerId::PLAYER_ONE));

    // Win reason should be life reached zero
    if let Some(GameResult::Win { reason, .. }) = &engine.state.result {
        assert_eq!(*reason, WinReason::LifeReachedZero);
    } else {
        panic!("Expected win result");
    }
}

#[test]
fn test_win_by_turn_limit_higher_life() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(simple_deck(), simple_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Set P1 life higher
    engine.state.players[0].life = 25;
    engine.state.players[1].life = 15;

    // Set turn to just before limit
    engine.state.current_turn = game::TURN_LIMIT as u16;

    // End turn to trigger turn limit
    engine.apply_action(Action::EndTurn).unwrap();

    // Game should be terminal
    assert!(engine.is_terminal());

    // P1 should win (higher life)
    assert_eq!(engine.winner(), Some(PlayerId::PLAYER_ONE));

    if let Some(GameResult::Win { reason, .. }) = &engine.state.result {
        assert_eq!(*reason, WinReason::TurnLimitHigherLife);
    }
}

#[test]
fn test_win_by_turn_limit_tie_p1_wins() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(simple_deck(), simple_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Equal life
    engine.state.players[0].life = 20;
    engine.state.players[1].life = 20;

    // Set turn to just before limit
    engine.state.current_turn = game::TURN_LIMIT as u16;

    // End turn to trigger turn limit
    engine.apply_action(Action::EndTurn).unwrap();

    // Game should be terminal
    assert!(engine.is_terminal());

    // P1 should win on tie
    assert_eq!(engine.winner(), Some(PlayerId::PLAYER_ONE));
}

#[test]
fn test_seeded_shuffle_deterministic() {
    let mut items1 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut items2 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    seeded_shuffle(&mut items1, 42);
    seeded_shuffle(&mut items2, 42);

    // Same seed should produce same result
    assert_eq!(items1, items2);

    // Different seed should (almost certainly) produce different result
    let mut items3 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    seeded_shuffle(&mut items3, 12345);
    assert_ne!(items1, items3);
}

#[test]
fn test_seeded_shuffle_actually_shuffles() {
    let original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut shuffled = original.clone();

    seeded_shuffle(&mut shuffled, 999);

    // Should not be in original order (very unlikely to be unchanged)
    assert_ne!(original, shuffled);

    // Should contain all the same elements
    let mut sorted = shuffled.clone();
    sorted.sort();
    assert_eq!(original, sorted);
}

#[test]
fn test_play_creature_card() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(simple_deck(), simple_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Give player essence to play cards
    engine.state.players[0].max_essence = 10;
    engine.state.players[0].current_essence = 10;

    // Find a creature card in hand
    let creature_card_idx = engine.state.players[0]
        .hand
        .iter()
        .position(|c| c.card_id.0 == 1 || c.card_id.0 == 2 || c.card_id.0 == 3);

    if let Some(idx) = creature_card_idx {
        let initial_ap = engine.state.players[0].action_points;
        let initial_essence = engine.state.players[0].current_essence;
        let card_id = engine.state.players[0].hand[idx].card_id;
        let card_cost = card_db.get(card_id).unwrap().cost;

        engine
            .apply_action(Action::PlayCard {
                hand_index: idx as u8,
                slot: Slot(0),
            })
            .unwrap();

        // Creature should be on board
        assert_eq!(engine.state.players[0].creatures.len(), 1);
        assert_eq!(engine.state.players[0].creatures[0].slot, Slot(0));

        // AP should be reduced by 1 (action cost, not card cost)
        assert_eq!(
            engine.state.players[0].action_points,
            initial_ap - 1
        );

        // Essence should be reduced by card cost
        assert_eq!(
            engine.state.players[0].current_essence,
            initial_essence - card_cost
        );

        // Card should be removed from hand
        assert!(!engine.state.players[0]
            .hand
            .iter()
            .any(|c| c.card_id == card_id)
            || engine.state.players[0]
                .hand
                .iter()
                .filter(|c| c.card_id == card_id)
                .count()
                < engine.state.players[0]
                    .hand
                    .iter()
                    .filter(|c| c.card_id == card_id)
                    .count()
                    + 1);
    }
}

#[test]
fn test_creature_combat() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(simple_deck(), simple_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Add creatures to both sides
    let p1_creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(2),
        attack: 3,
        current_health: 4,
        max_health: 4,
        base_attack: 3,
        base_health: 4,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
        frenzy_stacks: 0,
        token_abilities: None,
        token_name: None,
    };
    engine.state.players[0].creatures.push(p1_creature);

    let p2_creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_TWO,
        slot: Slot(2),
        attack: 2,
        current_health: 3,
        max_health: 3,
        base_attack: 2,
        base_health: 3,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
        frenzy_stacks: 0,
        token_abilities: None,
        token_name: None,
    };
    engine.state.players[1].creatures.push(p2_creature);

    // P1 attacks P2's creature
    engine
        .apply_action(Action::Attack {
            attacker: Slot(2),
            defender: Slot(2),
        })
        .unwrap();

    // P2's creature should be dead (3 health - 3 damage = 0)
    assert_eq!(engine.state.players[1].creatures.len(), 0);

    // P1's creature should be damaged but alive (4 health - 2 damage = 2)
    assert_eq!(engine.state.players[0].creatures.len(), 1);
    assert_eq!(engine.state.players[0].creatures[0].current_health, 2);

    // P1's creature should be exhausted
    assert!(engine.state.players[0].creatures[0].status.is_exhausted());
}

#[test]
fn test_direct_face_attack() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(simple_deck(), simple_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    let initial_life = engine.state.players[1].life;

    // Add creature to P1's board
    let creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(2),
        attack: 5,
        current_health: 5,
        max_health: 5,
        base_attack: 5,
        base_health: 5,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
        frenzy_stacks: 0,
        token_abilities: None,
        token_name: None,
    };
    engine.state.players[0].creatures.push(creature);

    // Attack empty slot (face damage)
    engine
        .apply_action(Action::Attack {
            attacker: Slot(2),
            defender: Slot(1),
        })
        .unwrap();

    // P2's life should be reduced
    assert_eq!(engine.state.players[1].life, initial_life - 5);
}

#[test]
fn test_illegal_action_rejected() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(simple_deck(), simple_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Try to attack with non-existent creature
    let result = engine.apply_action(Action::Attack {
        attacker: Slot(0),
        defender: Slot(0),
    });

    assert!(result.is_err());
}

#[test]
fn test_game_over_no_more_actions() {
    let card_db = test_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(simple_deck(), simple_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Set P2's life to 0 to end the game
    engine.state.players[1].life = 0;
    engine.check_life_victory();

    assert!(engine.is_terminal());

    // Any action should be rejected
    let result = engine.apply_action(Action::EndTurn);
    assert!(result.is_err());
}

// =============================================================================
// UseAbility Tests
// =============================================================================

use cardgame::actions::Target;
use cardgame::cards::{
    AbilityDefinition, CardDefinition, CardType, CommanderAbility, CommanderDefinition,
    CommanderPassiveAbility, CommanderPassiveEffect, EffectDefinition, Faction,
};
use cardgame::effects::{TargetingRule, Trigger};
use cardgame::types::Rarity;

/// Create a mock commander for local test databases.
fn local_mock_commander() -> CommanderDefinition {
    CommanderDefinition {
        id: DEFAULT_COMMANDER.0,
        name: "Mock Commander".to_string(),
        faction: Faction::Argentum,
        rarity: Rarity::Legendary,
        ability: CommanderAbility::Passive {
            passive_ability: CommanderPassiveAbility {
                description: "No effect".to_string(),
                effect: CommanderPassiveEffect::BuffStats { attack: 0, health: 0 },
            },
        },
        flavor: None,
    }
}

/// Create a card database with a creature that has an activated ability (damage effect)
fn ability_test_db() -> cardgame::cards::CardDatabase {
    let cards = vec![
        // Basic creature with no abilities
        CardDefinition {
            id: 1,
            name: "Basic Creature".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 2,
                health: 3,
                keywords: vec![],
                abilities: vec![],
                keywords_bits: cardgame::keywords::Keywords::none(),
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Creature with an ability that deals 2 damage to target enemy creature
        CardDefinition {
            id: 2,
            name: "Damage Creature".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 2,
                health: 2,
                keywords: vec![],
                abilities: vec![AbilityDefinition {
                    trigger: Trigger::OnPlay, // Using OnPlay as trigger for manual activation
                    targeting: TargetingRule::TargetEnemyCreature,
                    essence_cost: 0,
                    effects: vec![EffectDefinition::Damage { amount: 2, filter: None }],
                    conditional_effects: vec![],
                }],
                keywords_bits: cardgame::keywords::Keywords::none(),
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
        // Creature with a self-buff ability
        CardDefinition {
            id: 3,
            name: "Self Buffer".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 1,
                health: 3,
                keywords: vec![],
                abilities: vec![AbilityDefinition {
                    trigger: Trigger::OnPlay,
                    targeting: TargetingRule::NoTarget,
                    essence_cost: 0,
                    effects: vec![EffectDefinition::BuffStats { attack: 1, health: 1, filter: None }],
                    conditional_effects: vec![],
                }],
                keywords_bits: cardgame::keywords::Keywords::none(),
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
    ];
    cardgame::cards::CardDatabase::new_with_commanders(cards, vec![local_mock_commander()])
}

/// Create a deck for ability tests
fn ability_deck() -> Vec<CardId> {
    let mut deck = Vec::new();
    for _ in 0..10 {
        deck.push(CardId(1));
        deck.push(CardId(2));
        deck.push(CardId(3));
    }
    deck
}

/// Test that UseAbility actions are NOT generated for OnPlay-triggered abilities.
///
/// OnPlay abilities trigger automatically when a card is played - they are NOT
/// "activated" abilities that can be used at will via UseAbility actions.
/// The action space reserves indices 75-254 for future "Activated" trigger types.
#[test]
fn test_use_ability_basic() {
    let card_db = ability_test_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(ability_deck(), ability_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Add a creature with an OnPlay ability to P1's board
    let creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
        card_id: CardId(2), // Damage Creature (has OnPlay ability)
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        attack: 2,
        current_health: 2,
        max_health: 2,
        base_attack: 2,
        base_health: 2,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
        frenzy_stacks: 0,
        token_abilities: None,
        token_name: None,
    };
    engine.state.players[0].creatures.push(creature);

    // Verify that no UseAbility actions are in the legal actions
    let legal = engine.get_legal_actions();
    let has_use_ability = legal.iter().any(|a| matches!(a, Action::UseAbility { .. }));
    assert!(!has_use_ability, "UseAbility should NOT be in legal actions for OnPlay abilities");

    // Attempting to apply a UseAbility action directly should fail
    let result = engine.apply_action(Action::UseAbility {
        slot: Slot(0),
        ability_index: 0,
        target: Target::EnemySlot(Slot(0)),
    });
    assert!(result.is_err(), "UseAbility should fail because it's not in legal actions");
    assert!(result.unwrap_err().contains("Illegal action"));
}

#[test]
fn test_use_ability_silenced_fails() {
    let card_db = ability_test_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(ability_deck(), ability_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Add a silenced creature with ability to P1's board
    let mut creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
        card_id: CardId(2), // Damage Creature
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        attack: 2,
        current_health: 2,
        max_health: 2,
        base_attack: 2,
        base_health: 2,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
        frenzy_stacks: 0,
        token_abilities: None,
        token_name: None,
    };
    // Silence the creature
    creature.status.set_silenced(true);
    engine.state.players[0].creatures.push(creature);

    // Add a target creature to P2's board
    let target_creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_TWO,
        slot: Slot(0),
        attack: 2,
        current_health: 3,
        max_health: 3,
        base_attack: 2,
        base_health: 3,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
        frenzy_stacks: 0,
        token_abilities: None,
        token_name: None,
    };
    engine.state.players[1].creatures.push(target_creature);

    // Try to use the ability - should fail because creature is silenced
    let result = engine.apply_action(Action::UseAbility {
        slot: Slot(0),
        ability_index: 0,
        target: Target::EnemySlot(Slot(0)),
    });

    // The action should be rejected (not in legal actions because silenced)
    assert!(result.is_err(), "Use ability should fail when silenced");

    // Target creature health should be unchanged
    let target = engine.state.players[1].get_creature(Slot(0));
    assert!(target.is_some(), "Target creature should exist");
    assert_eq!(target.unwrap().current_health, 3, "Target health should be unchanged");
}

#[test]
fn test_use_ability_invalid_slot() {
    let card_db = ability_test_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(ability_deck(), ability_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Don't add any creatures - slot 0 is empty

    // Try to use ability on empty slot - should fail
    let result = engine.apply_action(Action::UseAbility {
        slot: Slot(0),
        ability_index: 0,
        target: Target::NoTarget,
    });

    assert!(result.is_err(), "Use ability should fail on empty slot");
}

#[test]
fn test_use_ability_invalid_ability_index() {
    let card_db = ability_test_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(ability_deck(), ability_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Add a creature with only 1 ability
    let creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
        card_id: CardId(2), // Has 1 ability at index 0
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        attack: 2,
        current_health: 2,
        max_health: 2,
        base_attack: 2,
        base_health: 2,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
        frenzy_stacks: 0,
        token_abilities: None,
        token_name: None,
    };
    engine.state.players[0].creatures.push(creature);

    // Try to use ability index 5 - which doesn't exist
    let result = engine.apply_action(Action::UseAbility {
        slot: Slot(0),
        ability_index: 5,
        target: Target::NoTarget,
    });

    assert!(result.is_err(), "Use ability should fail with invalid ability index");
}

/// Test that UseAbility with self-target is NOT generated for OnPlay abilities.
///
/// Even NoTarget/self-targeting OnPlay abilities should not be usable via UseAbility.
/// They trigger automatically when the card is played.
#[test]
fn test_use_ability_self_target() {
    let card_db = ability_test_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(ability_deck(), ability_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 12345, GameMode::default()).unwrap();

    // Add a creature with self-buff ability (OnPlay trigger)
    let creature = Creature {
        instance_id: engine.state.next_creature_instance_id(),
        card_id: CardId(3), // Self Buffer (has OnPlay ability)
        owner: PlayerId::PLAYER_ONE,
        slot: Slot(0),
        attack: 1,
        current_health: 3,
        max_health: 3,
        base_attack: 1,
        base_health: 3,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 0,
        frenzy_stacks: 0,
        token_abilities: None,
        token_name: None,
    };
    engine.state.players[0].creatures.push(creature);

    // Verify that no UseAbility actions are in the legal actions
    let legal = engine.get_legal_actions();
    let has_use_ability = legal.iter().any(|a| matches!(a, Action::UseAbility { .. }));
    assert!(!has_use_ability, "UseAbility should NOT be in legal actions for OnPlay self-buff");

    // Attempting to apply a UseAbility action directly should fail
    let result = engine.apply_action(Action::UseAbility {
        slot: Slot(0),
        ability_index: 0,
        target: Target::Self_,
    });
    assert!(result.is_err(), "UseAbility should fail because it's not in legal actions");
    assert!(result.unwrap_err().contains("Illegal action"));

    // Creature stats should remain unchanged (ability was NOT activated)
    let creature = engine.state.players[0].get_creature(Slot(0));
    assert!(creature.is_some(), "Creature should still exist");
    let creature = creature.unwrap();
    assert_eq!(creature.attack, 1, "Attack should be unchanged");
    assert_eq!(creature.current_health, 3, "Health should be unchanged");
}

// ============================================================================
// Conditional Spell Effect Tests (Phase 4B.1)
// ============================================================================

use cardgame::cards::ConditionalEffectGroup;
use cardgame::effects::Condition;
use cardgame::types::CreatureInstanceId;

/// Create a card database for conditional spell tests
fn conditional_card_db() -> cardgame::cards::CardDatabase {
    use cardgame::cards::{CardDefinition, CardType, EffectDefinition};
    use cardgame::effects::TargetingRule;
    use cardgame::types::Rarity;

    let cards = vec![
        // Basic creature to target
        CardDefinition {
            id: 1,
            name: "Target Dummy".to_string(),
            cost: 1,
            card_type: CardType::Creature {
                attack: 1,
                health: 2,
                keywords: vec![],
                abilities: vec![],
                keywords_bits: cardgame::keywords::Keywords::none(),
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Spell that deals 3 damage, if target dies, draw a card
        CardDefinition {
            id: 2,
            name: "Soul Harvest".to_string(),
            cost: 2,
            card_type: CardType::Spell {
                targeting: TargetingRule::TargetEnemyCreature,
                effects: vec![EffectDefinition::Damage { amount: 3, filter: None }],
                conditional_effects: vec![ConditionalEffectGroup {
                    condition: Condition::TargetDied,
                    effects: vec![EffectDefinition::Draw { count: 1 }],
                }],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
        // Spell that deals 1 damage, if target dies, draw a card (for non-kill scenario)
        CardDefinition {
            id: 3,
            name: "Weak Soul Harvest".to_string(),
            cost: 1,
            card_type: CardType::Spell {
                targeting: TargetingRule::TargetEnemyCreature,
                effects: vec![EffectDefinition::Damage { amount: 1, filter: None }],
                conditional_effects: vec![ConditionalEffectGroup {
                    condition: Condition::TargetDied,
                    effects: vec![EffectDefinition::Draw { count: 1 }],
                }],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
        // Tanky creature to survive weak spell
        CardDefinition {
            id: 4,
            name: "Tough Dummy".to_string(),
            cost: 1,
            card_type: CardType::Creature {
                attack: 1,
                health: 5,
                keywords: vec![],
                abilities: vec![],
                keywords_bits: cardgame::keywords::Keywords::none(),
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
    ];
    cardgame::cards::CardDatabase::new_with_commanders(cards, vec![local_mock_commander()])
}

/// Create a deck for conditional spell tests
fn conditional_deck() -> Vec<CardId> {
    vec![
        CardId(1), CardId(1), CardId(1), CardId(1), CardId(1),
        CardId(2), CardId(2), CardId(3), CardId(3), CardId(4),
        CardId(1), CardId(1), CardId(1), CardId(1), CardId(1),
        CardId(2), CardId(2), CardId(3), CardId(3), CardId(4),
    ]
}

#[test]
fn test_conditional_spell_triggers_on_kill() {
    use cardgame::state::CardInstance;

    let card_db = conditional_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(conditional_deck(), conditional_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 42, GameMode::default()).unwrap();

    // Directly set up the game state to avoid relying on random draws
    // Place a Target Dummy (id=1, 1/2) on P2's board
    let creature = Creature {
        instance_id: CreatureInstanceId(100),
        card_id: CardId(1),
        owner: PlayerId::PLAYER_TWO,
        slot: Slot(0),
        attack: 1,
        current_health: 2,
        max_health: 2,
        base_attack: 1,
        base_health: 2,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 1,
        frenzy_stacks: 0,
        token_abilities: None,
        token_name: None,
    };
    engine.state.players[1].creatures.push(creature);

    // Give P1 a Soul Harvest spell (id=2) in hand
    engine.state.players[0].hand.clear();
    engine.state.players[0].hand.push(CardInstance::new(CardId(2)));

    // Make sure P1 is active with resources
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].current_essence = 10;
    engine.state.players[0].action_points = 3;

    // Add cards to P1's deck so draw effect can work
    engine.state.players[0].deck.push(CardInstance::new(CardId(1)));
    engine.state.players[0].deck.push(CardInstance::new(CardId(1)));

    let p1_hand_before_spell = engine.state.players[0].hand.len(); // Should be 1

    // Play Soul Harvest targeting enemy creature in slot 0
    engine.apply_action(Action::PlayCard {
        hand_index: 0,
        slot: Slot(0), // Target enemy creature in slot 0
    }).expect("P1 should be able to play Soul Harvest");

    // Target Dummy should be dead (had 2 health, took 3 damage)
    assert!(engine.state.players[1].get_creature(Slot(0)).is_none(), "Target Dummy should be dead");

    // P1 should have drawn a card from conditional effect
    // Hand was: 1 card, -1 for playing spell, +1 for conditional draw = 1 card
    let expected_hand = p1_hand_before_spell - 1 + 1; // -1 for spell, +1 for draw
    assert_eq!(
        engine.state.players[0].hand.len(),
        expected_hand,
        "P1 should have drawn a card from conditional effect"
    );
}

#[test]
fn test_conditional_spell_does_not_trigger_when_target_survives() {
    use cardgame::state::CardInstance;

    let card_db = conditional_card_db();
    let mut engine = GameEngine::new(&card_db);

    engine.start_game_raw(conditional_deck(), conditional_deck(), DEFAULT_COMMANDER, DEFAULT_COMMANDER, 42, GameMode::default()).unwrap();

    // Directly set up the game state
    // Place a Tough Dummy (id=4, 1/5) on P2's board
    let creature = Creature {
        instance_id: CreatureInstanceId(100),
        card_id: CardId(4),
        owner: PlayerId::PLAYER_TWO,
        slot: Slot(0),
        attack: 1,
        current_health: 5,
        max_health: 5,
        base_attack: 1,
        base_health: 5,
        keywords: Keywords::none(),
        status: CreatureStatus::default(),
        turn_played: 1,
        frenzy_stacks: 0,
        token_abilities: None,
        token_name: None,
    };
    engine.state.players[1].creatures.push(creature);

    // Give P1 a Weak Soul Harvest spell (id=3) in hand
    engine.state.players[0].hand.clear();
    engine.state.players[0].hand.push(CardInstance::new(CardId(3)));

    // Make sure P1 is active with resources
    engine.state.active_player = PlayerId::PLAYER_ONE;
    engine.state.players[0].current_essence = 10;
    engine.state.players[0].action_points = 3;

    // Add cards to P1's deck (in case draw effect triggers, but it shouldn't)
    engine.state.players[0].deck.push(CardInstance::new(CardId(1)));
    engine.state.players[0].deck.push(CardInstance::new(CardId(1)));

    let p1_hand_before_spell = engine.state.players[0].hand.len(); // Should be 1

    // Play Weak Soul Harvest targeting enemy creature in slot 0
    engine.apply_action(Action::PlayCard {
        hand_index: 0,
        slot: Slot(0), // Target enemy creature in slot 0
    }).expect("P1 should be able to play Weak Soul Harvest");

    // Tough Dummy should still be alive (had 5 health, took 1 damage)
    let creature = engine.state.players[1].get_creature(Slot(0));
    assert!(creature.is_some(), "Tough Dummy should still be alive");
    assert_eq!(creature.unwrap().current_health, 4, "Tough Dummy should have 4 health");

    // P1 should NOT have drawn a card (conditional effect didn't trigger)
    // Hand was: 1 card, -1 for playing spell = 0 cards
    let expected_hand = p1_hand_before_spell - 1; // -1 for spell, no draw
    assert_eq!(
        engine.state.players[0].hand.len(),
        expected_hand,
        "P1 should NOT have drawn a card (target survived)"
    );
}

// ========================================
// High Artificer Token Ability Tests
// ========================================

/// Test that The High Artificer's Brass Cog tokens have the Volatile Overload ability.
#[test]
fn test_high_artificer_token_has_ability() {
    // Load real card database which includes commanders
    let card_db = load_real_card_db();
    let mut engine = GameEngine::new(&card_db);

    // Use a deck that works with The High Artificer (commander 5000)
    let deck = valid_yaml_deck();
    let high_artificer = CardId(5000);

    // Start game with P1 as High Artificer
    engine.start_game_raw(
        deck.clone(),
        deck.clone(),
        high_artificer,
        high_artificer,
        42,
        GameMode::default(),
    ).unwrap();

    // At turn start, P1 should have a Brass Cog token summoned
    // Find tokens (CardId(0) is the sentinel for tokens)
    let tokens: Vec<_> = engine.state.players[0]
        .creatures
        .iter()
        .filter(|c| c.card_id == CardId(0))
        .collect();

    assert!(
        !tokens.is_empty(),
        "High Artificer should summon a Brass Cog token at StartOfTurn. Creatures: {:?}",
        engine.state.players[0].creatures.iter().map(|c| c.card_id).collect::<Vec<_>>()
    );

    // The token should have abilities
    let token = tokens[0];
    assert!(
        token.token_abilities.is_some(),
        "Brass Cog token should have token_abilities. Token: card_id={:?}, attack={}, health={}",
        token.card_id,
        token.attack,
        token.current_health
    );

    let abilities = token.token_abilities.as_ref().unwrap();
    assert_eq!(abilities.len(), 1, "Brass Cog should have exactly 1 ability");
    assert_eq!(abilities[0].name, "Volatile Overload", "Ability should be Volatile Overload");
    assert_eq!(abilities[0].essence_cost, 1, "Volatile Overload should cost 1 essence");
}

/// Test that UseAbility actions are generated for Brass Cog tokens.
#[test]
fn test_high_artificer_token_generates_use_ability() {
    let card_db = load_real_card_db();
    let mut engine = GameEngine::new(&card_db);

    let deck = valid_yaml_deck();
    let high_artificer = CardId(5000);

    engine.start_game_raw(
        deck.clone(),
        deck.clone(),
        high_artificer,
        high_artificer,
        42,
        GameMode::default(),
    ).unwrap();

    // Advance to turn 2 so the token from turn 1 isn't summoning sick
    engine.apply_action(Action::EndTurn).unwrap();
    engine.apply_action(Action::EndTurn).unwrap();

    // Now it's turn 2, P1's turn again
    // P1 should have a token that can use abilities

    // Give P1 enough essence to use the ability
    engine.state.players[0].current_essence = 5;

    let actions = engine.get_legal_actions();

    let use_ability_count = actions
        .iter()
        .filter(|a| matches!(a, Action::UseAbility { .. }))
        .count();

    println!("Legal actions: {:?}", actions.iter().map(|a| format!("{:?}", a)).collect::<Vec<_>>());

    assert!(
        use_ability_count > 0,
        "Should have UseAbility actions for Brass Cog token. Actions: {:?}",
        actions
    );
}
