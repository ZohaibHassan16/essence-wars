//! Detailed trace to understand Lifesteal combat interactions.
//!
//! Updated 2026-01-14 to use new Core Set (Obsidion has Lifesteal creatures).

use cardgame::actions::Action;
use cardgame::bots::{Bot, GreedyBot};
use cardgame::cards::CardDatabase;
use cardgame::decks::DeckRegistry;
use cardgame::engine::GameEngine;
use cardgame::types::{CardId, PlayerId};

#[test]
fn trace_single_game_detailed() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory(cardgame::data_dir().join("decks")).expect("Failed to load decks");

    // Use Obsidion deck which has Lifesteal creatures
    let obsidion_deck = deck_registry.get("archon_burst").expect("Deck should exist");
    let aggressive_deck = deck_registry.get("broodmother_swarm").expect("Deck should exist");

    let deck1_cards: Vec<CardId> = obsidion_deck.cards.iter().map(|&id| CardId(id)).collect();
    let deck2_cards: Vec<CardId> = aggressive_deck.cards.iter().map(|&id| CardId(id)).collect();

    println!("\n=== DETAILED SINGLE GAME TRACE ===\n");

    // Create engine directly to observe state
    let mut engine = GameEngine::new(&card_db);
    engine.start_game(deck1_cards.clone(), deck2_cards.clone(), 42);

    let mut bot1 = GreedyBot::new(&card_db, 42);
    let mut bot2 = GreedyBot::new(&card_db, 43);

    let mut action_count = 0;
    let max_actions = 500;

    while !engine.is_game_over() && action_count < max_actions {
        let current_player = engine.current_player();
        let turn = engine.turn_number();

        // Get state info
        let p1_creatures: Vec<_> = engine.state.players[0].creatures.iter()
            .map(|c| {
                let card = card_db.get(c.card_id).unwrap();
                (c.slot.0, card.name.as_str(), card.keywords().to_names())
            })
            .collect();

        let p1_hand: Vec<_> = engine.state.players[0].hand.iter()
            .map(|inst| {
                let card = card_db.get(inst.card_id).unwrap();
                (card.name.as_str(), card.cost, card.keywords().to_names())
            })
            .collect();

        // Check for Lifesteal in hand or on board
        let lifesteal_in_hand = p1_hand.iter()
            .any(|(_, _, kws)| kws.contains(&"Lifesteal"));
        let lifesteal_on_board = p1_creatures.iter()
            .any(|(_, _, kws)| kws.contains(&"Lifesteal"));

        // Get bot action
        let state_tensor = engine.get_state_tensor();
        let legal_mask = engine.get_legal_action_mask();
        let legal_actions = engine.get_legal_actions();

        let action = if current_player == PlayerId::PLAYER_ONE {
            bot1.select_action(&state_tensor, &legal_mask, &legal_actions)
        } else {
            bot2.select_action(&state_tensor, &legal_mask, &legal_actions)
        };

        // Log interesting moments
        if current_player == PlayerId::PLAYER_ONE {
            if lifesteal_in_hand || lifesteal_on_board {
                println!("Turn {} (P1, {} essence): LS_hand={} LS_board={}",
                    turn,
                    engine.state.players[0].current_essence,
                    lifesteal_in_hand,
                    lifesteal_on_board);

                // Log action
                match action {
                    Action::PlayCard { hand_index, slot } => {
                        if (hand_index as usize) < p1_hand.len() {
                            let (name, cost, kws) = &p1_hand[hand_index as usize];
                            println!("  -> Playing {} (cost {}) to slot {} {:?}",
                                name, cost, slot.0, kws);
                        }
                    }
                    Action::Attack { attacker, defender } => {
                        println!("  -> Attack from slot {} to slot {}", attacker.0, defender.0);
                        // Check if attacker has Lifesteal
                        for (slot, name, kws) in &p1_creatures {
                            if *slot == attacker.0 {
                                println!("     Attacker: {} {:?}", name, kws);
                            }
                        }
                    }
                    Action::EndTurn => {
                        println!("  -> End Turn");
                        // Print board state
                        println!("     P1 board: {:?}", p1_creatures);
                    }
                    _ => {}
                }
            }
        }

        engine.apply_action(action).expect("Action should succeed");
        action_count += 1;
    }

    println!("\n=== GAME ENDED ===");
    println!("Winner: {:?}", engine.winner());
    println!("Turns: {}", engine.turn_number());
    println!("Total actions: {}", action_count);

    // Final board state
    println!("\nFinal P1 creatures:");
    for c in &engine.state.players[0].creatures {
        // Handle tokens (CardId(0)) which don't exist in card database
        if c.card_id.0 == 0 {
            println!("  Slot {}: Token ({}/{}) {:?}",
                c.slot.0, c.attack, c.current_health, c.keywords.to_names());
        } else if let Some(card) = card_db.get(c.card_id) {
            println!("  Slot {}: {} ({}/{}) {:?}",
                c.slot.0, card.name, c.attack, c.current_health, card.keywords().to_names());
        } else {
            println!("  Slot {}: Unknown[{}] ({}/{}) {:?}",
                c.slot.0, c.card_id.0, c.attack, c.current_health, c.keywords.to_names());
        }
    }

    println!("\nFinal P2 creatures:");
    for c in &engine.state.players[1].creatures {
        // Handle tokens (CardId(0)) which don't exist in card database
        if c.card_id.0 == 0 {
            println!("  Slot {}: Token ({}/{}) {:?}",
                c.slot.0, c.attack, c.current_health, c.keywords.to_names());
        } else if let Some(card) = card_db.get(c.card_id) {
            println!("  Slot {}: {} ({}/{}) {:?}",
                c.slot.0, card.name, c.attack, c.current_health, card.keywords().to_names());
        } else {
            println!("  Slot {}: Unknown[{}] ({}/{}) {:?}",
                c.slot.0, c.card_id.0, c.attack, c.current_health, c.keywords.to_names());
        }
    }
}

