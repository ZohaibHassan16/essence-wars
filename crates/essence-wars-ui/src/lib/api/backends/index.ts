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
 * Tauri 2 injects `__TAURI_INTERNALS__` into the window object.
 */
export function detectPlatform(): Platform {
  if (typeof window === "undefined") {
    return "web";
  }

  // Tauri 2.x uses __TAURI_INTERNALS__
  // Tauri 1.x used __TAURI__
  const isTauri = "__TAURI_INTERNALS__" in window || "__TAURI__" in window;

  console.log("[Backend] Platform detection:", {
    hasTauriInternals: "__TAURI_INTERNALS__" in window,
    hasTauri: "__TAURI__" in window,
    detected: isTauri ? "tauri" : "web",
  });

  return isTauri ? "tauri" : "web";
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
  if (_initialized) {
    console.log("[Backend] Already initialized, skipping");
    return;
  }

  const platform = detectPlatform();
  console.log(`[Backend] Initializing for platform: ${platform}`);

  try {
    if (platform === "tauri") {
      console.log("[Backend] Loading Tauri backend modules...");
      const { TauriGameBackend, TauriStorageBackend, TauriMcpBackend } =
        await import("./tauri");

      _gameBackend = new TauriGameBackend();
      _storageBackend = new TauriStorageBackend();
      _mcpBackend = new TauriMcpBackend();
      console.log("[Backend] Tauri backends created");
    } else {
      console.log("[Backend] Loading WASM backend modules...");
      const { WasmGameBackend, WasmStorageBackend, WasmMcpBackend } =
        await import("./wasm");

      _gameBackend = new WasmGameBackend();
      _storageBackend = new WasmStorageBackend();
      _mcpBackend = new WasmMcpBackend();
      console.log("[Backend] WASM backends created");
    }

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
 * Returns the detected platform even before initialization.
 */
export function getPlatform(): Platform {
  if (_gameBackend) {
    return _gameBackend.platform;
  }
  return detectPlatform();
}
