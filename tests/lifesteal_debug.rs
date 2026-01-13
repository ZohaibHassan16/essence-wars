//! Lifesteal verification tests.
//!
//! These tests verify that Lifesteal creatures are properly played and enter combat.
//! Originally created to debug an issue where Lifesteal never appeared in combat
//! (which was caused by the Essence system not being implemented, fixed 2026-01-12).

use cardgame::arena::GameRunner;
use cardgame::bots::{GreedyBot, RandomBot};
use cardgame::cards::CardDatabase;
use cardgame::decks::DeckRegistry;
use cardgame::types::{CardId, PlayerId};

#[test]
fn debug_lifesteal_investigation() {
    let card_db = CardDatabase::load_from_directory("data/cards/sets").expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory("data/decks").expect("Failed to load decks");

    // Verify Lifesteal cards exist
    println!("\n=== CARD DATABASE CHECK ===");
    let vampire_lord = card_db.get(CardId(21));
    let guardian_angel = card_db.get(CardId(28));

    println!("Card 21 (Vampire Lord): {:?}", vampire_lord.map(|c| (c.name.as_str(), c.keywords())));
    println!("Card 28 (Guardian Angel): {:?}", guardian_angel.map(|c| (c.name.as_str(), c.keywords())));

    // Verify defensive_control deck has Lifesteal
    println!("\n=== DECK CHECK ===");
    let defensive_deck = deck_registry.get("defensive_control").expect("Deck should exist");
    println!("Defensive deck cards: {:?}", defensive_deck.cards);

    let lifesteal_in_deck = defensive_deck.cards.iter()
        .filter(|&&id| id == 21 || id == 28)
        .count();
    println!("Lifesteal cards in deck: {} (should be 4 - two each of ID 21 and 28)", lifesteal_in_deck);

    // Run games and track what's happening
    println!("\n=== RUNNING 100 GAMES WITH TRACING ===");

    let deck1_cards: Vec<CardId> = defensive_deck.cards.iter().map(|&id| CardId(id)).collect();
    let deck2 = deck_registry.get("aggressive_assault").expect("Deck should exist");
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
            println!("\nGame {}: Turns={}, Winner={:?}, Combats={}, P1 creatures played: ?",
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

    // The real question: why aren't 4-cost Lifesteal creatures entering combat?
    println!("\n=== POSSIBLE CAUSES ===");
    if lifesteal_combats == 0 {
        println!("1. Lifesteal creatures (4+ cost) may not be played due to mana constraints");
        println!("2. Games may end before turn 4-6 when these cards can be played");
        println!("3. Lifesteal creatures may be killed before they can attack");
        println!("4. GreedyBot may not value playing them highly enough");
    }
}

#[test]
fn debug_game_length_and_mana() {
    let card_db = CardDatabase::load_from_directory("data/cards/sets").expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory("data/decks").expect("Failed to load decks");

    let defensive_deck = deck_registry.get("defensive_control").expect("Deck should exist");
    let aggressive_deck = deck_registry.get("aggressive_assault").expect("Deck should exist");

    let deck1_cards: Vec<CardId> = defensive_deck.cards.iter().map(|&id| CardId(id)).collect();
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

    println!("Turn distribution (Defensive vs Aggressive, {} games):", num_games);
    for (turn, count) in turn_histogram.iter().enumerate() {
        if *count > 0 {
            println!("  Turn {}: {} games", turn, count);
        }
    }

    println!("\nP1 (Defensive) win rate: {}%", p1_wins);
    println!("\nNote: Lifesteal creatures cost 4-6 mana.");
    println!("If games end before turn 4-6, Lifesteal creatures won't be played.");
}

#[test]
fn debug_what_cards_are_played() {
    let card_db = CardDatabase::load_from_directory("data/cards/sets").expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory("data/decks").expect("Failed to load decks");

    let defensive_deck = deck_registry.get("defensive_control").expect("Deck should exist");

    // List all cards in defensive deck with their costs
    println!("\n=== DEFENSIVE DECK CARD COSTS ===");
    for &card_id in &defensive_deck.cards {
        if let Some(card) = card_db.get(CardId(card_id)) {
            println!("ID {}: {} (cost {}) - {:?}",
                card_id, card.name, card.cost, card.keywords().to_names());
        }
    }

    // Calculate mana curve
    println!("\n=== MANA CURVE ===");
    let mut cost_counts = vec![0; 10];
    for &card_id in &defensive_deck.cards {
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

    println!("\nLifesteal cards:");
    println!("  - Vampire Lord: 4 mana (can play turn 4+)");
    println!("  - Guardian Angel: 6 mana (can play turn 6+)");
}
