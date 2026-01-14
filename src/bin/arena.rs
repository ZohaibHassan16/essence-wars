//! Arena CLI - Run matches between bots.
//!
//! Usage:
//!   cargo run --release --bin arena -- --bot1 random --bot2 random --games 100
//!   cargo run --release --bin arena -- --bot1 greedy --bot2 greedy --games 100
//!   cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --seed 12345 --debug
//!   cargo run --release --bin arena -- --deck1 symbiote_aggro --deck2 argentum_control
//!
//! Agent types (with auto-loaded specialist weights):
//!   cargo run --release --bin arena -- --bot1 agent-argentum --bot2 agent-symbiote
//!   cargo run --release --bin arena -- --bot1 agent-generalist --bot2 agent-obsidion

use std::path::PathBuf;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use clap::Parser;
use rayon::prelude::*;

use cardgame::arena::{
    ActionLogger, ActionRecord, CombatTracer, EffectTracer, MatchStats, StateSnapshot,
};
use cardgame::bots::{Bot, BotWeights, GreedyBot, MctsBot, MctsConfig, RandomBot};
use cardgame::cards::CardDatabase;
use cardgame::decks::{DeckRegistry, Faction};
use cardgame::engine::GameEngine;
use cardgame::types::{CardId, PlayerId};

/// Arena - Run matches between card game bots
#[derive(Parser, Debug)]
#[command(name = "arena")]
#[command(about = "Run matches between card game bots", long_about = None)]
struct Args {
    /// Bot 1 type (random, greedy, mcts)
    #[arg(long, default_value = "random")]
    bot1: String,

    /// Bot 2 type (random, greedy, mcts)
    #[arg(long, default_value = "random")]
    bot2: String,

    /// Deck 1 ID (use --list-decks to see available decks)
    #[arg(long)]
    deck1: Option<String>,

    /// Deck 2 ID (use --list-decks to see available decks)
    #[arg(long)]
    deck2: Option<String>,

    /// Number of games to play
    #[arg(long, short = 'n', default_value = "100")]
    games: usize,

    /// Random seed (uses system time if not specified)
    #[arg(long, short = 's')]
    seed: Option<u64>,

    /// Enable debug logging
    #[arg(long, short = 'd')]
    debug: bool,

    /// Verbose debug output (includes state snapshots)
    #[arg(long, short = 'v')]
    verbose: bool,

    /// Log file path (defaults to stdout if not specified)
    #[arg(long)]
    log_file: Option<PathBuf>,

    /// Path to card database
    #[arg(long, default_value = "data/cards/core_set")]
    cards: PathBuf,

    /// Path to deck definitions directory
    #[arg(long, default_value = "data/decks")]
    decks: PathBuf,

    /// List available decks and exit
    #[arg(long)]
    list_decks: bool,

    /// Show progress bar during match
    #[arg(long)]
    progress: bool,

    /// Custom weights file for bot 1 (TOML format, only for greedy/mcts)
    #[arg(long)]
    weights1: Option<PathBuf>,

    /// Custom weights file for bot 2 (TOML format, only for greedy/mcts)
    #[arg(long)]
    weights2: Option<PathBuf>,

    /// Number of threads for parallel execution (0 = use all cores)
    #[arg(long, short = 'j', default_value = "0")]
    threads: usize,

    /// Disable parallel execution (run sequentially)
    #[arg(long)]
    sequential: bool,

    /// Number of parallel trees for MCTS root parallelization (1 = sequential)
    #[arg(long, default_value = "1")]
    mcts_trees: u32,

    /// Number of simulations per MCTS tree
    #[arg(long, default_value = "500")]
    mcts_sims: u32,

    /// Number of parallel rollouts per MCTS leaf (1 = sequential)
    #[arg(long, default_value = "1")]
    mcts_rollouts: u32,

    /// Enable invariant checking after every action (forces sequential mode, slower)
    #[arg(long)]
    invariants: bool,

    /// Enable combat keyword resolution tracing (forces sequential mode)
    #[arg(long)]
    trace_combat: bool,

    /// Enable effect queue tracing (forces sequential mode)
    #[arg(long)]
    trace_effects: bool,

