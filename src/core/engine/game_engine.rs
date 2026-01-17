//! Game engine that manages turn flow and action execution.
//!
//! The GameEngine orchestrates game flow, processes actions, and manages
//! the game loop including turn structure and priority.

use crate::core::actions::Action;
use crate::core::cards::{CardDatabase, CardType};
use crate::core::state::CardInstance;
use crate::core::combat;
use crate::core::config::{game, player};
use crate::core::effects::{EffectSource, Trigger};
use crate::core::legal::legal_actions;
use crate::core::state::{Creature, CreatureStatus, GameMode, GamePhase, GameResult, GameState, Support, WinReason};
use crate::core::tracing::{CombatTracer, EffectTracer};
use crate::core::types::{CardId, PlayerId, Slot};

use super::effect_queue::EffectQueue;
use super::effect_convert::{resolve_spell_target, effect_def_to_effect_with_target, effect_def_to_triggered_effect};
use super::passive::{
    apply_all_support_passives_to_creature,
    apply_support_passives_to_all_creatures,
    remove_support_passives_from_all_creatures,
    support_effect_def_to_effect,
};
use super::seeded_shuffle;

/// Game engine that manages turn flow and action execution
pub struct GameEngine<'a> {
    pub state: GameState,
    card_db: &'a CardDatabase,
}

impl<'a> GameEngine<'a> {
    /// Create a new game engine with the given card database
    pub fn new(card_db: &'a CardDatabase) -> Self {
        Self {
            state: GameState::new(),
            card_db,
        }
    }

    /// Initialize a new game with the given decks.
    /// Shuffles decks using the provided seed, draws initial hands,
    /// and starts Player 1's first turn.
    pub fn start_game(&mut self, deck1: Vec<CardId>, deck2: Vec<CardId>, seed: u64) {
        // Reset state
        self.state = GameState::new();
        self.state.rng_state = seed;

        // Set up player 1's deck
        let mut deck1_cards: Vec<CardInstance> = deck1.into_iter().map(CardInstance::new).collect();
        seeded_shuffle(&mut deck1_cards, seed);
        for card in deck1_cards {
            if self.state.players[0].deck.len() < game::MAX_DECK_SIZE {
                self.state.players[0].deck.push(card);
            }
        }

        // Set up player 2's deck (use a different seed derived from the original)
        let seed2 = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut deck2_cards: Vec<CardInstance> = deck2.into_iter().map(CardInstance::new).collect();
        seeded_shuffle(&mut deck2_cards, seed2);
        for card in deck2_cards {
            if self.state.players[1].deck.len() < game::MAX_DECK_SIZE {
                self.state.players[1].deck.push(card);
            }
        }

        // Draw initial hands
        for _ in 0..player::STARTING_HAND_SIZE {
            self.draw_card(PlayerId::PLAYER_ONE);
            self.draw_card(PlayerId::PLAYER_TWO);
        }

        // P2 draws extra cards to compensate for First Player Advantage
        // Note: Currently P2_BONUS_CARDS is 0, using essence instead for FPA compensation
        #[allow(clippy::reversed_empty_ranges)]
        for _ in 0..player::P2_BONUS_CARDS {
            self.draw_card(PlayerId::PLAYER_TWO);
        }

        // Set up initial essence (P2 starts higher to compensate for FPA)
        // Note: start_turn() will add +1, so we set to (target - 1)
        self.state.players[0].max_essence = player::STARTING_ESSENCE_P1 - 1;
        self.state.players[1].max_essence = player::STARTING_ESSENCE_P2 - 1;

        // Set up initial game state
        self.state.current_turn = 0; // Will be incremented to 1 in start_turn
        self.state.active_player = PlayerId::PLAYER_ONE;
        self.state.phase = GamePhase::Main;

        // Start Player 1's first turn
        self.start_turn();

        // Validate initial state in debug builds
        self.state.debug_validate();
    }

    /// Initialize a new game with the given decks and game mode.
    /// Same as `start_game` but allows specifying the game mode.
    pub fn start_game_with_mode(
        &mut self,
        deck1: Vec<CardId>,
        deck2: Vec<CardId>,
        seed: u64,
        mode: GameMode,
    ) {
        self.start_game(deck1, deck2, seed);
        self.state.game_mode = mode;
    }

