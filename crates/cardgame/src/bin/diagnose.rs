//! P1/P2 Asymmetry Diagnostic Tool
//!
//! Collects detailed per-turn statistics to analyze why P1 has a lower win rate.
//! Outputs data for analysis of resource curves, tempo, and game progression.
//!
//! Usage:
//!   cargo run --release --bin diagnose [games]
//!   cargo run --release --bin diagnose 500
//!   cargo run --release --bin diagnose 500 --export csv --output ./diagnostics
//!   cargo run --release --bin diagnose 500 --export json --output ./diagnostics/results.json

use std::io::Write;
use std::path::PathBuf;
use std::process;

use clap::Parser;

use cardgame::cards::CardDatabase;
use cardgame::decks::DeckRegistry;
use cardgame::diagnostics::{
    export_csv, export_json, AggregatedStats, DiagnosticConfig, DiagnosticRunner, ExportFormat,
    print_report,
};
use cardgame::types::CardId;

/// P1/P2 Asymmetry Diagnostic Tool
#[derive(Parser, Debug)]
#[command(name = "diagnose")]
#[command(about = "Analyze P1/P2 asymmetry in card game matches")]
struct Args {
    /// Number of games to run
    #[arg(default_value = "200")]
    games: usize,

    /// Export format (csv, json, or all)
    #[arg(long, short = 'e')]
    export: Option<String>,

    /// Output path for export (directory for CSV, file for JSON)
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

    /// Include per-turn data in export (can be large)
    #[arg(long)]
    include_turns: bool,

    /// Deck ID to use (default: symbiote_aggro)
    #[arg(long, default_value = "symbiote_aggro")]
    deck: String,

    /// Random seed for reproducibility
    #[arg(long, short = 's', default_value = "42")]
    seed: u64,

    /// Show progress during execution
    #[arg(long, short = 'p')]
    progress: bool,
}

fn main() {
    let args = Args::parse();

    println!("P1/P2 Asymmetry Diagnostic Tool");
    println!("================================");
    println!(
        "Running {} games with GreedyBot vs GreedyBot...\n",
        args.games
    );

    // Load card database
    let cards_path = cardgame::data_dir().join("cards/core_set");
    let card_db = match CardDatabase::load_from_directory(&cards_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error loading card database: {}", e);
            process::exit(1);
        }
    };

    // Load deck registry
    let decks_path = cardgame::data_dir().join("decks");
    let deck_registry = match DeckRegistry::load_from_directory(&decks_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error loading decks: {}", e);
            process::exit(1);
        }
    };

    // Get the specified deck
    let deck = match deck_registry.get(&args.deck) {
        Some(d) => d,
        None => {
            eprintln!("Error: deck '{}' not found", args.deck);
            eprintln!("Available decks:");
            for d in deck_registry.decks() {
                eprintln!("  - {}", d.id);
            }
            process::exit(1);
        }
    };
    let deck_cards: Vec<CardId> = deck.cards.iter().map(|&id| CardId(id)).collect();

    // Configure diagnostics
    let config = DiagnosticConfig::new(deck_cards, args.games)
        .with_seed(args.seed)
        .with_progress(args.progress);

    // Run diagnostic games
    let runner = DiagnosticRunner::new(&card_db);

    // Simple progress indicator (if not using built-in progress)
    if !args.progress {
        for i in 0..args.games {
            if i % 50 == 0 {
                eprint!("\rProgress: {}/{}", i, args.games);
                std::io::stderr().flush().unwrap();
            }
        }
    }

    let diagnostics = runner.run(&config);

    if !args.progress {
        eprintln!("\rProgress: {}/{}", args.games, args.games);
    }

    // Analyze and report
    let stats = AggregatedStats::analyze(&diagnostics);

    // Handle export if requested
    if let Some(export_str) = &args.export {
        let format = match ExportFormat::parse(export_str) {
            Some(f) => f,
            None => {
                eprintln!(
                    "Unknown export format: '{}'. Use 'csv', 'json', or 'all'.",
                    export_str
                );
                process::exit(1);
            }
        };

        let output_path = args.output.unwrap_or_else(|| PathBuf::from("./diagnostics"));

        match format {
            ExportFormat::Csv => {
                if let Err(e) = export_csv(&output_path, &stats, &diagnostics, args.include_turns) {
                    eprintln!("Error exporting CSV: {}", e);
                    process::exit(1);
                }
                println!("\nExported CSV to: {:?}", output_path);
            }
            ExportFormat::Json => {
                let json_path = if output_path.extension().is_some() {
                    output_path.clone()
                } else {
                    output_path.join("diagnostics.json")
                };
                if let Err(e) = export_json(&json_path, &stats, &diagnostics, args.include_turns) {
                    eprintln!("Error exporting JSON: {}", e);
                    process::exit(1);
                }
                println!("\nExported JSON to: {:?}", json_path);
            }
            ExportFormat::All => {
                let csv_dir = if output_path.is_dir() || output_path.extension().is_none() {
                    output_path.clone()
                } else {
                    output_path.parent().unwrap_or(&output_path).to_path_buf()
                };

                if let Err(e) = export_csv(&csv_dir, &stats, &diagnostics, args.include_turns) {
                    eprintln!("Error exporting CSV: {}", e);
                    process::exit(1);
                }

                let json_path = csv_dir.join("diagnostics.json");
                if let Err(e) = export_json(&json_path, &stats, &diagnostics, args.include_turns) {
                    eprintln!("Error exporting JSON: {}", e);
                    process::exit(1);
                }

                println!("\nExported CSV and JSON to: {:?}", csv_dir);
            }
        }
    }

    // Always print report
    print_report(&stats);
}
