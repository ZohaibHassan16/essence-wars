//! Evaluator for measuring bot performance through matches.
//!
//! Supports:
//! - Generalist mode: Evaluate across multiple deck matchups
//! - Specialist mode: Optimize for a single deck matchup
//! - Multi-opponent evaluation for robust weights
//! - Parallel game execution for fast evaluation

use std::time::{Duration, Instant};

use rayon::prelude::*;

use crate::bots::{Bot, GreedyBot, GreedyWeights, MctsBot, MctsConfig, RandomBot};
use crate::cards::CardDatabase;
use crate::engine::GameEngine;
use crate::types::{CardId, PlayerId};

/// Tuning mode for weight optimization.
#[derive(Clone, Debug)]
pub enum TuningMode {
    /// Optimize weights to perform well against Random baseline
    VsRandom,
    /// Optimize weights to perform well against default Greedy baseline
    VsGreedy,
    /// Optimize weights against multiple opponents (Random, Greedy, MCTS)
    MultiOpponent,
    /// Optimize weights across multiple deck matchups (generalist)
    Generalist {
        /// List of (deck1, deck2) matchups to evaluate
        matchups: Vec<(Vec<CardId>, Vec<CardId>)>,
    },
    /// Optimize weights for a specific deck matchup (specialist)
    Specialist {
        /// Our deck
        deck: Vec<CardId>,
        /// Opponent's deck
        opponent_deck: Vec<CardId>,
    },
}

/// Configuration for the evaluator.
#[derive(Clone, Debug)]
pub struct EvaluatorConfig {
    /// Number of games per evaluation
    pub games_per_eval: usize,
    /// Tuning mode
    pub mode: TuningMode,
    /// Base random seed
    pub seed: u64,
    /// Maximum actions per game (prevents infinite games)
    pub max_actions: usize,
    /// Run games in parallel
    pub parallel: bool,
    /// MCTS simulations for multi-opponent mode
    pub mcts_sims: u32,
}

impl Default for EvaluatorConfig {
    fn default() -> Self {
        Self {
            games_per_eval: 50,
            mode: TuningMode::VsRandom,
            seed: 42,
            max_actions: 500,
            parallel: true,
            mcts_sims: 100, // Fast MCTS for tuning
        }
    }
}

/// Result of fitness evaluation.
#[derive(Clone, Debug)]
pub struct FitnessResult {
    /// Primary fitness score (higher is better)
    pub fitness: f64,
    /// Win rate (0.0 to 1.0)
    pub win_rate: f64,
    /// Number of games played
    pub games: usize,
    /// Average game length in turns
    pub avg_turns: f64,
    /// Total evaluation time
    pub eval_time: Duration,
}

/// Evaluator for measuring bot performance.
pub struct Evaluator<'a> {
    card_db: &'a CardDatabase,
    config: EvaluatorConfig,
    default_deck: Vec<CardId>,
    eval_count: u64,
}

impl<'a> Evaluator<'a> {
    /// Create a new evaluator.
    pub fn new(card_db: &'a CardDatabase, config: EvaluatorConfig) -> Self {
        // Default deck for simple evaluations (Core Set IDs)
        // Uses a mix of factions for balanced testing
        let default_deck = vec![
            // Argentum (1000+) - 6 cards
            CardId(1000), CardId(1000), // Iron Sentinel (2/4 Guard)
            CardId(1001), CardId(1001), // Brass Automaton (2/3)
            CardId(1004), CardId(1004), // Steel Vanguard (3/5 Guard)
            // Symbiote (2000+) - 6 cards
            CardId(2000), CardId(2000), // Spore Crawler (1/2)
            CardId(2003), CardId(2003), // Broodling (1/1 Rush)
            CardId(2005), CardId(2005), // Pack Hunter (2/2 Rush)
            // Obsidion (3000+) - 4 cards
            CardId(3000), CardId(3000), // Shadow Initiate (2/2 Lifesteal)
            CardId(3003), CardId(3003), // Nightblade (3/2 Quick)
            // Neutral (4000+) - 2 cards
            CardId(4001), CardId(4001), // Berserker (3/2 Charge)
        ];

        Self {
            card_db,
            config,
            default_deck,
            eval_count: 0,
        }
    }

