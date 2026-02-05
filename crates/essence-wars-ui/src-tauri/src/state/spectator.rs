//! Data structures for AI vs AI spectator mode.
//!
//! Supports pre-computed match playback with AI decision visualization.

use serde::{Deserialize, Serialize};
use super::serialization::{ActionInfo, GameEventDto, GameStateDto};
use crate::ai::DecisionInsightsDto;

/// Configuration for starting a spectator match
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectatorConfig {
    /// Deck ID for player 1
    pub player1_deck_id: String,
    /// Bot type for player 1 (random, greedy, mcts, alphabeta)
    pub player1_bot_type: String,
    /// Deck ID for player 2
    pub player2_deck_id: String,
    /// Bot type for player 2 (random, greedy, mcts, alphabeta)
    pub player2_bot_type: String,
    /// Optional fixed seed for reproducibility
    pub seed: Option<u64>,
    /// Number of MCTS simulations per move (default: 100 for fast playback)
    /// Higher values = stronger play but slower computation
    #[serde(default = "default_mcts_simulations")]
    pub mcts_simulations: u32,
    /// Alpha-Beta search depth (default: 4 for fast playback)
    /// Higher values = stronger play but exponentially slower
    #[serde(default = "default_alphabeta_depth")]
    pub alphabeta_depth: u32,
}

fn default_mcts_simulations() -> u32 {
    100 // Fast default for debug builds
}

fn default_alphabeta_depth() -> u32 {
    4 // Fast default for debug builds
}

/// A single action in the spectator match with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectatorAction {
    /// Turn number when this action occurred
    pub turn: u16,
    /// Which player took the action (1 or 2)
    pub player: u8,
    /// The action that was taken
    pub action: ActionInfo,
    /// Full game state after this action
    pub state_after: GameStateDto,
    /// Events triggered by this action (for animations)
    pub events: Vec<GameEventDto>,
    /// MCTS thinking data (if bot is MCTS) - legacy field
    pub thinking: Option<MctsThinkingDto>,
    /// AI decision insights with move scores and evaluation breakdown
    pub insights: Option<DecisionInsightsDto>,
    /// Time spent computing this move in milliseconds
    pub thinking_time_ms: u64,
}

/// MCTS thinking data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MctsThinkingDto {
    /// Total MCTS simulations performed
    pub total_simulations: u32,
    /// Top candidate moves with statistics
    pub top_moves: Vec<MctsMoveDto>,
    /// Win rate of the selected move
    pub selected_win_rate: f32,
}

/// A candidate move considered by MCTS
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MctsMoveDto {
    /// The action being considered
    pub action: ActionInfo,
    /// Number of times this move was visited in the search tree
    pub visits: u32,
    /// Estimated win rate for this move (0.0 - 1.0)
    pub win_rate: f32,
}

/// Complete pre-computed spectator match
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectatorMatch {
    /// Unique match identifier
    pub id: String,
    /// Original configuration used to create this match
    pub config: SpectatorConfig,
    /// Game state before any actions (starting state)
    pub initial_state: GameStateDto,
    /// All actions in the match, in order
    pub actions: Vec<SpectatorAction>,
    /// Final result of the match
    pub result: SpectatorResult,
    /// Total number of turns in the match
    pub total_turns: u16,
    /// Display name for player 1's deck
    pub player1_deck_name: String,
    /// Display name for player 2's deck
    pub player2_deck_name: String,
    /// Display name for player 1's bot
    pub player1_bot_name: String,
    /// Display name for player 2's bot
    pub player2_bot_name: String,
    /// Evaluation history for timeline visualization
    pub eval_history: Vec<EvalPoint>,
    /// Key moments where evaluation swung significantly
    pub key_moments: Vec<KeyMoment>,
}

/// A point on the evaluation timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalPoint {
    /// Action index (0-based)
    pub action_index: usize,
    /// Turn number
    pub turn: u16,
    /// Which player acted (1 or 2)
    pub player: u8,
    /// Evaluation score (positive = P1 advantage)
    pub eval_score: f32,
}

/// A key moment in the game (significant eval swing)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyMoment {
    /// Action index where this moment occurred
    pub action_index: usize,
    /// Turn number
    pub turn: u16,
    /// Which player acted
    pub player: u8,
    /// Description of the action
    pub action_description: String,
    /// Evaluation change (delta from previous)
    pub eval_delta: f32,
    /// New evaluation score after this action
    pub eval_after: f32,
    /// Type of moment
    pub moment_type: KeyMomentType,
}

