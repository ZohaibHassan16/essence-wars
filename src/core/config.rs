//! Game configuration constants and settings.
//!
//! This module contains all tunable game parameters. Having them in one place
//! makes balancing and experimentation easier.

/// Player-related constants
pub mod player {
    /// Starting life points for each player
    pub const STARTING_LIFE: u8 = 30;

    /// Maximum life (cannot heal above this)
    pub const MAX_LIFE: u8 = 30;

    /// Cards drawn at game start
    pub const STARTING_HAND_SIZE: usize = 4;

    /// Extra cards Player 2 draws at game start (First Player Advantage compensation)
    /// P2 starts with 6 cards (4 base + 2 bonus) vs P1's 4 cards.
    /// This compensates for P1's tempo advantage from playing first.
    /// Tested with 10k games: achieves ~50% P1 win rate (was 52.6% with equal starts).
    pub const P2_BONUS_CARDS: usize = 2;

    /// Maximum cards in hand (excess are burned)
    pub const MAX_HAND_SIZE: usize = 10;

    /// Action points restored each turn
    pub const AP_PER_TURN: u8 = 3;

    /// Starting essence for Player 1
    pub const STARTING_ESSENCE_P1: u8 = 1;

    /// Starting essence for Player 2 (equal to P1)
    /// FPA compensation is now done via bonus cards instead of essence.
    pub const STARTING_ESSENCE_P2: u8 = 1;

    /// Maximum essence pool
    pub const MAX_ESSENCE: u8 = 10;

    /// Essence gained per turn
    pub const ESSENCE_PER_TURN: u8 = 1;
}

/// Board-related constants
pub mod board {
    /// Number of creature slots per player
    pub const CREATURE_SLOTS: usize = 5;

    /// Number of support slots per player
    pub const SUPPORT_SLOTS: usize = 2;
}

/// Game length constants
pub mod game {
    /// Deck size for starter/testing decks
    pub const STARTER_DECK_SIZE: usize = 20;

    /// Deck size for standard/competitive play
    pub const STANDARD_DECK_SIZE: usize = 30;

    /// Maximum deck size
    pub const MAX_DECK_SIZE: usize = 30;

    /// Turn limit (game ends at this turn)
    pub const TURN_LIMIT: u8 = 30;

    /// Victory Points threshold for Essence Duel mode
    pub const VICTORY_POINTS_THRESHOLD: u16 = 50;
}

/// Action space constants for neural network interface
pub mod actions {
    /// Total action space size (for neural network output)
    pub const ACTION_SPACE_SIZE: usize = 256;

    /// Maximum hand positions for PlayCard
    pub const MAX_HAND_POSITIONS: usize = 10;

    /// Maximum ability index per creature
    pub const MAX_ABILITIES: usize = 6;
}

/// Tensor constants for neural network interface
pub mod tensor {
    /// Size of the state tensor
    pub const STATE_TENSOR_SIZE: usize = 326;

    /// Action mask size
    pub const ACTION_MASK_SIZE: usize = 256;
}

/// Balance formula helpers
pub mod balance {
    /// Calculate vanilla stat budget for a creature
    /// Formula: (Essence Cost × 2) + 1
    pub const fn vanilla_stats(essence_cost: u8) -> u8 {
        essence_cost * 2 + 1
    }
}

/// GameConfig struct for runtime-adjustable settings
#[derive(Debug, Clone, Copy)]
pub struct GameConfig {
    pub deck_size: usize,
    pub turn_limit: u8,
    pub starting_hand_size: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            deck_size: game::STARTER_DECK_SIZE,
            turn_limit: game::TURN_LIMIT,
            starting_hand_size: player::STARTING_HAND_SIZE,
        }
    }
}

impl GameConfig {
    /// Configuration for standard competitive play
    pub fn standard() -> Self {
        Self {
            deck_size: game::STANDARD_DECK_SIZE,
            turn_limit: game::TURN_LIMIT,
            starting_hand_size: player::STARTING_HAND_SIZE,
        }
    }

    /// Configuration for quick test games
    pub fn test() -> Self {
        Self {
            deck_size: 10,
            turn_limit: 15,
            starting_hand_size: 3,
        }
    }
}
