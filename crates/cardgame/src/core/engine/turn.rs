//! Turn management functions.
//!
//! Handles turn start/end logic including:
//! - Essence and AP restoration
//! - Card draw
//! - Creature status reset
//! - Regenerate, Ephemeral, and Frenzy processing
//! - Support durability and triggered effects

use crate::core::cards::{CardDatabase, CardType};
use crate::core::config::{game, player};
use crate::core::effects::{EffectSource, EffectTarget, Trigger};
use crate::core::state::GameState;
use crate::core::types::{CardId, PlayerId, Slot};

use super::effect_queue::EffectQueue;
use super::init::draw_card;
use super::passive::{
    collect_commander_start_of_turn_effects, remove_support_passives_from_all_creatures,
    support_effect_def_to_effect,
};
use super::victory::check_turn_limit_victory;

/// Start the current player's turn.
///
/// - Increment turn counter
/// - Check turn limit victory condition
/// - Increase max essence by 1 (cap at 10)
/// - Refill current essence to max
/// - Draw a card
/// - Restore AP to 3
/// - Reset creature attack flags (clear exhausted status)
/// - Reset Commander's Insight usage flag
/// - Process Regenerate healing
/// - Process StartOfTurn triggers for supports and commander
pub fn start_turn(state: &mut GameState, card_db: &CardDatabase, effect_queue: &mut EffectQueue) {
    // Increment turn counter
    state.current_turn += 1;

    // Check for turn limit win condition
    if state.current_turn > game::TURN_LIMIT as u16 {
        check_turn_limit_victory(state);
        return;
    }

    let current_player = state.active_player;
    let player_state = &mut state.players[current_player.index()];

    // Reset Commander's Insight usage for this turn
    player_state.used_commander_insight = false;

    // Increase max essence by 1 (capped at MAX_ESSENCE)
    if player_state.max_essence < player::MAX_ESSENCE {
        player_state.max_essence += player::ESSENCE_PER_TURN;
    }

    // Refill current essence to max
    player_state.current_essence = player_state.max_essence;

    // Draw a card
    draw_card(state, current_player);

    // Restore AP to 3
    state.players[current_player.index()].action_points = player::AP_PER_TURN;

    // Reset creature attack flags (clear exhausted status)
    // Creatures that survived a full round can now attack
    for creature in &mut state.players[current_player.index()].creatures {
        creature.status.set_exhausted(false);
    }

    // Process Regenerate - creatures with this keyword heal 2 HP at start of turn
    process_regenerate_healing(state, current_player);

    // Process StartOfTurn triggered effects for supports
    process_support_start_of_turn_triggers(state, card_db, effect_queue, current_player);

    // Process StartOfTurn triggered effects for commander
    process_commander_start_of_turn_triggers(state, card_db, effect_queue, current_player);

    // Process queued effects
    effect_queue.process_all(state, card_db);
}

/// End the current player's turn.
///
/// - Decrement support durability and remove depleted supports
/// - Process Ephemeral deaths
/// - Reset Frenzy stacks
/// - Switch to opponent
/// - Call start_turn for new player
pub fn end_turn(state: &mut GameState, card_db: &CardDatabase, effect_queue: &mut EffectQueue) {
    let current_player = state.active_player;

    // Decrement support durability and collect supports to remove
    let supports_to_remove: Vec<(Slot, CardId)> = {
        let player_state = &mut state.players[current_player.index()];
        let mut to_remove = Vec::new();

        for support in &mut player_state.supports {
            support.current_durability = support.current_durability.saturating_sub(1);
            if support.current_durability == 0 {
                to_remove.push((support.slot, support.card_id));
            }
        }

        to_remove
    };

    // Remove depleted supports and their passive effects
    for (slot, card_id) in supports_to_remove {
        // Remove passive effects from creatures before removing support
        remove_support_passives_from_all_creatures(
            card_id,
            &mut state.players[current_player.index()].creatures,
            card_db,
        );

        // Remove the support from the board
        state.players[current_player.index()]
            .supports
            .retain(|s| s.slot != slot);
    }

    // Process Ephemeral - creatures with this keyword die at end of turn
    process_ephemeral_deaths(state, card_db, effect_queue, current_player);

    // Reset Frenzy stacks - bonus resets at end of turn
    for creature in &mut state.players[current_player.index()].creatures {
        creature.frenzy_stacks = 0;
    }

    // Switch to opponent
    state.active_player = state.active_player.opponent();

    // Start opponent's turn
    start_turn(state, card_db, effect_queue);
}

