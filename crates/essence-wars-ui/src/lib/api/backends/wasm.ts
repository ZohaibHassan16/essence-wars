/**
 * WASM backend implementation.
 *
 * Game backend: Full WASM implementation
 * Storage backend: IndexedDB implementation
 * MCP backend: Not supported on web
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
import * as idb from "../../storage/indexeddb";
import type { StoredReplay, StoredCustomDeck } from "../../storage/indexeddb";

// WASM module imports
import wasmInit, {
  WasmGameManager,
  compute_spectator_match,
  init as wasmPanicHook,
} from "$wasm";

/**
 * Generate a unique ID for replays and decks.
 */
function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
}

/**
 * Format a date for display.
 */
function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleString();
}

/**
 * WASM implementation of the game backend.
 *
 * Uses the Rust game engine compiled to WebAssembly for full game logic
 * running entirely in the browser.
 */
export class WasmGameBackend implements GameBackend {
  readonly platform: Platform = "web";
  private manager: WasmGameManager | null = null;
  private initialized = false;

  async init(): Promise<void> {
    if (this.initialized) {
      return;
    }

    console.log("[WasmGameBackend] Initializing WASM module...");

    // Initialize the WASM module
    await wasmInit();

    // Set up panic hook for better error messages
    wasmPanicHook();

    // Create the game manager
    this.manager = new WasmGameManager();
    this.initialized = true;

    console.log("[WasmGameBackend] WASM module initialized successfully");
  }

  private getManager(): WasmGameManager {
    if (!this.manager) {
      throw new Error(
        "WasmGameBackend not initialized. Call init() first."
      );
    }
    return this.manager;
  }

  // ===========================================================================
  // Game Setup
  // ===========================================================================

  async listDecks(): Promise<DeckInfo[]> {
    const json = this.getManager().list_decks();
    return JSON.parse(json) as DeckInfo[];
  }

  async listBots(): Promise<BotInfo[]> {
    const json = this.getManager().list_bots();
    return JSON.parse(json) as BotInfo[];
  }

  async getDeckCards(deckId: string): Promise<CardDto[]> {
    const json = this.getManager().get_deck_cards(deckId);
    return JSON.parse(json) as CardDto[];
  }

  // ===========================================================================
  // Game Session
  // ===========================================================================

  async newGame(config: GameConfig): Promise<GameStateDto> {
    // WASM expects camelCase (Rust struct has #[serde(rename_all = "camelCase")])
    const wasmConfig = {
      playerDeckId: config.playerDeckId,
      opponentDeckId: config.opponentDeckId,
      opponentBotType: config.opponentBotType,
      playerGoesFirst: config.playerGoesFirst ?? true,
      seed: config.seed,
    };

    const json = this.getManager().new_game(JSON.stringify(wasmConfig));
    return JSON.parse(json) as GameStateDto;
  }

  async getGameState(gameId: string): Promise<GameStateDto> {
    const json = this.getManager().get_game_state(gameId);
    return JSON.parse(json) as GameStateDto;
  }

  async getLegalActions(gameId: string): Promise<ActionInfo[]> {
    const json = this.getManager().get_legal_actions(gameId);
    return JSON.parse(json) as ActionInfo[];
  }

  async applyAction(
    gameId: string,
    actionIndex: number
  ): Promise<GameStateUpdate> {
    const json = this.getManager().apply_action(gameId, actionIndex);
    return JSON.parse(json) as GameStateUpdate;
  }

  async getAiMove(gameId: string): Promise<ActionInfo> {
    const json = this.getManager().get_ai_move(gameId);
    return JSON.parse(json) as ActionInfo;
  }

  async getAiHint(gameId: string): Promise<AiHintResponse> {
    const json = this.getManager().get_ai_hint(gameId);
    return JSON.parse(json) as AiHintResponse;
  }

  async endGame(gameId: string): Promise<GameResultDto> {
    const json = this.getManager().end_game(gameId);
    return JSON.parse(json) as GameResultDto;
  }

  async undoAction(gameId: string): Promise<GameStateDto> {
    const json = this.getManager().undo_action(gameId);
    return JSON.parse(json) as GameStateDto;
  }

  async canUndo(gameId: string): Promise<boolean> {
    return this.getManager().can_undo(gameId);
  }

  // ===========================================================================
  // Spectator Mode
  // ===========================================================================

  async computeSpectatorMatch(
    config: SpectatorConfig
  ): Promise<SpectatorMatch> {
    // Rust SpectatorConfig has #[serde(rename_all = "camelCase")] so we send camelCase
    const wasmConfig = {
      player1DeckId: config.player1DeckId,
      player1BotType: config.player1BotType,
      player2DeckId: config.player2DeckId,
      player2BotType: config.player2BotType,
      seed: config.seed,
      mctsSimulations: config.mctsSimulations,
      alphabetaDepth: config.alphabetaDepth,
    };

    const json = compute_spectator_match(JSON.stringify(wasmConfig));
    return JSON.parse(json) as SpectatorMatch;
  }

  // ===========================================================================
  // Deck Builder
  // ===========================================================================

  async listAllCards(faction?: string): Promise<BrowsableCard[]> {
    const json = this.getManager().list_all_cards(faction ?? null);
    return JSON.parse(json) as BrowsableCard[];
  }

