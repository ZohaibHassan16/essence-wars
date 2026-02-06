/**
 * Progressive asset loader using Svelte 5 runes.
 *
 * Provides reactive state for asset loading with:
 * - Track loaded/loading/error states per asset
 * - Preload essential assets on init
 * - Lazy load card art on demand
 * - Intersection observer support for viewport-based loading
 */

import { SvelteMap, SvelteSet } from "svelte/reactivity";

export type AssetStatus = "idle" | "loading" | "loaded" | "error";

export interface AssetState {
  status: AssetStatus;
  url: string;
  error?: string;
}

/**
 * Asset categories for organization and priority.
 */
export type AssetCategory =
  | "commander" // Commander portraits - preloaded
  | "card" // Card art - lazy loaded
  | "ui" // UI elements - preloaded
  | "background" // Background images - lazy loaded
  | "token" // Token art - lazy loaded
  | "audio"; // Audio files - deferred

/**
 * Progressive asset loader with reactive state.
 */
class AssetLoader {
  // Reactive state using SvelteMap/SvelteSet
  private assets = new SvelteMap<string, AssetState>();
  private preloadedCategories = new SvelteSet<AssetCategory>();
  private loadingQueue = new SvelteSet<string>();

  // Statistics
  private _totalLoaded = $state(0);
  private _totalFailed = $state(0);
  private _totalBytes = $state(0);

  /**
   * Get the current status of an asset.
   */
  getStatus(path: string): AssetStatus {
    return this.assets.get(path)?.status ?? "idle";
  }

  /**
   * Check if an asset is loaded.
   */
  isLoaded(path: string): boolean {
    return this.getStatus(path) === "loaded";
  }

  /**
   * Check if an asset is currently loading.
   */
  isLoading(path: string): boolean {
    return this.getStatus(path) === "loading";
  }

  /**
   * Get total number of loaded assets.
   */
  get totalLoaded(): number {
    return this._totalLoaded;
  }

  /**
   * Get total number of failed assets.
   */
  get totalFailed(): number {
    return this._totalFailed;
  }

  /**
   * Check if a category has been preloaded.
   */
  isCategoryPreloaded(category: AssetCategory): boolean {
    return this.preloadedCategories.has(category);
  }

  /**
   * Load a single image asset.
   * Returns immediately if already loaded/loading.
   */
  async loadImage(path: string): Promise<string> {
    // Already loaded
    if (this.isLoaded(path)) {
      return path;
    }

    // Already loading - wait for it
    if (this.isLoading(path)) {
      return this.waitForLoad(path);
    }

    // Start loading
    this.assets.set(path, { status: "loading", url: path });
    this.loadingQueue.add(path);

    return new Promise((resolve, reject) => {
      const img = new Image();

      img.onload = () => {
        this.assets.set(path, { status: "loaded", url: path });
        this.loadingQueue.delete(path);
        this._totalLoaded++;
        resolve(path);
      };

      img.onerror = () => {
        const error = `Failed to load: ${path}`;
        this.assets.set(path, { status: "error", url: path, error });
        this.loadingQueue.delete(path);
        this._totalFailed++;
        reject(new Error(error));
      };

      img.src = path;
    });
  }

