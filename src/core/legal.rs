//! Legal action generation for the card game engine.
//!
//! This module generates all legal actions from a GameState, which is critical for:
//! - Game rule enforcement
//! - MCTS exploration
//! - Neural network action masking

use arrayvec::ArrayVec;
use crate::core::config::board;
use crate::core::types::Slot;
use crate::core::actions::{Action, Target};
use crate::core::state::GameState;
use crate::core::cards::{CardDatabase, CardType};
use crate::core::effects::{Trigger, TargetingRule};

/// Maximum number of legal actions possible in any game state
pub const MAX_LEGAL_ACTIONS: usize = 64;

/// Generate all legal actions for the current player
pub fn legal_actions(
    state: &GameState,
    card_db: &CardDatabase,
) -> ArrayVec<Action, MAX_LEGAL_ACTIONS> {
    let mut actions = ArrayVec::new();

    // If game is over, no actions are legal
    if state.is_terminal() {
        return actions;
    }

    // Generate PlayCard actions
    generate_play_card_actions(state, card_db, &mut actions);

    // Generate Attack actions
    generate_attack_actions(state, &mut actions);

    // Generate UseAbility actions
    generate_ability_actions(state, card_db, &mut actions);

    // EndTurn is always legal
    actions.push(Action::EndTurn);

    actions
}

/// Generate a legal action mask (256 bools) for neural network output masking
pub fn legal_action_mask(
    state: &GameState,
    card_db: &CardDatabase,
) -> [bool; 256] {
    let mut mask = [false; 256];
    for action in legal_actions(state, card_db) {
        mask[action.to_index() as usize] = true;
    }
    mask
}

