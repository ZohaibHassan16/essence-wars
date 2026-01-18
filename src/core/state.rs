//! Game state representation.
//!
//! The GameState struct contains all information needed to represent a game in progress.
//! It uses ArrayVec for stack allocation to enable fast cloning (critical for MCTS).

use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use crate::core::config::{board, game, player};
use crate::core::types::*;
use crate::core::keywords::Keywords;

/// Status flags for creatures (packed bitfield)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatureStatus(pub u8);

impl CreatureStatus {
    pub const EXHAUSTED: u8 = 0b0000_0001;
    pub const SILENCED: u8  = 0b0000_0010;

    #[inline]
    pub fn is_exhausted(self) -> bool { self.0 & Self::EXHAUSTED != 0 }

    #[inline]
    pub fn is_silenced(self) -> bool { self.0 & Self::SILENCED != 0 }

    #[inline]
    pub fn set_exhausted(&mut self, val: bool) {
        if val { self.0 |= Self::EXHAUSTED; } else { self.0 &= !Self::EXHAUSTED; }
    }

    #[inline]
    pub fn set_silenced(&mut self, val: bool) {
        if val { self.0 |= Self::SILENCED; } else { self.0 &= !Self::SILENCED; }
    }
}

/// A creature on the battlefield
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Creature {
    pub instance_id: CreatureInstanceId,
    pub card_id: CardId,
    pub owner: PlayerId,
    pub slot: Slot,
    pub attack: i8,           // Current attack (can be negative from debuffs)
    pub current_health: i8,   // Current health
    pub max_health: i8,       // Maximum health (for healing cap)
    pub base_attack: u8,      // Original attack from card
    pub base_health: u8,      // Original health from card
    pub keywords: Keywords,
    pub status: CreatureStatus,
    pub turn_played: u16,
    pub frenzy_stacks: u8,    // Frenzy bonus: +1 attack per stack (resets at end of turn)
}

impl Creature {
    /// Check if this creature can attack (not exhausted, not summoning sick unless Rush)
    pub fn can_attack(&self, current_turn: u16) -> bool {
        if self.status.is_exhausted() {
            return false;
        }
        if self.turn_played == current_turn && !self.keywords.has_rush() {
            return false;
        }
        if self.attack <= 0 {
            return false;
        }
        true
    }

    /// Check if this creature is alive
    pub fn is_alive(&self) -> bool {
        self.current_health > 0
    }
}

/// A support card on the battlefield
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Support {
    pub card_id: CardId,
    pub owner: PlayerId,
    pub slot: Slot,
    pub current_durability: u8,
}

/// A card instance (in hand or deck)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardInstance {
    pub card_id: CardId,
}

impl CardInstance {
    pub fn new(card_id: CardId) -> Self {
        Self { card_id }
    }
}

