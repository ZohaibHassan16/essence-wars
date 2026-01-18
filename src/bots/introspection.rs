//! Bot introspection for Glassbox mode.
//!
//! This module provides types and traits for exposing AI decision-making
//! details for visualization and debugging.

use serde::{Deserialize, Serialize};
use crate::core::actions::Action;
use crate::core::types::PlayerId;

/// Statistics for a single MCTS node (action candidate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MctsNodeStats {
    /// The action this node represents.
    pub action: Action,
    /// Number of times this action was visited.
    pub visits: u32,
    /// Number of wins from this action.
    pub wins: f32,
    /// Win rate (wins / visits).
    pub win_rate: f32,
    /// UCB1 score used for selection.
    pub ucb1_score: f32,
    /// Prior probability (if using policy network).
    pub prior: Option<f32>,
}

/// Snapshot of MCTS tree state for visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MctsTreeSnapshot {
    /// Total simulations performed.
    pub total_simulations: u32,
    /// Visits at the root node.
    pub root_visits: u32,
    /// Statistics for each child action.
    pub children: Vec<MctsNodeStats>,
    /// The action that was selected.
    pub selected_action: Action,
    /// Win rate estimate from root.
    pub root_value: f32,
    /// Maximum tree depth reached.
    pub max_depth: u32,
}

/// Source of the policy/evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicySource {
    /// Monte Carlo Tree Search.
    Mcts,
    /// Greedy heuristic evaluation.
    Greedy,
    /// Random selection.
    Random,
    /// Neural network policy.
    NeuralNetwork,
}

/// Unified policy output for any bot type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyOutput {
    /// Action probabilities or scores.
    pub action_scores: Vec<(Action, f32)>,
    /// Value estimate for current position (from active player perspective).
    pub value_estimate: f32,
    /// Confidence in the evaluation (0.0 - 1.0).
    pub confidence: f32,
    /// Source of this policy.
    pub source: PolicySource,
}

impl PolicyOutput {
    /// Get the best action according to this policy.
    pub fn best_action(&self) -> Option<Action> {
        self.action_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(action, _)| *action)
    }

    /// Get top N actions by score.
    pub fn top_actions(&self, n: usize) -> Vec<(Action, f32)> {
        let mut sorted = self.action_scores.clone();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(n).collect()
    }

    /// Normalize scores to probabilities (sum to 1.0).
    pub fn as_probabilities(&self) -> Vec<(Action, f32)> {
        let sum: f32 = self.action_scores.iter().map(|(_, s)| s).sum();
        if sum <= 0.0 {
            return self.action_scores.clone();
        }
        self.action_scores
            .iter()
            .map(|(a, s)| (*a, s / sum))
            .collect()
    }
}

/// Complete decision record for replay/visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotDecision {
    /// Turn number.
    pub turn: u16,
    /// Player making the decision.
    pub player: PlayerId,
    /// The action chosen.
    pub action: Action,
    /// Policy output (action scores).
    pub policy: Option<PolicyOutput>,
    /// MCTS tree snapshot (if available).
    pub mcts_snapshot: Option<MctsTreeSnapshot>,
    /// Time spent thinking (microseconds).
    pub thinking_time_us: u64,
}

impl BotDecision {
    /// Create a simple decision without introspection data.
    pub fn simple(turn: u16, player: PlayerId, action: Action, thinking_time_us: u64) -> Self {
        Self {
            turn,
            player,
            action,
            policy: None,
            mcts_snapshot: None,
            thinking_time_us,
        }
    }

    /// Create a decision with policy output.
    pub fn with_policy(
        turn: u16,
        player: PlayerId,
        action: Action,
        policy: PolicyOutput,
        thinking_time_us: u64,
    ) -> Self {
        Self {
            turn,
            player,
            action,
            policy: Some(policy),
            mcts_snapshot: None,
            thinking_time_us,
        }
    }

    /// Create a full MCTS decision.
    pub fn mcts(
        turn: u16,
        player: PlayerId,
        action: Action,
        policy: PolicyOutput,
        mcts_snapshot: MctsTreeSnapshot,
        thinking_time_us: u64,
    ) -> Self {
        Self {
            turn,
            player,
            action,
            policy: Some(policy),
            mcts_snapshot: Some(mcts_snapshot),
            thinking_time_us,
        }
    }
}

/// Configuration for introspection behavior.
#[derive(Debug, Clone, Default)]
pub struct IntrospectionConfig {
    /// Whether to record MCTS tree snapshots.
    pub record_mcts_tree: bool,
    /// Maximum depth to record in MCTS tree.
    pub max_tree_depth: u32,
    /// Whether to record all action scores.
    pub record_all_actions: bool,
    /// Number of top actions to record (if not recording all).
    pub top_n_actions: usize,
}

impl IntrospectionConfig {
    /// Configuration for full introspection (expensive).
    pub fn full() -> Self {
        Self {
            record_mcts_tree: true,
            max_tree_depth: 10,
            record_all_actions: true,
            top_n_actions: 0,
        }
    }

    /// Configuration for lightweight introspection.
    pub fn lightweight() -> Self {
        Self {
            record_mcts_tree: false,
            max_tree_depth: 0,
            record_all_actions: false,
            top_n_actions: 5,
        }
    }

    /// Disabled introspection.
    pub fn disabled() -> Self {
        Self::default()
    }
}
