//! P1/P2 Asymmetry Diagnostic Tool
//!
//! Collects detailed per-turn statistics to analyze why P1 has a lower win rate.
//! Outputs data for analysis of resource curves, tempo, and game progression.
//!
//! Default bot is Greedy for speed. Use --bot alphabeta or --bot mcts for accurate analysis.
//!
//! Usage:
//!   cargo run --release --bin diagnose -- -n 200                   # Fast with Greedy
//!   cargo run --release --bin diagnose -- -n 200 --bot alphabeta   # Accurate with Alpha-Beta
//!   cargo run --release --bin diagnose -- -n 200 --bot mcts        # Use MCTS
//!   cargo run --release --bin diagnose -- -n 200 --export csv -o ./diagnostics

use std::path::PathBuf;
use std::process;

use clap::Parser;

use cardgame::bots::{AlphaBetaConfig, BotType, MctsConfig};
use cardgame::diagnostics::{
    export_csv, export_json, AggregatedStats, DiagnosticConfig, DiagnosticRunner, ExportFormat,
    print_report,
};
use cardgame::execution::GameData;

/// P1/P2 Asymmetry Diagnostic Tool
#[derive(Parser, Debug)]
#[command(name = "diagnose")]
#[command(about = "Analyze P1/P2 asymmetry in card game matches")]
struct Args {
    /// Number of games to run (positional, deprecated - use -n instead)
    #[arg(default_value = "200")]
    games_positional: usize,

    /// Number of games to run
    #[arg(long, short = 'n')]
    games: Option<usize>,

    /// Bot type (greedy for quick tests, alphabeta/mcts for accurate analysis)
    #[arg(long, default_value = "greedy")]
    bot: String,

    /// Alpha-beta search depth (only used with --bot alphabeta)
    #[arg(long, default_value = "6")]
    ab_depth: u32,

    /// MCTS simulations per move (only used with --bot mcts)
    #[arg(long, default_value = "100")]
    mcts_sims: u32,

    /// Export format (csv, json, or all)
    #[arg(long, short = 'e')]
    export: Option<String>,

    /// Output path for export (directory for CSV, file for JSON)
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

    /// Include per-turn data in export (can be large)
    #[arg(long)]
    include_turns: bool,

    /// Deck ID to use
    #[arg(long, default_value = "broodmother_pack")]
    deck: String,

    /// Random seed for reproducibility
    #[arg(long, short = 's', default_value = "42")]
    seed: u64,

    /// Show progress during execution
    #[arg(long, short = 'p')]
    progress: bool,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    // Handle deprecated positional argument
    let num_games = if let Some(n) = args.games {
        n
    } else {
        // If positional arg was explicitly provided (not default), warn about deprecation
        if args.games_positional != 200 {
            eprintln!("Warning: positional games argument is deprecated, use -n/--games instead");
        }
        args.games_positional
    };

    // Parse bot type
    let bot_type: BotType = args.bot.parse().unwrap_or_else(|_| {
        eprintln!(
            "Error: Invalid bot type '{}'. Valid options: greedy, alphabeta, mcts, random",
            args.bot
        );
        process::exit(1);
    });

    // Warn about using non-competitive bots for serious analysis
    if !bot_type.is_competitive() {
        eprintln!("Note: Using {} bot. For accurate analysis, use --bot alphabeta or --bot mcts", args.bot);
    }

    // Build bot configuration string for display
    let bot_config_str = match bot_type {
        BotType::AlphaBeta => format!("Alpha-Beta depth {}", args.ab_depth),
        BotType::Mcts => format!("MCTS {} sims", args.mcts_sims),
        _ => format!("{:?}", bot_type),
    };

    println!("P1/P2 Asymmetry Diagnostic Tool");
    println!("================================");
    println!(
        "Running {} games with {} vs {}...\n",
        num_games, bot_config_str, bot_config_str
    );

    // Load game data using unified loader
    let game_data = match GameData::load_default(!args.progress) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading game data: {}", e);
            process::exit(1);
        }
    };

    // Get the specified deck (with its commander)
    let deck = match game_data.deck_registry.get(&args.deck) {
        Some(d) => d.clone(),
        None => {
            eprintln!("Error: deck '{}' not found", args.deck);
            eprintln!("Available decks:");
            for d in game_data.deck_registry.decks() {
                eprintln!("  - {} (commander: {})", d.id, d.commander);
            }
            process::exit(1);
        }
    };

    // Show deck info including commander
    if let Some(commander) = game_data.card_db.get_commander(cardgame::types::CardId(deck.commander)) {
        println!("Deck: {} (Commander: {})", deck.name, commander.name);
    } else {
        println!("Deck: {} (Commander ID: {})", deck.name, deck.commander);
    }
    println!();

    // Build MCTS and Alpha-Beta configs
    let mcts_config = MctsConfig {
        simulations: args.mcts_sims,
        exploration: 1.414,
        max_rollout_depth: 100,
        parallel_trees: 1,
        leaf_rollouts: 1,
    };
    let alphabeta_config = AlphaBetaConfig::with_depth(args.ab_depth);

    // Configure diagnostics using deck definition (includes commander)
    let config = DiagnosticConfig::new(deck, num_games)
        .with_bots(bot_type.clone(), bot_type)
        .with_mcts_config(mcts_config)
        .with_alphabeta_config(alphabeta_config)
        .with_seed(args.seed)
        .with_progress(args.progress);

    // Run diagnostic games
    let runner = DiagnosticRunner::new(&game_data.card_db);

    let diagnostics = match runner.run(&config) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

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
