//! Module for tensor representation of game state.
//!
//! This module converts game state into tensor format suitable for
//! machine learning and AI training purposes (PPO/AlphaZero).

use crate::config::{board, game, player, tensor as tensor_config};
use crate::keywords::Keywords;
use crate::state::{Creature, GameState, GameResult, PlayerState, Support};
use crate::types::Slot;

// Re-export STATE_TENSOR_SIZE from config for backward compatibility
pub use tensor_config::STATE_TENSOR_SIZE;

/// Number of floats per creature slot encoding
const CREATURE_SLOT_SIZE: usize = 10;

/// Number of floats per support slot encoding
const SUPPORT_SLOT_SIZE: usize = 5;

/// Convert a GameState to a neural network input tensor
pub fn state_to_tensor(state: &GameState) -> [f32; STATE_TENSOR_SIZE] {
    let mut tensor = [0.0f32; STATE_TENSOR_SIZE];
    let mut idx = 0;

    // Global state (6 floats)
    // [0]: turn_number normalized
    tensor[idx] = state.current_turn as f32 / game::TURN_LIMIT as f32;
    idx += 1;

    // [1]: current_player (0.0 or 1.0)
    tensor[idx] = state.active_player.0 as f32;
    idx += 1;

    // [2]: game_over (0.0 or 1.0)
    tensor[idx] = if state.result.is_some() { 1.0 } else { 0.0 };
    idx += 1;

    // [3]: winner (-1.0 = no winner, 0.0 = player 0, 1.0 = player 1)
    tensor[idx] = match &state.result {
        None => -1.0,
        Some(GameResult::Draw) => -1.0, // No winner in a draw
        Some(GameResult::Win { winner, .. }) => winner.0 as f32,
    };
    idx += 1;

    // [4-5]: reserved (2 floats)
    idx += 2;

    // Player states (80 floats each)
    for player_idx in 0..2 {
        encode_player_state(&state.players[player_idx], state.current_turn, &mut tensor, &mut idx);
    }

    // Card embedding IDs section
    // The remaining space is filled with card IDs from hands and boards
    // This provides the neural network with card identity information
    encode_card_embeddings(state, &mut tensor, &mut idx);

    tensor
}

/// Encode a single player's state into the tensor
fn encode_player_state(
    player: &PlayerState,
    current_turn: u16,
    tensor: &mut [f32],
    idx: &mut usize,
) {
    // [0]: life normalized (clamped 0-1)
    tensor[*idx] = (player.life as f32 / player::STARTING_LIFE as f32).clamp(0.0, 1.0);
    *idx += 1;

    // [1]: essence normalized (clamped 0-1)
    tensor[*idx] = (player.current_essence as f32 / player::MAX_ESSENCE as f32).clamp(0.0, 1.0);
    *idx += 1;

    // [2]: ap normalized
    tensor[*idx] = player.action_points as f32 / player::AP_PER_TURN as f32;
    *idx += 1;

    // [3]: deck_size normalized
    tensor[*idx] = player.deck.len() as f32 / game::MAX_DECK_SIZE as f32;
    *idx += 1;

    // [4]: hand_size normalized
    tensor[*idx] = player.hand.len() as f32 / player::MAX_HAND_SIZE as f32;
    *idx += 1;

    // [5-14]: hand_cards (card IDs as floats, 0 if empty)
    for i in 0..player::MAX_HAND_SIZE {
        tensor[*idx] = player
            .hand
            .get(i)
            .map(|c| c.card_id.0 as f32)
            .unwrap_or(0.0);
        *idx += 1;
    }

    // [15-64]: board_creatures (slot * 10 floats)
    for slot_idx in 0..board::CREATURE_SLOTS {
        let slot = Slot(slot_idx as u8);
        let creature = player.get_creature(slot);
        encode_creature_slot(creature, current_turn, tensor, idx);
    }

    // [65-74]: board_supports (slot * 5 floats)
    for slot_idx in 0..board::SUPPORT_SLOTS {
        let slot = Slot(slot_idx as u8);
        let support = player.get_support(slot);
        encode_support_slot(support, tensor, idx);
    }
}

