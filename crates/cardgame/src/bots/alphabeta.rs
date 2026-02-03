//! Alpha-Beta Pruning bot implementation.
//!
//! Alpha-Beta is a deterministic minimax search algorithm with pruning that finds
//! the optimal move within a given depth. Unlike MCTS which samples randomly,
//! Alpha-Beta guarantees finding forced wins/losses within the search depth.
//!
//! This implementation uses:
//! - Iterative deepening for time-bounded search
//! - GreedyBot's evaluation function at leaf nodes
//! - Move ordering with killer moves and history heuristic
//! - Transposition table for caching evaluated positions
//! - Aspiration windows for faster iterative deepening
//! - Late Move Reduction (LMR) for deeper effective search

use std::collections::HashMap;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::actions::Action;
use crate::bots::transposition::{zobrist_hash, Bound, TranspositionTable};
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
    /// Enable transposition table
    pub use_tt: bool,
    /// Transposition table size in MB
    pub tt_size_mb: usize,
    /// Enable aspiration windows
    pub aspiration_windows: bool,
    /// Aspiration window initial size
    pub aspiration_delta: f32,
    /// Enable Late Move Reduction
    pub use_lmr: bool,
    /// LMR: minimum depth to apply reduction
    pub lmr_min_depth: u32,
    /// LMR: minimum move index to apply reduction
    pub lmr_min_move: usize,
}

impl Default for AlphaBetaConfig {
    fn default() -> Self {
        Self {
            max_depth: 6,
            move_ordering: true,
            iterative_deepening: true,
            quiescence: false,
            quiescence_depth: 2,
            use_tt: true,
            tt_size_mb: 16,
            aspiration_windows: true,
            aspiration_delta: 50.0,
            use_lmr: true,
            lmr_min_depth: 3,
            lmr_min_move: 4,
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
            use_tt: true,
            tt_size_mb: 8,
            aspiration_windows: false,
            aspiration_delta: 50.0,
            use_lmr: false,
            lmr_min_depth: 3,
            lmr_min_move: 4,
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
            use_tt: true,
            tt_size_mb: 32,
            aspiration_windows: true,
            aspiration_delta: 25.0,
            use_lmr: true,
            lmr_min_depth: 2,
            lmr_min_move: 3,
        }
    }

    /// Create a config with a specific depth.
    pub fn with_depth(depth: u32) -> Self {
        Self {
            max_depth: depth,
            ..Default::default()
        }
    }

    /// Create a legacy config without new optimizations (for comparison).
    pub fn legacy(depth: u32) -> Self {
        Self {
            max_depth: depth,
            move_ordering: true,
            iterative_deepening: true,
            quiescence: false,
            quiescence_depth: 0,
            use_tt: false,
            tt_size_mb: 0,
            aspiration_windows: false,
            aspiration_delta: 50.0,
            use_lmr: false,
            lmr_min_depth: 3,
            lmr_min_move: 4,
        }
    }
}

/// Maximum depth for killer moves array.
const MAX_KILLER_DEPTH: usize = 32;

/// Number of killer moves to store per depth.
const KILLERS_PER_DEPTH: usize = 2;

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
    /// Number of transposition table hits
    pub tt_hits: u64,
    /// Number of TT cutoffs (search skipped due to TT)
    pub tt_cutoffs: u64,
    /// Number of LMR re-searches
    pub lmr_researches: u64,
    /// Number of aspiration window re-searches
    pub aspiration_researches: u64,
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
    /// Transposition table for caching positions
    tt: Option<TranspositionTable>,
    /// Killer moves: moves that caused beta cutoffs at each depth
    killer_moves: [[Option<Action>; KILLERS_PER_DEPTH]; MAX_KILLER_DEPTH],
    /// History heuristic: score for each action type that caused cutoffs
    history: HashMap<ActionKey, u32>,
    /// Previous iteration's best move (for PV reordering)
    pv_move: Option<Action>,
}

/// Key for history heuristic table.
/// We use a simplified key based on action type and some identifying info.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ActionKey {
    /// Discriminant for action type
    action_type: u8,
    /// Additional identifier (slot, card index, etc.)
    identifier: u16,
}

impl ActionKey {
    fn from_action(action: Action) -> Self {
        match action {
            Action::PlayCard { hand_index, slot } => Self {
                action_type: 0,
                identifier: ((hand_index as u16) << 8) | (slot.0 as u16),
            },
            Action::Attack { attacker, defender } => Self {
                action_type: 1,
                identifier: ((attacker.0 as u16) << 8) | (defender.0 as u16),
            },
            Action::UseAbility { slot, ability_index, target } => Self {
                action_type: 2,
                identifier: ((slot.0 as u16) << 12) | ((ability_index as u16) << 8) | (target.to_index() as u16),
            },
            Action::CommanderInsight => Self {
                action_type: 3,
                identifier: 0,
            },
            Action::EndTurn => Self {
                action_type: 4,
                identifier: 0,
            },
        }
    }
}

