/**
 * Deck Builder API - Backend-agnostic wrappers.
 *
 * This module provides the public API for deck builder operations.
 * It delegates to the appropriate backend (Tauri or WASM) based on the platform.
 */

import { getGameBackend, getStorageBackend } from "./backends";
import type {
  BrowsableCard,
  CommanderDto,
  CustomDeck,
  CustomDeckInfo,
  DeckValidation,
  PlaystyleScore,
} from "./types";

// ===========================================================================
// Card Browsing
// ===========================================================================

/**
 * List all cards in the game.
 * @param faction Optional faction filter. If provided, returns cards from that faction plus neutral cards.
 * @returns Array of browsable cards
 */
export async function listAllCards(faction?: string): Promise<BrowsableCard[]> {
  return await getGameBackend().listAllCards(faction);
}

/**
 * List all commanders.
 * @returns Array of commander DTOs
 */
export async function listCommanders(): Promise<CommanderDto[]> {
  return await getGameBackend().listCommanders();
}

// ===========================================================================
// Custom Deck Management
// ===========================================================================

/**
 * List all custom decks (metadata only).
 * @returns Array of custom deck info
 */
export async function listCustomDecks(): Promise<CustomDeckInfo[]> {
  return await getStorageBackend().listCustomDecks();
}

/**
 * Load a custom deck by ID.
 * @param deckId The deck ID to load
 * @returns The full custom deck
 */
export async function loadCustomDeck(deckId: string): Promise<CustomDeck> {
  return await getStorageBackend().loadCustomDeck(deckId);
}

/**
 * Save a custom deck.
 * Creates a new deck or overwrites an existing one with the same ID.
 * @param deck The deck to save
 * @returns The deck ID on success
 */
export async function saveCustomDeck(deck: CustomDeck): Promise<string> {
  return await getStorageBackend().saveCustomDeck(deck);
}

/**
 * Delete a custom deck by ID.
 * @param deckId The deck ID to delete
 */
export async function deleteCustomDeck(deckId: string): Promise<void> {
  return await getStorageBackend().deleteCustomDeck(deckId);
}

// ===========================================================================
// Deck Validation & Analysis
// ===========================================================================

/**
 * Validate a custom deck configuration.
 * @param deck The deck to validate
 * @returns Validation result with errors and warnings
 */
export async function validateCustomDeck(deck: CustomDeck): Promise<DeckValidation> {
  return await getGameBackend().validateCustomDeck(deck);
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
  return await getGameBackend().calculateDeckPlaystyle(cards, commanderId);
}
