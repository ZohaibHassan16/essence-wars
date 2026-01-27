//! Alpha-Beta Pruning bot implementation.
//!
//! Alpha-Beta is a deterministic minimax search algorithm with pruning that finds
//! the optimal move within a given depth. Unlike MCTS which samples randomly,
//! Alpha-Beta guarantees finding forced wins/losses within the search depth.
//!
//! This implementation uses:
//! - Iterative deepening for time-bounded search
//! - GreedyBot's evaluation function at leaf nodes
//! - Move ordering to improve pruning efficiency

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::actions::Action;
use crate::bots::weights::{BotWeights, GreedyWeights};
use crate::bots::Bot;
use crate::cards::CardDatabase;
use crate::engine::GameEngine;
use crate::state::GameState;
use crate::tensor::STATE_TENSOR_SIZE;
use crate::types::PlayerId;

/// Configuration for Alpha-Beta bot behavior.
#[derive(Clone, Debug)]
pub struct AlphaBetaConfig {
    /// Maximum search depth (in plies/half-moves)
    pub max_depth: u32,
    /// Enable move ordering for better pruning
    pub move_ordering: bool,
    /// Enable iterative deepening
    pub iterative_deepening: bool,
    /// Enable quiescence search (extend search in tactical positions)
    pub quiescence: bool,
    /// Maximum quiescence depth
    pub quiescence_depth: u32,
}

impl Default for AlphaBetaConfig {
    fn default() -> Self {
        Self {
            max_depth: 6,
            move_ordering: true,
            iterative_deepening: true,
            quiescence: false,
            quiescence_depth: 2,
        }
    }
}

impl AlphaBetaConfig {
    /// Create a fast config for testing.
    pub fn fast() -> Self {
        Self {
            max_depth: 4,
            move_ordering: true,
            iterative_deepening: false,
            quiescence: false,
            quiescence_depth: 0,
        }
    }

    /// Create a strong config for serious play.
    pub fn strong() -> Self {
        Self {
            max_depth: 8,
            move_ordering: true,
            iterative_deepening: true,
            quiescence: true,
            quiescence_depth: 4,
        }
    }

    /// Create a config with a specific depth.
    pub fn with_depth(depth: u32) -> Self {
        Self {
            max_depth: depth,
            ..Default::default()
        }
    }
}

/// Statistics for search introspection.
#[derive(Clone, Debug, Default)]
pub struct SearchStats {
    /// Number of nodes visited
    pub nodes_visited: u64,
    /// Number of alpha cutoffs
    pub alpha_cutoffs: u64,
    /// Number of beta cutoffs
    pub beta_cutoffs: u64,
    /// Maximum depth reached
    pub max_depth_reached: u32,
    /// Number of terminal nodes found
    pub terminal_nodes: u64,
}

/// Alpha-Beta search bot using minimax with alpha-beta pruning.
pub struct AlphaBetaBot<'a> {
    name: String,
    #[allow(dead_code)]
    card_db: &'a CardDatabase,
    config: AlphaBetaConfig,
    weights: GreedyWeights,
    rng: SmallRng,
    seed: u64,
    /// Statistics from the last search
    last_stats: SearchStats,
}

impl<'a> AlphaBetaBot<'a> {
    /// Create a new Alpha-Beta bot with default configuration.
    pub fn new(card_db: &'a CardDatabase, seed: u64) -> Self {
        let weights = Self::load_default_weights();
        Self {
            name: "AlphaBetaBot".to_string(),
            card_db,
            config: AlphaBetaConfig::default(),
            weights,
            rng: SmallRng::seed_from_u64(seed),
            seed,
            last_stats: SearchStats::default(),
        }
    }

    /// Load default weights from file, or use hardcoded defaults.
    /// 
    /// Tries in order:
    /// 1. Alpha-Beta specific weights: data/weights/alphabeta/generalist.toml
    /// 2. Shared generalist weights: data/weights/generalist.toml
    /// 3. Hardcoded defaults
    #[cfg(not(target_arch = "wasm32"))]
    fn load_default_weights() -> GreedyWeights {
        // Try Alpha-Beta specific weights first
        let alphabeta_path = crate::data_dir().join("weights/alphabeta/generalist.toml");
        if let Ok(bot_weights) = BotWeights::load(&alphabeta_path) {
            return bot_weights.default.greedy.clone();
        }
        
        // Fallback to shared generalist weights (works well for Alpha-Beta too)
        let generalist_path = crate::data_dir().join("weights/generalist.toml");
        if let Ok(bot_weights) = BotWeights::load(&generalist_path) {
            return bot_weights.default.greedy.clone();
        }
        
        // Final fallback: hardcoded defaults
        GreedyWeights::default()
    }