    /// Evaluate a candidate weight vector.
    pub fn evaluate(&mut self, weights_vec: &[f64]) -> FitnessResult {
        let start = Instant::now();

        // Convert f64 weights to f32 for GreedyWeights
        let weights_f32: Vec<f32> = weights_vec.iter().map(|&x| x as f32).collect();
        let greedy_weights = match GreedyWeights::from_vec(&weights_f32) {
            Some(w) => w,
            None => {
                return FitnessResult {
                    fitness: f64::NEG_INFINITY,
                    win_rate: 0.0,
                    games: 0,
                    avg_turns: 0.0,
                    eval_time: start.elapsed(),
                };
            }
        };

        let result = match &self.config.mode {
            TuningMode::VsRandom => {
                if self.config.parallel {
                    self.evaluate_vs_random_parallel(&greedy_weights)
                } else {
                    self.evaluate_vs_random(&greedy_weights)
                }
            }
            TuningMode::VsGreedy => {
                if self.config.parallel {
                    self.evaluate_vs_greedy_parallel(&greedy_weights)
                } else {
                    self.evaluate_vs_greedy(&greedy_weights)
                }
            }
            TuningMode::MultiOpponent => {
                self.evaluate_multi_opponent(&greedy_weights)
            }
            TuningMode::Generalist { matchups } => {
                self.evaluate_generalist(&greedy_weights, matchups)
            }
            TuningMode::Specialist { deck, opponent_deck } => {
                self.evaluate_specialist(&greedy_weights, deck, opponent_deck)
            }
        };

        self.eval_count += 1;

        FitnessResult {
            fitness: result.0,
            win_rate: result.1,
            games: result.2,
            avg_turns: result.3,
            eval_time: start.elapsed(),
        }
    }

    /// Evaluate against RandomBot baseline.
    fn evaluate_vs_random(&self, weights: &GreedyWeights) -> (f64, f64, usize, f64) {
        let mut wins = 0;
        let mut total_turns = 0u32;
        let games = self.config.games_per_eval;

        for i in 0..games {
            let seed = self.config.seed.wrapping_add(self.eval_count * 10000 + i as u64);
            let (winner, turns) = self.run_game_vs_random(weights, seed);

            if winner == Some(PlayerId::PLAYER_ONE) {
                wins += 1;
            }
            total_turns += turns;
        }

        let win_rate = wins as f64 / games as f64;
        let avg_turns = total_turns as f64 / games as f64;

        // Fitness: primarily win rate, with small bonus for faster wins
        let fitness = win_rate * 100.0 - avg_turns * 0.01;

        (fitness, win_rate, games, avg_turns)
    }

    /// Evaluate against default GreedyBot baseline.
    fn evaluate_vs_greedy(&self, weights: &GreedyWeights) -> (f64, f64, usize, f64) {
        let mut wins = 0;
        let mut total_turns = 0u32;
        let games = self.config.games_per_eval;

        for i in 0..games {
            let seed = self.config.seed.wrapping_add(self.eval_count * 10000 + i as u64);
            let (winner, turns) = self.run_game_vs_greedy(weights, seed);

            if winner == Some(PlayerId::PLAYER_ONE) {
                wins += 1;
            }
            total_turns += turns;
        }

        let win_rate = wins as f64 / games as f64;
        let avg_turns = total_turns as f64 / games as f64;

        // Fitness: win rate against Greedy is harder, so just use win rate
        let fitness = win_rate * 100.0;

        (fitness, win_rate, games, avg_turns)
    }

    /// Evaluate against RandomBot baseline (parallel version).
    fn evaluate_vs_random_parallel(&self, weights: &GreedyWeights) -> (f64, f64, usize, f64) {
        let games = self.config.games_per_eval;
        let base_seed = self.config.seed.wrapping_add(self.eval_count * 10000);
        let max_actions = self.config.max_actions;
        let default_deck = &self.default_deck;
        let card_db = self.card_db;

        let results: Vec<(bool, u32)> = (0..games)
            .into_par_iter()
            .map(|i| {
                let seed = base_seed.wrapping_add(i as u64);
                Self::run_game_vs_random_static(card_db, weights, default_deck, seed, max_actions)
            })
            .collect();

        let wins: usize = results.iter().filter(|(won, _)| *won).count();
        let total_turns: u32 = results.iter().map(|(_, turns)| *turns).sum();

        let win_rate = wins as f64 / games as f64;
        let avg_turns = total_turns as f64 / games as f64;
        let fitness = win_rate * 100.0 - avg_turns * 0.01;

        (fitness, win_rate, games, avg_turns)
    }