/// Encode a creature slot (10 floats)
fn encode_creature_slot(
    creature: Option<&Creature>,
    current_turn: u16,
    tensor: &mut [f32],
    idx: &mut usize,
) {
    match creature {
        None => {
            // 10 zeros for empty slot
            *idx += CREATURE_SLOT_SIZE;
        }
        Some(c) => {
            // [0]: occupied (0.0 or 1.0)
            tensor[*idx] = 1.0;
            *idx += 1;

            // [1]: attack / 10.0 (normalized)
            tensor[*idx] = c.attack as f32 / 10.0;
            *idx += 1;

            // [2]: health / 10.0 (normalized)
            tensor[*idx] = c.current_health as f32 / 10.0;
            *idx += 1;

            // [3]: max_health / 10.0 (normalized)
            tensor[*idx] = c.max_health as f32 / 10.0;
            *idx += 1;

            // [4]: can_attack (0.0 or 1.0)
            tensor[*idx] = if c.can_attack(current_turn) { 1.0 } else { 0.0 };
            *idx += 1;

            // [5]: attacked_this_turn (exhausted) (0.0 or 1.0)
            tensor[*idx] = if c.status.is_exhausted() { 1.0 } else { 0.0 };
            *idx += 1;

            // [6]: silenced (0.0 or 1.0)
            tensor[*idx] = if c.status.is_silenced() { 1.0 } else { 0.0 };
            *idx += 1;

            // [7]: Rush keyword (0.0 or 1.0)
            tensor[*idx] = if c.keywords.has(Keywords::RUSH) { 1.0 } else { 0.0 };
            *idx += 1;

            // [8]: Guard keyword (0.0 or 1.0)
            tensor[*idx] = if c.keywords.has(Keywords::GUARD) { 1.0 } else { 0.0 };
            *idx += 1;

            // [9]: Full keyword bitfield normalized (0-65535 -> 0.0-1.0)
            tensor[*idx] = c.keywords.0 as f32 / 65535.0;
            *idx += 1;
        }
    }
}

/// Encode a support slot (5 floats)
fn encode_support_slot(support: Option<&Support>, tensor: &mut [f32], idx: &mut usize) {
    match support {
        None => {
            // 5 zeros for empty slot
            *idx += SUPPORT_SLOT_SIZE;
        }
        Some(s) => {
            // [0]: occupied (0.0 or 1.0)
            tensor[*idx] = 1.0;
            *idx += 1;

            // [1]: durability / 5.0 (normalized)
            tensor[*idx] = s.current_durability as f32 / 5.0;
            *idx += 1;

            // [2]: card_id as float
            tensor[*idx] = s.card_id.0 as f32;
            *idx += 1;

            // [3-4]: reserved
            *idx += 2;
        }
    }
}

/// Encode card embedding IDs from all hands and boards
fn encode_card_embeddings(state: &GameState, tensor: &mut [f32], idx: &mut usize) {
    // Fill the remaining tensor space with card IDs for embedding lookup
    // This allows the neural network to learn card-specific behaviors

    for player in &state.players {
        // Hand card IDs
        for card in &player.hand {
            if *idx >= STATE_TENSOR_SIZE {
                break;
            }
            tensor[*idx] = card.card_id.0 as f32;
            *idx += 1;
        }

        // Board creature card IDs
        for creature in &player.creatures {
            if *idx >= STATE_TENSOR_SIZE {
                break;
            }
            tensor[*idx] = creature.card_id.0 as f32;
            *idx += 1;
        }

        // Board support card IDs
        for support in &player.supports {
            if *idx >= STATE_TENSOR_SIZE {
                break;
            }
            tensor[*idx] = support.card_id.0 as f32;
            *idx += 1;
        }
    }
}

/// Convert legal action mask to tensor format
pub fn legal_mask_to_tensor(mask: &[bool; 256]) -> [f32; 256] {
    let mut tensor = [0.0f32; 256];
    for (i, &legal) in mask.iter().enumerate() {
        tensor[i] = if legal { 1.0 } else { 0.0 };
    }
    tensor
}

/// Size of player state encoding
/// Base stats (5) + hand cards (10) + creatures (5*10=50) + supports (2*5=10) = 75
const PLAYER_STATE_SIZE: usize = 75;

/// Get the tensor index offset for player state (useful for debugging)
pub const fn player_state_offset(player_idx: usize) -> usize {
    6 + player_idx * PLAYER_STATE_SIZE
}

/// Get the tensor index offset for creature slot within player state
pub const fn creature_slot_offset(player_idx: usize, slot_idx: usize) -> usize {
    player_state_offset(player_idx) + 15 + slot_idx * CREATURE_SLOT_SIZE
}

/// Get the tensor index offset for support slot within player state
pub const fn support_slot_offset(player_idx: usize, slot_idx: usize) -> usize {
    player_state_offset(player_idx) + 65 + slot_idx * SUPPORT_SLOT_SIZE
}
