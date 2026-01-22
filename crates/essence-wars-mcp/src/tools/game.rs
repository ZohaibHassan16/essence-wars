//! Game control tools for starting, playing, and ending games.

use crate::ascii;
use crate::session::SessionManager;
use crate::tools::ui_sync::try_push_state_to_ui;
use cardgame::core::state::GameResult;
use cardgame::{Action, Target};

/// Start a new game session.
pub fn start_game(
    manager: &mut SessionManager,
    player_deck: &str,
    opponent_deck: &str,
    bot_type: &str,
    seed: Option<u64>,
) -> String {
    match manager.start_game(player_deck, opponent_deck, bot_type, seed) {
        Ok(()) => {
            let session = manager.active_session().unwrap();
            let state = session.client.get_state().unwrap();

            // Sync state to Tauri UI
            try_push_state_to_ui(
                &session.client,
                manager.card_db(),
                session.player_id,
                &session.game_id,
            );

            let mut output = String::new();
            output.push_str("# Game Started!\n\n");
            output.push_str(&format!(
                "**You**: {} (Player 1)\n",
                session.player_deck_id
            ));
            output.push_str(&format!(
                "**Opponent**: {} ({}, Player 2)\n",
                session.opponent_deck_id,
                session.opponent_bot_type.name()
            ));
            output.push_str(&format!("**Seed**: {}\n\n", session.game_seed));

            output.push_str("---\n\n");
            output.push_str(&ascii::render_board(
                state,
                manager.card_db(),
                session.player_id,
            ));
            output.push_str("\n---\n\n");
            output.push_str("Use `show_hand` to see your cards, `legal_actions` to see available moves.\n");

            output
        }
        Err(e) => format!("Error starting game: {}", e),
    }
}

/// Show the current game state.
pub fn show_state(manager: &SessionManager) -> String {
    match manager.active_session() {
        Some(session) => match session.client.get_state() {
            Some(state) => {
                let mut output = String::new();
                output.push_str(&format!("# Turn {} | ", state.current_turn));

                if session.client.is_game_over() {
                    output.push_str("Game Over\n\n");
                } else {
                    let active = state.active_player;
                    if active == session.player_id {
                        output.push_str("Your Turn\n\n");
                    } else {
                        output.push_str("Opponent's Turn\n\n");
                    }
                }

                output.push_str(&ascii::render_board(
                    state,
                    manager.card_db(),
                    session.player_id,
                ));

                if let Some(result) = session.client.get_result() {
                    output.push_str("\n\n");
                    output.push_str(&format_game_result(result, session.player_id));
                }

                output
            }
            None => "Error: Game state not available.".to_string(),
        },
        None => "No active game. Use `start_game` to begin.".to_string(),
    }
}

/// Show the player's hand.
pub fn show_hand(manager: &SessionManager) -> String {
    match manager.active_session() {
        Some(session) => match session.client.get_state() {
            Some(state) => {
                let player_state = &state.players[session.player_id.index()];
                let current_essence = player_state.current_essence;

                ascii::render_hand(&player_state.hand, manager.card_db(), current_essence)
            }
            None => "Error: Game state not available.".to_string(),
        },
        None => "No active game. Use `start_game` to begin.".to_string(),
    }
}

/// List all legal actions.
pub fn legal_actions(manager: &SessionManager) -> String {
    match manager.active_session() {
        Some(session) => {
            if session.client.is_game_over() {
                return "Game is over. No legal actions.".to_string();
            }

            let state = match session.client.get_state() {
                Some(s) => s,
                None => return "Error: Game state not available.".to_string(),
            };

            // Check if it's the player's turn
            if state.active_player != session.player_id {
                return "It's the opponent's turn. Use `play_action` with any index to let the AI play.".to_string();
            }

            let actions = session.client.get_legal_actions();
            let mut output = String::new();
            output.push_str(&format!("# Legal Actions ({})\n\n", actions.len()));

            output.push_str("| Index | Action | Description |\n");
            output.push_str("|-------|--------|-------------|\n");

            for action in &actions {
                let index = action.to_index();
                let desc = format_action_description(action, state, manager.card_db());
                output.push_str(&format!(
                    "| {} | {} | {} |\n",
                    index,
                    format_action_short(action),
                    desc
                ));
            }

            output.push_str("\nUse `play_action <index>` to execute a move.\n");
            output
        }
        None => "No active game. Use `start_game` to begin.".to_string(),
    }
}

