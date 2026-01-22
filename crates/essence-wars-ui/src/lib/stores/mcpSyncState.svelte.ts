// MCP Sync state store using Svelte 5 runes
//
// This store manages displaying game state that was synced from the MCP server.
// It polls the Tauri backend for synced state and displays it when available.

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
  private readonly POLL_INTERVAL_MS = 500; // Poll every 500ms
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

  /** Start watching for MCP-synced game state */
  startWatching() {
    if (this.isPolling) return;

    this.isPolling = true;
    this.phase = "watching";
    this.error = null;

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
