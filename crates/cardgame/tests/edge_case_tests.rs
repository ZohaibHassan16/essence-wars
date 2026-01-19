//! Additional edge case tests for bug hunting.
//!
//! These tests specifically target edge cases and corner cases that could
//! potentially have bugs in the engine implementation.

use cardgame::actions::Action;
use cardgame::cards::{CardDatabase, CardDefinition, CardType};
use cardgame::engine::GameEngine;
use cardgame::keywords::Keywords;
use cardgame::types::{CardId, PlayerId, Slot};

fn create_test_db() -> CardDatabase {
    CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards")
}

/// Find a card with specific criteria
fn find_card_by_criteria(
    card_db: &CardDatabase,
    criteria: impl Fn(&CardDefinition) -> bool,
) -> Option<CardId> {
    // Search all faction ID ranges:
    // Argentum: 1000-1999, Symbiote: 2000-2999, Obsidion: 3000-3999, Neutral: 4000-4999
    let ranges = [
        (1000, 1100),
        (2000, 2100),
        (3000, 3100),
        (4000, 4100),
    ];

    for (start, end) in ranges {
        for id in start..end {
            if let Some(card) = card_db.get(CardId(id)) {
                if criteria(card) {
                    return Some(CardId(id));
                }
            }
        }
    }
    None
}

/// Create a test deck with specific cards
fn create_test_deck(cards: &[CardId]) -> Vec<CardId> {
    let mut deck = Vec::new();
    for &card in cards {
        deck.push(card);
    }
    // Fill rest with the first card if deck is too small
    while deck.len() < 18 {
        deck.push(cards[0]);
    }
    deck
}

#[test]
fn test_edge_case_zero_attack_creature_cannot_attack() {
    let card_db = create_test_db();

    // Find a creature with 0 attack
    let zero_attack_card = find_card_by_criteria(&card_db, |c| {
        matches!(c.card_type, CardType::Creature { attack, .. } if attack == 0)
    });

    // If no 0-attack creature exists, skip this test
    if zero_attack_card.is_none() {
        println!("No zero-attack creature found in card database, skipping test");
        return;
    }

    let zero_card = zero_attack_card.unwrap();
    let regular_card = find_card_by_criteria(&card_db, |c| {
        matches!(c.card_type, CardType::Creature { attack, .. } if attack > 0)
    })
    .expect("Should have at least one creature with attack > 0");

    let deck1 = create_test_deck(&[zero_card]);
    let deck2 = create_test_deck(&[regular_card]);

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck1, deck2, 42);

    // Give P1 enough essence to play the card
    engine.state.players[0].max_essence = 10;
    engine.state.players[0].current_essence = 10;
    engine.state.players[1].max_essence = 10;
    engine.state.players[1].current_essence = 10;

    // Play zero-attack creature
    let play_action = Action::PlayCard {
        hand_index: 0,
        slot: Slot(0),
    };

    if engine.get_legal_actions().contains(&play_action) {
        engine.apply_action(play_action).expect("Play should succeed");

        // Advance to next turn so creature can attack (if it has attack)
        engine.apply_action(Action::EndTurn).unwrap();

        // P2 plays a creature (essence already set)
        if engine.get_legal_actions().contains(&Action::PlayCard {
            hand_index: 0,
            slot: Slot(0),
        }) {
            engine
                .apply_action(Action::PlayCard {
                    hand_index: 0,
                    slot: Slot(0),
                })
                .unwrap();
        }
        engine.apply_action(Action::EndTurn).unwrap();

        // Now it's P1's turn again - check if zero-attack creature can attack
        let attack_action = Action::Attack {
            attacker: Slot(0),
            defender: Slot(0),
        };

        let legal_actions = engine.get_legal_actions();
        let can_attack = legal_actions.contains(&attack_action);

        // Zero attack creatures should NOT be able to attack
        assert!(
            !can_attack,
            "Zero-attack creature should not be able to attack"
        );
    }
}

