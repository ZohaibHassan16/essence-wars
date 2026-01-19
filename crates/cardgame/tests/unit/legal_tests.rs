//! Unit tests for legal action generation.

use cardgame::actions::Action;
use cardgame::cards::{CardDatabase, CardDefinition, CardType};
use cardgame::effects::TargetingRule;
use cardgame::keywords::Keywords;
use cardgame::legal::{legal_action_mask, legal_actions};
use cardgame::state::{CardInstance, Creature, CreatureStatus, GameState};
use cardgame::types::{CardId, CreatureInstanceId, PlayerId, Rarity, Slot};

/// Create a test card database with some basic cards
fn test_card_db() -> CardDatabase {
    let cards = vec![
        // Basic creature: cost 2, 2/3
        CardDefinition {
            id: 1,
            name: "Test Creature".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 2,
                health: 3,
                keywords: vec![],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Expensive creature: cost 5, 4/4
        CardDefinition {
            id: 2,
            name: "Expensive Creature".to_string(),
            cost: 5,
            card_type: CardType::Creature {
                attack: 4,
                health: 4,
                keywords: vec![],
                abilities: vec![],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
        // Ranged creature: cost 3, 2/2, Ranged
        CardDefinition {
            id: 3,
            name: "Ranged Creature".to_string(),
            cost: 3,
            card_type: CardType::Creature {
                attack: 2,
                health: 2,
                keywords: vec!["Ranged".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Guard creature: cost 2, 1/4, Guard
        CardDefinition {
            id: 4,
            name: "Guard Creature".to_string(),
            cost: 2,
            card_type: CardType::Creature {
                attack: 1,
                health: 4,
                keywords: vec!["Guard".to_string()],
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Test spell: cost 1
        CardDefinition {
            id: 5,
            name: "Test Spell".to_string(),
            cost: 1,
            card_type: CardType::Spell {
                targeting: TargetingRule::NoTarget,
                effects: vec![],
                conditional_effects: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
        // Test support: cost 3
        CardDefinition {
            id: 6,
            name: "Test Support".to_string(),
            cost: 3,
            card_type: CardType::Support {
                durability: 2,
                passive_effects: vec![],
                triggered_effects: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![],
        },
    ];
    CardDatabase::new(cards)
}

/// Create a basic test creature
fn make_creature(
    card_id: u16,
    slot: u8,
    owner: PlayerId,
    turn_played: u16,
    keywords: Keywords,
) -> Creature {
    Creature {
        instance_id: CreatureInstanceId(slot as u32),
        card_id: CardId(card_id),
        owner,
        slot: Slot(slot),
        attack: 2,
        current_health: 3,
        max_health: 3,
        base_attack: 2,
        base_health: 3,
        keywords,
        status: CreatureStatus::default(),
        turn_played,
        frenzy_stacks: 0,
    }
}

/// Helper to set up resources for a player in test scenarios.
fn setup_resources(state: &mut GameState, player_idx: usize, ap: u8, essence: u8) {
    state.players[player_idx].action_points = ap;
    state.players[player_idx].max_essence = essence;
    state.players[player_idx].current_essence = essence;
}

#[test]
fn test_empty_board_only_end_turn() {
    let state = GameState::new();
    let card_db = CardDatabase::empty();

    let actions = legal_actions(&state, &card_db);

    // Only EndTurn should be legal
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], Action::EndTurn);
}

#[test]
fn test_hand_with_playable_card() {
    let card_db = test_card_db();
    let mut state = GameState::new();

    // Give player 3 AP, enough essence, and a card in hand (cost 2)
    setup_resources(&mut state, 0, 3, 10);
    state.players[0].hand.push(CardInstance::new(CardId(1)));

    let actions = legal_actions(&state, &card_db);

    // Should have PlayCard actions for all 5 empty slots + EndTurn
    assert_eq!(actions.len(), 6);

    // Verify we have PlayCard actions for slots 0-4
    let play_card_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_card_count, 5);
}

#[test]
fn test_not_enough_ap() {
    let card_db = test_card_db();
    let mut state = GameState::new();

    // Give player 1 AP and a card that costs 5
    state.players[0].action_points = 1;
    state.players[0].hand.push(CardInstance::new(CardId(2)));

    let actions = legal_actions(&state, &card_db);

    // Only EndTurn should be legal (can't afford the card)
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], Action::EndTurn);
}

#[test]
fn test_creature_attack_adjacent_targeting() {
    let card_db = test_card_db();
    let mut state = GameState::new();
    state.current_turn = 2; // Turn 2 so creatures aren't summoning sick

    // Place a creature at slot 2 (can attack slots 1, 2, 3)
    let creature = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none());
    state.players[0].creatures.push(creature);

    let actions = legal_actions(&state, &card_db);

    // Should have attacks on adjacent slots (1, 2, 3) + EndTurn
    let attack_count = actions
        .iter()
        .filter(|a| matches!(a, Action::Attack { .. }))
        .count();
    assert_eq!(attack_count, 3);

    // Verify specific attacks
    assert!(actions.contains(&Action::Attack {
        attacker: Slot(2),
        defender: Slot(1)
    }));
    assert!(actions.contains(&Action::Attack {
        attacker: Slot(2),
        defender: Slot(2)
    }));
    assert!(actions.contains(&Action::Attack {
        attacker: Slot(2),
        defender: Slot(3)
    }));

    // Should NOT be able to attack slot 0 or 4
    assert!(!actions.contains(&Action::Attack {
        attacker: Slot(2),
        defender: Slot(0)
    }));
    assert!(!actions.contains(&Action::Attack {
        attacker: Slot(2),
        defender: Slot(4)
    }));
}

#[test]
fn test_ranged_creature_any_target() {
    let card_db = test_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Place a ranged creature at slot 0
    let creature = make_creature(3, 0, PlayerId::PLAYER_ONE, 1, Keywords::none().with_ranged());
    state.players[0].creatures.push(creature);

    let actions = legal_actions(&state, &card_db);

    // Ranged should be able to attack all 5 slots
    let attack_count = actions
        .iter()
        .filter(|a| matches!(a, Action::Attack { .. }))
        .count();
    assert_eq!(attack_count, 5);

    // Verify can attack any slot
    for slot in 0..5 {
        assert!(actions.contains(&Action::Attack {
            attacker: Slot(0),
            defender: Slot(slot)
        }));
    }
}

#[test]
fn test_guard_enforcement() {
    let card_db = test_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Place an attacker at slot 2 for player 1
    let attacker = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none());
    state.players[0].creatures.push(attacker);

    // Place a guard creature at slot 1 for player 2
    let guard = make_creature(4, 1, PlayerId::PLAYER_TWO, 1, Keywords::none().with_guard());
    state.players[1].creatures.push(guard);

    // Place a non-guard creature at slot 3 for player 2
    let non_guard = make_creature(1, 3, PlayerId::PLAYER_TWO, 1, Keywords::none());
    state.players[1].creatures.push(non_guard);

    let actions = legal_actions(&state, &card_db);

    // Can only attack the guard creature
    let attacks: Vec<_> = actions
        .iter()
        .filter(|a| matches!(a, Action::Attack { .. }))
        .collect();
    assert_eq!(attacks.len(), 1);
    assert!(actions.contains(&Action::Attack {
        attacker: Slot(2),
        defender: Slot(1)
    }));

    // Cannot attack the non-guard or empty slots
    assert!(!actions.contains(&Action::Attack {
        attacker: Slot(2),
        defender: Slot(3)
    }));
    assert!(!actions.contains(&Action::Attack {
        attacker: Slot(2),
        defender: Slot(2)
    }));
}

#[test]
fn test_guard_enforcement_ranged() {
    let card_db = test_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Place a ranged attacker at slot 0 for player 1
    let attacker = make_creature(3, 0, PlayerId::PLAYER_ONE, 1, Keywords::none().with_ranged());
    state.players[0].creatures.push(attacker);

    // Place a guard creature at slot 4 for player 2
    let guard = make_creature(4, 4, PlayerId::PLAYER_TWO, 1, Keywords::none().with_guard());
    state.players[1].creatures.push(guard);

    // Place a non-guard creature at slot 1 for player 2
    let non_guard = make_creature(1, 1, PlayerId::PLAYER_TWO, 1, Keywords::none());
    state.players[1].creatures.push(non_guard);

    let actions = legal_actions(&state, &card_db);

    // Ranged can still only attack guard when guards are present
    let attacks: Vec<_> = actions
        .iter()
        .filter(|a| matches!(a, Action::Attack { .. }))
        .collect();
    assert_eq!(attacks.len(), 1);
    assert!(actions.contains(&Action::Attack {
        attacker: Slot(0),
        defender: Slot(4)
    }));
}

#[test]
fn test_summoning_sickness() {
    let card_db = test_card_db();
    let mut state = GameState::new();
    state.current_turn = 1;

    // Place a creature played this turn (has summoning sickness)
    let creature = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none());
    state.players[0].creatures.push(creature);

    let actions = legal_actions(&state, &card_db);

    // No attacks should be possible (summoning sickness)
    let attack_count = actions
        .iter()
        .filter(|a| matches!(a, Action::Attack { .. }))
        .count();
    assert_eq!(attack_count, 0);
}

#[test]
fn test_rush_ignores_summoning_sickness() {
    let card_db = test_card_db();
    let mut state = GameState::new();
    state.current_turn = 1;

    // Place a Rush creature played this turn
    let creature = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none().with_rush());
    state.players[0].creatures.push(creature);

    let actions = legal_actions(&state, &card_db);

    // Rush creature CAN attack on the turn it's played
    let attack_count = actions
        .iter()
        .filter(|a| matches!(a, Action::Attack { .. }))
        .count();
    assert!(attack_count > 0);
}

#[test]
fn test_legal_action_mask() {
    let card_db = test_card_db();
    let mut state = GameState::new();

    // Give player 3 AP, essence, and a card in hand
    setup_resources(&mut state, 0, 3, 10);
    state.players[0].hand.push(CardInstance::new(CardId(1)));

    let mask = legal_action_mask(&state, &card_db);

    // EndTurn (index 255) should be legal
    assert!(mask[255]);

    // PlayCard for hand_index=0, slots 0-4 should be legal
    // Index = hand_idx * 5 + slot
    for slot in 0..5 {
        assert!(mask[slot], "PlayCard(0, {}) should be legal", slot);
    }

    // Other PlayCard indices should be false
    assert!(!mask[5]); // hand_index=1, slot=0 (no card at index 1)
}

#[test]
fn test_spell_play_action() {
    let card_db = test_card_db();
    let mut state = GameState::new();

    // Give player AP, essence, and a spell in hand
    setup_resources(&mut state, 0, 5, 10);
    state.players[0].hand.push(CardInstance::new(CardId(5))); // Spell

    let actions = legal_actions(&state, &card_db);

    // Should have PlayCard for the spell + EndTurn
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 1); // Spells only get one PlayCard action

    assert!(actions.contains(&Action::PlayCard {
        hand_index: 0,
        slot: Slot(0)
    }));
}

#[test]
fn test_support_play_action() {
    let card_db = test_card_db();
    let mut state = GameState::new();

    // Give player AP, essence, and a support in hand
    setup_resources(&mut state, 0, 5, 10);
    state.players[0].hand.push(CardInstance::new(CardId(6))); // Support

    let actions = legal_actions(&state, &card_db);

    // Should have PlayCard for 2 support slots + EndTurn
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 2); // Two support slots

    assert!(actions.contains(&Action::PlayCard {
        hand_index: 0,
        slot: Slot(0)
    }));
    assert!(actions.contains(&Action::PlayCard {
        hand_index: 0,
        slot: Slot(1)
    }));
}