    /// Enable all tracing (combat + effects, forces sequential mode)
    #[arg(long)]
    trace_all: bool,
}

/// Bot types that can participate in arena matches.
enum BotType {
    Random,
    Greedy,
    Mcts,
    /// Agent specialist for a faction (uses MCTS with specialist weights)
    AgentSpecialist(Faction),
    /// Agent generalist (uses MCTS with generalist weights)
    AgentGeneralist,
}

impl BotType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "random" => Some(BotType::Random),
            "greedy" => Some(BotType::Greedy),
            "mcts" => Some(BotType::Mcts),
            "agent-argentum" => Some(BotType::AgentSpecialist(Faction::Argentum)),
            "agent-symbiote" => Some(BotType::AgentSpecialist(Faction::Symbiote)),
            "agent-obsidion" => Some(BotType::AgentSpecialist(Faction::Obsidion)),
            "agent-generalist" => Some(BotType::AgentGeneralist),
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            BotType::Random => "RandomBot",
            BotType::Greedy => "GreedyBot",
            BotType::Mcts => "MctsBot",
            BotType::AgentSpecialist(Faction::Argentum) => "Agent-Argentum",
            BotType::AgentSpecialist(Faction::Symbiote) => "Agent-Symbiote",
            BotType::AgentSpecialist(Faction::Obsidion) => "Agent-Obsidion",
            BotType::AgentSpecialist(Faction::Neutral) => "Agent-Neutral",
            BotType::AgentGeneralist => "Agent-Generalist",
        }
    }

    /// Returns the weights file path for Agent bot types, if applicable.
    fn agent_weights_path(&self) -> Option<PathBuf> {
        match self {
            BotType::AgentSpecialist(faction) => {
                Some(PathBuf::from(format!("data/weights/specialists/{}.toml", faction.as_tag())))
            }
            BotType::AgentGeneralist => {
                Some(PathBuf::from("data/weights/generalist.toml"))
            }
            _ => None,
        }
    }

}

