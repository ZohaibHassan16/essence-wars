//! Balance Validation CLI - Run comprehensive faction matchup testing.
//!
//! Tests all faction pairs (Argentum, Symbiote, Obsidion) in both player orders
//! using MCTS agents with faction-specific weights.
//!
//! Usage:
//!   cargo run --release --bin validate -- --games 100
//!   cargo run --release --bin validate -- --games 500 --output results.json
//!   cargo run --release --bin validate -- --games 50 --quiet

use std::collections::HashMap;
use std::path::PathBuf;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use clap::Parser;
use rayon::prelude::*;
use serde::Serialize;

use cardgame::bots::{Bot, BotWeights, MctsBot, MctsConfig};
use cardgame::cards::CardDatabase;
use cardgame::decks::{DeckRegistry, Faction};
use cardgame::engine::GameEngine;
use cardgame::types::{CardId, PlayerId};
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

    /// Quiet mode for scripted usage (minimal output)
    #[arg(long, short = 'q')]
    quiet: bool,

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
}

/// Configuration for validation run (for JSON output)
#[derive(Debug, Clone, Serialize)]
struct ValidationConfig {
    games_per_matchup: usize,
    mcts_simulations: u32,
    seed: u64,
    threads: usize,
}

/// Results for a single faction pair (both player orders)
#[derive(Debug, Clone, Serialize)]
struct MatchupResult {
    faction1: String,
    faction2: String,
    deck1_id: String,
    deck2_id: String,
    // Faction 1 as Player 1
    f1_as_p1_wins: u32,
    f1_as_p1_games: u32,
    // Faction 1 as Player 2
    f1_as_p2_wins: u32,
    f1_as_p2_games: u32,
    // Combined totals
    faction1_total_wins: u32,
    faction2_total_wins: u32,
    draws: u32,
    total_games: u32,
    // Derived rates
    faction1_win_rate: f64,
    faction2_win_rate: f64,
    // Timing
    avg_turns: f64,
    total_time_secs: f64,
}

/// Balance status classification
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum BalanceStatus {
    Balanced,
    Warning,
    Imbalanced,
}

impl std::fmt::Display for BalanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalanceStatus::Balanced => write!(f, "BALANCED"),
            BalanceStatus::Warning => write!(f, "WARNING"),
            BalanceStatus::Imbalanced => write!(f, "IMBALANCED"),
        }
    }
}

/// Balance analysis summary
#[derive(Debug, Clone, Serialize)]
struct BalanceSummary {
    p1_win_rate: f64,
    p1_status: BalanceStatus,
    faction_win_rates: HashMap<String, f64>,
    max_faction_delta: f64,
    faction_status: BalanceStatus,
    overall_status: BalanceStatus,
    warnings: Vec<String>,
}

/// Complete validation results (for JSON output)
#[derive(Debug, Clone, Serialize)]
struct ValidationResults {
    timestamp: String,
    version: VersionInfo,
    config: ValidationConfig,
    matchups: Vec<MatchupResult>,
    summary: BalanceSummary,
}

/// A matchup to test
struct Matchup {
    faction1: Faction,
    faction2: Faction,
    deck1_id: String,
    deck1_cards: Vec<CardId>,
    deck2_id: String,
    deck2_cards: Vec<CardId>,
}

/// Faction weights holder
struct FactionWeights {
    argentum: Option<BotWeights>,
    symbiote: Option<BotWeights>,
    obsidion: Option<BotWeights>,
}

impl FactionWeights {
    fn get(&self, faction: Faction) -> Option<&BotWeights> {
        match faction {
            Faction::Argentum => self.argentum.as_ref(),
            Faction::Symbiote => self.symbiote.as_ref(),
            Faction::Obsidion => self.obsidion.as_ref(),
            Faction::Neutral => None,
        }
    }
}

