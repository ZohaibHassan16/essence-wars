// Replay state store using Svelte 5 runes

import type {
  SpectatorMatch,
  SpectatorAction,
  GameStateDto,
  ReplayInfo,
} from "$lib/api/types";
import * as api from "$lib/api/game";
import {
  triggerDamage,
  triggerHeal,
  triggerDeath,
  triggerSpawn,
  triggerAttack,
} from "$lib/animations/actions";

export type ReplayPhase = "idle" | "browser" | "loading" | "watching" | "finished" | "gameOver";

class ReplayStore {
  // Phase management
  phase = $state<ReplayPhase>("idle");

  // Replay list (for browser)
  replays = $state<ReplayInfo[]>([]);

  // Current replay data
  match = $state<SpectatorMatch | null>(null);
  currentReplayPath = $state<string | null>(null);

  // Playback state
  currentActionIndex = $state<number>(-1); // -1 = initial state
  isPlaying = $state<boolean>(false);
  playbackSpeed = $state<number>(1.0); // 0.25, 0.5, 1, 2, 4
  private playbackTimeout: ReturnType<typeof setTimeout> | null = null;

  // Loading/error
  isLoading = $state<boolean>(false);
  error = $state<string | null>(null);

  // ============================================================================
  // Computed Properties
  // ============================================================================

  /** Current game state to display */
  get currentState(): GameStateDto | null {
    if (!this.match) return null;
    if (this.currentActionIndex < 0) return this.match.initialState;
    return this.match.actions[this.currentActionIndex].stateAfter;
  }

  /** Current action (null if at initial state) */
  get currentAction(): SpectatorAction | null {
    if (!this.match || this.currentActionIndex < 0) return null;
    return this.match.actions[this.currentActionIndex];
  }

  /** Total number of actions in the match */
  get totalActions(): number {
    return this.match?.actions.length ?? 0;
  }

  /** Whether at the initial state (before any actions) */
  get isAtStart(): boolean {
    return this.currentActionIndex < 0;
  }

  /** Whether at the final action */
  get isAtEnd(): boolean {
    return this.currentActionIndex >= this.totalActions - 1;
  }

  /** Current turn number */
  get currentTurn(): number {
    if (!this.match) return 0;
    if (this.currentActionIndex < 0) return this.match.initialState.turn;
    return this.match.actions[this.currentActionIndex].turn;
  }

  // ============================================================================
  // Browser Actions
  // ============================================================================

