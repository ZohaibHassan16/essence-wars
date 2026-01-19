//! Coverage verification tests - runs 10k+ games to ensure all game states are exercised.
//!
//! This module tracks:
//! - Action index coverage (all 256 indices)
//! - Keyword occurrence coverage (all 8 keywords)
//! - Action type coverage (PlayCard, Attack, UseAbility, EndTurn)
//! - Game outcome coverage (P1 win, P2 win, draw)
//!
//! Test failures are logged to experiments/test_failures/ for reproduction.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::fs;

use cardgame::actions::Action;
use cardgame::arena::{GameRunner, GameResult};
use cardgame::bots::{Bot, GreedyBot, MctsBot, MctsConfig, RandomBot};
use cardgame::bots::weights::BotWeights;
use cardgame::cards::CardDatabase;
use cardgame::decks::DeckRegistry;
use cardgame::keywords::Keywords;
use cardgame::types::{CardId, PlayerId};

/// Coverage statistics collected across all games
struct CoverageStats {
    /// Count of times each action index was used
    action_index_counts: [AtomicU64; 256],
    /// Count of times each keyword appeared in combat
    keyword_counts: Mutex<HashMap<&'static str, u64>>,
    /// Count of each action type
    action_type_counts: Mutex<HashMap<&'static str, u64>>,
    /// Game outcomes
    p1_wins: AtomicU64,
    p2_wins: AtomicU64,
    draws: AtomicU64,
    /// Total games played
    total_games: AtomicU64,
    /// Total turns across all games
    total_turns: AtomicU64,
}

impl CoverageStats {
    fn new() -> Self {
        Self {
            action_index_counts: std::array::from_fn(|_| AtomicU64::new(0)),
            keyword_counts: Mutex::new(HashMap::new()),
            action_type_counts: Mutex::new(HashMap::new()),
            p1_wins: AtomicU64::new(0),
            p2_wins: AtomicU64::new(0),
            draws: AtomicU64::new(0),
            total_games: AtomicU64::new(0),
            total_turns: AtomicU64::new(0),
        }
    }

    fn record_action(&self, action: Action) {
        let index = action.to_index();
        self.action_index_counts[index as usize].fetch_add(1, Ordering::Relaxed);

        let mut types = self.action_type_counts.lock().unwrap();
        let type_name = match action {
            Action::PlayCard { .. } => "PlayCard",
            Action::Attack { .. } => "Attack",
            Action::UseAbility { .. } => "UseAbility",
            Action::EndTurn => "EndTurn",
        };
        *types.entry(type_name).or_insert(0) += 1;
    }

    fn record_game_result(&self, result: &GameResult) {
        self.total_games.fetch_add(1, Ordering::Relaxed);
        self.total_turns.fetch_add(result.turns as u64, Ordering::Relaxed);

        match result.winner {
            Some(PlayerId::PLAYER_ONE) => {
                self.p1_wins.fetch_add(1, Ordering::Relaxed);
            }
            Some(PlayerId::PLAYER_TWO) => {
                self.p2_wins.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                self.draws.fetch_add(1, Ordering::Relaxed);
            }
        }

        // Record actions from this game
        for record in &result.actions {
            self.record_action(record.action);
        }
    }

    fn record_keywords(&self, keywords: Keywords) {
        let mut kw_counts = self.keyword_counts.lock().unwrap();
        if keywords.has_rush() {
            *kw_counts.entry("Rush").or_insert(0) += 1;
        }
        if keywords.has_ranged() {
            *kw_counts.entry("Ranged").or_insert(0) += 1;
        }
        if keywords.has_piercing() {
            *kw_counts.entry("Piercing").or_insert(0) += 1;
        }
        if keywords.has_guard() {
            *kw_counts.entry("Guard").or_insert(0) += 1;
        }
        if keywords.has_lifesteal() {
            *kw_counts.entry("Lifesteal").or_insert(0) += 1;
        }
        if keywords.has_lethal() {
            *kw_counts.entry("Lethal").or_insert(0) += 1;
        }
        if keywords.has_shield() {
            *kw_counts.entry("Shield").or_insert(0) += 1;
        }
        if keywords.has_quick() {
            *kw_counts.entry("Quick").or_insert(0) += 1;
        }
    }

