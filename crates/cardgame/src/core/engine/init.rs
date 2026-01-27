//! Game initialization functions.
//!
//! Handles setting up a new game with decks, commanders, and initial state.

use thiserror::Error;

use crate::core::cards::CardDatabase;
use crate::core::config::{game, player};
use crate::core::state::{CardInstance, GameMode, GamePhase, GameState};
use crate::core::types::{CardId, PlayerId};
use crate::decks::DeckDefinition;

use super::seeded_shuffle;

/// Error type for game initialization failures.
#[derive(Debug, Clone, Error)]
pub enum GameInitError {
    /// Commander not found in the card database.
    #[error("Commander not found: ID {commander_id} for player {player}")]
    CommanderNotFound {
        /// The commander ID that was not found.
        commander_id: u16,
        /// The player number (1 or 2).
        player: usize,
    },
}

/// Initialize a new game from deck definitions.
///
/// This is the primary initialization function. Sets up:
/// - Commanders for both players
/// - Shuffled decks
/// - Initial hands
/// - Starting essence (with P2 compensation for First Player Advantage)
///
/// # Errors
/// Returns `GameInitError::CommanderNotFound` if either commander is not found in the card database.
pub fn initialize_game(
    state: &mut GameState,
    card_db: &CardDatabase,
    deck1: &DeckDefinition,
    deck2: &DeckDefinition,
    seed: u64,
    mode: GameMode,
) -> Result<(), GameInitError> {
    initialize_game_raw(
        state,
        card_db,
        deck1.to_card_ids(),
        deck2.to_card_ids(),
        deck1.commander_id(),
        deck2.commander_id(),
        seed,
        mode,
    )
}

/// Low-level game initialization with raw components.
///
/// Use this when you have raw card IDs and commander IDs (e.g., in tests
/// or when constructing games programmatically without DeckDefinitions).
///
/// # Errors
/// Returns `GameInitError::CommanderNotFound` if either commander ID is not found in the card database.
#[allow(clippy::too_many_arguments)]
pub fn initialize_game_raw(
    state: &mut GameState,
    card_db: &CardDatabase,
    deck1: Vec<CardId>,
    deck2: Vec<CardId>,
    commander1: CardId,
    commander2: CardId,
    seed: u64,
    mode: GameMode,
) -> Result<(), GameInitError> {
    // Validate commanders exist in database
    if card_db.get_commander(commander1).is_none() {
        return Err(GameInitError::CommanderNotFound {
            commander_id: commander1.0,
            player: 1,
        });
    }
    if card_db.get_commander(commander2).is_none() {
        return Err(GameInitError::CommanderNotFound {
            commander_id: commander2.0,
            player: 2,
        });
    }

    // Reset state
    *state = GameState::new();
    state.rng_state = seed;
    state.game_mode = mode;

    // Set commanders
    state.commander_p1 = Some(commander1);
    state.commander_p2 = Some(commander2);

    // Set up player 1's deck
    setup_player_deck(&mut state.players[0].deck, deck1, seed);

    // Set up player 2's deck (use a different seed derived from the original)
    let seed2 = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    setup_player_deck(&mut state.players[1].deck, deck2, seed2);

    // Draw initial hands
    draw_starting_hands(state);

    // Set up initial essence (P2 starts higher to compensate for FPA)
    // Note: start_turn() will add +1, so we set to (target - 1)
    state.players[0].max_essence = player::STARTING_ESSENCE_P1 - 1;
    state.players[1].max_essence = player::STARTING_ESSENCE_P2 - 1;

    // Set up initial game state
    state.current_turn = 0; // Will be incremented to 1 in start_turn
    state.active_player = PlayerId::PLAYER_ONE;
    state.phase = GamePhase::Main;

    Ok(())
}

/// Set up a player's deck by shuffling card IDs.
fn setup_player_deck(
    deck: &mut arrayvec::ArrayVec<CardInstance, { game::MAX_DECK_SIZE }>,
    cards: Vec<CardId>,
    seed: u64,
) {
    let mut deck_cards: Vec<CardInstance> = cards.into_iter().map(CardInstance::new).collect();
    seeded_shuffle(&mut deck_cards, seed);
    for card in deck_cards {
        if deck.len() < game::MAX_DECK_SIZE {
            deck.push(card);
        }
    }
}

/// Draw initial hands for both players.
fn draw_starting_hands(state: &mut GameState) {
    for _ in 0..player::STARTING_HAND_SIZE {
        draw_card(state, PlayerId::PLAYER_ONE);
        draw_card(state, PlayerId::PLAYER_TWO);
    }

    // P2 draws extra cards to compensate for First Player Advantage
    // Note: Currently P2_BONUS_CARDS is 0, using essence instead for FPA compensation
    #[allow(clippy::reversed_empty_ranges)]
    for _ in 0..player::P2_BONUS_CARDS {
        draw_card(state, PlayerId::PLAYER_TWO);
    }
}

/// Draw a card for the specified player.
/// If deck is empty, nothing happens (no fatigue damage in this game).
/// If hand is full (10 cards), the drawn card is discarded (overdraw).
pub fn draw_card(state: &mut GameState, player: PlayerId) {
    let player_state = &mut state.players[player.index()];

    // Try to draw from deck
    if let Some(card) = player_state.deck.pop() {
        // Add to hand if not full
        if !player_state.is_hand_full() {
            player_state.hand.push(card);
        }
        // If hand is full, card is simply discarded (overdraw)
    }
}
