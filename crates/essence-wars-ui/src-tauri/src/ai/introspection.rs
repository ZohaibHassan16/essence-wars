//! AI introspection types and utilities for UI visualization.
//!
//! This module bridges between the core cardgame introspection types
//! and the UI's needs for displaying AI decision-making data.

use cardgame::bots::{evaluate_position, GreedyBot, GreedyWeights};
use cardgame::cards::CardDatabase;
use cardgame::engine::GameEngine;
use cardgame::Action;
use serde::{Deserialize, Serialize};

use crate::state::{action_to_info, ActionInfo};

/// Move score for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveScoreDto {
    /// The action info
    pub action: ActionInfo,
    /// Raw evaluation score
    pub score: f32,
    /// Normalized probability (0-1)
    pub probability: f32,
    /// Whether this was the chosen move
    pub is_chosen: bool,
    /// MCTS visits (if available)
    pub visits: Option<u32>,
    /// MCTS win rate (if available)
    pub win_rate: Option<f32>,
}

/// Evaluation factor for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalFactorDto {
    /// Factor name
    pub name: String,
    /// Player 1 value
    pub p1_value: f32,
    /// Player 2 value
    pub p2_value: f32,
    /// Weight
    pub weight: f32,
    /// Contribution to total score
    pub contribution: f32,
}

/// Position evaluation breakdown for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalBreakdownDto {
    /// Total evaluation score (positive = P1 advantage)
    pub total_score: f32,
    /// Individual factors
    pub factors: Vec<EvalFactorDto>,
}

/// Search statistics for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchStatsDto {
    /// Algorithm name
    pub algorithm: String,
    /// Think time in ms
    pub time_ms: u64,
    /// Number of legal actions
    pub num_actions: u32,
    /// MCTS simulations (if applicable)
    pub simulations: Option<u32>,
    /// AlphaBeta depth (if applicable)
    pub depth: Option<u32>,
    /// Nodes evaluated (if applicable)
    pub nodes: Option<u64>,
}

/// Complete decision insights for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionInsightsDto {
    /// Player who made this decision (1 or 2)
    pub player: u8,
    /// Turn number
    pub turn: u16,
    /// The chosen action
    pub chosen_action: ActionInfo,
    /// All moves ranked by score
    pub move_scores: Vec<MoveScoreDto>,
    /// Evaluation breakdown (if available)
    pub eval_breakdown: Option<EvalBreakdownDto>,
    /// Search statistics
    pub search_stats: SearchStatsDto,
}

/// Extract decision insights using greedy evaluation.
///
/// This function evaluates all legal actions using the greedy bot's
/// evaluation function to provide move rankings. It works for any bot
/// type by evaluating post-hoc (after the bot has chosen).
pub fn extract_decision_insights(
    engine: &GameEngine,
    chosen_action: &Action,
    legal_actions: &[Action],
    card_db: &CardDatabase,
    bot_type: &str,
    think_time_ms: u64,
    mcts_simulations: Option<u32>,
    alphabeta_depth: Option<u32>,
) -> DecisionInsightsDto {
    let current_player = engine.current_player();
    let turn = engine.state.current_turn;

    // Create greedy bot for evaluation
    let greedy = GreedyBot::new(card_db, 42);
    let weights = greedy.weights();

    // Evaluate each action
    let mut scores: Vec<(Action, f32)> = Vec::with_capacity(legal_actions.len());
    for &action in legal_actions {
        let mut sim = engine.fork();
        let score = if sim.apply_action(action).is_ok() {
            evaluate_position(&sim.state, current_player, weights)
        } else {
            f32::NEG_INFINITY
        };
        scores.push((action, score));
    }

    // Normalize to probabilities using softmax
    let max_score = scores.iter().map(|(_, s)| *s).fold(f32::NEG_INFINITY, f32::max);
    let temperature = 10.0; // Higher = more uniform distribution
    let exp_scores: Vec<f32> = scores
        .iter()
        .map(|(_, s)| ((s - max_score) / temperature).exp())
        .collect();
    let sum_exp: f32 = exp_scores.iter().sum();

    // Build move scores
    let mut move_scores: Vec<MoveScoreDto> = Vec::with_capacity(scores.len());
    for (i, (action, score)) in scores.iter().enumerate() {
        let probability = if sum_exp > 0.0 {
            exp_scores[i] / sum_exp
        } else {
            1.0 / legal_actions.len() as f32
        };

        move_scores.push(MoveScoreDto {
            action: action_to_info(action, action.to_index()),
            score: *score,
            probability,
            is_chosen: action == chosen_action,
            visits: None,
            win_rate: None,
        });
    }

    // Sort by score descending
    move_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // Extract evaluation breakdown
    let eval_breakdown = extract_eval_breakdown(engine, weights);

    // Build search stats
    let search_stats = SearchStatsDto {
        algorithm: bot_type.to_string(),
        time_ms: think_time_ms,
        num_actions: legal_actions.len() as u32,
        simulations: mcts_simulations,
        depth: alphabeta_depth,
        nodes: None,
    };

    DecisionInsightsDto {
        player: current_player.0 + 1,
        turn,
        chosen_action: action_to_info(chosen_action, chosen_action.to_index()),
        move_scores,
        eval_breakdown: Some(eval_breakdown),
        search_stats,
    }
}