    /// Evaluate against GreedyBot baseline (parallel version).
    fn evaluate_vs_greedy_parallel(&self, weights: &GreedyWeights) -> (f64, f64, usize, f64) {
        let games = self.config.games_per_eval;
        let base_seed = self.config.seed.wrapping_add(self.eval_count * 10000);
        let max_actions = self.config.max_actions;
        let default_deck = &self.default_deck;
        let card_db = self.card_db;

        let results: Vec<(bool, u32)> = (0..games)
            .into_par_iter()
            .map(|i| {
                let seed = base_seed.wrapping_add(i as u64);
                Self::run_game_vs_greedy_static(card_db, weights, default_deck, seed, max_actions)
            })
            .collect();

        let wins: usize = results.iter().filter(|(won, _)| *won).count();
        let total_turns: u32 = results.iter().map(|(_, turns)| *turns).sum();

        let win_rate = wins as f64 / games as f64;
        let avg_turns = total_turns as f64 / games as f64;
        let fitness = win_rate * 100.0;

        (fitness, win_rate, games, avg_turns)
    }

    /// Evaluate against multiple opponents (Random, Greedy, MCTS).
    fn evaluate_multi_opponent(&self, weights: &GreedyWeights) -> (f64, f64, usize, f64) {
        let games_per_opponent = self.config.games_per_eval / 3;
        let base_seed = self.config.seed.wrapping_add(self.eval_count * 10000);
        let max_actions = self.config.max_actions;
        let default_deck = &self.default_deck;
        let card_db = self.card_db;
        let mcts_sims = self.config.mcts_sims;

        // Create parallel iterators for each opponent type
        // 0 = Random, 1 = Greedy, 2 = MCTS
        
        let random_iter = (0..games_per_opponent).into_par_iter().map(|i| {
            let seed = base_seed.wrapping_add(i as u64);
            let (won, turns) = Self::run_game_vs_random_static(card_db, weights, default_deck, seed, max_actions);
            (0, won, turns)
        });

        let greedy_iter = (0..games_per_opponent).into_par_iter().map(|i| {
            let seed = base_seed.wrapping_add(10000 + i as u64);
            let (won, turns) = Self::run_game_vs_greedy_static(card_db, weights, default_deck, seed, max_actions);
            (1, won, turns)
        });

        let mcts_iter = (0..games_per_opponent).into_par_iter().map(|i| {
            let seed = base_seed.wrapping_add(20000 + i as u64);
            let (won, turns) = Self::run_game_vs_mcts_static(card_db, weights, default_deck, seed, max_actions, mcts_sims);
            (2, won, turns)
        });

        // Run all games in a single parallel batch
        let results: Vec<(u8, bool, u32)> = random_iter
            .chain(greedy_iter)
            .chain(mcts_iter)
            .collect();

        // Compute stats
        let mut random_wins = 0;
        let mut greedy_wins = 0;
        let mut mcts_wins = 0;
        let mut total_turns = 0;

        for (opponent_type, won, turns) in results {
            if won {
                match opponent_type {
                    0 => random_wins += 1,
                    1 => greedy_wins += 1,
                    2 => mcts_wins += 1,
                    _ => unreachable!(),
                }
            }
            total_turns += turns;
        }

        let total_wins = random_wins + greedy_wins + mcts_wins;
        let total_games = games_per_opponent * 3;

        let win_rate = total_wins as f64 / total_games as f64;
        let avg_turns = total_turns as f64 / total_games as f64;

        // Weighted fitness: MCTS wins count more (harder opponent)
        let random_wr = random_wins as f64 / games_per_opponent as f64;
        let greedy_wr = greedy_wins as f64 / games_per_opponent as f64;
        let mcts_wr = mcts_wins as f64 / games_per_opponent as f64;

        // Fitness = weighted average: Random 10%, Greedy 40%, MCTS 50%
        let fitness = random_wr * 10.0 + greedy_wr * 40.0 + mcts_wr * 50.0;

        (fitness, win_rate, total_games, avg_turns)
    }