#[test]
fn test_occupied_creature_slot() {
    let card_db = test_card_db();
    let mut state = GameState::new();

    // Give player AP, essence, and a creature card in hand
    setup_resources(&mut state, 0, 5, 10);
    state.players[0].hand.push(CardInstance::new(CardId(1))); // Creature

    // Occupy slots 0 and 2
    let creature1 = make_creature(1, 0, PlayerId::PLAYER_ONE, 0, Keywords::none());
    let creature2 = make_creature(1, 2, PlayerId::PLAYER_ONE, 0, Keywords::none());
    state.players[0].creatures.push(creature1);
    state.players[0].creatures.push(creature2);

    let actions = legal_actions(&state, &card_db);

    // Should only be able to play to empty slots 1, 3, 4
    let play_actions: Vec<_> = actions
        .iter()
        .filter_map(|a| match a {
            Action::PlayCard { hand_index, slot } => Some((*hand_index, slot.0)),
            _ => None,
        })
        .collect();

    assert_eq!(play_actions.len(), 3);
    assert!(play_actions.contains(&(0, 1)));
    assert!(play_actions.contains(&(0, 3)));
    assert!(play_actions.contains(&(0, 4)));
}

#[test]
fn test_exhausted_creature_cannot_attack() {
    let card_db = test_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Place an exhausted creature
    let mut creature = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none());
    creature.status.set_exhausted(true);
    state.players[0].creatures.push(creature);

    let actions = legal_actions(&state, &card_db);

    // No attacks should be possible (exhausted)
    let attack_count = actions
        .iter()
        .filter(|a| matches!(a, Action::Attack { .. }))
        .count();
    assert_eq!(attack_count, 0);
}

