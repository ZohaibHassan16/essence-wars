// Game API - Tauri command wrappers

import { invoke } from "@tauri-apps/api/core";
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

/** Error thrown when an IPC call times out */
export class IpcTimeoutError extends Error {
  constructor(operation: string, timeoutMs: number) {
    super(`Operation "${operation}" timed out after ${timeoutMs}ms`);
    this.name = "IpcTimeoutError";
  }
}

/** Wrap a promise with a timeout */
async function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  operation: string
): Promise<T> {
  let timeoutId: ReturnType<typeof setTimeout> | undefined;

  const timeoutPromise = new Promise<never>((_, reject) => {
    timeoutId = setTimeout(() => {
      reject(new IpcTimeoutError(operation, timeoutMs));
    }, timeoutMs);
  });

  try {
    return await Promise.race([promise, timeoutPromise]);
  } finally {
    if (timeoutId !== undefined) {
      clearTimeout(timeoutId);
    }
  }
}

export async function listDecks(): Promise<DeckInfo[]> {
  return await invoke<DeckInfo[]>("list_decks");
}

export async function listBots(): Promise<BotInfo[]> {
  return await invoke<BotInfo[]>("list_bots");
}

export async function newGame(config: GameConfig): Promise<GameStateDto> {
  return await invoke<GameStateDto>("new_game", { config });
}

export async function getGameState(gameId: string): Promise<GameStateDto> {
  return await invoke<GameStateDto>("get_game_state", { gameId });
}

export async function getLegalActions(gameId: string): Promise<ActionInfo[]> {
  return await invoke<ActionInfo[]>("get_legal_actions", { gameId });
}

export async function applyAction(
  gameId: string,
  actionIndex: number
): Promise<GameStateUpdate> {
  return await invoke<GameStateUpdate>("apply_action", { gameId, actionIndex });
}

export async function getAiMove(gameId: string): Promise<ActionInfo> {
  return await withTimeout(
    invoke<ActionInfo>("get_ai_move", { gameId }),
    30000,
    "getAiMove"
  );
}

export async function endGame(gameId: string): Promise<GameResultDto> {
  return await invoke<GameResultDto>("end_game", { gameId });
}

export async function getAiHint(gameId: string): Promise<AiHintResponse> {
  return await withTimeout(
    invoke<AiHintResponse>("get_ai_hint", { gameId }),
    30000,
    "getAiHint"
  );
}

export async function undoAction(gameId: string): Promise<GameStateDto> {
  return await invoke<GameStateDto>("undo_action", { gameId });
}

export async function canUndo(gameId: string): Promise<boolean> {
  return await invoke<boolean>("can_undo", { gameId });
}

/**
 * Get all cards in a deck (for deck library visualization)
 *
 * Supports both built-in deck IDs and custom deck IDs (with "custom:" prefix).
 * Returns the cards in deck order (index 0 = first card to draw).
 */
export async function getDeckCards(deckId: string): Promise<CardDto[]> {
  return await invoke<CardDto[]>("get_deck_cards", { deckId });
}

// ============================================================================
// Spectator Mode API
// ============================================================================

/** Compute a complete AI vs AI match for spectator playback */
export async function computeSpectatorMatch(
  config: SpectatorConfig
): Promise<SpectatorMatch> {
  return await withTimeout(
    invoke<SpectatorMatch>("compute_spectator_match", { config }),
    300000, // 5 minutes
    "computeSpectatorMatch"
  );
}

// ============================================================================
// Replay Mode API
// ============================================================================

/** Save a replay from a completed game session */
export async function saveReplay(
  gameId: string,
  name?: string
): Promise<string> {
  return await invoke<string>("save_replay", { gameId, name });
}

/** Save a spectator match (AI vs AI) as a replay */
export async function saveSpectatorReplay(
  spectatorMatch: SpectatorMatch,
  name?: string
): Promise<string> {
  return await invoke<string>("save_spectator_replay", { spectatorMatch, name });
}

/** List all saved replays */
export async function listReplays(): Promise<ReplayInfo[]> {
  return await invoke<ReplayInfo[]>("list_replays");
}

/** Load a replay file for playback */
export async function loadReplay(path: string): Promise<SpectatorMatch> {
  return await invoke<SpectatorMatch>("load_replay", { path });
}

/** Delete a replay file */
export async function deleteReplay(path: string): Promise<void> {
  return await invoke<void>("delete_replay", { path });
}

// ============================================================================
// MCP Sync Mode API
// ============================================================================

/** Get the current MCP-synced game state, if any */
export async function getMcpSyncedState(): Promise<McpSyncedState | null> {
  return await invoke<McpSyncedState | null>("get_mcp_synced_state");
}

/** Check if there is an MCP-synced game state available */
export async function hasMcpSyncedState(): Promise<boolean> {
  return await invoke<boolean>("has_mcp_synced_state");
}

/** Clear the MCP-synced game state */
export async function clearMcpSyncedState(): Promise<void> {
  return await invoke<void>("clear_mcp_synced_state");
}