    /// Evaluate across multiple matchups (generalist).
    fn evaluate_generalist(&self, weights: &GreedyWeights, matchups: &[(Vec<CardId>, Vec<CardId>)]) -> (f64, f64, usize, f64) {
        // Enhanced generalist: test each deck matchup against all 3 opponent types
        // Distributes games: 1/3 each for Random, Greedy, MCTS per matchup
        let games_per_matchup = (self.config.games_per_eval / matchups.len()).max(3);
        let games_per_opponent = games_per_matchup / 3;

        let mut random_wins = 0;
        let mut greedy_wins = 0;
        let mut mcts_wins = 0;
        let mut total_games = 0;
        let mut total_turns = 0u32;

        if self.config.parallel {
            // Parallel evaluation for speed
            let card_db = self.card_db;
            let max_actions = self.config.max_actions;
            let base_seed = self.config.seed.wrapping_add(self.eval_count * 100000);
            let mcts_sims = self.config.mcts_sims;

            // Process all matchups in parallel
            let results: Vec<_> = matchups.par_iter().enumerate().map(|(matchup_idx, (deck1, deck2))| {
                let matchup_seed = base_seed.wrapping_add((matchup_idx * 10000) as u64);
                
                // vs Random
                let random_results: Vec<(bool, u32)> = (0..games_per_opponent).into_par_iter().map(|i| {
                    let seed = matchup_seed.wrapping_add(i as u64);
                    Self::run_game_vs_random_with_decks_static(card_db, weights, deck1, deck2, seed, max_actions)
                }).collect();

                // vs Greedy
                let greedy_results: Vec<(bool, u32)> = (0..games_per_opponent).into_par_iter().map(|i| {
                    let seed = matchup_seed.wrapping_add(1000 + i as u64);
                    Self::run_game_vs_greedy_with_decks_static(card_db, weights, deck1, deck2, seed, max_actions)
                }).collect();

                // vs MCTS
                let mcts_results: Vec<(bool, u32)> = (0..games_per_opponent).into_par_iter().map(|i| {
                    let seed = matchup_seed.wrapping_add(2000 + i as u64);
                    Self::run_game_vs_mcts_with_decks_static(card_db, weights, deck1, deck2, seed, max_actions, mcts_sims)
                }).collect();

                (random_results, greedy_results, mcts_results)
            }).collect();

            // Aggregate results
            for (random_results, greedy_results, mcts_results) in results {
                random_wins += random_results.iter().filter(|(won, _)| *won).count();
                greedy_wins += greedy_results.iter().filter(|(won, _)| *won).count();
                mcts_wins += mcts_results.iter().filter(|(won, _)| *won).count();
                
                total_turns += random_results.iter().map(|(_, t)| t).sum::<u32>();
                total_turns += greedy_results.iter().map(|(_, t)| t).sum::<u32>();
                total_turns += mcts_results.iter().map(|(_, t)| t).sum::<u32>();
                
                total_games += random_results.len() + greedy_results.len() + mcts_results.len();
            }
        } else {
            // Sequential evaluation (fallback)
            for (matchup_idx, (deck1, deck2)) in matchups.iter().enumerate() {
                let matchup_seed = self.config.seed
                    .wrapping_add(self.eval_count * 100000)
                    .wrapping_add((matchup_idx * 10000) as u64);

                // vs Random
                for i in 0..games_per_opponent {
                    let seed = matchup_seed.wrapping_add(i as u64);
                    let (winner, turns) = self.run_game_vs_random_with_decks(weights, deck1, deck2, seed);
                    if winner == Some(PlayerId::PLAYER_ONE) { random_wins += 1; }
                    total_turns += turns;
                    total_games += 1;
                }

                // vs Greedy
                for i in 0..games_per_opponent {
                    let seed = matchup_seed.wrapping_add(1000 + i as u64);
                    let (winner, turns) = self.run_game_vs_greedy_with_decks(weights, deck1, deck2, seed);
                    if winner == Some(PlayerId::PLAYER_ONE) { greedy_wins += 1; }
                    total_turns += turns;
                    total_games += 1;
                }

                // vs MCTS
                for i in 0..games_per_opponent {
                    let seed = matchup_seed.wrapping_add(2000 + i as u64);
                    let (winner, turns) = self.run_game_vs_mcts_with_decks(weights, deck1, deck2, seed);
                    if winner == Some(PlayerId::PLAYER_ONE) { mcts_wins += 1; }
                    total_turns += turns;
                    total_games += 1;
                }
            }
        }

        let games_per_opponent_type = (total_games / 3).max(1);
        let random_wr = random_wins as f64 / games_per_opponent_type as f64;
        let greedy_wr = greedy_wins as f64 / games_per_opponent_type as f64;
        let mcts_wr = mcts_wins as f64 / games_per_opponent_type as f64;

        // Weighted fitness like multi-opponent: Random 10%, Greedy 40%, MCTS 50%
        let fitness = random_wr * 10.0 + greedy_wr * 40.0 + mcts_wr * 50.0;

        let total_wins = random_wins + greedy_wins + mcts_wins;
        let win_rate = total_wins as f64 / total_games as f64;
        let avg_turns = total_turns as f64 / total_games as f64;

        (fitness, win_rate, total_games, avg_turns)
    }

