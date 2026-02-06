/**
 * Backend factory for Essence Wars game engine.
 *
 * This module provides lazy initialization of the Tauri backend implementations.
 */

import type { GameBackend, StorageBackend, McpBackend, Platform } from "./interface";

// Re-export types for convenience
export type { GameBackend, StorageBackend, McpBackend, Platform } from "./interface";

// Singleton instances (lazy initialized)
let _gameBackend: GameBackend | null = null;
let _storageBackend: StorageBackend | null = null;
let _mcpBackend: McpBackend | null = null;
let _initialized = false;

/**
 * Initialize all backends.
 * Call this once at app startup.
 */
export async function initBackends(): Promise<void> {
  if (_initialized) {
    console.log("[Backend] Already initialized, skipping");
    return;
  }

  console.log("[Backend] Initializing Tauri backends...");

  try {
    const { TauriGameBackend, TauriStorageBackend, TauriMcpBackend } =
      await import("./tauri");

    _gameBackend = new TauriGameBackend();
    _storageBackend = new TauriStorageBackend();
    _mcpBackend = new TauriMcpBackend();
    console.log("[Backend] Tauri backends created");

    // Initialize all backends
    console.log("[Backend] Calling init() on backends...");
    await Promise.all([
      _gameBackend.init(),
      _storageBackend.init(),
    ]);

    _initialized = true;
    console.log("[Backend] Initialization complete!");
  } catch (error) {
    console.error("[Backend] Initialization failed:", error);
    throw error;
  }
}

/**
 * Get the game backend instance.
 * Throws if backends haven't been initialized.
 */
export function getGameBackend(): GameBackend {
  if (!_gameBackend) {
    throw new Error(
      "Game backend not initialized. Call initBackends() first."
    );
  }
  return _gameBackend;
}

/**
 * Get the storage backend instance.
 * Throws if backends haven't been initialized.
 */
export function getStorageBackend(): StorageBackend {
  if (!_storageBackend) {
    throw new Error(
      "Storage backend not initialized. Call initBackends() first."
    );
  }
  return _storageBackend;
}

/**
 * Get the MCP backend instance.
 * Throws if backends haven't been initialized.
 */
export function getMcpBackend(): McpBackend {
  if (!_mcpBackend) {
    throw new Error(
      "MCP backend not initialized. Call initBackends() first."
    );
  }
  return _mcpBackend;
}

/**
 * Check if backends have been initialized.
 */
export function isInitialized(): boolean {
  return _initialized;
}

/**
 * Get the current platform.
 */
export function getPlatform(): Platform {
  return "tauri";
}