/// Per-player state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerState {
    pub life: i16,
    pub max_essence: u8,
    pub current_essence: u8,
    pub action_points: u8,
    pub hand: ArrayVec<CardInstance, {player::MAX_HAND_SIZE}>,
    pub deck: ArrayVec<CardInstance, {game::MAX_DECK_SIZE}>,
    pub creatures: ArrayVec<Creature, {board::CREATURE_SLOTS}>,
    pub supports: ArrayVec<Support, {board::SUPPORT_SLOTS}>,
    pub total_damage_dealt: u16,               // For victory points tracking
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            life: player::STARTING_LIFE as i16,
            max_essence: 0,
            current_essence: 0,
            action_points: 0,
            hand: ArrayVec::new(),
            deck: ArrayVec::new(),
            creatures: ArrayVec::new(),
            supports: ArrayVec::new(),
            total_damage_dealt: 0,
        }
    }

    /// Get creature at a specific slot
    pub fn get_creature(&self, slot: Slot) -> Option<&Creature> {
        self.creatures.iter().find(|c| c.slot == slot)
    }

    /// Get mutable creature at a specific slot
    pub fn get_creature_mut(&mut self, slot: Slot) -> Option<&mut Creature> {
        self.creatures.iter_mut().find(|c| c.slot == slot)
    }

    /// Get support at a specific slot
    pub fn get_support(&self, slot: Slot) -> Option<&Support> {
        self.supports.iter().find(|s| s.slot == slot)
    }

    /// Find first empty creature slot
    pub fn find_empty_creature_slot(&self) -> Option<Slot> {
        for i in 0..board::CREATURE_SLOTS as u8 {
            let slot = Slot(i);
            if self.get_creature(slot).is_none() {
                return Some(slot);
            }
        }
        None
    }

    /// Find first empty support slot
    pub fn find_empty_support_slot(&self) -> Option<Slot> {
        for i in 0..board::SUPPORT_SLOTS as u8 {
            let slot = Slot(i);
            if self.get_support(slot).is_none() {
                return Some(slot);
            }
        }
        None
    }

    /// Check if hand is full
    pub fn is_hand_full(&self) -> bool {
        self.hand.len() >= player::MAX_HAND_SIZE
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Game phase
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GamePhase {
    #[default]
    Main,
    Ended,
}

/// Win reason
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WinReason {
    LifeReachedZero,
    TurnLimitHigherLife,
    VictoryPointsReached,
    Concession,
}

/// Game mode determines victory conditions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GameMode {
    /// Attrition: Reduce enemy to 0 life, or turn 30 → higher life wins
    #[default]
    Attrition,
    /// Essence Duel: First to 50 VP (cumulative face damage) or reduce to 0 life
    EssenceDuel,
}

/// Game result
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameResult {
    Win { winner: PlayerId, reason: WinReason },
    Draw,
}

/// Complete game state - everything needed to continue a game
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub players: [PlayerState; 2],
    pub current_turn: u16,
    pub active_player: PlayerId,
    pub phase: GamePhase,
    pub next_creature_id: u32,
    pub rng_state: u64,
    pub result: Option<GameResult>,
    pub game_mode: GameMode,
}

impl GameState {
    /// Create a new game state with default values
    pub fn new() -> Self {
        Self {
            players: [PlayerState::new(), PlayerState::new()],
            current_turn: 0,
            active_player: PlayerId::PLAYER_ONE,
            phase: GamePhase::Main,
            next_creature_id: 0,
            rng_state: 0,
            result: None,
            game_mode: GameMode::default(),
        }
    }

    /// Get a creature by owner and slot
    pub fn get_creature(&self, owner: PlayerId, slot: Slot) -> Option<&Creature> {
        self.players[owner.index()].get_creature(slot)
    }

    /// Get a mutable creature by owner and slot
    pub fn get_creature_mut(&mut self, owner: PlayerId, slot: Slot) -> Option<&mut Creature> {
        self.players[owner.index()].get_creature_mut(slot)
    }

    /// Get a support by owner and slot
    pub fn get_support(&self, owner: PlayerId, slot: Slot) -> Option<&Support> {
        self.players[owner.index()].get_support(slot)
    }

    /// Check if game is over
    pub fn is_terminal(&self) -> bool {
        self.result.is_some()
    }

    /// Get active player's state
    pub fn active_player_state(&self) -> &PlayerState {
        &self.players[self.active_player.index()]
    }

    /// Get active player's state mutably
    pub fn active_player_state_mut(&mut self) -> &mut PlayerState {
        &mut self.players[self.active_player.index()]
    }

    /// Get opponent's state
    pub fn opponent_state(&self) -> &PlayerState {
        &self.players[self.active_player.opponent().index()]
    }