    fn get_action_index_coverage(&self) -> (usize, usize, Vec<u8>) {
        let mut covered = 0;
        let mut uncovered_indices = Vec::new();

        for (i, count) in self.action_index_counts.iter().enumerate() {
            if count.load(Ordering::Relaxed) > 0 {
                covered += 1;
            } else {
                uncovered_indices.push(i as u8);
            }
        }

        (covered, 256, uncovered_indices)
    }

    fn print_report(&self) {
        println!("\n=== COVERAGE REPORT ===\n");

        // Game statistics
        let total = self.total_games.load(Ordering::Relaxed);
        let p1_wins = self.p1_wins.load(Ordering::Relaxed);
        let p2_wins = self.p2_wins.load(Ordering::Relaxed);
        let draws = self.draws.load(Ordering::Relaxed);
        let total_turns = self.total_turns.load(Ordering::Relaxed);

        println!("GAME STATISTICS:");
        println!("  Total games: {}", total);
        println!("  P1 wins: {} ({:.1}%)", p1_wins, 100.0 * p1_wins as f64 / total as f64);
        println!("  P2 wins: {} ({:.1}%)", p2_wins, 100.0 * p2_wins as f64 / total as f64);
        println!("  Draws: {} ({:.1}%)", draws, 100.0 * draws as f64 / total as f64);
        println!("  Avg turns/game: {:.1}", total_turns as f64 / total as f64);

        // Action index coverage
        let (covered, total_indices, uncovered) = self.get_action_index_coverage();
        println!("\nACTION INDEX COVERAGE:");
        println!("  Covered: {}/{} ({:.1}%)", covered, total_indices, 100.0 * covered as f64 / total_indices as f64);

        // Categorize uncovered indices
        let mut uncovered_play = Vec::new();
        let mut uncovered_attack = Vec::new();
        let mut uncovered_ability = Vec::new();

        for idx in &uncovered {
            match *idx {
                0..=49 => uncovered_play.push(*idx),
                50..=74 => uncovered_attack.push(*idx),
                75..=254 => uncovered_ability.push(*idx),
                255 => {} // EndTurn should always be covered
            }
        }

        if !uncovered_play.is_empty() {
            println!("  Uncovered PlayCard indices: {} (expected: hand indices 5-9 are typically unused)", uncovered_play.len());
        }
        if !uncovered_attack.is_empty() {
            println!("  Uncovered Attack indices: {}", uncovered_attack.len());
        }
        if !uncovered_ability.is_empty() {
            println!("  Uncovered UseAbility indices: {} (expected: most are invalid ability combos)", uncovered_ability.len());
        }

        // Action type coverage
        let types = self.action_type_counts.lock().unwrap();
        println!("\nACTION TYPE COUNTS:");
        for (action_type, count) in types.iter() {
            println!("  {}: {}", action_type, count);
        }

        // Keyword coverage
        let keywords = self.keyword_counts.lock().unwrap();
        println!("\nKEYWORD OCCURRENCES:");
        let all_keywords = ["Rush", "Ranged", "Piercing", "Guard", "Lifesteal", "Lethal", "Shield", "Quick"];
        for kw in &all_keywords {
            let count = keywords.get(*kw).copied().unwrap_or(0);
            println!("  {}: {}", kw, count);
        }

        println!("\n=== END COVERAGE REPORT ===\n");
    }
}

/// Bot type for coverage games
#[derive(Clone, Copy)]
enum BotType {
    Random,
    Greedy,
    Mcts,
}

/// Log a failing test seed for reproduction
fn log_test_failure(seed: u64, deck1_id: &str, deck2_id: &str, bot1_type: &str, bot2_type: &str, error: &str) {
    use std::io::Write;
    
    // Create failure log directory
    let _ = fs::create_dir_all("experiments/test_failures");
    
    // Generate timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let filename = format!("experiments/test_failures/{}_seed_{}.txt", timestamp, seed);
    
    if let Ok(mut file) = fs::File::create(&filename) {
        let _ = writeln!(file, "=== Test Failure Reproduction Info ===");
        let _ = writeln!(file, "Seed: {}", seed);
        let _ = writeln!(file, "Deck 1: {}", deck1_id);
        let _ = writeln!(file, "Deck 2: {}", deck2_id);
        let _ = writeln!(file, "Bot 1: {}", bot1_type);
        let _ = writeln!(file, "Bot 2: {}", bot2_type);
        let _ = writeln!(file, "Error: {}", error);
        let _ = writeln!(file, "");
        let _ = writeln!(file, "To reproduce:");
        let _ = writeln!(file, "cargo test --test coverage_tests -- --exact --nocapture");
        let _ = writeln!(file, "# Or use this seed in a custom test");
        
        eprintln!("⚠️  Test failure logged to: {}", filename);
    }
}

