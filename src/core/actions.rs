//! Module for game action definitions.
//!
//! Actions represent all possible moves a player can make, such as
//! playing cards, attacking, activating abilities, and passing priority.
//!
//! The game uses a fixed action space of 256 actions for neural network compatibility:
//! - Index 0-49:    PlayCard(hand_idx 0-9, slot 0-4)
//! - Index 50-74:   Attack(attacker_slot 0-4, defender_slot 0-4)
//! - Index 75-254:  UseAbility(slot, ability_idx, target)
//! - Index 255:     EndTurn

use serde::{Deserialize, Serialize};
use crate::core::config::actions as action_config;
use crate::core::types::Slot;

/// Target for ability effects
///
/// Target encoding for neural network:
/// - 0: no target
/// - 1-5: enemy slots 0-4
/// - 6: self (the creature using ability)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Target {
    /// No target required
    NoTarget,
    /// Target an enemy creature in a specific slot (0-4)
    EnemySlot(Slot),
    /// Target self (the creature using the ability)
    Self_,
}

impl Target {
    /// Convert target to index for neural network encoding
    /// - 0: no target
    /// - 1-5: enemy slots 0-4
    /// - 6: self
    pub fn to_index(&self) -> u8 {
        match self {
            Target::NoTarget => 0,
            Target::EnemySlot(slot) => 1 + slot.0,
            Target::Self_ => 6,
        }
    }

    /// Convert index back to target
    /// Returns None if index is out of range (> 6)
    pub fn from_index(index: u8) -> Option<Target> {
        match index {
            0 => Some(Target::NoTarget),
            1..=5 => Some(Target::EnemySlot(Slot(index - 1))),
            6 => Some(Target::Self_),
            _ => None,
        }
    }
}

/// Represents all possible game actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Play a card from hand to a board slot
    PlayCard {
        /// Index in hand (0-9)
        hand_index: u8,
        /// Target slot on board (0-4)
        slot: Slot,
    },
    /// Attack with a creature
    Attack {
        /// Slot of attacking creature (0-4)
        attacker: Slot,
        /// Slot of defending creature (0-4)
        defender: Slot,
    },
    /// Use a creature's activated ability
    UseAbility {
        /// Slot of creature using ability (0-4)
        slot: Slot,
        /// Index of ability to use (0-5)
        ability_index: u8,
        /// Target for the ability
        target: Target,
    },
    /// End the current turn
    EndTurn,
}

impl Action {
    /// Total number of possible action indices for neural network
    pub const ACTION_SPACE_SIZE: usize = action_config::ACTION_SPACE_SIZE;

    // Index ranges
    const PLAY_CARD_START: u8 = 0;
    const PLAY_CARD_END: u8 = 49;
    const ATTACK_START: u8 = 50;
    const ATTACK_END: u8 = 74;
    const USE_ABILITY_START: u8 = 75;
    const USE_ABILITY_END: u8 = 254;
    const END_TURN_INDEX: u8 = 255;

    // Limits
    const NUM_SLOTS: u8 = 5;
    const MAX_ABILITIES: u8 = 6;
    const NUM_TARGETS: u8 = 6; // 0 (no target) + 5 (enemy slots) - formula uses ability * 6 + target

    /// Convert action to neural network output index (0-255)
    ///
    /// Index mapping:
    /// - PlayCard: hand_idx * 5 + slot (0-49)
    /// - Attack: 50 + attacker * 5 + defender (50-74)
    /// - UseAbility: 75 + slot * 36 + ability * 6 + target (75-254)
    /// - EndTurn: 255
    ///
    /// Note: For UseAbility, target indices 0-5 are used (NoTarget and EnemySlot only).
    /// Self_ target (index 6) would exceed the index range and should not be used.
    pub fn to_index(&self) -> u8 {
        match self {
            Action::PlayCard { hand_index, slot } => hand_index * Self::NUM_SLOTS + slot.0,
            Action::Attack { attacker, defender } => {
                Self::ATTACK_START + attacker.0 * Self::NUM_SLOTS + defender.0
            }
            Action::UseAbility {
                slot,
                ability_index,
                target,
            } => {
                Self::USE_ABILITY_START
                    + slot.0 * (Self::MAX_ABILITIES * Self::NUM_TARGETS)
                    + ability_index * Self::NUM_TARGETS
                    + target.to_index()
            }
            Action::EndTurn => Self::END_TURN_INDEX,
        }
    }

    /// Convert neural network output index back to action
    ///
    /// Returns None if index is invalid or out of range
    pub fn from_index(index: u8) -> Option<Action> {
        match index {
            Self::PLAY_CARD_START..=Self::PLAY_CARD_END => {
                let hand_index = index / Self::NUM_SLOTS;
                let slot = index % Self::NUM_SLOTS;
                Some(Action::PlayCard {
                    hand_index,
                    slot: Slot(slot),
                })
            }
            Self::ATTACK_START..=Self::ATTACK_END => {
                let offset = index - Self::ATTACK_START;
                let attacker = offset / Self::NUM_SLOTS;
                let defender = offset % Self::NUM_SLOTS;
                Some(Action::Attack {
                    attacker: Slot(attacker),
                    defender: Slot(defender),
                })
            }
            Self::USE_ABILITY_START..=Self::USE_ABILITY_END => {
                let offset = index - Self::USE_ABILITY_START;
                let abilities_per_slot = Self::MAX_ABILITIES * Self::NUM_TARGETS;
                let slot = offset / abilities_per_slot;
                let remaining = offset % abilities_per_slot;
                let ability_index = remaining / Self::NUM_TARGETS;
                let target_index = remaining % Self::NUM_TARGETS;

                // Validate slot is within range (0-4)
                if slot >= Self::NUM_SLOTS {
                    return None;
                }

                let target = Target::from_index(target_index)?;

                Some(Action::UseAbility {
                    slot: Slot(slot),
                    ability_index,
                    target,
                })
            }
            Self::END_TURN_INDEX => Some(Action::EndTurn),
        }
    }
}
