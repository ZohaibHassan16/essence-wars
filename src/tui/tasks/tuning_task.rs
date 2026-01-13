//! Tuning background task for CMA-ES weight optimization

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;
use std::fs;
use std::io::Write;

use crate::bots::{BotWeights, GreedyWeights, WeightSet};
use crate::cards::CardDatabase;
use crate::decks::DeckRegistry;
use crate::tuning::{CmaEs, CmaEsConfig, Evaluator, EvaluatorConfig, TuningMode};
use crate::types::CardId;
use crate::version::{self, VersionInfo};

/// Configuration for a tuning task
#[derive(Debug, Clone)]
pub struct TuningConfig {
    pub mode: TuningModeConfig,
    pub generations: u32,
    pub population: Option<usize>,
    pub games_per_eval: usize,
    pub sigma: f64,
    pub min_sigma: f64,
    pub seed: u64,
    pub target_win_rate: Option<f64>,
    pub mcts_sims: u32,
    pub parallel: bool,
    pub tag: String,
    pub experiment_dir: PathBuf,
    pub initial_weights: Option<PathBuf>,
    /// Deck ID for specialist mode
    pub deck: Option<String>,
    /// Opponent deck ID for specialist mode
    pub opponent_deck: Option<String>,
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            mode: TuningModeConfig::VsRandom,
            generations: 50,
            population: None,
            games_per_eval: 50,
            sigma: 0.3,
            min_sigma: 0.001,
            seed: 42,
            target_win_rate: None,
            mcts_sims: 200,
            parallel: true,
            tag: "default".to_string(),
            experiment_dir: PathBuf::from("experiments"),
            initial_weights: None,
            deck: None,
            opponent_deck: None,
        }
    }
}

/// Tuning mode configuration (simpler enum for UI)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuningModeConfig {
    VsRandom,
    VsGreedy,
    MultiOpponent,
    Generalist,
    Specialist,
}

impl TuningModeConfig {
    pub fn as_str(&self) -> &'static str {
        match self {
            TuningModeConfig::VsRandom => "vs-random",
            TuningModeConfig::VsGreedy => "vs-greedy",
            TuningModeConfig::MultiOpponent => "multi-opponent",
            TuningModeConfig::Generalist => "generalist",
            TuningModeConfig::Specialist => "specialist",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TuningModeConfig::VsRandom => "Optimize against Random (easy baseline)",
            TuningModeConfig::VsGreedy => "Optimize against default Greedy",
            TuningModeConfig::MultiOpponent => "Optimize against Random+Greedy+MCTS",
            TuningModeConfig::Generalist => "Optimize across all deck matchups",
            TuningModeConfig::Specialist => "Optimize for specific deck matchup",
        }
    }

    pub fn all() -> Vec<TuningModeConfig> {
        vec![
            TuningModeConfig::VsRandom,
            TuningModeConfig::VsGreedy,
            TuningModeConfig::MultiOpponent,
            TuningModeConfig::Generalist,
            TuningModeConfig::Specialist,
        ]
    }
}

/// Progress update from the tuning task
#[derive(Debug, Clone)]
pub enum TuningProgress {
    /// Task started, includes experiment directory path
    Started(PathBuf),
    /// Generation completed
    Generation(GenerationStats),
    /// Task completed with final results
    Completed(TuningResult),
    /// Task paused
    Paused,
    /// Task resumed
    Resumed,
    /// Task stopped early by user
    Stopped(TuningResult),
    /// Task failed with error
    Error(String),
}

/// Statistics for a single generation
#[derive(Debug, Clone)]
pub struct GenerationStats {
    pub generation: u32,
    pub total_generations: u32,
    pub best_fitness: f64,
    pub best_win_rate: f64,
    pub sigma: f64,
    pub elapsed_secs: f64,
    pub gen_time_secs: f64,
}

/// Final results from tuning
#[derive(Debug, Clone)]
pub struct TuningResult {
    pub total_generations: u32,
    pub total_evaluations: u64,
    pub best_fitness: f64,
    pub best_win_rate: f64,
    pub elapsed_secs: f64,
    pub stop_reason: String,
    pub weights_path: PathBuf,
    pub experiment_dir: PathBuf,
}