/// Run games and collect coverage statistics
fn run_coverage_games(
    card_db: &CardDatabase,
    deck_registry: &DeckRegistry,
    stats: &CoverageStats,
    bot1_type: BotType,
    bot2_type: BotType,
    games: usize,
    base_seed: u64,
) {
    let deck_ids: Vec<_> = deck_registry.deck_ids().into_iter().collect();
    let num_decks = deck_ids.len();
    
    // Use stronger config for 300-card set (200 sims instead of 100)
    let mcts_config = MctsConfig {
        simulations: 200,
        exploration: 1.414,
        max_rollout_depth: 100,
        parallel_trees: 1,
        leaf_rollouts: 1,
    };
    
    // Load generalist weights for MCTS rollout policy
    let weights_path = cardgame::data_dir().join("weights/generalist.toml");
    let weights = BotWeights::load(&weights_path)
        .expect("Failed to load generalist weights");

    for i in 0..games {
        let seed = base_seed.wrapping_add(i as u64);

        // Rotate through deck combinations
        let deck1_idx = i % num_decks;
        let deck2_idx = (i / num_decks) % num_decks;

        let deck1 = deck_registry.get(&deck_ids[deck1_idx]).unwrap();
        let deck2 = deck_registry.get(&deck_ids[deck2_idx]).unwrap();

        // Convert u16 to CardId
        let deck1_cards: Vec<CardId> = deck1.cards.iter().map(|&id| CardId(id)).collect();
        let deck2_cards: Vec<CardId> = deck2.cards.iter().map(|&id| CardId(id)).collect();

        // Create bots based on type
        let mut greedy1;
        let mut greedy2;
        let mut random1;
        let mut random2;
        let mut mcts1;
        let mut mcts2;

        let bot1: &mut dyn Bot = match bot1_type {
            BotType::Random => {
                random1 = RandomBot::new(seed);
                &mut random1
            }
            BotType::Greedy => {
                greedy1 = GreedyBot::new(card_db, seed);
                &mut greedy1
            }
            BotType::Mcts => {
                mcts1 = MctsBot::with_config_and_weights(card_db, mcts_config.clone(), &weights, seed);
                &mut mcts1
            }
        };

        let bot2: &mut dyn Bot = match bot2_type {
            BotType::Random => {
                random2 = RandomBot::new(seed + 1);
                &mut random2
            }
            BotType::Greedy => {
                greedy2 = GreedyBot::new(card_db, seed + 1);
                &mut greedy2
            }
            BotType::Mcts => {
                mcts2 = MctsBot::with_config_and_weights(card_db, mcts_config.clone(), &weights, seed + 1);
                &mut mcts2
            }
        };

        let mut runner = GameRunner::new(card_db)
            .with_tracing(true, true); // Enable tracing to capture combat details

        let result = runner.run_game(
            bot1,
            bot2,
            deck1_cards.clone(),
            deck2_cards.clone(),
            seed,
        );
        
        // Check for game errors/panics (if result has error info)
        if result.turns == 0 {
            // Suspicious - game ended immediately
            let bot1_name = match bot1_type {
                BotType::Random => "Random",
                BotType::Greedy => "Greedy",
                BotType::Mcts => "MCTS",
            };
            let bot2_name = match bot2_type {
                BotType::Random => "Random",
                BotType::Greedy => "Greedy",
                BotType::Mcts => "MCTS",
            };
            
            log_test_failure(
                seed,
                &deck_ids[deck1_idx],
                &deck_ids[deck2_idx],
                bot1_name,
                bot2_name,
                "Game ended with 0 turns (possible panic/error)"
            );
        }

        // Record keywords from combat traces
        for trace in &result.combat_traces {
            stats.record_keywords(trace.attacker_keywords);
            if let Some(defender_kw) = trace.defender_keywords {
                stats.record_keywords(defender_kw);
            }
        }

        stats.record_game_result(&result);
    }
}