fn main() {
    let args = Args::parse();

    // Configure thread pool
    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()
            .ok();
    }
    let num_threads = if args.threads == 0 {
        rayon::current_num_threads()
    } else {
        args.threads
    };

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
    let faction_weights = load_faction_weights(&args.weights, args.quiet);

    // Build matchups (3 faction pairs, no mirrors)
    let matchups = build_matchups(&deck_registry, &card_db);
    if matchups.is_empty() {
        eprintln!("Error: No valid faction matchups found. Need decks for at least 2 factions.");
        process::exit(1);
    }

    // Print header
    if !args.quiet {
        println!("=== Balance Validation ===");
        println!("Version: {}", version::version_string());
        println!(
            "Config: {} games/matchup, {} MCTS sims, {} threads",
            args.games, args.mcts_sims, num_threads
        );
        println!("Matchups: {} pairs", matchups.len());
        println!();
    }

    // Run validation
    let start_time = Instant::now();
    let matchup_results = run_validation(
        &card_db,
        &matchups,
        &faction_weights,
        args.games,
        args.seed,
        args.mcts_sims,
        !args.quiet,
    );
    let total_time = start_time.elapsed();

    // Analyze balance
    let summary = analyze_balance(&matchup_results);

    // Create full results
    let results = ValidationResults {
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: VersionInfo::current(),
        config: ValidationConfig {
            games_per_matchup: args.games,
            mcts_simulations: args.mcts_sims,
            seed: args.seed,
            threads: num_threads,
        },
        matchups: matchup_results,
        summary,
    };

    // Output results
    if !args.quiet {
        print_results(&results, total_time);
    }

    // Save JSON if requested
    if let Some(ref output_path) = args.output {
        match serde_json::to_string_pretty(&results) {
            Ok(json) => {
                if let Err(e) = std::fs::write(output_path, json) {
                    eprintln!("Error writing JSON to {:?}: {}", output_path, e);
                    process::exit(1);
                }
                if !args.quiet {
                    println!("\nResults saved to: {:?}", output_path);
                }
            }
            Err(e) => {
                eprintln!("Error serializing results: {}", e);
                process::exit(1);
            }
        }
    }

    // Exit with appropriate code
    if results.summary.overall_status == BalanceStatus::Imbalanced {
        process::exit(1);
    }
}

/// Load faction-specific weights
fn load_faction_weights(weights_dir: &std::path::Path, quiet: bool) -> FactionWeights {
    let specialists_dir = weights_dir.join("specialists");

    let load_one = |faction: &str| -> Option<BotWeights> {
        let path = specialists_dir.join(format!("{}.toml", faction));
        match BotWeights::load(&path) {
            Ok(w) => {
                if !quiet {
                    println!("Loaded {} weights: {}", faction, w.name);
                }
                Some(w)
            }
            Err(_) => {
                if !quiet {
                    println!("Note: {} using default weights", faction);
                }
                None
            }
        }
    };

    FactionWeights {
        argentum: load_one("argentum"),
        symbiote: load_one("symbiote"),
        obsidion: load_one("obsidion"),
    }
}

/// Build all faction matchups (no mirrors)
fn build_matchups(registry: &DeckRegistry, card_db: &CardDatabase) -> Vec<Matchup> {
    let factions = [Faction::Argentum, Faction::Symbiote, Faction::Obsidion];
    let mut matchups = Vec::new();

    // Get first valid deck for each faction
    let mut faction_decks: HashMap<Faction, (String, Vec<CardId>)> = HashMap::new();
    for faction in &factions {
        let decks = registry.decks_for_faction(*faction);
        for deck in decks {
            if deck.validate(card_db).is_ok() {
                faction_decks.insert(*faction, (deck.id.clone(), deck.to_card_ids()));
                break;
            }
        }
    }

    // Create all pairs (no mirrors)
    for i in 0..factions.len() {
        for j in (i + 1)..factions.len() {
            let f1 = factions[i];
            let f2 = factions[j];

            if let (Some((id1, cards1)), Some((id2, cards2))) =
                (faction_decks.get(&f1), faction_decks.get(&f2))
            {
                matchups.push(Matchup {
                    faction1: f1,
                    faction2: f2,
                    deck1_id: id1.clone(),
                    deck1_cards: cards1.clone(),
                    deck2_id: id2.clone(),
                    deck2_cards: cards2.clone(),
                });
            }
        }
    }

    matchups
}