#[test]
fn check_vampire_lord_in_starting_hands() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory(cardgame::data_dir().join("decks")).expect("Failed to load decks");

    // Use Obsidion deck which has Lifesteal creatures
    let obsidion_deck = deck_registry.get("archon_burst").expect("Deck should exist");
    let aggressive_deck = deck_registry.get("broodmother_swarm").expect("Deck should exist");

    let deck1_cards: Vec<CardId> = obsidion_deck.cards.iter().map(|&id| CardId(id)).collect();
    let deck2_cards: Vec<CardId> = aggressive_deck.cards.iter().map(|&id| CardId(id)).collect();

    println!("\n=== CHECKING STARTING HANDS FOR LIFESTEAL ===\n");

    let mut games_with_lifesteal_start = 0;

    for seed in 0..100 {
        let mut engine = GameEngine::new(&card_db);
        engine.start_game(deck1_cards.clone(), deck2_cards.clone(), seed);

        // Check P1's starting hand
        let has_lifesteal = engine.state.players[0].hand.iter()
            .any(|inst| {
                let card = card_db.get(inst.card_id).unwrap();
                card.keywords().has_lifesteal()
            });

        if has_lifesteal {
            games_with_lifesteal_start += 1;
            if games_with_lifesteal_start <= 5 {
                println!("Seed {}: Starting hand has Lifesteal", seed);
                for inst in &engine.state.players[0].hand {
                    let card = card_db.get(inst.card_id).unwrap();
                    println!("  - {} (cost {}) {:?}",
                        card.name, card.cost, card.keywords().to_names());
                }
            }
        }
    }

    println!("\nGames with Lifesteal in starting hand: {}/100", games_with_lifesteal_start);

    // Also check: can we even draw Lifesteal cards?
    println!("\n=== DECK COMPOSITION CHECK ===");
    let mut total_cards = 0;
    let mut lifesteal_cards = 0;
    for &id in &obsidion_deck.cards {
        let card = card_db.get(CardId(id)).unwrap();
        total_cards += 1;
        if card.keywords().has_lifesteal() {
            lifesteal_cards += 1;
            println!("Lifesteal card: {} (ID {}, cost {})", card.name, id, card.cost);
        }
    }
    println!("Total Lifesteal cards in deck: {}/{}", lifesteal_cards, total_cards);

    // Should have some Lifesteal cards in the deck
    assert!(lifesteal_cards > 0, "Obsidion deck should have Lifesteal creatures");
}
