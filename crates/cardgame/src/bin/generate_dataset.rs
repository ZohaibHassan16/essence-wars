//! Dataset Generator - Generate MCTS self-play data for behavioral cloning.
//!
//! This binary generates high-quality training data from MCTS vs MCTS games.
//! Each game position includes:
//! - State tensor (326 floats)
//! - Action mask (256 bools)
//! - MCTS policy (visit count distribution over 256 actions)
//! - Game outcome (+1/-1 from the position's player perspective)
//!
//! Usage:
//!   cargo run --release --bin generate-dataset -- --games 100000 --sims 100 --output data.jsonl
//!   cargo run --release --bin generate-dataset -- --games 10000 --mode round-robin --output balanced.jsonl

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use clap::Parser;
use flate2::write::GzEncoder;
use flate2::Compression;
use indicatif::{ProgressBar, ProgressStyle};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use serde::Serialize;

use cardgame::actions::Action;
use cardgame::bots::{Bot, BotWeights, GreedyBot, GreedyWeights, MctsConfig};
use cardgame::cards::CardDatabase;
use cardgame::engine::GameEnvironment;
use cardgame::execution::configure_thread_pool;
use cardgame::legal::legal_action_mask;
use cardgame::state::GameMode;
use cardgame::tensor::state_to_tensor;
use cardgame::types::{CardId, PlayerId};
use cardgame::{DeckRegistry, GameEngine};

/// Dataset Generator - Generate MCTS self-play data for ML training
#[derive(Parser, Debug)]
#[command(name = "generate-dataset")]
#[command(about = "Generate MCTS self-play datasets for behavioral cloning")]
struct Args {
    /// Number of games to generate
    #[arg(long, short = 'n', default_value = "1000")]
    games: usize,

    /// MCTS simulations per move
    #[arg(long, default_value = "100")]
    sims: u32,

    /// Output file path (.jsonl or .jsonl.gz)
    #[arg(long, short = 'o', default_value = "dataset.jsonl")]
    output: PathBuf,

    /// Generation mode: "self-play" or "round-robin" (all deck matchups)
    #[arg(long, default_value = "self-play")]
    mode: String,

    /// Specific deck for self-play mode (random if not specified)
    #[arg(long)]
    deck: Option<String>,

    /// Random seed
    #[arg(long, short = 's')]
    seed: Option<u64>,

    /// Path to card database
    #[arg(long, default_value = "data/cards/core_set")]
    cards: PathBuf,

    /// Path to deck definitions directory
    #[arg(long, default_value = "data/decks")]
    decks: PathBuf,

    /// Path to MCTS weights file (optional)
    #[arg(long)]
    weights: Option<PathBuf>,

    /// Number of threads (0 = use all cores)
    #[arg(long, short = 'j', default_value = "0")]
    threads: usize,

    /// Game mode: "attrition" or "essence-duel"
    #[arg(long, default_value = "attrition")]
    game_mode: String,

    /// Disable compression (output .jsonl instead of .jsonl.gz)
    #[arg(long)]
    no_compress: bool,
}

/// A single move record in the dataset.
#[derive(Serialize)]
struct MoveRecord {
    /// Turn number (1-indexed)
    turn: u8,
    /// Player who made this move (0 or 1)
    player: u8,
    /// State tensor (326 floats)
    state_tensor: Vec<f32>,
    /// Action mask (256 bools as 0/1)
    action_mask: Vec<u8>,
    /// Action taken (0-255)
    action: u8,
    /// MCTS policy (visit count distribution, 256 floats)
    mcts_policy: Vec<f32>,
    /// MCTS value estimate (win rate from current player's perspective)
    mcts_value: f32,
}

/// A complete game record.
#[derive(Serialize)]
struct GameRecord {
    /// Unique game ID
    game_id: String,
    /// Deck 1 ID
    deck1: String,
    /// Deck 2 ID
    deck2: String,
    /// Winner (0 or 1, or -1 for draw)
    winner: i8,
    /// All moves in the game
    moves: Vec<MoveRecord>,
    /// Game metadata
    metadata: GameMetadata,
}

#[derive(Serialize)]
struct GameMetadata {
    total_turns: u8,
    mcts_sims: u32,
    seed: u64,
    game_mode: String,
}

/// MCTS search that returns visit count policy.
struct MctsSearcher<'a> {
    card_db: &'a CardDatabase,
    config: MctsConfig,
    rollout_weights: Option<GreedyWeights>,
}