/// Run all validation matchups
fn run_validation(
    card_db: &CardDatabase,
    matchups: &[Matchup],
    weights: &FactionWeights,
    games_per_matchup: usize,
    base_seed: u64,
    mcts_sims: u32,
    show_progress: bool,
) -> Vec<MatchupResult> {
    let mcts_config = MctsConfig {
        simulations: mcts_sims,
        exploration: 1.414,
        max_rollout_depth: 100,
        parallel_trees: 1,
        leaf_rollouts: 1,
    };

    let mut results = Vec::new();

    for (matchup_idx, matchup) in matchups.iter().enumerate() {
        let matchup_seed = base_seed.wrapping_add((matchup_idx * 1_000_000) as u64);

        if show_progress {
            println!(
                "{} vs {} ({} games each direction)...",
                matchup.faction1.display_name(),
                matchup.faction2.display_name(),
                games_per_matchup
            );
        }

        // Run F1 as P1 vs F2 as P2
        let (f1_p1_wins, f1_p1_turns, f1_p1_time, f1_p1_draws) = run_matchup_games(
            card_db,
            &matchup.deck1_cards,
            &matchup.deck2_cards,
            weights.get(matchup.faction1),
            weights.get(matchup.faction2),
            games_per_matchup,
            matchup_seed,
            &mcts_config,
            show_progress,
        );

        // Run F2 as P1 vs F1 as P2
        let (f2_p1_wins, f2_p1_turns, f2_p1_time, f2_p1_draws) = run_matchup_games(
            card_db,
            &matchup.deck2_cards,
            &matchup.deck1_cards,
            weights.get(matchup.faction2),
            weights.get(matchup.faction1),
            games_per_matchup,
            matchup_seed.wrapping_add(500_000),
            &mcts_config,
            show_progress,
        );

        // Calculate results
        let f1_as_p1_wins = f1_p1_wins;
        let f1_as_p1_games = games_per_matchup as u32;
        let f1_as_p2_wins = games_per_matchup as u32 - f2_p1_wins - f2_p1_draws;
        let f1_as_p2_games = games_per_matchup as u32;

        let total_games = (games_per_matchup * 2) as u32;
        let faction1_total_wins = f1_as_p1_wins + f1_as_p2_wins;
        let total_draws = f1_p1_draws + f2_p1_draws;
        let faction2_total_wins = total_games - faction1_total_wins - total_draws;

        let total_turns = f1_p1_turns + f2_p1_turns;
        let total_time = f1_p1_time + f2_p1_time;

        results.push(MatchupResult {
            faction1: matchup.faction1.as_tag().to_string(),
            faction2: matchup.faction2.as_tag().to_string(),
            deck1_id: matchup.deck1_id.clone(),
            deck2_id: matchup.deck2_id.clone(),
            f1_as_p1_wins,
            f1_as_p1_games,
            f1_as_p2_wins,
            f1_as_p2_games,
            faction1_total_wins,
            faction2_total_wins,
            draws: total_draws,
            total_games,
            faction1_win_rate: faction1_total_wins as f64 / (total_games - total_draws) as f64,
            faction2_win_rate: faction2_total_wins as f64 / (total_games - total_draws) as f64,
            avg_turns: total_turns as f64 / total_games as f64,
            total_time_secs: total_time.as_secs_f64(),
        });

        if show_progress {
            println!();
        }
    }

    results
}

/// Run games for a single matchup direction
#[allow(clippy::too_many_arguments)]
fn run_matchup_games(
    card_db: &CardDatabase,
    deck1: &[CardId],
    deck2: &[CardId],
    weights1: Option<&BotWeights>,
    weights2: Option<&BotWeights>,
    games: usize,
    base_seed: u64,
    mcts_config: &MctsConfig,
    show_progress: bool,
) -> (u32, u32, Duration, u32) {
    let start_time = Instant::now();
    let completed = Arc::new(AtomicUsize::new(0));
    let completed_clone = completed.clone();

    // Progress reporting thread
    let progress_handle = if show_progress {
        let total = games;
        Some(std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let done = completed_clone.load(Ordering::Relaxed);
                if done >= total {
                    break;
                }
                let progress = (done * 100) / total;
                eprint!("\r  Progress: {:3}% ({}/{})    ", progress, done, total);
            }
        }))
    } else {
        None
    };

    // Run games in parallel
    let results: Vec<_> = (0..games)
        .into_par_iter()
        .map(|i| {
            let game_seed = base_seed.wrapping_add(i as u64);
            let result =
                run_single_game(card_db, deck1, deck2, game_seed, weights1, weights2, mcts_config);
            completed.fetch_add(1, Ordering::Relaxed);
            result
        })
        .collect();

    // Wait for progress thread
    if let Some(handle) = progress_handle {
        let _ = handle.join();
        eprint!("\r  Progress: 100% ({}/{})    \n", games, games);
    }

    // Aggregate results
    let mut p1_wins = 0u32;
    let mut total_turns = 0u32;
    let mut draws = 0u32;
    let mut total_time = Duration::ZERO;

    for (winner, turns, duration) in results {
        match winner {
            Some(PlayerId::PLAYER_ONE) => p1_wins += 1,
            None => draws += 1,
            _ => {}
        }
        total_turns += turns;
        total_time += duration;
    }

    (p1_wins, total_turns, start_time.elapsed(), draws)
}

