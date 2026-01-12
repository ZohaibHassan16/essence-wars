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

use crate::core::config::actions as action_config;
use crate::core::types::Slot;

/// Target for ability effects
///
/// Target encoding for neural network:
/// - 0: no target
/// - 1-5: enemy slots 0-4
/// - 6: self (the creature using ability)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_round_trip() {
        // Test all valid target indices
        for i in 0..=6 {
            let target = Target::from_index(i).unwrap();
            assert_eq!(target.to_index(), i);
        }

        // Test specific targets
        assert_eq!(Target::NoTarget.to_index(), 0);
        assert_eq!(Target::EnemySlot(Slot(0)).to_index(), 1);
        assert_eq!(Target::EnemySlot(Slot(4)).to_index(), 5);
        assert_eq!(Target::Self_.to_index(), 6);
    }

    #[test]
    fn test_target_invalid_index() {
        assert!(Target::from_index(7).is_none());
        assert!(Target::from_index(255).is_none());
    }

    #[test]
    fn test_play_card_indices() {
        // PlayCard indices should be 0-49
        for hand_idx in 0..10 {
            for slot in 0..5 {
                let action = Action::PlayCard {
                    hand_index: hand_idx,
                    slot: Slot(slot),
                };
                let index = action.to_index();
                assert!(index <= 49, "PlayCard index {} out of range", index);
                assert_eq!(index, hand_idx * 5 + slot);
            }
        }

        // Verify boundary values
        let first = Action::PlayCard {
            hand_index: 0,
            slot: Slot(0),
        };
        assert_eq!(first.to_index(), 0);

        let last = Action::PlayCard {
            hand_index: 9,
            slot: Slot(4),
        };
        assert_eq!(last.to_index(), 49);
    }

    #[test]
    fn test_attack_indices() {
        // Attack indices should be 50-74
        for attacker in 0..5 {
            for defender in 0..5 {
                let action = Action::Attack {
                    attacker: Slot(attacker),
                    defender: Slot(defender),
                };
                let index = action.to_index();
                assert!(
                    index >= 50 && index <= 74,
                    "Attack index {} out of range",
                    index
                );
                assert_eq!(index, 50 + attacker * 5 + defender);
            }
        }

        // Verify boundary values
        let first = Action::Attack {
            attacker: Slot(0),
            defender: Slot(0),
        };
        assert_eq!(first.to_index(), 50);

        let last = Action::Attack {
            attacker: Slot(4),
            defender: Slot(4),
        };
        assert_eq!(last.to_index(), 74);
    }

    #[test]
    fn test_use_ability_indices() {
        // UseAbility indices should be 75-254
        // Using target indices 0-5 (NoTarget and EnemySlot only, not Self_)
        for slot in 0..5 {
            for ability in 0..6 {
                for target in 0..6 {
                    let target_enum = Target::from_index(target).unwrap();
                    let action = Action::UseAbility {
                        slot: Slot(slot),
                        ability_index: ability,
                        target: target_enum,
                    };
                    let index = action.to_index();
                    assert!(
                        index >= 75 && index <= 254,
                        "UseAbility index {} out of range for slot={}, ability={}, target={}",
                        index,
                        slot,
                        ability,
                        target
                    );
                    assert_eq!(index, 75 + slot * 36 + ability * 6 + target);
                }
            }
        }

        // Verify boundary values
        let first = Action::UseAbility {
            slot: Slot(0),
            ability_index: 0,
            target: Target::NoTarget,
        };
        assert_eq!(first.to_index(), 75);

        // Last valid UseAbility: slot=4, ability=5, target=5 (EnemySlot(4))
        let last = Action::UseAbility {
            slot: Slot(4),
            ability_index: 5,
            target: Target::EnemySlot(Slot(4)),
        };
        assert_eq!(last.to_index(), 254);
    }

    #[test]
    fn test_end_turn_index() {
        let action = Action::EndTurn;
        assert_eq!(action.to_index(), 255);
    }

    #[test]
    fn test_play_card_round_trip() {
        for hand_idx in 0..10 {
            for slot in 0..5 {
                let original = Action::PlayCard {
                    hand_index: hand_idx,
                    slot: Slot(slot),
                };
                let index = original.to_index();
                let restored = Action::from_index(index).unwrap();
                assert_eq!(original, restored);
            }
        }
    }

    #[test]
    fn test_attack_round_trip() {
        for attacker in 0..5 {
            for defender in 0..5 {
                let original = Action::Attack {
                    attacker: Slot(attacker),
                    defender: Slot(defender),
                };
                let index = original.to_index();
                let restored = Action::from_index(index).unwrap();
                assert_eq!(original, restored);
            }
        }
    }

    #[test]
    fn test_use_ability_round_trip() {
        // Only test targets 0-5 (NoTarget and EnemySlot) which fit in the index range
        for slot in 0..5 {
            for ability in 0..6 {
                for target_idx in 0..6 {
                    let target = Target::from_index(target_idx).unwrap();
                    let original = Action::UseAbility {
                        slot: Slot(slot),
                        ability_index: ability,
                        target,
                    };
                    let index = original.to_index();
                    let restored = Action::from_index(index).unwrap();
                    assert_eq!(original, restored);
                }
            }
        }
    }

    #[test]
    fn test_end_turn_round_trip() {
        let original = Action::EndTurn;
        let index = original.to_index();
        let restored = Action::from_index(index).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_all_indices_valid() {
        // Every index 0-255 should map to some action or None
        for i in 0u8..=255 {
            let action = Action::from_index(i);
            if let Some(a) = action {
                // Valid actions should round-trip
                assert_eq!(Action::from_index(a.to_index()), Some(a));
            }
        }
    }

    #[test]
    fn test_index_ranges_complete() {
        // Verify all expected indices are covered without gaps

        // PlayCard: 0-49 (50 indices = 10 hand positions * 5 slots)
        for i in 0..=49 {
            assert!(
                Action::from_index(i).is_some(),
                "PlayCard index {} should be valid",
                i
            );
            match Action::from_index(i).unwrap() {
                Action::PlayCard { .. } => {}
                _ => panic!("Index {} should be PlayCard", i),
            }
        }

        // Attack: 50-74 (25 indices = 5 attacker slots * 5 defender slots)
        for i in 50..=74 {
            assert!(
                Action::from_index(i).is_some(),
                "Attack index {} should be valid",
                i
            );
            match Action::from_index(i).unwrap() {
                Action::Attack { .. } => {}
                _ => panic!("Index {} should be Attack", i),
            }
        }

        // UseAbility: 75-254 (180 indices = 5 slots * 6 abilities * 6 targets)
        for i in 75..=254 {
            assert!(
                Action::from_index(i).is_some(),
                "UseAbility index {} should be valid",
                i
            );
            match Action::from_index(i).unwrap() {
                Action::UseAbility { .. } => {}
                _ => panic!("Index {} should be UseAbility", i),
            }
        }

        // EndTurn: 255
        assert!(Action::from_index(255).is_some());
        match Action::from_index(255).unwrap() {
            Action::EndTurn => {}
            _ => panic!("Index 255 should be EndTurn"),
        }
    }

    #[test]
    fn test_invalid_indices_out_of_range() {
        // All indices 0-255 are valid, so test that from_index doesn't panic
        // and returns Some for all valid indices
        let mut valid_count = 0;
        for i in 0u8..=255 {
            if Action::from_index(i).is_some() {
                valid_count += 1;
            }
        }

        // Expected: 50 + 25 + 180 + 1 = 256
        assert_eq!(valid_count, 256);
    }
}