  /** Load list of saved replays and show browser */
  async loadReplayList() {
    this.phase = "browser";
    this.isLoading = true;
    this.error = null;

    try {
      const replays = await api.listReplays();
      this.replays = replays;
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  /** Load and start playing a replay */
  async loadReplay(path: string) {
    this.phase = "loading";
    this.isLoading = true;
    this.error = null;
    this.match = null;
    this.currentActionIndex = -1;
    this.currentReplayPath = path;

    try {
      const match = await api.loadReplay(path);
      this.match = match;
      this.currentActionIndex = -1;
      this.phase = "watching";

      // Auto-start playback
      this.play();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      this.phase = "browser";
    } finally {
      this.isLoading = false;
    }
  }

  /** Delete a replay file */
  async deleteReplay(path: string) {
    try {
      await api.deleteReplay(path);
      // Refresh the list
      this.replays = this.replays.filter((r) => r.path !== path);
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  // ============================================================================
  // Playback Controls
  // ============================================================================

  /** Start/resume playback */
  play() {
    if (this.isAtEnd) {
      this.phase = "finished";
      return;
    }
    this.isPlaying = true;
    this.scheduleNextAction();
  }

  /** Pause playback */
  pause() {
    this.isPlaying = false;
    if (this.playbackTimeout) {
      clearTimeout(this.playbackTimeout);
      this.playbackTimeout = null;
    }
  }

  /** Schedule the next action in playback */
  private scheduleNextAction() {
    if (!this.isPlaying || this.isAtEnd) {
      this.isPlaying = false;
      if (this.isAtEnd) {
        this.phase = "finished";
      }
      return;
    }

    // Base delay between actions (ms)
    const baseDelay = 800;
    const delay = baseDelay / this.playbackSpeed;

    this.playbackTimeout = setTimeout(async () => {
      await this.stepForwardInternal(true);
      this.scheduleNextAction();
    }, delay);
  }

  /** Step forward one action (internal, with animation option) */
  private async stepForwardInternal(animate: boolean) {
    if (this.isAtEnd) return;

    this.currentActionIndex++;
    const action = this.currentAction;

    // Trigger animations if requested
    if (animate && action) {
      await this.processAnimations(action);
    }

    // Check if we reached the end
    if (this.isAtEnd) {
      this.phase = "finished";
    }
  }

  /** Step forward one action (user-initiated, always animates) */
  async stepForward() {
    if (this.isAtEnd) return;
    this.pause();
    await this.stepForwardInternal(true);
  }

  /** Step backward one action */
  stepBackward() {
    if (this.isAtStart) return;
    this.pause();
    this.currentActionIndex--;
    // Reset phase if we stepped back from finished
    if (this.phase === "finished") {
      this.phase = "watching";
    }
  }

  /** Jump to the initial state */
  jumpToStart() {
    this.pause();
    this.currentActionIndex = -1;
    if (this.phase === "finished") {
      this.phase = "watching";
    }
  }

  /** Jump to the final state */
  jumpToEnd() {
    this.pause();
    this.currentActionIndex = this.totalActions - 1;
    this.phase = "finished";
  }

  /** Jump to a specific action index */
  jumpToAction(index: number) {
    if (index < -1 || index >= this.totalActions) return;
    this.pause();
    this.currentActionIndex = index;
    if (index >= this.totalActions - 1) {
      this.phase = "finished";
    } else if (this.phase === "finished") {
      this.phase = "watching";
    }
  }

  /** Set playback speed */
  setSpeed(speed: number) {
    this.playbackSpeed = speed;
  }

  /** Go back to replay browser */
  backToBrowser() {
    this.pause();
    this.phase = "browser";
    this.match = null;
    this.currentActionIndex = -1;
    this.isPlaying = false;
    this.error = null;
    this.currentReplayPath = null;
  }

  /** Reset to idle state (go back to main menu) */
  reset() {
    this.pause();
    this.phase = "idle";
    this.match = null;
    this.replays = [];
    this.currentActionIndex = -1;
    this.isPlaying = false;
    this.error = null;
    this.currentReplayPath = null;
  }

  /** Show the game over screen with full results */
  showGameOver() {
    this.pause();
    this.phase = "gameOver";
  }

  /** Go back to watching from game over screen */
  backToWatching() {
    this.phase = "finished";
  }

  // ============================================================================
  // Animation Processing
  // ============================================================================

  /** Process animations for a spectator action */
  private async processAnimations(action: SpectatorAction): Promise<void> {
    for (const event of action.events) {
      const data = event.data as Record<string, unknown>;

      switch (event.eventType) {
        case "creature_spawned": {
          const player = data.player as number;
          const slot = data.slot as number;
          // In replay mode, player 1 is "player" side, player 2 is "opponent" side
          const elementId = `creature-${player === 1 ? "player" : "opponent"}-${slot}`;
          await new Promise((r) => setTimeout(r, 50));
          await triggerSpawn(elementId);
          break;
        }

        case "creature_died": {
          const player = data.player as number;
          const slot = data.slot as number;
          const elementId = `creature-${player === 1 ? "player" : "opponent"}-${slot}`;
          await triggerDeath(elementId);
          break;
        }

        case "life_changed": {
          // Could add life change animations here
          break;
        }
      }
    }

    // Handle attack animations based on the action
    if (
      action.action.actionType === "attack" &&
      action.action.sourceSlot !== undefined &&
      action.action.targetSlot !== undefined
    ) {
      // Determine which side is attacking
      const attackerSide = action.player === 1 ? "player" : "opponent";
      const defenderSide = action.player === 1 ? "opponent" : "player";

      const attackerId = `creature-${attackerSide}-${action.action.sourceSlot}`;
      const defenderId = `creature-${defenderSide}-${action.action.targetSlot}`;

      await triggerAttack(attackerId, defenderId, () => {
        triggerDamage(defenderId, 1);
      });
    }
  }
}

export const replayStore = new ReplayStore();
