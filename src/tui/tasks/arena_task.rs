//! Arena background task for running bot matches

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::bots::{BotWeights, GreedyBot, GreedyWeights, MctsBot, MctsConfig, RandomBot, Bot};
use crate::cards::CardDatabase;
use crate::decks::DeckRegistry;
use crate::engine::GameEngine;
use crate::types::{CardId, PlayerId};

/// Configuration for an arena task
#[derive(Debug, Clone)]
pub struct ArenaConfig {
    pub bot1_type: BotType,
    pub bot2_type: BotType,
    pub deck1: String,
    pub deck2: String,
    pub weights1: Option<PathBuf>,
    pub weights2: Option<PathBuf>,
    pub games: u32,
    pub seed: Option<u64>,
    pub verbose: bool,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            bot1_type: BotType::Greedy,
            bot2_type: BotType::Random,
            deck1: String::new(),
            deck2: String::new(),
            weights1: None,
            weights2: None,
            games: 100,
            seed: None,
            verbose: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotType {
    Random,
    Greedy,
    Mcts,
}

impl BotType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BotType::Random => "Random",
            BotType::Greedy => "Greedy",
            BotType::Mcts => "MCTS",
        }
    }

    pub fn all() -> Vec<BotType> {
        vec![BotType::Random, BotType::Greedy, BotType::Mcts]
    }
}

/// Progress update from the arena task
#[derive(Debug, Clone)]
pub enum ArenaProgress {
    /// Task started
    Started,
    /// Progress update (current game, total games)
    Progress(u32, u32),
    /// Task completed with results
    Completed(ArenaResult),
    /// Task failed with error
    Error(String),
}

/// Results from arena matches
#[derive(Debug, Clone)]
pub struct ArenaResult {
    pub total_games: u32,
    pub player1_wins: u32,
    pub player2_wins: u32,
    pub draws: u32,
    pub player1_win_rate: f64,
    pub elapsed_secs: f64,
    pub games_per_sec: f64,
}

/// Handle to a running arena task
pub struct ArenaTaskHandle {
    pub receiver: Receiver<ArenaProgress>,
    handle: Option<JoinHandle<()>>,
}