  async listCommanders(): Promise<CommanderDto[]> {
    const json = this.getManager().list_commanders();
    return JSON.parse(json) as CommanderDto[];
  }

  async validateCustomDeck(deck: CustomDeck): Promise<DeckValidation> {
    const json = this.getManager().validate_custom_deck(JSON.stringify(deck));
    return JSON.parse(json) as DeckValidation;
  }

  async calculateDeckPlaystyle(
    cards: number[],
    commanderId: number
  ): Promise<PlaystyleScore> {
    const json = this.getManager().calculate_deck_playstyle(
      JSON.stringify(cards),
      commanderId
    );
    return JSON.parse(json) as PlaystyleScore;
  }
}

/**
 * IndexedDB implementation of the storage backend for web.
 */
export class WasmStorageBackend implements StorageBackend {
  readonly platform: Platform = "web";

  async init(): Promise<void> {
    // Open the IndexedDB database
    await idb.openDatabase();
    console.log("[WasmStorageBackend] IndexedDB initialized");
  }

  // ===========================================================================
  // Replays
  // ===========================================================================

  async saveReplay(gameId: string, name?: string): Promise<string> {
    // On web, saveReplay requires the game state to be passed in.
    // This is different from Tauri where the backend can access the game.
    // For now, throw an error - use saveSpectatorReplay instead.
    void gameId;
    void name;
    throw new Error(
      "saveReplay() is not supported on web. Use saveSpectatorReplay() instead."
    );
  }

  async saveSpectatorReplay(
    match: SpectatorMatch,
    name?: string
  ): Promise<string> {
    const id = generateId();
    const timestamp = Date.now();

    const storedReplay: StoredReplay = {
      id,
      name: name || `${match.player1DeckName} vs ${match.player2DeckName}`,
      timestamp,
      player1DeckName: match.player1DeckName,
      player2DeckName: match.player2DeckName,
      player1Type: match.player1BotName || "Human",
      player2Type: match.player2BotName,
      winner: match.result.winner,
      totalTurns: match.totalTurns,
      totalActions: match.actions.length,
      data: JSON.stringify(match),
    };

    await idb.saveReplay(storedReplay);
    console.log(`[WasmStorageBackend] Saved replay: ${id}`);
    return id;
  }

  async listReplays(): Promise<ReplayInfo[]> {
    const storedReplays = await idb.getAllReplays();

    return storedReplays.map((replay) => ({
      path: replay.id, // Use ID as path for web
      filename: replay.name,
      timestamp: replay.timestamp,
      dateString: formatDate(replay.timestamp),
      player1DeckName: replay.player1DeckName,
      player2DeckName: replay.player2DeckName,
      player1Type: replay.player1Type,
      player2Type: replay.player2Type,
      winner: replay.winner,
      totalTurns: replay.totalTurns,
      totalActions: replay.totalActions,
    }));
  }

  async loadReplay(pathOrId: string): Promise<SpectatorMatch> {
    const storedReplay = await idb.getReplay(pathOrId);

    if (!storedReplay) {
      throw new Error(`Replay not found: ${pathOrId}`);
    }

    return JSON.parse(storedReplay.data) as SpectatorMatch;
  }

  async deleteReplay(pathOrId: string): Promise<void> {
    await idb.deleteReplay(pathOrId);
  }

  // ===========================================================================
  // Custom Decks
  // ===========================================================================

  async saveCustomDeck(deck: CustomDeck): Promise<string> {
    const now = new Date().toISOString();
    const existing = await idb.getCustomDeck(deck.id);

    const storedDeck: StoredCustomDeck = {
      id: deck.id,
      name: deck.name,
      commander: deck.commander,
      cards: deck.cards,
      description: deck.description,
      tags: deck.tags,
      createdAt: existing?.createdAt || now,
      modifiedAt: now,
    };

    await idb.saveCustomDeck(storedDeck);
    console.log(`[WasmStorageBackend] Saved custom deck: ${deck.id}`);
    return deck.id;
  }

  async listCustomDecks(): Promise<CustomDeckInfo[]> {
    const storedDecks = await idb.getAllCustomDecks();

    // Note: We don't have access to commander name or playstyle calculation
    // on web without the WASM engine. Return basic info for now.
    return storedDecks.map((deck) => ({
      id: deck.id,
      name: deck.name,
      description: deck.description,
      commanderId: deck.commander,
      commanderName: `Commander #${deck.commander}`, // Placeholder until WASM
      faction: "unknown", // Would need WASM to determine
      cardCount: deck.cards.length,
      createdAt: deck.createdAt,
      modifiedAt: deck.modifiedAt,
    }));
  }

  async loadCustomDeck(deckId: string): Promise<CustomDeck> {
    const storedDeck = await idb.getCustomDeck(deckId);

    if (!storedDeck) {
      throw new Error(`Custom deck not found: ${deckId}`);
    }

    return {
      id: storedDeck.id,
      name: storedDeck.name,
      commander: storedDeck.commander,
      cards: storedDeck.cards,
      description: storedDeck.description,
      tags: storedDeck.tags,
    };
  }

  async deleteCustomDeck(deckId: string): Promise<void> {
    await idb.deleteCustomDeck(deckId);
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
