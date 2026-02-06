//! AI hint tool with hybrid Alpha-Beta/MCTS analysis.

use crate::session::SessionManager;
use cardgame::bots::{AlphaBetaBot, AlphaBetaConfig, MctsBot, MctsConfig};
use cardgame::{Action, GameState, Target};
use std::time::Instant;

/// Get AI recommendation using Alpha-Beta (default) or MCTS analysis.
///
/// Alpha-Beta is faster and deterministic, making it ideal for quick hints.
/// MCTS can be enabled for deeper analysis with win rate estimates.
pub fn ai_hint(manager: &SessionManager, simulations: u32, use_mcts: bool, depth: u32) -> String {
    let session = match manager.active_session() {
        Some(s) => s,
        None => return "No active game. Use `start_game` to begin.".to_string(),
    };

    if session.client.is_game_over() {
        return "Game is over. No hints available.".to_string();
    }

    let state = match session.client.get_state() {
        Some(s) => s,
        None => return "Error: Game state not available.".to_string(),
    };

    // Check if it's player's turn
    if state.active_player != session.player_id {
        return "It's the opponent's turn. Wait for them to play.".to_string();
    }

    if use_mcts {
        mcts_analysis(manager, session, state, simulations)
    } else {
        alphabeta_analysis(manager, session, state, depth)
    }
}

/// Perform Alpha-Beta analysis (fast, deterministic) with ranked move scoring.
fn alphabeta_analysis(
    manager: &SessionManager,
    session: &crate::session::GameSession,
    state: &GameState,
    depth: u32,
) -> String {
    let config = AlphaBetaConfig {
        max_depth: depth,
        ..AlphaBetaConfig::default()
    };

    let mut ab_bot = AlphaBetaBot::with_config(manager.card_db(), config, 42);

    // Time the analysis
    let start = Instant::now();

    // Get ranked moves using the new search_ranked method
    let engine = session.client.engine().expect("engine available");
    let ranked_moves = ab_bot.search_ranked(engine);

    let elapsed = start.elapsed();

    // Get search statistics
    let stats = ab_bot.last_stats();

    if ranked_moves.is_empty() {
        return "Error: Could not get AI recommendation.".to_string();
    }

    let (best_action, best_score) = ranked_moves[0];
    let second_best_score = ranked_moves.get(1).map(|(_, s)| *s);

    // Calculate confidence based on score gap
    let confidence = calculate_confidence(best_score, second_best_score, ranked_moves.len());

    let mut output = String::new();
    output.push_str(&format!("# AI Analysis (Alpha-Beta, depth {})\n\n", depth));

    // Show recommended action with score
    output.push_str("## Recommended Move\n\n");
    output.push_str(&format!(
        "**Action**: {} (index: {})\n",
        format_action_description(&best_action, state, manager.card_db()),
        best_action.to_index()
    ));
    output.push_str(&format!("**Evaluation**: {}\n", format_score(best_score)));
    output.push_str(&format!("**Confidence**: {}\n\n", confidence));

    // Show analysis details
    output.push_str("## Analysis Details\n\n");
    output.push_str(&format!("- **Search Time**: {:.1}ms\n", elapsed.as_millis()));
    output.push_str(&format!("- **Depth Reached**: {}\n", stats.max_depth_reached));
    output.push_str(&format!("- **Nodes Visited**: {}\n", stats.nodes_visited));
    output.push_str(&format!("- **Legal Actions**: {}\n\n", ranked_moves.len()));

    // Show ranked moves table
    output.push_str("## Ranked Moves\n\n");
    output.push_str("| Rank | Score | Index | Action | Description |\n");
    output.push_str("|------|-------|-------|--------|-------------|\n");

    for (rank, (action, score)) in ranked_moves.iter().enumerate() {
        let rank_marker = if rank == 0 { "→" } else { " " };
        let desc = format_action_description(action, state, manager.card_db());
        output.push_str(&format!(
            "| {}{} | {} | {} | {} | {} |\n",
            rank_marker,
            rank + 1,
            format_score_short(*score),
            action.to_index(),
            format_action_short(action),
            desc
        ));
    }
    output.push('\n');

    // Score interpretation guide
    output.push_str("## Score Guide\n\n");
    output.push_str("| Score | Meaning |\n");
    output.push_str("|-------|--------|\n");
    output.push_str("| > +200 | Winning position |\n");
    output.push_str("| +50 to +200 | Significant advantage |\n");
    output.push_str("| -50 to +50 | Roughly even |\n");
    output.push_str("| < -50 | Disadvantage |\n");
    output.push_str("| ±10000 | Forced win/loss |\n\n");

    output.push_str("---\n");
    output.push_str(&format!(
        "Use `play_action {}` to play the recommended move.\n",
        best_action.to_index()
    ));

    output
}