/// Types of key moments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KeyMomentType {
    /// Large positive swing for P1
    P1Surge,
    /// Large positive swing for P2 (negative eval delta)
    P2Surge,
    /// Lead changed hands
    LeadChange,
    /// Game-deciding moment
    Decisive,
}

/// Result of a completed spectator match
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectatorResult {
    /// Winner (1 or 2), or None for draw
    pub winner: Option<u8>,
    /// Reason for game end (e.g., "LifeZero", "TurnLimit", "draw")
    pub reason: String,
    /// Player 1's final life total
    pub player1_final_life: i16,
    /// Player 2's final life total
    pub player2_final_life: i16,
}

/// Progress update during match computation (for future async support)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Reserved for future streaming progress updates
pub struct ComputeProgress {
    /// Current turn being computed
    pub current_turn: u16,
    /// Total actions computed so far
    pub actions_computed: usize,
    /// Whether computation is complete
    pub is_complete: bool,
    /// The completed match (only present when is_complete is true)
    pub match_data: Option<SpectatorMatch>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameManager;

    #[test]
    fn test_spectator_config_serialization() {
        let config = SpectatorConfig {
            player1_deck_id: "architect_fortify".to_string(),
            player1_bot_type: "mcts".to_string(),
            player2_deck_id: "broodmother_pack".to_string(),
            player2_bot_type: "greedy".to_string(),
            seed: Some(12345),
            mcts_simulations: 100,
            alphabeta_depth: 4,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("player1DeckId"));
        assert!(json.contains("player1BotType"));
        assert!(json.contains("mctsSimulations"));
        assert!(json.contains("alphabetaDepth"));

        let parsed: SpectatorConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.player1_deck_id, "architect_fortify");
        assert_eq!(parsed.seed, Some(12345));
        assert_eq!(parsed.mcts_simulations, 100);
        assert_eq!(parsed.alphabeta_depth, 4);
    }

    #[test]
    fn test_mcts_thinking_dto_serialization() {
        let thinking = MctsThinkingDto {
            total_simulations: 1024,
            top_moves: vec![
                MctsMoveDto {
                    action: ActionInfo {
                        index: 42,
                        action_type: "attack".to_string(),
                        description: "Attack slot 2".to_string(),
                        source_slot: Some(1),
                        target_slot: Some(2),
                        hand_index: None,
                        card_id: None,
                        ability_index: None,
                    },
                    visits: 512,
                    win_rate: 0.58,
                },
            ],
            selected_win_rate: 0.58,
        };

        let json = serde_json::to_string(&thinking).unwrap();
        assert!(json.contains("totalSimulations"));
        assert!(json.contains("topMoves"));
        assert!(json.contains("winRate"));
    }

    #[test]
    fn test_spectator_result_serialization() {
        // Test with winner
        let result_win = SpectatorResult {
            winner: Some(1),
            reason: "LifeZero".to_string(),
            player1_final_life: 12,
            player2_final_life: 0,
        };
        let json = serde_json::to_string(&result_win).unwrap();
        assert!(json.contains("\"winner\":1"));

        // Test draw
        let result_draw = SpectatorResult {
            winner: None,
            reason: "TurnLimit".to_string(),
            player1_final_life: 15,
            player2_final_life: 15,
        };
        let json = serde_json::to_string(&result_draw).unwrap();
        assert!(json.contains("\"winner\":null"));
    }

    #[test]
    fn test_compute_spectator_match_random_vs_random() {
        use super::super::SpectatorComputer;

        let game_manager = GameManager::new().expect("Failed to create game manager");
        let computer = SpectatorComputer::new(&game_manager, None);

        let config = SpectatorConfig {
            player1_deck_id: "architect_fortify".to_string(),
            player1_bot_type: "random".to_string(),
            player2_deck_id: "broodmother_pack".to_string(),
            player2_bot_type: "random".to_string(),
            seed: Some(12345), // Fixed seed for reproducibility
            mcts_simulations: 100,
            alphabeta_depth: 4,
        };

        let result = computer.compute_match(config);
        assert!(result.is_ok(), "Match computation failed: {:?}", result.err());

        let match_data = result.unwrap();

        // Verify basic structure
        assert!(!match_data.id.is_empty());
        assert!(!match_data.actions.is_empty(), "Match should have at least one action");
        assert!(match_data.total_turns > 0, "Match should have at least one turn");

        // Verify deck names are populated
        assert!(!match_data.player1_deck_name.is_empty());
        assert!(!match_data.player2_deck_name.is_empty());

        // Verify bot names
        assert_eq!(match_data.player1_bot_name, "Random Bot");
        assert_eq!(match_data.player2_bot_name, "Random Bot");

        // Verify result makes sense
        let result = &match_data.result;
        // Winner can be None (draw), 1, or 2
        assert!(
            result.winner.is_none() || result.winner == Some(1) || result.winner == Some(2),
            "Winner must be None, 1, or 2"
        );

        // Verify actions have valid structure
        for action in &match_data.actions {
            assert!(action.player == 1 || action.player == 2);
            assert!(action.turn > 0);
        }

        // Verify initial state is populated
        assert!(!match_data.initial_state.id.is_empty());

        println!(
            "Match completed: {} turns, {} actions, winner: {:?}",
            match_data.total_turns,
            match_data.actions.len(),
            match_data.result.winner
        );
    }

    #[test]
    fn test_compute_spectator_match_greedy_vs_greedy() {
        use super::super::SpectatorComputer;

        let game_manager = GameManager::new().expect("Failed to create game manager");
        let computer = SpectatorComputer::new(&game_manager, None);

        let config = SpectatorConfig {
            player1_deck_id: "sovereign_lifesteal".to_string(),
            player1_bot_type: "greedy".to_string(),
            player2_deck_id: "alpha_frenzy".to_string(),
            player2_bot_type: "greedy".to_string(),
            seed: Some(54321),
            mcts_simulations: 100,
            alphabeta_depth: 4,
        };

        let result = computer.compute_match(config);
        assert!(result.is_ok(), "Match computation failed: {:?}", result.err());

        let match_data = result.unwrap();

        // Verify bot names
        assert_eq!(match_data.player1_bot_name, "Greedy Bot");
        assert_eq!(match_data.player2_bot_name, "Greedy Bot");

        // Greedy bots should produce a reasonable game
        assert!(match_data.actions.len() > 10, "Greedy match should have meaningful actions");

        println!(
            "Greedy match: {} turns, {} actions",
            match_data.total_turns,
            match_data.actions.len()
        );
    }

    #[test]
    fn test_insights_are_populated() {
        use super::super::SpectatorComputer;

        let game_manager = GameManager::new().expect("Failed to create game manager");
        let computer = SpectatorComputer::new(&game_manager, None);

        let config = SpectatorConfig {
            player1_deck_id: "architect_fortify".to_string(),
            player1_bot_type: "greedy".to_string(),
            player2_deck_id: "broodmother_pack".to_string(),
            player2_bot_type: "greedy".to_string(),
            seed: Some(99999),
            mcts_simulations: 100,
            alphabeta_depth: 4,
        };

        let result = computer.compute_match(config);
        assert!(result.is_ok(), "Match computation failed: {:?}", result.err());

        let match_data = result.unwrap();

        // Verify that at least some actions have insights
        let actions_with_insights = match_data.actions.iter()
            .filter(|a| a.insights.is_some())
            .count();

        assert!(
            actions_with_insights > 0,
            "Expected at least some actions to have insights, found none"
        );

        // Verify insights structure for the first action with insights
        let first_insight = match_data.actions.iter()
            .find_map(|a| a.insights.as_ref())
            .expect("Should have at least one insight");

        // Verify insights have proper structure
        assert!(!first_insight.move_scores.is_empty(), "Move scores should not be empty");
        assert!(first_insight.eval_breakdown.is_some(), "Eval breakdown should be present");
        assert_eq!(first_insight.search_stats.algorithm, "GreedyBot");

        // Verify move scores have valid probabilities
        let total_prob: f32 = first_insight.move_scores.iter()
            .map(|m| m.probability)
            .sum();
        assert!(
            (total_prob - 1.0).abs() < 0.01,
            "Probabilities should sum to ~1.0, got {}",
            total_prob
        );

        // Verify exactly one move is marked as chosen
        let chosen_count = first_insight.move_scores.iter()
            .filter(|m| m.is_chosen)
            .count();
        assert_eq!(chosen_count, 1, "Exactly one move should be marked as chosen");

        println!(
            "Insights test: {} actions, {} with insights, first insight has {} move scores",
            match_data.actions.len(),
            actions_with_insights,
            first_insight.move_scores.len()
        );
    }
}
