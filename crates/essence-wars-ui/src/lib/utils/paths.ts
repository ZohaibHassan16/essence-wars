/**
 * Path utilities for asset URLs.
 *
 * For the Tauri desktop app, assets are served from the static directory
 * with no base path prefix needed.
 */

/**
 * Get the full URL for a static asset.
 *
 * @param path - The asset path (e.g., "/sounds/battle/attack.ogg")
 * @returns The full path (same as input for Tauri)
 */
export function assetUrl(path: string): string {
  // Ensure path starts with /
  if (!path.startsWith("/")) {
    path = "/" + path;
  }
  return path;
}

/**
 * Get the card art URL for a card ID.
 * Cards are stored in /cards/core_set/{id}.webp
 */
export function cardArtUrl(cardId: number): string {
  return `/cards/core_set/${cardId}.webp`;
}

/**
 * Get the commander portrait URL.
 * Portraits are stored in /portrait/{name}.webp
 */
export function portraitUrl(portraitPath: string): string {
  // portraitPath might be "portrait/commander_name.webp" or "/portrait/commander_name.webp"
  if (portraitPath.startsWith("/")) {
    return portraitPath;
  }
  return `/${portraitPath}`;
}

/**
 * Get the token art URL.
 */
export function tokenArtUrl(tokenPath: string): string {
  if (tokenPath.startsWith("/")) {
    return tokenPath;
  }
  return `/${tokenPath}`;
}
