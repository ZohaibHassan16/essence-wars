//! Weight tuning CLI - Optimize bot weights using CMA-ES.
//!
//! Usage:
//!   cargo run --release --bin tune -- --generations 50 --population 20
//!   cargo run --release --bin tune -- --mode vs-greedy --games 100
//!   cargo run --release --bin tune -- --mode specialist --deck broodmother_pack --opponent architect_fortify
//!   cargo run --release --bin tune -- --mode faction-specialist --faction argentum
//!   cargo run --release --bin tune -- --mode agent-generalist
//!   cargo run --release --bin tune -- --tag baseline
//!   cargo run --release --bin tune -- --resume 2026-01-31_1430_baseline

use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::time::Instant;

use chrono::Utc;
use clap::Parser;

use cardgame::bots::{BotWeights, GreedyWeights};
use cardgame::cards::CardDatabase;
use cardgame::decks::{DeckDefinition, DeckRegistry, Faction};
use cardgame::execution::GameData;
use cardgame::tuning::{
    deploy_weights, is_interrupted, load_checkpoint, register_interrupt_handler, save_checkpoint,
    CmaEs, CmaEsConfig, Evaluator, EvaluatorCheckpoint, EvaluatorConfig, ExperimentConfig,
    ExperimentDir, TuningCheckpoint, TuningConfig, TuningMode,
};
use cardgame::version;

/// Weight tuning CLI using CMA-ES optimization
#[derive(Parser, Debug)]
#[command(name = "tune")]
#[command(about = "Optimize bot weights using CMA-ES evolution strategy", long_about = None)]
struct Args {
    // ===== Tuning Mode =====
    /// Tuning mode for weight optimization
    ///
    /// Available modes:
    /// - generalist: Train across all deck matchups (vs Random/Greedy/AlphaBeta)
    /// - specialist: Train for specific deck matchup (requires --deck and --opponent)
    /// - faction-specialist: Train for a faction (requires --faction) [DEPRECATED]
    /// - archetype: Train for an archetype (requires --archetype: aggro/control/tempo/midrange)
    /// - alphabeta: Train Alpha-Beta weights against MCTS (uses --ab-depth)
    /// - alphabeta-specialist: Train Alpha-Beta weights for a faction (requires --faction and --ab-depth)
    #[arg(long, default_value = "generalist")]
    mode: String,

    /// Alpha-Beta search depth for alphabeta modes
    #[arg(long, default_value = "6")]
    ab_depth: u32,

    /// Faction for faction-specialist mode: argentum, symbiote, obsidion
    #[arg(long, requires_if("faction-specialist", "mode"))]
    faction: Option<String>,

    /// Archetype for archetype mode: aggro, control, tempo, midrange
    #[arg(long)]
    archetype: Option<String>,

    /// Deck ID for specialist mode (our deck)
    #[arg(long, requires_if("specialist", "mode"))]
    deck: Option<String>,

    /// Opponent deck ID for specialist mode
    #[arg(long, requires_if("specialist", "mode"))]
    opponent: Option<String>,

    // ===== Training Parameters =====
    /// Number of CMA-ES generations
    #[arg(long, short = 'g', default_value = "100")]
    generations: u32,

    /// Games per evaluation
    #[arg(long, default_value = "100")]
    games: usize,

    /// Population size (candidates per generation). Auto-calculated if not specified.
    #[arg(long, short = 'p')]
    population: Option<usize>,

    /// MCTS simulations per move (only for `alphabeta` mode vs MCTS opponent)
    #[arg(long, default_value = "50")]
    mcts_sims: u32,

    // ===== CMA-ES Hyperparameters =====
    /// Initial sigma (step size) for CMA-ES
    #[arg(long, default_value = "0.3")]
    sigma: f64,

    /// Minimum sigma to stop (convergence threshold)
    #[arg(long, default_value = "0.001")]
    min_sigma: f64,

    /// Target win rate to stop early (0.0 to 1.0)
    #[arg(long)]
    target_win_rate: Option<f64>,

    /// Start from existing weights file
    #[arg(long)]
    initial_weights: Option<PathBuf>,

    // ===== Experiment Configuration =====
    /// Experiment tag (descriptive name for this run)
    #[arg(long, short = 't', default_value = "default")]
    tag: String,

    /// Random seed
    #[arg(long, short = 's', default_value = "42")]
    seed: u64,

    /// Enable parallel game evaluation (uses all CPU cores)
    #[arg(long, default_value = "true")]
    parallel: bool,