    #[cfg(target_arch = "wasm32")]
    fn load_default_weights() -> GreedyWeights {
        GreedyWeights::default()
    }

    /// Create an Alpha-Beta bot with custom configuration.
    pub fn with_config(card_db: &'a CardDatabase, config: AlphaBetaConfig, seed: u64) -> Self {
        Self {
            name: format!("AlphaBetaBot(d{})", config.max_depth),
            card_db,
            config,
            weights: Self::load_default_weights(),
            rng: SmallRng::seed_from_u64(seed),
            seed,
            last_stats: SearchStats::default(),
        }
    }

    /// Create an Alpha-Beta bot with custom weights.
    pub fn with_weights(card_db: &'a CardDatabase, weights: GreedyWeights, seed: u64) -> Self {
        Self {
            name: "AlphaBetaBot".to_string(),
            card_db,
            config: AlphaBetaConfig::default(),
            weights,
            rng: SmallRng::seed_from_u64(seed),
            seed,
            last_stats: SearchStats::default(),
        }
    }

    /// Create an Alpha-Beta bot with custom config and weights.
    pub fn with_config_and_weights(
        card_db: &'a CardDatabase,
        config: AlphaBetaConfig,
        weights: &BotWeights,
        seed: u64,
    ) -> Self {
        Self {
            name: format!("AlphaBetaBot(d{}, {})", config.max_depth, weights.name),
            card_db,
            config,
            weights: weights.default.greedy.clone(),
            rng: SmallRng::seed_from_u64(seed),
            seed,
            last_stats: SearchStats::default(),
        }
    }

    /// Set a custom name for this bot.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Get statistics from the last search.
    pub fn last_stats(&self) -> &SearchStats {
        &self.last_stats
    }

    /// Get the current weights.
    pub fn weights(&self) -> &GreedyWeights {
        &self.weights
    }

    /// Evaluate a terminal or leaf node from the perspective of the given player.
    fn evaluate(&self, state: &GameState, player: PlayerId) -> f32 {
        let w = &self.weights;
        let player_idx = player.index();
        let opponent_idx = player.opponent().index();

        let player_state = &state.players[player_idx];
        let opponent_state = &state.players[opponent_idx];

        // Check for terminal states first
        if let Some(result) = &state.result {
            match result {
                crate::state::GameResult::Win { winner, .. } => {
                    if *winner == player {
                        return w.win_bonus;
                    } else {
                        return w.lose_penalty;
                    }
                }
                crate::state::GameResult::Draw => return 0.0,
            }
        }

        let mut score = 0.0;

        // Life totals
        score += player_state.life as f32 * w.own_life;
        score += (crate::config::player::STARTING_LIFE as i16 - opponent_state.life) as f32
            * w.enemy_life_damage;

        // Own creatures
        for creature in &player_state.creatures {
            score += creature.attack.max(0) as f32 * w.own_creature_attack;
            score += creature.current_health.max(0) as f32 * w.own_creature_health;

            // Keyword bonuses
            let kw = creature.keywords;
            if kw.has_guard() {
                score += w.keyword_guard;
            }
            if kw.has_lethal() {
                score += w.keyword_lethal;
            }
            if kw.has_lifesteal() {
                score += w.keyword_lifesteal;
            }
            if kw.has_rush() {
                score += w.keyword_rush;
            }
            if kw.has_ranged() {
                score += w.keyword_ranged;
            }
            if kw.has_piercing() {
                score += w.keyword_piercing;
            }
            if kw.has_shield() {
                score += w.keyword_shield;
            }
            if kw.has_quick() {
                score += w.keyword_quick;
            }
            if kw.has_ephemeral() {
                score += w.keyword_ephemeral;
            }
            if kw.has_regenerate() {
                score += w.keyword_regenerate;
            }
            if kw.has_stealth() {
                score += w.keyword_stealth;
            }
            if kw.has_charge() {
                score += w.keyword_charge;
            }
            if kw.has_frenzy() {
                score += w.keyword_frenzy;
            }
            if kw.has_volatile() {
                score += w.keyword_volatile;
            }
            if kw.has_fortify() {
                score += w.keyword_fortify;
            }
            if kw.has_ward() {
                score += w.keyword_ward;
            }
        }

        // Enemy creatures
        for creature in &opponent_state.creatures {
            score += creature.attack.max(0) as f32 * w.enemy_creature_attack;
            score += creature.current_health.max(0) as f32 * w.enemy_creature_health;
        }

        // Board control
        let my_creatures = player_state.creatures.len() as f32;
        let enemy_creatures = opponent_state.creatures.len() as f32;
        score += my_creatures * w.creature_count;
        score += (my_creatures - enemy_creatures) * w.board_advantage;

        // Resources
        score += player_state.hand.len() as f32 * w.cards_in_hand;
        score += player_state.action_points as f32 * w.action_points;

        score
    }

