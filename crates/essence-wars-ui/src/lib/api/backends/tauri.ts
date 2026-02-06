/**
 * Tauri backend implementation.
 * Uses Tauri IPC to communicate with the native Rust backend.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  GameBackend,
  StorageBackend,
  McpBackend,
  Platform,
} from "./interface";
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
  CardDto,
  BrowsableCard,
  CommanderDto,
  CustomDeck,
  CustomDeckInfo,
  DeckValidation,
  PlaystyleScore,
  ReplayInfo,
  McpSyncedState,
} from "../types";

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

/**
 * Tauri implementation of the game backend.
 */
export class TauriGameBackend implements GameBackend {
  readonly platform: Platform = "tauri";

  async init(): Promise<void> {
    // Tauri is ready immediately when running in Tauri context
  }

  // ===========================================================================
  // Game Setup
  // ===========================================================================

  async listDecks(): Promise<DeckInfo[]> {
    return await invoke<DeckInfo[]>("list_decks");
  }

  async listBots(): Promise<BotInfo[]> {
    return await invoke<BotInfo[]>("list_bots");
  }

  async getDeckCards(deckId: string): Promise<CardDto[]> {
    return await invoke<CardDto[]>("get_deck_cards", { deckId });
  }

  // ===========================================================================
  // Game Session
  // ===========================================================================

  async newGame(config: GameConfig): Promise<GameStateDto> {
    return await invoke<GameStateDto>("new_game", { config });
  }

  async getGameState(gameId: string): Promise<GameStateDto> {
    return await invoke<GameStateDto>("get_game_state", { gameId });
  }

  async getLegalActions(gameId: string): Promise<ActionInfo[]> {
    return await invoke<ActionInfo[]>("get_legal_actions", { gameId });
  }

  async applyAction(
    gameId: string,
    actionIndex: number
  ): Promise<GameStateUpdate> {
    return await invoke<GameStateUpdate>("apply_action", {
      gameId,
      actionIndex,
    });
  }

  async getAiMove(gameId: string): Promise<ActionInfo> {
    return await withTimeout(
      invoke<ActionInfo>("get_ai_move", { gameId }),
      30000,
      "getAiMove"
    );
  }

  async getAiHint(gameId: string): Promise<AiHintResponse> {
    return await withTimeout(
      invoke<AiHintResponse>("get_ai_hint", { gameId }),
      30000,
      "getAiHint"
    );
  }

  async endGame(gameId: string): Promise<GameResultDto> {
    return await invoke<GameResultDto>("end_game", { gameId });
  }

  async undoAction(gameId: string): Promise<GameStateDto> {
    return await invoke<GameStateDto>("undo_action", { gameId });
  }

  async canUndo(gameId: string): Promise<boolean> {
    return await invoke<boolean>("can_undo", { gameId });
  }

  // ===========================================================================
  // Spectator Mode
  // ===========================================================================

  async computeSpectatorMatch(config: SpectatorConfig): Promise<SpectatorMatch> {
    return await withTimeout(
      invoke<SpectatorMatch>("compute_spectator_match", { config }),
      300000, // 5 minutes
      "computeSpectatorMatch"
    );
  }

  // ===========================================================================
  // Deck Builder
  // ===========================================================================

  async listAllCards(faction?: string): Promise<BrowsableCard[]> {
    return await invoke<BrowsableCard[]>("list_all_cards", {
      faction: faction ?? null,
    });
  }

  async listCommanders(): Promise<CommanderDto[]> {
    return await invoke<CommanderDto[]>("list_commanders");
  }

  async validateCustomDeck(deck: CustomDeck): Promise<DeckValidation> {
    return await invoke<DeckValidation>("validate_custom_deck", { deck });
  }

  async calculateDeckPlaystyle(
    cards: number[],
    commanderId: number
  ): Promise<PlaystyleScore> {
    return await invoke<PlaystyleScore>("calculate_deck_playstyle", {
      cards,
      commanderId,
    });
  }
}

/**
 * Tauri implementation of the storage backend.
 */
export class TauriStorageBackend implements StorageBackend {
  readonly platform: Platform = "tauri";

  async init(): Promise<void> {
    // Filesystem is ready immediately
  }

  // ===========================================================================
  // Replays
  // ===========================================================================

  async saveReplay(gameId: string, name?: string): Promise<string> {
    return await invoke<string>("save_replay", { gameId, name });
  }

  async saveSpectatorReplay(
    spectatorMatch: SpectatorMatch,
    name?: string
  ): Promise<string> {
    return await invoke<string>("save_spectator_replay", {
      spectatorMatch,
      name,
    });
  }

  async listReplays(): Promise<ReplayInfo[]> {
    return await invoke<ReplayInfo[]>("list_replays");
  }

  async loadReplay(path: string): Promise<SpectatorMatch> {
    return await invoke<SpectatorMatch>("load_replay", { path });
  }

  async deleteReplay(path: string): Promise<void> {
    return await invoke<void>("delete_replay", { path });
  }

  // ===========================================================================
  // Custom Decks
  // ===========================================================================

  async saveCustomDeck(deck: CustomDeck): Promise<string> {
    return await invoke<string>("save_custom_deck", { deck });
  }

  async listCustomDecks(): Promise<CustomDeckInfo[]> {
    return await invoke<CustomDeckInfo[]>("list_custom_decks");
  }

  async loadCustomDeck(deckId: string): Promise<CustomDeck> {
    return await invoke<CustomDeck>("load_custom_deck", { deckId });
  }

  async deleteCustomDeck(deckId: string): Promise<void> {
    return await invoke<void>("delete_custom_deck", { deckId });
  }
}

/**
 * Tauri implementation of MCP backend.
 */
export class TauriMcpBackend implements McpBackend {
  readonly supported = true;

  async getMcpSyncedState(): Promise<McpSyncedState | null> {
    return await invoke<McpSyncedState | null>("get_mcp_synced_state");
  }

  async hasMcpSyncedState(): Promise<boolean> {
    return await invoke<boolean>("has_mcp_synced_state");
  }

  async clearMcpSyncedState(): Promise<void> {
    return await invoke<void>("clear_mcp_synced_state");
  }
}