  /**
   * Wait for an already-loading asset to complete.
   */
  private waitForLoad(path: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const check = () => {
        const status = this.getStatus(path);
        if (status === "loaded") {
          resolve(path);
        } else if (status === "error") {
          reject(new Error(this.assets.get(path)?.error));
        } else {
          // Still loading, check again
          requestAnimationFrame(check);
        }
      };
      check();
    });
  }

  /**
   * Load a card image by card ID.
   * Handles the card art path convention.
   */
  async loadCard(cardId: number): Promise<string> {
    const path = `/cards/creatures/${cardId}.webp`;

    // Try creatures first, then spells, then supports
    try {
      return await this.loadImage(path);
    } catch {
      // Try spells folder
      const spellPath = `/cards/spells/${cardId}.webp`;
      try {
        return await this.loadImage(spellPath);
      } catch {
        // Try supports folder
        const supportPath = `/cards/supports/${cardId}.webp`;
        return await this.loadImage(supportPath);
      }
    }
  }

  /**
   * Load a commander portrait.
   */
  async loadCommander(commanderId: number): Promise<string> {
    // Commander portraits use a name-based path
    // We'll need to map ID to name or use ID directly
    const path = `/portrait/commander_${commanderId}.webp`;
    return this.loadImage(path);
  }

  /**
   * Load a token image.
   */
  async loadToken(tokenId: number): Promise<string> {
    const path = `/tokens/${tokenId}.webp`;
    return this.loadImage(path);
  }

  /**
   * Load a background image.
   */
  async loadBackground(name: string): Promise<string> {
    const path = `/backgrounds/${name}.webp`;
    return this.loadImage(path);
  }

  /**
   * Preload a batch of images in parallel.
   * Continues even if some fail.
   */
  async preloadBatch(paths: string[]): Promise<void> {
    const promises = paths.map((path) =>
      this.loadImage(path).catch((err) => {
        console.warn(`[AssetLoader] Failed to preload: ${path}`, err);
        return null;
      })
    );

    await Promise.all(promises);
  }

  /**
   * Preload essential UI assets.
   */
  async preloadEssentials(): Promise<void> {
    if (this.isCategoryPreloaded("ui")) {
      return;
    }

    console.log("[AssetLoader] Preloading essential UI assets...");

    const essentialPaths = [
      // UI elements
      "/ui/card-back.webp",
      "/ui/card-frame.webp",
      "/ui/board-bg.webp",
    ];

    await this.preloadBatch(essentialPaths);
    this.preloadedCategories.add("ui");

    console.log("[AssetLoader] Essential UI assets loaded");
  }

  /**
   * Preload commander portraits.
   */
  async preloadCommanders(): Promise<void> {
    if (this.isCategoryPreloaded("commander")) {
      return;
    }

    console.log("[AssetLoader] Preloading commander portraits...");

    // Commander IDs: 5000-5011 (12 commanders)
    const commanderPaths: string[] = [];
    for (let i = 5000; i <= 5011; i++) {
      commanderPaths.push(`/portrait/commander_${i}.webp`);
    }

    await this.preloadBatch(commanderPaths);
    this.preloadedCategories.add("commander");

    console.log("[AssetLoader] Commander portraits loaded");
  }

  /**
   * Create an intersection observer action for lazy loading.
   * Use with Svelte's use: directive.
   *
   * Example:
   * <img use:lazyLoad={'/cards/1000.webp'} />
   */
  lazyLoad = (
    node: HTMLImageElement,
    path: string
  ): { destroy: () => void } => {
    // Set placeholder initially
    node.style.opacity = "0";
    node.style.transition = "opacity 0.3s ease-in-out";

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            // Load the image
            this.loadImage(path)
              .then(() => {
                node.src = path;
                node.style.opacity = "1";
              })
              .catch(() => {
                // Could set a fallback image here
                node.style.opacity = "0.5";
              });

            // Stop observing once loaded
            observer.unobserve(node);
          }
        });
      },
      {
        rootMargin: "50px", // Start loading slightly before visible
        threshold: 0.1,
      }
    );

    observer.observe(node);

    return {
      destroy() {
        observer.disconnect();
      },
    };
  };

  /**
   * Get loading progress for a set of paths.
   * Returns 0-100 percentage.
   */
  getProgress(paths: string[]): number {
    if (paths.length === 0) return 100;

    const loaded = paths.filter((p) => this.isLoaded(p)).length;
    return Math.round((loaded / paths.length) * 100);
  }

  /**
   * Clear cached state for an asset.
   * Useful for forcing a reload.
   */
  clear(path: string): void {
    this.assets.delete(path);
  }

  /**
   * Clear all cached state.
   */
  clearAll(): void {
    this.assets.clear();
    this.preloadedCategories.clear();
    this._totalLoaded = 0;
    this._totalFailed = 0;
  }
}

/**
 * Singleton asset loader instance.
 */
export const assetLoader = new AssetLoader();

/**
 * Svelte action for lazy loading images.
 * Usage: <img use:lazyLoadImage={path} alt="..." />
 */
export function lazyLoadImage(
  node: HTMLImageElement,
  path: string
): { destroy: () => void; update: (newPath: string) => void } {
  let currentPath = path;

  const loadImage = (p: string) => {
    node.style.opacity = "0";
    node.style.transition = "opacity 0.3s ease-in-out";

    assetLoader
      .loadImage(p)
      .then(() => {
        node.src = p;
        node.style.opacity = "1";
      })
      .catch(() => {
        node.style.opacity = "0.3";
      });
  };

  // Create intersection observer
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          loadImage(currentPath);
          observer.unobserve(node);
        }
      });
    },
    {
      rootMargin: "100px",
      threshold: 0.1,
    }
  );

  observer.observe(node);

  return {
    update(newPath: string) {
      if (newPath !== currentPath) {
        currentPath = newPath;
        loadImage(newPath);
      }
    },
    destroy() {
      observer.disconnect();
    },
  };
}
