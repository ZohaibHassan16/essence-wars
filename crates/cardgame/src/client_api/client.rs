//! GameClient - High-level game controller for clients.
//!
//! GameClient wraps GameEngine and provides:
//! - Event emission for reactive UIs
//! - Unified handling of human and AI players
//! - Event log for replay and debugging

use std::sync::Arc;

use crate::bots::{AnalyzableBot, Bot, BotDecision, IntrospectionConfig};
use crate::client_api::diff::{diff_states, StateSnapshot};
use crate::client_api::events::{GameEvent, KeywordType};
use crate::core::keywords::Keywords;
use crate::core::actions::Action;
use crate::core::cards::CardDatabase;
use crate::core::engine::GameEngine;
use crate::core::state::{GameMode, GameResult, GameState};
use crate::core::types::{CardId, PlayerId};
use crate::decks::DeckDefinition;
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

    /// Start a new game with the given deck definitions and seed.
    ///
    /// This is the primary API for starting games. Each deck definition includes
    /// the commander, so commanders are always used.
    pub fn start_game(&mut self, deck1: &DeckDefinition, deck2: &DeckDefinition, seed: u64) {
        self.start_game_with_mode(deck1, deck2, seed, GameMode::default())
    }

    /// Start a new game with specific game mode.
    pub fn start_game_with_mode(
        &mut self,
        deck1: &DeckDefinition,
        deck2: &DeckDefinition,
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
        engine
            .start_game_with_mode(deck1, deck2, seed, mode)
            .expect("Failed to start game - commander not found in card database");

        // Take initial snapshot
        let snapshot = StateSnapshot::from_state(&engine.state);

        // Emit GameStarted event
        self.emit_event(GameEvent::GameStarted {
            seed,
            mode,
            player1_deck_size: deck1.cards.len(),
            player2_deck_size: deck2.cards.len(),
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

    /// Low-level game initialization with raw card IDs and commanders.
    ///
    /// Use this when you have raw card IDs and commander IDs (e.g., in tests
    /// or when constructing games programmatically without DeckDefinitions).
    ///
    /// # Panics
    /// Panics if either commander ID is not found in the card database.
    pub fn start_game_raw(
        &mut self,
        deck1: Vec<CardId>,
        deck2: Vec<CardId>,
        commander1: CardId,
        commander2: CardId,
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

        // Create and initialize engine with commanders
        let mut engine = GameEngine::new(db_ref);
        engine
            .start_game_raw(deck1.clone(), deck2.clone(), commander1, commander2, seed, mode)
            .expect("Failed to start game - commander not found in card database");

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

        // Return and drain recent events
        Ok(std::mem::take(&mut self.event_buffer))
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
                let attacker_player = before.active_player;
                let defender_player = attacker_player.opponent();

                // Get creature snapshots before and after combat
                let before_attacker = before.player(attacker_player).get_creature(attacker);
                let before_defender = before.player(defender_player).get_creature(defender);
                let after_attacker = after.player(attacker_player).get_creature(attacker);
                let after_defender = after.player(defender_player).get_creature(defender);

                if let (Some(atk_before), Some(def_before)) = (before_attacker, before_defender) {
                    // Emit CombatStarted
                    events_to_emit.push(GameEvent::CombatStarted {
                        attacker_player,
                        attacker_slot: attacker,
                        defender_player,
                        defender_slot: defender,
                    });

                    let atk_keywords = Keywords(atk_before.keywords);
                    let def_keywords = Keywords(def_before.keywords);

                    // Calculate damage dealt (for keyword detection)
                    let atk_damage = atk_before.attack.max(0) as u8;
                    let def_damage = def_before.attack.max(0) as u8;

                    // Check for defender death
                    let defender_died = after_defender.is_none();
                    // Check for attacker death
                    let attacker_died = after_attacker.is_none();

                    // Calculate actual damage taken by defender
                    let defender_damage_taken = if defender_died {
                        def_before.health as u8
                    } else if let Some(def_after) = after_defender {
                        (def_before.health - def_after.health).max(0) as u8
                    } else {
                        0
                    };

                    // Calculate actual damage taken by attacker
                    let attacker_damage_taken = if attacker_died {
                        atk_before.health as u8
                    } else if let Some(atk_after) = after_attacker {
                        (atk_before.health - atk_after.health).max(0) as u8
                    } else {
                        0
                    };

                    // ==== KEYWORD ACTIVATIONS ====

                    // Shield: Defender had Shield and lost it (blocked damage)
                    if def_keywords.has_shield() {
                        if let Some(def_after) = after_defender {
                            let after_keywords = Keywords(def_after.keywords);
                            if !after_keywords.has_shield() {
                                // Shield blocked the attack (blocked all damage)
                                events_to_emit.push(GameEvent::KeywordActivated {
                                    player: defender_player,
                                    slot: defender,
                                    keyword: KeywordType::Shield,
                                    value: atk_damage, // Damage that was blocked
                                    target_player: None,
                                    target_slot: None,
                                });
                            }
                        }
                    }

                    // Attacker's Shield consumed if it existed
                    if atk_keywords.has_shield() {
                        if let Some(atk_after) = after_attacker {
                            let after_keywords = Keywords(atk_after.keywords);
                            if !after_keywords.has_shield() && def_damage > 0 {
                                events_to_emit.push(GameEvent::KeywordActivated {
                                    player: attacker_player,
                                    slot: attacker,
                                    keyword: KeywordType::Shield,
                                    value: def_damage, // Damage that was blocked
                                    target_player: None,
                                    target_slot: None,
                                });
                            }
                        }
                    }

                    // Ranged: Attacker didn't take counter-damage
                    if atk_keywords.has_ranged() && attacker_damage_taken == 0 && !attacker_died {
                        events_to_emit.push(GameEvent::KeywordActivated {
                            player: attacker_player,
                            slot: attacker,
                            keyword: KeywordType::Ranged,
                            value: 0,
                            target_player: None,
                            target_slot: None,
                        });
                    }

                    // Quick: Attacker struck first and killed defender before counter
                    if atk_keywords.has_quick() && defender_died && !attacker_died && attacker_damage_taken == 0 {
                        events_to_emit.push(GameEvent::KeywordActivated {
                            player: attacker_player,
                            slot: attacker,
                            keyword: KeywordType::Quick,
                            value: atk_damage,
                            target_player: Some(defender_player),
                            target_slot: Some(defender),
                        });
                    }

                    // Lethal: Attacker killed defender (regardless of damage amount)
                    if atk_keywords.has_lethal() && defender_died {
                        events_to_emit.push(GameEvent::KeywordActivated {
                            player: attacker_player,
                            slot: attacker,
                            keyword: KeywordType::Lethal,
                            value: 1, // Lethal kill
                            target_player: Some(defender_player),
                            target_slot: Some(defender),
                        });
                    }

                    // Lifesteal: Check if attacker's controller gained life
                    let before_atk_player_life = before.player(attacker_player).life;
                    let after_atk_player_life = after.player(attacker_player).life;
                    if atk_keywords.has_lifesteal() && after_atk_player_life > before_atk_player_life {
                        let healed = (after_atk_player_life - before_atk_player_life) as u8;
                        events_to_emit.push(GameEvent::KeywordActivated {
                            player: attacker_player,
                            slot: attacker,
                            keyword: KeywordType::Lifesteal,
                            value: healed,
                            target_player: Some(attacker_player),
                            target_slot: None,
                        });
                    }

                    // Piercing: Check if overflow damage hit defender's commander
                    let before_def_player_life = before.player(defender_player).life;
                    let after_def_player_life = after.player(defender_player).life;
                    if atk_keywords.has_piercing() && defender_died && after_def_player_life < before_def_player_life {
                        let overflow = (before_def_player_life - after_def_player_life) as u8;
                        events_to_emit.push(GameEvent::KeywordActivated {
                            player: attacker_player,
                            slot: attacker,
                            keyword: KeywordType::Piercing,
                            value: overflow,
                            target_player: Some(defender_player),
                            target_slot: None,
                        });
                    }

                    // Guard: Defender had Guard (forced the attack)
                    if def_keywords.has_guard() {
                        events_to_emit.push(GameEvent::KeywordActivated {
                            player: defender_player,
                            slot: defender,
                            keyword: KeywordType::Guard,
                            value: 1,
                            target_player: None,
                            target_slot: None,
                        });
                    }

                    // Fortify: Defender reduced incoming damage
                    if def_keywords.has_fortify() && defender_damage_taken > 0 {
                        // Fortify reduces damage by 1 (min 1)
                        events_to_emit.push(GameEvent::KeywordActivated {
                            player: defender_player,
                            slot: defender,
                            keyword: KeywordType::Fortify,
                            value: 1, // Damage reduced
                            target_player: None,
                            target_slot: None,
                        });
                    }

                    // Charge: Attacker gained bonus attack
                    if atk_keywords.has_charge() {
                        events_to_emit.push(GameEvent::KeywordActivated {
                            player: attacker_player,
                            slot: attacker,
                            keyword: KeywordType::Charge,
                            value: 2, // Charge gives +2 attack
                            target_player: None,
                            target_slot: None,
                        });
                    }

                    // Frenzy: Attacker gains +1 attack after attacking
                    if atk_keywords.has_frenzy() {
                        events_to_emit.push(GameEvent::KeywordActivated {
                            player: attacker_player,
                            slot: attacker,
                            keyword: KeywordType::Frenzy,
                            value: 1, // +1 attack gained
                            target_player: None,
                            target_slot: None,
                        });
                    }

                    // Emit CombatResolved
                    events_to_emit.push(GameEvent::CombatResolved {
                        attacker_damage_dealt: defender_damage_taken,
                        defender_damage_dealt: attacker_damage_taken,
                        attacker_died,
                        defender_died,
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
            Action::CommanderInsight => {
                // Commander's Insight draws a card - card draw events are handled by diff
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
    /// Note: This uses the basic `select_action()` method which may not work
    /// optimally for bots that need engine simulation (like MCTS/Greedy).
    /// Use `select_bot_action()` for full bot functionality.
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

    /// Select an action using a bot with full engine access.
    ///
    /// This method gives bots access to the game engine, enabling simulation-based
    /// bots like MCTS and GreedyBot to use their full capabilities (tree search,
    /// action evaluation via simulation, etc.).
    ///
    /// Returns None if the game is not active or is already terminal.
    pub fn select_bot_action(&self, bot: &mut dyn Bot) -> Option<Action> {
        let engine = self.engine.as_ref()?;

        if engine.is_terminal() {
            return None;
        }

        Some(bot.select_action_with_engine(engine))
    }

    /// Select an action using an analyzable bot with introspection data.
    ///
    /// This method returns both the selected action and introspection data
    /// (policy outputs, MCTS tree snapshots, etc.) for visualization.
    ///
    /// Returns None if the game is not active or is already terminal.
    pub fn select_bot_action_with_introspection(
        &self,
        bot: &mut dyn AnalyzableBot,
        config: &IntrospectionConfig,
    ) -> Option<(Action, Option<BotDecision>)> {
        let engine = self.engine.as_ref()?;

        if engine.is_terminal() {
            return None;
        }

        Some(bot.select_action_with_introspection(engine, config))
    }

    /// Get the card database.
    pub fn card_db(&self) -> &CardDatabase {
        &self.card_db
    }

    /// Get read-only access to the game engine for introspection.
    ///
    /// This is useful for AI visualization tools that need to evaluate
    /// positions without modifying game state.
    pub fn engine(&self) -> Option<&GameEngine<'static>> {
        self.engine.as_ref()
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
