//! Weight tuning CLI - Optimize bot weights using CMA-ES.
//!
//! Usage:
//!   cargo run --release --bin tune -- --generations 50 --population 20
//!   cargo run --release --bin tune -- --mode vs-greedy --games 100
//!   cargo run --release --bin tune -- --mode specialist --deck aggressive_assault --opponent defensive_control
//!   cargo run --release --bin tune -- --tag baseline

use std::path::PathBuf;
use std::process;
use std::time::Instant;
use std::fs;
use std::io::Write;

use clap::Parser;

use cardgame::bots::{BotWeights, GreedyWeights, WeightSet};
use cardgame::cards::CardDatabase;
use cardgame::decks::DeckRegistry;
use cardgame::tuning::{CmaEs, CmaEsConfig, Evaluator, EvaluatorConfig, TuningMode};
use cardgame::types::CardId;
use cardgame::version::{self, VersionInfo};

/// Weight tuning CLI using CMA-ES optimization
#[derive(Parser, Debug)]
#[command(name = "tune")]
#[command(about = "Optimize bot weights using CMA-ES evolution strategy", long_about = None)]
struct Args {
    /// Tuning mode: vs-random, vs-greedy, multi-opponent, generalist, specialist
    #[arg(long, default_value = "vs-random")]
    mode: String,

    /// Enable parallel game evaluation (uses all CPU cores)
    #[arg(long, default_value = "true")]
    parallel: bool,

    /// MCTS simulations for multi-opponent mode
    #[arg(long, default_value = "200")]
    mcts_sims: u32,

    /// Number of CMA-ES generations
    #[arg(long, short = 'g', default_value = "50")]
    generations: u32,

    /// Population size (candidates per generation)
    #[arg(long, short = 'p')]
    population: Option<usize>,

    /// Games per evaluation
    #[arg(long, default_value = "50")]
    games: usize,

    /// Initial sigma (step size) for CMA-ES
    #[arg(long, default_value = "0.3")]
    sigma: f64,

    /// Minimum sigma to stop (convergence threshold)
    #[arg(long, default_value = "0.001")]
    min_sigma: f64,

    /// Target win rate to stop early (0.0 to 1.0)
    #[arg(long)]
    target_win_rate: Option<f64>,

    /// Random seed
    #[arg(long, short = 's', default_value = "42")]
    seed: u64,

    /// Deck ID for specialist mode (our deck)
    #[arg(long)]
    deck: Option<String>,

    /// Opponent deck ID for specialist mode
    #[arg(long)]
    opponent: Option<String>,

    /// Experiment tag (descriptive name for this run)
    #[arg(long, short = 't', default_value = "default")]
    tag: String,

    /// Base output directory for experiments
    #[arg(long, default_value = "experiments")]
    experiment_dir: PathBuf,

    /// Path to card database
    #[arg(long, default_value = "data/cards")]
    cards: PathBuf,

    /// Path to deck definitions directory
    #[arg(long, default_value = "data/decks")]
    decks: PathBuf,

    /// Verbose output (print each generation)
    #[arg(long, short = 'v')]
    verbose: bool,

    /// Start from existing weights file
    #[arg(long)]
    initial_weights: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    // Create experiment directory with timestamp
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H%M").to_string();
    let exp_id = format!("{}_{}", timestamp, args.tag);
    let exp_dir = args.experiment_dir.join("mcts").join(&exp_id);
    
    fs::create_dir_all(&exp_dir).unwrap_or_else(|e| {
        eprintln!("Error creating experiment directory {:?}: {}", exp_dir, e);
        process::exit(1);
    });

    let plots_dir = exp_dir.join("plots");
    fs::create_dir_all(&plots_dir).unwrap_or_else(|e| {
        eprintln!("Error creating plots directory: {}", e);
        process::exit(1);
    });

    println!("📁 Experiment directory: {:?}", exp_dir);
    println!("🔖 Engine version: {}", version::version_string());
    println!();

