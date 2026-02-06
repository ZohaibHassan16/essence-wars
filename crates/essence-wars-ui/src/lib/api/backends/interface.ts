/**
 * Backend abstraction for Essence Wars game engine.
 *
 * This interface defines the contract between the Svelte UI and the Tauri backend.
 */

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

export type Platform = "tauri";

/**
 * Core game engine operations.
 * These are the pure game logic functions that both backends must implement.
 */
export interface GameBackend {
  /** Platform identifier */
  readonly platform: Platform;

  /** Initialize the backend (load WASM, connect to Tauri, etc.) */
  init(): Promise<void>;

  // ===========================================================================
  // Game Setup
  // ===========================================================================

  /** List all available decks */
  listDecks(): Promise<DeckInfo[]>;

  /** List all available bot types */
  listBots(): Promise<BotInfo[]>;

  /** Get all cards in a deck */
  getDeckCards(deckId: string): Promise<CardDto[]>;

  // ===========================================================================
  // Game Session
  // ===========================================================================

  /** Start a new game with the given configuration */
  newGame(config: GameConfig): Promise<GameStateDto>;

  /** Get current game state */
  getGameState(gameId: string): Promise<GameStateDto>;

  /** Get list of legal actions for current player */
  getLegalActions(gameId: string): Promise<ActionInfo[]>;

  /** Apply an action and get the updated state */
  applyAction(gameId: string, actionIndex: number): Promise<GameStateUpdate>;

  /** Get AI's recommended move */
  getAiMove(gameId: string): Promise<ActionInfo>;

  /** Get AI hint with analysis */
  getAiHint(gameId: string): Promise<AiHintResponse>;

  /** End the game and get results */
  endGame(gameId: string): Promise<GameResultDto>;

  /** Undo the last action */
  undoAction(gameId: string): Promise<GameStateDto>;

  /** Check if undo is available */
  canUndo(gameId: string): Promise<boolean>;

  // ===========================================================================
  // Spectator Mode
  // ===========================================================================

  /** Compute a complete AI vs AI match */
  computeSpectatorMatch(config: SpectatorConfig): Promise<SpectatorMatch>;

  // ===========================================================================
  // Deck Builder
  // ===========================================================================

  /** List all cards, optionally filtered by faction */
  listAllCards(faction?: string): Promise<BrowsableCard[]>;

  /** List all commanders */
  listCommanders(): Promise<CommanderDto[]>;

  /** Validate a custom deck configuration */
  validateCustomDeck(deck: CustomDeck): Promise<DeckValidation>;

  /** Calculate the playstyle of a deck */
  calculateDeckPlaystyle(
    cards: number[],
    commanderId: number
  ): Promise<PlaystyleScore>;
}

/**
 * Storage operations for replays and custom decks.
 */
export interface StorageBackend {
  /** Platform identifier */
  readonly platform: Platform;

  /** Initialize storage (open IndexedDB, etc.) */
  init(): Promise<void>;

  // ===========================================================================
  // Replays
  // ===========================================================================

  /** Save a replay from a game session */
  saveReplay(gameId: string, name?: string): Promise<string>;

  /** Save a spectator match as a replay */
  saveSpectatorReplay(match: SpectatorMatch, name?: string): Promise<string>;

  /** List all saved replays */
  listReplays(): Promise<ReplayInfo[]>;

  /** Load a replay by path/id */
  loadReplay(pathOrId: string): Promise<SpectatorMatch>;

  /** Delete a replay */
  deleteReplay(pathOrId: string): Promise<void>;

  // ===========================================================================
  // Custom Decks
  // ===========================================================================

  /** Save a custom deck */
  saveCustomDeck(deck: CustomDeck): Promise<string>;

  /** List all custom decks (metadata only) */
  listCustomDecks(): Promise<CustomDeckInfo[]>;

  /** Load a custom deck by ID */
  loadCustomDeck(deckId: string): Promise<CustomDeck>;

  /** Delete a custom deck */
  deleteCustomDeck(deckId: string): Promise<void>;
}

/**
 * MCP (Model Context Protocol) sync operations.
 * Desktop-only feature for Claude Code integration.
 */
export interface McpBackend {
  /** Check if MCP sync is supported on this platform */
  readonly supported: boolean;

  /** Get the current MCP-synced game state */
  getMcpSyncedState(): Promise<McpSyncedState | null>;

  /** Check if there's an MCP-synced state available */
  hasMcpSyncedState(): Promise<boolean>;

  /** Clear the MCP-synced state */
  clearMcpSyncedState(): Promise<void>;
}
