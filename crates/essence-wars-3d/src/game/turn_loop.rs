//! Turn loop management for game progression.
//!
//! This module handles the game turn cycle, including both AI-controlled
//! and human-controlled players.

use std::collections::VecDeque;
use std::time::Instant;

use bevy::prelude::*;
use cardgame::actions::Action;
use cardgame::bots::{BotDecision, BotType, IntrospectionConfig, MctsConfig, PolicyOutput, PolicySource, create_bot};
use cardgame::client_api::GameEvent;
use cardgame::types::PlayerId;

use super::{AppState, GameBridge, HeadlessStats};
use crate::ui::{GameModeConfig, PlayerInputState};
use crate::CliArgs;

/// Plugin for turn loop management.
pub struct TurnLoopPlugin;

impl Plugin for TurnLoopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TurnState>()
            .init_resource::<BotConfig>()
            .init_resource::<GameEventQueue>()
            .add_event::<GameEventWrapper>()
            .add_systems(OnEnter(AppState::Playing), reset_turn_state)
            .add_systems(
                Update,
                (
                    execute_human_action,
                    execute_ai_turn,
                    dispatch_game_events,
                    log_game_events,
                    check_game_end,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// Bevy event wrapper for game events.
/// This allows multiple systems to read the same event.
#[derive(Event, Clone)]
pub struct GameEventWrapper(pub GameEvent);

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
    /// When the current game started (for duration tracking)
    pub game_start_time: Option<Instant>,
}

impl TurnState {
    /// Reset for a new game.
    pub fn reset(&mut self) {
        self.waiting = false;
        self.wait_timer = Timer::from_seconds(0.3, TimerMode::Once);
        self.actions_executed = 0;
        self.paused = false;
        self.game_start_time = Some(Instant::now());
    }

    /// Get game duration since start.
    pub fn game_duration(&self) -> std::time::Duration {
        self.game_start_time
            .map(|t| t.elapsed())
            .unwrap_or_default()
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
                simulations: 500,
                exploration: 1.414,
                max_rollout_depth: 100, // Match arena default
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
    cli_args: Option<Res<CliArgs>>,
    headless_stats: Option<Res<HeadlessStats>>,
) {
    // Don't execute if paused
    if turn_state.paused {
        return;
    }

    // Check if we're in fast mode (skip visual delays)
    let fast_mode = cli_args.as_ref().map(|a| a.fast).unwrap_or(false);
    let debug_mode = cli_args.as_ref().map(|a| a.debug).unwrap_or(false);

    // Handle pacing between actions (skip in fast mode)
    if turn_state.waiting {
        if fast_mode {
            // In fast mode, skip the wait entirely
            turn_state.waiting = false;
            turn_state.wait_timer.reset();
        } else {
            turn_state.wait_timer.tick(time.delta());
            if !turn_state.wait_timer.finished() {
                return;
            }
            turn_state.waiting = false;
            turn_state.wait_timer.reset();
        }
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

    // Get bot type for current player
    let bot_type = if current_player == PlayerId::PLAYER_ONE {
        &bot_config.player1_type
    } else {
        &bot_config.player2_type
    };

    // Get game info for logging (before bot creation to avoid borrow issues)
    let turn = client.get_state().map(|s| s.current_turn).unwrap_or(0);
    let game_num = headless_stats.as_ref().map(|s| s.games_played + 1).unwrap_or(1);

    // Create bot for the current player and select action
    // Scoped to release borrow of bridge.card_db before we mutate bridge
    let player_seed = if current_player == PlayerId::PLAYER_ONE {
        bot_config.bot_seed
    } else {
        bot_config.bot_seed.wrapping_add(1)
    };

    let (action, thinking_time_us) = {
        let mut bot = create_bot(
            &bridge.card_db,
            bot_type,
            None, // Use default weights
            &bot_config.mcts_config,
            player_seed,
        );

        // Time the decision for introspection
        let decision_start = Instant::now();
        let Some(action) = client.select_bot_action(bot.as_mut()) else {
            return;
        };
        let thinking_time_us = decision_start.elapsed().as_micros() as u64;
        (action, thinking_time_us)
    }; // bot is dropped here, releasing borrow of bridge.card_db

    // Debug logging (arena-style)
    if debug_mode {
        let player_num = if current_player == PlayerId::PLAYER_ONE { 1 } else { 2 };
        eprintln!("[Game {}] Turn {}: P{} ({}) {:?}", game_num, turn, player_num, bot_type.name(), action);
    } else {
        info!(
            "Turn {}: {} ({:?}) plays {:?}",
            turn, bot_type.name(), current_player, action
        );
    }

    // Determine policy source from bot type
    let policy_source = match bot_type {
        BotType::Random => PolicySource::Random,
        BotType::Greedy => PolicySource::Greedy,
        BotType::Mcts | BotType::AgentSpecialist(_) | BotType::AgentGeneralist => PolicySource::Mcts,
    };

    // Create a basic BotDecision for Glassbox visualization
    // Note: Full MCTS tree introspection requires implementing AnalyzableBot for MctsBot
    let bot_decision = BotDecision {
        turn,
        player: current_player,
        action: action.clone(),
        policy: Some(PolicyOutput {
            action_scores: vec![(action.clone(), 1.0)], // Placeholder - just the selected action
            value_estimate: 0.0, // Unknown without full introspection
            confidence: 1.0,
            source: policy_source,
        }),
        mcts_snapshot: None, // Full snapshot requires AnalyzableBot implementation
        thinking_time_us,
    };

    // Store decision for Glassbox visualization
    bridge.last_decision = Some(bot_decision);

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

            // In fast mode, don't wait between actions
            if fast_mode {
                turn_state.waiting = false;
            } else {
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
        }
        Err(e) => {
            error!("Failed to apply AI action {:?}: {}", action, e);
        }
    }
}

/// System to dispatch game events from queue to Bevy events.
/// This drains the queue and sends events that can be read by multiple systems.
pub fn dispatch_game_events(
    mut event_queue: ResMut<GameEventQueue>,
    mut event_writer: EventWriter<GameEventWrapper>,
) {
    // Drain all events from the queue and dispatch as Bevy events
    while let Some(event) = event_queue.events.pop_front() {
        event_writer.send(GameEventWrapper(event));
    }
}

/// System to log game events.
fn log_game_events(
    mut event_reader: EventReader<GameEventWrapper>,
    bridge: Res<GameBridge>,
) {
    for GameEventWrapper(event) in event_reader.read() {
        match event {
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
