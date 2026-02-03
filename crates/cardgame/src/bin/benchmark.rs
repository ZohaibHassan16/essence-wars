//! Thorough Balance Benchmark CLI - Statistical analysis with strong bots.
//!
//! Tests all deck combinations across faction pairs using Alpha-Beta or MCTS bots.
//! Designed for daily CI, overnight runs, and pre-release validation.
//!
//! For quick sanity checks, use `validate` instead (Greedy bot, ~1 sec).
//!
//! Usage:
//!   cargo run --release --bin benchmark -- --progress                    # Default: depth 4 (~2h for full suite)
//!   cargo run --release --bin benchmark -- --preset overnight --progress # Depth 6 (~80h, run overnight)
//!   cargo run --release --bin benchmark -- --preset release --progress   # Depth 8 (~400h, pre-release only)
//!   cargo run --release --bin benchmark -- --ab-depth 6 --progress       # Custom depth
//!   cargo run --release --bin benchmark -- --bot mcts --progress         # Use MCTS instead

use std::path::PathBuf;
use std::process;
use std::time::Instant;

use clap::Parser;

use cardgame::bots::BotType;
use cardgame::execution::{configure_thread_pool, GameData, MatchupBuilder};
use cardgame::validation::{
    export_json, export_matrix_csv, find_outliers, print_matchup_matrix, print_results,
    run_outlier_diagnostics, ArchetypeWeights, AutoDiagnoseConfig,
    BalanceAnalyzer, ValidationConfig, ValidationExecutor, ValidationResults,
};
use cardgame::version::{self, VersionInfo};

/// Thorough Balance Benchmark - Statistical analysis with strong bots
#[derive(Parser, Debug)]
#[command(name = "benchmark")]
#[command(about = "Thorough balance benchmark using Alpha-Beta or MCTS", long_about = None)]
struct Args {
    /// Preset configuration (overrides individual settings)
    /// - fast: depth 4, 50 games (~2h for full suite, daily CI)
    /// - overnight: depth 6, 200 games (~80h, weekly validation)
    /// - release: depth 8, 500 games (~400h, pre-release only)
    #[arg(long, value_parser = ["fast", "overnight", "release"])]
    preset: Option<String>,

    /// Games per matchup per player order (total = matchups × 2 × games)
    #[arg(long, short = 'n', default_value = "50")]
    games_per_matchup: usize,

    /// Bot type: alphabeta (default) or mcts
    #[arg(long, default_value = "alphabeta")]
    bot: String,

    /// Alpha-Beta search depth (only used with --bot alphabeta)
    /// Recommended: 4 (fast), 6 (strong), 8 (exhaustive)
    #[arg(long, default_value = "4")]
    ab_depth: u32,

    /// MCTS simulations per move (only used with --bot mcts)
    #[arg(long, default_value = "200")]
    mcts_sims: u32,

    /// Output JSON file path (legacy, prefer --run-id for full output)
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

    /// Output directory for benchmark results (creates experiments/benchmark/{run_id}/)
    /// If not specified, uses timestamp: YYYY-MM-DD_HHMM
    #[arg(long)]
    run_id: Option<String>,

    /// Show progress indicator (recommended for long runs)
    #[arg(long)]
    progress: bool,

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

    /// Run only a specific faction matchup (e.g., "argentum-symbiote", "argentum-obsidion", "symbiote-obsidion")
    #[arg(long, short = 'm')]
    matchup: Option<String>,

    /// Run only matchups where this deck is player 1 (for parallelization)
    /// Use deck ID, e.g., "artificer_tokens", "broodmother_pack"
    #[arg(long)]
    deck1: Option<String>,

    /// Run only matchups where this deck is player 2 (for parallelization)
    /// Use deck ID, e.g., "artificer_tokens", "broodmother_pack"
    #[arg(long)]
    deck2: Option<String>,

    /// Print matchup matrix after results
    #[arg(long)]
    matrix: bool,

    /// Export matchup matrix to CSV file
    #[arg(long)]
    matrix_csv: Option<PathBuf>,

