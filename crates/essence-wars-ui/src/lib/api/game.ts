/**
 * Game API - Backend-agnostic wrappers.
 *
 * This module provides the public API for game operations.
 * It delegates to the appropriate backend (Tauri or WASM) based on the platform.
 */

import {
  getGameBackend,
  getStorageBackend,
  getMcpBackend,
} from "./backends";
import type {
  DeckInfo,
  BotInfo,
  GameConfig,
  GameStateDto,
  ActionInfo,
  GameStateUpdate,
  GameResultDto,
  AiHintResponse,
  SpectatorConfig,
  SpectatorMatch,
  ReplayInfo,
  McpSyncedState,
  CardDto,
} from "./types";

// Re-export the IpcTimeoutError for backwards compatibility
// Note: This class is defined in the tauri backend but is a generic error type
export class IpcTimeoutError extends Error {
  constructor(operation: string, timeoutMs: number) {
    super(`Operation "${operation}" timed out after ${timeoutMs}ms`);
    this.name = "IpcTimeoutError";
  }
}

// ===========================================================================
// Game Setup
// ===========================================================================

export async function listDecks(): Promise<DeckInfo[]> {
  return await getGameBackend().listDecks();
}

export async function listBots(): Promise<BotInfo[]> {
  return await getGameBackend().listBots();
}

/**
 * Get all cards in a deck (for deck library visualization)
 *
 * Supports both built-in deck IDs and custom deck IDs (with "custom:" prefix).
 * Returns the cards in deck order (index 0 = first card to draw).
 */
export async function getDeckCards(deckId: string): Promise<CardDto[]> {
  return await getGameBackend().getDeckCards(deckId);
}

// ===========================================================================
// Game Session
// ===========================================================================

export async function newGame(config: GameConfig): Promise<GameStateDto> {
  return await getGameBackend().newGame(config);
}

export async function getGameState(gameId: string): Promise<GameStateDto> {
  return await getGameBackend().getGameState(gameId);
}

export async function getLegalActions(gameId: string): Promise<ActionInfo[]> {
  return await getGameBackend().getLegalActions(gameId);
}

export async function applyAction(
  gameId: string,
  actionIndex: number
): Promise<GameStateUpdate> {
  return await getGameBackend().applyAction(gameId, actionIndex);
}

export async function getAiMove(gameId: string): Promise<ActionInfo> {
  return await getGameBackend().getAiMove(gameId);
}

export async function endGame(gameId: string): Promise<GameResultDto> {
  return await getGameBackend().endGame(gameId);
}

export async function getAiHint(gameId: string): Promise<AiHintResponse> {
  return await getGameBackend().getAiHint(gameId);
}

export async function undoAction(gameId: string): Promise<GameStateDto> {
  return await getGameBackend().undoAction(gameId);
}

export async function canUndo(gameId: string): Promise<boolean> {
  return await getGameBackend().canUndo(gameId);
}

// ===========================================================================
// Spectator Mode API
// ===========================================================================

/** Compute a complete AI vs AI match for spectator playback */
export async function computeSpectatorMatch(
  config: SpectatorConfig
): Promise<SpectatorMatch> {
  return await getGameBackend().computeSpectatorMatch(config);
}

// ===========================================================================
// Replay Mode API
// ===========================================================================

/** Save a replay from a completed game session */
export async function saveReplay(
  gameId: string,
  name?: string
): Promise<string> {
  return await getStorageBackend().saveReplay(gameId, name);
}

/** Save a spectator match (AI vs AI) as a replay */
export async function saveSpectatorReplay(
  spectatorMatch: SpectatorMatch,
  name?: string
): Promise<string> {
  return await getStorageBackend().saveSpectatorReplay(spectatorMatch, name);
}

/** List all saved replays */
export async function listReplays(): Promise<ReplayInfo[]> {
  return await getStorageBackend().listReplays();
}

/** Load a replay file for playback */
export async function loadReplay(path: string): Promise<SpectatorMatch> {
  return await getStorageBackend().loadReplay(path);
}

/** Delete a replay file */
export async function deleteReplay(path: string): Promise<void> {
  return await getStorageBackend().deleteReplay(path);
}

// ===========================================================================
// MCP Sync Mode API
// ===========================================================================

/** Get the current MCP-synced game state, if any */
export async function getMcpSyncedState(): Promise<McpSyncedState | null> {
  const mcpBackend = getMcpBackend();
  if (!mcpBackend.supported) {
    return null;
  }
  return await mcpBackend.getMcpSyncedState();
}

/** Check if there is an MCP-synced game state available */
export async function hasMcpSyncedState(): Promise<boolean> {
  const mcpBackend = getMcpBackend();
  if (!mcpBackend.supported) {
    return false;
  }
  return await mcpBackend.hasMcpSyncedState();
}

/** Clear the MCP-synced game state */
export async function clearMcpSyncedState(): Promise<void> {
  const mcpBackend = getMcpBackend();
  if (!mcpBackend.supported) {
    return;
  }
  return await mcpBackend.clearMcpSyncedState();
}