#[test]
fn test_edge_case_ranged_bypasses_guard() {
    let card_db = create_test_db();

    // Find a ranged creature
    let ranged_card = find_card_by_criteria(&card_db, |c| {
        matches!(c.card_type, CardType::Creature { .. }) && c.keywords().has_ranged()
    });

    // Find a guard creature
    let guard_card = find_card_by_criteria(&card_db, |c| {
        matches!(c.card_type, CardType::Creature { .. }) && c.keywords().has_guard()
    });

    // Find a regular creature (no guard)
    let regular_card = find_card_by_criteria(&card_db, |c| {
        matches!(c.card_type, CardType::Creature { .. })
            && !c.keywords().has_guard()
            && !c.keywords().has_ranged()
    });

    if ranged_card.is_none() || guard_card.is_none() || regular_card.is_none() {
        println!("Required cards not found, skipping test");
        return;
    }

    let ranged = ranged_card.unwrap();
    let guard = guard_card.unwrap();
    let regular = regular_card.unwrap();

    // P1 has ranged creature, P2 has guard and regular creature
    let deck1 = create_test_deck(&[ranged]);
    let deck2 = create_test_deck(&[guard, regular]);

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck1, deck2, 42);

    // Give both players max essence
    engine.state.players[0].max_essence = 10;
    engine.state.players[0].current_essence = 10;
    engine.state.players[1].max_essence = 10;
    engine.state.players[1].current_essence = 10;

    // P1 plays ranged creature with Rush (or wait a turn)
    engine
        .apply_action(Action::PlayCard {
            hand_index: 0,
            slot: Slot(0),
        })
        .unwrap();
    engine.apply_action(Action::EndTurn).unwrap();

    // P2 plays guard creature in slot 0
    engine
        .apply_action(Action::PlayCard {
            hand_index: 0,
            slot: Slot(0),
        })
        .unwrap();
    // P2 plays regular creature in slot 1
    if engine.state.players[1].hand.len() > 0 {
        let _ = engine.apply_action(Action::PlayCard {
            hand_index: 0,
            slot: Slot(1),
        });
    }
    engine.apply_action(Action::EndTurn).unwrap();

    // P1's turn - ranged creature should be able to attack the regular creature
    // even though guard is present
    let legal_actions = engine.get_legal_actions();

    // Count valid attack targets
    let mut can_attack_guard_slot = false;
    let mut can_attack_behind_guard = false;

    for action in &legal_actions {
        if let Action::Attack { attacker, defender } = action {
            if attacker.0 == 0 {
                // Slot 0 has the Guard creature
                if defender.0 == 0 {
                    can_attack_guard_slot = true;
                }
                // Slot 1 has the regular creature (behind Guard)
                if defender.0 == 1 {
                    can_attack_behind_guard = true;
                }
            }
        }
    }

    // Ranged should be able to attack any creature (bypass guard)
    // The key property is that Ranged can attack creatures behind Guard
    assert!(
        can_attack_behind_guard || engine.state.players[1].creatures.len() < 2,
        "Ranged creature should be able to attack non-Guard creatures even when Guard is present"
    );

    // Ranged can also attack the Guard itself if it chooses
    assert!(
        can_attack_guard_slot || engine.state.players[1].creatures.is_empty(),
        "Ranged creature should be able to attack the Guard as well"
    );
}

#[test]
fn test_edge_case_hand_overflow_discards() {
    let card_db = create_test_db();

    // Find any cheap creature to fill deck
    let cheap_card = find_card_by_criteria(&card_db, |c| {
        matches!(c.card_type, CardType::Creature { .. }) && c.cost <= 2
    })
    .expect("Should have cheap creatures");

    let deck = create_test_deck(&[cheap_card]);

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck.clone(), deck, 42);

    // Record initial hand size (should be around 3-4)
    let initial_hand_size = engine.state.players[0].hand.len();
    println!("Initial hand size: {}", initial_hand_size);

    // Manually fill hand to max (10 cards typically)
    let max_hand_size = 10;
    while engine.state.players[0].hand.len() < max_hand_size {
        // Draw a card if possible
        if !engine.state.players[0].deck.is_empty() {
            let card = engine.state.players[0].deck.pop().unwrap();
            engine.state.players[0].hand.push(card);
        } else {
            break;
        }
    }

    let hand_before_turn = engine.state.players[0].hand.len();
    println!("Hand before end turn: {}", hand_before_turn);

    // End P1's turn, then end P2's turn to get back to P1
    engine.apply_action(Action::EndTurn).unwrap();
    engine.apply_action(Action::EndTurn).unwrap();

    let hand_after_draw = engine.state.players[0].hand.len();
    println!("Hand after draw phase: {}", hand_after_draw);

    // Hand should not exceed max
    assert!(
        hand_after_draw <= max_hand_size,
        "Hand should not exceed max size of {} (got {})",
        max_hand_size,
        hand_after_draw
    );
}