impl ArenaTaskHandle {
    /// Check for new progress updates (non-blocking)
    pub fn try_recv(&self) -> Option<ArenaProgress> {
        self.receiver.try_recv().ok()
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

/// Spawn an arena task in a background thread
pub fn spawn_arena_task(config: ArenaConfig) -> ArenaTaskHandle {
    let (sender, receiver) = mpsc::channel();

    let handle = thread::spawn(move || {
        run_arena_task(config, sender);
    });

    ArenaTaskHandle {
        receiver,
        handle: Some(handle),
    }
}

fn run_arena_task(config: ArenaConfig, sender: Sender<ArenaProgress>) {
    let _ = sender.send(ArenaProgress::Started);

    // Load card database (path should be "data/cards" - the function appends "/sets" internally)
    let card_db: Arc<CardDatabase> = match CardDatabase::load_from_directory("data/cards/sets") {
        Ok(db) => Arc::new(db),
        Err(e) => {
            let _ = sender.send(ArenaProgress::Error(format!("Failed to load cards: {}", e)));
            return;
        }
    };
    // Get a reference to the inner CardDatabase for bot creation
    let card_db_ref: &CardDatabase = &*card_db;

    // Load deck registry
    let deck_registry = match DeckRegistry::load_from_directory("data/decks") {
        Ok(registry) => registry,
        Err(e) => {
            let _ = sender.send(ArenaProgress::Error(format!("Failed to load decks: {}", e)));
            return;
        }
    };

    // Get deck IDs (use first deck if not specified)
    let deck1_id = if config.deck1.is_empty() {
        deck_registry.decks().next().map(|d| d.id.clone())
    } else {
        Some(config.deck1.clone())
    };

    let deck2_id = if config.deck2.is_empty() {
        deck_registry.decks().next().map(|d| d.id.clone())
    } else {
        Some(config.deck2.clone())
    };

    let deck1 = match deck1_id.and_then(|id| deck_registry.get(&id).cloned()) {
        Some(d) => d,
        None => {
            let _ = sender.send(ArenaProgress::Error("Deck 1 not found".to_string()));
            return;
        }
    };

    let deck2 = match deck2_id.and_then(|id| deck_registry.get(&id).cloned()) {
        Some(d) => d,
        None => {
            let _ = sender.send(ArenaProgress::Error("Deck 2 not found".to_string()));
            return;
        }
    };

    // Load weights if specified (keep full BotWeights for MCTS, extract GreedyWeights for Greedy)
    let bot_weights1: Option<BotWeights> = config
        .weights1
        .as_ref()
        .and_then(|p| BotWeights::load(p).ok());
    let bot_weights2: Option<BotWeights> = config
        .weights2
        .as_ref()
        .and_then(|p| BotWeights::load(p).ok());

    let greedy_weights1: Option<GreedyWeights> = bot_weights1.as_ref().map(|w| w.default.greedy.clone());
    let greedy_weights2: Option<GreedyWeights> = bot_weights2.as_ref().map(|w| w.default.greedy.clone());

    let base_seed = config.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    let start_time = Instant::now();
    let mut player1_wins = 0u32;
    let mut player2_wins = 0u32;
    let mut draws = 0u32;

    // MCTS config for MCTS bots
    let mcts_config = MctsConfig {
        simulations: 500,
        exploration: 1.414,
        max_rollout_depth: 100,
        parallel_trees: 1,
        leaf_rollouts: 1,
    };

    for i in 0..config.games {
        let game_seed = base_seed.wrapping_add(i as u64);
        let bot1_seed = game_seed;
        let bot2_seed = game_seed.wrapping_add(1_000_000);

        // Create concrete bots - we need to keep them as concrete types
        // to call select_action_with_engine()
        let mut random_bot1 = RandomBot::new(bot1_seed);
        let mut random_bot2 = RandomBot::new(bot2_seed);

        let mut greedy_bot1 = match &greedy_weights1 {
            Some(w) => GreedyBot::with_weights(card_db_ref, w.clone(), bot1_seed),
            None => GreedyBot::new(card_db_ref, bot1_seed),
        };
        let mut greedy_bot2 = match &greedy_weights2 {
            Some(w) => GreedyBot::with_weights(card_db_ref, w.clone(), bot2_seed),
            None => GreedyBot::new(card_db_ref, bot2_seed),
        };

        let mut mcts_bot1 = match &bot_weights1 {
            Some(w) => MctsBot::with_config_and_weights(card_db_ref, mcts_config.clone(), w, bot1_seed),
            None => MctsBot::with_config(card_db_ref, mcts_config.clone(), bot1_seed),
        };
        let mut mcts_bot2 = match &bot_weights2 {
            Some(w) => MctsBot::with_config_and_weights(card_db_ref, mcts_config.clone(), w, bot2_seed),
            None => MctsBot::with_config(card_db_ref, mcts_config.clone(), bot2_seed),
        };

        // Create game engine - convert u16 card ids to CardId
        let deck1_cards: Vec<CardId> = deck1.cards.iter().map(|&id| CardId(id)).collect();
        let deck2_cards: Vec<CardId> = deck2.cards.iter().map(|&id| CardId(id)).collect();

        let mut engine = GameEngine::new(card_db_ref);
        engine.start_game(deck1_cards, deck2_cards, game_seed);

        // Run game
        let max_actions = 1000;
        let mut action_count = 0;

        while !engine.is_game_over() && action_count < max_actions {
            let current_player = engine.state.active_player;

            // Select action using the proper method for each bot type
            let action = if current_player.0 == 0 {
                match config.bot1_type {
                    BotType::Random => {
                        let state_tensor = engine.get_state_tensor();
                        let legal_mask = engine.get_legal_action_mask();
                        let legal_actions = engine.get_legal_actions();
                        random_bot1.select_action(&state_tensor, &legal_mask, &legal_actions)
                    }
                    BotType::Greedy => greedy_bot1.select_action_with_engine(&engine),
                    BotType::Mcts => mcts_bot1.select_action_with_engine(&engine),
                }
            } else {
                match config.bot2_type {
                    BotType::Random => {
                        let state_tensor = engine.get_state_tensor();
                        let legal_mask = engine.get_legal_action_mask();
                        let legal_actions = engine.get_legal_actions();
                        random_bot2.select_action(&state_tensor, &legal_mask, &legal_actions)
                    }
                    BotType::Greedy => greedy_bot2.select_action_with_engine(&engine),
                    BotType::Mcts => mcts_bot2.select_action_with_engine(&engine),
                }
            };

            let _ = engine.apply_action(action);
            action_count += 1;
        }

        // Record result
        match engine.winner() {
            Some(PlayerId(0)) => player1_wins += 1,
            Some(PlayerId(1)) => player2_wins += 1,
            _ => draws += 1,
        }

        // Send progress every 10 games or at the end
        if (i + 1) % 10 == 0 || i + 1 == config.games {
            let _ = sender.send(ArenaProgress::Progress(i + 1, config.games));
        }
    }

    let elapsed = start_time.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();

    let result = ArenaResult {
        total_games: config.games,
        player1_wins,
        player2_wins,
        draws,
        player1_win_rate: player1_wins as f64 / config.games as f64,
        elapsed_secs,
        games_per_sec: config.games as f64 / elapsed_secs,
    };

    let _ = sender.send(ArenaProgress::Completed(result));
}
