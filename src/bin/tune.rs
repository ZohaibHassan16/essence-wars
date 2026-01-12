//! Weight tuning CLI - Optimize bot weights using CMA-ES.
//!
//! Usage:
//!   cargo run --release --bin tune -- --generations 50 --population 20
//!   cargo run --release --bin tune -- --mode vs-greedy --games 100
//!   cargo run --release --bin tune -- --mode specialist --deck aggressive_assault --opponent defensive_control
//!   cargo run --release --bin tune -- --output tuned_weights.toml

use std::path::PathBuf;
use std::process;
use std::time::Instant;

use clap::Parser;

use cardgame::bots::{BotWeights, GreedyWeights, WeightSet};
use cardgame::cards::CardDatabase;
use cardgame::decks::DeckRegistry;
use cardgame::tuning::{CmaEs, CmaEsConfig, Evaluator, EvaluatorConfig, TuningMode};
use cardgame::types::CardId;

/// Weight tuning CLI using CMA-ES optimization
#[derive(Parser, Debug)]
#[command(name = "tune")]
#[command(about = "Optimize bot weights using CMA-ES evolution strategy", long_about = None)]
struct Args {
    /// Tuning mode: vs-random, vs-greedy, generalist, specialist
    #[arg(long, default_value = "vs-random")]
    mode: String,

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

    /// Output file for tuned weights (TOML format)
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

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
            eprintln!("Unknown mode: {}. Available: vs-random, vs-greedy, generalist, specialist", args.mode);
            process::exit(1);
        }
    };

    // Create evaluator config
    let eval_config = EvaluatorConfig {
        games_per_eval: args.games,
        mode: tuning_mode,
        seed: args.seed,
        max_actions: 500,
    };

    // Create CMA-ES config
    let target_fitness = args.target_win_rate.map(|wr| wr * 100.0);
    let cmaes_config = CmaEsConfig {
        population_size: args.population,
        initial_sigma: args.sigma,
        max_generations: args.generations,
        target_fitness,
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
    println!("Generations: {}", args.generations);
    println!("Population: {}", cmaes_config.population_size.unwrap_or(4 + (3.0 * (20.0_f64).ln()).floor() as usize));
    println!("Games/eval: {}", args.games);
    println!("Initial sigma: {:.3}", args.sigma);
    println!("Seed: {}", args.seed);
    if let Some(wr) = args.target_win_rate {
        println!("Target win rate: {:.1}%", wr * 100.0);
    }
    println!();

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
        if args.verbose || gen % 5 == 0 || gen == 0 {
            println!(
                "Gen {:3}: best_fit={:6.2}, best_wr={:5.1}%, sigma={:.4}, time={:.1}s",
                gen,
                best_fitness,
                best_win_rate * 100.0,
                cmaes.sigma(),
                gen_time.as_secs_f64()
            );
        }
    }

    let total_time = start_time.elapsed();

    // Print final results
    println!();
    println!("Optimization Complete");
    println!("=====================");
    println!("Total time: {:.1}s", total_time.as_secs_f64());
    println!("Generations: {}", cmaes.generation());
    println!("Evaluations: {}", evaluator.eval_count());
    println!("Best fitness: {:.2}", best_fitness);
    println!("Best win rate: {:.1}%", best_win_rate * 100.0);
    println!();

    // Convert best weights to GreedyWeights
    let weights_f32: Vec<f32> = best_weights.iter().map(|&x| x as f32).collect();
    if let Some(tuned_weights) = GreedyWeights::from_vec(&weights_f32) {
        println!("Tuned Weights:");
        println!("--------------");
        print_weights(&tuned_weights);

        // Save to file if requested
        if let Some(output_path) = args.output {
            let bot_weights = BotWeights {
                name: format!("tuned_{}", args.mode),
                version: 1,
                default: WeightSet { greedy: tuned_weights },
                deck_specific: std::collections::HashMap::new(),
            };

            match bot_weights.save(&output_path) {
                Ok(_) => println!("\nWeights saved to {:?}", output_path),
                Err(e) => eprintln!("\nError saving weights: {}", e),
            }
        }
    } else {
        eprintln!("Error: Could not reconstruct weights from vector");
    }
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