impl<'a> MctsSearcher<'a> {
    fn new(card_db: &'a CardDatabase, sims: u32) -> Self {
        Self {
            card_db,
            config: MctsConfig {
                simulations: sims,
                exploration: 1.414,
                max_rollout_depth: 100,
                parallel_trees: 1,
                leaf_rollouts: 1,
            },
            rollout_weights: None,
        }
    }

    fn with_weights(mut self, weights: GreedyWeights) -> Self {
        self.rollout_weights = Some(weights);
        self
    }

    /// Run MCTS and return (action, policy, value).
    /// Policy is a 256-float vector of visit count proportions.
    fn search(&self, engine: &GameEngine, rng: &mut SmallRng) -> (Action, Vec<f32>, f32) {
        use std::cell::RefCell;
        use std::rc::Rc;

        const ACTION_SPACE_SIZE: usize = 256;
        let legal_actions = engine.get_legal_actions();

        // Handle trivial cases
        if legal_actions.is_empty() {
            let mut policy = vec![0.0f32; ACTION_SPACE_SIZE];
            policy[255] = 1.0; // EndTurn
            return (Action::EndTurn, policy, 0.0);
        }

        if legal_actions.len() == 1 {
            let action = legal_actions[0];
            let idx = action.to_index() as usize;
            let mut policy = vec![0.0f32; ACTION_SPACE_SIZE];
            policy[idx] = 1.0;
            return (action, policy, 0.0);
        }

        // Create MCTS tree
        let root = Rc::new(RefCell::new(MctsNode::root()));
        root.borrow_mut().expand(&legal_actions);

        let player = engine.current_player();
        let mut total_wins = 0i32;

        // Run simulations
        for _ in 0..self.config.simulations {
            let mut sim_engine = engine.fork();
            let mut path: Vec<Rc<RefCell<MctsNode>>> = vec![root.clone()];

            // Selection
            let mut current = root.clone();
            while !sim_engine.is_game_over() {
                let child = {
                    let node = current.borrow();
                    node.select_child(self.config.exploration)
                };

                if let Some(child) = child {
                    if let Some(action) = child.borrow().action {
                        if sim_engine.apply_action(action).is_err() {
                            break;
                        }
                    }
                    path.push(child.clone());
                    current = child;
                } else {
                    // Leaf node - expand
                    if !sim_engine.is_game_over() {
                        let actions = sim_engine.get_legal_actions();
                        if !actions.is_empty() {
                            current.borrow_mut().expand(&actions);
                            // Select first child
                            if let Some(child) = current.borrow().children.first().cloned() {
                                if let Some(action) = child.borrow().action {
                                    let _ = sim_engine.apply_action(action);
                                }
                                path.push(child);
                            }
                        }
                    }
                    break;
                }
            }

            // Rollout
            let winner = self.rollout(&mut sim_engine, rng);
            let win = winner == Some(player);
            if win {
                total_wins += 1;
            }

            // Backprop
            for node in path.iter().rev() {
                node.borrow_mut().update(win);
            }
        }

        // Extract policy from visit counts
        let mut policy = vec![0.0f32; ACTION_SPACE_SIZE];
        let total_visits: u32 = root.borrow().children.iter().map(|c| c.borrow().visits).sum();

        if total_visits > 0 {
            for child in root.borrow().children.iter() {
                let child_ref = child.borrow();
                if let Some(action) = child_ref.action {
                    let idx = action.to_index() as usize;
                    policy[idx] = child_ref.visits as f32 / total_visits as f32;
                }
            }
        }

        // Select action by most visits
        let best_child = root.borrow().most_visited_child();
        let action = best_child
            .and_then(|c| c.borrow().action)
            .unwrap_or(Action::EndTurn);

        // Value estimate: win rate from simulations
        let value = total_wins as f32 / self.config.simulations as f32;

        (action, policy, value)
    }

    fn rollout(&self, engine: &mut GameEngine, rng: &mut SmallRng) -> Option<PlayerId> {
        let mut greedy = if let Some(ref weights) = self.rollout_weights {
            GreedyBot::with_weights(self.card_db, weights.clone(), rng.gen())
        } else {
            GreedyBot::new(self.card_db, rng.gen())
        };

        for _ in 0..self.config.max_rollout_depth {
            if engine.is_game_over() {
                break;
            }

            let tensor = state_to_tensor(engine.get_state());
            let bool_mask = legal_action_mask(engine.get_state(), self.card_db);
            let mask: [f32; 256] = std::array::from_fn(|i| if bool_mask[i] { 1.0 } else { 0.0 });
            let legal = engine.get_legal_actions();

            let action = greedy.select_action(&tensor, &mask, &legal);
            if engine.apply_action(action).is_err() {
                break;
            }
        }

        engine.winner()
    }
}