/// Run a single game between two MCTS bots
fn run_single_game(
    card_db: &CardDatabase,
    deck1: &[CardId],
    deck2: &[CardId],
    seed: u64,
    weights1: Option<&BotWeights>,
    weights2: Option<&BotWeights>,
    mcts_config: &MctsConfig,
) -> (Option<PlayerId>, u32, Duration) {
    let start = Instant::now();

    let bot1_seed = seed;
    let bot2_seed = seed.wrapping_add(1_000_000);

    // Create MCTS bots
    let mut mcts_bot1 = match weights1 {
        Some(w) => MctsBot::with_config_and_weights(card_db, mcts_config.clone(), w, bot1_seed),
        None => MctsBot::with_config(card_db, mcts_config.clone(), bot1_seed),
    };
    let mut mcts_bot2 = match weights2 {
        Some(w) => MctsBot::with_config_and_weights(card_db, mcts_config.clone(), w, bot2_seed),
        None => MctsBot::with_config(card_db, mcts_config.clone(), bot2_seed),
    };

    // Create and start game
    let mut engine = GameEngine::new(card_db);
    engine.start_game(deck1.to_vec(), deck2.to_vec(), seed);

    // Main game loop
    let max_actions = 1000;
    let mut action_count = 0;

    while !engine.is_game_over() && action_count < max_actions {
        let current_player = engine.current_player();
        let action = if current_player == PlayerId::PLAYER_ONE {
            mcts_bot1.select_action_with_engine(&engine)
        } else {
            mcts_bot2.select_action_with_engine(&engine)
        };

        if engine.apply_action(action).is_err() {
            break;
        }
        action_count += 1;
    }

    (engine.winner(), engine.turn_number() as u32, start.elapsed())
}

