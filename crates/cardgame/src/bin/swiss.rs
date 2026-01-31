//! Swiss Tournament CLI - Run Swiss-system tournaments between decks.
//!
//! Usage:
//!   cargo run --release --bin swiss -- --games 20 --progress
//!   cargo run --release --bin swiss -- --faction argentum --games 30
//!   cargo run --release --bin swiss -- --bot mcts --mcts-sims 200 --rounds 5
//!   cargo run --release --bin swiss -- --games 50 --output tournament.json

use std::path::PathBuf;
use std::process;

use clap::Parser;

use cardgame::arena::{SwissConfig, SwissTournament};
use cardgame::bots::{AlphaBetaConfig, MctsConfig};
use cardgame::decks::Faction;
use cardgame::execution::{configure_thread_pool, parse_bot_type_or_exit, resolve_seed, GameData};

/// Swiss Tournament - Run Swiss-system tournaments between decks
#[derive(Parser, Debug)]
#[command(name = "swiss")]
#[command(about = "Run a Swiss-system tournament between decks", long_about = None)]
struct Args {
    /// Bot type for all participants (greedy, mcts, alphabeta)
    #[arg(long, default_value = "greedy")]
    bot: String,

    /// Number of games per match
    #[arg(long, short = 'n', default_value = "20")]
    games: usize,

    /// Number of rounds (default: auto-calculated from participant count)
    #[arg(long, short = 'r')]
    rounds: Option<usize>,

    /// Filter to specific faction (argentum, symbiote, obsidion)
    #[arg(long, short = 'f')]
    faction: Option<String>,

    /// Random seed for reproducibility
    #[arg(long, short = 's')]
    seed: Option<u64>,

    /// Show progress during matches
    #[arg(long)]
    progress: bool,

    /// Export results to JSON file
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

    /// Path to custom weights file
    #[arg(long)]
    weights: Option<PathBuf>,

    /// MCTS simulations (if using mcts bot)
    #[arg(long, default_value = "500")]
    mcts_sims: u32,

    /// Alpha-Beta depth (if using alphabeta bot)
    #[arg(long, default_value = "6")]
    ab_depth: u32,

    /// Number of threads (0 = use all cores)
    #[arg(long, short = 'j', default_value = "0")]
    threads: usize,

    /// Path to card database
    #[arg(long, default_value = "data/cards/core_set")]
    cards: PathBuf,

    /// Path to commander definitions directory
    #[arg(long, default_value = "data/commanders")]
    commanders: PathBuf,

    /// Path to deck definitions directory
    #[arg(long, default_value = "data/decks")]
    decks: PathBuf,

    /// List available decks and exit
    #[arg(long)]
    list_decks: bool,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    // Configure thread pool
    let _num_threads = configure_thread_pool(args.threads);

    // Load game data
    let game_data = match GameData::load_with_overrides(
        Some(&args.cards),
        Some(&args.commanders),
        Some(&args.decks),
        None,
        true, // quiet
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading game data: {}", e);
            process::exit(1);
        }
    };

    // Handle --list-decks
    if args.list_decks {
        println!("Available decks:");
        if game_data.deck_registry.is_empty() {
            println!("  (no decks found in {:?})", args.decks);
        } else {
            for deck in game_data.deck_registry.decks() {
                let commander_info = game_data
                    .card_db
                    .get_commander(cardgame::types::CardId(deck.commander))
                    .map(|c| format!(" [{}]", c.name))
                    .unwrap_or_default();
                let faction = deck
                    .faction()
                    .map(|f| format!("{:?}", f))
                    .unwrap_or_else(|| "Unknown".to_string());
                println!(
                    "  {} - {} ({}) {}",
                    deck.id, deck.name, faction, commander_info
                );
            }
        }
        return;
    }

    // Parse bot type
    let bot_type = parse_bot_type_or_exit(&args.bot);

    // Parse faction filter
    let faction_filter = args.faction.as_ref().map(|s| {
        match s.to_lowercase().as_str() {
            "argentum" | "a" => Faction::Argentum,
            "symbiote" | "s" => Faction::Symbiote,
            "obsidion" | "o" => Faction::Obsidion,
            _ => {
                eprintln!(
                    "Invalid faction: '{}'. Use: argentum, symbiote, or obsidion",
                    s
                );
                process::exit(1);
            }
        }
    });

    // Count participants
    let participant_count = match faction_filter {
        Some(f) => game_data.deck_registry.decks_for_faction(f).len(),
        None => game_data.deck_registry.len(),
    };

    if participant_count < 2 {
        eprintln!(
            "Error: Need at least 2 participants for a tournament (found {})",
            participant_count
        );
        process::exit(1);
    }

    // Calculate rounds
    let rounds = args
        .rounds
        .unwrap_or_else(|| SwissConfig::recommended_rounds(participant_count));

    // Resolve seed
    let seed = resolve_seed(args.seed);

    // Load weights if specified
    let weights = args.weights.as_ref().map(|path| {
        match cardgame::bots::BotWeights::load(path) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Error loading weights from {:?}: {}", path, e);
                process::exit(1);
            }
        }
    });

    // Create tournament config
    let config = SwissConfig::new(rounds, args.games, bot_type, seed)
        .with_progress(args.progress)
        .with_faction(faction_filter)
        .with_weights(weights)
        .with_mcts_config(MctsConfig {
            simulations: args.mcts_sims,
            exploration: 1.414,
            max_rollout_depth: 100,
            parallel_trees: 1,
            leaf_rollouts: 1,
        })
        .with_alphabeta_config(AlphaBetaConfig::with_depth(args.ab_depth));

    // Create and run tournament
    let mut tournament = SwissTournament::new(&game_data.deck_registry, config);
    let result = tournament.run(&game_data.card_db);

    // Print final summary
    println!();
    println!("{}", "=".repeat(60));
    println!("FINAL STANDINGS");
    println!("{}", "=".repeat(60));
    println!();

    for standing in &result.standings {
        println!(
            "{:>2}. {} ({}) - {} pts, {:.1}% win rate",
            standing.rank,
            standing.deck_name,
            standing.commander_name,
            standing.score,
            standing.win_rate * 100.0
        );
    }

    println!();
    println!(
        "Tournament completed in {:.1}s",
        result.total_duration_secs
    );

    // Export if requested
    if let Some(ref path) = args.output {
        match result.to_json() {
            Ok(json) => {
                if let Err(e) = std::fs::write(path, json) {
                    eprintln!("Error writing results to {:?}: {}", path, e);
                    process::exit(1);
                }
                println!("Results exported to {:?}", path);
            }
            Err(e) => {
                eprintln!("Error serializing results: {}", e);
                process::exit(1);
            }
        }
    }
}