    /// Generate next creature instance ID
    pub fn next_creature_instance_id(&mut self) -> CreatureInstanceId {
        let id = CreatureInstanceId(self.next_creature_id);
        self.next_creature_id += 1;
        id
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// DEBUG VALIDATION
// =============================================================================

impl PlayerState {
    /// Validate player state invariants (debug builds only).
    /// Panics if any invariant is violated.
    #[cfg(debug_assertions)]
    pub fn debug_validate(&self, player_id: PlayerId) {
        // Note: We don't validate action_points since tests may set arbitrary values
        // and bonus AP effects could exist in future card designs.

        // Validate creatures
        let mut seen_slots: Vec<u8> = Vec::new();
        for creature in &self.creatures {
            // Creature slot should be valid
            debug_assert!(
                (creature.slot.0 as usize) < board::CREATURE_SLOTS,
                "Creature at invalid slot {} (max {})",
                creature.slot.0, board::CREATURE_SLOTS - 1
            );

            // Creature owner should match the player
            debug_assert!(
                creature.owner == player_id,
                "Creature owner {:?} doesn't match player {:?}",
                creature.owner, player_id
            );

            // No duplicate slots
            debug_assert!(
                !seen_slots.contains(&creature.slot.0),
                "Duplicate creature at slot {}",
                creature.slot.0
            );
            seen_slots.push(creature.slot.0);

            // Creatures on board should be alive
            // Note: Temporarily allowing 0-health creatures since death processing is asynchronous
            // through the effect queue's pending_deaths system. The creature will be removed
            // on the next process_deaths() call.
            debug_assert!(
                creature.current_health >= 0,
                "Creature with invalid health ({}) at slot {}",
                creature.current_health, creature.slot.0
            );
        }

        // Validate supports
        let mut seen_support_slots: Vec<u8> = Vec::new();
        for support in &self.supports {
            // Support slot should be valid
            debug_assert!(
                (support.slot.0 as usize) < board::SUPPORT_SLOTS,
                "Support at invalid slot {} (max {})",
                support.slot.0, board::SUPPORT_SLOTS - 1
            );

            // Support owner should match the player
            debug_assert!(
                support.owner == player_id,
                "Support owner {:?} doesn't match player {:?}",
                support.owner, player_id
            );

            // No duplicate slots
            debug_assert!(
                !seen_support_slots.contains(&support.slot.0),
                "Duplicate support at slot {}",
                support.slot.0
            );
            seen_support_slots.push(support.slot.0);

            // Supports on board should have durability > 0
            debug_assert!(
                support.current_durability > 0,
                "Support with 0 durability still on board at slot {}",
                support.slot.0
            );
        }
    }

    /// No-op in release builds
    #[cfg(not(debug_assertions))]
    #[inline(always)]
    pub fn debug_validate(&self, _player_id: PlayerId) {}
}

impl GameState {
    /// Validate game state invariants (debug builds only).
    /// Panics if any invariant is violated.
    #[cfg(debug_assertions)]
    pub fn debug_validate(&self) {
        // Active player should be valid
        debug_assert!(
            self.active_player.0 < 2,
            "Invalid active player: {}",
            self.active_player.0
        );

        // Turn counter should be reasonable
        debug_assert!(
            self.current_turn <= (game::TURN_LIMIT as u16) + 10,
            "Turn counter {} exceeds reasonable limit",
            self.current_turn
        );

        // If game has a result, phase should be Ended (or transitioning)
        if self.result.is_some() {
            debug_assert!(
                self.phase == GamePhase::Ended,
                "Game has result but phase is {:?}, expected Ended",
                self.phase
            );
        }

        // Validate both players
        self.players[0].debug_validate(PlayerId::PLAYER_ONE);
        self.players[1].debug_validate(PlayerId::PLAYER_TWO);

        // Validate creature instance IDs are unique across both players
        #[cfg(debug_assertions)]
        {
            let mut all_ids: Vec<u32> = Vec::new();
            for player in &self.players {
                for creature in &player.creatures {
                    debug_assert!(
                        !all_ids.contains(&creature.instance_id.0),
                        "Duplicate creature instance ID: {}",
                        creature.instance_id.0
                    );
                    all_ids.push(creature.instance_id.0);
                }
            }
        }
    }

    /// No-op in release builds
    #[cfg(not(debug_assertions))]
    #[inline(always)]
    pub fn debug_validate(&self) {}
}
