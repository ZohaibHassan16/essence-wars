//! Game state representation.
//!
//! The GameState struct contains all information needed to represent a game in progress.
//! It uses ArrayVec for stack allocation to enable fast cloning (critical for MCTS).

use arrayvec::ArrayVec;
use crate::core::config::{board, game, player};
use crate::core::types::*;
use crate::core::keywords::Keywords;

/// Status flags for creatures (packed bitfield)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct Support {
    pub card_id: CardId,
    pub owner: PlayerId,
    pub slot: Slot,
    pub current_durability: u8,
}

/// A card instance (in hand or deck)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardInstance {
    pub card_id: CardId,
}

impl CardInstance {
    pub fn new(card_id: CardId) -> Self {
        Self { card_id }
    }
}

/// Per-player state
#[derive(Clone, Debug)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GamePhase {
    #[default]
    Main,
    Ended,
}

/// Win reason
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WinReason {
    LifeReachedZero,
    TurnLimitHigherLife,
    VictoryPointsReached,
    Concession,
}

/// Game result
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult {
    Win { winner: PlayerId, reason: WinReason },
    Draw,
}

/// Complete game state - everything needed to continue a game
#[derive(Clone, Debug)]
pub struct GameState {
    pub players: [PlayerState; 2],
    pub current_turn: u16,
    pub active_player: PlayerId,
    pub phase: GamePhase,
    pub next_creature_id: u32,
    pub rng_state: u64,
    pub result: Option<GameResult>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creature_status() {
        let mut status = CreatureStatus::default();
        assert!(!status.is_exhausted());
        assert!(!status.is_silenced());

        status.set_exhausted(true);
        assert!(status.is_exhausted());
        assert!(!status.is_silenced());

        status.set_silenced(true);
        assert!(status.is_exhausted());
        assert!(status.is_silenced());

        status.set_exhausted(false);
        assert!(!status.is_exhausted());
        assert!(status.is_silenced());
    }

    #[test]
    fn test_player_state_creature_slots() {
        let mut player = PlayerState::new();

        // All slots should be empty initially
        assert!(player.find_empty_creature_slot().is_some());
        assert_eq!(player.find_empty_creature_slot(), Some(Slot(0)));

        // Add a creature to slot 0
        player.creatures.push(Creature {
            instance_id: CreatureInstanceId(0),
            card_id: CardId(1),
            owner: PlayerId::PLAYER_ONE,
            slot: Slot(0),
            attack: 2,
            current_health: 3,
            max_health: 3,
            base_attack: 2,
            base_health: 3,
            keywords: Keywords::none(),
            status: CreatureStatus::default(),
            turn_played: 1,
        });

        // Next empty slot should be 1
        assert_eq!(player.find_empty_creature_slot(), Some(Slot(1)));
        assert!(player.get_creature(Slot(0)).is_some());
        assert!(player.get_creature(Slot(1)).is_none());
    }

    #[test]
    fn test_game_state_helpers() {
        let mut state = GameState::new();

        assert!(!state.is_terminal());
        assert_eq!(state.active_player, PlayerId::PLAYER_ONE);

        // Test creature ID generation
        let id1 = state.next_creature_instance_id();
        let id2 = state.next_creature_instance_id();
        assert_eq!(id1, CreatureInstanceId(0));
        assert_eq!(id2, CreatureInstanceId(1));
    }

    #[test]
    fn test_creature_can_attack() {
        let mut creature = Creature {
            instance_id: CreatureInstanceId(0),
            card_id: CardId(1),
            owner: PlayerId::PLAYER_ONE,
            slot: Slot(0),
            attack: 2,
            current_health: 3,
            max_health: 3,
            base_attack: 2,
            base_health: 3,
            keywords: Keywords::none(),
            status: CreatureStatus::default(),
            turn_played: 1,
        };

        // Can attack on turn 2 (no summoning sickness)
        assert!(creature.can_attack(2));

        // Cannot attack on turn 1 (summoning sickness)
        assert!(!creature.can_attack(1));

        // Can attack on turn 1 with Rush
        creature.keywords = Keywords::none().with_rush();
        assert!(creature.can_attack(1));

        // Cannot attack when exhausted
        creature.status.set_exhausted(true);
        assert!(!creature.can_attack(1));
        assert!(!creature.can_attack(2));
    }

    #[test]
    fn test_game_state_size() {
        // GameState should be reasonably small for fast cloning
        let size = std::mem::size_of::<GameState>();
        println!("GameState size: {} bytes", size);
        // Should be under 2KB for efficient MCTS cloning
        assert!(size < 2048, "GameState too large: {} bytes", size);
    }
}
