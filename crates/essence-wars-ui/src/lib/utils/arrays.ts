/**
 * Utility functions for array manipulation in game state handling.
 */

import type { CreatureDto, SupportDto } from "$lib/api/types";

/**
 * Ensure the creatures array has exactly 5 elements, padding with null if needed.
 * This handles cases where the backend might send fewer elements than expected.
 */
export function padCreatures(creatures: (CreatureDto | null)[] | undefined | null): (CreatureDto | null)[] {
  const result: (CreatureDto | null)[] = [null, null, null, null, null];
  if (creatures) {
    for (let i = 0; i < Math.min(creatures.length, 5); i++) {
      result[i] = creatures[i];
    }
  }
  return result;
}

/**
 * Ensure the supports array has exactly 2 elements, padding with null if needed.
 * This handles cases where the backend might send fewer elements than expected.
 */
export function padSupports(supports: (SupportDto | null)[] | undefined | null): (SupportDto | null)[] {
  const result: (SupportDto | null)[] = [null, null];
  if (supports) {
    for (let i = 0; i < Math.min(supports.length, 2); i++) {
      result[i] = supports[i];
    }
  }
  return result;
}
