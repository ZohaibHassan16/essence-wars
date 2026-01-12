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

            // [9]: Full keyword bitfield normalized (0-255 -> 0.0-1.0)
            tensor[*idx] = c.keywords.0 as f32 / 255.0;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{CardInstance, Creature, CreatureStatus, Support};
    use crate::types::{CardId, CreatureInstanceId, PlayerId};

    #[test]
    fn test_empty_state_tensor_size() {
        let state = GameState::new();
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor.len(), STATE_TENSOR_SIZE);
    }

    #[test]
    fn test_turn_number_normalization() {
        let mut state = GameState::new();

        // Turn 0
        state.current_turn = 0;
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor[0], 0.0);

        // Turn 15 (midpoint)
        state.current_turn = 15;
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor[0], 0.5);

        // Turn 30 (normalized to 1.0)
        state.current_turn = 30;
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor[0], 1.0);

        // Turn 60 (beyond normalization range)
        state.current_turn = 60;
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor[0], 2.0);
    }

    #[test]
    fn test_current_player_encoding() {
        let mut state = GameState::new();

        state.active_player = PlayerId::PLAYER_ONE;
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor[1], 0.0);

        state.active_player = PlayerId::PLAYER_TWO;
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor[1], 1.0);
    }

    #[test]
    fn test_game_over_encoding() {
        let mut state = GameState::new();

        // Game not over
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor[2], 0.0); // game_over
        assert_eq!(tensor[3], -1.0); // winner (no winner)

        // Player 0 wins
        state.result = Some(GameResult::Win {
            winner: PlayerId::PLAYER_ONE,
            reason: crate::state::WinReason::LifeReachedZero,
        });
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor[2], 1.0); // game_over
        assert_eq!(tensor[3], 0.0); // winner = player 0

        // Player 1 wins
        state.result = Some(GameResult::Win {
            winner: PlayerId::PLAYER_TWO,
            reason: crate::state::WinReason::LifeReachedZero,
        });
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor[2], 1.0); // game_over
        assert_eq!(tensor[3], 1.0); // winner = player 1

        // Draw
        state.result = Some(GameResult::Draw);
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor[2], 1.0); // game_over
        assert_eq!(tensor[3], -1.0); // no winner
    }

    #[test]
    fn test_player_life_normalization() {
        let mut state = GameState::new();

        // Default life (30)
        let tensor = state_to_tensor(&state);
        let player1_offset = player_state_offset(0);
        assert_eq!(tensor[player1_offset], 1.0);

        // Half life (15)
        state.players[0].life = 15;
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor[player1_offset], 0.5);

        // Zero life
        state.players[0].life = 0;
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor[player1_offset], 0.0);

        // Negative life (clamped to 0)
        state.players[0].life = -10;
        let tensor = state_to_tensor(&state);
        assert_eq!(tensor[player1_offset], 0.0);
    }

    #[test]
    fn test_creature_encoding_empty_slot() {
        let state = GameState::new();
        let tensor = state_to_tensor(&state);

        // Check first creature slot for player 0
        let creature_offset = creature_slot_offset(0, 0);

        // All 10 floats should be 0 for empty slot
        for i in 0..CREATURE_SLOT_SIZE {
            assert_eq!(tensor[creature_offset + i], 0.0, "Slot float {} should be 0", i);
        }
    }

    #[test]
    fn test_creature_encoding_occupied_slot() {
        let mut state = GameState::new();
        state.current_turn = 2;

        // Add a creature to slot 0
        let creature = Creature {
            instance_id: CreatureInstanceId(0),
            card_id: CardId(42),
            owner: PlayerId::PLAYER_ONE,
            slot: Slot(0),
            attack: 5,
            current_health: 3,
            max_health: 4,
            base_attack: 5,
            base_health: 4,
            keywords: Keywords::none().with_rush().with_guard(),
            status: CreatureStatus::default(),
            turn_played: 1,
        };
        state.players[0].creatures.push(creature);

        let tensor = state_to_tensor(&state);
        let creature_offset = creature_slot_offset(0, 0);

        // [0]: occupied = 1.0
        assert_eq!(tensor[creature_offset], 1.0);
        // [1]: attack / 10.0 = 0.5
        assert_eq!(tensor[creature_offset + 1], 0.5);
        // [2]: health / 10.0 = 0.3
        assert_eq!(tensor[creature_offset + 2], 0.3);
        // [3]: max_health / 10.0 = 0.4
        assert_eq!(tensor[creature_offset + 3], 0.4);
        // [4]: can_attack = 1.0 (Rush + turn > turn_played)
        assert_eq!(tensor[creature_offset + 4], 1.0);
        // [5]: attacked_this_turn = 0.0
        assert_eq!(tensor[creature_offset + 5], 0.0);
        // [6]: silenced = 0.0
        assert_eq!(tensor[creature_offset + 6], 0.0);
        // [7]: Rush = 1.0
        assert_eq!(tensor[creature_offset + 7], 1.0);
        // [8]: Guard = 1.0
        assert_eq!(tensor[creature_offset + 8], 1.0);
        // [9]: keywords bitfield normalized
        let expected_kw = (Keywords::RUSH | Keywords::GUARD) as f32 / 255.0;
        assert!((tensor[creature_offset + 9] - expected_kw).abs() < 0.001);
    }

    #[test]
    fn test_keywords_encoded_correctly() {
        let mut state = GameState::new();
        state.current_turn = 1;

        // Creature with Rush keyword
        let creature = Creature {
            instance_id: CreatureInstanceId(0),
            card_id: CardId(1),
            owner: PlayerId::PLAYER_ONE,
            slot: Slot(0),
            attack: 2,
            current_health: 2,
            max_health: 2,
            base_attack: 2,
            base_health: 2,
            keywords: Keywords::none().with_rush(),
            status: CreatureStatus::default(),
            turn_played: 1,
        };
        state.players[0].creatures.push(creature);

        let tensor = state_to_tensor(&state);
        let creature_offset = creature_slot_offset(0, 0);

        // Rush should be 1.0
        assert_eq!(tensor[creature_offset + 7], 1.0);
        // Guard should be 0.0
        assert_eq!(tensor[creature_offset + 8], 0.0);
    }

    #[test]
    fn test_support_encoding_empty() {
        let state = GameState::new();
        let tensor = state_to_tensor(&state);

        // Check first support slot for player 0
        let support_offset = support_slot_offset(0, 0);

        // All 5 floats should be 0 for empty slot
        for i in 0..SUPPORT_SLOT_SIZE {
            assert_eq!(tensor[support_offset + i], 0.0, "Support slot float {} should be 0", i);
        }
    }

    #[test]
    fn test_support_encoding_occupied() {
        let mut state = GameState::new();

        // Add a support to slot 0
        let support = Support {
            card_id: CardId(100),
            owner: PlayerId::PLAYER_ONE,
            slot: Slot(0),
            current_durability: 3,
        };
        state.players[0].supports.push(support);

        let tensor = state_to_tensor(&state);
        let support_offset = support_slot_offset(0, 0);

        // [0]: occupied = 1.0
        assert_eq!(tensor[support_offset], 1.0);
        // [1]: durability / 5.0 = 0.6
        assert_eq!(tensor[support_offset + 1], 0.6);
        // [2]: card_id = 100.0
        assert_eq!(tensor[support_offset + 2], 100.0);
    }

    #[test]
    fn test_hand_cards_encoding() {
        let mut state = GameState::new();

        // Add cards to hand
        state.players[0].hand.push(CardInstance::new(CardId(1)));
        state.players[0].hand.push(CardInstance::new(CardId(5)));
        state.players[0].hand.push(CardInstance::new(CardId(10)));

        let tensor = state_to_tensor(&state);
        let player1_offset = player_state_offset(0);

        // Hand cards start at offset 5 within player state
        assert_eq!(tensor[player1_offset + 5], 1.0); // Card ID 1
        assert_eq!(tensor[player1_offset + 6], 5.0); // Card ID 5
        assert_eq!(tensor[player1_offset + 7], 10.0); // Card ID 10
        assert_eq!(tensor[player1_offset + 8], 0.0); // Empty slot
    }

    #[test]
    fn test_legal_mask_conversion() {
        let mut mask = [false; 256];
        mask[0] = true;
        mask[5] = true;
        mask[255] = true;

        let tensor = legal_mask_to_tensor(&mask);

        assert_eq!(tensor[0], 1.0);
        assert_eq!(tensor[1], 0.0);
        assert_eq!(tensor[5], 1.0);
        assert_eq!(tensor[254], 0.0);
        assert_eq!(tensor[255], 1.0);
    }

    #[test]
    fn test_tensor_values_in_expected_ranges() {
        let mut state = GameState::new();
        state.current_turn = 15;
        state.players[0].life = 15;
        state.players[0].current_essence = 5;
        state.players[0].action_points = 2;

        // Add some hand cards
        state.players[0].hand.push(CardInstance::new(CardId(1)));
        state.players[0].hand.push(CardInstance::new(CardId(2)));

        // Add a creature
        let creature = Creature {
            instance_id: CreatureInstanceId(0),
            card_id: CardId(10),
            owner: PlayerId::PLAYER_ONE,
            slot: Slot(2),
            attack: 3,
            current_health: 4,
            max_health: 4,
            base_attack: 3,
            base_health: 4,
            keywords: Keywords::none(),
            status: CreatureStatus::default(),
            turn_played: 10,
        };
        state.players[0].creatures.push(creature);

        let tensor = state_to_tensor(&state);

        // Global state values should be in reasonable ranges
        assert!(tensor[0] >= 0.0); // turn number
        assert!(tensor[1] >= 0.0 && tensor[1] <= 1.0); // current player
        assert!(tensor[2] >= 0.0 && tensor[2] <= 1.0); // game over

        // Player state values should be normalized (mostly 0-1)
        let p1_offset = player_state_offset(0);
        assert!(tensor[p1_offset] >= 0.0 && tensor[p1_offset] <= 1.0); // life
        assert!(tensor[p1_offset + 1] >= 0.0 && tensor[p1_offset + 1] <= 1.0); // essence
        assert!(tensor[p1_offset + 2] >= 0.0); // AP
        assert!(tensor[p1_offset + 3] >= 0.0 && tensor[p1_offset + 3] <= 1.0); // deck size
        assert!(tensor[p1_offset + 4] >= 0.0 && tensor[p1_offset + 4] <= 1.0); // hand size
    }

    #[test]
    fn test_offset_calculations() {
        // Verify offset calculations are consistent with tensor layout
        // Player state size: 5 base + 10 hand + 50 creatures + 10 supports = 75 floats
        assert_eq!(player_state_offset(0), 6);
        assert_eq!(player_state_offset(1), 81); // 6 + 75

        // Creature slots start at player_offset + 15
        assert_eq!(creature_slot_offset(0, 0), 21);
        assert_eq!(creature_slot_offset(0, 1), 31);
        assert_eq!(creature_slot_offset(0, 4), 61);

        // Support slots start at player_offset + 65
        assert_eq!(support_slot_offset(0, 0), 71);
        assert_eq!(support_slot_offset(0, 1), 76);

        // Player 2 offsets
        assert_eq!(creature_slot_offset(1, 0), 96);  // 81 + 15
        assert_eq!(support_slot_offset(1, 0), 146);  // 81 + 65
    }

    #[test]
    fn test_both_players_encoded() {
        let mut state = GameState::new();

        // Set different values for each player
        state.players[0].life = 20;
        state.players[1].life = 25;

        let tensor = state_to_tensor(&state);

        let p1_life = tensor[player_state_offset(0)];
        let p2_life = tensor[player_state_offset(1)];

        // Player 1: 20/30 = 0.666...
        assert!((p1_life - 20.0 / 30.0).abs() < 0.001);
        // Player 2: 25/30 = 0.833...
        assert!((p2_life - 25.0 / 30.0).abs() < 0.001);
    }

    #[test]
    fn test_exhausted_creature_encoding() {
        let mut state = GameState::new();
        state.current_turn = 2;

        let mut creature = Creature {
            instance_id: CreatureInstanceId(0),
            card_id: CardId(1),
            owner: PlayerId::PLAYER_ONE,
            slot: Slot(0),
            attack: 2,
            current_health: 2,
            max_health: 2,
            base_attack: 2,
            base_health: 2,
            keywords: Keywords::none(),
            status: CreatureStatus::default(),
            turn_played: 1,
        };
        creature.status.set_exhausted(true);
        state.players[0].creatures.push(creature);

        let tensor = state_to_tensor(&state);
        let creature_offset = creature_slot_offset(0, 0);

        // can_attack should be 0.0 (exhausted)
        assert_eq!(tensor[creature_offset + 4], 0.0);
        // attacked_this_turn (exhausted flag) should be 1.0
        assert_eq!(tensor[creature_offset + 5], 1.0);
    }

    #[test]
    fn test_silenced_creature_encoding() {
        let mut state = GameState::new();
        state.current_turn = 2;

        let mut creature = Creature {
            instance_id: CreatureInstanceId(0),
            card_id: CardId(1),
            owner: PlayerId::PLAYER_ONE,
            slot: Slot(0),
            attack: 2,
            current_health: 2,
            max_health: 2,
            base_attack: 2,
            base_health: 2,
            keywords: Keywords::none().with_rush(),
            status: CreatureStatus::default(),
            turn_played: 1,
        };
        creature.status.set_silenced(true);
        state.players[0].creatures.push(creature);

        let tensor = state_to_tensor(&state);
        let creature_offset = creature_slot_offset(0, 0);

        // silenced should be 1.0
        assert_eq!(tensor[creature_offset + 6], 1.0);
    }
}