    // Save version info for reproducibility
    let version_info = VersionInfo::current();
    let version_path = exp_dir.join("version.toml");
    if let Ok(toml_str) = toml::to_string_pretty(&version_info) {
        let _ = fs::write(&version_path, toml_str);
    }

    // Load card database
    let card_db = match CardDatabase::load_from_directory(&args.cards) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error loading card database from {:?}: {}", args.cards, e);
            process::exit(1);
        }
    };

    // Load deck registry if needed
    let deck_registry = match DeckRegistry::load_from_directory(&args.decks) {
        Ok(r) => r,
        Err(e) => {
            if args.mode == "specialist" || args.mode == "generalist" {
                eprintln!("Error loading decks from {:?}: {}", args.decks, e);
                process::exit(1);
            }
            DeckRegistry::new()
        }
    };

    // Parse tuning mode
    let tuning_mode = match args.mode.as_str() {
        "vs-random" => TuningMode::VsRandom,
        "vs-greedy" => TuningMode::VsGreedy,
        "multi-opponent" => TuningMode::MultiOpponent,
        "generalist" => {
            // Use all deck combinations
            let matchups = create_generalist_matchups(&deck_registry, &card_db);
            if matchups.is_empty() {
                eprintln!("No valid matchups found for generalist mode");
                process::exit(1);
            }
            println!("Generalist mode with {} matchups", matchups.len());
            TuningMode::Generalist { matchups }
        }
        "specialist" => {
            let deck_id = args.deck.as_ref().expect("--deck required for specialist mode");
            let opponent_id = args.opponent.as_ref().expect("--opponent required for specialist mode");

            let deck = match deck_registry.get(deck_id) {
                Some(d) => d.to_card_ids(),
                None => {
                    eprintln!("Deck '{}' not found", deck_id);
                    process::exit(1);
                }
            };

            let opponent_deck = match deck_registry.get(opponent_id) {
                Some(d) => d.to_card_ids(),
                None => {
                    eprintln!("Opponent deck '{}' not found", opponent_id);
                    process::exit(1);
                }
            };

            println!("Specialist mode: {} vs {}", deck_id, opponent_id);
            TuningMode::Specialist { deck, opponent_deck }
        }
        _ => {
            eprintln!("Unknown mode: {}. Available: vs-random, vs-greedy, multi-opponent, generalist, specialist", args.mode);
            process::exit(1);
        }
    };

    // Create evaluator config
    let eval_config = EvaluatorConfig {
        games_per_eval: args.games,
        mode: tuning_mode,
        seed: args.seed,
        max_actions: 500,
        parallel: args.parallel,
        mcts_sims: args.mcts_sims,
    };

    // Create CMA-ES config
    let target_fitness = args.target_win_rate.map(|wr| wr * 100.0);
    let cmaes_config = CmaEsConfig {
        population_size: args.population,
        initial_sigma: args.sigma,
        max_generations: args.generations,
        target_fitness,
        min_sigma: args.min_sigma,
        seed: args.seed,
    };

    // Initial weights
    let initial_weights: Vec<f64> = if let Some(ref path) = args.initial_weights {
        match BotWeights::load(path) {
            Ok(w) => w.default.greedy.to_vec().iter().map(|&x| x as f64).collect(),
            Err(e) => {
                eprintln!("Error loading initial weights from {:?}: {}", path, e);
                process::exit(1);
            }
        }
    } else {
        GreedyWeights::default().to_vec().iter().map(|&x| x as f64).collect()
    };

    // Parameter bounds
    let bounds: Vec<(f64, f64)> = GreedyWeights::bounds()
        .iter()
        .map(|&(min, max)| (min as f64, max as f64))
        .collect();

    // Print configuration
    println!("Weight Tuning");
    println!("=============");
    println!("Mode: {}", args.mode);
    println!("Parallel: {}", args.parallel);
    println!("Generations: {}", args.generations);
    println!("Population: {}", cmaes_config.population_size.unwrap_or(4 + (3.0 * (20.0_f64).ln()).floor() as usize));
    println!("Games/eval: {}", args.games);
    println!("Initial sigma: {:.3}", args.sigma);
    println!("Seed: {}", args.seed);
    if args.mode == "multi-opponent" {
        println!("MCTS sims: {}", args.mcts_sims);
    }
    if let Some(wr) = args.target_win_rate {
        println!("Target win rate: {:.1}%", wr * 100.0);
    }
    println!();

    // Create log file
    let log_path = exp_dir.join("train.log");
    let mut log_file = fs::File::create(&log_path).unwrap_or_else(|e| {
        eprintln!("Error creating log file: {}", e);
        process::exit(1);
    });

    // Write config to log
    writeln!(log_file, "Weight Tuning Configuration").unwrap();
    writeln!(log_file, "===========================").unwrap();
    writeln!(log_file, "Experiment ID: {}", exp_id).unwrap();
    writeln!(log_file, "Mode: {}", args.mode).unwrap();
    writeln!(log_file, "Parallel: {}", args.parallel).unwrap();
    writeln!(log_file, "Generations: {}", args.generations).unwrap();
    writeln!(log_file, "Population: {}", cmaes_config.population_size.unwrap_or(4 + (3.0 * (20.0_f64).ln()).floor() as usize)).unwrap();
    writeln!(log_file, "Games/eval: {}", args.games).unwrap();
    writeln!(log_file, "Initial sigma: {:.3}", args.sigma).unwrap();
    writeln!(log_file, "Seed: {}", args.seed).unwrap();
    if args.mode == "multi-opponent" {
        writeln!(log_file, "MCTS sims: {}", args.mcts_sims).unwrap();
    }
    writeln!(log_file, "").unwrap();

    // Create optimizer and evaluator
    let mut cmaes = CmaEs::new(initial_weights, bounds, cmaes_config);
    let mut evaluator = Evaluator::new(&card_db, eval_config);

    // Track best result
    let mut best_weights: Vec<f64> = Vec::new();
    let mut best_fitness = f64::NEG_INFINITY;
    let mut best_win_rate = 0.0;

    let start_time = Instant::now();

    // Main optimization loop
    while !cmaes.should_stop(best_fitness) {
        let gen = cmaes.generation();
        let gen_start = Instant::now();

        // Sample population
        let population = cmaes.sample_population();

        // Evaluate each candidate
        let mut evaluated: Vec<(Vec<f64>, f64)> = Vec::with_capacity(population.len());

        for candidate in population {
            let result = evaluator.evaluate(&candidate);
            evaluated.push((candidate, result.fitness));

            // Track best
            if result.fitness > best_fitness {
                best_fitness = result.fitness;
                best_win_rate = result.win_rate;
                best_weights = evaluated.last().unwrap().0.clone();
            }
        }

        // Update CMA-ES
        cmaes.update(evaluated);

        let gen_time = gen_start.elapsed();

        // Print progress
        let progress_msg = format!(
            "Gen {:3}: best_fit={:6.2}, best_wr={:5.1}%, sigma={:.4}, time={:.1}s",
            gen,
            best_fitness,
            best_win_rate * 100.0,
            cmaes.sigma(),
            gen_time.as_secs_f64()
        );
        
        if args.verbose || gen % 5 == 0 || gen == 0 {
            println!("{}", progress_msg);
        }
        
        // Always log to file
        writeln!(log_file, "{}", progress_msg).unwrap();
    }

    let total_time = start_time.elapsed();
    let stop_reason = cmaes.stop_reason(best_fitness).unwrap_or("unknown");

    // Print final results
    let summary = format!("\n\
Optimization Complete\n\
=====================\n\
Stop reason: {}\n\
Total time: {:.1}s\n\
Generations: {}\n\
Evaluations: {}\n\
Best fitness: {:.2}\n\
Best win rate: {:.1}%\n",
        stop_reason,
        total_time.as_secs_f64(),
        cmaes.generation(),
        evaluator.eval_count(),
        best_fitness,
        best_win_rate * 100.0
    );
    
    println!("{}", summary);
    writeln!(log_file, "{}", summary).unwrap();

    // Convert best weights to GreedyWeights
    let weights_f32: Vec<f32> = best_weights.iter().map(|&x| x as f32).collect();
    if let Some(tuned_weights) = GreedyWeights::from_vec(&weights_f32) {
        println!("Tuned Weights:");
        println!("--------------");
        print_weights(&tuned_weights);

        // Save weights to experiment directory
        let weights_path = exp_dir.join("weights.toml");
        let bot_weights = BotWeights {
            name: format!("tuned_{}", args.mode),
            version: 1,
            default: WeightSet { greedy: tuned_weights },
            deck_specific: std::collections::HashMap::new(),
        };

        match bot_weights.save(&weights_path) {
            Ok(_) => {
                println!("\n✓ Weights saved to {:?}", weights_path);
                writeln!(log_file, "\nWeights saved to {:?}", weights_path).unwrap();
            }
            Err(e) => {
                eprintln!("\n❌ Error saving weights: {}", e);
                writeln!(log_file, "\nError saving weights: {}", e).unwrap();
            }
        }
    } else {
        eprintln!("Error: Could not reconstruct weights from vector");
    }

    // Save summary metadata
    let summary_path = exp_dir.join("summary.txt");
    let mut summary_file = fs::File::create(&summary_path).unwrap();
    writeln!(summary_file, "Experiment: {}", exp_id).unwrap();
    writeln!(summary_file, "Mode: {}", args.mode).unwrap();
    writeln!(summary_file, "Best Fitness: {:.2}", best_fitness).unwrap();
    writeln!(summary_file, "Best Win Rate: {:.1}%", best_win_rate * 100.0).unwrap();
    writeln!(summary_file, "Total Time: {:.1}s", total_time.as_secs_f64()).unwrap();
    writeln!(summary_file, "Generations: {}", cmaes.generation()).unwrap();
    
    println!("\n📁 All results saved to: {:?}", exp_dir);
    println!("   Run 'python python/scripts/analyze_tuning.py {:?}' to generate visualizations", exp_dir);
}

