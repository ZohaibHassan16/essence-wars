//! Deck builder Tauri commands.

use crate::state::{
    BrowsableCard, CommanderDto, CustomDeck, CustomDeckInfo, CustomDeckManager,
    DeckValidation, PlaystyleScore,
};
use tauri::State;

/// List all cards in the game (optionally filtered by faction).
///
/// If `faction` is provided, returns cards from that faction plus neutral cards.
/// If `faction` is None, returns all cards.
#[tauri::command]
pub fn list_all_cards(
    faction: Option<String>,
    deck_manager: State<'_, CustomDeckManager>,
) -> Vec<BrowsableCard> {
    match faction {
        Some(f) => deck_manager.list_cards_for_faction(&f),
        None => deck_manager.list_all_cards(),
    }
}

/// List all commanders.
#[tauri::command]
pub fn list_commanders(
    deck_manager: State<'_, CustomDeckManager>,
) -> Vec<CommanderDto> {
    deck_manager.list_commanders()
}

/// List all custom decks (metadata only).
#[tauri::command]
pub fn list_custom_decks(
    deck_manager: State<'_, CustomDeckManager>,
) -> Result<Vec<CustomDeckInfo>, String> {
    deck_manager.list_decks()
}

/// Load a custom deck by ID.
#[tauri::command]
pub fn load_custom_deck(
    deck_id: String,
    deck_manager: State<'_, CustomDeckManager>,
) -> Result<CustomDeck, String> {
    deck_manager.load_deck(&deck_id)
}

/// Save a custom deck.
///
/// Creates a new deck or overwrites an existing one with the same ID.
/// Returns the deck ID on success.
#[tauri::command]
pub fn save_custom_deck(
    deck: CustomDeck,
    deck_manager: State<'_, CustomDeckManager>,
) -> Result<String, String> {
    deck_manager.save_deck(&deck)
}

/// Delete a custom deck by ID.
#[tauri::command]
pub fn delete_custom_deck(
    deck_id: String,
    deck_manager: State<'_, CustomDeckManager>,
) -> Result<(), String> {
    deck_manager.delete_deck(&deck_id)
}

/// Validate a custom deck configuration.
///
/// Returns validation result with errors and warnings.
#[tauri::command]
pub fn validate_custom_deck(
    deck: CustomDeck,
    deck_manager: State<'_, CustomDeckManager>,
) -> DeckValidation {
    deck_manager.validate_deck(&deck)
}

/// Calculate the playstyle for a deck based on its cards and commander.
///
/// Returns the detected playstyle (Aggro, Control, Tempo, Midrange) with scores.
#[tauri::command]
pub fn calculate_deck_playstyle(
    cards: Vec<u16>,
    commander_id: u16,
    deck_manager: State<'_, CustomDeckManager>,
) -> PlaystyleScore {
    deck_manager.calculate_playstyle(&cards, commander_id)
}