    /// Auto-diagnose outlier decks after benchmark
    #[arg(long)]
    auto_diagnose: bool,

    /// Win rate delta from 50% to trigger diagnosis (default: 0.10 = 40-60% range)
    #[arg(long, default_value = "0.10")]
    outlier_threshold: f64,

    /// Number of games for each diagnostic run
    #[arg(long, default_value = "50")]
    diagnose_games: usize,
}

fn parse_bot_type(bot_str: &str) -> Result<BotType, String> {
    match bot_str.to_lowercase().as_str() {
        "alphabeta" | "alpha-beta" | "ab" => Ok(BotType::AlphaBeta),
        "mcts" => Ok(BotType::Mcts),
        _ => Err(format!(
            "Invalid bot type '{}'. Valid options: alphabeta, mcts",
            bot_str
        )),
    }
}

fn main() {
    env_logger::init();
    let mut args = Args::parse();

    // Apply preset if specified
    if let Some(ref preset) = args.preset {
        match preset.as_str() {
            "fast" => {
                args.ab_depth = 4;
                args.games_per_matchup = 50;
                args.mcts_sims = 200;
            }
            "overnight" => {
                args.ab_depth = 6;
                args.games_per_matchup = 200;
                args.mcts_sims = 500;
            }
            "release" => {
                args.ab_depth = 8;
                args.games_per_matchup = 500;
                args.mcts_sims = 1000;
            }
            _ => {
                eprintln!("Error: Invalid preset '{}'. Valid options: fast, overnight, release", preset);
                process::exit(1);
            }
        }
    }

    // Parse bot type (only alphabeta or mcts allowed)
    let bot_type = match parse_bot_type(&args.bot) {
        Ok(bt) => bt,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    // Configure thread pool
    let num_threads = configure_thread_pool(args.threads);

    // Load game data using unified loader
    let game_data = match GameData::load_with_overrides(
        Some(&args.cards),
        Some(&args.commanders),
        Some(&args.decks),
        Some(&args.weights),
        !args.progress,
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading game data: {}", e);
            process::exit(1);
        }
    };

    // Load archetype weights for bots
    let archetype_weights = ArchetypeWeights::load_from_directory(&args.weights, !args.progress);

    // Build matchups using MatchupBuilder (preserves commander info)
    let builder = MatchupBuilder::new(&game_data.deck_registry);
    let mut matchups = builder.build_inter_faction_matchups();

    if matchups.is_empty() {
        eprintln!("Error: No valid faction matchups found. Need decks for at least 2 factions.");
        process::exit(1);
    }

    // Filter to specific faction matchup if requested
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

    // Filter to specific deck1 if requested (for parallelization)
    if let Some(ref deck1_filter) = args.deck1 {
        matchups.retain(|m| m.deck1.id == *deck1_filter);
        if matchups.is_empty() {
            let available: Vec<_> = game_data.deck_registry.decks().map(|d| d.id.as_str()).collect();
            eprintln!(
                "Error: No matchups found with deck1='{}'. Available decks: {}",
                deck1_filter,
                available.join(", ")
            );
            process::exit(1);
        }
    }

    // Filter to specific deck2 if requested (for parallelization)
    if let Some(ref deck2_filter) = args.deck2 {
        matchups.retain(|m| m.deck2.id == *deck2_filter);
        if matchups.is_empty() {
            let available: Vec<_> = game_data.deck_registry.decks().map(|d| d.id.as_str()).collect();
            eprintln!(
                "Error: No matchups found with deck2='{}'. Available decks: {}",
                deck2_filter,
                available.join(", ")
            );
            process::exit(1);
        }
    }

    // Print header
    println!("=== Thorough Balance Benchmark ===");
    println!("Version: {}", version::version_string());
    let bot_config_str = match bot_type {
        BotType::AlphaBeta => format!("Alpha-Beta depth {}", args.ab_depth),
        BotType::Mcts => format!("MCTS {} sims", args.mcts_sims),
        _ => unreachable!(),
    };
    println!(
        "Config: {} games/matchup, {}, {} threads",
        args.games_per_matchup, bot_config_str, num_threads
    );
    println!("Matchups: {} deck pairs (round-robin)", matchups.len());
    let total_games = matchups.len() * 2 * args.games_per_matchup;
    println!(
        "Total games: {} (matchups × 2 directions × {})",
        total_games, args.games_per_matchup
    );

    // Estimate time (based on empirical measurements from Feb 2026)
    // Hardware baseline: 16-thread CPU (Ryzen/similar), parallel execution
    let estimated_time = match bot_type {
        BotType::AlphaBeta => {
            // Empirical measurements (100 games, 16 threads):
            // - Depth 4: 269ms/game effective (5.4s wall-clock / 2 bots)
            // - Depth 6: 10.8s/game effective (78s wall-clock / 100 games for MCTS matchup)
            // Conservative estimates accounting for parallelization overhead:
            let secs_per_game = match args.ab_depth {
                d if d <= 3 => 2.0,    // Very fast
                d if d <= 4 => 5.0,    // Measured: 2.7s, use 5s for safety
                d if d <= 5 => 12.0,   // Interpolated
                d if d <= 6 => 25.0,   // Measured: ~11s, use 25s for safety
                d if d <= 7 => 60.0,   // Exponential growth
                d if d <= 8 => 120.0,  // Depth 8 is 4-5× slower than depth 6
                d if d <= 9 => 300.0,  // Very slow
                _ => 600.0,            // Depth 10+ is extremely slow
            };
            total_games as f64 * secs_per_game / num_threads as f64
        }
        BotType::Mcts => {
            // MCTS scales roughly linearly with simulations
            // Measured: 500 sims ≈ 425ms per decision
            let secs_per_game = (args.mcts_sims as f64 * 0.001).max(5.0);
            total_games as f64 * secs_per_game / num_threads as f64
        }
        _ => 0.0,
    };
    if estimated_time > 60.0 {
        let hours = estimated_time / 3600.0;
        if hours >= 1.0 {
            println!("Estimated time: ~{:.1} hours", hours);
        } else {
            println!("Estimated time: ~{:.0} minutes", estimated_time / 60.0);
        }
    }
    println!();

    // Run benchmark
    let start_time = Instant::now();
    let executor = ValidationExecutor::new(&game_data.card_db, args.mcts_sims)
        .with_bot_type(bot_type.clone())
        .with_alphabeta_depth(args.ab_depth)
        .with_progress(args.progress);

    let matchup_results = match executor.run_all(
        &matchups,
        &archetype_weights,
        args.games_per_matchup,
        args.seed,
    ) {
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
    let config = ValidationConfig::new(args.games_per_matchup, args.mcts_sims, args.seed, num_threads)
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

    // Print matchup matrix if requested
    if args.matrix {
        print_matchup_matrix(&results.matchups);
    }

    // Export matrix to CSV if requested
    if let Some(ref csv_path) = args.matrix_csv {
        if let Err(e) = export_matrix_csv(&results.matchups, csv_path) {
            eprintln!("Error exporting matrix CSV: {}", e);
            process::exit(1);
        }
        println!("Matrix exported to {:?}", csv_path);
    }

    // Save to timestamped directory by default
    let output_dir = if let Some(ref output_path) = args.output {
        // Legacy mode: save only JSON to specified path
        if let Err(e) = export_json(&results, output_path) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
        None
    } else {
        // Save to experiments/benchmark/ instead of experiments/validation/
        let run_id = args.run_id.as_deref().unwrap_or("");
        match save_benchmark_results(&results, total_time, if run_id.is_empty() { None } else { Some(run_id) }) {
            Ok(dir) => Some(dir),
            Err(e) => {
                eprintln!("Error saving results: {}", e);
                process::exit(1);
            }
        }
    };

    // Auto-diagnose outlier decks if requested
    if args.auto_diagnose {
        let outliers = find_outliers(
            &results.summary,
            &game_data.deck_registry,
            args.outlier_threshold,
        );

        if outliers.is_empty() {
            println!(
                "\nNo outlier decks found (all within {:.0}-{:.0}% win rate range).",
                (0.5 - args.outlier_threshold) * 100.0,
                (0.5 + args.outlier_threshold) * 100.0
            );
        } else {
            println!(
                "\n=== Auto-Diagnosing {} Outlier Deck(s) ===",
                outliers.len()
            );

            let diagnose_config = AutoDiagnoseConfig {
                games: args.diagnose_games,
                bot_type: bot_type.clone(),
                alphabeta_depth: args.ab_depth,
                mcts_sims: args.mcts_sims,
                seed: args.seed,
            };

            // Determine output directory for diagnostics
            let diag_output_dir = output_dir.unwrap_or_else(|| {
                args.output
                    .as_ref()
                    .and_then(|p| p.parent())
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("experiments/benchmark/diagnostics"))
            });

            if let Err(e) = run_outlier_diagnostics(
                &outliers,
                &game_data.deck_registry,
                &game_data.card_db,
                &diagnose_config,
                &diag_output_dir,
                args.progress,
            ) {
                eprintln!("Error running auto-diagnostics: {}", e);
                process::exit(1);
            }
        }
    }
}

