//! Arena configuration structures.
//!
//! Provides configuration types for running matches between bots.

use crate::bots::{BotType, BotWeights, MctsConfig};
use crate::core::state::GameMode;
use crate::types::CardId;

/// Configuration for a single match between two bots.
#[derive(Clone, Debug)]
pub struct MatchConfig {
    /// Bot 1 type
    pub bot1_type: BotType,
    /// Bot 2 type
    pub bot2_type: BotType,
    /// Deck for bot 1
    pub deck1: Vec<CardId>,
    /// Deck for bot 2
    pub deck2: Vec<CardId>,
    /// Custom weights for bot 1 (optional)
    pub weights1: Option<BotWeights>,
    /// Custom weights for bot 2 (optional)
    pub weights2: Option<BotWeights>,
    /// Number of games to play
    pub games: usize,
    /// Base seed for deterministic execution
    pub seed: u64,
    /// MCTS configuration
    pub mcts_config: MctsConfig,
    /// Whether to show progress during execution
    pub show_progress: bool,
    /// Game mode (Attrition or EssenceDuel)
    pub game_mode: GameMode,
}

impl MatchConfig {
    /// Create a new match configuration.
    pub fn new(
        bot1_type: BotType,
        bot2_type: BotType,
        deck1: Vec<CardId>,
        deck2: Vec<CardId>,
        games: usize,
        seed: u64,
    ) -> Self {
        Self {
            bot1_type,
            bot2_type,
            deck1,
            deck2,
            weights1: None,
            weights2: None,
            games,
            seed,
            mcts_config: MctsConfig::default(),
            show_progress: false,
            game_mode: GameMode::default(),
        }
    }

    /// Set the game mode.
    pub fn with_game_mode(mut self, mode: GameMode) -> Self {
        self.game_mode = mode;
        self
    }

    /// Set weights for bot 1.
    pub fn with_weights1(mut self, weights: Option<BotWeights>) -> Self {
        self.weights1 = weights;
        self
    }

    /// Set weights for bot 2.
    pub fn with_weights2(mut self, weights: Option<BotWeights>) -> Self {
        self.weights2 = weights;
        self
    }

    /// Set MCTS configuration.
    pub fn with_mcts_config(mut self, config: MctsConfig) -> Self {
        self.mcts_config = config;
        self
    }

    /// Enable progress display.
    pub fn with_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }
}

/// Configuration for sequential execution with debugging features.
#[derive(Clone, Debug, Default)]
pub struct SequentialConfig {
    /// Enable invariant checking after each action
    pub check_invariants: bool,
    /// Enable combat resolution tracing
    pub trace_combat: bool,
    /// Enable effect queue tracing
    pub trace_effects: bool,
}

impl SequentialConfig {
    /// Create a new sequential configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable invariant checking.
    pub fn with_invariants(mut self, enabled: bool) -> Self {
        self.check_invariants = enabled;
        self
    }

    /// Enable combat tracing.
    pub fn with_combat_tracing(mut self, enabled: bool) -> Self {
        self.trace_combat = enabled;
        self
    }

    /// Enable effect tracing.
    pub fn with_effect_tracing(mut self, enabled: bool) -> Self {
        self.trace_effects = enabled;
        self
    }

    /// Check if any tracing is enabled.
    pub fn tracing_enabled(&self) -> bool {
        self.trace_combat || self.trace_effects
    }
}