fn main() {
    let args = Args::parse();

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
            // Only warn if decks were specifically requested
            if args.deck1.is_some() || args.deck2.is_some() || args.list_decks {
                eprintln!("Error loading decks from {:?}: {}", args.decks, e);
                process::exit(1);
            }
            DeckRegistry::new()
        }
    };

    // Handle --list-decks
    if args.list_decks {
        println!("Available decks:");
        if deck_registry.is_empty() {
            println!("  (no decks found in {:?})", args.decks);
        } else {
            for deck in deck_registry.decks() {
                println!("  {} - {} ({} cards)", deck.id, deck.name, deck.size());
                if !deck.tags.is_empty() {
                    println!("    Tags: {}", deck.tags.join(", "));
                }
            }
        }
        return;
    }

    // Parse bot types
    let bot1_type = match BotType::from_str(&args.bot1) {
        Some(t) => t,
        None => {
            eprintln!("Unknown bot type: {}. Available: random, greedy, mcts, agent-argentum, agent-symbiote, agent-obsidion, agent-generalist", args.bot1);
            process::exit(1);
        }
    };

    let bot2_type = match BotType::from_str(&args.bot2) {
        Some(t) => t,
        None => {
            eprintln!("Unknown bot type: {}. Available: random, greedy, mcts, agent-argentum, agent-symbiote, agent-obsidion, agent-generalist", args.bot2);
            process::exit(1);
        }
    };

    // Create seed
    let seed = args.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    // Load decks
    let (deck1, deck1_name) = load_deck(&args.deck1, &deck_registry, &card_db, "1");
    let (deck2, deck2_name) = load_deck(&args.deck2, &deck_registry, &card_db, "2");

    // Validate faction-deck binding for specialist agents
    validate_faction_deck_binding(&bot1_type, &args.deck1, &deck_registry, "Bot 1");
    validate_faction_deck_binding(&bot2_type, &args.deck2, &deck_registry, "Bot 2");

    // Load custom weights if specified, or auto-load for Agent types
    let weights1 = if args.weights1.is_some() {
        load_weights(&args.weights1, "bot1")
    } else if let Some(agent_path) = bot1_type.agent_weights_path() {
        load_agent_weights(&agent_path, bot1_type.name())
    } else {
        None
    };

    let weights2 = if args.weights2.is_some() {
        load_weights(&args.weights2, "bot2")
    } else if let Some(agent_path) = bot2_type.agent_weights_path() {
        load_agent_weights(&agent_path, bot2_type.name())
    } else {
        None
    };

    // Create logger if needed
    let mut logger = if args.debug || args.verbose {
        let l = if let Some(ref path) = args.log_file {
            match ActionLogger::to_file(path, args.verbose) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error creating log file {:?}: {}", path, e);
                    process::exit(1);
                }
            }
        } else {
            ActionLogger::stdout(args.verbose)
        };
        Some(l)
    } else {
        None
    };

    // Configure thread pool
    let num_threads = if args.threads == 0 {
        rayon::current_num_threads()
    } else {
        args.threads
    };

    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()
            .ok(); // Ignore if already initialized
    }

    // Determine tracing options
    let trace_combat = args.trace_combat || args.trace_all;
    let trace_effects = args.trace_effects || args.trace_all;
    let tracing_enabled = trace_combat || trace_effects;

    // Can't parallelize with logging, invariant checking, or tracing
    let parallel = !args.sequential && logger.is_none() && !args.invariants && !tracing_enabled;

    // Print match info
    println!("Arena Match");
    println!("===========");
    println!("Bot 1: {} ({}){}", bot1_type.name(), deck1_name,
        weights1.as_ref().map(|w| format!(" [weights: {}]", w.name)).unwrap_or_default());
    println!("Bot 2: {} ({}){}", bot2_type.name(), deck2_name,
        weights2.as_ref().map(|w| format!(" [weights: {}]", w.name)).unwrap_or_default());
    println!("Games: {}", args.games);
    println!("Base seed: {}", seed);
    if parallel {
        println!("Threads: {} (parallel)", num_threads);
    } else {
        let mut mode_notes = Vec::new();
        if logger.is_some() { mode_notes.push("logging"); }
        if args.invariants { mode_notes.push("invariants"); }
        if trace_combat { mode_notes.push("combat-trace"); }
        if trace_effects { mode_notes.push("effect-trace"); }
        let note = if mode_notes.is_empty() {
            String::new()
        } else {
            format!(" ({})", mode_notes.join(", "))
        };
        println!("Mode: sequential{}", note);
    }
    println!();

    // Create MCTS config
    let mcts_config = MctsConfig {
        simulations: args.mcts_sims,
        exploration: 1.414,
        max_rollout_depth: 100,
        parallel_trees: args.mcts_trees,
        leaf_rollouts: args.mcts_rollouts,
    };

    // Print MCTS config if using MCTS
    if matches!(bot1_type, BotType::Mcts) || matches!(bot2_type, BotType::Mcts) {
        println!("MCTS: {} sims x {} trees x {} rollouts/leaf",
            mcts_config.simulations, mcts_config.parallel_trees, mcts_config.leaf_rollouts);
    }

    // Run the match
    let stats = if parallel {
        run_match_parallel(
            &card_db,
            &bot1_type,
            &bot2_type,
            &deck1,
            &deck2,
            args.games,
            seed,
            args.progress,
            weights1.as_ref(),
            weights2.as_ref(),
            &mcts_config,
        )
    } else {
        run_match_sequential(
            &card_db,
            &bot1_type,
            &bot2_type,
            &deck1,
            &deck2,
            args.games,
            seed,
            &mut logger,
            args.progress,
            weights1.as_ref(),
            weights2.as_ref(),
            &mcts_config,
            args.invariants,
            trace_combat,
            trace_effects,
        )
    };

    // Print results
    println!("{}", stats.summary());
}