/// Extract evaluation breakdown for the current position.
fn extract_eval_breakdown(engine: &GameEngine, weights: &GreedyWeights) -> EvalBreakdownDto {
    let state = &engine.state;
    let p1 = &state.players[0];
    let p2 = &state.players[1];

    let mut factors = Vec::new();
    let mut total = 0.0;

    // Life (using own_life weight - relative evaluation)
    let p1_life = p1.life as f32;
    let p2_life = p2.life as f32;
    let life_contrib = p1_life * weights.own_life - p2_life * weights.own_life;
    total += life_contrib;
    factors.push(EvalFactorDto {
        name: "Life".to_string(),
        p1_value: p1_life,
        p2_value: p2_life,
        weight: weights.own_life,
        contribution: life_contrib,
    });

    // Board power (attack + health of creatures)
    let p1_power: f32 = p1.creatures.iter()
        .map(|c| c.attack as f32 * weights.own_creature_attack
              + c.current_health as f32 * weights.own_creature_health)
        .sum();
    let p2_power: f32 = p2.creatures.iter()
        .map(|c| c.attack as f32 * weights.enemy_creature_attack.abs()
              + c.current_health as f32 * weights.enemy_creature_health.abs())
        .sum();
    let power_contrib = p1_power - p2_power;
    total += power_contrib;
    factors.push(EvalFactorDto {
        name: "Board Power".to_string(),
        p1_value: p1_power,
        p2_value: p2_power,
        weight: 1.0, // Combined weight
        contribution: power_contrib,
    });

    // Card advantage
    let p1_cards = p1.hand.len() as f32;
    let p2_cards = p2.hand.len() as f32;
    let card_contrib = (p1_cards - p2_cards) * weights.cards_in_hand;
    total += card_contrib;
    factors.push(EvalFactorDto {
        name: "Cards in Hand".to_string(),
        p1_value: p1_cards,
        p2_value: p2_cards,
        weight: weights.cards_in_hand,
        contribution: card_contrib,
    });

    // Creature count
    let p1_creatures = p1.creatures.len() as f32;
    let p2_creatures = p2.creatures.len() as f32;
    let creature_contrib = (p1_creatures - p2_creatures) * weights.creature_count;
    total += creature_contrib;
    factors.push(EvalFactorDto {
        name: "Creatures".to_string(),
        p1_value: p1_creatures,
        p2_value: p2_creatures,
        weight: weights.creature_count,
        contribution: creature_contrib,
    });

    // Essence extracted (win condition progress)
    let p1_essence = p1.total_damage_dealt as f32;
    let p2_essence = p2.total_damage_dealt as f32;
    let essence_weight = 0.3; // Arbitrary weight for essence progress
    let essence_contrib = (p1_essence - p2_essence) * essence_weight;
    total += essence_contrib;
    factors.push(EvalFactorDto {
        name: "Essence".to_string(),
        p1_value: p1_essence,
        p2_value: p2_essence,
        weight: essence_weight,
        contribution: essence_contrib,
    });

    EvalBreakdownDto {
        total_score: total,
        factors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_factor_serialization() {
        let factor = EvalFactorDto {
            name: "Life".to_string(),
            p1_value: 20.0,
            p2_value: 15.0,
            weight: 1.0,
            contribution: 5.0,
        };
        let json = serde_json::to_string(&factor).unwrap();
        assert!(json.contains("p1Value"));
        assert!(json.contains("contribution"));
    }
}
