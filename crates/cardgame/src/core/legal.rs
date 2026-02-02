//! Legal action generation for the card game engine.
//!
//! This module generates all legal actions from a GameState, which is critical for:
//! - Game rule enforcement
//! - MCTS exploration
//! - Neural network action masking

use arrayvec::ArrayVec;
use crate::core::config::{board, insight};
use crate::core::types::Slot;
use crate::core::actions::{Action, Target};
use crate::core::state::GameState;
use crate::core::cards::{CardDatabase, CardType};
use crate::core::effects::{TargetingRule, Trigger};

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

    // Generate CommanderInsight action (if eligible)
    if is_commander_insight_legal(state) && actions.len() < MAX_LEGAL_ACTIONS {
        actions.push(Action::CommanderInsight);
    }

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
                    if player.get_creature(slot).is_none()
                        && actions.len() < MAX_LEGAL_ACTIONS {
                        actions.push(Action::PlayCard {
                            hand_index: hand_idx as u8,
                            slot,
                        });
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
                    if player.get_support(slot).is_none()
                        && actions.len() < MAX_LEGAL_ACTIONS {
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

/// Generate all legal Attack actions
fn generate_attack_actions(
    state: &GameState,
    actions: &mut ArrayVec<Action, MAX_LEGAL_ACTIONS>,
) {
    let player_state = state.active_player_state();
    let opponent_state = state.opponent_state();

    // Check if any enemy creature has GUARD (and is not stealthed - Stealth masks Guard)
    let guards_present = opponent_state.creatures.iter()
        .any(|c| c.keywords.has_guard() && !c.keywords.has_stealth());

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

                // Check if target creature has STEALTH (can't be targeted)
                if let Some(defender) = opponent_state.get_creature(defender_slot) {
                    if defender.keywords.has_stealth() {
                        continue; // Can't attack stealthed creatures
                    }
                }

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
                // Check if target creature has STEALTH (can't be targeted)
                if let Some(defender) = opponent_state.get_creature(defender_slot) {
                    if defender.keywords.has_stealth() {
                        continue; // Can't attack stealthed creatures
                    }
                }

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
///
/// This generates actions for:
/// 1. Token abilities (stored in creature.token_data)
/// 2. Card-based activated abilities (with Trigger::Activated)
///
/// UseAbility actions use action space indices 150-249.
fn generate_ability_actions(
    state: &GameState,
    card_db: &CardDatabase,
    actions: &mut ArrayVec<Action, MAX_LEGAL_ACTIONS>,
) {
    let player = state.active_player_state();
    let opponent = state.opponent_state();

    for creature in &player.creatures {
        // Skip silenced creatures - they can't use abilities
        if creature.status.is_silenced() {
            continue;
        }

        // Skip exhausted creatures - activated abilities require tapping
        if creature.status.is_exhausted() {
            continue;
        }

        // Check token abilities first (using helper method for token_data access)
        if let Some(abilities) = creature.token_abilities() {
            generate_actions_for_abilities_token(
                creature.slot,
                abilities,
                player.current_essence,
                opponent,
                actions,
            );
        }

        // Check card-based activated abilities
        if let Some(card_def) = card_db.get(creature.card_id) {
            if let CardType::Creature { abilities, .. } = &card_def.card_type {
                generate_actions_for_abilities_card(
                    creature.slot,
                    abilities,
                    player.current_essence,
                    player,
                    opponent,
                    actions,
                );
            }
        }
    }
}

/// Generate UseAbility actions for token abilities
fn generate_actions_for_abilities_token(
    slot: Slot,
    abilities: &[crate::core::effects::TokenAbility],
    current_essence: u8,
    opponent: &crate::core::state::PlayerState,
    actions: &mut ArrayVec<Action, MAX_LEGAL_ACTIONS>,
) {
    for (ability_idx, ability) in abilities.iter().enumerate() {
        // Check essence cost
        if ability.essence_cost > current_essence {
            continue;
        }

        generate_targeting_actions(
            slot,
            ability_idx as u8,
            &ability.targeting,
            None, // player state not needed for token abilities
            opponent,
            actions,
        );
    }
}

/// Generate UseAbility actions for card-based abilities
fn generate_actions_for_abilities_card(
    slot: Slot,
    abilities: &[crate::core::cards::AbilityDefinition],
    current_essence: u8,
    player: &crate::core::state::PlayerState,
    opponent: &crate::core::state::PlayerState,
    actions: &mut ArrayVec<Action, MAX_LEGAL_ACTIONS>,
) {
    for (ability_idx, ability) in abilities.iter().enumerate() {
        // Only process Activated trigger abilities
        if ability.trigger != Trigger::Activated {
            continue;
        }

        // Check essence cost
        if ability.essence_cost > current_essence {
            continue;
        }

        generate_targeting_actions(
            slot,
            ability_idx as u8,
            &ability.targeting,
            Some(player),
            opponent,
            actions,
        );
    }
}

/// Generate actions based on targeting rule (shared by token and card abilities)
fn generate_targeting_actions(
    slot: Slot,
    ability_idx: u8,
    targeting: &TargetingRule,
    player: Option<&crate::core::state::PlayerState>,
    opponent: &crate::core::state::PlayerState,
    actions: &mut ArrayVec<Action, MAX_LEGAL_ACTIONS>,
) {
    match targeting {
        TargetingRule::TargetAny => {
            // Can target any enemy creature
            for enemy_creature in &opponent.creatures {
                if enemy_creature.keywords.has_stealth() {
                    continue;
                }
                if actions.len() < MAX_LEGAL_ACTIONS {
                    actions.push(Action::UseAbility {
                        slot,
                        ability_index: ability_idx,
                        target: Target::EnemySlot(enemy_creature.slot),
                    });
                }
            }
            // Can also target enemy commander (NoTarget = face)
            if actions.len() < MAX_LEGAL_ACTIONS {
                actions.push(Action::UseAbility {
                    slot,
                    ability_index: ability_idx,
                    target: Target::NoTarget,
                });
            }
        }
        TargetingRule::TargetEnemyCreature => {
            for enemy_creature in &opponent.creatures {
                if enemy_creature.keywords.has_stealth() {
                    continue;
                }
                if actions.len() < MAX_LEGAL_ACTIONS {
                    actions.push(Action::UseAbility {
                        slot,
                        ability_index: ability_idx,
                        target: Target::EnemySlot(enemy_creature.slot),
                    });
                }
            }
        }
        TargetingRule::TargetAllyCreature => {
            // Can target any ally creature
            if let Some(player_state) = player {
                if let Some(ally_creature) = player_state.creatures.iter().next() {
                    if actions.len() < MAX_LEGAL_ACTIONS {
                        actions.push(Action::UseAbility {
                            slot,
                            ability_index: ability_idx,
                            target: Target::Self_, // Self_ is used for ally targeting via slot encoding
                        });
                    }
                    // Note: For now, using Self_ for ally targeting. In the future,
                    // we may want to add a Target::AllySlot variant for more precise targeting.
                    let _ = ally_creature; // Use the variable to avoid warning
                    // Only add one action for now (self-target)
                }
            }
        }
        TargetingRule::TargetEnemyPlayer => {
            if actions.len() < MAX_LEGAL_ACTIONS {
                actions.push(Action::UseAbility {
                    slot,
                    ability_index: ability_idx,
                    target: Target::NoTarget,
                });
            }
        }
        TargetingRule::NoTarget => {
            if actions.len() < MAX_LEGAL_ACTIONS {
                actions.push(Action::UseAbility {
                    slot,
                    ability_index: ability_idx,
                    target: Target::NoTarget,
                });
            }
        }
        // Other targeting rules not yet fully implemented
        _ => {}
    }
}

/// Check if Commander's Insight is legal for the current player.
///
/// Commander's Insight is a catch-up mechanic that allows struggling players
/// to draw a card for essence (no AP cost). It's available when:
/// - Turn >= 10 (late game only)
/// - Hand size <= 1 (top-decking)
/// - Behind on creatures OR behind on life (catch-up restriction)
/// - Have enough essence (4)
/// - Haven't used it this turn
pub fn is_commander_insight_legal(state: &GameState) -> bool {
    // Must be turn 10 or later
    if state.current_turn < insight::MIN_TURN {
        return false;
    }

    let player = state.active_player_state();
    let opponent = state.opponent_state();

    // Must not have used insight this turn
    if player.used_commander_insight {
        return false;
    }

    // Must have hand size <= 1 (starving)
    if player.hand.len() > insight::MAX_HAND_SIZE {
        return false;
    }

    // Must have enough essence
    if player.current_essence < insight::ESSENCE_COST {
        return false;
    }

    // Must be behind on creatures OR behind on life (strict inequality)
    let behind_on_creatures = player.creatures.len() < opponent.creatures.len();
    let behind_on_life = player.life < opponent.life;

    if !behind_on_creatures && !behind_on_life {
        return false;
    }

    true
}
