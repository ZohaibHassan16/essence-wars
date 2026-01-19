//! Lifesteal verification tests.
//!
//! These tests verify that Lifesteal creatures are properly played and enter combat.
//! Originally created to debug an issue where Lifesteal never appeared in combat
//! (which was caused by the Essence system not being implemented, fixed 2026-01-12).
//!
//! Updated 2026-01-14 to use new Core Set (Obsidion has Lifesteal creatures).

use cardgame::arena::GameRunner;
use cardgame::bots::{GreedyBot, RandomBot};
use cardgame::cards::CardDatabase;
use cardgame::decks::DeckRegistry;
use cardgame::types::{CardId, PlayerId};

#[test]
fn debug_lifesteal_investigation() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory(cardgame::data_dir().join("decks")).expect("Failed to load decks");

    // Verify Lifesteal cards exist in Obsidion faction
    println!("\n=== CARD DATABASE CHECK ===");
    // Obsidion Lifesteal cards: Blood Acolyte (3002), Hemomancer (3004), The Eternal One (3007)
    let blood_acolyte = card_db.get(CardId(3002));
    let hemomancer = card_db.get(CardId(3004));
    let eternal_one = card_db.get(CardId(3007));

    println!("Card 3002 (Blood Acolyte): {:?}", blood_acolyte.map(|c| (c.name.as_str(), c.keywords())));
    println!("Card 3004 (Hemomancer): {:?}", hemomancer.map(|c| (c.name.as_str(), c.keywords())));
    println!("Card 3007 (The Eternal One): {:?}", eternal_one.map(|c| (c.name.as_str(), c.keywords())));

    // Verify archon_burst deck has Lifesteal
    println!("\n=== DECK CHECK ===");
    let obsidion_deck = deck_registry.get("archon_burst").expect("Deck should exist");
    println!("Obsidion Burst deck cards: {:?}", obsidion_deck.cards);

    let lifesteal_in_deck = obsidion_deck.cards.iter()
        .filter(|&&id| id == 3002 || id == 3004 || id == 3007)
        .count();
    println!("Lifesteal cards in deck: {}", lifesteal_in_deck);

    // Run games and track what's happening
    println!("\n=== RUNNING 100 GAMES WITH TRACING ===");

    let deck1_cards: Vec<CardId> = obsidion_deck.cards.iter().map(|&id| CardId(id)).collect();
    let deck2 = deck_registry.get("broodmother_swarm").expect("Deck should exist");
    let deck2_cards: Vec<CardId> = deck2.cards.iter().map(|&id| CardId(id)).collect();

    let mut total_combats = 0;
    let mut lifesteal_combats = 0;
    let mut games_with_lifesteal_in_combat = 0;

    for game_num in 0..100 {
        let seed = game_num as u64;

        let mut bot1 = GreedyBot::new(&card_db, seed);
        let mut bot2 = RandomBot::new(seed + 1);

        let mut runner = GameRunner::new(&card_db)
            .with_tracing(true, false); // Enable combat tracing

        let result = runner.run_game(
            &mut bot1,
            &mut bot2,
            deck1_cards.clone(),
            deck2_cards.clone(),
            seed,
        );

        // Check combat traces for Lifesteal
        let mut this_game_lifesteal = false;
        for trace in &result.combat_traces {
            total_combats += 1;

            let attacker_has_lifesteal = trace.attacker_keywords.has_lifesteal();
            let defender_has_lifesteal = trace.defender_keywords
                .map_or(false, |k| k.has_lifesteal());

            if attacker_has_lifesteal || defender_has_lifesteal {
                lifesteal_combats += 1;
                this_game_lifesteal = true;

                if game_num < 5 {
                    println!("Game {}: Lifesteal in combat! Attacker={:?}, Defender={:?}",
                        game_num,
                        trace.attacker_keywords.to_names(),
                        trace.defender_keywords.map(|k| k.to_names()));
                }
            }
        }

        if this_game_lifesteal {
            games_with_lifesteal_in_combat += 1;
        }

        // Print first few games' details
        if game_num < 5 {
            println!("\nGame {}: Turns={}, Winner={:?}, Combats={}",
                game_num, result.turns, result.winner, result.combat_traces.len());
        }
    }

    println!("\n=== SUMMARY ===");
    println!("Total combats across 100 games: {}", total_combats);
    println!("Combats involving Lifesteal: {}", lifesteal_combats);
    println!("Games where Lifesteal entered combat: {}", games_with_lifesteal_in_combat);

    // After the Essence fix, we should see Lifesteal in combat
    assert!(
        games_with_lifesteal_in_combat > 0,
        "With proper Essence system, Lifesteal creatures should enter combat"
    );
    assert!(
        lifesteal_combats > 0,
        "Should have some combats involving Lifesteal"
    );
}

