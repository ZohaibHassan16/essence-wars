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
    analyze_critical_turns, export_csv, export_json, print_comparative_report,
    print_critical_turns_report, AggregatedStats, CriticalTurnConfig, DiagnosticConfig,
    DiagnosticRunner, ExportFormat, print_report,
};
use cardgame::execution::{parse_bot_type_or_exit, GameData};

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

    /// First deck ID (or only deck for mirror match)
    #[arg(long, default_value = "broodmother_pack")]
    deck1: String,

    /// Second deck ID (optional, for comparative mode)
    #[arg(long)]
    deck2: Option<String>,

    /// Legacy deck argument (deprecated, use --deck1 instead)
    #[arg(long, hide = true)]
    deck: Option<String>,

    /// Random seed for reproducibility
    #[arg(long, short = 's', default_value = "42")]
    seed: u64,

    /// Show progress during execution
    #[arg(long, short = 'p')]
    progress: bool,

    /// Analyze critical turns (game-deciding moments)
    #[arg(long)]
    critical_turns: bool,

    /// Life swing threshold for critical turn detection (default: 5)
    #[arg(long, default_value = "5")]
    life_swing_threshold: i32,

    /// Board advantage swing threshold for critical turn detection (default: 3.0)
    #[arg(long, default_value = "3.0")]
    board_swing_threshold: f64,
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

    // Parse bot type using shared helper
    let bot_type = parse_bot_type_or_exit(&args.bot);

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

    // Handle legacy --deck argument
    let deck1_id = args.deck.clone().unwrap_or_else(|| args.deck1.clone());
    if args.deck.is_some() {
        eprintln!("Warning: --deck is deprecated, use --deck1 instead");
    }

    // Get deck 1
    let deck1 = match game_data.deck_registry.get(&deck1_id) {
        Some(d) => d.clone(),
        None => {
            eprintln!("Error: deck '{}' not found", deck1_id);
            eprintln!("Available decks:");
            for d in game_data.deck_registry.decks() {
                eprintln!("  - {} (commander: {})", d.id, d.commander);
            }
            process::exit(1);
        }
    };

    // Get deck 2 (same as deck 1 for mirror match, or different for comparative mode)
    let is_comparative = args.deck2.is_some();
    let deck2 = if let Some(ref deck2_id) = args.deck2 {
        match game_data.deck_registry.get(deck2_id) {
            Some(d) => d.clone(),
            None => {
                eprintln!("Error: deck '{}' not found", deck2_id);
                eprintln!("Available decks:");
                for d in game_data.deck_registry.decks() {
                    eprintln!("  - {} (commander: {})", d.id, d.commander);
                }
                process::exit(1);
            }
        }
    } else {
        deck1.clone()
    };

    // Show deck info
    let deck1_commander_name = game_data
        .card_db
        .get_commander(cardgame::types::CardId(deck1.commander))
        .map(|c| c.name.clone())
        .unwrap_or_else(|| format!("ID {}", deck1.commander));

    if is_comparative {
        let deck2_commander_name = game_data
            .card_db
            .get_commander(cardgame::types::CardId(deck2.commander))
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("ID {}", deck2.commander));
        println!("Comparative Mode: {} vs {}", deck1.name, deck2.name);
        println!("  P1: {} ({})", deck1.name, deck1_commander_name);
        println!("  P2: {} ({})", deck2.name, deck2_commander_name);
    } else {
        println!("Mirror Match: {} (Commander: {})", deck1.name, deck1_commander_name);
    }
    println!();

    // Build MCTS and Alpha-Beta configs
    let mcts_config = MctsConfig {
        simulations: args.mcts_sims,
        exploration: 1.414,
        max_rollout_depth: 100,
        parallel_trees: 1,
        leaf_rollouts: 1,
        ..MctsConfig::default()
    };
    let alphabeta_config = AlphaBetaConfig::with_depth(args.ab_depth);

    // Configure diagnostics using deck definitions (includes commanders)
    let config = DiagnosticConfig::new(deck1.clone(), num_games)
        .with_decks(deck1.clone(), deck2.clone())
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

    // Print appropriate report based on mode
    if is_comparative {
        print_comparative_report(&stats, &deck1.name, &deck2.name);
    } else {
        print_report(&stats);
    }

    // Critical turn analysis if requested
    if args.critical_turns {
        let ct_config = CriticalTurnConfig::new(
            args.life_swing_threshold,
            args.board_swing_threshold,
        );
        let ct_stats = analyze_critical_turns(&diagnostics, &ct_config);
        print_critical_turns_report(&ct_stats, &ct_config);
    }
}