/// Load a deck by ID or use default.
fn load_deck(
    deck_id: &Option<String>,
    registry: &DeckRegistry,
    card_db: &CardDatabase,
    player: &str,
) -> (Vec<CardId>, String) {
    match deck_id {
        Some(id) => {
            match registry.get(id) {
                Some(deck) => {
                    if let Err(e) = deck.validate(card_db) {
                        eprintln!("Deck '{}' validation error: {}", id, e);
                        process::exit(1);
                    }
                    (deck.to_card_ids(), deck.name.clone())
                }
                None => {
                    eprintln!("Deck '{}' not found. Use --list-decks to see available decks.", id);
                    process::exit(1);
                }
            }
        }
        None => (create_default_deck(), format!("Default Deck {}", player)),
    }
}

/// Load custom weights from a TOML file.
fn load_weights(path: &Option<PathBuf>, bot_name: &str) -> Option<BotWeights> {
    match path {
        Some(p) => {
            match BotWeights::load(p) {
                Ok(w) => {
                    println!("Loaded weights for {}: {} ({} deck-specific)",
                        bot_name, w.name, w.deck_specific.len());
                    Some(w)
                }
                Err(e) => {
                    eprintln!("Error loading weights from {:?}: {}", p, e);
                    process::exit(1);
                }
            }
        }
        None => None,
    }
}

/// Load agent weights from a known path, silently returning None if not found.
/// Agent types use specialist weights when available, falling back to defaults.
fn load_agent_weights(path: &PathBuf, agent_name: &str) -> Option<BotWeights> {
    match BotWeights::load(path) {
        Ok(w) => {
            println!("Auto-loaded {} weights: {}", agent_name, w.name);
            Some(w)
        }
        Err(_) => {
            // Silently fall back to defaults if weights file doesn't exist
            println!("Note: {} using default weights (no specialist weights at {:?})", agent_name, path);
            None
        }
    }
}

/// Validate that specialist agents are paired with their faction's decks.
///
/// Specialist agents should only play decks of their faction.
/// Prints a warning if there's a mismatch but allows the game to continue.
fn validate_faction_deck_binding(
    bot_type: &BotType,
    deck_id: &Option<String>,
    deck_registry: &DeckRegistry,
    bot_label: &str,
) {
    // Only validate for specialist agents
    let specialist_faction = match bot_type {
        BotType::AgentSpecialist(faction) => faction,
        _ => return,
    };

    // Only validate if a specific deck was chosen
    let deck_id = match deck_id {
        Some(id) => id,
        None => return,
    };

    // Get the deck and check its faction
    if let Some(deck) = deck_registry.get(deck_id) {
        match deck.faction() {
            Some(deck_faction) => {
                if deck_faction != *specialist_faction {
                    eprintln!(
                        "Warning: {} ({}) is using a {} deck ('{}'), but specialists work best with their faction's decks.",
                        bot_label,
                        bot_type.name(),
                        deck_faction.display_name(),
                        deck_id
                    );
                    eprintln!(
                        "  Recommended: Use a {} deck for {} specialists.",
                        specialist_faction.display_name(),
                        specialist_faction.display_name()
                    );
                }
            }
            None => {
                eprintln!(
                    "Warning: {} ({}) is using a non-faction deck ('{}').",
                    bot_label,
                    bot_type.name(),
                    deck_id
                );
                eprintln!(
                    "  Recommended: Use a {} deck for {} specialists.",
                    specialist_faction.display_name(),
                    specialist_faction.display_name()
                );
            }
        }
    }
}

