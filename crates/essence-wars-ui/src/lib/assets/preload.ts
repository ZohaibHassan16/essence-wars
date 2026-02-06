/**
 * Essential asset preloading for app initialization.
 *
 * This module handles the initial load of critical assets needed
 * for the app to function. Less critical assets are lazy-loaded.
 *
 * Asset Loading Priority:
 * 1. WASM module (handled by backend init)
 * 2. Essential UI elements (loading screen, etc.)
 * 3. Commander portraits (needed for deck selection)
 * 4. Card art (lazy loaded during gameplay)
 * 5. Audio (deferred, loaded in background)
 */

import { assetLoader } from "./loader.svelte";
import { getPlatform } from "$lib/api/backends";

/**
 * Essential asset paths that must load before app is interactive.
 */
const ESSENTIAL_ASSETS = {
  // UI elements needed for loading screen and basic UI
  ui: [
    // Add essential UI assets here as needed
  ],

  // Commander portraits - needed for deck selection wizard
  commanders: [
    // Dynamically generated in preloadCommanders()
  ],
};

/**
 * Asset paths that can be deferred but should load early.
 */
const DEFERRED_ASSETS = {
  // Backgrounds for the menu and game board
  backgrounds: [
    "/backgrounds/menu_bg.webp",
    "/backgrounds/game_board.webp",
  ],
};

/**
 * Preload essential assets for app startup.
 * Returns a promise that resolves when all critical assets are loaded.
 */
export async function preloadEssentialAssets(): Promise<void> {
  const platform = getPlatform();
  console.log(`[Preload] Starting essential asset preload (platform: ${platform})`);

  const startTime = performance.now();

  try {
    // Phase 1: Essential UI (parallel)
    await assetLoader.preloadEssentials();

    // Phase 2: Commander portraits (parallel)
    await assetLoader.preloadCommanders();

    const duration = Math.round(performance.now() - startTime);
    console.log(`[Preload] Essential assets loaded in ${duration}ms`);
    console.log(`[Preload] Stats: ${assetLoader.totalLoaded} loaded, ${assetLoader.totalFailed} failed`);
  } catch (error) {
    console.error("[Preload] Failed to load essential assets:", error);
    // Don't throw - app should still work with missing assets
  }
}

/**
 * Preload deferred assets in the background.
 * Call this after the app becomes interactive.
 */
export async function preloadDeferredAssets(): Promise<void> {
  console.log("[Preload] Starting deferred asset preload...");

  // Load backgrounds in background
  assetLoader.preloadBatch(DEFERRED_ASSETS.backgrounds).catch((err) => {
    console.warn("[Preload] Some deferred assets failed:", err);
  });
}

/**
 * Preload card art for a specific deck.
 * Call this when a deck is selected.
 */
export async function preloadDeckCards(cardIds: number[]): Promise<void> {
  console.log(`[Preload] Preloading ${cardIds.length} deck cards...`);

  const paths = cardIds.map((id) => {
    // Determine card type folder based on ID range
    // This is a simplified approach - actual path depends on card type
    return `/cards/creatures/${id}.webp`;
  });

  await assetLoader.preloadBatch(paths);
}

/**
 * Get the loading progress percentage.
 * Useful for loading screens.
 */
export function getLoadingProgress(): {
  loaded: number;
  failed: number;
  percentage: number;
} {
  const loaded = assetLoader.totalLoaded;
  const failed = assetLoader.totalFailed;

  // Estimate total based on known essential assets
  const estimatedTotal = 20; // UI + commanders
  const percentage = Math.min(
    100,
    Math.round(((loaded + failed) / estimatedTotal) * 100)
  );

  return { loaded, failed, percentage };
}

/**
 * Check if essential assets are loaded.
 */
export function areEssentialsLoaded(): boolean {
  return (
    assetLoader.isCategoryPreloaded("ui") &&
    assetLoader.isCategoryPreloaded("commander")
  );
}
