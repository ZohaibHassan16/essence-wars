/**
 * Path utilities for handling base path in asset URLs.
 *
 * On GitHub Pages, the app is served from /essence-wars/
 * so all asset URLs need to be prefixed with the base path.
 */

import { base } from "$app/paths";

/**
 * Get the full URL for a static asset.
 * Prepends the base path for GitHub Pages compatibility.
 *
 * @param path - The asset path (e.g., "/sounds/battle/attack.ogg")
 * @returns The full path with base prefix (e.g., "/essence-wars/sounds/battle/attack.ogg")
 */
export function assetUrl(path: string): string {
  // Ensure path starts with /
  if (!path.startsWith("/")) {
    path = "/" + path;
  }
  return `${base}${path}`;
}

/**
 * Get the card art URL for a card ID.
 * Cards are stored in /cards/core_set/{id}.webp
 */
export function cardArtUrl(cardId: number): string {
  return assetUrl(`/cards/core_set/${cardId}.webp`);
}

/**
 * Get the commander portrait URL.
 * Portraits are stored in /portrait/{name}.webp
 */
export function portraitUrl(portraitPath: string): string {
  // portraitPath might be "portrait/commander_name.webp" or "/portrait/commander_name.webp"
  if (portraitPath.startsWith("/")) {
    return assetUrl(portraitPath);
  }
  return assetUrl(`/${portraitPath}`);
}

/**
 * Get the token art URL.
 */
export function tokenArtUrl(tokenPath: string): string {
  if (tokenPath.startsWith("/")) {
    return assetUrl(tokenPath);
  }
  return assetUrl(`/${tokenPath}`);
}
