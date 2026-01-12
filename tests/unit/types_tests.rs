//! Unit tests for types module.

use cardgame::types::{PlayerId, Slot};

#[test]
fn test_player_opponent() {
    assert_eq!(PlayerId::PLAYER_ONE.opponent(), PlayerId::PLAYER_TWO);
    assert_eq!(PlayerId::PLAYER_TWO.opponent(), PlayerId::PLAYER_ONE);
}

#[test]
fn test_player_index() {
    assert_eq!(PlayerId::PLAYER_ONE.index(), 0);
    assert_eq!(PlayerId::PLAYER_TWO.index(), 1);
}

#[test]
fn test_slot_adjacency() {
    // Slot 0 can attack 0, 1
    assert!(Slot(0).can_attack(Slot(0)));
    assert!(Slot(0).can_attack(Slot(1)));
    assert!(!Slot(0).can_attack(Slot(2)));

    // Slot 2 (center) can attack 1, 2, 3
    assert!(!Slot(2).can_attack(Slot(0)));
    assert!(Slot(2).can_attack(Slot(1)));
    assert!(Slot(2).can_attack(Slot(2)));
    assert!(Slot(2).can_attack(Slot(3)));
    assert!(!Slot(2).can_attack(Slot(4)));

    // Slot 4 can attack 3, 4
    assert!(!Slot(4).can_attack(Slot(2)));
    assert!(Slot(4).can_attack(Slot(3)));
    assert!(Slot(4).can_attack(Slot(4)));
}
