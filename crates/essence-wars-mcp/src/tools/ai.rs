//! AI hint tool using MCTS analysis.

use crate::session::SessionManager;
use cardgame::bots::{MctsBot, MctsConfig};
use cardgame::{Action, Target};
use std::time::Instant;

/// Get AI recommendation with MCTS analysis.
pub fn ai_hint(manager: &SessionManager, simulations: u32) -> String {
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

    // Create MCTS bot with specified simulations
    let mcts_config = MctsConfig {
        simulations,
        ..MctsConfig::default()
    };

    let mut mcts_bot = MctsBot::with_config(manager.card_db(), mcts_config, 42);

    // Time the analysis
    let start = Instant::now();

    // Get action using the regular bot API
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
    output.push_str(&format!("# AI Analysis ({} simulations)\n\n", simulations));

    // Show recommended action
    output.push_str(&format!(
        "## Recommended Move\n\n**Action**: {} (index: {})\n\n",
        format_action_description(&action, state, manager.card_db()),
        action.to_index()
    ));

    // Show analysis details
    output.push_str(&format!(
        "**Analysis Time**: {:.1}ms\n",
        elapsed.as_millis()
    ));
    output.push_str(&format!("**Legal Actions**: {}\n\n", legal_actions.len()));

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
        Action::EndTurn => "End your turn".to_string(),
    }
}
