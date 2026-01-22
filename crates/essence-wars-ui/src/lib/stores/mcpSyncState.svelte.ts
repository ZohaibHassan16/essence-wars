// MCP Sync state store using Svelte 5 runes
//
// This store manages displaying game state that was synced from the MCP server.
// It polls the Tauri backend for synced state and displays it when available.
//
// AUTO-SWITCH MODE: The store automatically polls for synced state even when idle,
// and will auto-switch to "watching" phase when MCP syncs game state.

import type { GameStateDto } from "$lib/api/types";
import * as api from "$lib/api/game";

export type McpSyncPhase = "idle" | "watching" | "disconnected";

class McpSyncStore {
  // Phase management
  phase = $state<McpSyncPhase>("idle");

  // Current synced game state
  gameState = $state<GameStateDto | null>(null);

  // Metadata
  lastSyncTimestamp = $state<number>(0);
  stateAgeMs = $state<number>(0);

  // Polling
  isPolling = $state<boolean>(false);
  private pollInterval: ReturnType<typeof setInterval> | null = null;
  private autoWatchInterval: ReturnType<typeof setInterval> | null = null;
  private readonly POLL_INTERVAL_MS = 500; // Poll every 500ms
  private readonly AUTO_WATCH_INTERVAL_MS = 1000; // Check for state every 1s when idle
  private readonly STALE_THRESHOLD_MS = 5000; // Consider stale after 5s

  // Error handling
  error = $state<string | null>(null);

  // ============================================================================
  // Computed Properties
  // ============================================================================

  /** Whether the state is considered stale (no recent updates) */
  get isStale(): boolean {
    return this.stateAgeMs > this.STALE_THRESHOLD_MS;
  }

  /** Whether player 1 is the active player */
  get isP1Turn(): boolean {
    return this.gameState?.activePlayer === 1;
  }

  /** Current turn number */
  get currentTurn(): number {
    return this.gameState?.turn ?? 0;
  }

  // ============================================================================
  // Actions
  // ============================================================================

  /**
   * Start auto-watch mode - polls in background and auto-switches to watching
   * when MCP syncs game state. Call this on app initialization.
   */
  startAutoWatch() {
    if (this.autoWatchInterval) return;

    this.autoWatchInterval = setInterval(() => {
      this.checkForAutoSwitch();
    }, this.AUTO_WATCH_INTERVAL_MS);

    // Initial check
    this.checkForAutoSwitch();
  }

  /** Stop auto-watch mode */
  stopAutoWatch() {
    if (this.autoWatchInterval) {
      clearInterval(this.autoWatchInterval);
      this.autoWatchInterval = null;
    }
  }

  /** Check for synced state and auto-switch to watching if found */
  private async checkForAutoSwitch() {
    // Only auto-switch when in idle phase
    if (this.phase !== "idle") return;

    try {
      const result = await api.getMcpSyncedState();
      if (result && result.ageMs < this.STALE_THRESHOLD_MS) {
        // Fresh state detected - auto-switch to watching mode
        this.gameState = result.state;
        this.lastSyncTimestamp = result.timestamp;
        this.stateAgeMs = result.ageMs;
        this.phase = "watching";
        this.error = null;

        // Start active polling now that we're watching
        this.startActivePolling();
      }
    } catch (e) {
      // Silently ignore errors during auto-watch - it's just background checking
      console.debug("MCP auto-watch check error:", e);
    }
  }

  /** Start watching for MCP-synced game state (manual trigger) */
  startWatching() {
    if (this.isPolling) return;

    this.phase = "watching";
    this.error = null;
    this.startActivePolling();
  }

  /** Start active polling (internal) */
  private startActivePolling() {
    if (this.isPolling) return;

    this.isPolling = true;

    // Initial poll
    this.pollSyncedState();

    // Start polling interval
    this.pollInterval = setInterval(() => {
      this.pollSyncedState();
    }, this.POLL_INTERVAL_MS);
  }

  /** Stop watching for MCP-synced game state */
  stopWatching() {
    this.isPolling = false;
    this.phase = "idle";

    if (this.pollInterval) {
      clearInterval(this.pollInterval);
      this.pollInterval = null;
    }

    this.gameState = null;
    this.lastSyncTimestamp = 0;
    this.stateAgeMs = 0;
  }

  /** Poll for synced state from Tauri backend */
  private async pollSyncedState() {
    try {
      const result = await api.getMcpSyncedState();

      if (result) {
        this.gameState = result.state;
        this.lastSyncTimestamp = result.timestamp;
        this.stateAgeMs = result.ageMs;
        this.phase = "watching";
        this.error = null;
      } else {
        // No synced state available
        this.stateAgeMs = this.lastSyncTimestamp > 0
          ? Date.now() - this.lastSyncTimestamp
          : 0;

        // If we had a state before but now don't, mark as disconnected
        if (this.gameState !== null && this.stateAgeMs > this.STALE_THRESHOLD_MS) {
          this.phase = "disconnected";
        }
      }
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      console.error("MCP sync poll error:", e);
    }
  }

  /** Clear synced state and return to idle */
  reset() {
    this.stopWatching();
    api.clearMcpSyncedState().catch(e => {
      console.error("Failed to clear MCP synced state:", e);
    });
  }
}

export const mcpSyncStore = new McpSyncStore();

// Auto-start the background watcher when this module loads
// This enables auto-switch to MCP sync view when state is received
mcpSyncStore.startAutoWatch();
