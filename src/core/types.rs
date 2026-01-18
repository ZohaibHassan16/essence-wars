//! Module for core type definitions used throughout the card game engine.
//!
//! This module contains fundamental types such as CardId, PlayerId, and other
//! primitive types that form the foundation of the game's type system.

use serde::{Deserialize, Serialize};
use crate::core::config::board;

/// Unique identifier for a card definition (not an instance)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CardId(pub u16);

/// Player identifier (0 or 1)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u8);

impl PlayerId {
    pub const PLAYER_ONE: PlayerId = PlayerId(0);
    pub const PLAYER_TWO: PlayerId = PlayerId(1);

    #[inline]
    pub fn opponent(self) -> PlayerId {
        PlayerId(1 - self.0)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Board slot position (0-4 for creatures, 0-1 for supports)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Slot(pub u8);

impl Slot {
    pub const CREATURE_SLOTS: usize = board::CREATURE_SLOTS;
    pub const SUPPORT_SLOTS: usize = board::SUPPORT_SLOTS;

    /// Get slots that can be attacked from this slot (lane adjacency)
    /// Per DESIGN.md: Slot 0 can attack 0,1; Slot 1 can attack 0,1,2; etc.
    pub fn adjacent_slots(self) -> &'static [Slot] {
        match self.0 {
            0 => &[Slot(0), Slot(1)],
            1 => &[Slot(0), Slot(1), Slot(2)],
            2 => &[Slot(1), Slot(2), Slot(3)],
            3 => &[Slot(2), Slot(3), Slot(4)],
            4 => &[Slot(3), Slot(4)],
            _ => &[],
        }
    }

    /// Check if this slot can attack target slot
    pub fn can_attack(self, target: Slot) -> bool {
        self.adjacent_slots().contains(&target)
    }
}

/// Unique identifier for a creature instance on the board
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CreatureInstanceId(pub u32);

/// Card rarity
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Legendary,
}

/// Creature type tags
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tag {
    Soldier,
    Beast,
    Mage,
    Undead,
    Construct,
    Divine,
    Assassin,
    Healer,
    Cultist,
    Giant,
}
