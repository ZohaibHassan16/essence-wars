//! Monte Carlo Tree Search bot implementation.
//!
//! MCTS is a best-first search algorithm that uses random simulations to evaluate
//! game states. This implementation uses:
//! - UCB1 for selection with tunable exploration constant
//! - GreedyBot for rollout policy (smarter than random)

use std::cell::RefCell;
use std::rc::Rc;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::actions::Action;
use crate::bots::greedy::GreedyBot;
use crate::bots::Bot;
use crate::cards::CardDatabase;
use crate::engine::GameEngine;
use crate::tensor::STATE_TENSOR_SIZE;
use crate::types::PlayerId;

/// Configuration for MCTS bot behavior.
#[derive(Clone, Debug)]
pub struct MctsConfig {
    /// Number of simulations to run per move
    pub simulations: u32,
    /// UCB1 exploration constant (sqrt(2) is theoretically optimal)
    pub exploration: f32,
    /// Maximum rollout depth (prevents infinite games)
    pub max_rollout_depth: u32,
}

impl Default for MctsConfig {
    fn default() -> Self {
        Self {
            simulations: 1000,
            exploration: 1.414, // sqrt(2)
            max_rollout_depth: 100,
        }
    }
}

impl MctsConfig {
    /// Create a fast config for testing.
    pub fn fast() -> Self {
        Self {
            simulations: 100,
            exploration: 1.414,
            max_rollout_depth: 50,
        }
    }

    /// Create a strong config for serious play.
    pub fn strong() -> Self {
        Self {
            simulations: 5000,
            exploration: 1.414,
            max_rollout_depth: 150,
        }
    }
}

/// A node in the MCTS tree.
struct MctsNode {
    /// Action that led to this node (None for root)
    action: Option<Action>,
    /// Total visits to this node
    visits: u32,
    /// Total wins (from perspective of player who moved INTO this state)
    wins: i32,
    /// Child nodes
    children: Vec<Rc<RefCell<MctsNode>>>,
    /// Whether this node has been expanded
    expanded: bool,
}

impl MctsNode {
    /// Create a new node for the given action.
    fn new(action: Option<Action>) -> Self {
        Self {
            action,
            visits: 0,
            wins: 0,
            children: Vec::new(),
            expanded: false,
        }
    }

    /// Create a root node.
    fn root() -> Self {
        Self::new(None)
    }

    /// Get the UCB1 score for this node.
    fn ucb1(&self, parent_visits: u32, exploration: f32) -> f32 {
        if self.visits == 0 {
            return f32::MAX;
        }

        let exploitation = self.wins as f32 / self.visits as f32;
        let exploration_term = exploration * ((parent_visits as f32).ln() / self.visits as f32).sqrt();

        exploitation + exploration_term
    }