/// Simple MCTS node for search.
struct MctsNode {
    action: Option<Action>,
    visits: u32,
    wins: i32,
    children: Vec<Rc<RefCell<MctsNode>>>,
    expanded: bool,
}

use std::cell::RefCell;
use std::rc::Rc;

impl MctsNode {
    fn root() -> Self {
        Self {
            action: None,
            visits: 0,
            wins: 0,
            children: Vec::new(),
            expanded: false,
        }
    }

    fn new(action: Action) -> Self {
        Self {
            action: Some(action),
            visits: 0,
            wins: 0,
            children: Vec::new(),
            expanded: false,
        }
    }

    fn expand(&mut self, actions: &[Action]) {
        if self.expanded {
            return;
        }
        for &action in actions {
            self.children.push(Rc::new(RefCell::new(MctsNode::new(action))));
        }
        self.expanded = true;
    }

    fn ucb1(&self, parent_visits: u32, exploration: f32) -> f32 {
        if self.visits == 0 {
            return f32::MAX;
        }
        let exploitation = self.wins as f32 / self.visits as f32;
        let exploration_term = exploration * ((parent_visits as f32).ln() / self.visits as f32).sqrt();
        exploitation + exploration_term
    }

    fn select_child(&self, exploration: f32) -> Option<Rc<RefCell<MctsNode>>> {
        self.children
            .iter()
            .max_by(|a, b| {
                let a_ucb = a.borrow().ucb1(self.visits, exploration);
                let b_ucb = b.borrow().ucb1(self.visits, exploration);
                a_ucb.partial_cmp(&b_ucb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    fn most_visited_child(&self) -> Option<Rc<RefCell<MctsNode>>> {
        self.children.iter().max_by_key(|c| c.borrow().visits).cloned()
    }

    fn update(&mut self, win: bool) {
        self.visits += 1;
        if win {
            self.wins += 1;
        }
    }
}

/// Configuration for game generation.
struct GameConfig<'a> {
    deck1_id: &'a str,
    deck2_id: &'a str,
    sims: u32,
    seed: u64,
    game_mode: GameMode,
    weights: Option<&'a BotWeights>,
}

/// Generate a single game and return the record.
fn generate_game(
    card_db: &CardDatabase,
    deck_registry: &DeckRegistry,
    config: GameConfig,
) -> GameRecord {
    let mut rng = SmallRng::seed_from_u64(config.seed);

    // Load decks
    let deck1 = deck_registry.get(config.deck1_id).expect("Deck 1 not found");
    let deck2 = deck_registry.get(config.deck2_id).expect("Deck 2 not found");

    // Create engine
    let mut engine = GameEngine::new(card_db);
    let deck1_cards: Vec<CardId> = deck1.cards.iter().map(|&id| CardId(id)).collect();
    let deck2_cards: Vec<CardId> = deck2.cards.iter().map(|&id| CardId(id)).collect();
    engine.start_game_with_mode(deck1_cards, deck2_cards, config.seed, config.game_mode);

    // Create MCTS searcher
    let searcher = if let Some(w) = config.weights {
        MctsSearcher::new(card_db, config.sims).with_weights(w.default.greedy.clone())
    } else {
        MctsSearcher::new(card_db, config.sims)
    };

    let mut moves = Vec::new();

    // Play game
    while !engine.is_game_over() {
        let state = engine.get_state();
        let turn = state.current_turn as u8;
        let player = engine.current_player();

        // Get state tensor and mask
        let state_tensor = state_to_tensor(state);
        let bool_mask = legal_action_mask(state, card_db);

        // Run MCTS search
        let (action, mcts_policy, mcts_value) = searcher.search(&engine, &mut rng);

        // Record move
        moves.push(MoveRecord {
            turn,
            player: player.0,
            state_tensor: state_tensor.to_vec(),
            action_mask: bool_mask.iter().map(|&x| if x { 1 } else { 0 }).collect(),
            action: action.to_index(),
            mcts_policy,
            mcts_value,
        });

        // Apply action
        if engine.apply_action(action).is_err() {
            break;
        }
    }

    // Determine winner
    let winner = match engine.winner() {
        Some(PlayerId(0)) => 0,
        Some(PlayerId(1)) => 1,
        Some(_) => -1, // Invalid player ID, treat as draw
        None => -1,
    };

    GameRecord {
        game_id: format!("g_{:08x}", config.seed),
        deck1: config.deck1_id.to_string(),
        deck2: config.deck2_id.to_string(),
        winner,
        moves,
        metadata: GameMetadata {
            total_turns: engine.get_state().current_turn as u8,
            mcts_sims: config.sims,
            seed: config.seed,
            game_mode: format!("{:?}", config.game_mode),
        },
    }
}

fn main() {
    let args = Args::parse();

    // Configure thread pool
    configure_thread_pool(args.threads);

    // Load card database
    println!("Loading card database from {:?}...", args.cards);
    let card_db = CardDatabase::load_from_directory(&args.cards).expect("Failed to load card database");

    // Load deck registry
    println!("Loading decks from {:?}...", args.decks);
    let deck_registry = DeckRegistry::load_from_directory(&args.decks).expect("Failed to load decks");

    // Load weights if specified
    let weights = args.weights.as_ref().map(|path| {
        BotWeights::load(path).expect("Failed to load weights")
    });

    // Parse game mode
    let game_mode = match args.game_mode.to_lowercase().as_str() {
        "attrition" => GameMode::Attrition,
        "essence-duel" | "essenceduel" => GameMode::EssenceDuel,
        _ => {
            eprintln!("Unknown game mode: {}. Using Attrition.", args.game_mode);
            GameMode::Attrition
        }
    };

    // Get base seed
    let base_seed = args.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    // Get deck list
    let deck_ids: Vec<String> = deck_registry.deck_ids().iter().map(|s| s.to_string()).collect();

    println!("\n=== Dataset Generation ===");
    println!("  Games:     {}", args.games);
    println!("  MCTS sims: {}", args.sims);
    println!("  Mode:      {}", args.mode);
    println!("  Game mode: {:?}", game_mode);
    println!("  Output:    {:?}", args.output);
    println!("  Seed:      {}", base_seed);
    println!("  Decks:     {} available", deck_ids.len());
    println!();

    // Generate matchup list
    let matchups: Vec<(String, String, u64)> = match args.mode.as_str() {
        "self-play" => {
            // Random deck selection or specific deck
            let mut rng = SmallRng::seed_from_u64(base_seed);
            (0..args.games)
                .map(|i| {
                    let (d1, d2) = if let Some(ref deck) = args.deck {
                        (deck.clone(), deck.clone())
                    } else {
                        let d1 = deck_ids[rng.gen_range(0..deck_ids.len())].clone();
                        let d2 = deck_ids[rng.gen_range(0..deck_ids.len())].clone();
                        (d1, d2)
                    };
                    (d1, d2, base_seed + i as u64)
                })
                .collect()
        }
        "round-robin" => {
            // All deck combinations
            let games_per_matchup = args.games / (deck_ids.len() * deck_ids.len());
            let games_per_matchup = games_per_matchup.max(1);
            let mut matchups = Vec::new();
            let mut seed_counter = base_seed;

            for d1 in &deck_ids {
                for d2 in &deck_ids {
                    for _ in 0..games_per_matchup {
                        matchups.push((d1.clone(), d2.clone(), seed_counter));
                        seed_counter += 1;
                    }
                }
            }
            matchups
        }
        _ => {
            eprintln!("Unknown mode: {}. Using self-play.", args.mode);
            let mut rng = SmallRng::seed_from_u64(base_seed);
            (0..args.games)
                .map(|i| {
                    let d1 = deck_ids[rng.gen_range(0..deck_ids.len())].clone();
                    let d2 = deck_ids[rng.gen_range(0..deck_ids.len())].clone();
                    (d1, d2, base_seed + i as u64)
                })
                .collect()
        }
    };

    let total_games = matchups.len();
    println!("Generating {} games...", total_games);

    // Setup progress bar
    let progress = ProgressBar::new(total_games as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let start = Instant::now();
    let games_completed = AtomicU64::new(0);
    let total_moves = AtomicU64::new(0);

    // Prepare output file path
    let use_compression = !args.no_compress && args.output.extension().is_some_and(|e| e == "gz");
    let output_path = if use_compression {
        args.output.clone()
    } else if args.output.extension().is_none_or(|e| e != "jsonl") {
        args.output.with_extension("jsonl")
    } else {
        args.output.clone()
    };

    println!("Output file: {:?}", output_path);

    // Create thread-safe channel for streaming records to disk
    use std::sync::mpsc;
    let (tx, rx) = mpsc::sync_channel(100); // Buffer up to 100 records

    // Track stats for final report
    let wins_p1 = AtomicU64::new(0);
    let wins_p2 = AtomicU64::new(0);
    let draws = AtomicU64::new(0);

    // Spawn writer thread (clone path for closure)
    let output_path_clone = output_path.clone();
    let writer_handle = std::thread::spawn(move || {
        let file = File::create(&output_path_clone).expect("Failed to create output file");
        
        if use_compression || output_path_clone.extension().is_some_and(|e| e == "gz") {
            let encoder = GzEncoder::new(file, Compression::default());
            let mut writer = BufWriter::new(encoder);
            for record in rx {
                serde_json::to_writer(&mut writer, &record).expect("Failed to write record");
                writeln!(writer).expect("Failed to write newline");
            }
            // CRITICAL: Flush buffer and finalize gzip stream
            writer.flush().expect("Failed to flush buffer");
            let encoder = writer.into_inner().expect("Failed to unwrap encoder");
            encoder.finish().expect("Failed to finalize gzip stream");
        } else {
            let mut writer = BufWriter::new(file);
            for record in rx {
                serde_json::to_writer(&mut writer, &record).expect("Failed to write record");
                writeln!(writer).expect("Failed to write newline");
            }
            writer.flush().expect("Failed to flush buffer");
        }
    });

    println!("Generating {} games (streaming to disk)...", total_games);

    // Generate games in parallel, sending to writer thread
    matchups
        .into_par_iter()
        .for_each(|(d1, d2, seed)| {
            let record = generate_game(
                &card_db,
                &deck_registry,
                GameConfig {
                    deck1_id: &d1,
                    deck2_id: &d2,
                    sims: args.sims,
                    seed,
                    game_mode,
                    weights: weights.as_ref(),
                },
            );

            let moves = record.moves.len() as u64;
            total_moves.fetch_add(moves, Ordering::Relaxed);
            
            // Track winner stats
            match record.winner {
                0 => wins_p1.fetch_add(1, Ordering::Relaxed),
                1 => wins_p2.fetch_add(1, Ordering::Relaxed),
                _ => draws.fetch_add(1, Ordering::Relaxed),
            };

            // Send record to writer thread (blocks if buffer full)
            tx.send(record).expect("Failed to send record to writer");

            let completed = games_completed.fetch_add(1, Ordering::Relaxed) + 1;
            progress.set_position(completed);
            progress.set_message(format!("{} moves", total_moves.load(Ordering::Relaxed)));
        });

    // Close channel and wait for writer to finish
    drop(tx);
    writer_handle.join().expect("Writer thread panicked");

    progress.finish_with_message("done");

    let duration = start.elapsed();
    let total_move_count = total_moves.load(Ordering::Relaxed);
    let total_game_count = games_completed.load(Ordering::Relaxed);

    println!("\nGeneration complete!");
    println!("  Games:     {}", total_game_count);
    println!("  Moves:     {}", total_move_count);
    println!("  Duration:  {:.1}s", duration.as_secs_f64());
    println!(
        "  Speed:     {:.1} games/s, {:.1} moves/s",
        total_game_count as f64 / duration.as_secs_f64(),
        total_move_count as f64 / duration.as_secs_f64()
    );
    println!("  Output:    {:?}", output_path);

    // Print stats
    let p1_wins = wins_p1.load(Ordering::Relaxed);
    let p2_wins = wins_p2.load(Ordering::Relaxed);
    let draw_count = draws.load(Ordering::Relaxed);

    println!("\nGame Statistics:");
    println!(
        "  P1 wins: {} ({:.1}%)",
        p1_wins,
        100.0 * p1_wins as f64 / total_game_count as f64
    );
    println!(
        "  P2 wins: {} ({:.1}%)",
        p2_wins,
        100.0 * p2_wins as f64 / total_game_count as f64
    );
    println!(
        "  Draws:   {} ({:.1}%)",
        draw_count,
        100.0 * draw_count as f64 / total_game_count as f64
    );
    println!(
        "  Avg moves/game: {:.1}",
        total_move_count as f64 / total_game_count as f64
    );
}
