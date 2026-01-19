//! Greedy bot implementation - selects the best action based on heuristic evaluation.
//!
//! GreedyBot simulates each legal action and evaluates the resulting game state
//! using a weighted sum of features. This provides a stronger baseline than
//! RandomBot while remaining fast enough for use in MCTS rollouts.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::actions::Action;
use crate::bots::weights::{BotWeights, GreedyWeights};
use crate::bots::Bot;
use crate::cards::CardDatabase;
use crate::config::player as player_config;
use crate::engine::GameEngine;
use crate::state::GameState;
use crate::tensor::STATE_TENSOR_SIZE;
use crate::types::PlayerId;

/// A bot that selects the action with the highest heuristic value.
///
/// For each legal action, GreedyBot simulates the action and evaluates the
/// resulting state. It then selects the action that leads to the best state
/// (from its perspective).
pub struct GreedyBot<'a> {
    name: String,
    #[allow(dead_code)] // Kept for future card lookup features
    card_db: &'a CardDatabase,
    weights: GreedyWeights,
    rng: SmallRng,
    seed: u64,
    #[allow(dead_code)] // Used for specialist bot functionality
    deck_id: Option<String>,
}

impl<'a> GreedyBot<'a> {
    /// Create a new GreedyBot with default weights.
    /// Tries to load from data/weights/default.toml, falls back to hardcoded defaults.
    pub fn new(card_db: &'a CardDatabase, seed: u64) -> Self {
        let weights = Self::load_default_weights();
        Self {
            name: "GreedyBot".to_string(),
            card_db,
            weights,
            rng: SmallRng::seed_from_u64(seed),
            seed,
            deck_id: None,
        }
    }

    /// Load default weights from file, or use hardcoded defaults.
    fn load_default_weights() -> GreedyWeights {
        let default_path = crate::data_dir().join("weights/default.toml");
        match BotWeights::load(&default_path) {
            Ok(bot_weights) => {
                // eprintln!("Loaded default weights from {:?}", default_path);
                bot_weights.default.greedy.clone()
            }
            Err(e) => {
                eprintln!("Failed to load weights from {:?}: {}", default_path, e);
                eprintln!("Using hardcoded default weights instead");
                GreedyWeights::default()
            }
        }
    }

    /// Create a GreedyBot with custom weights.
    pub fn with_weights(card_db: &'a CardDatabase, weights: GreedyWeights, seed: u64) -> Self {
        Self {
            name: "GreedyBot".to_string(),
            card_db,
            weights,
            rng: SmallRng::seed_from_u64(seed),
            seed,
            deck_id: None,
        }
    }

    /// Create a GreedyBot from BotWeights configuration.
    pub fn from_bot_weights(card_db: &'a CardDatabase, bot_weights: &BotWeights, deck_id: Option<&str>, seed: u64) -> Self {
        let weights = match deck_id {
            Some(id) => bot_weights.for_deck(id).greedy.clone(),
            None => bot_weights.default.greedy.clone(),
        };

        Self {
            name: format!("GreedyBot({})", bot_weights.name),
            card_db,
            weights,
            rng: SmallRng::seed_from_u64(seed),
            seed,
            deck_id: deck_id.map(|s| s.to_string()),
        }
    }

    /// Set a custom name for this bot.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Get the current weights (for inspection/logging).
    pub fn weights(&self) -> &GreedyWeights {
        &self.weights
    }

