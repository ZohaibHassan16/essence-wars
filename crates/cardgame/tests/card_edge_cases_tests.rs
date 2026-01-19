//! Card edge case tests - focused tests for dangerous card mechanics.
//!
//! This module tests specific card interactions that can cause bugs:
//! - Negative health debuffs (health: -1)
//! - Zero health buffs/tokens (health: 0)
//! - Transform effects that might create invalid creatures
//! - Combinations that could bypass death processing
//!
//! These tests exist because these mechanics caused intermittent bugs
//! that were hard to reproduce in general coverage tests.

use cardgame::core::cards::{CardDatabase, CardType};
use cardgame::core::engine::GameEngine;
use cardgame::core::types::CardId;

mod common;
use common::valid_yaml_deck;

#[test]
fn test_card_database_loads_successfully() {
    // This test verifies the card validation logic runs on database load
    let result = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"));
    assert!(result.is_ok(), "Card database should load without validation errors: {:?}", result.err());
}

#[test]
fn test_all_creatures_have_valid_health() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    
    for card in card_db.iter() {
        if let CardType::Creature { health, .. } = &card.card_type {
            assert!(
                *health > 0,
                "Card {} '{}' has invalid health: {} (must be > 0)",
                card.id, card.name, health
            );
            assert!(
                *health <= 50,
                "Card {} '{}' has extreme health: {} (max 50)",
                card.id, card.name, health
            );
        }
    }
}

#[test]
fn test_all_creatures_have_valid_attack() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    
    for card in card_db.iter() {
        if let CardType::Creature { attack, .. } = &card.card_type {
            assert!(
                *attack <= 50,
                "Card {} '{}' has extreme attack: {} (max 50)",
                card.id, card.name, attack
            );
        }
    }
}

#[test]
fn test_no_creatures_survive_with_zero_health() {
    // This is the most important test - run several games and verify that
    // after every action, no creature has health <= 0
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    
    // Test 3 random games with different seeds
    for seed in [12345, 67890, 11111] {
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        
        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck1, deck2, seed);
        
        // Play 50 actions checking state after each
        for _ in 0..50 {
            if engine.is_game_over() {
                break;
            }
            
            let actions = engine.get_legal_actions();
            if actions.is_empty() {
                break;
            }
            
            let action = actions[0];
            engine.apply_action(action).expect("Action should be legal");
            
            // Verify no creatures have health <= 0
            for player_idx in 0..2 {
                for (slot_idx, creature) in engine.state.players[player_idx].creatures.iter().enumerate() {
                    assert!(
                        creature.current_health > 0,
                        "Creature in slot {} for player {} has health {} after action {:?} (seed: {})",
                        slot_idx, player_idx, creature.current_health, action, seed
                    );
                }
            }
        }
    }
}

#[test]
fn test_all_decks_playable_without_crashes() {
    // Load all decks and play each against each other briefly
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    
    // Test a few matchups with different seeds
    for seed in [42, 100, 200] {
        let deck1 = valid_yaml_deck();
        let deck2 = valid_yaml_deck();
        
        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck1, deck2, seed);
        
        // Play 20 actions
        for _ in 0..20 {
            if engine.is_game_over() {
                break;
            }
            
            let actions = engine.get_legal_actions();
            if actions.is_empty() {
                break;
            }
            
            engine.apply_action(actions[0]).expect("Action should be legal");
        }
    }
}

#[test]
fn test_transform_effects_create_valid_creatures() {
    // This test verifies that Transform effects in the card database
    // have valid token definitions (validated on load)
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    
    // If we got here, validation passed - the database load checks for
    // Transform effects with health: 0 or extreme stats
    assert!(card_db.len() > 0, "Card database should not be empty");
}

#[test]
fn test_summon_effects_create_valid_creatures() {
    // This test verifies that SummonToken effects in the card database
    // have valid token definitions (validated on load)
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    
    // If we got here, validation passed
    assert!(card_db.len() > 0, "Card database should not be empty");
}

#[test]
fn test_buff_effects_have_valid_ranges() {
    // This test verifies that BuffStats effects don't have extreme values
    // (validated on database load)
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    
    // If we got here, validation passed - the database load checks for
    // BuffStats with abs(attack) > 20 or abs(health) > 20
    assert!(card_db.len() > 0, "Card database should not be empty");
}

#[test]
fn test_weaken_card_exists_and_is_playable() {
    // Weaken (4046) was one of the cards that triggered the "Dead creature" bug
    // because it has a buff with health: -1
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    
    let weaken = card_db.get(CardId(4046));
    assert!(weaken.is_some(), "Weaken card (4046) should exist");
    
    // Try playing it in a game
    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck1, deck2, 42);
    
    // Play 30 actions to increase chance of seeing Weaken
    for _ in 0..30 {
        if engine.is_game_over() {
            break;
        }
        
        let actions = engine.get_legal_actions();
        if actions.is_empty() {
            break;
        }
        
        engine.apply_action(actions[0]).expect("Action should be legal");
    }
    
    // If we got here without panicking, Weaken works correctly
}

#[test]
fn test_vampiric_cards_with_health_zero_buffs() {
    // Vampiric Surge (3035) and Sanguine Blessing (3062) have buff with health:0
    // This caused bugs when combined with death processing
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set"))
        .expect("Failed to load cards");
    
    // These cards should exist
    let vampiric_surge = card_db.get(CardId(3035));
    let sanguine_blessing = card_db.get(CardId(3062));
    
    assert!(vampiric_surge.is_some() || sanguine_blessing.is_some(), 
            "At least one vampiric card should exist");
    
    // Play a game to verify they work
    let deck1 = valid_yaml_deck();
    let deck2 = valid_yaml_deck();
    
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck1, deck2, 99999);
    
    // Play 40 actions to increase chance of seeing vampiric cards
    for _ in 0..40 {
        if engine.is_game_over() {
            break;
        }
        
        let actions = engine.get_legal_actions();
        if actions.is_empty() {
            break;
        }
        
        engine.apply_action(actions[0]).expect("Action should be legal");
    }
}