    /// Evaluate for a specific matchup (specialist).
    fn evaluate_specialist(&self, weights: &GreedyWeights, deck: &[CardId], opponent_deck: &[CardId]) -> (f64, f64, usize, f64) {
        // Enhanced specialist: test against Random, Greedy, AND MCTS (like generalist)
        // Distributes games: 1/3 each for Random, Greedy, MCTS
        let games_per_opponent = self.config.games_per_eval / 3;

        let mut random_wins = 0;
        let mut greedy_wins = 0;
        let mut mcts_wins = 0;
        let mut total_turns = 0u32;

        if self.config.parallel {
            let card_db = self.card_db;
            let max_actions = self.config.max_actions;
            let base_seed = self.config.seed.wrapping_add(self.eval_count * 10000);
            let mcts_sims = self.config.mcts_sims;

            // vs Random
            let random_results: Vec<(bool, u32)> = (0..games_per_opponent).into_par_iter().map(|i| {
                let seed = base_seed.wrapping_add(i as u64);
                Self::run_game_vs_random_with_decks_static(card_db, weights, deck, opponent_deck, seed, max_actions)
            }).collect();

            // vs Greedy
            let greedy_results: Vec<(bool, u32)> = (0..games_per_opponent).into_par_iter().map(|i| {
                let seed = base_seed.wrapping_add(1000 + i as u64);
                Self::run_game_vs_greedy_with_decks_static(card_db, weights, deck, opponent_deck, seed, max_actions)
            }).collect();

            // vs MCTS
            let mcts_results: Vec<(bool, u32)> = (0..games_per_opponent).into_par_iter().map(|i| {
                let seed = base_seed.wrapping_add(2000 + i as u64);
                Self::run_game_vs_mcts_with_decks_static(card_db, weights, deck, opponent_deck, seed, max_actions, mcts_sims)
            }).collect();

            // Aggregate results
            random_wins = random_results.iter().filter(|(won, _)| *won).count();
            greedy_wins = greedy_results.iter().filter(|(won, _)| *won).count();
            mcts_wins = mcts_results.iter().filter(|(won, _)| *won).count();
            
            total_turns += random_results.iter().map(|(_, t)| t).sum::<u32>();
            total_turns += greedy_results.iter().map(|(_, t)| t).sum::<u32>();
            total_turns += mcts_results.iter().map(|(_, t)| t).sum::<u32>();
        } else {
            // Sequential fallback
            let base_seed = self.config.seed.wrapping_add(self.eval_count * 10000);
            
            // vs Random
            for i in 0..games_per_opponent {
                let seed = base_seed.wrapping_add(i as u64);
                let (winner, turns) = self.run_game_vs_random_with_decks(weights, deck, opponent_deck, seed);
                if winner == Some(PlayerId::PLAYER_ONE) { random_wins += 1; }
                total_turns += turns;
            }

            // vs Greedy
            for i in 0..games_per_opponent {
                let seed = base_seed.wrapping_add(1000 + i as u64);
                let (winner, turns) = self.run_game_vs_greedy_with_decks(weights, deck, opponent_deck, seed);
                if winner == Some(PlayerId::PLAYER_ONE) { greedy_wins += 1; }
                total_turns += turns;
            }

            // vs MCTS
            for i in 0..games_per_opponent {
                let seed = base_seed.wrapping_add(2000 + i as u64);
                let (winner, turns) = self.run_game_vs_mcts_with_decks(weights, deck, opponent_deck, seed);
                if winner == Some(PlayerId::PLAYER_ONE) { mcts_wins += 1; }
                total_turns += turns;
            }
        }

        let total_wins = random_wins + greedy_wins + mcts_wins;
        let total_games = games_per_opponent * 3;
        let win_rate = total_wins as f64 / total_games as f64;
        let avg_turns = total_turns as f64 / total_games as f64;
        let fitness = win_rate * 100.0 - avg_turns * 0.01;

        (fitness, win_rate, total_games, avg_turns)
    }