    /// Verbose output (print each generation)
    #[arg(long, short = 'v')]
    verbose: bool,

    // ===== Data Paths =====
    /// Base output directory for experiments
    #[arg(long, default_value = "experiments")]
    experiment_dir: PathBuf,

    /// Path to card database
    #[arg(long, default_value = "data/cards/core_set")]
    cards: PathBuf,

    /// Path to commanders directory
    #[arg(long, default_value = "data/commanders")]
    commanders: PathBuf,

    /// Path to deck definitions directory
    #[arg(long, default_value = "data/decks")]
    decks: PathBuf,

    // ===== Resume =====
    /// Resume from a previous run checkpoint (run ID or partial match)
    ///
    /// Examples:
    ///   --resume baseline              # Matches "2026-01-31_1430_baseline"
    ///   --resume 2026-01-31_1430       # Exact prefix match
    #[arg(long)]
    resume: Option<String>,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    // Register interrupt handler for graceful shutdown
    register_interrupt_handler();

    // Determine category based on mode
    let category = if args.mode.starts_with("alphabeta") {
        "alphabeta"
    } else {
        "mcts"
    };

    // Check if we're resuming from a checkpoint
    let loaded_checkpoint = if let Some(ref resume_id) = args.resume {
        match load_checkpoint(resume_id, &args.experiment_dir, category) {
            Ok(checkpoint) => {
                println!("📂 Resuming from checkpoint: {}", checkpoint.run_id);
                println!("   Generation: {}", checkpoint.cmaes.generation);
                println!("   Best fitness: {:.2}", checkpoint.cmaes.best_fitness);
                println!("   Best win rate: {:.1}%", checkpoint.cmaes.best_win_rate * 100.0);
                println!();
                Some(checkpoint)
            }
            Err(e) => {
                eprintln!("Error loading checkpoint '{}': {}", resume_id, e);
                process::exit(1);
            }
        }
    } else {
        None
    };

