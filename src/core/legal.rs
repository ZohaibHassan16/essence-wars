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

    for (hand_idx, card_instance) in player.hand.iter().enumerate() {
        // Look up the card definition
        let Some(card_def) = card_db.get(card_instance.card_id) else {
            continue;
        };

        // Check if player can afford the card:
        // - 1 AP per action (playing a card is an action)
        // - Essence equal to card's cost
        if player.action_points < 1 || card_def.cost > player.current_essence {
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
