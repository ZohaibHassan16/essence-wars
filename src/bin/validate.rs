//! Balance Validation CLI - Run comprehensive faction matchup testing.
//!
//! Tests all deck combinations across faction pairs (Argentum, Symbiote, Obsidion)
//! in both player orders using MCTS agents with faction-specific weights.
//! Uses round-robin matchup generation to test ALL valid deck combinations.
//!
//! Current deck counts: Argentum (3), Symbiote (4), Obsidion (4)
//! Total matchups: 40 deck combinations (3×4 + 3×4 + 4×4)
//! Total games: matchups × 2 directions × games parameter
//!
//! Usage:
//!   cargo run --release --bin validate -- --games 100              # 8k total - quick check
//!   cargo run --release --bin validate -- --games 500 --output results.json  # 40k total - full test
//!   cargo run --release --bin validate -- --games 1500             # 120k total - comprehensive

use std::path::PathBuf;
use std::process;
use std::time::Instant;

use clap::Parser;

use cardgame::cards::CardDatabase;
use cardgame::decks::DeckRegistry;
use cardgame::execution::configure_thread_pool;
use cardgame::validation::{
    export_json, filter_matchups, print_results, save_validation_results, BalanceAnalyzer,
    BalanceStatus, FactionWeights, MatchupBuilder, ValidationConfig, ValidationExecutor,
    ValidationResults,
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

    /// MCTS simulations per move
    #[arg(long, default_value = "100")]
    mcts_sims: u32,

    /// Output JSON file path
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

    /// Output directory for validation results (creates experiments/validation/{run_id}/)
    /// If not specified, uses timestamp: YYYY-MM-DD_HHMM
    #[arg(long)]
    run_id: Option<String>,

    /// Interactive mode - show progress spinners (for terminal use)
    #[arg(long, short = 'i')]
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
    let args = Args::parse();

    // Configure thread pool
    let num_threads = configure_thread_pool(args.threads);

    // Load card database
    let card_db = match CardDatabase::load_from_directory(&args.cards) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error loading card database from {:?}: {}", args.cards, e);
            process::exit(1);
        }
    };

    // Load deck registry
    let deck_registry = match DeckRegistry::load_from_directory(&args.decks) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error loading decks from {:?}: {}", args.decks, e);
            process::exit(1);
        }
    };

    // Load faction weights
    let faction_weights = FactionWeights::load_from_directory(&args.weights, !args.interactive);

    // Build matchups (all deck combinations, round-robin)
    let builder = MatchupBuilder::new(&deck_registry, &card_db);
    let mut matchups = builder.build_all_deck_matchups();

    if matchups.is_empty() {
        eprintln!("Error: No valid faction matchups found. Need decks for at least 2 factions.");
        process::exit(1);
    }

    // Filter to specific matchup if requested
    if let Some(ref matchup_filter) = args.matchup {
        matchups = filter_matchups(matchups, matchup_filter);
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
    println!(
        "Config: {} games/matchup, {} MCTS sims, {} threads",
        args.games, args.mcts_sims, num_threads
    );
    println!("Matchups: {} deck pairs (round-robin)", matchups.len());
    println!("Total games: {} (matchups × 2 directions × {})", matchups.len() * 2 * args.games, args.games);
    println!();

    // Run validation
    let start_time = Instant::now();
    let executor = ValidationExecutor::new(&card_db, args.mcts_sims).with_progress(args.interactive);

    let matchup_results = executor.run_all(&matchups, &faction_weights, args.games, args.seed);
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
