//! Balance Validation CLI - Run comprehensive faction matchup testing.
//!
//! Tests all deck combinations across faction pairs (Argentum, Symbiote, Obsidion)
//! in both player orders using configurable bots with faction-specific weights.
//! Uses round-robin matchup generation to test ALL valid deck combinations.
//!
//! Default bot is Alpha-Beta (depth 6) for speed. Use --bot mcts for MCTS validation.
//!
//! Usage:
//!   cargo run --release --bin validate -- -n 100                   # Fast with Alpha-Beta
//!   cargo run --release --bin validate -- -n 100 --bot mcts        # Use MCTS instead
//!   cargo run --release --bin validate -- -n 100 --ab-depth 8      # Deeper Alpha-Beta search

use std::path::PathBuf;
use std::process;
use std::time::Instant;

use clap::Parser;

use cardgame::bots::BotType;
use cardgame::execution::{configure_thread_pool, parse_bot_type_or_exit, GameData, MatchupBuilder};
use cardgame::validation::{
    export_json, print_results, save_validation_results, ArchetypeWeights, BalanceAnalyzer,
    BalanceStatus, ValidationConfig, ValidationExecutor, ValidationResults,
};
use cardgame::version::{self, VersionInfo};

/// Balance Validation - Test faction matchup balance
#[derive(Parser, Debug)]
#[command(name = "validate")]
#[command(about = "Run comprehensive faction balance testing", long_about = None)]
struct Args {
    /// Games per matchup per player order (total = matchups * 2 * games)
    #[arg(long, short = 'n', default_value = "500")]
    games: usize,

    /// Bot type for validation (alphabeta for speed, mcts for consistency)
    #[arg(long, default_value = "alphabeta")]
    bot: String,

    /// Alpha-beta search depth (only used with --bot alphabeta)
    #[arg(long, default_value = "6")]
    ab_depth: u32,

    /// MCTS simulations per move (only used with --bot mcts)
    #[arg(long, default_value = "100")]
    mcts_sims: u32,

    /// Output JSON file path
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

    /// Output directory for validation results (creates experiments/validation/{run_id}/)
    /// If not specified, uses timestamp: YYYY-MM-DD_HHMM
    #[arg(long)]
    run_id: Option<String>,

    /// Show progress indicator
    #[arg(long)]
    progress: bool,

    /// Interactive mode - show progress spinners (deprecated, use --progress)
    #[arg(long, short = 'i', hide = true)]
    interactive: bool,

    /// Random seed for reproducibility
    #[arg(long, short = 's', default_value = "42")]
    seed: u64,

    /// Number of threads (0 = use all cores)
    #[arg(long, short = 'j', default_value = "0")]
    threads: usize,

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

    /// Run only a specific matchup (e.g., "argentum-symbiote", "argentum-obsidion", "symbiote-obsidion")
    #[arg(long, short = 'm')]
    matchup: Option<String>,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    // Handle deprecated --interactive flag
    let show_progress = args.progress || args.interactive;
    if args.interactive && !args.progress {
        eprintln!("Warning: --interactive is deprecated, use --progress instead");
    }

    // Parse bot type
    let bot_type = parse_bot_type_or_exit(&args.bot);

    // Configure thread pool
    let num_threads = configure_thread_pool(args.threads);

    // Load game data using unified loader
    let game_data = match GameData::load_with_overrides(
        Some(&args.cards),
        Some(&args.commanders),
        Some(&args.decks),
        Some(&args.weights),
        !show_progress,
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading game data: {}", e);
            process::exit(1);
        }
    };

    // Load archetype weights for bots
    let archetype_weights = ArchetypeWeights::load_from_directory(&args.weights, !show_progress);

    // Build matchups using new MatchupBuilder (preserves commander info)
    let builder = MatchupBuilder::new(&game_data.deck_registry);
    let mut matchups = builder.build_inter_faction_matchups();

    if matchups.is_empty() {
        eprintln!("Error: No valid faction matchups found. Need decks for at least 2 factions.");
        process::exit(1);
    }

    // Filter to specific matchup if requested
    if let Some(ref matchup_filter) = args.matchup {
        matchups = MatchupBuilder::filter_matchups(matchups, matchup_filter);
        if matchups.is_empty() {
            eprintln!(
                "Error: No matchup found matching '{}'. Valid options: argentum-symbiote, argentum-obsidion, symbiote-obsidion",
                matchup_filter
            );
            process::exit(1);
        }
    }

    // Print header
    println!("=== Balance Validation ===");
    println!("Version: {}", version::version_string());
    let bot_config_str = match bot_type {
        BotType::AlphaBeta => format!("Alpha-Beta depth {}", args.ab_depth),
        BotType::Mcts => format!("MCTS {} sims", args.mcts_sims),
        _ => format!("{:?}", bot_type),
    };
    println!(
        "Config: {} games/matchup, {}, {} threads",
        args.games, bot_config_str, num_threads
    );
    println!("Matchups: {} deck pairs (round-robin)", matchups.len());
    println!("Total games: {} (matchups × 2 directions × {})", matchups.len() * 2 * args.games, args.games);
    println!();

    // Run validation
    let start_time = Instant::now();
    let executor = ValidationExecutor::new(&game_data.card_db, args.mcts_sims)
        .with_bot_type(bot_type)
        .with_alphabeta_depth(args.ab_depth)
        .with_progress(show_progress);

    let matchup_results = match executor.run_all(&matchups, &archetype_weights, args.games, args.seed) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    let total_time = start_time.elapsed();

    // Analyze balance
    let analyzer = BalanceAnalyzer::new();
    let summary = analyzer.analyze(&matchup_results);

    // Create full results
    let config = ValidationConfig::new(args.games, args.mcts_sims, args.seed, num_threads)
        .with_matchup_filter(args.matchup.clone());

    let results = ValidationResults {
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: VersionInfo::current(),
        config,
        matchups: matchup_results,
        summary,
    };

    // Output results
    print_results(&results, total_time);

    // Save to timestamped directory by default (or use specified output for backward compatibility)
    if let Some(ref output_path) = args.output {
        // Legacy mode: save only JSON to specified path
        if let Err(e) = export_json(&results, output_path) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    } else {
        // New mode: save full results to timestamped directory
        if let Err(e) = save_validation_results(&results, total_time, args.run_id.as_deref()) {
            eprintln!("Error saving results: {}", e);
            process::exit(1);
        }
    }

    // Exit with appropriate code
    if results.summary.overall_status == BalanceStatus::Imbalanced {
        process::exit(1);
    }
}
