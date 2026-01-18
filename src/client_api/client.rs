//! GameClient - High-level game controller for clients.
//!
//! GameClient wraps GameEngine and provides:
//! - Event emission for reactive UIs
//! - Unified handling of human and AI players
//! - Event log for replay and debugging

use std::sync::Arc;

use crate::bots::Bot;
use crate::client_api::diff::{diff_states, StateSnapshot};
use crate::client_api::events::GameEvent;
use crate::core::actions::Action;
use crate::core::cards::CardDatabase;
use crate::core::engine::GameEngine;
use crate::core::state::{GameMode, GameResult, GameState};
use crate::core::types::{CardId, PlayerId};
use crate::tensor::{legal_mask_to_tensor, state_to_tensor};

/// High-level game client that wraps GameEngine.
///
/// GameClient provides a higher-level interface than GameEngine, with:
/// - Automatic event emission for all state changes
/// - Built-in event history for replays
/// - Support for both human and AI players
///
/// # Example
/// ```ignore
/// use cardgame::client_api::GameClient;
/// use cardgame::cards::CardDatabase;
/// use std::sync::Arc;
///
/// let db = Arc::new(CardDatabase::load_all().unwrap());
/// let mut client = GameClient::new(db);
/// client.start_game(deck1, deck2, 12345);
///
/// while !client.is_game_over() {
///     let events = client.drain_events();
///     // Process events for UI updates
///
///     let legal = client.get_legal_actions();
///     // Get player input...
///     client.apply_action(action);
/// }
/// ```
pub struct GameClient {
    card_db: Arc<CardDatabase>,
    engine: Option<GameEngine<'static>>,
    /// Event buffer for recently emitted events
    event_buffer: Vec<GameEvent>,
    /// Complete event history (for replay)
    event_history: Vec<GameEvent>,
    /// Pre-action state snapshot for diffing
    pre_action_snapshot: Option<StateSnapshot>,
    /// Configuration options
    config: GameClientConfig,
}

/// Configuration options for GameClient.
#[derive(Clone, Debug)]
pub struct GameClientConfig {
    /// Whether to keep full event history (required for replay export)
    pub keep_history: bool,
    /// Maximum events to buffer before auto-draining
    pub max_buffer_size: usize,
}

impl Default for GameClientConfig {
    fn default() -> Self {
        Self {
            keep_history: true,
            max_buffer_size: 1000,
        }
    }
}

impl GameClient {
    /// Create a new GameClient with the given card database.
    pub fn new(card_db: Arc<CardDatabase>) -> Self {
        Self {
            card_db,
            engine: None,
            event_buffer: Vec::new(),
            event_history: Vec::new(),
            pre_action_snapshot: None,
            config: GameClientConfig::default(),
        }
    }

    /// Create a new GameClient with custom configuration.
    pub fn with_config(card_db: Arc<CardDatabase>, config: GameClientConfig) -> Self {
        Self {
            card_db,
            engine: None,
            event_buffer: Vec::new(),
            event_history: Vec::new(),
            pre_action_snapshot: None,
            config,
        }
    }

    /// Start a new game with the given decks and seed.
    ///
    /// This initializes the game and emits GameStarted event.
    pub fn start_game(&mut self, deck1: Vec<CardId>, deck2: Vec<CardId>, seed: u64) {
        self.start_game_with_mode(deck1, deck2, seed, GameMode::default())
    }

    /// Start a new game with specific game mode.
    pub fn start_game_with_mode(
        &mut self,
        deck1: Vec<CardId>,
        deck2: Vec<CardId>,
        seed: u64,
        mode: GameMode,
    ) {
        // Clear previous state
        self.event_buffer.clear();
        self.event_history.clear();

        // We need to leak the Arc to get a 'static reference
        // This is safe because the GameClient owns the Arc and will keep it alive
        let db_ref: &'static CardDatabase = unsafe {
            &*Arc::as_ptr(&self.card_db)
        };

        // Create and initialize engine
        let mut engine = GameEngine::new(db_ref);
        engine.start_game_with_mode(deck1.clone(), deck2.clone(), seed, mode);

        // Take initial snapshot
        let snapshot = StateSnapshot::from_state(&engine.state);

        // Emit GameStarted event
        self.emit_event(GameEvent::GameStarted {
            seed,
            mode,
            player1_deck_size: deck1.len(),
            player2_deck_size: deck2.len(),
        });

        // Emit TurnStarted for the first turn
        let active = engine.state.active_player;
        let player_state = &engine.state.players[active.index()];
        self.emit_event(GameEvent::TurnStarted {
            player: active,
            turn_number: engine.state.current_turn,
            max_essence: player_state.max_essence,
            action_points: player_state.action_points,
        });