    /// Run a single game against RandomBot.
    fn run_game_vs_random(&self, weights: &GreedyWeights, seed: u64) -> (Option<PlayerId>, u32) {
        let mut greedy_bot = GreedyBot::with_weights(self.card_db, weights.clone(), seed);
        let mut random_bot = RandomBot::new(seed.wrapping_add(1000));

        let mut engine = GameEngine::new(self.card_db);
        engine.start_game(self.default_deck.clone(), self.default_deck.clone(), seed);

        let mut action_count = 0;
        while !engine.is_game_over() && action_count < self.config.max_actions {
            let current_player = engine.current_player();

            let action = if current_player == PlayerId::PLAYER_ONE {
                greedy_bot.select_action_with_engine(&engine)
            } else {
                let state_tensor = engine.get_state_tensor();
                let legal_mask = engine.get_legal_action_mask();
                let legal_actions = engine.get_legal_actions();
                random_bot.select_action(&state_tensor, &legal_mask, &legal_actions)
            };

            if engine.apply_action(action).is_err() {
                break;
            }
            action_count += 1;
        }

        (engine.winner(), engine.turn_number() as u32)
    }

    /// Run a single game against default GreedyBot.
    fn run_game_vs_greedy(&self, weights: &GreedyWeights, seed: u64) -> (Option<PlayerId>, u32) {
        let mut candidate_bot = GreedyBot::with_weights(self.card_db, weights.clone(), seed);
        let mut baseline_bot = GreedyBot::new(self.card_db, seed.wrapping_add(1000));

        let mut engine = GameEngine::new(self.card_db);
        engine.start_game(self.default_deck.clone(), self.default_deck.clone(), seed);

        let mut action_count = 0;
        while !engine.is_game_over() && action_count < self.config.max_actions {
            let current_player = engine.current_player();

            let action = if current_player == PlayerId::PLAYER_ONE {
                candidate_bot.select_action_with_engine(&engine)
            } else {
                baseline_bot.select_action_with_engine(&engine)
            };

            if engine.apply_action(action).is_err() {
                break;
            }
            action_count += 1;
        }

        (engine.winner(), engine.turn_number() as u32)
    }

    /// Get the number of evaluations performed.
    pub fn eval_count(&self) -> u64 {
        self.eval_count
    }

    // Static helpers for parallel game execution (no &self needed)

    /// Run a single game vs Random (static version for parallel).
    fn run_game_vs_random_static(
        card_db: &CardDatabase,
        weights: &GreedyWeights,
        deck: &[CardId],
        seed: u64,
        max_actions: usize,
    ) -> (bool, u32) {
        let mut greedy_bot = GreedyBot::with_weights(card_db, weights.clone(), seed);
        let mut random_bot = RandomBot::new(seed.wrapping_add(1000));

        let mut engine = GameEngine::new(card_db);
        engine.start_game(deck.to_vec(), deck.to_vec(), seed);

        let mut action_count = 0;
        while !engine.is_game_over() && action_count < max_actions {
            let current_player = engine.current_player();

            let action = if current_player == PlayerId::PLAYER_ONE {
                greedy_bot.select_action_with_engine(&engine)
            } else {
                let state_tensor = engine.get_state_tensor();
                let legal_mask = engine.get_legal_action_mask();
                let legal_actions = engine.get_legal_actions();
                random_bot.select_action(&state_tensor, &legal_mask, &legal_actions)
            };

            if engine.apply_action(action).is_err() {
                break;
            }
            action_count += 1;
        }

        let won = engine.winner() == Some(PlayerId::PLAYER_ONE);
        (won, engine.turn_number() as u32)
    }

    /// Run a single game vs Greedy (static version for parallel).
    fn run_game_vs_greedy_static(
        card_db: &CardDatabase,
        weights: &GreedyWeights,
        deck: &[CardId],
        seed: u64,
        max_actions: usize,
    ) -> (bool, u32) {
        let mut candidate_bot = GreedyBot::with_weights(card_db, weights.clone(), seed);
        let mut baseline_bot = GreedyBot::new(card_db, seed.wrapping_add(1000));

        let mut engine = GameEngine::new(card_db);
        engine.start_game(deck.to_vec(), deck.to_vec(), seed);

        let mut action_count = 0;
        while !engine.is_game_over() && action_count < max_actions {
            let current_player = engine.current_player();

            let action = if current_player == PlayerId::PLAYER_ONE {
                candidate_bot.select_action_with_engine(&engine)
            } else {
                baseline_bot.select_action_with_engine(&engine)
            };

            if engine.apply_action(action).is_err() {
                break;
            }
            action_count += 1;
        }

        let won = engine.winner() == Some(PlayerId::PLAYER_ONE);
        (won, engine.turn_number() as u32)
    }