    /// Draw a card for the specified player.
    /// If deck is empty, nothing happens (no fatigue damage in this game).
    /// If hand is full (10 cards), the drawn card is discarded (overdraw).
    fn draw_card(&mut self, player: PlayerId) {
        let player_state = &mut self.state.players[player.index()];

        // Try to draw from deck
        if let Some(card) = player_state.deck.pop() {
            // Add to hand if not full
            if !player_state.is_hand_full() {
                player_state.hand.push(card);
            }
            // If hand is full, card is simply discarded (overdraw)
        }
    }

    /// Start the current player's turn.
    /// - Increment turn counter
    /// - Increase max essence by 1 (cap at 10)
    /// - Refill current essence to max
    /// - Draw a card
    /// - Restore AP to 3
    /// - Reset creature attack flags (clear exhausted status)
    fn start_turn(&mut self) {
        // Increment turn counter
        self.state.current_turn += 1;

        // Check for turn limit win condition
        if self.state.current_turn > game::TURN_LIMIT as u16 {
            self.check_turn_limit_victory();
            return;
        }

        let current_player = self.state.active_player;
        let player_state = &mut self.state.players[current_player.index()];

        // Increase max essence by 1 (capped at MAX_ESSENCE)
        if player_state.max_essence < player::MAX_ESSENCE {
            player_state.max_essence += player::ESSENCE_PER_TURN;
        }

        // Refill current essence to max
        player_state.current_essence = player_state.max_essence;

        // Draw a card
        self.draw_card(current_player);

        // Restore AP to 3
        self.state.players[current_player.index()].action_points = player::AP_PER_TURN;

        // Reset creature attack flags (clear exhausted status)
        // Creatures that survived a full round can now attack
        for creature in &mut self.state.players[current_player.index()].creatures {
            creature.status.set_exhausted(false);
        }

        // Process Regenerate - creatures with this keyword heal 2 HP at start of turn
        self.process_regenerate_healing(current_player);

        // Process StartOfTurn triggered effects for supports
        self.process_support_start_of_turn_triggers(current_player);
    }