#[test]
fn test_coverage_random_vs_random_1k() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory(cardgame::data_dir().join("decks"))
        .expect("Failed to load decks");

    let stats = CoverageStats::new();

    run_coverage_games(
        &card_db,
        &deck_registry,
        &stats,
        BotType::Random,
        BotType::Random,
        1000,
        12345,
    );

    stats.print_report();

    // Basic assertions
    let (covered, _, _) = stats.get_action_index_coverage();
    assert!(covered > 50, "Should cover at least 50 action indices in 1k random games");

    let keywords = stats.keyword_counts.lock().unwrap();
    assert!(keywords.len() >= 4, "Should see at least 4 different keywords");
}

#[test]
fn test_coverage_greedy_vs_random_1k() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory(cardgame::data_dir().join("decks"))
        .expect("Failed to load decks");

    let stats = CoverageStats::new();

    run_coverage_games(
        &card_db,
        &deck_registry,
        &stats,
        BotType::Greedy,
        BotType::Random,
        1000,
        54321,
    );

    stats.print_report();

    // Greedy bot should win more than it loses (excluding draws)
    // Note: High draw rate (~48%) observed due to 30-turn limit
    let p1_wins = stats.p1_wins.load(Ordering::Relaxed);
    let p2_wins = stats.p2_wins.load(Ordering::Relaxed);
    let draws = stats.draws.load(Ordering::Relaxed);
    let total = stats.total_games.load(Ordering::Relaxed);

    println!("Greedy vs Random: P1={} ({:.1}%), P2={} ({:.1}%), Draws={} ({:.1}%)",
        p1_wins, 100.0 * p1_wins as f64 / total as f64,
        p2_wins, 100.0 * p2_wins as f64 / total as f64,
        draws, 100.0 * draws as f64 / total as f64);

    // GreedyBot should win more than RandomBot when games complete (not draw)
    // Note: threshold lowered from 70% to 55% due to FPA compensation (P2 gets +1 card, +1 essence)
    let decisive_games = p1_wins + p2_wins;
    if decisive_games > 0 {
        let win_rate_in_decisive = p1_wins as f64 / decisive_games as f64;
        assert!(
            win_rate_in_decisive > 0.55,
            "GreedyBot should win >55% of decisive games vs RandomBot (got {:.1}%)",
            win_rate_in_decisive * 100.0
        );
    }
}