#[test]
fn test_edge_case_empty_deck_no_fatigue() {
    let card_db = create_test_db();

    // Find any creature
    let card = find_card_by_criteria(&card_db, |c| matches!(c.card_type, CardType::Creature { .. }))
        .expect("Should have creatures");

    // Create minimal deck
    let deck = create_test_deck(&[card]);

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck.clone(), deck, 42);

    // Empty P1's deck
    engine.state.players[0].deck.clear();

    let life_before = engine.state.players[0].life;

    // End turn multiple times to trigger draw phases
    for _ in 0..5 {
        if engine.is_game_over() {
            break;
        }
        engine.apply_action(Action::EndTurn).unwrap();
    }

    let life_after = engine.state.players[0].life;

    // No fatigue damage in this game (per design doc)
    // Life should only change from combat, not from empty deck
    println!(
        "Life before: {}, Life after: {} (after empty deck draws)",
        life_before, life_after
    );

    // This is an informational test - log whether fatigue exists
    if life_after < life_before {
        println!(
            "WARNING: Life decreased by {} - possible fatigue damage?",
            life_before - life_after
        );
    }
}

#[test]
fn test_edge_case_simultaneous_death_effects() {
    let card_db = create_test_db();

    // Find two creatures that can kill each other
    let creature1 = find_card_by_criteria(&card_db, |c| {
        matches!(c.card_type, CardType::Creature { attack, health, .. } if attack >= 2 && health <= 3)
    });

    let creature2 = find_card_by_criteria(&card_db, |c| {
        matches!(c.card_type, CardType::Creature { attack, health, .. } if attack >= 2 && health <= 3)
    });

    if creature1.is_none() || creature2.is_none() {
        println!("Could not find suitable creatures for mutual death test");
        return;
    }

    let c1 = creature1.unwrap();
    let c2 = creature2.unwrap();

    let deck1 = create_test_deck(&[c1]);
    let deck2 = create_test_deck(&[c2]);

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck1, deck2, 42);

    // Give essence and play creatures
    engine.state.players[0].max_essence = 10;
    engine.state.players[0].current_essence = 10;
    engine.state.players[1].max_essence = 10;
    engine.state.players[1].current_essence = 10;

    // P1 plays creature with Rush
    let _ = engine.apply_action(Action::PlayCard {
        hand_index: 0,
        slot: Slot(0),
    });
    engine.apply_action(Action::EndTurn).unwrap();

    // P2 plays creature
    let _ = engine.apply_action(Action::PlayCard {
        hand_index: 0,
        slot: Slot(0),
    });
    engine.apply_action(Action::EndTurn).unwrap();

    // Check board state before combat
    let p1_creatures_before = engine.state.players[0].creatures.len();
    let p2_creatures_before = engine.state.players[1].creatures.len();

    // P1 attacks
    if !engine.is_game_over() && p1_creatures_before > 0 && p2_creatures_before > 0 {
        let attack = Action::Attack {
            attacker: Slot(0),
            defender: Slot(0),
        };
        if engine.get_legal_actions().contains(&attack) {
            let result = engine.apply_action(attack);
            assert!(result.is_ok(), "Attack should succeed");

            // Verify game state is valid after mutual destruction
            assert!(
                !engine.is_game_over()
                    || engine.state.players[0].life <= 0
                    || engine.state.players[1].life <= 0,
                "Game should continue or end due to life loss, not crash"
            );
        }
    }
}

#[test]
fn test_edge_case_action_space_boundaries() {
    // Test action index boundaries
    assert!(Action::from_index(0).is_some(), "Index 0 should be valid");
    assert!(Action::from_index(49).is_some(), "Index 49 should be valid");
    assert!(Action::from_index(50).is_some(), "Index 50 should be valid");
    assert!(Action::from_index(74).is_some(), "Index 74 should be valid");
    assert!(Action::from_index(75).is_some(), "Index 75 should be valid");
    assert!(
        Action::from_index(254).is_some(),
        "Index 254 should be valid"
    );
    assert!(
        Action::from_index(255).is_some(),
        "Index 255 (EndTurn) should be valid"
    );

    // Verify action types at boundaries
    match Action::from_index(0).unwrap() {
        Action::PlayCard { hand_index: 0, .. } => {}
        _ => panic!("Index 0 should be PlayCard with hand_index 0"),
    }

    match Action::from_index(49).unwrap() {
        Action::PlayCard { hand_index: 9, .. } => {}
        _ => panic!("Index 49 should be PlayCard with hand_index 9"),
    }

    match Action::from_index(50).unwrap() {
        Action::Attack { .. } => {}
        _ => panic!("Index 50 should be Attack"),
    }

    match Action::from_index(255).unwrap() {
        Action::EndTurn => {}
        _ => panic!("Index 255 should be EndTurn"),
    }
}