    // Create or find experiment directory
    let experiment = if let Some(ref checkpoint) = loaded_checkpoint {
        // Use existing experiment directory
        let exp_dir = args.experiment_dir.join(category).join(&checkpoint.run_id);
        ExperimentDir::from_existing(&exp_dir)
    } else {
        // Create new experiment directory with timestamp
        let exp_config = ExperimentConfig::new(&args.experiment_dir, category, &args.tag);
        match ExperimentDir::create(&exp_config) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error creating experiment directory: {}", e);
                process::exit(1);
            }
        }
    };

    println!("📁 Experiment directory: {:?}", experiment.root);
    println!("🔖 Engine version: {}", version::version_string());
    println!();

    // Save version info for reproducibility
    if let Err(e) = experiment.save_version() {
        eprintln!("Warning: Could not save version info: {}", e);
    }

    // Load game data using unified loader
    let game_data = match GameData::load_with_overrides(
        Some(&args.cards),
        Some(&args.commanders),
        Some(&args.decks),
        None, // weights - not needed for tuning
        !args.verbose,
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading game data: {}", e);
            process::exit(1);
        }
    };
    let card_db = game_data.card_db;
    let deck_registry = game_data.deck_registry;

    // Parse tuning mode
    let tuning_mode = match args.mode.as_str() {
        "generalist" => {
            // Use all deck combinations
            let matchups = create_generalist_matchups(&deck_registry, &card_db);
            if matchups.is_empty() {
                eprintln!("No valid matchups found for generalist mode");
                process::exit(1);
            }
            println!("Generalist mode:");
            println!("  {} deck matchups", matchups.len());
            println!("  Testing vs Random, Greedy, AND AlphaBeta-{} per matchup", args.ab_depth);
            println!("  Total games per evaluation: {}", args.games);
            TuningMode::Generalist { matchups }
        }
        "specialist" => {
            let deck_id = args.deck.as_ref().expect("--deck required for specialist mode");
            let opponent_id = args.opponent.as_ref().expect("--opponent required for specialist mode");

            let deck = match deck_registry.get(deck_id) {
                Some(d) => d.clone(),
                None => {
                    eprintln!("Deck '{}' not found", deck_id);
                    process::exit(1);
                }
            };

            let opponent_deck = match deck_registry.get(opponent_id) {
                Some(d) => d.clone(),
                None => {
                    eprintln!("Opponent deck '{}' not found", opponent_id);
                    process::exit(1);
                }
            };

            println!("Specialist mode: {} vs {}", deck_id, opponent_id);
            println!("  Testing vs Random, Greedy, AND AlphaBeta-{}", args.ab_depth);
            println!("  Total games per evaluation: {}", args.games);
            TuningMode::Specialist { deck: Box::new(deck), opponent_deck: Box::new(opponent_deck) }
        }
        "faction-specialist" => {
            eprintln!("Warning: faction-specialist mode is deprecated. Consider using --mode archetype instead.");
            let faction_str = args.faction.as_ref().expect("--faction required for faction-specialist mode");
            let faction: Faction = faction_str.parse().unwrap_or_else(|e| {
                eprintln!("{}", e);
                process::exit(1);
            });

            // Get all decks for this faction
            let faction_decks = deck_registry.decks_for_faction(faction);
            if faction_decks.is_empty() {
                eprintln!("No decks found for faction '{}'", faction);
                process::exit(1);
            }

            // Get opponent decks from other factions
            let opponent_decks: Vec<_> = Faction::all_factions()
                .iter()
                .filter(|f| **f != faction)
                .flat_map(|f| deck_registry.decks_for_faction(*f))
                .collect();

            if opponent_decks.is_empty() {
                eprintln!("No opponent decks found for faction-specialist mode");
                process::exit(1);
            }

            // Create matchups: each faction deck vs each opponent deck
            let matchups = create_faction_matchups(&faction_decks, &opponent_decks, &card_db);

            println!("Faction Specialist mode: {}", faction);
            println!("  {} faction decks", faction_decks.len());
            println!("  {} opponent decks (from other factions)", opponent_decks.len());
            println!("  {} total matchups", matchups.len());
            println!("  Testing vs Random, Greedy, AND AlphaBeta-{} per matchup", args.ab_depth);
            println!("  Total games per evaluation: {}", args.games);
            TuningMode::Generalist { matchups }
        }
        "archetype" => {
            let archetype_str = args.archetype.as_ref().expect("--archetype required for archetype mode");
            let archetype_lower = archetype_str.to_lowercase();

            // Validate archetype
            if !matches!(archetype_lower.as_str(), "aggro" | "control" | "tempo" | "midrange") {
                eprintln!("Invalid archetype: {}. Valid archetypes: aggro, control, tempo, midrange", archetype_str);
                process::exit(1);
            }

            // Get all decks with this archetype (matching playstyle field)
            let archetype_decks: Vec<_> = deck_registry
                .decks()
                .filter(|d| d.playstyle.eq_ignore_ascii_case(archetype_str))
                .collect();

            if archetype_decks.is_empty() {
                eprintln!("No decks found with archetype '{}'", archetype_str);
                eprintln!("Available playstyles in decks:");
                for deck in deck_registry.decks() {
                    eprintln!("  {} -> {}", deck.id, deck.playstyle);
                }
                process::exit(1);
            }

            // Get opponent decks (all other archetypes)
            let opponent_decks: Vec<_> = deck_registry
                .decks()
                .filter(|d| !d.playstyle.eq_ignore_ascii_case(archetype_str))
                .collect();

            if opponent_decks.is_empty() {
                eprintln!("No opponent decks found for archetype mode");
                process::exit(1);
            }

            // Create matchups: each archetype deck vs each opponent deck
            let matchups = create_archetype_matchups(&archetype_decks, &opponent_decks, &card_db);

            println!("Archetype mode: {}", archetype_str);
            println!("  {} {} decks", archetype_decks.len(), archetype_str);
            println!("  {} opponent decks (other archetypes)", opponent_decks.len());
            println!("  {} total matchups", matchups.len());
            println!("  Testing vs Random, Greedy, AND AlphaBeta-{} per matchup", args.ab_depth);
            println!("  Total games per evaluation: {}", args.games);
            TuningMode::Generalist { matchups }
        }
        "alphabeta" => {
            // Use all deck combinations for Alpha-Beta tuning
            let matchups = create_generalist_matchups(&deck_registry, &card_db);
            if matchups.is_empty() {
                eprintln!("No valid matchups found for alphabeta mode");
                process::exit(1);
            }
            println!("Alpha-Beta mode:");
            println!("  {} deck matchups", matchups.len());
            println!("  Alpha-Beta depth: {}", args.ab_depth);
            println!("  Testing vs MCTS-{}", args.mcts_sims);
            println!("  Total games per evaluation: {}", args.games);
            TuningMode::AlphaBetaVsMcts { matchups, ab_depth: args.ab_depth }
        }
        "alphabeta-specialist" => {
            let faction_str = args.faction.as_ref().expect("--faction required for alphabeta-specialist mode");
            let faction: Faction = faction_str.parse().unwrap_or_else(|e| {
                eprintln!("{}", e);
                process::exit(1);
            });

            // Get all decks for this faction
            let faction_decks = deck_registry.decks_for_faction(faction);
            if faction_decks.is_empty() {
                eprintln!("No decks found for faction '{}'", faction);
                process::exit(1);
            }

            // Get opponent decks from other factions
            let opponent_decks: Vec<_> = Faction::all_factions()
                .iter()
                .filter(|f| **f != faction)
                .flat_map(|f| deck_registry.decks_for_faction(*f))
                .collect();

            if opponent_decks.is_empty() {
                eprintln!("No opponent decks found for alphabeta-specialist mode");
                process::exit(1);
            }

            // Create matchups: each faction deck vs each opponent deck
            let matchups = create_faction_matchups(&faction_decks, &opponent_decks, &card_db);

            println!("Alpha-Beta Faction Specialist mode:");
            println!("  Faction: {:?}", faction);
            println!("  {} faction decks", faction_decks.len());
            println!("  {} opponent decks", opponent_decks.len());
            println!("  {} deck matchups", matchups.len());
            println!("  Alpha-Beta depth: {}", args.ab_depth);
            println!("  Testing vs MCTS-{}", args.mcts_sims);
            println!("  Total games per evaluation: {}", args.games);

            TuningMode::AlphaBetaVsMcts { matchups, ab_depth: args.ab_depth }
        }
        _ => {
            eprintln!("Unknown mode: {}. Available modes: generalist, specialist, archetype, faction-specialist, alphabeta, alphabeta-specialist", args.mode);
            process::exit(1);
        }
    };

    // Create evaluator config
    let eval_config = EvaluatorConfig {
        games_per_eval: args.games,
        mode: tuning_mode,
        candidate_type: cardgame::tuning::CandidateType::default(),
        seed: args.seed,
        max_actions: 500,
        parallel: args.parallel,
        mcts_sims: args.mcts_sims,
        ab_depth: args.ab_depth,
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
    println!("Population: {}", cmaes_config.population_size.unwrap_or(4 + (3.0_f64 * (initial_weights.len() as f64).ln()).floor() as usize));
    println!("Games/eval: {}", args.games);
    println!("Initial sigma: {:.3}", args.sigma);
    println!("Seed: {}", args.seed);
    if let Some(wr) = args.target_win_rate {
        println!("Target win rate: {:.1}%", wr * 100.0);
    }
    println!();

    // Create log file
    let log_path = experiment.root.join("train.log");
    let mut log_file = match std::fs::File::create(&log_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error creating log file: {}", e);
            process::exit(1);
        }
    };

    // Write config to log
    writeln!(log_file, "Weight Tuning Configuration").unwrap();
    writeln!(log_file, "===========================").unwrap();
    writeln!(log_file, "Experiment ID: {}", experiment.id).unwrap();
    writeln!(log_file, "Mode: {}", args.mode).unwrap();
    writeln!(log_file, "Parallel: {}", args.parallel).unwrap();
    writeln!(log_file, "Generations: {}", args.generations).unwrap();
    writeln!(log_file, "Population: {}", cmaes_config.population_size.unwrap_or(4 + (3.0_f64 * (initial_weights.len() as f64).ln()).floor() as usize)).unwrap();
    writeln!(log_file, "Games/eval: {}", args.games).unwrap();
    writeln!(log_file, "Initial sigma: {:.3}", args.sigma).unwrap();
    writeln!(log_file, "Seed: {}", args.seed).unwrap();
    writeln!(log_file).unwrap();

    // Create or restore optimizer and evaluator
    let (mut cmaes, mut best_weights, mut best_fitness, mut best_win_rate, elapsed_before_resume) =
        if let Some(ref checkpoint) = loaded_checkpoint {
            // Restore from checkpoint
            let cmaes = match CmaEs::from_checkpoint(&checkpoint.cmaes, bounds, cmaes_config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error restoring CMA-ES from checkpoint: {}", e);
                    process::exit(1);
                }
            };

            let best_weights = checkpoint.cmaes.best_weights.clone();
            let best_fitness = checkpoint.cmaes.best_fitness;
            let best_win_rate = checkpoint.cmaes.best_win_rate;
            let elapsed = checkpoint.elapsed_secs;

            (cmaes, best_weights, best_fitness, best_win_rate, elapsed)
        } else {
            // Fresh start
            let cmaes = CmaEs::new(initial_weights, bounds, cmaes_config);
            (cmaes, Vec::new(), f64::NEG_INFINITY, 0.0, 0.0)
        };

    let mut evaluator = Evaluator::new(&card_db, eval_config);

    // Restore evaluator eval_count if resuming (for deterministic seeding)
    if let Some(ref checkpoint) = loaded_checkpoint {
        evaluator.set_eval_count(checkpoint.evaluator.eval_count);
    }

    let start_time = Instant::now();

    // Build TuningConfig for checkpoint
    let tuning_config = TuningConfig {
        mode: args.mode.clone(),
        generations: args.generations,
        games_per_eval: args.games,
        population_size: args.population,
        initial_sigma: args.sigma,
        seed: args.seed,
        mcts_sims: args.mcts_sims,
        ab_depth: args.ab_depth,
        deck: args.deck.clone(),
        opponent: args.opponent.clone(),
        faction: args.faction.clone(),
        archetype: args.archetype.clone(),
    };

    // Main optimization loop
    let mut interrupted = false;
    while !cmaes.should_stop(best_fitness) {
        // Check for interrupt before starting generation
        if is_interrupted() {
            interrupted = true;
            break;
        }

        let gen = cmaes.generation();
        let gen_start = Instant::now();

        // Sample population
        let population = cmaes.sample_population();

        // Evaluate each candidate
        let mut evaluated: Vec<(Vec<f64>, f64)> = Vec::with_capacity(population.len());

        for candidate in population {
            // Check for interrupt during evaluation
            if is_interrupted() {
                interrupted = true;
                break;
            }

            let result = evaluator.evaluate(&candidate);
            evaluated.push((candidate, result.fitness));

            // Track best
            if result.fitness > best_fitness {
                best_fitness = result.fitness;
                best_win_rate = result.win_rate;
                best_weights = evaluated.last().unwrap().0.clone();
            }
        }

        // Exit if interrupted during evaluation
        if interrupted {
            break;
        }

        // Update CMA-ES
        cmaes.update(evaluated);

        let gen_time = gen_start.elapsed();
        let total_elapsed = elapsed_before_resume + start_time.elapsed().as_secs_f64();

        // Print progress
        let progress_msg = format!(
            "Gen {:3}: best_fit={:6.2}, best_wr={:5.1}%, sigma={:.4}, time={:.1}s",
            gen,
            best_fitness,
            best_win_rate * 100.0,
            cmaes.sigma(),
            gen_time.as_secs_f64()
        );

        if args.verbose || gen.is_multiple_of(5) || gen == 0 {
            println!("{}", progress_msg);
        }

        // Always log to file
        writeln!(log_file, "{}", progress_msg).unwrap();

        // Save checkpoint after each generation
        let checkpoint = TuningCheckpoint {
            version: TuningCheckpoint::VERSION,
            created_at: Utc::now().to_rfc3339(),
            run_id: experiment.id.clone(),
            cmaes: cmaes.to_checkpoint(&best_weights, best_win_rate),
            evaluator: EvaluatorCheckpoint {
                eval_count: evaluator.eval_count(),
            },
            config: tuning_config.clone(),
            elapsed_secs: total_elapsed,
        };

        if let Err(e) = save_checkpoint(&checkpoint, &experiment.root) {
            eprintln!("Warning: Could not save checkpoint: {}", e);
        }
    }

    // Handle interrupt - save final checkpoint
    if interrupted {
        let total_elapsed = elapsed_before_resume + start_time.elapsed().as_secs_f64();
        let checkpoint = TuningCheckpoint {
            version: TuningCheckpoint::VERSION,
            created_at: Utc::now().to_rfc3339(),
            run_id: experiment.id.clone(),
            cmaes: cmaes.to_checkpoint(&best_weights, best_win_rate),
            evaluator: EvaluatorCheckpoint {
                eval_count: evaluator.eval_count(),
            },
            config: tuning_config.clone(),
            elapsed_secs: total_elapsed,
        };

        match save_checkpoint(&checkpoint, &experiment.root) {
            Ok(path) => {
                eprintln!("\n✅ Checkpoint saved to {:?}", path);
                eprintln!("   Resume with: cargo run --release --bin tune -- --resume {}", experiment.id);
            }
            Err(e) => {
                eprintln!("\n❌ Error saving checkpoint: {}", e);
            }
        }

        println!("\n⚠️  Training interrupted at generation {}", cmaes.generation());
        process::exit(0);
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

        // Determine weight name based on mode
        let weight_name = if args.mode == "faction-specialist" {
            let faction_str = args.faction.as_ref().expect("faction should be set");
            format!("agent_{}", faction_str.to_lowercase())
        } else if args.mode == "archetype" {
            let archetype_str = args.archetype.as_ref().expect("archetype should be set");
            format!("archetype_{}", archetype_str.to_lowercase())
        } else if args.mode == "agent-generalist" {
            "agent_generalist".to_string()
        } else {
            format!("tuned_{}", args.mode)
        };

        // Save weights to experiment directory
        match experiment.save_weights(&tuned_weights, &weight_name) {
            Ok(path) => {
                println!("\n✓ Weights saved to {:?}", path);
                writeln!(log_file, "\nWeights saved to {:?}", path).unwrap();
            }
            Err(e) => {
                eprintln!("\n❌ Error saving weights: {}", e);
                writeln!(log_file, "\nError saving weights: {}", e).unwrap();
            }
        }

        // Auto-deploy: Copy weights to data/weights/ for easy access
        // For archetype mode, pass the archetype; otherwise pass faction
        let deploy_key = args.archetype.as_deref().or(args.faction.as_deref());
        match deploy_weights(
            &tuned_weights,
            &weight_name,
            &args.mode,
            deploy_key,
            args.deck.as_deref(),
        ) {
            Ok(Some(path)) => {
                println!("✅ Auto-deployed to {:?}", path);
                writeln!(log_file, "Auto-deployed to {:?}", path).unwrap();
            }
            Ok(None) => {} // No deployment path for this mode
            Err(e) => {
                eprintln!("⚠️  Warning: Could not auto-deploy weights: {}", e);
            }
        }
    } else {
        eprintln!("Error: Could not reconstruct weights from vector");
    }

    // Save summary metadata
    if let Err(e) = experiment.save_summary(
        &args.mode,
        best_fitness,
        best_win_rate,
        total_time.as_secs_f64(),
        cmaes.generation(),
    ) {
        eprintln!("Warning: Could not save summary: {}", e);
    }

    println!("\n📁 All results saved to: {:?}", experiment.root);
    println!(
        "   Run 'python python/scripts/analyze_tuning.py {:?}' to generate visualizations",
        experiment.root
    );
}

