//! Turn loop management for game progression.
//!
//! This module handles the game turn cycle, including both AI-controlled
//! and human-controlled players.

use std::collections::VecDeque;

use bevy::prelude::*;
use cardgame::actions::Action;
use cardgame::bots::{BotType, IntrospectionConfig, MctsBot, MctsConfig};
use cardgame::client_api::GameEvent;

use super::{AppState, GameBridge};
use crate::ui::{GameModeConfig, PlayerInputState};

/// Plugin for turn loop management.
pub struct TurnLoopPlugin;

impl Plugin for TurnLoopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TurnState>()
            .init_resource::<BotConfig>()
            .init_resource::<GameEventQueue>()
            .add_systems(OnEnter(AppState::Playing), reset_turn_state)
            .add_systems(
                Update,
                (
                    execute_human_action,
                    execute_ai_turn,
                    process_game_events,
                    check_game_end,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// Current state of the turn loop.
#[derive(Resource, Default)]
pub struct TurnState {
    /// Whether we're waiting between actions for visual pacing
    pub waiting: bool,
    /// Timer for pacing between actions
    pub wait_timer: Timer,
    /// Number of actions executed this game
    pub actions_executed: u32,
    /// Is the game paused (for spectator controls)
    pub paused: bool,
}

impl TurnState {
    /// Reset for a new game.
    pub fn reset(&mut self) {
        self.waiting = false;
        self.wait_timer = Timer::from_seconds(0.3, TimerMode::Once);
        self.actions_executed = 0;
        self.paused = false;
    }
}

/// Configuration for the AI players.
#[derive(Resource)]
pub struct BotConfig {
    /// Bot type for Player 1
    pub player1_type: BotType,
    /// Bot type for Player 2
    pub player2_type: BotType,
    /// MCTS configuration (shared)
    pub mcts_config: MctsConfig,
    /// Introspection configuration
    pub introspection_config: IntrospectionConfig,
    /// Random seed for bot decisions
    pub bot_seed: u64,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            player1_type: BotType::Mcts,
            player2_type: BotType::Mcts,
            mcts_config: MctsConfig {
                simulations: 200,
                exploration: 1.414,
                max_rollout_depth: 50,
                parallel_trees: 1,  // Single-threaded for WASM compatibility
                leaf_rollouts: 1,
            },
            introspection_config: IntrospectionConfig::full(),
            bot_seed: 42,
        }
    }
}

/// Queue of game events to process (for visual feedback).
#[derive(Resource, Default)]
pub struct GameEventQueue {
    /// Events waiting to be processed
    pub events: VecDeque<GameEvent>,
}

/// System to reset turn state when entering Playing state.
fn reset_turn_state(mut turn_state: ResMut<TurnState>) {
    turn_state.reset();
    info!("Turn state reset for new game");
}

/// System to execute human player actions.
fn execute_human_action(
    mut turn_state: ResMut<TurnState>,
    mut input_state: ResMut<PlayerInputState>,
    game_mode: Res<GameModeConfig>,
    mut bridge: ResMut<GameBridge>,
    mut event_queue: ResMut<GameEventQueue>,
) {
    // Check if there's a pending action from the human player
    let Some(action) = input_state.pending_action.take() else {
        return;
    };

    let Some(client) = bridge.client.as_ref() else {
        return;
    };

    // Verify it's actually a human player's turn
    let Some(current_player) = client.current_player() else {
        return;
    };

    if !game_mode.is_human(current_player) {
        // Put the action back - it shouldn't have been set
        input_state.pending_action = Some(action);
        return;
    }

    // Log the action
    let turn = client.get_state().map(|s| s.current_turn).unwrap_or(0);
    info!(
        "Turn {}: Human ({:?}) plays {:?}",
        turn, current_player, action
    );

    // Get mutable access to client for applying the action
    let client = bridge.client.as_mut().unwrap();

    // Apply the action
    match client.apply_action(action.clone()) {
        Ok(events) => {
            // Add events to queue for processing
            for event in events {
                event_queue.events.push_back(event);
            }
            turn_state.actions_executed += 1;

            // Brief delay for visual feedback
            turn_state.waiting = true;
            turn_state.wait_timer = Timer::from_seconds(0.1, TimerMode::Once);
        }
        Err(e) => {
            error!("Failed to apply human action {:?}: {}", action, e);
        }
    }

    // Clear card selection after action
    input_state.selected_card = None;
}