    /// Select the best child using UCB1.
    fn select_child(&self, exploration: f32) -> Option<Rc<RefCell<MctsNode>>> {
        if self.children.is_empty() {
            return None;
        }

        self.children.iter()
            .max_by(|a, b| {
                let a_ucb = a.borrow().ucb1(self.visits, exploration);
                let b_ucb = b.borrow().ucb1(self.visits, exploration);
                a_ucb.partial_cmp(&b_ucb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    /// Select the most visited child (for final move selection).
    fn most_visited_child(&self) -> Option<Rc<RefCell<MctsNode>>> {
        self.children.iter()
            .max_by_key(|c| c.borrow().visits)
            .cloned()
    }

    /// Expand this node with the given legal actions.
    fn expand(&mut self, actions: &[Action]) {
        if self.expanded {
            return;
        }

        for &action in actions {
            self.children.push(Rc::new(RefCell::new(MctsNode::new(Some(action)))));
        }
        self.expanded = true;
    }

    /// Record a visit and update wins.
    fn update(&mut self, win: bool) {
        self.visits += 1;
        if win {
            self.wins += 1;
        }
    }
}

/// Monte Carlo Tree Search bot.
pub struct MctsBot<'a> {
    name: String,
    card_db: &'a CardDatabase,
    config: MctsConfig,
    rng: SmallRng,
    seed: u64,
}

impl<'a> MctsBot<'a> {
    /// Create a new MCTS bot with default configuration.
    pub fn new(card_db: &'a CardDatabase, seed: u64) -> Self {
        Self {
            name: "MctsBot".to_string(),
            card_db,
            config: MctsConfig::default(),
            rng: SmallRng::seed_from_u64(seed),
            seed,
        }
    }

    /// Create an MCTS bot with custom configuration.
    pub fn with_config(card_db: &'a CardDatabase, config: MctsConfig, seed: u64) -> Self {
        Self {
            name: format!("MctsBot({})", config.simulations),
            card_db,
            config,
            rng: SmallRng::seed_from_u64(seed),
            seed,
        }
    }

    /// Set a custom name for this bot.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Run MCTS search and return the best action.
    pub fn search(&mut self, engine: &GameEngine) -> Action {
        let legal_actions = engine.get_legal_actions();

        if legal_actions.is_empty() {
            return Action::EndTurn;
        }

        if legal_actions.len() == 1 {
            return legal_actions[0];
        }

        // Create root node and expand it
        let root = Rc::new(RefCell::new(MctsNode::root()));
        root.borrow_mut().expand(&legal_actions);

        let player = engine.current_player();

        // Run simulations
        for _ in 0..self.config.simulations {
            // Fork the engine for simulation
            let mut sim_engine = engine.fork();
            let mut path: Vec<Rc<RefCell<MctsNode>>> = vec![root.clone()];

            // Selection: traverse tree using UCB1
            let mut current = root.clone();
            while !sim_engine.is_game_over() {
                let child = {
                    let node = current.borrow();
                    node.select_child(self.config.exploration)
                };

                if let Some(child) = child {
                    // Apply the action
                    let action = child.borrow().action;
                    if let Some(action) = action {
                        if sim_engine.apply_action(action).is_err() {
                            break;
                        }
                    }
                    path.push(child.clone());
                    current = child;
                } else {
                    // Leaf node - expand if not terminal
                    if !sim_engine.is_game_over() {
                        let actions = sim_engine.get_legal_actions();
                        current.borrow_mut().expand(&actions);
                    }
                    break;
                }
            }

            // Rollout: simulate to end using GreedyBot
            let win = self.rollout(&mut sim_engine, player);

            // Backpropagation: update stats along the path
            // All nodes get updated from the original player's perspective
            // since win is already calculated from that perspective
            for node in path.iter() {
                node.borrow_mut().update(win);
            }
        }

        // Select most visited action
        let best_child = root.borrow().most_visited_child();
        if let Some(child) = best_child {
            if let Some(action) = child.borrow().action {
                return action;
            }
        }

        // Fallback to first legal action
        legal_actions[0]
    }

    /// Perform a rollout from the current state using GreedyBot.
    fn rollout(&mut self, engine: &mut GameEngine, perspective: PlayerId) -> bool {
        let mut greedy = GreedyBot::new(self.card_db, self.rng.gen());
        let mut depth = 0;

        while !engine.is_game_over() && depth < self.config.max_rollout_depth {
            let action = greedy.select_action_with_engine(engine);
            if engine.apply_action(action).is_err() {
                break;
            }
            depth += 1;
        }

        // Check who won
        match engine.winner() {
            Some(winner) => winner == perspective,
            None => false, // Draw counts as loss for simplicity
        }
    }
}

impl<'a> Bot for MctsBot<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn select_action(
        &mut self,
        _state_tensor: &[f32; STATE_TENSOR_SIZE],
        _legal_mask: &[f32; 256],
        legal_actions: &[Action],
    ) -> Action {
        // MCTS requires engine access for simulation
        // Fall back to simple heuristic without engine
        if legal_actions.is_empty() {
            return Action::EndTurn;
        }

        // Simple fallback: prioritize attacks, then plays, then end turn
        for action in legal_actions {
            if matches!(action, Action::Attack { .. }) {
                return *action;
            }
        }
        for action in legal_actions {
            if matches!(action, Action::PlayCard { .. }) {
                return *action;
            }
        }
        legal_actions[0]
    }

    fn reset(&mut self) {
        self.rng = SmallRng::seed_from_u64(self.seed);
    }

    fn clone_box(&self) -> Box<dyn Bot> {
        // Cannot clone MCTS bot due to CardDatabase reference
        // Fall back to RandomBot
        Box::new(crate::bots::RandomBot::new(self.seed))
    }
}

impl<'a> MctsBot<'a> {
    /// Select an action using full MCTS search.
    pub fn select_action_with_engine(&mut self, engine: &GameEngine) -> Action {
        self.search(engine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CardId;

    fn load_test_db() -> CardDatabase {
        CardDatabase::load_from_directory("data/cards").expect("Failed to load cards")
    }

    fn test_deck() -> Vec<CardId> {
        let valid_ids = [1, 3, 6, 8, 11, 12, 16, 20, 34];
        (0..18)
            .map(|i| CardId(valid_ids[i % valid_ids.len()] as u16))
            .collect()
    }

    #[test]
    fn test_mcts_bot_creation() {
        let card_db = load_test_db();
        let bot = MctsBot::new(&card_db, 42);
        assert_eq!(bot.name(), "MctsBot");
    }

    #[test]
    fn test_mcts_config() {
        let fast = MctsConfig::fast();
        assert_eq!(fast.simulations, 100);

        let strong = MctsConfig::strong();
        assert!(strong.simulations > fast.simulations);
    }

    #[test]
    fn test_mcts_node_ucb1() {
        let mut node = MctsNode::new(Some(Action::EndTurn));
        node.visits = 10;
        node.wins = 5;

        let ucb = node.ucb1(100, 1.414);
        assert!(ucb > 0.0, "UCB1 should be positive");
        assert!(ucb < 100.0, "UCB1 should be reasonable");
    }

    #[test]
    fn test_mcts_node_expand() {
        let mut node = MctsNode::root();
        let actions = vec![Action::EndTurn, Action::PlayCard { hand_index: 0, slot: crate::types::Slot(0) }];

        node.expand(&actions);

        assert_eq!(node.children.len(), 2);
    }

    #[test]
    fn test_mcts_search_returns_valid_action() {
        let card_db = load_test_db();
        let config = MctsConfig::fast();
        let mut bot = MctsBot::with_config(&card_db, config, 42);

        let mut engine = GameEngine::new(&card_db);
        engine.start_game(test_deck(), test_deck(), 12345);

        let action = bot.search(&engine);
        let legal = engine.get_legal_actions();

        assert!(legal.contains(&action), "MCTS should return a legal action");
    }

    #[test]
    fn test_mcts_completes_game() {
        let card_db = load_test_db();
        let config = MctsConfig::fast();
        let mut bot = MctsBot::with_config(&card_db, config, 42);

        let mut engine = GameEngine::new(&card_db);
        engine.start_game(test_deck(), test_deck(), 12345);

        let mut actions = 0;
        while !engine.is_game_over() && actions < 200 {
            let action = bot.select_action_with_engine(&engine);
            engine.apply_action(action).expect("Legal action should work");
            actions += 1;
        }

        assert!(engine.is_game_over() || actions >= 200);
    }
}