#[test]
#[ignore = "tier_long"] // ~10 min: 10k game coverage test
fn stress_test_coverage_10k_games() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory(cardgame::data_dir().join("decks"))
        .expect("Failed to load decks");

    let stats = CoverageStats::new();

    // Run 5k random vs random (high variance, explores edge cases)
    println!("Running 5000 Random vs Random games...");
    run_coverage_games(
        &card_db,
        &deck_registry,
        &stats,
        BotType::Random,
        BotType::Random,
        5000,
        0,
    );

    // Run 3k greedy vs random
    println!("Running 3000 Greedy vs Random games...");
    run_coverage_games(
        &card_db,
        &deck_registry,
        &stats,
        BotType::Greedy,
        BotType::Random,
        3000,
        10000,
    );

    // Run 2k greedy vs greedy
    println!("Running 2000 Greedy vs Greedy games...");
    run_coverage_games(
        &card_db,
        &deck_registry,
        &stats,
        BotType::Greedy,
        BotType::Greedy,
        2000,
        20000,
    );

    stats.print_report();

    // Comprehensive assertions for 10k games
    let (covered, _, _uncovered) = stats.get_action_index_coverage();

    // We expect to cover most valid action indices
    // Note: Many UseAbility indices (75-254) are invalid because:
    // - Creatures have 0-2 abilities (not 6)
    // - Many target combinations are invalid
    // So realistic coverage is ~100-150 indices out of 256

    println!("\nCOVERAGE ANALYSIS:");
    println!("Total indices covered: {}/256", covered);

    // PlayCard should cover hand indices 0-4 commonly (0-24)
    // Attack should cover most of 50-74
    // EndTurn (255) should always be covered

    // Count by category
    let mut play_covered = 0;
    let mut attack_covered = 0;
    let mut ability_covered = 0;
    let end_turn_covered = stats.action_index_counts[255].load(Ordering::Relaxed) > 0;

    for i in 0..=49 {
        if stats.action_index_counts[i].load(Ordering::Relaxed) > 0 {
            play_covered += 1;
        }
    }
    for i in 50..=74 {
        if stats.action_index_counts[i].load(Ordering::Relaxed) > 0 {
            attack_covered += 1;
        }
    }
    for i in 75..=254 {
        if stats.action_index_counts[i].load(Ordering::Relaxed) > 0 {
            ability_covered += 1;
        }
    }

    println!("PlayCard coverage: {}/50", play_covered);
    println!("Attack coverage: {}/25", attack_covered);
    println!("UseAbility coverage: {}/180", ability_covered);
    println!("EndTurn covered: {}", end_turn_covered);

    // Assertions
    assert!(play_covered >= 20, "Should cover at least 20 PlayCard indices");
    assert!(attack_covered >= 20, "Should cover at least 20 Attack indices (out of 25)");
    assert!(end_turn_covered, "EndTurn should always be covered");

    // Check keyword coverage - report any missing keywords
    let keywords = stats.keyword_counts.lock().unwrap();
    let all_keywords = ["Rush", "Ranged", "Piercing", "Guard", "Lifesteal", "Lethal", "Shield", "Quick"];
    let mut missing_keywords = Vec::new();
    for kw in &all_keywords {
        if keywords.get(*kw).copied().unwrap_or(0) == 0 {
            missing_keywords.push(*kw);
        }
    }

    if !missing_keywords.is_empty() {
        println!("\nWARNING: Keywords not observed in combat: {:?}", missing_keywords);
        println!("This may indicate these creatures never enter combat or aren't in test decks.");
    }

    // At least 6 keywords should appear (allowing for some edge cases)
    let keywords_observed = all_keywords.iter()
        .filter(|kw| keywords.get(**kw).copied().unwrap_or(0) > 0)
        .count();
    assert!(
        keywords_observed >= 6,
        "Should observe at least 6 different keywords in combat, got {}",
        keywords_observed
    );

    // No panics or errors should have occurred
    assert_eq!(
        stats.total_games.load(Ordering::Relaxed),
        10000,
        "All 10,000 games should complete successfully"
    );
}

#[test]
#[ignore = "tier_long"] // ~15 min: 500 MCTS coverage games
fn stress_test_mcts_coverage_500_games() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory(cardgame::data_dir().join("decks"))
        .expect("Failed to load decks");

    let stats = CoverageStats::new();

    println!("Running 500 MCTS vs Greedy games (this may take a while)...");

    run_coverage_games(
        &card_db,
        &deck_registry,
        &stats,
        BotType::Mcts,
        BotType::Greedy,
        500,
        99999,
    );

    stats.print_report();

    // MCTS should have reasonable win rate against Greedy
    let p1_wins = stats.p1_wins.load(Ordering::Relaxed);
    let total = stats.total_games.load(Ordering::Relaxed);
    let win_rate = p1_wins as f64 / total as f64;

    println!("\nMCTS win rate vs Greedy: {:.1}%", win_rate * 100.0);
    // With 300-card set's defensive options causing ~20% draws,
    // MCTS needs at least 22% wins (not counting draws) to demonstrate competence
    assert!(
        win_rate > 0.22,
        "MCTS should beat Greedy at least 22% of the time (got {:.1}%)",
        win_rate * 100.0
    );
}