/// Format score for display (with interpretation).
fn format_score(score: f32) -> String {
    if score >= 10000.0 {
        format!("{:+.0} (Forced win detected)", score)
    } else if score <= -10000.0 {
        format!("{:+.0} (Forced loss detected)", score)
    } else if score > 200.0 {
        format!("{:+.0} (Winning)", score)
    } else if score > 50.0 {
        format!("{:+.0} (Advantage)", score)
    } else if score < -200.0 {
        format!("{:+.0} (Losing)", score)
    } else if score < -50.0 {
        format!("{:+.0} (Disadvantage)", score)
    } else {
        format!("{:+.0} (Even)", score)
    }
}

/// Format score for table display (compact).
fn format_score_short(score: f32) -> String {
    if score >= 10000.0 {
        "+WIN".to_string()
    } else if score <= -10000.0 {
        "-LOSS".to_string()
    } else {
        format!("{:+.0}", score)
    }
}

/// Calculate confidence level based on score gap and position.
fn calculate_confidence(best_score: f32, second_best: Option<f32>, num_moves: usize) -> String {
    // Forced win/loss = maximum confidence
    if best_score.abs() >= 10000.0 {
        return "Very High (forced outcome)".to_string();
    }

    // Only one legal move = no choice
    if num_moves == 1 {
        return "N/A (only one legal move)".to_string();
    }

    // Calculate gap between best and second-best
    let gap = match second_best {
        Some(second) => best_score - second,
        None => 0.0,
    };

    // Confidence based on score gap
    if gap >= 100.0 {
        "Very High (clear best move)".to_string()
    } else if gap >= 50.0 {
        "High (strong preference)".to_string()
    } else if gap >= 20.0 {
        "Medium (moderate preference)".to_string()
    } else if gap >= 5.0 {
        "Low (close alternatives exist)".to_string()
    } else {
        "Very Low (multiple similar options)".to_string()
    }
}