/// Create matchups for generalist mode using all available decks.
fn create_generalist_matchups(registry: &DeckRegistry, card_db: &CardDatabase) -> Vec<(DeckDefinition, DeckDefinition)> {
    let mut matchups = Vec::new();

    let decks: Vec<_> = registry.decks()
        .filter(|d| d.validate(card_db).is_ok())
        .collect();

    // Create all pairs
    for deck1 in &decks {
        for deck2 in &decks {
            matchups.push(((*deck1).clone(), (*deck2).clone()));
        }
    }

    matchups
}

/// Create matchups for faction-specialist mode.
///
/// Creates all combinations of faction decks vs opponent decks.
fn create_faction_matchups(
    faction_decks: &[&DeckDefinition],
    opponent_decks: &[&DeckDefinition],
    card_db: &CardDatabase,
) -> Vec<(DeckDefinition, DeckDefinition)> {
    let mut matchups = Vec::new();

    for faction_deck in faction_decks {
        if faction_deck.validate(card_db).is_err() {
            continue;
        }

        for opponent_deck in opponent_decks {
            if opponent_deck.validate(card_db).is_err() {
                continue;
            }

            matchups.push(((*faction_deck).clone(), (*opponent_deck).clone()));
        }
    }

    matchups
}

/// Create matchups for archetype mode.
///
/// Creates all combinations of archetype decks vs opponent decks.
fn create_archetype_matchups(
    archetype_decks: &[&DeckDefinition],
    opponent_decks: &[&DeckDefinition],
    card_db: &CardDatabase,
) -> Vec<(DeckDefinition, DeckDefinition)> {
    let mut matchups = Vec::new();

    for arch_deck in archetype_decks {
        if arch_deck.validate(card_db).is_err() {
            continue;
        }

        for opponent_deck in opponent_decks {
            if opponent_deck.validate(card_db).is_err() {
                continue;
            }

            matchups.push(((*arch_deck).clone(), (*opponent_deck).clone()));
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
