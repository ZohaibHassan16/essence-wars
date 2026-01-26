//! Find good seeds for the tutorial.
//!
//! A good tutorial seed should produce a starting hand with:
//! - At least one low-cost creature (1-2 mana)
//! - At least one creature with Guard (to teach the keyword)
//! - At least one spell (to teach spell mechanics)
//! - Good variety (no more than 2 copies of any card)

use cardgame::{CardDatabase, CardType, DeckRegistry};
use cardgame::client_api::GameClient;
use std::collections::HashMap;
use std::sync::Arc;

fn main() {
    let data_dir = cardgame::data_dir();

    // Load card database
    let cards_path = data_dir.join("cards/core_set");
    let card_db = CardDatabase::load_from_directory(&cards_path)
        .expect("Failed to load card database");
    let card_db = Arc::new(card_db);

    // Load deck registry
    let deck_registry = DeckRegistry::load_from_directory(data_dir.join("decks"))
        .expect("Failed to load deck registry");

    // Get tutorial decks (DeckDefinitions include commanders)
    let player_deck = deck_registry.get("architect_fortify")
        .expect("Player deck not found: architect_fortify")
        .clone();
    let opponent_deck = deck_registry.get("broodmother_swarm")
        .expect("Opponent deck not found: broodmother_swarm")
        .clone();

    println!("Searching for good tutorial seeds...\n");
    println!("Criteria:");
    println!("  - Low-cost creature (1-2 mana)");
    println!("  - Creature with Guard keyword");
    println!("  - At least one spell");
    println!("  - Good variety (max 2 copies of any card)");
    println!();

    let mut good_seeds: Vec<(u64, i32, String)> = Vec::new();

    for seed in 0..10000u64 {
        let mut client = GameClient::new(card_db.clone());
        client.start_game(&player_deck, &opponent_deck, seed);

        let state = client.get_state().expect("Game should have state");
        let hand = &state.players[0].hand;

        // Analyze hand
        let mut has_low_cost_creature = false;
        let mut has_guard = false;
        let mut has_spell = false;
        let mut card_counts: HashMap<u16, u32> = HashMap::new();
        let mut hand_details: Vec<String> = Vec::new();

        for card_inst in hand.iter() {
            let card = card_db.get(card_inst.card_id).expect("Card should exist");

            *card_counts.entry(card_inst.card_id.0).or_insert(0) += 1;

            let keywords = card.keywords();

            match &card.card_type {
                CardType::Creature { .. } => {
                    if card.cost <= 2 {
                        has_low_cost_creature = true;
                    }
                    if keywords.has_guard() {
                        has_guard = true;
                    }
                    let kw_str = keywords.to_names().join(", ");
                    hand_details.push(format!("{} ({}c, {})", card.name, card.cost,
                        if kw_str.is_empty() { "Creature".to_string() } else { kw_str }));
                }
                CardType::Spell { .. } => {
                    has_spell = true;
                    hand_details.push(format!("{} ({}c, Spell)", card.name, card.cost));
                }
                CardType::Support { .. } => {
                    hand_details.push(format!("{} ({}c, Support)", card.name, card.cost));
                }
            }
        }

        // Check variety (max 2 copies)
        let max_copies = card_counts.values().max().copied().unwrap_or(0);
        let good_variety = max_copies <= 2;

        // Score the hand
        let mut score = 0;
        if has_low_cost_creature { score += 3; }
        if has_guard { score += 3; }
        if has_spell { score += 2; }
        if good_variety { score += 1; }
        if max_copies == 1 { score += 1; } // Bonus for all unique

        // Count creatures and spells
        let creature_count = hand.iter().filter(|c| {
            matches!(card_db.get(c.card_id).unwrap().card_type, CardType::Creature { .. })
        }).count();
        let spell_count = hand.iter().filter(|c| {
            matches!(card_db.get(c.card_id).unwrap().card_type, CardType::Spell { .. })
        }).count();

        // Prefer 2-3 creatures and 1-2 spells
        if (2..=3).contains(&creature_count) { score += 1; }
        if (1..=2).contains(&spell_count) { score += 1; }

        // Check total mana curve (want some playable cards early)
        let playable_turn2 = hand.iter().filter(|c| {
            card_db.get(c.card_id).unwrap().cost <= 2
        }).count();

        if playable_turn2 >= 2 { score += 1; }

        if score >= 8 {
            let details = hand_details.join(", ");
            good_seeds.push((seed, score, details));
        }
    }

    // Sort by score descending
    good_seeds.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Found {} good seeds:\n", good_seeds.len());

    for (seed, score, details) in good_seeds.iter().take(20) {
        println!("Seed {}: (score {})", seed, score);
        println!("  Hand: {}", details);
        println!();
    }

    if let Some((best_seed, best_score, best_details)) = good_seeds.first() {
        println!("=== RECOMMENDED SEED ===");
        println!("Seed: {}", best_seed);
        println!("Score: {}", best_score);
        println!("Hand: {}", best_details);
    }
}
