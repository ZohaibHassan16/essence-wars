//! Card Statistics CLI - Analyze per-card win contribution and usage rates.
//!
//! Tracks which cards correlate with winning games, enabling identification of
//! problematic cards for balance analysis.
//!
//! Usage:
//!   cargo run --release --bin card_stats -- --deck broodmother_pack -n 200
//!   cargo run --release --bin card_stats -- --deck1 broodmother_pack --deck2 architect_fortify -n 200
//!   cargo run --release --bin card_stats -- --mode cross-deck -n 50

use std::path::PathBuf;
use std::process;

use clap::Parser;

use cardgame::bots::{create_bot, AlphaBetaConfig, BotType, MctsConfig};
use cardgame::engine::GameEngine;
use cardgame::execution::{
    configure_thread_pool, parse_bot_type_or_exit, run_game_loop, GameData, GameLoopConfig,
    GameSeeds, MatchupBuilder,
};
use cardgame::stats::{
    export_csv, export_json, export_synergy_csv, export_synergy_json, print_report,
    print_synergy_report, CardStatsCallback, CardStatsCollector, ReportConfig,
};

/// Card Statistics - Analyze per-card win contribution
#[derive(Parser, Debug)]
#[command(name = "card_stats")]
#[command(about = "Analyze per-card win contribution and usage rates")]
struct Args {
    /// Number of games to run
    #[arg(long, short = 'n', default_value = "100")]
    games: usize,

    /// Analysis mode: mirror, matchup, cross-deck
    #[arg(long, default_value = "mirror")]
    mode: String,

    /// Deck ID for mirror match analysis
    #[arg(long, default_value = "broodmother_pack")]
    deck: String,

    /// First deck for matchup analysis
    #[arg(long)]
    deck1: Option<String>,

    /// Second deck for matchup analysis
    #[arg(long)]
    deck2: Option<String>,

    /// Bot type (greedy, mcts, alphabeta)
    #[arg(long, default_value = "greedy")]
    bot: String,

    /// MCTS simulations per move
    #[arg(long, default_value = "100")]
    mcts_sims: u32,

    /// Alpha-Beta search depth
    #[arg(long, default_value = "6")]
    ab_depth: u32,

    /// Minimum games played for card to be included in report
    #[arg(long, default_value = "5")]
    min_plays: u32,

    /// Number of top/bottom cards to show
    #[arg(long, default_value = "10")]
    top_n: usize,

    /// Export to CSV file
    #[arg(long)]
    csv: Option<PathBuf>,

    /// Export to JSON file
    #[arg(long)]
    json: Option<PathBuf>,

    /// Enable synergy analysis (track card pair co-occurrence)
    #[arg(long)]
    synergy: bool,

    /// Minimum co-occurrence for synergy pairs to be included
    #[arg(long, default_value = "5")]
    min_cooccur: u32,

    /// Number of top synergy/anti-synergy pairs to show
    #[arg(long, default_value = "10")]
    synergy_top_n: usize,

    /// Export synergy data to separate CSV file
    #[arg(long)]
    synergy_csv: Option<PathBuf>,

    /// Export synergy data to separate JSON file
    #[arg(long)]
    synergy_json: Option<PathBuf>,

    /// Random seed for reproducibility
    #[arg(long, short = 's', default_value = "42")]
    seed: u64,

    /// Number of threads (0 = all cores)
    #[arg(long, short = 'j', default_value = "0")]
    threads: usize,

    /// Show progress indicator
    #[arg(long, short = 'p')]
    progress: bool,

    /// Path to card database
    #[arg(long, default_value = "data/cards/core_set")]
    cards: PathBuf,

    /// Path to commanders directory
    #[arg(long, default_value = "data/commanders")]
    commanders: PathBuf,

    /// Path to deck definitions directory
    #[arg(long, default_value = "data/decks")]
    decks: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    // Parse bot type
    let bot_type = parse_bot_type_or_exit(&args.bot);

