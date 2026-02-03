/**
 * API wrappers for deck builder Tauri commands.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  BrowsableCard,
  CommanderDto,
  CustomDeck,
  CustomDeckInfo,
  DeckValidation,
  PlaystyleScore,
} from "./types";

/**
 * List all cards in the game.
 * @param faction Optional faction filter. If provided, returns cards from that faction plus neutral cards.
 * @returns Array of browsable cards
 */
export async function listAllCards(faction?: string): Promise<BrowsableCard[]> {
  return await invoke<BrowsableCard[]>("list_all_cards", { faction: faction ?? null });
}

/**
 * List all commanders.
 * @returns Array of commander DTOs
 */
export async function listCommanders(): Promise<CommanderDto[]> {
  return await invoke<CommanderDto[]>("list_commanders");
}

/**
 * List all custom decks (metadata only).
 * @returns Array of custom deck info
 */
export async function listCustomDecks(): Promise<CustomDeckInfo[]> {
  return await invoke<CustomDeckInfo[]>("list_custom_decks");
}

/**
 * Load a custom deck by ID.
 * @param deckId The deck ID to load
 * @returns The full custom deck
 */
export async function loadCustomDeck(deckId: string): Promise<CustomDeck> {
  return await invoke<CustomDeck>("load_custom_deck", { deckId });
}

/**
 * Save a custom deck.
 * Creates a new deck or overwrites an existing one with the same ID.
 * @param deck The deck to save
 * @returns The deck ID on success
 */
export async function saveCustomDeck(deck: CustomDeck): Promise<string> {
  return await invoke<string>("save_custom_deck", { deck });
}

/**
 * Delete a custom deck by ID.
 * @param deckId The deck ID to delete
 */
export async function deleteCustomDeck(deckId: string): Promise<void> {
  return await invoke<void>("delete_custom_deck", { deckId });
}

/**
 * Validate a custom deck configuration.
 * @param deck The deck to validate
 * @returns Validation result with errors and warnings
 */
export async function validateCustomDeck(deck: CustomDeck): Promise<DeckValidation> {
  return await invoke<DeckValidation>("validate_custom_deck", { deck });
}

/**
 * Calculate the playstyle for a deck based on its cards and commander.
 * @param cards The card IDs in the deck
 * @param commanderId The commander card ID
 * @returns The detected playstyle with scores
 */
export async function calculateDeckPlaystyle(
  cards: number[],
  commanderId: number
): Promise<PlaystyleScore> {
  return await invoke<PlaystyleScore>("calculate_deck_playstyle", {
    cards,
    commanderId,
  });
}