    /// Run a single game vs MCTS (static version for parallel).
    fn run_game_vs_mcts_static(
        card_db: &CardDatabase,
        weights: &GreedyWeights,
        deck: &[CardId],
        seed: u64,
        max_actions: usize,
        mcts_sims: u32,
    ) -> (bool, u32) {
        let mut candidate_bot = GreedyBot::with_weights(card_db, weights.clone(), seed);
        let mcts_config = MctsConfig {
            simulations: mcts_sims,
            exploration: 1.414,
            max_rollout_depth: 50,
            parallel_trees: 1,
            leaf_rollouts: 1,
        };
        let mut mcts_bot = MctsBot::with_config(card_db, mcts_config, seed.wrapping_add(1000));

        let mut engine = GameEngine::new(card_db);
        engine.start_game(deck.to_vec(), deck.to_vec(), seed);

        let mut action_count = 0;
        while !engine.is_game_over() && action_count < max_actions {
            let current_player = engine.current_player();

            let action = if current_player == PlayerId::PLAYER_ONE {
                candidate_bot.select_action_with_engine(&engine)
            } else {
                mcts_bot.select_action_with_engine(&engine)
            };

            if engine.apply_action(action).is_err() {
                break;
            }
            action_count += 1;
        }

        let won = engine.winner() == Some(PlayerId::PLAYER_ONE);
        (won, engine.turn_number() as u32)
    }

    // ============================================================================
    // Helper methods for generalist mode with custom decks
    // ============================================================================

    /// Run game vs Random with custom decks (non-static version).
    fn run_game_vs_random_with_decks(&self, weights: &GreedyWeights, deck1: &[CardId], deck2: &[CardId], seed: u64) -> (Option<PlayerId>, u32) {
        let mut greedy_bot = GreedyBot::with_weights(self.card_db, weights.clone(), seed);
        let mut random_bot = RandomBot::new(seed.wrapping_add(1000));

        let mut engine = GameEngine::new(self.card_db);
        engine.start_game(deck1.to_vec(), deck2.to_vec(), seed);

        let mut action_count = 0;
        while !engine.is_game_over() && action_count < self.config.max_actions {
            let current_player = engine.current_player();

            let action = if current_player == PlayerId::PLAYER_ONE {
                greedy_bot.select_action_with_engine(&engine)
            } else {
                let state_tensor = engine.get_state_tensor();
                let legal_mask = engine.get_legal_action_mask();
                let legal_actions = engine.get_legal_actions();
                random_bot.select_action(&state_tensor, &legal_mask, &legal_actions)
            };

            if engine.apply_action(action).is_err() {
                break;
            }
            action_count += 1;
        }

        (engine.winner(), engine.turn_number() as u32)
    }

    /// Run game vs Greedy with custom decks (non-static version).
    fn run_game_vs_greedy_with_decks(&self, weights: &GreedyWeights, deck1: &[CardId], deck2: &[CardId], seed: u64) -> (Option<PlayerId>, u32) {
        let mut candidate_bot = GreedyBot::with_weights(self.card_db, weights.clone(), seed);
        let mut baseline_bot = GreedyBot::new(self.card_db, seed.wrapping_add(1000));

        let mut engine = GameEngine::new(self.card_db);
        engine.start_game(deck1.to_vec(), deck2.to_vec(), seed);

        let mut action_count = 0;
        while !engine.is_game_over() && action_count < self.config.max_actions {
            let current_player = engine.current_player();

            let action = if current_player == PlayerId::PLAYER_ONE {
                candidate_bot.select_action_with_engine(&engine)
            } else {
                baseline_bot.select_action_with_engine(&engine)
            };

            if engine.apply_action(action).is_err() {
                break;
            }
            action_count += 1;
        }

        (engine.winner(), engine.turn_number() as u32)
    }

    /// Run game vs MCTS with custom decks (non-static version).
    fn run_game_vs_mcts_with_decks(&self, weights: &GreedyWeights, deck1: &[CardId], deck2: &[CardId], seed: u64) -> (Option<PlayerId>, u32) {
        let mut candidate_bot = GreedyBot::with_weights(self.card_db, weights.clone(), seed);
        let mcts_config = MctsConfig {
            simulations: self.config.mcts_sims,
            exploration: 1.414,
            max_rollout_depth: 50,
            parallel_trees: 1,
            leaf_rollouts: 1,
        };
        let mut mcts_bot = MctsBot::with_config(self.card_db, mcts_config, seed.wrapping_add(1000));

        let mut engine = GameEngine::new(self.card_db);
        engine.start_game(deck1.to_vec(), deck2.to_vec(), seed);

        let mut action_count = 0;
        while !engine.is_game_over() && action_count < self.config.max_actions {
            let current_player = engine.current_player();

            let action = if current_player == PlayerId::PLAYER_ONE {
                candidate_bot.select_action_with_engine(&engine)
            } else {
                mcts_bot.select_action_with_engine(&engine)
            };

            if engine.apply_action(action).is_err() {
                break;
            }
            action_count += 1;
        }

        (engine.winner(), engine.turn_number() as u32)
    }