impl<'a> AlphaBetaBot<'a> {
    /// Create a new Alpha-Beta bot with default configuration.
    pub fn new(card_db: &'a CardDatabase, seed: u64) -> Self {
        let config = AlphaBetaConfig::default();
        let tt = if config.use_tt {
            Some(TranspositionTable::with_memory_mb(config.tt_size_mb))
        } else {
            None
        };
        let weights = Self::load_default_weights();
        Self {
            name: "AlphaBetaBot".to_string(),
            card_db,
            config,
            weights,
            rng: SmallRng::seed_from_u64(seed),
            seed,
            last_stats: SearchStats::default(),
            tt,
            killer_moves: [[None; KILLERS_PER_DEPTH]; MAX_KILLER_DEPTH],
            history: HashMap::new(),
            pv_move: None,
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
            log::debug!("AlphaBetaBot: loaded weights from {:?}", alphabeta_path);
            return bot_weights.default.greedy.clone();
        }

        // Fallback to shared generalist weights (works well for Alpha-Beta too)
        let generalist_path = crate::data_dir().join("weights/generalist.toml");
        if let Ok(bot_weights) = BotWeights::load(&generalist_path) {
            log::debug!("AlphaBetaBot: loaded weights from {:?}", generalist_path);
            return bot_weights.default.greedy.clone();
        }

        // Final fallback: hardcoded defaults
        log::debug!("AlphaBetaBot: using hardcoded default weights");
        GreedyWeights::default()
    }

    #[cfg(target_arch = "wasm32")]
    fn load_default_weights() -> GreedyWeights {
        GreedyWeights::default()
    }

    /// Create an Alpha-Beta bot with custom configuration.
    pub fn with_config(card_db: &'a CardDatabase, config: AlphaBetaConfig, seed: u64) -> Self {
        let tt = if config.use_tt {
            Some(TranspositionTable::with_memory_mb(config.tt_size_mb))
        } else {
            None
        };
        Self {
            name: format!("AlphaBetaBot(d{})", config.max_depth),
            card_db,
            config,
            weights: Self::load_default_weights(),
            rng: SmallRng::seed_from_u64(seed),
            seed,
            last_stats: SearchStats::default(),
            tt,
            killer_moves: [[None; KILLERS_PER_DEPTH]; MAX_KILLER_DEPTH],
            history: HashMap::new(),
            pv_move: None,
        }
    }

    /// Create an Alpha-Beta bot with custom weights.
    pub fn with_weights(card_db: &'a CardDatabase, weights: GreedyWeights, seed: u64) -> Self {
        let config = AlphaBetaConfig::default();
        let tt = if config.use_tt {
            Some(TranspositionTable::with_memory_mb(config.tt_size_mb))
        } else {
            None
        };
        Self {
            name: "AlphaBetaBot".to_string(),
            card_db,
            config,
            weights,
            rng: SmallRng::seed_from_u64(seed),
            seed,
            last_stats: SearchStats::default(),
            tt,
            killer_moves: [[None; KILLERS_PER_DEPTH]; MAX_KILLER_DEPTH],
            history: HashMap::new(),
            pv_move: None,
        }
    }