/// Control commands for the tuning task
#[derive(Debug, Clone)]
pub enum TuningCommand {
    Pause,
    Resume,
    Stop,
}

/// Handle to a running tuning task
pub struct TuningTaskHandle {
    pub receiver: Receiver<TuningProgress>,
    command_sender: Sender<TuningCommand>,
    handle: Option<JoinHandle<()>>,
    paused: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
}

impl TuningTaskHandle {
    /// Check for new progress updates (non-blocking)
    pub fn try_recv(&self) -> Option<TuningProgress> {
        self.receiver.try_recv().ok()
    }

    /// Send a pause command
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
        let _ = self.command_sender.send(TuningCommand::Pause);
    }

    /// Send a resume command
    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
        let _ = self.command_sender.send(TuningCommand::Resume);
    }

    /// Send a stop command
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        let _ = self.command_sender.send(TuningCommand::Stop);
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    /// Check if stopped
    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }

    /// Wait for the task to complete
    pub fn join(mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    /// Check if the task is still running
    pub fn is_finished(&self) -> bool {
        self.handle.as_ref().map(|h| h.is_finished()).unwrap_or(true)
    }
}

/// Spawn a tuning task in a background thread
pub fn spawn_tuning_task(config: TuningConfig) -> TuningTaskHandle {
    let (progress_sender, progress_receiver) = mpsc::channel();
    let (command_sender, command_receiver) = mpsc::channel();
    let paused = Arc::new(AtomicBool::new(false));
    let stopped = Arc::new(AtomicBool::new(false));

    let paused_clone = paused.clone();
    let stopped_clone = stopped.clone();

    let handle = thread::spawn(move || {
        run_tuning_task(config, progress_sender, command_receiver, paused_clone, stopped_clone);
    });

    TuningTaskHandle {
        receiver: progress_receiver,
        command_sender,
        handle: Some(handle),
        paused,
        stopped,
    }
}

