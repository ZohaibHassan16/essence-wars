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
} from "./types";

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
  return await invoke<ActionInfo>("get_ai_move", { gameId });
}

export async function endGame(gameId: string): Promise<GameResultDto> {
  return await invoke<GameResultDto>("end_game", { gameId });
}

export async function getAiHint(gameId: string): Promise<AiHintResponse> {
  return await invoke<AiHintResponse>("get_ai_hint", { gameId });
}

export async function undoAction(gameId: string): Promise<GameStateDto> {
  return await invoke<GameStateDto>("undo_action", { gameId });
}

export async function canUndo(gameId: string): Promise<boolean> {
  return await invoke<boolean>("can_undo", { gameId });
}

// ============================================================================
// Spectator Mode API
// ============================================================================

/** Compute a complete AI vs AI match for spectator playback */
export async function computeSpectatorMatch(
  config: SpectatorConfig
): Promise<SpectatorMatch> {
  return await invoke<SpectatorMatch>("compute_spectator_match", { config });
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