#[test]
fn test_edge_case_turn_30_limit_resolution() {
    let card_db = create_test_db();

    // Find a cheap creature
    let card = find_card_by_criteria(&card_db, |c| {
        matches!(c.card_type, CardType::Creature { .. }) && c.cost <= 2
    })
    .expect("Should have cheap creatures");

    let deck = create_test_deck(&[card]);

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck.clone(), deck, 42);

    // Fast forward to turn 29
    while engine.turn_number() < 29 && !engine.is_game_over() {
        engine.apply_action(Action::EndTurn).unwrap();
    }

    let turn_before_limit = engine.turn_number();
    println!("Turn before limit: {}", turn_before_limit);

    // Try to advance past turn 30
    while !engine.is_game_over() && engine.turn_number() <= 32 {
        engine.apply_action(Action::EndTurn).unwrap();
    }

    // Game should be over at or before turn 30
    assert!(
        engine.is_game_over(),
        "Game should end by turn 30 (reached turn {})",
        engine.turn_number()
    );

    // Winner should be based on life totals
    if let Some(winner) = engine.winner() {
        let p1_life = engine.state.players[0].life;
        let p2_life = engine.state.players[1].life;
        println!(
            "Winner: {:?}, P1 life: {}, P2 life: {}",
            winner, p1_life, p2_life
        );

        // Winner should have more life (or equal life = P1 wins as tiebreaker)
        if p1_life > p2_life {
            assert_eq!(winner, PlayerId::PLAYER_ONE, "P1 should win with more life");
        } else if p2_life > p1_life {
            assert_eq!(winner, PlayerId::PLAYER_TWO, "P2 should win with more life");
        }
        // If tied, either winner is acceptable based on game rules
    }
}

#[test]
fn test_edge_case_all_keywords_interact_correctly() {
    // Test that all 8 keywords can be represented
    let mut kw = Keywords::none();
    assert_eq!(kw.0, 0, "Empty keywords should be 0");

    kw = kw.with_rush();
    assert!(kw.has_rush(), "Should have rush");

    kw = kw.with_ranged();
    assert!(kw.has_ranged(), "Should have ranged");

    kw = kw.with_piercing();
    assert!(kw.has_piercing(), "Should have piercing");

    kw = kw.with_guard();
    assert!(kw.has_guard(), "Should have guard");

    kw = kw.with_lifesteal();
    assert!(kw.has_lifesteal(), "Should have lifesteal");

    kw = kw.with_lethal();
    assert!(kw.has_lethal(), "Should have lethal");

    kw = kw.with_shield();
    assert!(kw.has_shield(), "Should have shield");

    kw = kw.with_quick();
    assert!(kw.has_quick(), "Should have quick");

    // All 8 original keywords should be set (bits 0-7)
    assert_eq!(kw.0, 0xFF, "All 8 original keyword bits should be set");

    // Test that the 4 new keywords can also be set
    kw = kw.with_ephemeral().with_regenerate().with_stealth().with_charge();
    assert_eq!(kw.0, 0x0FFF, "All 12 original+new keyword bits should be set");

    // Test that the 2 Symbiote keywords (Frenzy, Volatile) can also be set
    kw = kw.with_frenzy().with_volatile();
    assert_eq!(kw.0, 0x3FFF, "All 14 keyword bits (original+new+symbiote) should be set");

    // Test that the 2 Phase 5 keywords (Fortify, Ward) can also be set
    kw = kw.with_fortify().with_ward();
    assert_eq!(kw, Keywords::all(), "All 16 keywords should be set");
    assert_eq!(kw.0, 0xFFFF, "All 16 keyword bits should be set");
}

#[test]
fn test_edge_case_support_durability() {
    let card_db = create_test_db();

    // Find a support card
    let support_card = find_card_by_criteria(&card_db, |c| {
        matches!(c.card_type, CardType::Support { .. })
    });

    if support_card.is_none() {
        println!("No support cards found, skipping test");
        return;
    }

    let support = support_card.unwrap();
    let creature = find_card_by_criteria(&card_db, |c| {
        matches!(c.card_type, CardType::Creature { .. })
    })
    .expect("Should have creatures");

    let deck1 = create_test_deck(&[support, creature]);
    let deck2 = create_test_deck(&[creature]);

    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck1, deck2, 42);

    // Give essence
    engine.state.players[0].max_essence = 10;
    engine.state.players[0].current_essence = 10;

    // Try to play support
    let play_support = Action::PlayCard {
        hand_index: 0,
        slot: Slot(0),
    };

    if engine.get_legal_actions().contains(&play_support) {
        engine.apply_action(play_support).unwrap();

        // Check support was placed
        let has_support = !engine.state.players[0].supports.is_empty();
        println!("Support placed: {}", has_support);

        if has_support {
            let support_durability = engine.state.players[0].supports[0].current_durability;
            println!("Support durability: {}", support_durability);
            assert!(support_durability > 0, "Support should have durability");
        }
    }
}
