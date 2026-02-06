/**
 * Asset loading module.
 *
 * Provides progressive asset loading with:
 * - Reactive loading state
 * - Lazy loading via intersection observer
 * - Preloading for essential assets
 */

export {
  assetLoader,
  lazyLoadImage,
  type AssetStatus,
  type AssetState,
  type AssetCategory,
} from "./loader.svelte";

export {
  preloadEssentialAssets,
  preloadDeferredAssets,
  preloadDeckCards,
  getLoadingProgress,
  areEssentialsLoaded,
} from "./preload";