/// Analyze balance from matchup results
fn analyze_balance(matchups: &[MatchupResult]) -> BalanceSummary {
    let mut warnings = Vec::new();

    // Calculate P1 win rate across all games
    let mut total_p1_wins = 0u32;
    let mut total_games = 0u32;
    let mut total_draws = 0u32;

    for m in matchups {
        total_p1_wins += m.f1_as_p1_wins;
        total_p1_wins += m.total_games / 2 - m.f1_as_p2_wins - m.draws / 2; // F2 as P1 wins
        total_games += m.total_games;
        total_draws += m.draws;
    }

    let decisive_games = total_games - total_draws;
    let p1_win_rate = if decisive_games > 0 {
        total_p1_wins as f64 / decisive_games as f64
    } else {
        0.5
    };

    // Determine P1 balance status
    let p1_status = if (0.50..=0.55).contains(&p1_win_rate) {
        BalanceStatus::Balanced
    } else if (0.45..=0.60).contains(&p1_win_rate) {
        warnings.push(format!(
            "P1 win rate {:.1}% is outside ideal range (50-55%)",
            p1_win_rate * 100.0
        ));
        BalanceStatus::Warning
    } else {
        warnings.push(format!(
            "P1 win rate {:.1}% is significantly imbalanced",
            p1_win_rate * 100.0
        ));
        BalanceStatus::Imbalanced
    };

    // Calculate faction win rates
    let mut faction_wins: HashMap<String, u32> = HashMap::new();
    let mut faction_games: HashMap<String, u32> = HashMap::new();

    for m in matchups {
        *faction_wins.entry(m.faction1.clone()).or_insert(0) += m.faction1_total_wins;
        *faction_wins.entry(m.faction2.clone()).or_insert(0) += m.faction2_total_wins;
        *faction_games.entry(m.faction1.clone()).or_insert(0) += m.total_games - m.draws;
        *faction_games.entry(m.faction2.clone()).or_insert(0) += m.total_games - m.draws;
    }

    let faction_win_rates: HashMap<String, f64> = faction_wins
        .iter()
        .map(|(faction, wins)| {
            let games = faction_games.get(faction).copied().unwrap_or(1);
            (faction.clone(), *wins as f64 / games as f64)
        })
        .collect();

    // Calculate max faction delta
    let rates: Vec<f64> = faction_win_rates.values().copied().collect();
    let max_faction_delta = if rates.len() >= 2 {
        let max = rates.iter().cloned().fold(f64::MIN, f64::max);
        let min = rates.iter().cloned().fold(f64::MAX, f64::min);
        max - min
    } else {
        0.0
    };

    // Determine faction balance status
    let faction_status = if max_faction_delta < 0.10 {
        BalanceStatus::Balanced
    } else if max_faction_delta < 0.15 {
        warnings.push(format!(
            "Faction delta {:.1}% is above target (<10%)",
            max_faction_delta * 100.0
        ));
        BalanceStatus::Warning
    } else {
        warnings.push(format!(
            "Faction delta {:.1}% indicates significant imbalance",
            max_faction_delta * 100.0
        ));
        BalanceStatus::Imbalanced
    };

    // Overall status is the worse of P1 and faction
    let overall_status = match (p1_status, faction_status) {
        (BalanceStatus::Imbalanced, _) | (_, BalanceStatus::Imbalanced) => BalanceStatus::Imbalanced,
        (BalanceStatus::Warning, _) | (_, BalanceStatus::Warning) => BalanceStatus::Warning,
        _ => BalanceStatus::Balanced,
    };

    BalanceSummary {
        p1_win_rate,
        p1_status,
        faction_win_rates,
        max_faction_delta,
        faction_status,
        overall_status,
        warnings,
    }
}

/// Print formatted results to stdout
fn print_results(results: &ValidationResults, total_time: Duration) {
    println!("=== Matchup Results ===");

    for m in &results.matchups {
        let f1_name = capitalize(&m.faction1);
        let f2_name = capitalize(&m.faction2);

        println!("\n{} vs {}:", f1_name, f2_name);
        println!(
            "  {} P1: {}/{} ({:.1}%)  |  {} P2: {}/{} ({:.1}%)",
            f1_name,
            m.f1_as_p1_wins,
            m.f1_as_p1_games,
            m.f1_as_p1_wins as f64 / m.f1_as_p1_games as f64 * 100.0,
            f1_name,
            m.f1_as_p2_wins,
            m.f1_as_p2_games,
            m.f1_as_p2_wins as f64 / m.f1_as_p2_games as f64 * 100.0,
        );
        println!(
            "  Total: {} {:.1}% / {} {:.1}%  (draws: {})",
            f1_name,
            m.faction1_win_rate * 100.0,
            f2_name,
            m.faction2_win_rate * 100.0,
            m.draws,
        );
        println!("  Avg turns: {:.1}, Time: {:.1}s", m.avg_turns, m.total_time_secs);
    }

    println!("\n=== Summary ===");
    println!(
        "P1 Win Rate: {:.1}% [{}]",
        results.summary.p1_win_rate * 100.0,
        results.summary.p1_status
    );

    println!("\nFaction Win Rates:");
    for (faction, rate) in &results.summary.faction_win_rates {
        println!("  {}: {:.1}%", capitalize(faction), rate * 100.0);
    }
    println!(
        "Max Delta: {:.1}% [{}]",
        results.summary.max_faction_delta * 100.0,
        results.summary.faction_status
    );

    if !results.summary.warnings.is_empty() {
        println!("\nWarnings:");
        for w in &results.summary.warnings {
            println!("  - {}", w);
        }
    }

    println!("\nOverall Status: {}", results.summary.overall_status);
    println!("Total Time: {:.1}s", total_time.as_secs_f64());
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}