        self.engine = Some(engine);
        self.pre_action_snapshot = Some(snapshot);
    }

    /// Apply an action to the game.
    ///
    /// This applies the action and emits events for all state changes.
    ///
    /// # Returns
    /// - `Ok(events)` with the list of events generated by this action
    /// - `Err(message)` if the action was illegal or game is over
    pub fn apply_action(&mut self, action: Action) -> Result<Vec<GameEvent>, String> {
        // Collect all data we need before any mutations
        let (before, active_player, turn, after, result, final_turn) = {
            let engine = self.engine.as_mut().ok_or("Game not started")?;

            // Take snapshot before action
            let before = StateSnapshot::from_state(&engine.state);
            let active_player = engine.state.active_player;
            let turn = engine.state.current_turn;

            // Apply the action
            engine.apply_action(action)?;

            // Take snapshot after action
            let after = StateSnapshot::from_state(&engine.state);
            let result = engine.state.result;
            let final_turn = engine.state.current_turn;

            (before, active_player, turn, after, result, final_turn)
        };

        // Emit action taken event
        self.emit_event(GameEvent::ActionTaken {
            player: active_player,
            action,
            turn,
        });

        // Generate diff events
        let diff_events = diff_states(&before, &after);
        for event in diff_events {
            self.emit_event(event);
        }

        // Enhance events based on action type
        self.enhance_events_for_action(action, &before, &after);

        // Check for game end
        if let Some(result) = result {
            self.emit_event(GameEvent::GameEnded {
                result,
                final_turn,
            });
        }

        // Return recent events
        Ok(self.event_buffer.clone())
    }

    /// Enhance events based on the action that was taken.
    ///
    /// This adds context that the diff alone cannot determine
    /// (e.g., which card was played, damage source).
    fn enhance_events_for_action(
        &mut self,
        action: Action,
        before: &StateSnapshot,
        after: &StateSnapshot,
    ) {
        // Collect events to emit, avoiding borrow conflicts
        let mut events_to_emit = Vec::new();

        // Check if game is terminal (need to do this before borrowing engine)
        let is_terminal = self.engine.as_ref().is_none_or(|e| e.state.is_terminal());

        match action {
            Action::PlayCard { hand_index, slot } => {
                let player = before.active_player;
                let before_player = before.player(player);
                let after_player = after.player(player);

                // Check if a creature was spawned at this slot
                if let Some(creature) = after_player.get_creature(slot) {
                    if before_player.get_creature(slot).is_none() {
                        // Look up essence cost from card database
                        let essence_cost = self.card_db.get(CardId(creature.card_id))
                            .map(|c| c.cost)
                            .unwrap_or(0);

                        events_to_emit.push(GameEvent::CardPlayed {
                            player,
                            card_id: CardId(creature.card_id),
                            hand_index: hand_index as usize,
                            target_slot: slot,
                            essence_cost,
                        });
                    }
                }

                // Check if a support was placed
                if let Some(support) = after_player.get_support(slot) {
                    if before_player.get_support(slot).is_none() {
                        let essence_cost = self.card_db.get(CardId(support.card_id))
                            .map(|c| c.cost)
                            .unwrap_or(0);

                        events_to_emit.push(GameEvent::CardPlayed {
                            player,
                            card_id: CardId(support.card_id),
                            hand_index: hand_index as usize,
                            target_slot: slot,
                            essence_cost,
                        });
                    }
                }
            }
            Action::Attack { attacker, defender } => {
                // Combat events already emitted by diff
                // We could add CombatStarted/CombatResolved here for more detail
                let attacker_player = before.active_player;
                let defender_player = attacker_player.opponent();

                if let (Some(_attacker_c), Some(_defender_c)) = (
                    before.player(attacker_player).get_creature(attacker),
                    before.player(defender_player).get_creature(defender),
                ) {
                    events_to_emit.push(GameEvent::CombatStarted {
                        attacker_player,
                        attacker_slot: attacker,
                        defender_player,
                        defender_slot: defender,
                    });
                }
            }
            Action::EndTurn => {
                // Emit TurnEnded and TurnStarted events
                let old_player = before.active_player;
                let new_player = after.active_player;

                events_to_emit.push(GameEvent::TurnEnded {
                    player: old_player,
                    turn_number: before.turn,
                });

                if !is_terminal {
                    let new_player_state = after.player(new_player);
                    events_to_emit.push(GameEvent::TurnStarted {
                        player: new_player,
                        turn_number: after.turn,
                        max_essence: new_player_state.max_essence,
                        action_points: new_player_state.action_points,
                    });
                }
            }
            Action::UseAbility { .. } => {
                // Ability events are complex - would need more tracking
            }
        }

        // Now emit all collected events
        for event in events_to_emit {
            self.emit_event(event);
        }
    }

    /// Get the list of legal actions for the current player.
    pub fn get_legal_actions(&self) -> Vec<Action> {
        match &self.engine {
            Some(engine) => engine.get_legal_actions(),
            None => vec![],
        }
    }

    /// Get the current game state (if game is active).
    pub fn get_state(&self) -> Option<&GameState> {
        self.engine.as_ref().map(|e| &e.state)
    }

    /// Check if the game is over.
    pub fn is_game_over(&self) -> bool {
        self.engine.as_ref().is_none_or(|e| e.is_terminal())
    }

    /// Get the game result (if game is over).
    pub fn get_result(&self) -> Option<GameResult> {
        self.engine.as_ref().and_then(|e| e.state.result)
    }

    /// Get the current player.
    pub fn current_player(&self) -> Option<PlayerId> {
        self.engine.as_ref().map(|e| e.state.active_player)
    }

    /// Get the current turn number.
    pub fn turn_number(&self) -> u16 {
        self.engine.as_ref().map_or(0, |e| e.state.current_turn)
    }

    /// Get the current turn number (alias for turn_number).
    pub fn get_turn(&self) -> u16 {
        self.turn_number()
    }

    /// Get the active player (alias for current_player).
    pub fn get_active_player(&self) -> PlayerId {
        self.current_player().unwrap_or(PlayerId(0))
    }

    /// Apply an action by its index in the action space (0-255).
    pub fn apply_action_by_index(&mut self, index: u8) -> Result<Vec<GameEvent>, String> {
        let action = Action::from_index(index)
            .ok_or_else(|| format!("Invalid action index: {}", index))?;
        self.apply_action(action)
    }

    /// Get the legal action mask (256 floats, 1.0 = legal, 0.0 = illegal).
    pub fn get_legal_action_mask(&self) -> [f32; 256] {
        match &self.engine {
            Some(engine) => {
                legal_mask_to_tensor(&crate::core::legal::legal_action_mask(&engine.state, engine.card_db()))
            }
            None => [0.0; 256],
        }
    }

    /// Get the state tensor (326 floats) for neural network input.
    pub fn get_state_tensor(&self) -> [f32; crate::tensor::STATE_TENSOR_SIZE] {
        match &self.engine {
            Some(engine) => state_to_tensor(&engine.state),
            None => [0.0; crate::tensor::STATE_TENSOR_SIZE],
        }
    }

    /// Drain the event buffer (returns and clears buffered events).
    pub fn drain_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.event_buffer)
    }

    /// Get the complete event history.
    pub fn event_history(&self) -> &[GameEvent] {
        &self.event_history
    }

    /// Clear the event history (but keep game state).
    pub fn clear_history(&mut self) {
        self.event_history.clear();
    }

    /// Get a hint from an AI bot for the current position.
    ///
    /// This is useful for showing suggested moves to human players.
    pub fn get_ai_hint(&self, bot: &mut dyn Bot) -> Option<Action> {
        let engine = self.engine.as_ref()?;

        if engine.is_terminal() {
            return None;
        }

        let state_tensor = state_to_tensor(&engine.state);
        let legal_mask =
            legal_mask_to_tensor(&crate::core::legal::legal_action_mask(&engine.state, engine.card_db()));
        let legal_actions = engine.get_legal_actions();

        Some(bot.select_action(&state_tensor, &legal_mask, &legal_actions))
    }

    /// Get the card database.
    pub fn card_db(&self) -> &CardDatabase {
        &self.card_db
    }

    /// Emit an event (adds to buffer and history).
    fn emit_event(&mut self, event: GameEvent) {
        self.event_buffer.push(event.clone());

        if self.config.keep_history {
            self.event_history.push(event);
        }

        // Auto-drain if buffer is too large
        if self.event_buffer.len() > self.config.max_buffer_size {
            // In a real application, this would notify listeners
            self.event_buffer.clear();
        }
    }
}

/// Builder for GameClient with fluent configuration.
pub struct GameClientBuilder {
    card_db: Arc<CardDatabase>,
    config: GameClientConfig,
}

impl GameClientBuilder {
    /// Create a new builder with an empty card database.
    ///
    /// Use `with_card_db()` to set a real database.
    pub fn new() -> Self {
        Self {
            card_db: Arc::new(CardDatabase::empty()),
            config: GameClientConfig::default(),
        }
    }

    /// Create a new builder with the given card database.
    pub fn with_card_db(mut self, card_db: Arc<CardDatabase>) -> Self {
        self.card_db = card_db;
        self
    }

    /// Set whether to keep event history.
    pub fn keep_history(mut self, keep: bool) -> Self {
        self.config.keep_history = keep;
        self
    }

    /// Set maximum buffer size before auto-drain.
    pub fn max_buffer_size(mut self, size: usize) -> Self {
        self.config.max_buffer_size = size;
        self
    }

    /// Build the GameClient.
    pub fn build(self) -> GameClient {
        GameClient::with_config(self.card_db, self.config)
    }
}

impl Default for GameClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}
