//! Single matchup runner for cloud validation.
//!
//! Runs a single deck-vs-deck matchup and outputs JSON results.
//! Designed for parallel execution in Modal containers.
//!
//! Usage:
//!   matchup --deck1 broodmother_pack --deck2 architect_fortify --games 250 --json
//!   matchup --deck1 X --deck2 Y --ab-depth 6 --seed 42 --json --quiet

use std::path::PathBuf;
use std::process;

use clap::Parser;

use cardgame::bots::BotType;
use cardgame::execution::{parse_bot_type_or_exit, GameData, UnifiedMatchup};
use cardgame::validation::{ArchetypeWeights, ValidationExecutor};

/// Single matchup runner for cloud validation
#[derive(Parser, Debug)]
#[command(name = "matchup")]
#[command(about = "Run a single deck matchup and output JSON results", long_about = None)]
struct Args {
    /// First deck ID (e.g., broodmother_pack)
    #[arg(long)]
    deck1: String,

    /// Second deck ID (e.g., architect_fortify)
    #[arg(long)]
    deck2: String,

    /// Games per player order (total games = 2 × this value)
    #[arg(long, short = 'n', default_value = "250")]
    games: usize,

    /// Bot type for validation
    #[arg(long, default_value = "alphabeta")]
    bot: String,

    /// Alpha-Beta search depth
    #[arg(long, default_value = "6")]
    ab_depth: u32,

    /// MCTS simulations per move (only used with --bot mcts)
    #[arg(long, default_value = "100")]
    mcts_sims: u32,

    /// Random seed for reproducibility
    #[arg(long, short = 's', default_value = "42")]
    seed: u64,

    /// Output JSON to stdout
    #[arg(long)]
    json: bool,

    /// Suppress progress output (for container usage)
    #[arg(long, short = 'q')]
    quiet: bool,

    /// Path to card database
    #[arg(long, default_value = "data/cards/core_set")]
    cards: PathBuf,

    /// Path to commanders directory
    #[arg(long, default_value = "data/commanders")]
    commanders: PathBuf,

    /// Path to deck definitions directory
    #[arg(long, default_value = "data/decks")]
    decks: PathBuf,

    /// Path to weights directory
    #[arg(long, default_value = "data/weights")]
    weights: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    // Parse bot type
    let bot_type = parse_bot_type_or_exit(&args.bot);

    // Load game data
    let game_data = match GameData::load_with_overrides(
        Some(&args.cards),
        Some(&args.commanders),
        Some(&args.decks),
        Some(&args.weights),
        args.quiet,
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading game data: {}", e);
            process::exit(1);
        }
    };

    // Load archetype weights
    let archetype_weights = ArchetypeWeights::load_from_directory(&args.weights, args.quiet);

    // Get deck definitions
    let deck1 = match game_data.deck_registry.get(&args.deck1) {
        Some(d) => d.clone(),
        None => {
            eprintln!("Error: Deck '{}' not found", args.deck1);
            eprintln!("Available decks:");
            for deck in game_data.deck_registry.decks() {
                eprintln!("  - {}", deck.id);
            }
            process::exit(1);
        }
    };

    let deck2 = match game_data.deck_registry.get(&args.deck2) {
        Some(d) => d.clone(),
        None => {
            eprintln!("Error: Deck '{}' not found", args.deck2);
            eprintln!("Available decks:");
            for deck in game_data.deck_registry.decks() {
                eprintln!("  - {}", deck.id);
            }
            process::exit(1);
        }
    };

    // Create matchup
    let matchup = UnifiedMatchup::new(deck1, deck2);

    // Print header (unless quiet)
    if !args.quiet {
        let bot_desc = match bot_type {
            BotType::AlphaBeta => format!("Alpha-Beta depth {}", args.ab_depth),
            BotType::Mcts => format!("MCTS {} sims", args.mcts_sims),
            _ => format!("{:?}", bot_type),
        };
        eprintln!(
            "Running: {} vs {} ({} games × 2, {})",
            args.deck1, args.deck2, args.games, bot_desc
        );
    }

    // Create executor and run matchup
    let executor = ValidationExecutor::new(&game_data.card_db, args.mcts_sims)
        .with_bot_type(bot_type)
        .with_alphabeta_depth(args.ab_depth)
        .with_progress(false); // No progress for single matchup

    let result = executor.run_matchup(&matchup, &archetype_weights, args.games, args.seed);

    // Output results
    if args.json {
        match serde_json::to_string_pretty(&result) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("Error serializing result: {}", e);
                process::exit(1);
            }
        }
    } else {
        // Human-readable output
        println!();
        println!("=== Matchup Result ===");
        println!(
            "{} vs {}",
            result.deck1_id, result.deck2_id
        );
        println!(
            "Commanders: {} vs {}",
            result.commander1_name, result.commander2_name
        );
        println!();
        println!("Total games: {}", result.total_games);
        println!(
            "{} wins: {} ({:.1}%)",
            result.deck1_id,
            result.faction1_total_wins,
            result.faction1_win_rate * 100.0
        );
        println!(
            "{} wins: {} ({:.1}%)",
            result.deck2_id,
            result.faction2_total_wins,
            result.faction2_win_rate * 100.0
        );
        if result.draws > 0 {
            println!("Draws: {}", result.draws);
        }
        println!("Avg turns: {:.1}", result.avg_turns);
        println!("Time: {:.1}s", result.total_time_secs);
        println!();
        println!("P1 win rate: {:.1}%", result.diagnostics.p1_win_rate * 100.0);
    }
}