/// Generate all legal PlayCard actions
fn generate_play_card_actions(
    state: &GameState,
    card_db: &CardDatabase,
    actions: &mut ArrayVec<Action, MAX_LEGAL_ACTIONS>,
) {
    let player = state.active_player_state();
    let ap = player.action_points;

    for (hand_idx, card_instance) in player.hand.iter().enumerate() {
        // Look up the card definition
        let Some(card_def) = card_db.get(card_instance.card_id) else {
            continue;
        };

        // Check if player can afford the card
        if card_def.cost > ap {
            continue;
        }

        match &card_def.card_type {
            CardType::Creature { .. } => {
                // For creatures: generate action for each empty creature slot
                for slot_idx in 0..board::CREATURE_SLOTS as u8 {
                    let slot = Slot(slot_idx);
                    if player.get_creature(slot).is_none() {
                        if actions.len() < MAX_LEGAL_ACTIONS {
                            actions.push(Action::PlayCard {
                                hand_index: hand_idx as u8,
                                slot,
                            });
                        }
                    }
                }
            }
            CardType::Spell { .. } => {
                // For spells: slot 0 is used (spells don't occupy slots)
                // We generate one action per spell with slot 0
                if actions.len() < MAX_LEGAL_ACTIONS {
                    actions.push(Action::PlayCard {
                        hand_index: hand_idx as u8,
                        slot: Slot(0),
                    });
                }
            }
            CardType::Support { .. } => {
                // For supports: generate action for each empty support slot
                for slot_idx in 0..board::SUPPORT_SLOTS as u8 {
                    let slot = Slot(slot_idx);
                    if player.get_support(slot).is_none() {
                        if actions.len() < MAX_LEGAL_ACTIONS {
                            actions.push(Action::PlayCard {
                                hand_index: hand_idx as u8,
                                slot,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Generate all legal Attack actions
fn generate_attack_actions(
    state: &GameState,
    actions: &mut ArrayVec<Action, MAX_LEGAL_ACTIONS>,
) {
    let player_state = state.active_player_state();
    let opponent_state = state.opponent_state();

    // Check if any enemy creature has GUARD
    let guards_present = opponent_state.creatures.iter()
        .any(|c| c.keywords.has_guard());

    // For each of our creatures that can attack
    for attacker in &player_state.creatures {
        // Check if creature can attack
        if !attacker.can_attack(state.current_turn) {
            continue;
        }

        let attacker_slot = attacker.slot;
        let has_ranged = attacker.keywords.has_ranged();

        // Determine valid target slots
        if has_ranged {
            // Ranged creatures can target any enemy slot (0-4)
            for defender_slot_idx in 0..board::CREATURE_SLOTS as u8 {
                let defender_slot = Slot(defender_slot_idx);

                // Check GUARD enforcement for ranged attacks on creatures
                if guards_present {
                    // Must target a creature with GUARD if any exists
                    if let Some(defender) = opponent_state.get_creature(defender_slot) {
                        if !defender.keywords.has_guard() {
                            continue;
                        }
                    } else {
                        // Cannot attack empty slot if guards are present
                        continue;
                    }
                }

                if actions.len() < MAX_LEGAL_ACTIONS {
                    actions.push(Action::Attack {
                        attacker: attacker_slot,
                        defender: defender_slot,
                    });
                }
            }
        } else {
            // Non-ranged creatures can only target adjacent slots
            for &defender_slot in attacker_slot.adjacent_slots() {
                // Check GUARD enforcement for non-ranged attacks
                if guards_present {
                    // Must target a creature with GUARD if any exists
                    if let Some(defender) = opponent_state.get_creature(defender_slot) {
                        if !defender.keywords.has_guard() {
                            continue;
                        }
                    } else {
                        // Cannot attack empty slot if guards are present
                        continue;
                    }
                }

                if actions.len() < MAX_LEGAL_ACTIONS {
                    actions.push(Action::Attack {
                        attacker: attacker_slot,
                        defender: defender_slot,
                    });
                }
            }
        }
    }
}

/// Generate all legal UseAbility actions
fn generate_ability_actions(
    state: &GameState,
    card_db: &CardDatabase,
    actions: &mut ArrayVec<Action, MAX_LEGAL_ACTIONS>,
) {
    let player_state = state.active_player_state();
    let opponent = state.active_player.opponent();

    // For each of our creatures with abilities
    for creature in &player_state.creatures {
        // Silenced creatures cannot use abilities
        if creature.status.is_silenced() {
            continue;
        }

        // Look up the card definition to get abilities
        let Some(card_def) = card_db.get(creature.card_id) else {
            continue;
        };

        // Get creature abilities
        let Some(abilities) = card_def.creature_abilities() else {
            continue;
        };

        // Check each ability
        for (ability_idx, ability) in abilities.iter().enumerate() {
            // Only activated abilities are usable via UseAbility action
            // Check if this is an activated ability (using Trigger types)
            // For now, we consider OnPlay as the only "activated" trigger
            // that can be used via UseAbility (this may need refinement)
            if !matches!(ability.trigger, Trigger::OnPlay) {
                continue;
            }

            // Generate actions based on targeting rule
            match &ability.targeting {
                TargetingRule::NoTarget => {
                    // NoTarget abilities can target self
                    if actions.len() < MAX_LEGAL_ACTIONS {
                        actions.push(Action::UseAbility {
                            slot: creature.slot,
                            ability_index: ability_idx as u8,
                            target: Target::Self_,
                        });
                    }
                }
                TargetingRule::TargetCreature(_) | TargetingRule::TargetAny => {
                    // Target any enemy creature slot
                    for slot_idx in 0..board::CREATURE_SLOTS as u8 {
                        if actions.len() < MAX_LEGAL_ACTIONS {
                            actions.push(Action::UseAbility {
                                slot: creature.slot,
                                ability_index: ability_idx as u8,
                                target: Target::EnemySlot(Slot(slot_idx)),
                            });
                        }
                    }
                    // Also allow targeting self for TargetAny
                    if matches!(ability.targeting, TargetingRule::TargetAny) {
                        if actions.len() < MAX_LEGAL_ACTIONS {
                            actions.push(Action::UseAbility {
                                slot: creature.slot,
                                ability_index: ability_idx as u8,
                                target: Target::Self_,
                            });
                        }
                    }
                }
                TargetingRule::TargetEnemyCreature => {
                    // Target enemy creature slots that have creatures
                    let opponent_state = &state.players[opponent.index()];
                    for opp_creature in &opponent_state.creatures {
                        if actions.len() < MAX_LEGAL_ACTIONS {
                            actions.push(Action::UseAbility {
                                slot: creature.slot,
                                ability_index: ability_idx as u8,
                                target: Target::EnemySlot(opp_creature.slot),
                            });
                        }
                    }
                }
                TargetingRule::TargetAllyCreature => {
                    // Target self (the creature using the ability)
                    if actions.len() < MAX_LEGAL_ACTIONS {
                        actions.push(Action::UseAbility {
                            slot: creature.slot,
                            ability_index: ability_idx as u8,
                            target: Target::Self_,
                        });
                    }
                }
                TargetingRule::TargetPlayer | TargetingRule::TargetEnemyPlayer => {
                    // Target player - use NoTarget as we don't have player target
                    if actions.len() < MAX_LEGAL_ACTIONS {
                        actions.push(Action::UseAbility {
                            slot: creature.slot,
                            ability_index: ability_idx as u8,
                            target: Target::NoTarget,
                        });
                    }
                }
                TargetingRule::TargetSlot => {
                    // Target any slot
                    for slot_idx in 0..board::CREATURE_SLOTS as u8 {
                        if actions.len() < MAX_LEGAL_ACTIONS {
                            actions.push(Action::UseAbility {
                                slot: creature.slot,
                                ability_index: ability_idx as u8,
                                target: Target::EnemySlot(Slot(slot_idx)),
                            });
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::{Creature, CreatureStatus};
    use crate::core::types::{CreatureInstanceId, PlayerId};
    use crate::core::cards::{CardDefinition, CardType};
    use crate::core::keywords::Keywords;

    /// Create a test card database with some basic cards
    fn test_card_db() -> CardDatabase {
        let cards = vec![
            // Basic creature: cost 2, 2/3
            CardDefinition {
                id: 1,
                name: "Test Creature".to_string(),
                cost: 2,
                card_type: CardType::Creature {
                    attack: 2,
                    health: 3,
                    keywords: vec![],
                    abilities: vec![],
                },
                rarity: crate::types::Rarity::Common,
                tags: vec![],
            },
            // Expensive creature: cost 5, 4/4
            CardDefinition {
                id: 2,
                name: "Expensive Creature".to_string(),
                cost: 5,
                card_type: CardType::Creature {
                    attack: 4,
                    health: 4,
                    keywords: vec![],
                    abilities: vec![],
                },
                rarity: crate::types::Rarity::Uncommon,
                tags: vec![],
            },
            // Ranged creature: cost 3, 2/2, Ranged
            CardDefinition {
                id: 3,
                name: "Ranged Creature".to_string(),
                cost: 3,
                card_type: CardType::Creature {
                    attack: 2,
                    health: 2,
                    keywords: vec!["Ranged".to_string()],
                    abilities: vec![],
                },
                rarity: crate::types::Rarity::Common,
                tags: vec![],
            },
            // Guard creature: cost 2, 1/4, Guard
            CardDefinition {
                id: 4,
                name: "Guard Creature".to_string(),
                cost: 2,
                card_type: CardType::Creature {
                    attack: 1,
                    health: 4,
                    keywords: vec!["Guard".to_string()],
                    abilities: vec![],
                },
                rarity: crate::types::Rarity::Common,
                tags: vec![],
            },
            // Test spell: cost 1
            CardDefinition {
                id: 5,
                name: "Test Spell".to_string(),
                cost: 1,
                card_type: CardType::Spell {
                    targeting: TargetingRule::NoTarget,
                    effects: vec![],
                },
                rarity: crate::types::Rarity::Common,
                tags: vec![],
            },
            // Test support: cost 3
            CardDefinition {
                id: 6,
                name: "Test Support".to_string(),
                cost: 3,
                card_type: CardType::Support {
                    durability: 2,
                    passive_effects: vec![],
                    triggered_effects: vec![],
                },
                rarity: crate::types::Rarity::Common,
                tags: vec![],
            },
        ];
        CardDatabase::new(cards)
    }

    /// Create a basic test creature
    fn make_creature(card_id: u16, slot: u8, owner: PlayerId, turn_played: u16, keywords: Keywords) -> Creature {
        Creature {
            instance_id: CreatureInstanceId(slot as u32),
            card_id: crate::types::CardId(card_id),
            owner,
            slot: Slot(slot),
            attack: 2,
            current_health: 3,
            max_health: 3,
            base_attack: 2,
            base_health: 3,
            keywords,
            status: CreatureStatus::default(),
            turn_played,
        }
    }

    #[test]
    fn test_empty_board_only_end_turn() {
        let state = GameState::new();
        let card_db = CardDatabase::empty();

        let actions = legal_actions(&state, &card_db);

        // Only EndTurn should be legal
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::EndTurn);
    }

    #[test]
    fn test_hand_with_playable_card() {
        let card_db = test_card_db();
        let mut state = GameState::new();

        // Give player 3 AP and a card in hand (cost 2)
        state.players[0].action_points = 3;
        state.players[0].hand.push(crate::state::CardInstance::new(crate::types::CardId(1)));

        let actions = legal_actions(&state, &card_db);

        // Should have PlayCard actions for all 5 empty slots + EndTurn
        assert_eq!(actions.len(), 6);

        // Verify we have PlayCard actions for slots 0-4
        let play_card_count = actions.iter().filter(|a| matches!(a, Action::PlayCard { .. })).count();
        assert_eq!(play_card_count, 5);
    }

    #[test]
    fn test_not_enough_ap() {
        let card_db = test_card_db();
        let mut state = GameState::new();

        // Give player 1 AP and a card that costs 5
        state.players[0].action_points = 1;
        state.players[0].hand.push(crate::state::CardInstance::new(crate::types::CardId(2)));

        let actions = legal_actions(&state, &card_db);

        // Only EndTurn should be legal (can't afford the card)
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::EndTurn);
    }

    #[test]
    fn test_creature_attack_adjacent_targeting() {
        let card_db = test_card_db();
        let mut state = GameState::new();
        state.current_turn = 2; // Turn 2 so creatures aren't summoning sick

        // Place a creature at slot 2 (can attack slots 1, 2, 3)
        let creature = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none());
        state.players[0].creatures.push(creature);

        let actions = legal_actions(&state, &card_db);

        // Should have attacks on adjacent slots (1, 2, 3) + EndTurn
        let attack_count = actions.iter().filter(|a| matches!(a, Action::Attack { .. })).count();
        assert_eq!(attack_count, 3);

        // Verify specific attacks
        assert!(actions.contains(&Action::Attack { attacker: Slot(2), defender: Slot(1) }));
        assert!(actions.contains(&Action::Attack { attacker: Slot(2), defender: Slot(2) }));
        assert!(actions.contains(&Action::Attack { attacker: Slot(2), defender: Slot(3) }));

        // Should NOT be able to attack slot 0 or 4
        assert!(!actions.contains(&Action::Attack { attacker: Slot(2), defender: Slot(0) }));
        assert!(!actions.contains(&Action::Attack { attacker: Slot(2), defender: Slot(4) }));
    }

    #[test]
    fn test_ranged_creature_any_target() {
        let card_db = test_card_db();
        let mut state = GameState::new();
        state.current_turn = 2;

        // Place a ranged creature at slot 0
        let creature = make_creature(3, 0, PlayerId::PLAYER_ONE, 1, Keywords::none().with_ranged());
        state.players[0].creatures.push(creature);

        let actions = legal_actions(&state, &card_db);

        // Ranged should be able to attack all 5 slots
        let attack_count = actions.iter().filter(|a| matches!(a, Action::Attack { .. })).count();
        assert_eq!(attack_count, 5);

        // Verify can attack any slot
        for slot in 0..5 {
            assert!(actions.contains(&Action::Attack { attacker: Slot(0), defender: Slot(slot) }));
        }
    }

    #[test]
    fn test_guard_enforcement() {
        let card_db = test_card_db();
        let mut state = GameState::new();
        state.current_turn = 2;

        // Place an attacker at slot 2 for player 1
        let attacker = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none());
        state.players[0].creatures.push(attacker);

        // Place a guard creature at slot 1 for player 2
        let guard = make_creature(4, 1, PlayerId::PLAYER_TWO, 1, Keywords::none().with_guard());
        state.players[1].creatures.push(guard);

        // Place a non-guard creature at slot 3 for player 2
        let non_guard = make_creature(1, 3, PlayerId::PLAYER_TWO, 1, Keywords::none());
        state.players[1].creatures.push(non_guard);

        let actions = legal_actions(&state, &card_db);

        // Can only attack the guard creature
        let attacks: Vec<_> = actions.iter().filter(|a| matches!(a, Action::Attack { .. })).collect();
        assert_eq!(attacks.len(), 1);
        assert!(actions.contains(&Action::Attack { attacker: Slot(2), defender: Slot(1) }));

        // Cannot attack the non-guard or empty slots
        assert!(!actions.contains(&Action::Attack { attacker: Slot(2), defender: Slot(3) }));
        assert!(!actions.contains(&Action::Attack { attacker: Slot(2), defender: Slot(2) }));
    }

    #[test]
    fn test_guard_enforcement_ranged() {
        let card_db = test_card_db();
        let mut state = GameState::new();
        state.current_turn = 2;

        // Place a ranged attacker at slot 0 for player 1
        let attacker = make_creature(3, 0, PlayerId::PLAYER_ONE, 1, Keywords::none().with_ranged());
        state.players[0].creatures.push(attacker);

        // Place a guard creature at slot 4 for player 2
        let guard = make_creature(4, 4, PlayerId::PLAYER_TWO, 1, Keywords::none().with_guard());
        state.players[1].creatures.push(guard);

        // Place a non-guard creature at slot 1 for player 2
        let non_guard = make_creature(1, 1, PlayerId::PLAYER_TWO, 1, Keywords::none());
        state.players[1].creatures.push(non_guard);

        let actions = legal_actions(&state, &card_db);

        // Ranged can still only attack guard when guards are present
        let attacks: Vec<_> = actions.iter().filter(|a| matches!(a, Action::Attack { .. })).collect();
        assert_eq!(attacks.len(), 1);
        assert!(actions.contains(&Action::Attack { attacker: Slot(0), defender: Slot(4) }));
    }

    #[test]
    fn test_summoning_sickness() {
        let card_db = test_card_db();
        let mut state = GameState::new();
        state.current_turn = 1;

        // Place a creature played this turn (has summoning sickness)
        let creature = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none());
        state.players[0].creatures.push(creature);

        let actions = legal_actions(&state, &card_db);

        // No attacks should be possible (summoning sickness)
        let attack_count = actions.iter().filter(|a| matches!(a, Action::Attack { .. })).count();
        assert_eq!(attack_count, 0);
    }

    #[test]
    fn test_rush_ignores_summoning_sickness() {
        let card_db = test_card_db();
        let mut state = GameState::new();
        state.current_turn = 1;

        // Place a Rush creature played this turn
        let creature = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none().with_rush());
        state.players[0].creatures.push(creature);

        let actions = legal_actions(&state, &card_db);

        // Rush creature CAN attack on the turn it's played
        let attack_count = actions.iter().filter(|a| matches!(a, Action::Attack { .. })).count();
        assert!(attack_count > 0);
    }

    #[test]
    fn test_legal_action_mask() {
        let card_db = test_card_db();
        let mut state = GameState::new();

        // Give player 3 AP and a card in hand
        state.players[0].action_points = 3;
        state.players[0].hand.push(crate::state::CardInstance::new(crate::types::CardId(1)));

        let mask = legal_action_mask(&state, &card_db);

        // EndTurn (index 255) should be legal
        assert!(mask[255]);

        // PlayCard for hand_index=0, slots 0-4 should be legal
        // Index = hand_idx * 5 + slot
        for slot in 0..5 {
            assert!(mask[slot], "PlayCard(0, {}) should be legal", slot);
        }

        // Other PlayCard indices should be false
        assert!(!mask[5]); // hand_index=1, slot=0 (no card at index 1)
    }

    #[test]
    fn test_spell_play_action() {
        let card_db = test_card_db();
        let mut state = GameState::new();

        // Give player AP and a spell in hand
        state.players[0].action_points = 5;
        state.players[0].hand.push(crate::state::CardInstance::new(crate::types::CardId(5))); // Spell

        let actions = legal_actions(&state, &card_db);

        // Should have PlayCard for the spell + EndTurn
        let play_count = actions.iter().filter(|a| matches!(a, Action::PlayCard { .. })).count();
        assert_eq!(play_count, 1); // Spells only get one PlayCard action

        assert!(actions.contains(&Action::PlayCard { hand_index: 0, slot: Slot(0) }));
    }

    #[test]
    fn test_support_play_action() {
        let card_db = test_card_db();
        let mut state = GameState::new();

        // Give player AP and a support in hand
        state.players[0].action_points = 5;
        state.players[0].hand.push(crate::state::CardInstance::new(crate::types::CardId(6))); // Support

        let actions = legal_actions(&state, &card_db);

        // Should have PlayCard for 2 support slots + EndTurn
        let play_count = actions.iter().filter(|a| matches!(a, Action::PlayCard { .. })).count();
        assert_eq!(play_count, 2); // Two support slots

        assert!(actions.contains(&Action::PlayCard { hand_index: 0, slot: Slot(0) }));
        assert!(actions.contains(&Action::PlayCard { hand_index: 0, slot: Slot(1) }));
    }

    #[test]
    fn test_occupied_creature_slot() {
        let card_db = test_card_db();
        let mut state = GameState::new();

        // Give player AP and a creature card in hand
        state.players[0].action_points = 5;
        state.players[0].hand.push(crate::state::CardInstance::new(crate::types::CardId(1))); // Creature

        // Occupy slots 0 and 2
        let creature1 = make_creature(1, 0, PlayerId::PLAYER_ONE, 0, Keywords::none());
        let creature2 = make_creature(1, 2, PlayerId::PLAYER_ONE, 0, Keywords::none());
        state.players[0].creatures.push(creature1);
        state.players[0].creatures.push(creature2);

        let actions = legal_actions(&state, &card_db);

        // Should only be able to play to empty slots 1, 3, 4
        let play_actions: Vec<_> = actions.iter()
            .filter_map(|a| match a {
                Action::PlayCard { hand_index, slot } => Some((*hand_index, slot.0)),
                _ => None,
            })
            .collect();

        assert_eq!(play_actions.len(), 3);
        assert!(play_actions.contains(&(0, 1)));
        assert!(play_actions.contains(&(0, 3)));
        assert!(play_actions.contains(&(0, 4)));
    }

    #[test]
    fn test_exhausted_creature_cannot_attack() {
        let card_db = test_card_db();
        let mut state = GameState::new();
        state.current_turn = 2;

        // Place an exhausted creature
        let mut creature = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none());
        creature.status.set_exhausted(true);
        state.players[0].creatures.push(creature);

        let actions = legal_actions(&state, &card_db);

        // No attacks should be possible (exhausted)
        let attack_count = actions.iter().filter(|a| matches!(a, Action::Attack { .. })).count();
        assert_eq!(attack_count, 0);
    }

    #[test]
    fn test_terminal_state_no_actions() {
        let card_db = test_card_db();
        let mut state = GameState::new();

        // Mark game as ended
        state.result = Some(crate::state::GameResult::Draw);

        let actions = legal_actions(&state, &card_db);

        // No actions should be legal when game is over
        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn test_multiple_cards_in_hand() {
        let card_db = test_card_db();
        let mut state = GameState::new();

        // Give player 5 AP and two cards in hand
        state.players[0].action_points = 5;
        state.players[0].hand.push(crate::state::CardInstance::new(crate::types::CardId(1))); // cost 2 creature
        state.players[0].hand.push(crate::state::CardInstance::new(crate::types::CardId(2))); // cost 5 creature

        let actions = legal_actions(&state, &card_db);

        // Both cards can be played (5 AP available)
        // Card 0: 5 slots, Card 1: 5 slots = 10 PlayCard + EndTurn
        let play_count = actions.iter().filter(|a| matches!(a, Action::PlayCard { .. })).count();
        assert_eq!(play_count, 10);
    }

    #[test]
    fn test_attack_empty_slot_face_damage() {
        let card_db = test_card_db();
        let mut state = GameState::new();
        state.current_turn = 2;

        // Place a creature that can attack
        let creature = make_creature(1, 2, PlayerId::PLAYER_ONE, 1, Keywords::none());
        state.players[0].creatures.push(creature);

        // No enemy creatures on board

        let actions = legal_actions(&state, &card_db);

        // Should be able to attack empty slots (for face damage)
        let attack_count = actions.iter().filter(|a| matches!(a, Action::Attack { .. })).count();
        assert_eq!(attack_count, 3); // Adjacent slots 1, 2, 3
    }
}