/// Save benchmark results to experiments/benchmark/{run_id}/
fn save_benchmark_results(
    results: &ValidationResults,
    total_time: std::time::Duration,
    run_id: Option<&str>,
) -> Result<PathBuf, String> {
    use std::fs;
    use std::io::Write;

    // Generate run_id from timestamp if not provided
    let run_id = run_id
        .map(String::from)
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d_%H%M").to_string());

    // Create output directory
    let output_dir = PathBuf::from(format!("experiments/benchmark/{}", run_id));
    fs::create_dir_all(&output_dir).map_err(|e| format!("Failed to create directory: {}", e))?;

    // Save JSON results
    let json_path = output_dir.join("results.json");
    let json_content =
        serde_json::to_string_pretty(results).map_err(|e| format!("JSON error: {}", e))?;
    fs::write(&json_path, json_content).map_err(|e| format!("Write error: {}", e))?;

    // Save human-readable summary
    let summary_path = output_dir.join("summary.txt");
    let mut summary_file =
        fs::File::create(&summary_path).map_err(|e| format!("Create error: {}", e))?;

    writeln!(summary_file, "=== Benchmark Summary ===").ok();
    writeln!(summary_file, "Timestamp: {}", results.timestamp).ok();
    writeln!(summary_file, "Version: {}", results.version.version).ok();
    writeln!(summary_file, "Git: {}", results.version.git_hash.as_deref().unwrap_or("unknown")).ok();
    writeln!(summary_file, "Total time: {:.1}s", total_time.as_secs_f64()).ok();
    writeln!(summary_file).ok();
    writeln!(
        summary_file,
        "Games per matchup: {}",
        results.config.games_per_matchup
    )
    .ok();
    writeln!(summary_file, "Seed: {}", results.config.seed).ok();
    writeln!(summary_file).ok();
    writeln!(
        summary_file,
        "P1 Win Rate: {:.1}%",
        results.summary.p1_win_rate * 100.0
    )
    .ok();
    writeln!(
        summary_file,
        "Max Faction Delta: {:.1}%",
        results.summary.max_faction_delta * 100.0
    )
    .ok();
    writeln!(summary_file, "Faction Status: {:?}", results.summary.faction_status).ok();

    // Save config
    let config_path = output_dir.join("config.toml");
    let config_content = format!(
        r#"# Benchmark Configuration
[benchmark]
games_per_matchup = {}
seed = {}
threads = {}
timestamp = "{}"
"#,
        results.config.games_per_matchup,
        results.config.seed,
        results.config.threads,
        results.timestamp
    );
    fs::write(&config_path, config_content).map_err(|e| format!("Write error: {}", e))?;

    println!();
    println!("💾 Benchmark results saved to: {}", output_dir.display());
    println!("   📄 results.json - Full JSON data");
    println!("   📝 summary.txt  - Human-readable summary");
    println!("   ⚙️  config.toml  - Run configuration");

    Ok(output_dir)
}