/// Main system that executes AI turns.
fn execute_ai_turn(
    time: Res<Time>,
    mut turn_state: ResMut<TurnState>,
    bot_config: Res<BotConfig>,
    game_mode: Res<GameModeConfig>,
    mut bridge: ResMut<GameBridge>,
    mut event_queue: ResMut<GameEventQueue>,
) {
    // Don't execute if paused
    if turn_state.paused {
        return;
    }

    // Handle pacing between actions
    if turn_state.waiting {
        turn_state.wait_timer.tick(time.delta());
        if !turn_state.wait_timer.finished() {
            return;
        }
        turn_state.waiting = false;
        turn_state.wait_timer.reset();
    }

    // Get the game client
    let Some(client) = bridge.client.as_ref() else {
        return;
    };

    // Check if game is over
    if client.is_game_over() {
        return;
    }

    // Get current player
    let Some(current_player) = client.current_player() else {
        return;
    };

    // If it's a human player's turn, don't execute AI
    if game_mode.is_human(current_player) {
        return;
    }

    // Create MCTS bot for the current player
    // We create a fresh bot each time since MCTS doesn't benefit from persistence
    let mut bot = MctsBot::with_config(
        &bridge.card_db,
        bot_config.mcts_config.clone(),
        bot_config.bot_seed.wrapping_add(turn_state.actions_executed as u64),
    );

    // Select action using the bot with full engine access
    // TODO: Add introspection support for Glassbox visualization
    let Some(action) = client.select_bot_action(&mut bot) else {
        return;
    };

    // Log the action
    let turn = client.get_state().map(|s| s.current_turn).unwrap_or(0);
    info!(
        "Turn {}: AI ({:?}) plays {:?}",
        turn, current_player, action
    );

    // Now get mutable access to client for applying the action
    let client = bridge.client.as_mut().unwrap();

    // Apply the action
    match client.apply_action(action.clone()) {
        Ok(events) => {
            // Add events to queue for processing
            for event in events {
                event_queue.events.push_back(event);
            }
            turn_state.actions_executed += 1;

            // Start wait timer for pacing (unless it's just EndTurn)
            if !matches!(action, Action::EndTurn) {
                turn_state.waiting = true;
                turn_state.wait_timer = Timer::from_seconds(0.2, TimerMode::Once);
            } else {
                // Shorter delay for end turn
                turn_state.waiting = true;
                turn_state.wait_timer = Timer::from_seconds(0.1, TimerMode::Once);
            }
        }
        Err(e) => {
            error!("Failed to apply AI action {:?}: {}", action, e);
        }
    }
}

/// System to process game events from the queue.
fn process_game_events(
    mut event_queue: ResMut<GameEventQueue>,
    bridge: Res<GameBridge>,
) {
    // Process a batch of events each frame
    let batch_size = 10;
    for _ in 0..batch_size {
        let Some(event) = event_queue.events.pop_front() else {
            break;
        };

        // Log significant events
        match &event {
            GameEvent::TurnStarted { turn_number, player, .. } => {
                info!("=== Turn {} - {:?} ===", turn_number, player);
            }
            GameEvent::CreatureSpawned { instance_id, card_id, player, slot, .. } => {
                if let Some(card) = bridge.card_db.get(*card_id) {
                    info!(
                        "{:?} spawns {} (id:{}) in slot {:?}",
                        player, card.name, instance_id.0, slot
                    );
                }
            }
            GameEvent::CreatureDied { instance_id, card_id, player, .. } => {
                if let Some(card) = bridge.card_db.get(*card_id) {
                    info!("{:?}'s {} (id:{}) died", player, card.name, instance_id.0);
                }
            }
            GameEvent::LifeChanged { player, old_life, new_life, .. } => {
                if old_life != new_life {
                    info!("{:?} life: {} -> {}", player, old_life, new_life);
                }
            }
            GameEvent::GameEnded { result, .. } => {
                info!("Game ended: {:?}", result);
            }
            _ => {
                // Other events - don't log for now to reduce noise
            }
        }
    }
}

/// System to check if the game has ended and transition state.
fn check_game_end(
    bridge: Res<GameBridge>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(client) = &bridge.client else {
        return;
    };

    if client.is_game_over() {
        info!("Game over detected, transitioning to GameOver state");
        next_state.set(AppState::GameOver);
    }
}