/// Play an action by index.
pub fn play_action(manager: &mut SessionManager, action_index: u8) -> String {
    // First check if we have a session
    if !manager.has_active_session() {
        return "No active game. Use `start_game` to begin.".to_string();
    }

    // Check if it's opponent's turn - if so, run AI
    {
        let session = manager.active_session().unwrap();
        if session.client.is_game_over() {
            return "Game is already over.".to_string();
        }

        let state = session.client.get_state().unwrap();
        if state.active_player != session.player_id {
            // It's AI's turn - run the AI
            let _ = session;
            return run_ai_and_show_state(manager);
        }
    }

    // Apply player action
    let action_result = {
        let actions = manager.active_session().unwrap().client.get_legal_actions();
        let action = actions.iter().find(|a| a.to_index() == action_index);
        action.cloned()
    };

    let action = match action_result {
        Some(a) => a,
        None => {
            return format!(
                "Invalid action index {}. Use `legal_actions` to see valid moves.",
                action_index
            );
        }
    };

    // Apply the action
    if let Err(e) = manager.apply_action(action_index) {
        return format!("Error applying action: {}", e);
    }

    // Sync state to UI after action
    sync_state_to_ui(manager);

    let mut output = String::new();
    output.push_str(&format!(
        "**You played**: {}\n\n",
        format_action_short(&action)
    ));

    // Check if game ended
    {
        let session = manager.active_session().unwrap();
        if session.client.is_game_over() {
            let state = session.client.get_state().unwrap();
            output.push_str(&ascii::render_board(
                state,
                manager.card_db(),
                session.player_id,
            ));
            output.push_str("\n\n");
            if let Some(result) = session.client.get_result() {
                output.push_str(&format_game_result(result, session.player_id));
            }
            return output;
        }
    }

    // If it's now opponent's turn, run AI
    {
        let session = manager.active_session().unwrap();
        let state = session.client.get_state().unwrap();
        if state.active_player != session.player_id {
            let _ = session;
            output.push_str("---\n\n");
            output.push_str(&run_ai_and_show_state(manager));
            return output;
        }
    }

    // Still player's turn - show state
    let session = manager.active_session().unwrap();
    let state = session.client.get_state().unwrap();
    output.push_str(&ascii::render_board(
        state,
        manager.card_db(),
        session.player_id,
    ));

    output
}

/// End the current game.
pub fn end_game(manager: &mut SessionManager) -> String {
    match manager.end_game() {
        Some(session) => {
            let state = session.client.get_state();
            let result = session.client.get_result();

            let mut output = String::new();
            output.push_str("# Game Ended\n\n");

            if let Some(result) = result {
                output.push_str(&format_game_result(result, session.player_id));
            } else {
                output.push_str("Game was ended early (no result).\n");
            }

            if let Some(state) = state {
                output.push_str(&format!(
                    "\n**Final Turn**: {}\n",
                    state.current_turn
                ));
                let player_life = state.players[session.player_id.index()].life;
                let opponent_life = state.players[session.player_id.opponent().index()].life;
                output.push_str(&format!(
                    "**Your Life**: {} | **Opponent Life**: {}\n",
                    player_life, opponent_life
                ));
            }

            output
        }
        None => "No active game to end.".to_string(),
    }
}

// Helper functions

/// Sync the current game state to the Tauri UI (non-fatal on failure).
fn sync_state_to_ui(manager: &SessionManager) {
    if let Some(session) = manager.active_session() {
        try_push_state_to_ui(
            &session.client,
            manager.card_db(),
            session.player_id,
            &session.game_id,
        );
    }
}

fn run_ai_and_show_state(manager: &mut SessionManager) -> String {
    let mut output = String::new();

    match manager.run_ai_turn() {
        Ok(actions) => {
            if !actions.is_empty() {
                output.push_str("**Opponent's moves:**\n");
                for action in &actions {
                    output.push_str(&format!("- {}\n", format_action_short(action)));
                }
                output.push('\n');
            }
        }
        Err(e) => {
            output.push_str(&format!("Error during AI turn: {}\n\n", e));
        }
    }

    // Sync state to UI after AI turn
    sync_state_to_ui(manager);

    // Show updated state
    let session = manager.active_session().unwrap();
    let state = session.client.get_state().unwrap();
    output.push_str(&ascii::render_board(
        state,
        manager.card_db(),
        session.player_id,
    ));

    if session.client.is_game_over() {
        output.push_str("\n\n");
        if let Some(result) = session.client.get_result() {
            output.push_str(&format_game_result(result, session.player_id));
        }
    }

    output
}

fn format_action_short(action: &Action) -> String {
    match action {
        Action::PlayCard { hand_index, slot } => {
            format!("PlayCard(hand:{}, slot:{})", hand_index, slot.0)
        }
        Action::Attack { attacker, defender } => {
            format!("Attack(slot:{} -> slot:{})", attacker.0, defender.0)
        }
        Action::UseAbility { slot, ability_index, target } => {
            let target_str = format_target(target);
            format!(
                "UseAbility(slot:{}, ability:{}, target:{})",
                slot.0,
                ability_index,
                target_str
            )
        }
        Action::EndTurn => "EndTurn".to_string(),
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
                    format!("Play card to slot {}", slot.0)
                }
            } else {
                format!("Play card to slot {}", slot.0)
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

fn format_game_result(result: GameResult, player_id: cardgame::PlayerId) -> String {
    match result {
        GameResult::Win { winner, reason } => {
            if winner == player_id {
                format!("**YOU WIN!** ({:?})", reason)
            } else {
                format!("**You lost.** ({:?})", reason)
            }
        }
        GameResult::Draw => "**DRAW** - Turn limit reached with equal life.".to_string(),
    }
}