/// Create matchups for generalist mode using all available decks.
fn create_generalist_matchups(registry: &DeckRegistry, card_db: &CardDatabase) -> Vec<(Vec<CardId>, Vec<CardId>)> {
    let mut matchups = Vec::new();

    let decks: Vec<_> = registry.decks()
        .filter(|d| d.validate(card_db).is_ok())
        .collect();

    // Create all pairs
    for deck1 in &decks {
        for deck2 in &decks {
            matchups.push((deck1.to_card_ids(), deck2.to_card_ids()));
        }
    }

    matchups
}

/// Print weights in a readable format.
fn print_weights(w: &GreedyWeights) {
    println!("  Life:");
    println!("    own_life: {:.3}", w.own_life);
    println!("    enemy_life_damage: {:.3}", w.enemy_life_damage);
    println!("  Creatures:");
    println!("    own_creature_attack: {:.3}", w.own_creature_attack);
    println!("    own_creature_health: {:.3}", w.own_creature_health);
    println!("    enemy_creature_attack: {:.3}", w.enemy_creature_attack);
    println!("    enemy_creature_health: {:.3}", w.enemy_creature_health);
    println!("  Board:");
    println!("    creature_count: {:.3}", w.creature_count);
    println!("    board_advantage: {:.3}", w.board_advantage);
    println!("  Resources:");
    println!("    cards_in_hand: {:.3}", w.cards_in_hand);
    println!("    action_points: {:.3}", w.action_points);
    println!("  Keywords:");
    println!("    guard: {:.3}", w.keyword_guard);
    println!("    lethal: {:.3}", w.keyword_lethal);
    println!("    lifesteal: {:.3}", w.keyword_lifesteal);
    println!("    rush: {:.3}", w.keyword_rush);
    println!("    ranged: {:.3}", w.keyword_ranged);
    println!("    piercing: {:.3}", w.keyword_piercing);
    println!("    shield: {:.3}", w.keyword_shield);
    println!("    quick: {:.3}", w.keyword_quick);
    println!("  Terminal:");
    println!("    win_bonus: {:.1}", w.win_bonus);
    println!("    lose_penalty: {:.1}", w.lose_penalty);
}
