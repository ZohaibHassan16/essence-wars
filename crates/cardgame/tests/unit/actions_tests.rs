//! Unit tests for actions module.

use cardgame::actions::{Action, Target};
use cardgame::types::Slot;

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