#[test]
fn debug_game_length_and_mana() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory(cardgame::data_dir().join("decks")).expect("Failed to load decks");

    let obsidion_deck = deck_registry.get("archon_burst").expect("Deck should exist");
    let aggressive_deck = deck_registry.get("broodmother_swarm").expect("Deck should exist");

    let deck1_cards: Vec<CardId> = obsidion_deck.cards.iter().map(|&id| CardId(id)).collect();
    let deck2_cards: Vec<CardId> = aggressive_deck.cards.iter().map(|&id| CardId(id)).collect();

    println!("\n=== GAME LENGTH ANALYSIS ===");

    let mut turn_histogram = vec![0u32; 35];
    let mut p1_wins = 0;
    let num_games = 100;

    for seed in 0..num_games {
        let mut bot1 = GreedyBot::new(&card_db, seed);
        let mut bot2 = GreedyBot::new(&card_db, seed + 1);

        let mut runner = GameRunner::new(&card_db);

        let result = runner.run_game(
            &mut bot1,
            &mut bot2,
            deck1_cards.clone(),
            deck2_cards.clone(),
            seed,
        );

        if result.winner == Some(PlayerId::PLAYER_ONE) {
            p1_wins += 1;
        }

        let turn = result.turns as usize;
        if turn < turn_histogram.len() {
            turn_histogram[turn] += 1;
        }
    }

    println!("Turn distribution (Obsidion Burst vs Symbiote Aggro, {} games):", num_games);
    for (turn, count) in turn_histogram.iter().enumerate() {
        if *count > 0 {
            println!("  Turn {}: {} games", turn, count);
        }
    }

    println!("\nP1 (Obsidion Burst) win rate: {}%", p1_wins);
    println!("\nNote: Lifesteal creatures cost 2-7 mana.");
    println!("Blood Acolyte (2), Hemomancer (4), The Eternal One (7).");
}

#[test]
fn debug_what_cards_are_played() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory(cardgame::data_dir().join("decks")).expect("Failed to load decks");

    let obsidion_deck = deck_registry.get("archon_burst").expect("Deck should exist");

    // List all cards in obsidion deck with their costs
    println!("\n=== OBSIDION BURST DECK CARD COSTS ===");
    for &card_id in &obsidion_deck.cards {
        if let Some(card) = card_db.get(CardId(card_id)) {
            println!("ID {}: {} (cost {}) - {:?}",
                card_id, card.name, card.cost, card.keywords().to_names());
        }
    }

    // Calculate mana curve
    println!("\n=== MANA CURVE ===");
    let mut cost_counts = vec![0; 10];
    for &card_id in &obsidion_deck.cards {
        if let Some(card) = card_db.get(CardId(card_id)) {
            let cost = card.cost as usize;
            if cost < cost_counts.len() {
                cost_counts[cost] += 1;
            }
        }
    }

    for (cost, count) in cost_counts.iter().enumerate() {
        if *count > 0 {
            println!("  {}-cost: {} cards", cost, count);
        }
    }

    println!("\nLifesteal cards in Obsidion:");
    println!("  - Blood Acolyte (3002): 2 mana (can play turn 2+)");
    println!("  - Hemomancer (3004): 4 mana (can play turn 4+)");
    println!("  - The Eternal One (3007): 7 mana (can play turn 7+)");
}
