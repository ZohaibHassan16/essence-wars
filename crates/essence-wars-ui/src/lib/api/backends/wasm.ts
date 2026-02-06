/**
 * WASM backend implementation (stub).
 *
 * This will be implemented in Phase 3-4 to provide browser-based gameplay.
 * For now, all methods throw "not implemented" errors.
 */

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
} from "../types";

class NotImplementedError extends Error {
  constructor(method: string) {
    super(
      `WasmBackend.${method}() is not yet implemented. ` +
        `WASM support will be added in Phase 3-4.`
    );
    this.name = "NotImplementedError";
  }
}

/**
 * WASM implementation of the game backend (stub).
 */
export class WasmGameBackend implements GameBackend {
  readonly platform: Platform = "web";

  async init(): Promise<void> {
    // TODO: Phase 3 - Initialize WASM module
    // await init();
    // this.manager = new WasmGameManager();
    console.warn(
      "WasmGameBackend: WASM support not yet implemented. " +
        "This stub will be replaced in Phase 3-4."
    );
  }

  // ===========================================================================
  // Game Setup
  // ===========================================================================

  async listDecks(): Promise<DeckInfo[]> {
    throw new NotImplementedError("listDecks");
  }

  async listBots(): Promise<BotInfo[]> {
    throw new NotImplementedError("listBots");
  }

  async getDeckCards(deckId: string): Promise<CardDto[]> {
    void deckId;
    throw new NotImplementedError("getDeckCards");
  }

  // ===========================================================================
  // Game Session
  // ===========================================================================

  async newGame(config: GameConfig): Promise<GameStateDto> {
    void config;
    throw new NotImplementedError("newGame");
  }

  async getGameState(gameId: string): Promise<GameStateDto> {
    void gameId;
    throw new NotImplementedError("getGameState");
  }

  async getLegalActions(gameId: string): Promise<ActionInfo[]> {
    void gameId;
    throw new NotImplementedError("getLegalActions");
  }

  async applyAction(
    gameId: string,
    actionIndex: number
  ): Promise<GameStateUpdate> {
    void gameId;
    void actionIndex;
    throw new NotImplementedError("applyAction");
  }

  async getAiMove(gameId: string): Promise<ActionInfo> {
    void gameId;
    throw new NotImplementedError("getAiMove");
  }

  async getAiHint(gameId: string): Promise<AiHintResponse> {
    void gameId;
    throw new NotImplementedError("getAiHint");
  }

  async endGame(gameId: string): Promise<GameResultDto> {
    void gameId;
    throw new NotImplementedError("endGame");
  }

  async undoAction(gameId: string): Promise<GameStateDto> {
    void gameId;
    throw new NotImplementedError("undoAction");
  }

  async canUndo(gameId: string): Promise<boolean> {
    void gameId;
    throw new NotImplementedError("canUndo");
  }

  // ===========================================================================
  // Spectator Mode
  // ===========================================================================

  async computeSpectatorMatch(
    config: SpectatorConfig
  ): Promise<SpectatorMatch> {
    void config;
    throw new NotImplementedError("computeSpectatorMatch");
  }

  // ===========================================================================
  // Deck Builder
  // ===========================================================================

  async listAllCards(faction?: string): Promise<BrowsableCard[]> {
    void faction;
    throw new NotImplementedError("listAllCards");
  }

  async listCommanders(): Promise<CommanderDto[]> {
    throw new NotImplementedError("listCommanders");
  }

  async validateCustomDeck(deck: CustomDeck): Promise<DeckValidation> {
    void deck;
    throw new NotImplementedError("validateCustomDeck");
  }

  async calculateDeckPlaystyle(
    cards: number[],
    commanderId: number
  ): Promise<PlaystyleScore> {
    void cards;
    void commanderId;
    throw new NotImplementedError("calculateDeckPlaystyle");
  }
}

/**
 * WASM/IndexedDB implementation of the storage backend (stub).
 */
export class WasmStorageBackend implements StorageBackend {
  readonly platform: Platform = "web";

  async init(): Promise<void> {
    // TODO: Phase 2 - Initialize IndexedDB
    console.warn(
      "WasmStorageBackend: IndexedDB support not yet implemented. " +
        "This stub will be replaced in Phase 2."
    );
  }

  // ===========================================================================
  // Replays
  // ===========================================================================

  async saveReplay(gameId: string, name?: string): Promise<string> {
    void gameId;
    void name;
    throw new NotImplementedError("saveReplay");
  }

  async saveSpectatorReplay(
    match: SpectatorMatch,
    name?: string
  ): Promise<string> {
    void match;
    void name;
    throw new NotImplementedError("saveSpectatorReplay");
  }

  async listReplays(): Promise<ReplayInfo[]> {
    throw new NotImplementedError("listReplays");
  }

  async loadReplay(pathOrId: string): Promise<SpectatorMatch> {
    void pathOrId;
    throw new NotImplementedError("loadReplay");
  }

  async deleteReplay(pathOrId: string): Promise<void> {
    void pathOrId;
    throw new NotImplementedError("deleteReplay");
  }

  // ===========================================================================
  // Custom Decks
  // ===========================================================================

  async saveCustomDeck(deck: CustomDeck): Promise<string> {
    void deck;
    throw new NotImplementedError("saveCustomDeck");
  }

  async listCustomDecks(): Promise<CustomDeckInfo[]> {
    throw new NotImplementedError("listCustomDecks");
  }

  async loadCustomDeck(deckId: string): Promise<CustomDeck> {
    void deckId;
    throw new NotImplementedError("loadCustomDeck");
  }

  async deleteCustomDeck(deckId: string): Promise<void> {
    void deckId;
    throw new NotImplementedError("deleteCustomDeck");
  }
}

/**
 * WASM implementation of MCP backend.
 * MCP sync is not supported on web - it's a desktop-only feature.
 */
export class WasmMcpBackend implements McpBackend {
  readonly supported = false;

  async getMcpSyncedState(): Promise<null> {
    return null;
  }

  async hasMcpSyncedState(): Promise<boolean> {
    return false;
  }

  async clearMcpSyncedState(): Promise<void> {
    // No-op on web
  }
}