/// Process Regenerate keyword - heal creatures by 2 HP at start of turn.
fn process_regenerate_healing(state: &mut GameState, player: PlayerId) {
    const REGEN_AMOUNT: i8 = 2;

    for creature in &mut state.players[player.index()].creatures {
        // Skip dead/dying creatures (health <= 0) - they'll be removed by death processing
        if creature.current_health <= 0 {
            continue;
        }

        if creature.keywords.has_regenerate() && creature.current_health < creature.max_health {
            // Use saturating add to prevent overflow from edge cases
            creature.current_health =
                creature.current_health.saturating_add(REGEN_AMOUNT).min(creature.max_health);
        }
    }
}

/// Process Ephemeral keyword - creatures with this keyword die at end of turn.
fn process_ephemeral_deaths(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    player: PlayerId,
) {
    // Collect slots of ephemeral creatures to process
    let ephemeral_slots: Vec<Slot> = state.players[player.index()]
        .creatures
        .iter()
        .filter(|c| c.keywords.has_ephemeral())
        .map(|c| c.slot)
        .collect();

    if ephemeral_slots.is_empty() {
        return;
    }

    // Queue OnDeath triggers for each ephemeral creature before removing them
    for slot in &ephemeral_slots {
        if let Some(creature) = state.players[player.index()].get_creature(*slot) {
            let card_id = creature.card_id;

            // Check for OnDeath triggers
            if let Some(card_def) = card_db.get(card_id) {
                if let CardType::Creature { abilities, .. } = &card_def.card_type {
                    for ability in abilities {
                        if ability.trigger == Trigger::OnDeath {
                            let source = EffectSource::Creature { owner: player, slot: *slot };
                            for effect_def in &ability.effects {
                                if let Some(effect) = crate::core::engine::effect_convert::effect_def_to_effect_with_target(
                                    effect_def,
                                    EffectTarget::Creature { owner: player, slot: *slot },
                                    player,
                                ) {
                                    effect_queue.push(effect, source);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Remove ephemeral creatures
    state.players[player.index()]
        .creatures
        .retain(|c| !c.keywords.has_ephemeral());

    // Process any OnDeath effects
    effect_queue.process_all(state, card_db);
}

/// Process StartOfTurn triggered effects for a player's supports.
fn process_support_start_of_turn_triggers(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    player: PlayerId,
) {
    // Collect support info to avoid borrow conflicts
    let support_info: Vec<(Slot, CardId)> = state.players[player.index()]
        .supports
        .iter()
        .map(|s| (s.slot, s.card_id))
        .collect();

    for (slot, card_id) in support_info {
        if let Some(card_def) = card_db.get(card_id) {
            if let CardType::Support { triggered_effects, .. } = &card_def.card_type {
                for ability in triggered_effects {
                    if ability.trigger == Trigger::StartOfTurn {
                        let source = EffectSource::Support { owner: player, slot };
                        for effect_def in &ability.effects {
                            // Use support-specific effect conversion
                            if let Some(effect) = support_effect_def_to_effect(
                                effect_def,
                                player,
                                ability,
                            ) {
                                effect_queue.push(effect, source);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Process StartOfTurn triggered effects for a player's commander.
fn process_commander_start_of_turn_triggers(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    player: PlayerId,
) {
    for (effect, source) in collect_commander_start_of_turn_effects(state, player, card_db) {
        effect_queue.push(effect, source);
    }
}