    // Configure thread pool
    let _num_threads = configure_thread_pool(args.threads);

    // Load game data
    let game_data = match GameData::load_with_overrides(
        Some(&args.cards),
        Some(&args.commanders),
        Some(&args.decks),
        None, // No weights needed
        !args.progress,
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading game data: {}", e);
            process::exit(1);
        }
    };

    // Determine mode and decks
    let (mode_str, deck_info, matchups) = match args.mode.as_str() {
        "mirror" => {
            let deck = match game_data.deck_registry.get(&args.deck) {
                Some(d) => d.clone(),
                None => {
                    eprintln!("Error: deck '{}' not found", args.deck);
                    list_available_decks(&game_data);
                    process::exit(1);
                }
            };
            (
                "Mirror Match".to_string(),
                deck.name.clone(),
                vec![(deck.clone(), deck)],
            )
        }
        "matchup" => {
            let deck1_id = args.deck1.as_ref().unwrap_or(&args.deck);
            let deck2_id = match &args.deck2 {
                Some(d) => d,
                None => {
                    eprintln!("Error: --deck2 required for matchup mode");
                    process::exit(1);
                }
            };

            let deck1 = match game_data.deck_registry.get(deck1_id) {
                Some(d) => d.clone(),
                None => {
                    eprintln!("Error: deck '{}' not found", deck1_id);
                    list_available_decks(&game_data);
                    process::exit(1);
                }
            };

            let deck2 = match game_data.deck_registry.get(deck2_id) {
                Some(d) => d.clone(),
                None => {
                    eprintln!("Error: deck '{}' not found", deck2_id);
                    list_available_decks(&game_data);
                    process::exit(1);
                }
            };

            (
                "Matchup".to_string(),
                format!("{} vs {}", deck1.name, deck2.name),
                vec![(deck1, deck2)],
            )
        }
        "cross-deck" => {
            let builder = MatchupBuilder::new(&game_data.deck_registry);
            let unified_matchups = builder.build_inter_faction_matchups();
            let matchups: Vec<_> = unified_matchups
                .into_iter()
                .map(|m| (m.deck1, m.deck2))
                .collect();
            (
                "Cross-Deck".to_string(),
                format!("{} matchups", matchups.len()),
                matchups,
            )
        }
        _ => {
            eprintln!(
                "Error: unknown mode '{}'. Use: mirror, matchup, or cross-deck",
                args.mode
            );
            process::exit(1);
        }
    };

    // Build bot configs
    let mcts_config = MctsConfig {
        simulations: args.mcts_sims,
        exploration: 1.414,
        max_rollout_depth: 100,
        parallel_trees: 1,
        leaf_rollouts: 1,
        ..MctsConfig::default()
    };
    let alphabeta_config = AlphaBetaConfig::with_depth(args.ab_depth);

    // Print header
    println!("=== Card Statistics Analysis ===");
    println!("Mode: {} ({})", mode_str, deck_info);
    println!("Games per matchup: {}", args.games);
    println!(
        "Bot: {}",
        match bot_type {
            BotType::AlphaBeta => format!("Alpha-Beta depth {}", args.ab_depth),
            BotType::Mcts => format!("MCTS {} sims", args.mcts_sims),
            _ => format!("{:?}", bot_type),
        }
    );
    println!();

    // Run games and collect statistics
    let mut collector = CardStatsCollector::with_synergy(args.synergy);
    let total_games = matchups.len() * args.games;
    let mut games_completed = 0;

    for (matchup_idx, (deck1, deck2)) in matchups.iter().enumerate() {
        for game_idx in 0..args.games {
            let seeds = GameSeeds::for_matchup(args.seed, matchup_idx, game_idx, false);

            // Create engine and start game
            let mut engine = GameEngine::new(&game_data.card_db);
            if let Err(e) = engine.start_game(deck1, deck2, seeds.game) {
                eprintln!("Error starting game: {}", e);
                continue;
            }

            // Create bots
            let mut bot1 = create_bot(
                &game_data.card_db,
                &bot_type,
                None,
                &mcts_config,
                &alphabeta_config,
                seeds.bot1,
            );
            let mut bot2 = create_bot(
                &game_data.card_db,
                &bot_type,
                None,
                &mcts_config,
                &alphabeta_config,
                seeds.bot2,
            );

            // Create callback and run game
            let mut callback = CardStatsCallback::new(&mut collector);
            let config = GameLoopConfig::new(seeds.game);
            run_game_loop(
                &mut engine,
                &mut *bot1,
                &mut *bot2,
                &config,
                &mut callback,
            );

            games_completed += 1;

            // Progress indicator
            if args.progress && games_completed % 10 == 0 {
                eprint!(
                    "\rProgress: {}/{} games ({:.1}%)",
                    games_completed,
                    total_games,
                    games_completed as f64 / total_games as f64 * 100.0
                );
            }
        }
    }

    if args.progress {
        eprintln!(); // Clear progress line
    }

    // Print report
    print_report(
        &collector,
        &game_data.card_db,
        &mode_str,
        &deck_info,
        args.min_plays,
        args.top_n,
    );

    // Export CSV if requested
    if let Some(ref csv_path) = args.csv {
        if let Err(e) = export_csv(&collector, &game_data.card_db, csv_path, args.min_plays) {
            eprintln!("Error exporting CSV: {}", e);
            process::exit(1);
        }
        println!("\nExported CSV to: {:?}", csv_path);
    }

    // Export JSON if requested
    if let Some(ref json_path) = args.json {
        let config = ReportConfig {
            mode: mode_str.clone(),
            deck: Some(args.deck.clone()),
            deck1: args.deck1.clone(),
            deck2: args.deck2.clone(),
            games: collector.total_games,
            bot: args.bot.clone(),
        };
        if let Err(e) = export_json(
            &collector,
            &game_data.card_db,
            json_path,
            config,
            args.min_plays,
        ) {
            eprintln!("Error exporting JSON: {}", e);
            process::exit(1);
        }
        println!("Exported JSON to: {:?}", json_path);
    }

    // Print synergy report if enabled
    if let Some(ref synergy) = collector.synergy {
        print_synergy_report(
            synergy,
            &collector,
            &game_data.card_db,
            &mode_str,
            &deck_info,
            args.min_cooccur,
            args.synergy_top_n,
        );

        // Export synergy CSV if requested
        if let Some(ref csv_path) = args.synergy_csv {
            if let Err(e) = export_synergy_csv(
                synergy,
                &collector,
                &game_data.card_db,
                csv_path,
                args.min_cooccur,
            ) {
                eprintln!("Error exporting synergy CSV: {}", e);
                process::exit(1);
            }
            println!("\nExported synergy CSV to: {:?}", csv_path);
        }

        // Export synergy JSON if requested
        if let Some(ref json_path) = args.synergy_json {
            let config = ReportConfig {
                mode: mode_str.clone(),
                deck: Some(args.deck.clone()),
                deck1: args.deck1.clone(),
                deck2: args.deck2.clone(),
                games: collector.total_games,
                bot: args.bot.clone(),
            };
            if let Err(e) = export_synergy_json(
                synergy,
                &collector,
                &game_data.card_db,
                json_path,
                config,
                args.min_cooccur,
                args.synergy_top_n,
            ) {
                eprintln!("Error exporting synergy JSON: {}", e);
                process::exit(1);
            }
            println!("Exported synergy JSON to: {:?}", json_path);
        }
    }
}

fn list_available_decks(game_data: &GameData) {
    eprintln!("Available decks:");
    for deck in game_data.deck_registry.decks() {
        eprintln!("  - {} ({})", deck.id, deck.name);
    }
}
