// Match Comparison state store using Svelte 5 runes

import type { SpectatorMatch, ReplayInfo } from "$lib/api/types";
import type { MatchStatistics } from "$lib/stats/types";
import { computeMatchStatistics } from "$lib/stats/statsComputer";
import * as api from "$lib/api/game";

export type ComparisonPhase = "idle" | "selecting" | "comparing";

export interface ComparisonSlot {
  match: SpectatorMatch | null;
  statistics: MatchStatistics | null;
  source: "spectator" | "replay" | null;
  label: string;
}

class ComparisonStore {
  // Phase management
  phase = $state<ComparisonPhase>("idle");

  // Slot 1 (left side)
  slot1 = $state<ComparisonSlot>({
    match: null,
    statistics: null,
    source: null,
    label: "Match 1",
  });

  // Slot 2 (right side)
  slot2 = $state<ComparisonSlot>({
    match: null,
    statistics: null,
    source: null,
    label: "Match 2",
  });

  // Which slot is being selected (for replay browser integration)
  selectingSlot = $state<1 | 2 | null>(null);

  // Available replays for selection
  replays = $state<ReplayInfo[]>([]);
  isLoadingReplays = $state<boolean>(false);

  // Loading state
  isLoading = $state<boolean>(false);
  error = $state<string | null>(null);

  // ============================================================================
  // Computed Properties
  // ============================================================================

  /** Whether both slots have loaded matches */
  get canCompare(): boolean {
    return this.slot1.match !== null && this.slot2.match !== null;
  }

  /** Whether slot 1 has a match */
  get hasSlot1(): boolean {
    return this.slot1.match !== null;
  }

  /** Whether slot 2 has a match */
  get hasSlot2(): boolean {
    return this.slot2.match !== null;
  }

  // ============================================================================
  // Actions
  // ============================================================================

  /** Start comparison mode - show selection screen */
  async startComparison() {
    this.phase = "selecting";
    this.error = null;
    await this.loadReplayList();
  }

  /** Load list of available replays */
  async loadReplayList() {
    this.isLoadingReplays = true;
    try {
      this.replays = await api.listReplays();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoadingReplays = false;
    }
  }

  /** Load a match from a spectator match (current or cached) */
  loadFromSpectatorMatch(slot: 1 | 2, match: SpectatorMatch, label?: string) {
    const statistics = computeMatchStatistics(match);
    const slotData: ComparisonSlot = {
      match,
      statistics,
      source: "spectator",
      label: label ?? `${match.player1DeckName} vs ${match.player2DeckName}`,
    };

    if (slot === 1) {
      this.slot1 = slotData;
    } else {
      this.slot2 = slotData;
    }

    this.selectingSlot = null;

    // If both slots are loaded, go to comparison view
    if (this.canCompare) {
      this.phase = "comparing";
    }
  }

  /** Load a match from a replay file */
  async loadFromReplay(slot: 1 | 2, path: string) {
    this.isLoading = true;
    this.error = null;

    try {
      const match = await api.loadReplay(path);
      const statistics = computeMatchStatistics(match);
      const slotData: ComparisonSlot = {
        match,
        statistics,
        source: "replay",
        label: `${match.player1DeckName} vs ${match.player2DeckName}`,
      };

      if (slot === 1) {
        this.slot1 = slotData;
      } else {
        this.slot2 = slotData;
      }

      this.selectingSlot = null;

      // If both slots are loaded, go to comparison view
      if (this.canCompare) {
        this.phase = "comparing";
      }
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  /** Start selecting a replay for a slot */
  selectSlot(slot: 1 | 2) {
    this.selectingSlot = slot;
  }

  /** Cancel slot selection */
  cancelSelection() {
    this.selectingSlot = null;
  }

  /** Clear a slot */
  clearSlot(slot: 1 | 2) {
    if (slot === 1) {
      this.slot1 = {
        match: null,
        statistics: null,
        source: null,
        label: "Match 1",
      };
    } else {
      this.slot2 = {
        match: null,
        statistics: null,
        source: null,
        label: "Match 2",
      };
    }

    // If we were comparing, go back to selecting
    if (this.phase === "comparing") {
      this.phase = "selecting";
    }
  }

  /** Swap the two slots */
  swapSlots() {
    const temp = this.slot1;
    this.slot1 = this.slot2;
    this.slot2 = temp;
  }

  /** Go back to selection screen from comparison view */
  backToSelection() {
    this.phase = "selecting";
  }

  /** Reset and exit comparison mode */
  reset() {
    this.phase = "idle";
    this.slot1 = {
      match: null,
      statistics: null,
      source: null,
      label: "Match 1",
    };
    this.slot2 = {
      match: null,
      statistics: null,
      source: null,
      label: "Match 2",
    };
    this.selectingSlot = null;
    this.replays = [];
    this.isLoading = false;
    this.isLoadingReplays = false;
    this.error = null;
  }

  /** Clear error */
  clearError() {
    this.error = null;
  }
}

export const comparisonStore = new ComparisonStore();