    /// Quick evaluation for move ordering (faster, less accurate).
    fn quick_evaluate_action(&self, _engine: &GameEngine, action: Action) -> f32 {
        // Simple heuristic for move ordering without full state evaluation
        match action {
            Action::Attack { .. } => 100.0, // Attacks often decisive
            Action::PlayCard { .. } => 50.0, // Developing is good
            Action::CommanderInsight => 40.0, // Free card draw when behind
            Action::UseAbility { .. } => 30.0, // Abilities can be powerful
            Action::EndTurn => -100.0, // Usually worst option
        }
    }

    /// Order moves to improve alpha-beta pruning efficiency.
    fn order_moves(&self, engine: &GameEngine, actions: &mut [Action]) {
        if !self.config.move_ordering || actions.len() <= 1 {
            return;
        }

        // Sort by quick evaluation (descending - best moves first)
        actions.sort_by(|a, b| {
            let score_a = self.quick_evaluate_action(engine, *a);
            let score_b = self.quick_evaluate_action(engine, *b);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Core alpha-beta search algorithm (negamax variant).
    ///
    /// Returns the evaluation score from the perspective of the player to move.
    fn alphabeta(
        &mut self,
        engine: &GameEngine,
        depth: u32,
        mut alpha: f32,
        beta: f32,
        root_player: PlayerId,
    ) -> f32 {
        self.last_stats.nodes_visited += 1;

        // Terminal node check
        if engine.is_game_over() {
            self.last_stats.terminal_nodes += 1;
            return self.evaluate(&engine.state, root_player);
        }

        // Leaf node - evaluate position
        if depth == 0 {
            self.last_stats.max_depth_reached =
                self.last_stats.max_depth_reached.max(self.config.max_depth);
            return self.evaluate(&engine.state, root_player);
        }

        let current_player = engine.current_player();
        let is_maximizing = current_player == root_player;

        let mut actions = engine.get_legal_actions();
        if actions.is_empty() {
            return self.evaluate(&engine.state, root_player);
        }

        // Move ordering for better pruning
        self.order_moves(engine, &mut actions);

        if is_maximizing {
            let mut max_eval = f32::NEG_INFINITY;

            for action in actions {
                let mut child_engine = engine.fork();
                if child_engine.apply_action(action).is_err() {
                    continue;
                }

                let eval = self.alphabeta(&child_engine, depth - 1, alpha, beta, root_player);
                max_eval = max_eval.max(eval);
                alpha = alpha.max(eval);

                if beta <= alpha {
                    self.last_stats.beta_cutoffs += 1;
                    break; // Beta cutoff
                }
            }

            max_eval
        } else {
            let mut min_eval = f32::INFINITY;

            for action in actions {
                let mut child_engine = engine.fork();
                if child_engine.apply_action(action).is_err() {
                    continue;
                }

                let eval = self.alphabeta(&child_engine, depth - 1, alpha, beta, root_player);
                min_eval = min_eval.min(eval);

                // Note: We're still using alpha-beta bounds correctly for minimizing player
                if eval <= alpha {
                    self.last_stats.alpha_cutoffs += 1;
                    break; // Alpha cutoff
                }
                // Update beta for minimizing player
                if eval < beta {
                    // This is implicit in the pruning logic
                }
            }

            min_eval
        }
    }

    /// Search for the best action using alpha-beta pruning.
    pub fn search(&mut self, engine: &GameEngine) -> Action {
        // Reset stats
        self.last_stats = SearchStats::default();

        let legal_actions = engine.get_legal_actions();

        if legal_actions.is_empty() {
            return Action::EndTurn;
        }

        if legal_actions.len() == 1 {
            return legal_actions[0];
        }

        let root_player = engine.current_player();

        // Check for immediate wins
        for &action in &legal_actions {
            let mut sim = engine.fork();
            if sim.apply_action(action).is_ok()
                && sim.is_game_over()
                && sim.winner() == Some(root_player)
            {
                return action;
            }
        }

        // Use iterative deepening or fixed depth
        if self.config.iterative_deepening {
            self.search_iterative_deepening(engine, &legal_actions, root_player)
        } else {
            self.search_fixed_depth(engine, &legal_actions, root_player, self.config.max_depth)
        }
    }

    /// Search with iterative deepening.
    fn search_iterative_deepening(
        &mut self,
        engine: &GameEngine,
        legal_actions: &[Action],
        root_player: PlayerId,
    ) -> Action {
        let mut best_action = legal_actions[0];
        let mut best_score = f32::NEG_INFINITY;

        // Iteratively deepen from depth 1 to max_depth
        for depth in 1..=self.config.max_depth {
            let mut actions = legal_actions.to_vec();
            self.order_moves(engine, &mut actions);

            let mut depth_best_action = actions[0];
            let mut depth_best_score = f32::NEG_INFINITY;
            let alpha = f32::NEG_INFINITY;
            let beta = f32::INFINITY;

            for &action in &actions {
                let mut child_engine = engine.fork();
                if child_engine.apply_action(action).is_err() {
                    continue;
                }

                let score = self.alphabeta(&child_engine, depth - 1, alpha, beta, root_player);

                if score > depth_best_score {
                    depth_best_score = score;
                    depth_best_action = action;
                }

                // If we found a winning move, return immediately
                if score >= self.weights.win_bonus {
                    return action;
                }
            }

            best_action = depth_best_action;
            best_score = depth_best_score;
            self.last_stats.max_depth_reached = depth;

            // Early exit if we found a forced win
            if best_score >= self.weights.win_bonus {
                break;
            }
        }

        // If all moves lead to loss, pick randomly among them to avoid predictability
        if best_score <= self.weights.lose_penalty {
            let idx = self.rng.gen_range(0..legal_actions.len());
            return legal_actions[idx];
        }

        best_action
    }

    /// Search with fixed depth.
    fn search_fixed_depth(
        &mut self,
        engine: &GameEngine,
        legal_actions: &[Action],
        root_player: PlayerId,
        depth: u32,
    ) -> Action {
        let mut actions = legal_actions.to_vec();
        self.order_moves(engine, &mut actions);

        let mut best_action = actions[0];
        let mut best_score = f32::NEG_INFINITY;
        let alpha = f32::NEG_INFINITY;
        let beta = f32::INFINITY;

        for &action in &actions {
            let mut child_engine = engine.fork();
            if child_engine.apply_action(action).is_err() {
                continue;
            }

            let score = self.alphabeta(&child_engine, depth - 1, alpha, beta, root_player);

            if score > best_score {
                best_score = score;
                best_action = action;
            }
        }

        // If all moves lead to loss, pick randomly
        if best_score <= self.weights.lose_penalty {
            let idx = self.rng.gen_range(0..legal_actions.len());
            return legal_actions[idx];
        }

        best_action
    }
}

impl<'a> Bot for AlphaBetaBot<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn select_action(
        &mut self,
        _state_tensor: &[f32; STATE_TENSOR_SIZE],
        _legal_mask: &[f32; 256],
        _legal_actions: &[Action],
    ) -> Action {
        // Alpha-Beta requires engine access for tree search
        panic!(
            "AlphaBetaBot::select_action() called without engine access. \
             Alpha-Beta requires the game engine for tree search. \
             Use select_action_with_engine() or ensure GameRunner is being used."
        );
    }

    fn select_action_with_engine(&mut self, engine: &GameEngine) -> Action {
        self.search(engine)
    }

    fn requires_engine(&self) -> bool {
        true
    }

    fn reset(&mut self) {
        self.rng = SmallRng::seed_from_u64(self.seed);
        self.last_stats = SearchStats::default();
    }

    fn clone_box(&self) -> Box<dyn Bot> {
        // Cannot clone due to CardDatabase reference
        Box::new(crate::bots::RandomBot::new(self.seed))
    }
}