#[test]
fn test_terminal_state_no_actions() {
    let card_db = test_card_db();
    let mut state = GameState::new();

    // Mark game as ended
    state.result = Some(cardgame::state::GameResult::Draw);

    let actions = legal_actions(&state, &card_db);

    // No actions should be legal when game is over
    assert_eq!(actions.len(), 0);
}

#[test]
fn test_multiple_cards_in_hand() {
    let card_db = test_card_db();
    let mut state = GameState::new();

    // Give player 5 AP, 10 essence, and two cards in hand
    setup_resources(&mut state, 0, 5, 10);
    state.players[0].hand.push(CardInstance::new(CardId(1))); // cost 2 creature
    state.players[0].hand.push(CardInstance::new(CardId(2))); // cost 5 creature

    let actions = legal_actions(&state, &card_db);

    // Both cards can be played (5 AP available, 10 essence covers both costs)
    // Card 0: 5 slots, Card 1: 5 slots = 10 PlayCard + EndTurn
    let play_count = actions
        .iter()
        .filter(|a| matches!(a, Action::PlayCard { .. }))
        .count();
    assert_eq!(play_count, 10);
}

#[test]
fn test_attack_empty_slot_face_damage() {
    let card_db = test_card_db();
    let mut state = GameState::new();
    state.current_turn = 2;

    // Place a creature that can attack
    let creature = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none());
    state.players[0].creatures.push(creature);

    // No enemy creatures on board

    let actions = legal_actions(&state, &card_db);

    // Should be able to attack empty slots (for face damage)
    let attack_count = actions
        .iter()
        .filter(|a| matches!(a, Action::Attack { .. }))
        .count();
    assert_eq!(attack_count, 3); // Adjacent slots 1, 2, 3
}
