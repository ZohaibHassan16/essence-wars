//! P1/P2 Asymmetry Diagnostic Tool
//!
//! Collects detailed per-turn statistics to analyze why P1 has a lower win rate.
//! Outputs data for analysis of resource curves, tempo, and game progression.
//!
//! Usage:
//!   cargo run --release --bin diagnose [games]
//!   cargo run --release --bin diagnose 500

use std::io::Write;
use std::process;

use cardgame::cards::CardDatabase;
use cardgame::decks::DeckRegistry;
use cardgame::diagnostics::{AggregatedStats, DiagnosticConfig, DiagnosticRunner, print_report};
use cardgame::types::CardId;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let games: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(200);

    println!("P1/P2 Asymmetry Diagnostic Tool");
    println!("================================");
    println!("Running {} games with GreedyBot vs GreedyBot...\n", games);

    // Load card database
    let card_db = match CardDatabase::load_from_directory("data/cards/core_set") {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error loading card database: {}", e);
            process::exit(1);
        }
    };

    // Load deck registry
    let deck_registry = match DeckRegistry::load_from_directory("data/decks") {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error loading decks: {}", e);
            process::exit(1);
        }
    };

    // Use symbiote_aggro as our test deck (most common in validation)
    let deck = match deck_registry.get("symbiote_aggro") {
        Some(d) => d,
        None => {
            eprintln!("Error: symbiote_aggro deck not found");
            process::exit(1);
        }
    };
    let deck_cards: Vec<CardId> = deck.cards.iter().map(|&id| CardId(id)).collect();

    // Configure diagnostics
    let config = DiagnosticConfig::new(deck_cards, games).with_seed(42);

    // Run diagnostic games
    let runner = DiagnosticRunner::new(&card_db);

    // Simple progress indicator
    for i in 0..games {
        if i % 50 == 0 {
            eprint!("\rProgress: {}/{}", i, games);
            std::io::stderr().flush().unwrap();
        }
    }
    eprintln!("\rProgress: {}/{}", games, games);

    let diagnostics = runner.run(&config);

    // Analyze and report
    let stats = AggregatedStats::analyze(&diagnostics);
    print_report(&stats);
}
