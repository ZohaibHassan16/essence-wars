//! Arena CLI - Run matches between bots.
//!
//! Usage:
//!   cargo run --release --bin arena -- --bot1 random --bot2 random --games 100
//!   cargo run --release --bin arena -- --bot1 greedy --bot2 greedy --games 100
//!   cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --seed 12345 --debug
//!   cargo run --release --bin arena -- --deck1 aggressive_assault --deck2 defensive_control

use std::path::PathBuf;
use std::process;
use std::time::{Duration, Instant};

use clap::Parser;

use cardgame::arena::{ActionLogger, ActionRecord, MatchStats, StateSnapshot};
use cardgame::bots::{Bot, BotWeights, GreedyBot, MctsBot, MctsConfig, RandomBot};
use cardgame::cards::CardDatabase;
use cardgame::decks::DeckRegistry;
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
    #[arg(long, default_value = "data/cards")]
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
}

/// Bot types that can participate in arena matches.
enum BotType {
    Random,
    Greedy,
    Mcts,
}

impl BotType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "random" => Some(BotType::Random),
            "greedy" => Some(BotType::Greedy),
            "mcts" => Some(BotType::Mcts),
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            BotType::Random => "RandomBot",
            BotType::Greedy => "GreedyBot",
            BotType::Mcts => "MctsBot",
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
            eprintln!("Unknown bot type: {}. Available: random, greedy, mcts", args.bot1);
            process::exit(1);
        }
    };

    let bot2_type = match BotType::from_str(&args.bot2) {
        Some(t) => t,
        None => {
            eprintln!("Unknown bot type: {}. Available: random, greedy, mcts", args.bot2);
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

    // Load custom weights if specified
    let weights1 = load_weights(&args.weights1, "bot1");
    let weights2 = load_weights(&args.weights2, "bot2");

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

    // Print match info
    println!("Arena Match");
    println!("===========");
    println!("Bot 1: {} ({}){}", bot1_type.name(), deck1_name,
        weights1.as_ref().map(|w| format!(" [weights: {}]", w.name)).unwrap_or_default());
    println!("Bot 2: {} ({}){}", bot2_type.name(), deck2_name,
        weights2.as_ref().map(|w| format!(" [weights: {}]", w.name)).unwrap_or_default());
    println!("Games: {}", args.games);
    println!("Base seed: {}", seed);
    println!();

    // Run the match
    let stats = run_match(
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
    );

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

/// Run a match between two bots.
fn run_match(
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
    let mcts_config = MctsConfig { simulations: 500, exploration: 1.414, max_rollout_depth: 100 };
    let mut mcts_bot1 = match weights1 {
        Some(w) => MctsBot::with_config_and_weights(card_db, mcts_config.clone(), w, bot1_seed),
        None => MctsBot::with_config(card_db, mcts_config.clone(), bot1_seed),
    };
    let mut mcts_bot2 = match weights2 {
        Some(w) => MctsBot::with_config_and_weights(card_db, mcts_config, w, bot2_seed),
        None => MctsBot::with_config(card_db, mcts_config, bot2_seed),
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
                BotType::Mcts => mcts_bot1.select_action_with_engine(&engine),
            }
        } else {
            match bot2_type {
                BotType::Random => random_bot2.select_action(&state_tensor, &legal_mask, &legal_actions),
                BotType::Greedy => greedy_bot2.select_action_with_engine(&engine),
                BotType::Mcts => mcts_bot2.select_action_with_engine(&engine),
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

        // Apply action
        if let Err(e) = engine.apply_action(action) {
            eprintln!("Error applying action {:?}: {:?}", action, e);
            break;
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
