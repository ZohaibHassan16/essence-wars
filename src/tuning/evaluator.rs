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
        // Default deck for simple evaluations
        let default_deck = vec![
            CardId(1), CardId(1),   // Eager Recruit x2
            CardId(3), CardId(3),   // Nimble Scout x2
            CardId(6), CardId(6),   // Frontier Ranger x2
            CardId(8), CardId(8),   // Shielded Squire x2
            CardId(11), CardId(11), // Centaur Charger x2
            CardId(12), CardId(12), // Blade Dancer x2
            CardId(16), CardId(16), // Piercing Striker x2
            CardId(20), CardId(20), // Siege Breaker x2
            CardId(34), CardId(34), // Lightning Bolt x2
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

        // Run games against all three opponents in parallel
        let vs_random: Vec<(bool, u32)> = (0..games_per_opponent)
            .into_par_iter()
            .map(|i| {
                let seed = base_seed.wrapping_add(i as u64);
                Self::run_game_vs_random_static(card_db, weights, default_deck, seed, max_actions)
            })
            .collect();

        let vs_greedy: Vec<(bool, u32)> = (0..games_per_opponent)
            .into_par_iter()
            .map(|i| {
                let seed = base_seed.wrapping_add(10000 + i as u64);
                Self::run_game_vs_greedy_static(card_db, weights, default_deck, seed, max_actions)
            })
            .collect();

        let vs_mcts: Vec<(bool, u32)> = (0..games_per_opponent)
            .into_par_iter()
            .map(|i| {
                let seed = base_seed.wrapping_add(20000 + i as u64);
                Self::run_game_vs_mcts_static(card_db, weights, default_deck, seed, max_actions, mcts_sims)
            })
            .collect();

        // Compute stats
        let random_wins: usize = vs_random.iter().filter(|(won, _)| *won).count();
        let greedy_wins: usize = vs_greedy.iter().filter(|(won, _)| *won).count();
        let mcts_wins: usize = vs_mcts.iter().filter(|(won, _)| *won).count();

        let total_wins = random_wins + greedy_wins + mcts_wins;
        let total_games = games_per_opponent * 3;
        let total_turns: u32 = vs_random.iter().chain(vs_greedy.iter()).chain(vs_mcts.iter())
            .map(|(_, turns)| *turns).sum();

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
        let mut total_wins = 0;
        let mut total_games = 0;
        let mut total_turns = 0u32;

        let games_per_matchup = (self.config.games_per_eval / matchups.len()).max(1);

        for (matchup_idx, (deck1, deck2)) in matchups.iter().enumerate() {
            for i in 0..games_per_matchup {
                let seed = self.config.seed
                    .wrapping_add(self.eval_count * 10000)
                    .wrapping_add((matchup_idx * 1000 + i) as u64);

                let (winner, turns) = self.run_game_with_decks(weights, deck1, deck2, seed);

                if winner == Some(PlayerId::PLAYER_ONE) {
                    total_wins += 1;
                }
                total_turns += turns;
                total_games += 1;
            }
        }

        let win_rate = total_wins as f64 / total_games as f64;
        let avg_turns = total_turns as f64 / total_games as f64;
        let fitness = win_rate * 100.0 - avg_turns * 0.01;

        (fitness, win_rate, total_games, avg_turns)
    }

    /// Evaluate for a specific matchup (specialist).
    fn evaluate_specialist(&self, weights: &GreedyWeights, deck: &[CardId], opponent_deck: &[CardId]) -> (f64, f64, usize, f64) {
        let mut wins = 0;
        let mut total_turns = 0u32;
        let games = self.config.games_per_eval;

        for i in 0..games {
            let seed = self.config.seed.wrapping_add(self.eval_count * 10000 + i as u64);
            let (winner, turns) = self.run_game_with_decks(weights, deck, opponent_deck, seed);

            if winner == Some(PlayerId::PLAYER_ONE) {
                wins += 1;
            }
            total_turns += turns;
        }

        let win_rate = wins as f64 / games as f64;
        let avg_turns = total_turns as f64 / games as f64;
        let fitness = win_rate * 100.0 - avg_turns * 0.01;

        (fitness, win_rate, games, avg_turns)
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

    /// Run a single game with specific decks against default GreedyBot.
    fn run_game_with_decks(&self, weights: &GreedyWeights, deck1: &[CardId], deck2: &[CardId], seed: u64) -> (Option<PlayerId>, u32) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_vs_random() {
        let card_db = CardDatabase::load_from_directory("data/cards")
            .expect("Failed to load cards");

        let config = EvaluatorConfig {
            games_per_eval: 10,
            mode: TuningMode::VsRandom,
            seed: 42,
            max_actions: 500,
            parallel: false, // Sequential for test stability
            mcts_sims: 100,
        };

        let mut evaluator = Evaluator::new(&card_db, config);

        // Use default weights
        let weights = GreedyWeights::default();
        let result = evaluator.evaluate(&weights.to_vec().iter().map(|&x| x as f64).collect::<Vec<_>>());

        // Should win most games against random
        assert!(result.win_rate > 0.5, "Should beat random, got {:.2}%", result.win_rate * 100.0);
        assert!(result.games == 10);
    }

    #[test]
    fn test_evaluate_vs_greedy() {
        let card_db = CardDatabase::load_from_directory("data/cards")
            .expect("Failed to load cards");

        let config = EvaluatorConfig {
            games_per_eval: 10,
            mode: TuningMode::VsGreedy,
            seed: 42,
            max_actions: 500,
            parallel: false,
            mcts_sims: 100,
        };

        let mut evaluator = Evaluator::new(&card_db, config);

        // Default vs default should be close to 50%
        let weights = GreedyWeights::default();
        let result = evaluator.evaluate(&weights.to_vec().iter().map(|&x| x as f64).collect::<Vec<_>>());

        // Mirror match should be around 50%
        assert!(result.win_rate >= 0.2 && result.win_rate <= 0.8,
                "Mirror match should be close to 50%, got {:.2}%", result.win_rate * 100.0);
    }

    #[test]
    fn test_bad_weights_lose() {
        let card_db = CardDatabase::load_from_directory("data/cards")
            .expect("Failed to load cards");

        let config = EvaluatorConfig {
            games_per_eval: 10,
            mode: TuningMode::VsGreedy,
            seed: 42,
            max_actions: 500,
            parallel: false,
            mcts_sims: 100,
        };

        let mut evaluator = Evaluator::new(&card_db, config);

        // Terrible weights - negative for good things, positive for bad
        let bad_weights = GreedyWeights {
            own_life: -1.0,           // Penalize own life
            enemy_life_damage: -1.0,  // Penalize damaging enemy
            own_creature_attack: -1.0,
            own_creature_health: -1.0,
            enemy_creature_attack: 1.0,
            enemy_creature_health: 1.0,
            creature_count: -1.0,
            board_advantage: -1.0,
            cards_in_hand: -1.0,
            action_points: 0.0,
            keyword_guard: -1.0,
            keyword_lethal: -1.0,
            keyword_lifesteal: -1.0,
            keyword_rush: -1.0,
            keyword_ranged: -1.0,
            keyword_piercing: -1.0,
            keyword_shield: -1.0,
            keyword_quick: -1.0,
            win_bonus: -1000.0,       // Penalize winning!
            lose_penalty: 1000.0,     // Reward losing!
        };

        let result = evaluator.evaluate(&bad_weights.to_vec().iter().map(|&x| x as f64).collect::<Vec<_>>());

        // Should lose most games with these terrible weights
        assert!(result.win_rate < 0.5, "Bad weights should lose, got {:.2}%", result.win_rate * 100.0);
    }
}