#[test]
fn test_all_decks_exercised() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory(cardgame::data_dir().join("decks"))
        .expect("Failed to load decks");

    let deck_ids: Vec<_> = deck_registry.deck_ids().into_iter().collect();
    let mut decks_used = HashSet::new();

    for (i, deck1_id) in deck_ids.iter().enumerate() {
        for deck2_id in deck_ids.iter() {
            let deck1 = deck_registry.get(deck1_id).unwrap();
            let deck2 = deck_registry.get(deck2_id).unwrap();

            // Convert u16 to CardId
            let deck1_cards: Vec<CardId> = deck1.cards.iter().map(|&id| CardId(id)).collect();
            let deck2_cards: Vec<CardId> = deck2.cards.iter().map(|&id| CardId(id)).collect();

            let mut bot1 = RandomBot::new(i as u64);
            let mut bot2 = RandomBot::new((i + 1) as u64);

            let mut runner = GameRunner::new(&card_db);
            let result = runner.run_game(
                &mut bot1,
                &mut bot2,
                deck1_cards,
                deck2_cards,
                i as u64,
            );

            decks_used.insert((*deck1_id).to_string());
            decks_used.insert((*deck2_id).to_string());

            // Verify game completed normally
            assert!(result.turns > 0, "Game should have at least 1 turn");
        }
    }

    // All decks should have been used
    assert_eq!(
        decks_used.len(),
        deck_ids.len(),
        "All decks should be exercised"
    );

    println!("All {} decks exercised in {} matchups", deck_ids.len(), deck_ids.len() * deck_ids.len());
}

#[test]
fn test_action_index_roundtrip_exhaustive() {
    // Verify all 256 action indices can be encoded and decoded correctly
    for i in 0u8..=255 {
        if let Some(action) = Action::from_index(i) {
            let encoded = action.to_index();
            assert_eq!(
                i, encoded,
                "Action index {} should roundtrip correctly, got {}",
                i, encoded
            );
        }
        // Some indices are invalid (e.g., UseAbility with slot >= 5)
        // That's expected and fine
    }

    // Count valid indices
    let valid_count = (0u8..=255)
        .filter(|&i| Action::from_index(i).is_some())
        .count();

    println!("Valid action indices: {}/256", valid_count);

    // We expect:
    // - 50 PlayCard (0-49)
    // - 25 Attack (50-74)
    // - Some UseAbility (75-254, but only slot 0-4 are valid = 5 * 36 = 180)
    // - 1 EndTurn (255)
    // Total valid: 50 + 25 + 180 + 1 = 256 (all should be valid by design)
    assert_eq!(valid_count, 256, "All 256 indices should be valid actions");
}

#[test]
fn test_game_determinism_verification() {
    let card_db = CardDatabase::load_from_directory(cardgame::data_dir().join("cards/core_set")).expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory(cardgame::data_dir().join("decks"))
        .expect("Failed to load decks");

    let deck_ids: Vec<_> = deck_registry.deck_ids().into_iter().collect();
    let deck1 = deck_registry.get(&deck_ids[0]).unwrap();
    let deck2 = deck_registry.get(&deck_ids[1 % deck_ids.len()]).unwrap();

    // Convert u16 to CardId
    let deck1_cards: Vec<CardId> = deck1.cards.iter().map(|&id| CardId(id)).collect();
    let deck2_cards: Vec<CardId> = deck2.cards.iter().map(|&id| CardId(id)).collect();

    // Run same game twice with same seed
    let seed = 42424242u64;

    let run_game = |seed: u64, d1: Vec<CardId>, d2: Vec<CardId>| {
        // Use same seed for both bots to ensure determinism
        let mut bot1 = GreedyBot::new(&card_db, seed);
        let mut bot2 = GreedyBot::new(&card_db, seed + 1);
        let mut runner = GameRunner::new(&card_db);

        runner.run_game(
            &mut bot1,
            &mut bot2,
            d1,
            d2,
            seed,
        )
    };

    let result1 = run_game(seed, deck1_cards.clone(), deck2_cards.clone());
    let result2 = run_game(seed, deck1_cards, deck2_cards);

    // Verify identical outcomes
    assert_eq!(result1.winner, result2.winner, "Winner should be deterministic");
    assert_eq!(result1.turns, result2.turns, "Turn count should be deterministic");
    assert_eq!(
        result1.actions.len(),
        result2.actions.len(),
        "Action count should be deterministic"
    );

    // Verify all actions match
    for (i, (a1, a2)) in result1.actions.iter().zip(result2.actions.iter()).enumerate() {
        assert_eq!(
            a1.action, a2.action,
            "Action {} should be deterministic",
            i
        );
    }

    println!("Determinism verified: same seed produces identical game");
}