/// Run a match between two bots (parallel version).
#[allow(clippy::too_many_arguments)]
fn run_match_parallel(
    card_db: &CardDatabase,
    bot1_type: &BotType,
    bot2_type: &BotType,
    deck1: &[CardId],
    deck2: &[CardId],
    games: usize,
    base_seed: u64,
    show_progress: bool,
    weights1: Option<&BotWeights>,
    weights2: Option<&BotWeights>,
    mcts_config: &MctsConfig,
) -> MatchStats {
    let start_time = Instant::now();
    let completed = Arc::new(AtomicUsize::new(0));
    let completed_clone = completed.clone();

    // Progress reporting thread
    let progress_handle = if show_progress {
        let total = games;
        Some(std::thread::spawn(move || {
            let mut last_progress = 0;
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let done = completed_clone.load(Ordering::Relaxed);
                if done >= total {
                    break;
                }
                let progress = (done * 100) / total;
                if progress > last_progress {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let games_per_sec = done as f64 / elapsed.max(0.001);
                    let eta = if games_per_sec > 0.0 {
                        (total - done) as f64 / games_per_sec
                    } else {
                        0.0
                    };
                    eprint!("\rProgress: {:3}% ({}/{}) | {:.0} games/sec | ETA: {:.1}s    ",
                        progress, done, total, games_per_sec, eta);
                    last_progress = progress;
                }
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
            let result = run_single_game_no_log(
                card_db,
                bot1_type,
                bot2_type,
                deck1,
                deck2,
                game_seed,
                weights1,
                weights2,
                mcts_config,
            );
            completed.fetch_add(1, Ordering::Relaxed);
            result
        })
        .collect();

    // Wait for progress thread
    if let Some(handle) = progress_handle {
        let _ = handle.join();
        eprintln!("\rProgress: 100% ({}/{}) | Done!                              ", games, games);
    }

    let wall_clock_time = start_time.elapsed();

    // Aggregate results
    let mut stats = MatchStats::new(
        bot1_type.name().to_string(),
        bot2_type.name().to_string(),
    );
    for (winner, turns, duration) in results {
        stats.record_game(winner, turns, duration);
    }
    stats.set_wall_clock_time(wall_clock_time);

    stats
}

/// Run a match between two bots (sequential version with logging support).
#[allow(clippy::too_many_arguments)]
fn run_match_sequential(
    card_db: &CardDatabase,
    bot1_type: &BotType,
    bot2_type: &BotType,
    deck1: &[CardId],
    deck2: &[CardId],
    games: usize,
    base_seed: u64,
    logger: &mut Option<ActionLogger>,
    show_progress: bool,
    weights1: Option<&BotWeights>,
    weights2: Option<&BotWeights>,
    mcts_config: &MctsConfig,
    check_invariants: bool,
    trace_combat: bool,
    trace_effects: bool,
) -> MatchStats {
    let mut stats = MatchStats::new(
        bot1_type.name().to_string(),
        bot2_type.name().to_string(),
    );

    let start_time = Instant::now();
    let mut last_progress = 0;

    for i in 0..games {
        let game_seed = base_seed.wrapping_add(i as u64);
        let result = run_single_game(
            card_db,
            bot1_type,
            bot2_type,
            deck1,
            deck2,
            game_seed,
            logger,
            weights1,
            weights2,
            mcts_config,
            check_invariants,
            trace_combat,
            trace_effects,
        );
        stats.record_game(result.0, result.1, result.2);

        // Show progress every 10%
        if show_progress {
            let progress = ((i + 1) * 100) / games;
            if progress >= last_progress + 10 || i + 1 == games {
                let elapsed = start_time.elapsed().as_secs_f64();
                let games_per_sec = (i + 1) as f64 / elapsed.max(0.001);
                let eta = if games_per_sec > 0.0 {
                    (games - i - 1) as f64 / games_per_sec
                } else {
                    0.0
                };

                eprint!("\rProgress: {:3}% ({}/{}) | {:.0} games/sec | ETA: {:.1}s    ",
                    progress, i + 1, games, games_per_sec, eta);
                last_progress = progress;
            }
        }
    }

    if show_progress {
        eprintln!(); // New line after progress
    }

    stats
}

/// Run a single game between two bots.
/// Returns (winner, turns, duration).
#[allow(clippy::too_many_arguments)]
fn run_single_game(
    card_db: &CardDatabase,
    bot1_type: &BotType,
    bot2_type: &BotType,
    deck1: &[CardId],
    deck2: &[CardId],
    seed: u64,
    logger: &mut Option<ActionLogger>,
    weights1: Option<&BotWeights>,
    weights2: Option<&BotWeights>,
    mcts_config: &MctsConfig,
    check_invariants: bool,
    trace_combat: bool,
    trace_effects: bool,
) -> (Option<PlayerId>, u32, Duration) {
    let start = Instant::now();

    // Create tracers if enabled
    let mut combat_tracer = CombatTracer::new(trace_combat);
    let mut effect_tracer = EffectTracer::new(trace_effects);
    let tracing_enabled = trace_combat || trace_effects;

    // Create bots with appropriate seeds and weights
    let bot1_seed = seed;
    let bot2_seed = seed.wrapping_add(1000000);

    let mut random_bot1 = RandomBot::new(bot1_seed);
    let mut random_bot2 = RandomBot::new(bot2_seed);

    // Create greedy bots with custom weights if provided
    let mut greedy_bot1 = match weights1 {
        Some(w) => GreedyBot::from_bot_weights(card_db, w, None, bot1_seed),
        None => GreedyBot::new(card_db, bot1_seed),
    };
    let mut greedy_bot2 = match weights2 {
        Some(w) => GreedyBot::from_bot_weights(card_db, w, None, bot2_seed),
        None => GreedyBot::new(card_db, bot2_seed),
    };

    // Create MCTS bots with custom rollout weights if provided
    let mut mcts_bot1 = match weights1 {
        Some(w) => MctsBot::with_config_and_weights(card_db, mcts_config.clone(), w, bot1_seed),
        None => MctsBot::with_config(card_db, mcts_config.clone(), bot1_seed),
    };
    let mut mcts_bot2 = match weights2 {
        Some(w) => MctsBot::with_config_and_weights(card_db, mcts_config.clone(), w, bot2_seed),
        None => MctsBot::with_config(card_db, mcts_config.clone(), bot2_seed),
    };

    // Reset bots
    random_bot1.reset();
    random_bot2.reset();
    greedy_bot1.reset();
    greedy_bot2.reset();
    mcts_bot1.reset();
    mcts_bot2.reset();

    // Create and start game engine
    let mut engine = GameEngine::new(card_db);
    engine.start_game(deck1.to_vec(), deck2.to_vec(), seed);

    // Log game start
    if let Some(ref mut l) = logger {
        let _ = l.log_game_start(seed, bot1_type.name(), bot2_type.name());
    }

    // Main game loop
    let max_actions = 1000;
    let mut action_count = 0;

    while !engine.is_game_over() && action_count < max_actions {
        let action_start = Instant::now();

        let current_player = engine.current_player();
        let turn = engine.turn_number() as u32;

        // Get state info for logging
        let state_tensor = engine.get_state_tensor();
        let legal_mask = engine.get_legal_action_mask();
        let legal_actions = engine.get_legal_actions();

        // Select action based on current player and bot type
        let action = if current_player == PlayerId::PLAYER_ONE {
            match bot1_type {
                BotType::Random => random_bot1.select_action(&state_tensor, &legal_mask, &legal_actions),
                BotType::Greedy => greedy_bot1.select_action_with_engine(&engine),
                BotType::Mcts | BotType::AgentSpecialist(_) | BotType::AgentGeneralist => {
                    mcts_bot1.select_action_with_engine(&engine)
                }
            }
        } else {
            match bot2_type {
                BotType::Random => random_bot2.select_action(&state_tensor, &legal_mask, &legal_actions),
                BotType::Greedy => greedy_bot2.select_action_with_engine(&engine),
                BotType::Mcts | BotType::AgentSpecialist(_) | BotType::AgentGeneralist => {
                    mcts_bot2.select_action_with_engine(&engine)
                }
            }
        };

        let thinking_time = action_start.elapsed();

        // Log action
        if let Some(ref mut l) = logger {
            let record = ActionRecord {
                turn,
                player: current_player,
                action,
                thinking_time_us: thinking_time.as_micros() as u64,
                state_snapshot: if l.is_verbose() {
                    Some(StateSnapshot::from_state(&engine.state))
                } else {
                    None
                },
            };
            let _ = l.log_action(&record);
        }

        // Apply action - use traced version when tracing is enabled
        let result = if tracing_enabled {
            engine.apply_action_with_tracers(
                action,
                if trace_combat { Some(&mut combat_tracer) } else { None },
                if trace_effects { Some(&mut effect_tracer) } else { None },
            )
        } else {
            engine.apply_action(action)
        };

        if let Err(e) = result {
            eprintln!("Error applying action {:?}: {:?}", action, e);
            break;
        }

        // Check invariants if enabled
        if check_invariants {
            verify_invariants(&engine, seed, action_count, action);
        }

        action_count += 1;
    }

    // Get final result
    let winner = engine.winner();
    let turns = engine.turn_number() as u32;
    let duration = start.elapsed();

    // Log game end
    if let Some(ref mut l) = logger {
        let _ = l.log_game_end(
            winner,
            turns,
            engine.state.players[0].life,
            engine.state.players[1].life,
        );
    }

    // Print trace output if tracing is enabled
    if trace_combat && !combat_tracer.traces.is_empty() {
        println!("{}", combat_tracer.format_all());
    }
    if trace_effects && !effect_tracer.events.is_empty() {
        println!("{}", effect_tracer.format());
    }

    (winner, turns, duration)
}

/// Verify game state invariants. Panics on violation.
fn verify_invariants(engine: &GameEngine, seed: u64, action_num: usize, last_action: cardgame::actions::Action) {
    let state = &engine.state;
    let context = format!("seed={}, action #{}, last={:?}", seed, action_num, last_action);

    for (player_idx, player) in state.players.iter().enumerate() {
        let player_name = if player_idx == 0 { "P1" } else { "P2" };

        // Life should be in valid range (can go negative but not excessively)
        assert!(
            player.life <= 30,
            "[{}] {}: Life {} exceeds 30",
            context, player_name, player.life
        );

        // AP should be in valid range
        assert!(
            player.action_points <= 10,
            "[{}] {}: AP {} exceeds 10",
            context, player_name, player.action_points
        );

        // Creature count should not exceed slots
        assert!(
            player.creatures.len() <= 5,
            "[{}] {}: {} creatures exceeds 5 slots",
            context, player_name, player.creatures.len()
        );

        // Support count should not exceed slots
        assert!(
            player.supports.len() <= 2,
            "[{}] {}: {} supports exceeds 2 slots",
            context, player_name, player.supports.len()
        );

        // No duplicate creature slots
        let mut seen_slots = [false; 5];
        for creature in &player.creatures {
            let slot = creature.slot.0 as usize;
            assert!(
                slot < 5,
                "[{}] {}: Creature in invalid slot {}",
                context, player_name, slot
            );
            assert!(
                !seen_slots[slot],
                "[{}] {}: Duplicate creature in slot {}",
                context, player_name, slot
            );
            seen_slots[slot] = true;

            // Creatures should have positive health (dead ones removed)
            assert!(
                creature.current_health > 0,
                "[{}] {}: Creature in slot {} has {} health (should be dead)",
                context, player_name, slot, creature.current_health
            );
        }

        // No duplicate support slots
        let mut seen_support_slots = [false; 2];
        for support in &player.supports {
            let slot = support.slot.0 as usize;
            assert!(
                slot < 2,
                "[{}] {}: Support in invalid slot {}",
                context, player_name, slot
            );
            assert!(
                !seen_support_slots[slot],
                "[{}] {}: Duplicate support in slot {}",
                context, player_name, slot
            );
            seen_support_slots[slot] = true;

            // Supports should have positive durability
            assert!(
                support.current_durability > 0,
                "[{}] {}: Support in slot {} has 0 durability (should be removed)",
                context, player_name, slot
            );
        }

        // Hand size should not exceed maximum
        assert!(
            player.hand.len() <= 20,
            "[{}] {}: Hand size {} exceeds 20",
            context, player_name, player.hand.len()
        );
    }

    // If game is over, result should be set
    if state.players[0].life <= 0 || state.players[1].life <= 0 {
        assert!(
            state.result.is_some(),
            "[{}] Player dead but game result not set (P1: {}, P2: {})",
            context, state.players[0].life, state.players[1].life
        );
    }
}

/// Run a single game without logging (for parallel execution).
/// Returns (winner, turns, duration).
#[allow(clippy::too_many_arguments)]
fn run_single_game_no_log(
    card_db: &CardDatabase,
    bot1_type: &BotType,
    bot2_type: &BotType,
    deck1: &[CardId],
    deck2: &[CardId],
    seed: u64,
    weights1: Option<&BotWeights>,
    weights2: Option<&BotWeights>,
    mcts_config: &MctsConfig,
) -> (Option<PlayerId>, u32, Duration) {
    let start = Instant::now();

    // Create bots with appropriate seeds and weights
    let bot1_seed = seed;
    let bot2_seed = seed.wrapping_add(1000000);

    let mut random_bot1 = RandomBot::new(bot1_seed);
    let mut random_bot2 = RandomBot::new(bot2_seed);

    // Create greedy bots with custom weights if provided
    let mut greedy_bot1 = match weights1 {
        Some(w) => GreedyBot::from_bot_weights(card_db, w, None, bot1_seed),
        None => GreedyBot::new(card_db, bot1_seed),
    };
    let mut greedy_bot2 = match weights2 {
        Some(w) => GreedyBot::from_bot_weights(card_db, w, None, bot2_seed),
        None => GreedyBot::new(card_db, bot2_seed),
    };

    // Create MCTS bots with custom rollout weights if provided
    let mut mcts_bot1 = match weights1 {
        Some(w) => MctsBot::with_config_and_weights(card_db, mcts_config.clone(), w, bot1_seed),
        None => MctsBot::with_config(card_db, mcts_config.clone(), bot1_seed),
    };
    let mut mcts_bot2 = match weights2 {
        Some(w) => MctsBot::with_config_and_weights(card_db, mcts_config.clone(), w, bot2_seed),
        None => MctsBot::with_config(card_db, mcts_config.clone(), bot2_seed),
    };

    // Create and start game engine
    let mut engine = GameEngine::new(card_db);
    engine.start_game(deck1.to_vec(), deck2.to_vec(), seed);

    // Main game loop
    let max_actions = 1000;
    let mut action_count = 0;

    while !engine.is_game_over() && action_count < max_actions {
        let current_player = engine.current_player();

        // Select action based on current player and bot type
        let action = if current_player == PlayerId::PLAYER_ONE {
            match bot1_type {
                BotType::Random => {
                    let state_tensor = engine.get_state_tensor();
                    let legal_mask = engine.get_legal_action_mask();
                    let legal_actions = engine.get_legal_actions();
                    random_bot1.select_action(&state_tensor, &legal_mask, &legal_actions)
                }
                BotType::Greedy => greedy_bot1.select_action_with_engine(&engine),
                BotType::Mcts | BotType::AgentSpecialist(_) | BotType::AgentGeneralist => {
                    mcts_bot1.select_action_with_engine(&engine)
                }
            }
        } else {
            match bot2_type {
                BotType::Random => {
                    let state_tensor = engine.get_state_tensor();
                    let legal_mask = engine.get_legal_action_mask();
                    let legal_actions = engine.get_legal_actions();
                    random_bot2.select_action(&state_tensor, &legal_mask, &legal_actions)
                }
                BotType::Greedy => greedy_bot2.select_action_with_engine(&engine),
                BotType::Mcts | BotType::AgentSpecialist(_) | BotType::AgentGeneralist => {
                    mcts_bot2.select_action_with_engine(&engine)
                }
            }
        };

        // Apply action
        if engine.apply_action(action).is_err() {
            break;
        }

        action_count += 1;
    }

    // Get final result
    let winner = engine.winner();
    let turns = engine.turn_number() as u32;
    let duration = start.elapsed();

    (winner, turns, duration)
}

/// Create a default deck for testing.
fn create_default_deck() -> Vec<CardId> {
    // Aggressive Assault deck from design doc (simplified)
    let card_ids = [
        1, 1,   // Eager Recruit x2
        3, 3,   // Nimble Scout x2
        6, 6,   // Frontier Ranger x2
        8, 8,   // Shielded Squire x2
        11, 11, // Centaur Charger x2
        12, 12, // Blade Dancer x2
        16, 16, // Piercing Striker x2
        20, 20, // Siege Breaker x2
        34, 34, // Lightning Bolt x2
    ];
    card_ids.iter().map(|&id| CardId(id)).collect()
}