    /// Evaluate a game state from the perspective of the given player.
    ///
    /// Returns a score where higher is better for the specified player.
    pub fn evaluate_state(&self, state: &GameState, player: PlayerId) -> f32 {
        let w = &self.weights;
        let player_idx = player.index();
        let opponent_idx = player.opponent().index();

        let player_state = &state.players[player_idx];
        let opponent_state = &state.players[opponent_idx];

        let mut score = 0.0;

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

        // Life totals
        score += player_state.life as f32 * w.own_life;
        score += (player_config::STARTING_LIFE as i16 - opponent_state.life) as f32 * w.enemy_life_damage;

        // Own creatures
        for creature in &player_state.creatures {
            score += creature.attack.max(0) as f32 * w.own_creature_attack;
            score += creature.current_health.max(0) as f32 * w.own_creature_health;

            // Keyword bonuses (original 8)
            let kw = creature.keywords;
            if kw.has_guard() { score += w.keyword_guard; }
            if kw.has_lethal() { score += w.keyword_lethal; }
            if kw.has_lifesteal() { score += w.keyword_lifesteal; }
            if kw.has_rush() { score += w.keyword_rush; }
            if kw.has_ranged() { score += w.keyword_ranged; }
            if kw.has_piercing() { score += w.keyword_piercing; }
            if kw.has_shield() { score += w.keyword_shield; }
            if kw.has_quick() { score += w.keyword_quick; }
            // Keyword bonuses (new 4)
            if kw.has_ephemeral() { score += w.keyword_ephemeral; }
            if kw.has_regenerate() { score += w.keyword_regenerate; }
            if kw.has_stealth() { score += w.keyword_stealth; }
            if kw.has_charge() { score += w.keyword_charge; }
            // Keyword bonuses (Symbiote v0.5.0)
            if kw.has_frenzy() { score += w.keyword_frenzy; }
            if kw.has_volatile() { score += w.keyword_volatile; }
            // Keyword bonuses (Phase 5 v0.5.0)
            if kw.has_fortify() { score += w.keyword_fortify; }
            if kw.has_ward() { score += w.keyword_ward; }
        }

        // Enemy creatures (these weights are typically negative)
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

    /// Evaluate an action by simulating it and scoring the resulting state.
    ///
    /// Returns (action, score) where score is the evaluation of the state after
    /// the action is applied.
    pub fn evaluate_action(&self, engine: &GameEngine, action: Action) -> f32 {
        let player = engine.current_player();

        // Fork the engine to simulate the action
        let mut sim_engine = engine.fork();

        // Apply the action
        if sim_engine.apply_action(action).is_err() {
            // If action fails, return very low score
            return f32::MIN;
        }

        // Evaluate the resulting state from our perspective
        self.evaluate_state(&sim_engine.state, player)
    }

    /// Select the best action from a list of legal actions.
    ///
    /// This is the core selection logic - it evaluates each action and returns
    /// the one with the highest score. Ties are broken randomly.
    pub fn select_best_action(&mut self, engine: &GameEngine, legal_actions: &[Action]) -> Action {
        if legal_actions.is_empty() {
            // This shouldn't happen in a well-formed game, but default to EndTurn
            return Action::EndTurn;
        }

        if legal_actions.len() == 1 {
            return legal_actions[0];
        }

        // Evaluate all actions
        let mut best_score = f32::MIN;
        let mut best_actions: Vec<Action> = Vec::new();

        for &action in legal_actions {
            let score = self.evaluate_action(engine, action);

            if score > best_score {
                best_score = score;
                best_actions.clear();
                best_actions.push(action);
            } else if (score - best_score).abs() < 0.001 {
                // Tie - add to candidates
                best_actions.push(action);
            }
        }

        // Break ties randomly
        if best_actions.len() == 1 {
            best_actions[0]
        } else {
            let idx = self.rng.gen_range(0..best_actions.len());
            best_actions[idx]
        }
    }
}

impl<'a> Bot for GreedyBot<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn select_action(
        &mut self,
        _state_tensor: &[f32; STATE_TENSOR_SIZE],
        _legal_mask: &[f32; 256],
        legal_actions: &[Action],
    ) -> Action {
        // We need to reconstruct the engine from the state to simulate actions.
        // This is a limitation - GreedyBot needs access to the actual GameEngine.
        // For now, we'll use a workaround: the arena will need to pass us the engine.
        //
        // Since we can't access the engine from here, we'll need to use a different
        // approach. The bot interface is designed for neural networks that don't
        // need to simulate. For GreedyBot, we'll need to work around this.
        //
        // WORKAROUND: We store the engine state and reconstruct it.
        // This is inefficient but maintains the Bot trait interface.
        //
        // For better performance, use select_best_action() directly when you have
        // access to the GameEngine.

        // Since we can't reconstruct the full engine from just the tensor,
        // we fall back to a simple heuristic that doesn't require simulation.
        // This is a degraded mode - prefer using select_action_with_engine().
        self.select_action_fallback(legal_actions)
    }

    /// Override to use full simulation when engine is available.
    ///
    /// This is the preferred method - GameRunner and other callers should use this
    /// when they have access to the GameEngine, enabling full action simulation.
    fn select_action_with_engine(&mut self, engine: &GameEngine) -> Action {
        let legal_actions = engine.get_legal_actions();
        self.select_best_action(engine, &legal_actions)
    }

    fn reset(&mut self) {
        self.rng = SmallRng::seed_from_u64(self.seed);
    }

    fn clone_box(&self) -> Box<dyn Bot> {
        // We can't clone GreedyBot because it holds a reference to CardDatabase.
        // Instead, create a RandomBot as a fallback.
        // The proper way to use GreedyBot is through the arena which manages lifetimes.
        Box::new(crate::bots::RandomBot::new(self.seed))
    }
}

impl<'a> GreedyBot<'a> {
    /// Fallback action selection when we can't access the GameEngine.
    ///
    /// Uses simple heuristics based on action type rather than simulation.
    pub fn select_action_fallback(&mut self, legal_actions: &[Action]) -> Action {
        if legal_actions.is_empty() {
            return Action::EndTurn;
        }

        // Simple priority-based selection as fallback
        // This is much weaker than full simulation but works without engine access
        let mut best_action = legal_actions[0];
        let mut best_priority = action_priority(&best_action);

        for &action in legal_actions.iter().skip(1) {
            let priority = action_priority(&action);
            if priority > best_priority {
                best_priority = priority;
                best_action = action;
            } else if priority == best_priority && self.rng.gen_bool(0.5) {
                best_action = action;
            }
        }

        best_action
    }
}

/// Simple priority heuristic for actions (used in fallback mode).
fn action_priority(action: &Action) -> i32 {
    match action {
        // Attacks are generally good
        Action::Attack { .. } => 100,
        // Playing cards is important
        Action::PlayCard { .. } => 80,
        // Using abilities can be valuable
        Action::UseAbility { .. } => 60,
        // End turn only if nothing else to do
        Action::EndTurn => 0,
    }
}

