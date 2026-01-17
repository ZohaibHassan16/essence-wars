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

use clap::Parser;

use cardgame::arena::{
    load_deck, run_match_parallel, run_match_sequential, validate_faction_deck_binding,
    ActionLogger, MatchConfig, SequentialConfig,
};
use cardgame::bots::{BotType, MctsConfig};
use cardgame::cards::CardDatabase;
use cardgame::core::state::GameMode;
use cardgame::decks::DeckRegistry;
use cardgame::execution::configure_thread_pool;

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

    /// Game mode: attrition (default) or essence-duel
    #[arg(long, default_value = "attrition")]
    mode: String,
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

    // Parse bot types using factory
    let bot1_type: BotType = match args.bot1.parse() {
        Ok(t) => t,
        Err(_) => {
            eprintln!(
                "Unknown bot type: {}. Available: random, greedy, mcts, agent-argentum, agent-symbiote, agent-obsidion, agent-generalist",
                args.bot1
            );
            process::exit(1);
        }
    };

    let bot2_type: BotType = match args.bot2.parse() {
        Ok(t) => t,
        Err(_) => {
            eprintln!(
                "Unknown bot type: {}. Available: random, greedy, mcts, agent-argentum, agent-symbiote, agent-obsidion, agent-generalist",
                args.bot2
            );
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
    let loaded1 = match load_deck(args.deck1.as_deref(), &deck_registry, &card_db, "1") {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    let loaded2 = match load_deck(args.deck2.as_deref(), &deck_registry, &card_db, "2") {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    let (deck1, deck1_name) = (loaded1.cards, loaded1.name);
    let (deck2, deck2_name) = (loaded2.cards, loaded2.name);

    // Validate faction-deck binding for specialist agents
    if let Some(warning) = validate_faction_deck_binding(
        &bot1_type,
        args.deck1.as_deref(),
        &deck_registry,
        "Bot 1",
    ) {
        eprintln!("{}", warning);
    }
    if let Some(warning) = validate_faction_deck_binding(
        &bot2_type,
        args.deck2.as_deref(),
        &deck_registry,
        "Bot 2",
    ) {
        eprintln!("{}", warning);
    }

    // Get current working directory for weight resolution
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Resolve weights for both bots
    let weights1 = match cardgame::bots::resolve_weights_verbose(
        &bot1_type,
        args.weights1.as_deref(),
        &cwd,
        "Bot 1",
    ) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error loading weights for Bot 1: {}", e);
            process::exit(1);
        }
    };
    let weights2 = match cardgame::bots::resolve_weights_verbose(
        &bot2_type,
        args.weights2.as_deref(),
        &cwd,
        "Bot 2",
    ) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error loading weights for Bot 2: {}", e);
            process::exit(1);
        }
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
    let num_threads = configure_thread_pool(args.threads);

    // Determine tracing options
    let trace_combat = args.trace_combat || args.trace_all;
    let trace_effects = args.trace_effects || args.trace_all;
    let tracing_enabled = trace_combat || trace_effects;

    // Can't parallelize with logging, invariant checking, or tracing
    let parallel = !args.sequential && logger.is_none() && !args.invariants && !tracing_enabled;

    // Print match info
    println!("Arena Match");
    println!("===========");
    println!(
        "Bot 1: {} ({}){}",
        bot1_type.name(),
        deck1_name,
        weights1
            .as_ref()
            .map(|w| format!(" [weights: {}]", w.name))
            .unwrap_or_default()
    );
    println!(
        "Bot 2: {} ({}){}",
        bot2_type.name(),
        deck2_name,
        weights2
            .as_ref()
            .map(|w| format!(" [weights: {}]", w.name))
            .unwrap_or_default()
    );
    println!("Games: {}", args.games);
    println!("Base seed: {}", seed);
    if parallel {
        println!("Threads: {} (parallel)", num_threads);
    } else {
        let mut mode_notes = Vec::new();
        if logger.is_some() {
            mode_notes.push("logging");
        }
        if args.invariants {
            mode_notes.push("invariants");
        }
        if trace_combat {
            mode_notes.push("combat-trace");
        }
        if trace_effects {
            mode_notes.push("effect-trace");
        }
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
    if bot1_type.uses_mcts() || bot2_type.uses_mcts() {
        println!(
            "MCTS: {} sims x {} trees x {} rollouts/leaf",
            mcts_config.simulations, mcts_config.parallel_trees, mcts_config.leaf_rollouts
        );
    }

    // Parse game mode
    let game_mode = match args.mode.to_lowercase().as_str() {
        "attrition" => GameMode::Attrition,
        "essence-duel" | "essenceduel" | "duel" => GameMode::EssenceDuel,
        other => {
            eprintln!("Unknown game mode: '{}'. Use 'attrition' or 'essence-duel'.", other);
            process::exit(1);
        }
    };

    if game_mode == GameMode::EssenceDuel {
        println!("Mode: Essence Duel (first to 50 VP wins)");
    }

    // Build match configuration
    let config = MatchConfig::new(bot1_type, bot2_type, deck1, deck2, args.games, seed)
        .with_weights1(weights1)
        .with_weights2(weights2)
        .with_mcts_config(mcts_config)
        .with_progress(args.progress)
        .with_game_mode(game_mode);

    // Run the match
    let stats = if parallel {
        run_match_parallel(&card_db, &config)
    } else {
        let seq_config = SequentialConfig::new()
            .with_invariants(args.invariants)
            .with_combat_tracing(trace_combat)
            .with_effect_tracing(trace_effects);
        run_match_sequential(&card_db, &config, &seq_config, &mut logger)
    };

    // Print results
    println!("{}", stats.summary());
}