    /// Run a single game with specific decks against default GreedyBot (static version for parallel).
    /// Run game vs Random with custom decks (static version for parallel).
    fn run_game_vs_random_with_decks_static(
        card_db: &CardDatabase,
        weights: &GreedyWeights,
        deck1: &[CardId],
        deck2: &[CardId],
        seed: u64,
        max_actions: usize,
    ) -> (bool, u32) {
        let mut greedy_bot = GreedyBot::with_weights(card_db, weights.clone(), seed);
        let mut random_bot = RandomBot::new(seed.wrapping_add(1000));

        let mut engine = GameEngine::new(card_db);
        engine.start_game(deck1.to_vec(), deck2.to_vec(), seed);

        let mut action_count = 0;
        while !engine.is_game_over() && action_count < max_actions {
            let current_player = engine.current_player();

            let action = if current_player == PlayerId::PLAYER_ONE {
                greedy_bot.select_action_with_engine(&engine)
            } else {
                let state_tensor = engine.get_state_tensor();
                let legal_mask = engine.get_legal_action_mask();
                let legal_actions = engine.get_legal_actions();
                random_bot.select_action(&state_tensor, &legal_mask, &legal_actions)
            };

            if engine.apply_action(action).is_err() {
                break;
            }
            action_count += 1;
        }

        let won = engine.winner() == Some(PlayerId::PLAYER_ONE);
        (won, engine.turn_number() as u32)
    }

    /// Run game vs Greedy with custom decks (static version for parallel).
    fn run_game_vs_greedy_with_decks_static(
        card_db: &CardDatabase,
        weights: &GreedyWeights,
        deck1: &[CardId],
        deck2: &[CardId],
        seed: u64,
        max_actions: usize,
    ) -> (bool, u32) {
        let mut candidate_bot = GreedyBot::with_weights(card_db, weights.clone(), seed);
        let mut baseline_bot = GreedyBot::new(card_db, seed.wrapping_add(1000));

        let mut engine = GameEngine::new(card_db);
        engine.start_game(deck1.to_vec(), deck2.to_vec(), seed);

        let mut action_count = 0;
        while !engine.is_game_over() && action_count < max_actions {
            let current_player = engine.current_player();

            let action = if current_player == PlayerId::PLAYER_ONE {
                candidate_bot.select_action_with_engine(&engine)
            } else {
                baseline_bot.select_action_with_engine(&engine)
            };

            if engine.apply_action(action).is_err() {
                break;
            }
            action_count += 1;
        }

        let won = engine.winner() == Some(PlayerId::PLAYER_ONE);
        (won, engine.turn_number() as u32)
    }

    /// Run game vs MCTS with custom decks (static version for parallel).
    fn run_game_vs_mcts_with_decks_static(
        card_db: &CardDatabase,
        weights: &GreedyWeights,
        deck1: &[CardId],
        deck2: &[CardId],
        seed: u64,
        max_actions: usize,
        mcts_sims: u32,
    ) -> (bool, u32) {
        let mut candidate_bot = GreedyBot::with_weights(card_db, weights.clone(), seed);
        let mcts_config = MctsConfig {
            simulations: mcts_sims,
            exploration: 1.414,
            max_rollout_depth: 50,
            parallel_trees: 1,
            leaf_rollouts: 1,
        };
        let mut mcts_bot = MctsBot::with_config(card_db, mcts_config, seed.wrapping_add(1000));

        let mut engine = GameEngine::new(card_db);
        engine.start_game(deck1.to_vec(), deck2.to_vec(), seed);

        let mut action_count = 0;
        while !engine.is_game_over() && action_count < max_actions {
            let current_player = engine.current_player();

            let action = if current_player == PlayerId::PLAYER_ONE {
                candidate_bot.select_action_with_engine(&engine)
            } else {
                mcts_bot.select_action_with_engine(&engine)
            };

            if engine.apply_action(action).is_err() {
                break;
            }
            action_count += 1;
        }

        let won = engine.winner() == Some(PlayerId::PLAYER_ONE);
        (won, engine.turn_number() as u32)
    }
}