/// Perform MCTS analysis (slower, provides win rate estimates).
fn mcts_analysis(
    manager: &SessionManager,
    session: &crate::session::GameSession,
    state: &cardgame::GameState,
    simulations: u32,
) -> String {
    // Use interactive config for MCP single-game scenarios
    // This uses parallel trees for faster response time
    let mcts_config = MctsConfig::interactive(simulations);
    let parallel_trees = mcts_config.parallel_trees;

    let mut mcts_bot = MctsBot::with_config(manager.card_db(), mcts_config, 42);

    // Time the analysis
    let start = Instant::now();

    // Get action using the bot API
    let action = match session.client.select_bot_action(&mut mcts_bot) {
        Some(a) => a,
        None => {
            return "Error: Could not get AI recommendation.".to_string();
        }
    };

    let elapsed = start.elapsed();

    // Get all legal actions for comparison
    let legal_actions = session.client.get_legal_actions();

    let mut output = String::new();
    output.push_str(&format!("# AI Analysis (MCTS, {} simulations)\n\n", simulations));

    // Show recommended action
    output.push_str(&format!(
        "## Recommended Move\n\n**Action**: {} (index: {})\n\n",
        format_action_description(&action, state, manager.card_db()),
        action.to_index()
    ));

    // Show analysis details
    output.push_str("## Analysis Details\n\n");
    output.push_str(&format!("- **Analysis Time**: {:.1}ms\n", elapsed.as_millis()));
    output.push_str(&format!("- **Simulations**: {}\n", simulations));
    output.push_str(&format!("- **Legal Actions**: {}\n", legal_actions.len()));

    // MCTS provides implicit win rate through visit counts
    output.push_str(&format!(
        "\n**Note**: MCTS explores game trees via random rollouts ({} parallel trees).\n",
        parallel_trees
    ));
    output.push_str("Higher simulation count = more accurate recommendations.\n\n");

    // Show all legal actions as alternatives
    if legal_actions.len() > 1 {
        output.push_str("## Other Options\n\n");
        output.push_str("| Index | Action | Description |\n");
        output.push_str("|-------|--------|-------------|\n");

        for legal_action in &legal_actions {
            if *legal_action == action {
                continue; // Skip the recommended action
            }
            let desc = format_action_description(legal_action, state, manager.card_db());
            output.push_str(&format!(
                "| {} | {} | {} |\n",
                legal_action.to_index(),
                format_action_short(legal_action),
                desc
            ));
        }
        output.push('\n');
    }

    output.push_str("---\n");
    output.push_str(&format!(
        "Use `play_action {}` to play the recommended move.\n",
        action.to_index()
    ));

    output
}

fn format_action_short(action: &Action) -> String {
    match action {
        Action::PlayCard { hand_index, slot } => {
            format!("Play hand:{} -> slot:{}", hand_index, slot.0)
        }
        Action::Attack { attacker, defender } => {
            format!("Attack {} -> {}", attacker.0, defender.0)
        }
        Action::UseAbility { slot, ability_index, target } => {
            let target_str = format_target(target);
            format!(
                "Ability {} #{} -> {}",
                slot.0,
                ability_index,
                target_str
            )
        }
        Action::CommanderInsight => "Commander's Insight".to_string(),
        Action::EndTurn => "End Turn".to_string(),
    }
}

fn format_target(target: &Target) -> String {
    match target {
        Target::NoTarget => "-".to_string(),
        Target::EnemySlot(slot) => format!("enemy:{}", slot.0),
        Target::Self_ => "self".to_string(),
    }
}

fn format_action_description(
    action: &Action,
    state: &cardgame::GameState,
    card_db: &cardgame::CardDatabase,
) -> String {
    let player = &state.players[state.active_player.index()];
    let opponent = &state.players[state.active_player.opponent().index()];

    match action {
        Action::PlayCard { hand_index, slot } => {
            let idx = *hand_index as usize;
            if idx < player.hand.len() {
                let card_id = player.hand[idx].card_id;
                if let Some(card) = card_db.get(card_id) {
                    format!("Play {} (cost {}) to slot {}", card.name, card.cost, slot.0)
                } else {
                    format!("Play card from hand {} to slot {}", hand_index, slot.0)
                }
            } else {
                format!("Play card from hand {} to slot {}", hand_index, slot.0)
            }
        }
        Action::Attack { attacker, defender } => {
            let attacker_name = player
                .get_creature(*attacker)
                .and_then(|c| card_db.get(c.card_id))
                .map(|c| c.name.as_str())
                .unwrap_or("creature");

            let defender_name = opponent
                .get_creature(*defender)
                .and_then(|c| card_db.get(c.card_id))
                .map(|c| c.name.as_str())
                .unwrap_or("creature");

            format!("{} attacks {}", attacker_name, defender_name)
        }
        Action::UseAbility { slot, .. } => {
            let creature_name = player
                .get_creature(*slot)
                .and_then(|c| card_db.get(c.card_id))
                .map(|c| c.name.as_str())
                .unwrap_or("creature");

            format!("Use {}'s ability", creature_name)
        }
        Action::CommanderInsight => "Use Commander's Insight (draw a card for 4 essence)".to_string(),
        Action::EndTurn => "End your turn".to_string(),
    }
}
