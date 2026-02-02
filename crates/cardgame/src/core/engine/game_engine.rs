//! Game engine that manages turn flow and action execution.
//!
//! The GameEngine is a thin facade that delegates to specialized modules:
//! - `init`: Game initialization
//! - `turn`: Turn start/end logic
//! - `actions`: Action execution (play_card, attack, ability)
//! - `victory`: Victory condition checking

use crate::core::actions::Action;
use crate::core::cards::CardDatabase;
use crate::core::config::insight;
use crate::core::legal::legal_actions;
use crate::core::state::{GameMode, GameResult, GameState};
use crate::core::tracing::{CombatTracer, EffectTracer};
use crate::core::types::{PlayerId, Slot};
use crate::decks::DeckDefinition;

use super::actions::{execute_attack, execute_attack_with_tracers, execute_play_card, execute_use_ability, ActionContext};
use super::effect_queue::EffectQueue;
use super::init::{draw_card, initialize_game_raw, GameInitError};
use super::passive::remove_support_passives_from_all_creatures;
use super::turn;
use super::victory;

/// Game engine that manages turn flow and action execution.
///
/// The engine is a thin facade that delegates to specialized modules for
/// initialization, turn management, action execution, and victory checking.
pub struct GameEngine<'a> {
    pub state: GameState,
    card_db: &'a CardDatabase,
    effect_queue: EffectQueue,
}

impl<'a> GameEngine<'a> {
    /// Create a new game engine with the given card database
    pub fn new(card_db: &'a CardDatabase) -> Self {
        Self {
            state: GameState::new(),
            card_db,
            effect_queue: EffectQueue::new(),
        }
    }

    /// Initialize a new game with the given deck definitions.
    ///
    /// This is the primary API for starting games. Each deck definition includes
    /// the commander, so commanders are always used. Uses Attrition mode by default.
    ///
    /// # Errors
    /// Returns `GameInitError::CommanderNotFound` if either commander is not found in the card database.
    pub fn start_game(&mut self, deck1: &DeckDefinition, deck2: &DeckDefinition, seed: u64) -> Result<(), GameInitError> {
        self.start_game_with_mode(deck1, deck2, seed, GameMode::default())
    }

    /// Initialize a new game with the given deck definitions and game mode.
    ///
    /// # Errors
    /// Returns `GameInitError::CommanderNotFound` if either commander is not found in the card database.
    pub fn start_game_with_mode(
        &mut self,
        deck1: &DeckDefinition,
        deck2: &DeckDefinition,
        seed: u64,
        mode: GameMode,
    ) -> Result<(), GameInitError> {
        self.start_game_raw(
            deck1.to_card_ids(),
            deck2.to_card_ids(),
            deck1.commander_id(),
            deck2.commander_id(),
            seed,
            mode,
        )
    }

    /// Low-level game initialization with raw components.
    ///
    /// Use this when you have raw card IDs and commander IDs (e.g., in tests
    /// or when constructing games programmatically without DeckDefinitions).
    /// Commanders are mandatory - Essence Wars is built around commanders.
    ///
    /// # Errors
    /// Returns `GameInitError::CommanderNotFound` if either commander ID is not found in the card database.
    pub fn start_game_raw(
        &mut self,
        deck1: Vec<crate::core::types::CardId>,
        deck2: Vec<crate::core::types::CardId>,
        commander1: crate::core::types::CardId,
        commander2: crate::core::types::CardId,
        seed: u64,
        mode: GameMode,
    ) -> Result<(), GameInitError> {
        // Delegate to init module
        initialize_game_raw(
            &mut self.state,
            self.card_db,
            deck1,
            deck2,
            commander1,
            commander2,
            seed,
            mode,
        )?;

        // Start Player 1's first turn
        turn::start_turn(&mut self.state, self.card_db, &mut self.effect_queue);

        // Validate initial state in debug builds
        self.state.debug_validate();

        Ok(())
    }

    /// Get the commander definition for a player.
    /// Returns None if the player has no commander set or if the commander isn't found.
    pub fn get_commander_def(&self, player: PlayerId) -> Option<&crate::core::cards::CommanderDefinition> {
        let commander_id = self.state.get_commander(player)?;
        self.card_db.get_commander(commander_id)
    }

    /// Draw a card for the specified player.
    /// If deck is empty, nothing happens (no fatigue damage in this game).
    /// If hand is full (10 cards), the drawn card is discarded (overdraw).
    fn draw_card_internal(&mut self, player: PlayerId) {
        draw_card(&mut self.state, player);
    }

    /// End the current player's turn.
    ///
    /// Delegates to the turn module for:
    /// - Support durability processing
    /// - Ephemeral death processing
    /// - Frenzy stack reset
    /// - Player switch
    /// - Next player's turn start
    fn end_turn(&mut self) {
        turn::end_turn(&mut self.state, self.card_db, &mut self.effect_queue);
    }

