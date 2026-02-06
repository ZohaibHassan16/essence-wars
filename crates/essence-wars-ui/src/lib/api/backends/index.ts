/**
 * Backend factory and detection.
 *
 * This module provides runtime detection of the platform (Tauri vs Web)
 * and lazy initialization of the appropriate backend implementations.
 */

import type { GameBackend, StorageBackend, McpBackend, Platform } from "./interface";

// Re-export types for convenience
export type { GameBackend, StorageBackend, McpBackend, Platform } from "./interface";

/**
 * Detect the current platform at runtime.
 * Tauri injects `__TAURI__` into the window object.
 */
export function detectPlatform(): Platform {
  if (typeof window !== "undefined" && "__TAURI__" in window) {
    return "tauri";
  }
  return "web";
}

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
  if (_initialized) return;

  const platform = detectPlatform();
  console.log(`[Backend] Detected platform: ${platform}`);

  if (platform === "tauri") {
    const { TauriGameBackend, TauriStorageBackend, TauriMcpBackend } =
      await import("./tauri");

    _gameBackend = new TauriGameBackend();
    _storageBackend = new TauriStorageBackend();
    _mcpBackend = new TauriMcpBackend();
  } else {
    const { WasmGameBackend, WasmStorageBackend, WasmMcpBackend } =
      await import("./wasm");

    _gameBackend = new WasmGameBackend();
    _storageBackend = new WasmStorageBackend();
    _mcpBackend = new WasmMcpBackend();
  }

  // Initialize all backends
  await Promise.all([
    _gameBackend.init(),
    _storageBackend.init(),
  ]);

  _initialized = true;
  console.log(`[Backend] Initialized successfully`);
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
 * Returns the detected platform even before initialization.
 */
export function getPlatform(): Platform {
  if (_gameBackend) {
    return _gameBackend.platform;
  }
  return detectPlatform();
}