fn run_tuning_task(
    config: TuningConfig,
    sender: Sender<TuningProgress>,
    _command_receiver: Receiver<TuningCommand>,
    paused: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
) {
    // Create experiment directory with timestamp
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H%M").to_string();
    let exp_id = format!("{}_{}", timestamp, config.tag);
    let exp_dir = config.experiment_dir.join("mcts").join(&exp_id);

    if let Err(e) = fs::create_dir_all(&exp_dir) {
        let _ = sender.send(TuningProgress::Error(format!(
            "Failed to create experiment directory: {}",
            e
        )));
        return;
    }

    let plots_dir = exp_dir.join("plots");
    let _ = fs::create_dir_all(&plots_dir);

    let _ = sender.send(TuningProgress::Started(exp_dir.clone()));

    // Save version info
    let version_info = VersionInfo::current();
    let version_path = exp_dir.join("version.toml");
    if let Ok(toml_str) = toml::to_string_pretty(&version_info) {
        let _ = fs::write(&version_path, toml_str);
    }

    // Load card database
    let card_db = match CardDatabase::load_from_directory("data/cards/sets") {
        Ok(db) => db,
        Err(e) => {
            let _ = sender.send(TuningProgress::Error(format!("Failed to load cards: {}", e)));
            return;
        }
    };

    // Load deck registry
    let deck_registry = match DeckRegistry::load_from_directory("data/decks") {
        Ok(r) => r,
        Err(e) => {
            if matches!(config.mode, TuningModeConfig::Specialist | TuningModeConfig::Generalist) {
                let _ = sender.send(TuningProgress::Error(format!("Failed to load decks: {}", e)));
                return;
            }
            DeckRegistry::new()
        }
    };

    // Convert mode config to actual TuningMode
    let tuning_mode = match config.mode {
        TuningModeConfig::VsRandom => TuningMode::VsRandom,
        TuningModeConfig::VsGreedy => TuningMode::VsGreedy,
        TuningModeConfig::MultiOpponent => TuningMode::MultiOpponent,
        TuningModeConfig::Generalist => {
            let matchups = create_generalist_matchups(&deck_registry, &card_db);
            if matchups.is_empty() {
                let _ = sender.send(TuningProgress::Error(
                    "No valid matchups found for generalist mode".to_string(),
                ));
                return;
            }
            TuningMode::Generalist { matchups }
        }
        TuningModeConfig::Specialist => {
            let deck_id = match &config.deck {
                Some(id) => id,
                None => {
                    let _ = sender.send(TuningProgress::Error(
                        "Deck required for specialist mode".to_string(),
                    ));
                    return;
                }
            };
            let opponent_id = match &config.opponent_deck {
                Some(id) => id,
                None => {
                    let _ = sender.send(TuningProgress::Error(
                        "Opponent deck required for specialist mode".to_string(),
                    ));
                    return;
                }
            };

            let deck = match deck_registry.get(deck_id) {
                Some(d) => d.to_card_ids(),
                None => {
                    let _ = sender.send(TuningProgress::Error(format!("Deck '{}' not found", deck_id)));
                    return;
                }
            };

            let opponent_deck = match deck_registry.get(opponent_id) {
                Some(d) => d.to_card_ids(),
                None => {
                    let _ = sender.send(TuningProgress::Error(format!(
                        "Opponent deck '{}' not found",
                        opponent_id
                    )));
                    return;
                }
            };

            TuningMode::Specialist { deck, opponent_deck }
        }
    };

    // Create evaluator config
    let eval_config = EvaluatorConfig {
        games_per_eval: config.games_per_eval,
        mode: tuning_mode,
        seed: config.seed,
        max_actions: 500,
        parallel: config.parallel,
        mcts_sims: config.mcts_sims,
    };

    // Create CMA-ES config
    let target_fitness = config.target_win_rate.map(|wr| wr * 100.0);
    let cmaes_config = CmaEsConfig {
        population_size: config.population,
        initial_sigma: config.sigma,
        max_generations: config.generations,
        target_fitness,
        min_sigma: config.min_sigma,
        seed: config.seed,
    };

    // Initial weights
    let initial_weights: Vec<f64> = if let Some(ref path) = config.initial_weights {
        match BotWeights::load(path) {
            Ok(w) => w.default.greedy.to_vec().iter().map(|&x| x as f64).collect(),
            Err(_) => GreedyWeights::default().to_vec().iter().map(|&x| x as f64).collect(),
        }
    } else {
        GreedyWeights::default().to_vec().iter().map(|&x| x as f64).collect()
    };

    // Parameter bounds
    let bounds: Vec<(f64, f64)> = GreedyWeights::bounds()
        .iter()
        .map(|&(min, max)| (min as f64, max as f64))
        .collect();

    // Create log file
    let log_path = exp_dir.join("train.log");
    let mut log_file = match fs::File::create(&log_path) {
        Ok(f) => f,
        Err(e) => {
            let _ = sender.send(TuningProgress::Error(format!("Failed to create log: {}", e)));
            return;
        }
    };

    // Write config to log
    let _ = writeln!(log_file, "Weight Tuning - TUI");
    let _ = writeln!(log_file, "==================");
    let _ = writeln!(log_file, "Experiment ID: {}", exp_id);
    let _ = writeln!(log_file, "Mode: {}", config.mode.as_str());
    let _ = writeln!(log_file, "Generations: {}", config.generations);
    let _ = writeln!(log_file, "Games/eval: {}", config.games_per_eval);
    let _ = writeln!(log_file, "Sigma: {:.3}", config.sigma);
    let _ = writeln!(log_file, "Seed: {}", config.seed);
    let _ = writeln!(log_file, "Version: {}", version::version_string());
    let _ = writeln!(log_file);

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
        // Check for stop command
        if stopped.load(Ordering::SeqCst) {
            break;
        }

        // Check for pause
        while paused.load(Ordering::SeqCst) && !stopped.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_millis(100));
        }

        let gen = cmaes.generation();
        let gen_start = Instant::now();

        // Sample population
        let population = cmaes.sample_population();

        // Evaluate each candidate
        let mut evaluated: Vec<(Vec<f64>, f64)> = Vec::with_capacity(population.len());

        for candidate in population {
            // Check for stop during evaluation
            if stopped.load(Ordering::SeqCst) {
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

        if stopped.load(Ordering::SeqCst) {
            break;
        }

        // Update CMA-ES
        cmaes.update(evaluated);

        let gen_time = gen_start.elapsed();
        let total_elapsed = start_time.elapsed();

        // Log progress
        let progress_msg = format!(
            "Gen {:3}: best_fit={:6.2}, best_wr={:5.1}%, sigma={:.4}, time={:.1}s",
            gen, best_fitness, best_win_rate * 100.0, cmaes.sigma(), gen_time.as_secs_f64()
        );
        let _ = writeln!(log_file, "{}", progress_msg);

        // Send progress update
        let stats = GenerationStats {
            generation: gen,
            total_generations: config.generations,
            best_fitness,
            best_win_rate,
            sigma: cmaes.sigma(),
            elapsed_secs: total_elapsed.as_secs_f64(),
            gen_time_secs: gen_time.as_secs_f64(),
        };
        let _ = sender.send(TuningProgress::Generation(stats));
    }

    let total_elapsed = start_time.elapsed();
    let was_stopped = stopped.load(Ordering::SeqCst);

    let stop_reason = if was_stopped {
        "user stopped".to_string()
    } else {
        cmaes.stop_reason(best_fitness).unwrap_or("unknown").to_string()
    };

    // Save weights
    let weights_path = exp_dir.join("weights.toml");
    let weights_f32: Vec<f32> = best_weights.iter().map(|&x| x as f32).collect();
    if let Some(tuned_weights) = GreedyWeights::from_vec(&weights_f32) {
        let bot_weights = BotWeights {
            name: format!("tuned_{}", config.mode.as_str()),
            version: 1,
            default: WeightSet { greedy: tuned_weights },
            deck_specific: std::collections::HashMap::new(),
        };
        let _ = bot_weights.save(&weights_path);
    }

    // Save summary
    let summary_path = exp_dir.join("summary.txt");
    if let Ok(mut f) = fs::File::create(&summary_path) {
        let _ = writeln!(f, "Experiment: {}", exp_id);
        let _ = writeln!(f, "Mode: {}", config.mode.as_str());
        let _ = writeln!(f, "Best Fitness: {:.2}", best_fitness);
        let _ = writeln!(f, "Best Win Rate: {:.1}%", best_win_rate * 100.0);
        let _ = writeln!(f, "Total Time: {:.1}s", total_elapsed.as_secs_f64());
        let _ = writeln!(f, "Generations: {}", cmaes.generation());
        let _ = writeln!(f, "Stop Reason: {}", stop_reason);
    }

    let result = TuningResult {
        total_generations: cmaes.generation(),
        total_evaluations: evaluator.eval_count(),
        best_fitness,
        best_win_rate,
        elapsed_secs: total_elapsed.as_secs_f64(),
        stop_reason: stop_reason.clone(),
        weights_path,
        experiment_dir: exp_dir,
    };

    if was_stopped {
        let _ = sender.send(TuningProgress::Stopped(result));
    } else {
        let _ = sender.send(TuningProgress::Completed(result));
    }
}

/// Create matchups for generalist mode using all available decks.
fn create_generalist_matchups(
    registry: &DeckRegistry,
    card_db: &CardDatabase,
) -> Vec<(Vec<CardId>, Vec<CardId>)> {
    let mut matchups = Vec::new();

    let decks: Vec<_> = registry
        .decks()
        .filter(|d| d.validate(card_db).is_ok())
        .collect();

    for deck1 in &decks {
        for deck2 in &decks {
            matchups.push((deck1.to_card_ids(), deck2.to_card_ids()));
        }
    }

    matchups
}