    /// Apply an action to the game state.
    /// Returns Ok(()) on success, Err with description on illegal action.
    pub fn apply_action(&mut self, action: Action) -> Result<(), String> {
        // Check if game is over
        if self.state.is_terminal() {
            return Err("Game is already over".to_string());
        }

        // Validate action is legal
        let legal = legal_actions(&self.state, self.card_db);
        if !legal.contains(&action) {
            return Err(format!("Illegal action: {:?}", action));
        }

        match action {
            Action::PlayCard { hand_index, slot } => {
                let mut ctx = ActionContext::new(
                    &mut self.state,
                    self.card_db,
                    &mut self.effect_queue,
                    None,
                );
                execute_play_card(&mut ctx, hand_index as usize, slot)?;
            }
            Action::Attack { attacker, defender } => {
                let mut ctx = ActionContext::new(
                    &mut self.state,
                    self.card_db,
                    &mut self.effect_queue,
                    None,
                );
                execute_attack(&mut ctx, attacker, defender)?;
            }
            Action::UseAbility {
                slot,
                ability_index,
                target,
            } => {
                let mut ctx = ActionContext::new(
                    &mut self.state,
                    self.card_db,
                    &mut self.effect_queue,
                    None,
                );
                execute_use_ability(&mut ctx, slot, ability_index, target)?;
            }
            Action::CommanderInsight => {
                self.execute_commander_insight()?;
            }
            Action::EndTurn => {
                self.end_turn();
            }
        }

        // Check for victory after each action
        victory::check_victory_conditions(&mut self.state);

        // Validate state invariants in debug builds
        self.state.debug_validate();

        Ok(())
    }

    /// Apply an action with optional tracers for debugging.
    ///
    /// This is the same as `apply_action` but allows passing combat and effect
    /// tracers for step-by-step debugging of combat resolution and effect processing.
    pub fn apply_action_with_tracers(
        &mut self,
        action: Action,
        combat_tracer: Option<&mut CombatTracer>,
        mut effect_tracer: Option<&mut EffectTracer>,
    ) -> Result<(), String> {
        // Check if game is over
        if self.state.is_terminal() {
            return Err("Game is already over".to_string());
        }

        // Validate action is legal
        let legal = legal_actions(&self.state, self.card_db);
        if !legal.contains(&action) {
            return Err(format!("Illegal action: {:?}", action));
        }

        match action {
            Action::PlayCard { hand_index, slot } => {
                let mut ctx = ActionContext::new(
                    &mut self.state,
                    self.card_db,
                    &mut self.effect_queue,
                    effect_tracer.as_deref_mut(),
                );
                execute_play_card(&mut ctx, hand_index as usize, slot)?;
            }
            Action::Attack { attacker, defender } => {
                let mut ctx = ActionContext::new(
                    &mut self.state,
                    self.card_db,
                    &mut self.effect_queue,
                    effect_tracer.as_deref_mut(),
                );
                execute_attack_with_tracers(&mut ctx, attacker, defender, combat_tracer)?;
            }
            Action::UseAbility {
                slot,
                ability_index,
                target,
            } => {
                let mut ctx = ActionContext::new(
                    &mut self.state,
                    self.card_db,
                    &mut self.effect_queue,
                    effect_tracer,
                );
                execute_use_ability(&mut ctx, slot, ability_index, target)?;
            }
            Action::CommanderInsight => {
                self.execute_commander_insight()?;
            }
            Action::EndTurn => {
                self.end_turn();
            }
        }

        // Check for victory after each action
        victory::check_victory_conditions(&mut self.state);

        // Validate state invariants in debug builds
        self.state.debug_validate();

        Ok(())
    }

    /// Execute a PlayCard action directly (for testing convenience).
    ///
    /// This is a convenience method that bypasses action validation. Use
    /// `apply_action` for normal gameplay which includes legality checking.
    pub fn execute_play_card(&mut self, hand_index: usize, slot: Slot) -> Result<(), String> {
        let mut ctx = ActionContext::new(
            &mut self.state,
            self.card_db,
            &mut self.effect_queue,
            None,
        );
        execute_play_card(&mut ctx, hand_index, slot)
    }

    /// Check if a player has lost due to life reaching 0.
    /// If both players reach 0 life simultaneously, the game is a draw.
    pub fn check_life_victory(&mut self) {
        victory::check_life_victory(&mut self.state);
    }

    /// Check if a player has won via Essence Extraction (EssenceWar mode only).
    /// In EssenceWar, first player to extract 50 essence (cumulative face damage) wins.
    pub fn check_essence_extraction_victory(&mut self) {
        victory::check_essence_extraction_victory(&mut self.state);
    }