    /// Process StartOfTurn triggered effects for a player's supports.
    fn process_support_start_of_turn_triggers(&mut self, player: PlayerId) {
        // Collect support info to avoid borrow conflicts
        let support_info: Vec<(Slot, CardId)> = self.state.players[player.index()]
            .supports
            .iter()
            .map(|s| (s.slot, s.card_id))
            .collect();

        let mut effect_queue = EffectQueue::new();

        for (slot, card_id) in support_info {
            if let Some(card_def) = self.card_db.get(card_id) {
                if let CardType::Support { triggered_effects, .. } = &card_def.card_type {
                    for ability in triggered_effects {
                        if ability.trigger == Trigger::StartOfTurn {
                            let source = EffectSource::Support { owner: player, slot };
                            for effect_def in &ability.effects {
                                // Use support-specific effect conversion
                                if let Some(effect) = support_effect_def_to_effect(
                                    effect_def,
                                    player,
                                    ability,
                                ) {
                                    effect_queue.push(effect, source);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Process all queued effects
        effect_queue.process_all(&mut self.state, self.card_db);
    }

    /// Process Regenerate keyword - heal creatures by 2 HP at start of turn.
    fn process_regenerate_healing(&mut self, player: PlayerId) {
        const REGEN_AMOUNT: i8 = 2;

        for creature in &mut self.state.players[player.index()].creatures {
            // Skip dead/dying creatures (health <= 0) - they'll be removed by death processing
            if creature.current_health <= 0 {
                continue;
            }
            
            if creature.keywords.has_regenerate() && creature.current_health < creature.max_health {
                // Use saturating add to prevent overflow from edge cases
                creature.current_health =
                    creature.current_health.saturating_add(REGEN_AMOUNT).min(creature.max_health);
            }
        }
    }

    /// Process Ephemeral keyword - creatures with this keyword die at end of turn.
    fn process_ephemeral_deaths(&mut self, player: PlayerId) {
        use crate::core::effects::{EffectSource, EffectTarget, Trigger};
        use crate::core::engine::effect_queue::EffectQueue;
        use crate::core::cards::CardType;

        // Collect slots of ephemeral creatures to process
        let ephemeral_slots: Vec<Slot> = self.state.players[player.index()]
            .creatures
            .iter()
            .filter(|c| c.keywords.has_ephemeral())
            .map(|c| c.slot)
            .collect();

        if ephemeral_slots.is_empty() {
            return;
        }

        let mut effect_queue = EffectQueue::new();

        // Queue OnDeath triggers for each ephemeral creature before removing them
        for slot in &ephemeral_slots {
            if let Some(creature) = self.state.players[player.index()].get_creature(*slot) {
                let card_id = creature.card_id;

                // Check for OnDeath triggers
                if let Some(card_def) = self.card_db.get(card_id) {
                    if let CardType::Creature { abilities, .. } = &card_def.card_type {
                        for ability in abilities {
                            if ability.trigger == Trigger::OnDeath {
                                let source = EffectSource::Creature { owner: player, slot: *slot };
                                for effect_def in &ability.effects {
                                    if let Some(effect) = crate::core::engine::effect_convert::effect_def_to_effect_with_target(
                                        effect_def,
                                        EffectTarget::Creature { owner: player, slot: *slot },
                                        player,
                                    ) {
                                        effect_queue.push(effect, source);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Remove ephemeral creatures
        self.state.players[player.index()]
            .creatures
            .retain(|c| !c.keywords.has_ephemeral());

        // Process any OnDeath effects
        effect_queue.process_all(&mut self.state, self.card_db);
    }

    /// End the current player's turn.
    /// - Decrement support durability and remove depleted supports
    /// - Switch to other player
    /// - Call start_turn for new player
    fn end_turn(&mut self) {
        let current_player = self.state.active_player;

        // Decrement support durability and collect supports to remove
        let supports_to_remove: Vec<(Slot, CardId)> = {
            let player_state = &mut self.state.players[current_player.index()];
            let mut to_remove = Vec::new();

            for support in &mut player_state.supports {
                support.current_durability = support.current_durability.saturating_sub(1);
                if support.current_durability == 0 {
                    to_remove.push((support.slot, support.card_id));
                }
            }

            to_remove
        };

        // Remove depleted supports and their passive effects
        for (slot, card_id) in supports_to_remove {
            // Remove passive effects from creatures before removing support
            remove_support_passives_from_all_creatures(
                card_id,
                &mut self.state.players[current_player.index()].creatures,
                self.card_db,
            );

            // Remove the support from the board
            self.state.players[current_player.index()]
                .supports
                .retain(|s| s.slot != slot);
        }

        // Process Ephemeral - creatures with this keyword die at end of turn
        self.process_ephemeral_deaths(current_player);

        // Reset Frenzy stacks - bonus resets at end of turn
        for creature in &mut self.state.players[current_player.index()].creatures {
            creature.frenzy_stacks = 0;
        }

        // Switch to opponent
        self.state.active_player = self.state.active_player.opponent();

        // Start opponent's turn
        self.start_turn();
    }

    /// Check for turn limit victory condition.
    /// Player with higher life wins. On tie, Player 1 wins.
    fn check_turn_limit_victory(&mut self) {
        let p1_life = self.state.players[0].life;
        let p2_life = self.state.players[1].life;

        let winner = if p1_life >= p2_life {
            PlayerId::PLAYER_ONE
        } else {
            PlayerId::PLAYER_TWO
        };

        self.state.result = Some(GameResult::Win {
            winner,
            reason: WinReason::TurnLimitHigherLife,
        });
        self.state.phase = GamePhase::Ended;
    }

    /// Check if a player has lost due to life reaching 0.
    pub fn check_life_victory(&mut self) {
        for player_idx in 0..2 {
            if self.state.players[player_idx].life <= 0 {
                let loser = PlayerId(player_idx as u8);
                let winner = loser.opponent();
                self.state.result = Some(GameResult::Win {
                    winner,
                    reason: WinReason::LifeReachedZero,
                });
                self.state.phase = GamePhase::Ended;
                return;
            }
        }
    }

    /// Check if a player has won via Victory Points (Essence Duel mode only).
    /// In Essence Duel, first player to deal 50 cumulative face damage wins.
    pub fn check_victory_points_victory(&mut self) {
        // Only applies in Essence Duel mode
        if self.state.game_mode != GameMode::EssenceDuel {
            return;
        }

        // Already have a result? Don't override
        if self.state.result.is_some() {
            return;
        }

        for player_idx in 0..2 {
            let vp = self.state.players[player_idx].total_damage_dealt;
            if vp >= game::VICTORY_POINTS_THRESHOLD {
                let winner = PlayerId(player_idx as u8);
                self.state.result = Some(GameResult::Win {
                    winner,
                    reason: WinReason::VictoryPointsReached,
                });
                self.state.phase = GamePhase::Ended;
                return;
            }
        }
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
                self.execute_play_card(hand_index as usize, slot)?;
            }
            Action::Attack { attacker, defender } => {
                self.execute_attack(attacker, defender)?;
            }
            Action::UseAbility {
                slot,
                ability_index,
                target,
            } => {
                self.execute_use_ability(slot, ability_index, target)?;
            }
            Action::EndTurn => {
                self.end_turn();
            }
        }

        // Check for victory after each action
        self.check_life_victory();
        self.check_victory_points_victory();

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
        effect_tracer: Option<&mut EffectTracer>,
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
                // PlayCard doesn't have combat, use regular execution
                self.execute_play_card(hand_index as usize, slot)?;
            }
            Action::Attack { attacker, defender } => {
                self.execute_attack_with_tracers(attacker, defender, combat_tracer, effect_tracer)?;
            }
            Action::UseAbility {
                slot,
                ability_index,
                target,
            } => {
                // UseAbility doesn't have combat, use regular execution
                self.execute_use_ability(slot, ability_index, target)?;
            }
            Action::EndTurn => {
                self.end_turn();
            }
        }

        // Check for victory after each action
        self.check_life_victory();
        self.check_victory_points_victory();

        // Validate state invariants in debug builds
        self.state.debug_validate();

        Ok(())
    }

    /// Execute a PlayCard action.
    ///
    /// Handles creatures, spells, and supports with full effect queue integration:
    /// - Creatures: Place on board with summoning sickness (unless Rush), trigger OnPlay
    /// - Spells: Resolve targeting, queue effects, process queue
    /// - Supports: Place in support slot, trigger OnPlay if present
    pub fn execute_play_card(&mut self, hand_index: usize, slot: Slot) -> Result<(), String> {
        let current_player = self.state.active_player;

        // Get the card from hand
        if hand_index >= self.state.players[current_player.index()].hand.len() {
            return Err("Invalid hand index".to_string());
        }
        let card_instance = self.state.players[current_player.index()].hand.remove(hand_index);
        let card_id = card_instance.card_id;

        // Look up card definition
        let card_def = self
            .card_db
            .get(card_id)
            .ok_or_else(|| "Card not found in database".to_string())?;

        // Check and deduct costs
        // Per design: playing a card costs 1 AP + card's Essence cost
        let player_state = &mut self.state.players[current_player.index()];

        // Check AP (1 AP per card play)
        if player_state.action_points < 1 {
            return Err("Not enough AP".to_string());
        }

        // Check Essence (card's cost)
        if card_def.cost > player_state.current_essence {
            return Err("Not enough Essence".to_string());
        }

        // Deduct 1 AP for the action
        player_state.action_points -= 1;

        // Deduct Essence equal to card cost
        player_state.current_essence -= card_def.cost;

        // Create effect queue for triggered effects
        let mut effect_queue = EffectQueue::new();

        match &card_def.card_type {
            CardType::Creature { attack, health, abilities, .. } => {
                // Create creature instance
                let instance_id = self.state.next_creature_instance_id();
                let keywords = card_def.keywords();

                let creature = Creature {
                    instance_id,
                    card_id,
                    owner: current_player,
                    slot,
                    attack: *attack as i8,
                    current_health: *health as i8,
                    max_health: *health as i8,
                    base_attack: *attack,
                    base_health: *health,
                    keywords,
                    status: CreatureStatus::default(),
                    turn_played: self.state.current_turn,
                    frenzy_stacks: 0,
                };

                // Add creature to board
                self.state.players[current_player.index()].creatures.push(creature);

                // Apply passive effects from existing supports to the new creature
                let supports: Vec<Support> = self.state.players[current_player.index()]
                    .supports.iter().cloned().collect();
                if let Some(new_creature) = self.state.players[current_player.index()]
                    .get_creature_mut(slot) {
                    apply_all_support_passives_to_creature(new_creature, &supports, self.card_db);
                }

                // Queue OnPlay triggered effects
                for ability in abilities {
                    if ability.trigger == Trigger::OnPlay {
                        let source = EffectSource::Creature { owner: current_player, slot };
                        for effect_def in &ability.effects {
                            if let Some(effect) = effect_def_to_triggered_effect(
                                effect_def,
                                current_player,
                                slot,
                                ability,
                            ) {
                                effect_queue.push(effect, source);
                            }
                        }
                    }
                }

                // Check for OnAllyPlayed triggers on other friendly creatures
                let other_creatures: Vec<(Slot, CardId)> = self.state.players[current_player.index()]
                    .creatures
                    .iter()
                    .filter(|c| c.slot != slot && !c.status.is_silenced())
                    .map(|c| (c.slot, c.card_id))
                    .collect();

                for (ally_slot, ally_card_id) in other_creatures {
                    if let Some(ally_card_def) = self.card_db.get(ally_card_id) {
                        if let Some(ally_abilities) = ally_card_def.creature_abilities() {
                            for ability in ally_abilities {
                                if ability.trigger == Trigger::OnAllyPlayed {
                                    let source = EffectSource::Creature {
                                        owner: current_player,
                                        slot: ally_slot,
                                    };
                                    for effect_def in &ability.effects {
                                        if let Some(effect) = effect_def_to_triggered_effect(
                                            effect_def,
                                            current_player,
                                            ally_slot,
                                            ability,
                                        ) {
                                            effect_queue.push(effect, source);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            CardType::Spell { targeting, effects, conditional_effects } => {
                // Resolve the spell target based on targeting rule and slot parameter
                let target = resolve_spell_target(targeting, slot, current_player)?;
                let source = EffectSource::Card(card_id);

                // Queue all spell effects with the resolved target
                for effect_def in effects {
                    if let Some(effect) = effect_def_to_effect_with_target(
                        effect_def,
                        target,
                        current_player,
                    ) {
                        effect_queue.push(effect, source);
                    }
                }

                // If spell has conditional effects, process primary effects first and check conditions
                if !conditional_effects.is_empty() {
                    // Reset accumulated result before processing
                    effect_queue.reset_accumulated_result();

                    // Process primary effects
                    effect_queue.process_all(&mut self.state, self.card_db);

                    // Check conditions and queue bonus effects (clone to avoid borrow issues)
                    let result = effect_queue.accumulated_result().clone();
                    for cond_group in conditional_effects {
                        if result.check(&cond_group.condition) {
                            for effect_def in &cond_group.effects {
                                if let Some(effect) = effect_def_to_effect_with_target(
                                    effect_def,
                                    target,
                                    current_player,
                                ) {
                                    effect_queue.push(effect, source);
                                }
                            }
                        }
                    }
                }

                // Spell is consumed (already removed from hand, no discard pile tracking)
            }
            CardType::Support { durability, triggered_effects, .. } => {
                // Create support instance
                let support = Support {
                    card_id,
                    owner: current_player,
                    slot,
                    current_durability: *durability,
                };

                // Add support to board
                self.state.players[current_player.index()].supports.push(support);

                // Apply passive effects from this support to all existing creatures
                apply_support_passives_to_all_creatures(
                    card_id,
                    &mut self.state.players[current_player.index()].creatures,
                    self.card_db,
                );

                // Queue OnPlay triggered effects for support
                for ability in triggered_effects {
                    if ability.trigger == Trigger::OnPlay {
                        let source = EffectSource::Support { owner: current_player, slot };
                        for effect_def in &ability.effects {
                            // Use support-specific effect conversion
                            if let Some(effect) = support_effect_def_to_effect(
                                effect_def,
                                current_player,
                                ability,
                            ) {
                                effect_queue.push(effect, source);
                            }
                        }
                    }
                }
            }
        }

        // Process all queued effects
        effect_queue.process_all(&mut self.state, self.card_db);

        Ok(())
    }

    /// Execute an Attack action.
    ///
    /// Delegates to the combat module for full keyword resolution including:
    /// Quick, Ranged, Shield, Piercing, Lethal, and Lifesteal.
    fn execute_attack(&mut self, attacker_slot: Slot, defender_slot: Slot) -> Result<(), String> {
        let current_player = self.state.active_player;

        // Validate attacker exists
        let attacker = self
            .state
            .players[current_player.index()]
            .get_creature(attacker_slot)
            .ok_or_else(|| "No creature at attacker slot".to_string())?;

        // Validate attacker can attack
        if !attacker.can_attack(self.state.current_turn) {
            return Err("Creature cannot attack".to_string());
        }

        // Create effect queue for triggered effects (OnAttack, OnKill, OnDeath, etc.)
        let mut effect_queue = EffectQueue::new();

        // Delegate to combat module for full keyword resolution
        // Note: Pass None for tracer - use execute_attack_with_tracer for tracing support
        let _result = combat::resolve_combat(
            &mut self.state,
            self.card_db,
            &mut effect_queue,
            current_player,
            attacker_slot,
            defender_slot,
            None, // No tracing by default
        );

        // Process any triggered effects from combat
        effect_queue.process_all(&mut self.state, self.card_db);

        Ok(())
    }

    /// Execute an attack action with optional tracers for debugging.
    fn execute_attack_with_tracers(
        &mut self,
        attacker_slot: Slot,
        defender_slot: Slot,
        combat_tracer: Option<&mut CombatTracer>,
        effect_tracer: Option<&mut EffectTracer>,
    ) -> Result<(), String> {
        let current_player = self.state.active_player;

        // Validate attacker exists
        let attacker = self
            .state
            .players[current_player.index()]
            .get_creature(attacker_slot)
            .ok_or_else(|| "No creature at attacker slot".to_string())?;

        // Validate attacker can attack
        if !attacker.can_attack(self.state.current_turn) {
            return Err("Creature cannot attack".to_string());
        }

        // Create effect queue for triggered effects
        let mut effect_queue = EffectQueue::new();

        // Delegate to combat module with tracer
        let _result = combat::resolve_combat(
            &mut self.state,
            self.card_db,
            &mut effect_queue,
            current_player,
            attacker_slot,
            defender_slot,
            combat_tracer,
        );

        // Process any triggered effects from combat with tracer
        effect_queue.process_all_with_tracer(&mut self.state, self.card_db, effect_tracer);

        Ok(())
    }

    /// Execute a UseAbility action.
    ///
    /// Uses a creature's ability at the specified slot with the given target.
    /// The ability is identified by ability_index (0-based).
    ///
    /// # Errors
    /// - Returns error if no creature exists at the slot
    /// - Returns error if creature is silenced
    /// - Returns error if ability_index is out of bounds
    /// - Returns error if the card definition is not found
    fn execute_use_ability(
        &mut self,
        slot: Slot,
        ability_index: u8,
        target: crate::actions::Target,
    ) -> Result<(), String> {
        use crate::core::actions::Target;
        use crate::core::effects::EffectTarget;

        let current_player = self.state.active_player;

        // Get creature at slot
        let creature = self.state.players[current_player.index()]
            .get_creature(slot)
            .ok_or("No creature at slot")?;

        // Silenced creatures can't use abilities
        if creature.status.is_silenced() {
            return Err("Creature is silenced".to_string());
        }

        let card_id = creature.card_id;

        // Get card definition
        let card_def = self.card_db.get(card_id)
            .ok_or("Card not found")?;

        // Get abilities from card type
        let abilities = match &card_def.card_type {
            CardType::Creature { abilities, .. } => abilities,
            _ => return Err("Not a creature card".to_string()),
        };

        // Get the specific ability
        let ability = abilities.get(ability_index as usize)
            .ok_or("Invalid ability index")?;

        // Convert target to EffectTarget
        let effect_target = match target {
            Target::NoTarget => EffectTarget::None,
            Target::EnemySlot(s) => EffectTarget::Creature {
                owner: current_player.opponent(),
                slot: s
            },
            Target::Self_ => EffectTarget::Creature {
                owner: current_player,
                slot
            },
        };

        // Create effect queue and queue ability effects
        let mut effect_queue = EffectQueue::new();
        let source = EffectSource::Creature { owner: current_player, slot };

        for effect_def in &ability.effects {
            // Convert EffectDefinition to Effect with the resolved target
            if let Some(effect) = effect_def_to_effect_with_target(effect_def, effect_target, current_player) {
                effect_queue.push(effect, source);
            }
        }

        // Process all effects
        effect_queue.process_all(&mut self.state, self.card_db);

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