    /// Create an Alpha-Beta bot with custom config and weights.
    pub fn with_config_and_weights(
        card_db: &'a CardDatabase,
        config: AlphaBetaConfig,
        weights: &BotWeights,
        seed: u64,
    ) -> Self {
        let tt = if config.use_tt {
            Some(TranspositionTable::with_memory_mb(config.tt_size_mb))
        } else {
            None
        };
        Self {
            name: format!("AlphaBetaBot(d{}, {})", config.max_depth, weights.name),
            card_db,
            config,
            weights: weights.default.greedy.clone(),
            rng: SmallRng::seed_from_u64(seed),
            seed,
            last_stats: SearchStats::default(),
            tt,
            killer_moves: [[None; KILLERS_PER_DEPTH]; MAX_KILLER_DEPTH],
            history: HashMap::new(),
            pv_move: None,
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
    /// Ordering priority: PV move > Killer moves > History heuristic > Quick evaluation
    fn order_moves(&self, engine: &GameEngine, actions: &mut [Action], depth: usize) {
        if !self.config.move_ordering || actions.len() <= 1 {
            return;
        }

        let killers = if depth < MAX_KILLER_DEPTH {
            &self.killer_moves[depth]
        } else {
            &[None; KILLERS_PER_DEPTH]
        };

        // Sort by combined priority (descending - best moves first)
        actions.sort_by(|a, b| {
            let score_a = self.move_order_score(*a, killers, engine);
            let score_b = self.move_order_score(*b, killers, engine);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Compute move ordering score for a single action.
    fn move_order_score(&self, action: Action, killers: &[Option<Action>; KILLERS_PER_DEPTH], engine: &GameEngine) -> f32 {
        // PV move gets highest priority
        if Some(action) == self.pv_move {
            return 1_000_000.0;
        }

        // Killer moves get second highest priority
        for (i, killer) in killers.iter().enumerate() {
            if *killer == Some(action) {
                return 900_000.0 - (i as f32 * 1000.0);
            }
        }

        // History heuristic
        let history_score = self.history.get(&ActionKey::from_action(action)).copied().unwrap_or(0) as f32;

        // Combine with quick evaluation
        let quick_score = self.quick_evaluate_action(engine, action);

        history_score * 10.0 + quick_score
    }

    /// Store a killer move at the given depth.
    fn store_killer(&mut self, action: Action, depth: usize) {
        if depth >= MAX_KILLER_DEPTH {
            return;
        }

        // Don't store captures or end turn as killers (they're already well-ordered)
        match action {
            Action::EndTurn => return,
            _ => {}
        }

        let killers = &mut self.killer_moves[depth];

        // Don't duplicate
        if killers[0] == Some(action) {
            return;
        }

        // Shift and insert
        killers[1] = killers[0];
        killers[0] = Some(action);
    }

    /// Update history score for a move that caused a cutoff.
    fn update_history(&mut self, action: Action, depth: u32) {
        let key = ActionKey::from_action(action);
        let bonus = depth * depth; // Deeper cutoffs are more valuable
        let entry = self.history.entry(key).or_insert(0);
        *entry = entry.saturating_add(bonus);

        // Aging: prevent overflow by periodically dividing all scores
        if *entry > 100_000 {
            for value in self.history.values_mut() {
                *value /= 2;
            }
        }
    }

    /// Clear search-specific state for a new search.
    fn clear_search_state(&mut self) {
        self.killer_moves = [[None; KILLERS_PER_DEPTH]; MAX_KILLER_DEPTH];
        self.pv_move = None;
        // Note: history is NOT cleared - it persists across searches within a game

        // Signal new search to transposition table
        if let Some(ref mut tt) = self.tt {
            tt.new_search();
        }
    }

    /// Core alpha-beta search algorithm (negamax variant).
    ///
    /// Returns the evaluation score from the perspective of the player to move.
    fn alphabeta(
        &mut self,
        engine: &GameEngine,
        depth: u32,
        mut alpha: f32,
        mut beta: f32,
        root_player: PlayerId,
    ) -> f32 {
        self.last_stats.nodes_visited += 1;
        let original_alpha = alpha;

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

        // Transposition table probe
        let hash = if self.config.use_tt {
            let h = zobrist_hash(&engine.state);
            if let Some(ref mut tt) = self.tt {
                if let Some(entry) = tt.probe(h) {
                    self.last_stats.tt_hits += 1;
                    // Only use TT entry if depth is sufficient
                    if entry.depth >= depth {
                        let tt_score = entry.score;
                        let usable = match entry.bound {
                            Bound::Exact => true,
                            Bound::Lower => tt_score >= beta,
                            Bound::Upper => tt_score <= alpha,
                        };
                        if usable {
                            self.last_stats.tt_cutoffs += 1;
                            return tt_score;
                        }
                    }
                }
            }
            Some(h)
        } else {
            None
        };

        let mut actions = engine.get_legal_actions();
        if actions.is_empty() {
            return self.evaluate(&engine.state, root_player);
        }

        // Move ordering for better pruning
        self.order_moves(engine, &mut actions, depth as usize);

        let mut best_move: Option<Action> = None;

        if is_maximizing {
            let mut max_eval = f32::NEG_INFINITY;

            for (move_idx, action) in actions.iter().enumerate() {
                let mut child_engine = engine.fork();
                if child_engine.apply_action(*action).is_err() {
                    continue;
                }

                // Late Move Reduction
                let eval = if self.config.use_lmr
                    && depth >= self.config.lmr_min_depth
                    && move_idx >= self.config.lmr_min_move
                    && !matches!(action, Action::Attack { .. }) // Don't reduce tactical moves
                {
                    // Search with reduced depth first
                    let reduced_depth = depth.saturating_sub(1 + (move_idx / 6) as u32).max(1);
                    let score = self.alphabeta(&child_engine, reduced_depth - 1, alpha, beta, root_player);

                    // Re-search at full depth if the reduced search fails high
                    if score > alpha {
                        self.last_stats.lmr_researches += 1;
                        self.alphabeta(&child_engine, depth - 1, alpha, beta, root_player)
                    } else {
                        score
                    }
                } else {
                    self.alphabeta(&child_engine, depth - 1, alpha, beta, root_player)
                };

                if eval > max_eval {
                    max_eval = eval;
                    best_move = Some(*action);
                }
                if eval > alpha {
                    alpha = eval;
                }

                if beta <= alpha {
                    self.last_stats.beta_cutoffs += 1;
                    // Store killer and update history on cutoff
                    self.store_killer(*action, depth as usize);
                    self.update_history(*action, depth);
                    break; // Beta cutoff
                }
            }

            // Store in transposition table
            if let (Some(h), Some(ref mut tt)) = (hash, &mut self.tt) {
                let bound = if max_eval <= original_alpha {
                    Bound::Upper
                } else if max_eval >= beta {
                    Bound::Lower
                } else {
                    Bound::Exact
                };
                tt.store(h, depth, max_eval, bound, best_move);
            }

            max_eval
        } else {
            let mut min_eval = f32::INFINITY;

            for (move_idx, action) in actions.iter().enumerate() {
                let mut child_engine = engine.fork();
                if child_engine.apply_action(*action).is_err() {
                    continue;
                }

                // Late Move Reduction
                let eval = if self.config.use_lmr
                    && depth >= self.config.lmr_min_depth
                    && move_idx >= self.config.lmr_min_move
                    && !matches!(action, Action::Attack { .. })
                {
                    let reduced_depth = depth.saturating_sub(1 + (move_idx / 6) as u32).max(1);
                    let score = self.alphabeta(&child_engine, reduced_depth - 1, alpha, beta, root_player);

                    if score < beta {
                        self.last_stats.lmr_researches += 1;
                        self.alphabeta(&child_engine, depth - 1, alpha, beta, root_player)
                    } else {
                        score
                    }
                } else {
                    self.alphabeta(&child_engine, depth - 1, alpha, beta, root_player)
                };

                if eval < min_eval {
                    min_eval = eval;
                    best_move = Some(*action);
                }
                if eval < beta {
                    beta = eval;
                }

                if beta <= alpha {
                    self.last_stats.alpha_cutoffs += 1;
                    self.store_killer(*action, depth as usize);
                    self.update_history(*action, depth);
                    break; // Alpha cutoff
                }
            }

            // Store in transposition table
            if let (Some(h), Some(ref mut tt)) = (hash, &mut self.tt) {
                let bound = if min_eval >= beta {
                    Bound::Lower // From opponent's perspective, this is a lower bound
                } else if min_eval <= alpha {
                    Bound::Upper
                } else {
                    Bound::Exact
                };
                tt.store(h, depth, min_eval, bound, best_move);
            }

            min_eval
        }
    }

    /// Search for the best action using alpha-beta pruning.
    pub fn search(&mut self, engine: &GameEngine) -> Action {
        // Reset stats and clear per-search state
        self.last_stats = SearchStats::default();
        self.clear_search_state();

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

    /// Search with iterative deepening and aspiration windows.
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
            self.order_moves(engine, &mut actions, depth as usize);

            // Set PV move from previous iteration for better ordering
            self.pv_move = Some(best_action);

            // Aspiration windows: start with narrow window around previous score
            let (mut alpha, mut beta) = if self.config.aspiration_windows && depth > 1 && best_score.is_finite() {
                let delta = self.config.aspiration_delta;
                (best_score - delta, best_score + delta)
            } else {
                (f32::NEG_INFINITY, f32::INFINITY)
            };

            let mut depth_best_action = actions[0];
            let mut depth_best_score;

            // Aspiration window loop: widen if score falls outside window
            loop {
                depth_best_score = f32::NEG_INFINITY;

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

                // Check if we need to re-search with wider window
                if self.config.aspiration_windows && depth > 1 {
                    if depth_best_score <= alpha {
                        // Fail low: widen alpha
                        self.last_stats.aspiration_researches += 1;
                        alpha = f32::NEG_INFINITY;
                        continue;
                    } else if depth_best_score >= beta {
                        // Fail high: widen beta
                        self.last_stats.aspiration_researches += 1;
                        beta = f32::INFINITY;
                        continue;
                    }
                }

                // Search completed within window
                break;
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
        self.order_moves(engine, &mut actions, depth as usize);

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
        self.killer_moves = [[None; KILLERS_PER_DEPTH]; MAX_KILLER_DEPTH];
        self.history.clear();
        self.pv_move = None;
        if let Some(ref mut tt) = self.tt {
            tt.clear();
        }
    }

    fn clone_box(&self) -> Box<dyn Bot> {
        // Cannot clone due to CardDatabase reference
        Box::new(crate::bots::RandomBot::new(self.seed))
    }
}