    /// Execute Commander's Insight action.
    ///
    /// Commander's Insight is a catch-up mechanic that allows struggling players
    /// to draw a card by paying essence (no AP cost). Prerequisites are checked
    /// by the legal action generator.
    ///
    /// Effect: Pay 4 essence, draw 1 card, mark as used for this turn.
    fn execute_commander_insight(&mut self) -> Result<(), String> {
        let current_player = self.state.active_player;
        let player_state = &mut self.state.players[current_player.index()];

        // Deduct essence cost
        if player_state.current_essence < insight::ESSENCE_COST {
            return Err("Not enough essence for Commander's Insight".to_string());
        }
        player_state.current_essence -= insight::ESSENCE_COST;

        // Mark as used this turn (prevents multiple uses)
        player_state.used_commander_insight = true;

        // Draw a card
        self.draw_card_internal(current_player);

        Ok(())
    }

    /// Remove a support from the board, properly cleaning up its passive effects.
    ///
    /// This should be used instead of directly manipulating state.supports
    /// to ensure passive effects are properly removed from creatures.
    pub fn remove_support(&mut self, player: PlayerId, slot: Slot) {
        // Find the support's card_id
        let card_id = self.state.players[player.index()]
            .supports
            .iter()
            .find(|s| s.slot == slot)
            .map(|s| s.card_id);

        if let Some(card_id) = card_id {
            // Remove passive effects from creatures
            remove_support_passives_from_all_creatures(
                card_id,
                &mut self.state.players[player.index()].creatures,
                self.card_db,
            );

            // Remove the support from the board
            self.state.players[player.index()]
                .supports
                .retain(|s| s.slot != slot);
        }
    }

    /// Check if game is over.
    pub fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }

    /// Get winner (None if game not over, Some(player) if won).
    pub fn winner(&self) -> Option<PlayerId> {
        match &self.state.result {
            Some(GameResult::Win { winner, .. }) => Some(*winner),
            Some(GameResult::Draw) => None,
            None => None,
        }
    }

    /// Get the card database reference.
    pub fn card_db(&self) -> &CardDatabase {
        self.card_db
    }

    // =========================================================================
    // GAME INTERFACE METHODS (Task 6.2)
    // =========================================================================
    // These methods provide a clean interface for AI/external code to interact
    // with the game, particularly useful for MCTS and neural network training.

    /// Get the current state as a tensor for neural network input.
    ///
    /// Returns a fixed-size array of f32 values encoding all relevant game state.
    pub fn get_state_tensor(&self) -> [f32; crate::tensor::STATE_TENSOR_SIZE] {
        crate::tensor::state_to_tensor(&self.state)
    }

    /// Get legal action mask (256 bools as f32).
    ///
    /// Returns 1.0 for legal actions and 0.0 for illegal actions.
    /// This is useful for masking neural network outputs.
    pub fn get_legal_action_mask(&self) -> [f32; 256] {
        crate::tensor::legal_mask_to_tensor(&crate::legal::legal_action_mask(&self.state, self.card_db))
    }

    /// Get list of legal actions.
    ///
    /// Returns a Vec of all currently legal actions for the active player.
    pub fn get_legal_actions(&self) -> Vec<Action> {
        legal_actions(&self.state, self.card_db).to_vec()
    }

    /// Apply an action by index (from neural network output).
    ///
    /// Converts the neural network action index (0-255) to an Action and applies it.
    /// Returns an error if the index is invalid or the action is illegal.
    pub fn apply_action_by_index(&mut self, index: u8) -> Result<(), String> {
        let action = Action::from_index(index)
            .ok_or_else(|| format!("Invalid action index: {}", index))?;
        self.apply_action(action)
    }

    /// Check if game is in terminal state.
    ///
    /// This is an alias for is_terminal() for interface consistency.
    pub fn is_game_over(&self) -> bool {
        self.is_terminal()
    }

    /// Get reward for the specified player.
    ///
    /// Returns:
    /// - 1.0 for win
    /// - -1.0 for loss
    /// - 0.0 for ongoing/draw
    pub fn get_reward(&self, player: PlayerId) -> f32 {
        match self.winner() {
            None => 0.0,
            Some(winner) if winner == player => 1.0,
            Some(_) => -1.0,
        }
    }

    /// Clone the game state for MCTS tree search.
    ///
    /// This is efficient because GameState is designed for fast cloning
    /// (uses ArrayVec for stack allocation, no heap allocations for most data).
    pub fn clone_state(&self) -> GameState {
        self.state.clone()
    }

    /// Create a new engine with a cloned state (for MCTS).
    ///
    /// This creates an independent copy of the game that can be modified
    /// without affecting the original. Useful for tree search algorithms.
    pub fn fork(&self) -> GameEngine<'a> {
        GameEngine {
            state: self.state.clone(),
            card_db: self.card_db,
            effect_queue: EffectQueue::new(),
        }
    }

    /// Get current player.
    pub fn current_player(&self) -> PlayerId {
        self.state.active_player
    }

    /// Get turn number.
    pub fn turn_number(&self) -> u16 {
        self.state.current_turn
    }
}
