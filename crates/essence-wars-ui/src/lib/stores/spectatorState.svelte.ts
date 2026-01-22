// Spectator state store using Svelte 5 runes

import type {
  SpectatorMatch,
  SpectatorAction,
  SpectatorConfig,
  GameStateDto,
  DeckInfo,
  BotInfo,
  GameEventDto,
} from "$lib/api/types";
import * as api from "$lib/api/game";
import {
  triggerDamage,
  triggerHeal,
  triggerDeath,
  triggerSpawn,
  triggerAttack,
} from "$lib/animations/actions";

export type SpectatorPhase = "setup" | "computing" | "watching" | "finished" | "gameOver";

class SpectatorStore {
  // Phase management
  phase = $state<SpectatorPhase>("setup");

  // Setup data (shared with gameStore, loaded once)
  decks = $state<DeckInfo[]>([]);
  bots = $state<BotInfo[]>([]);

  // Match data (after computation)
  match = $state<SpectatorMatch | null>(null);

  // Playback state
  currentActionIndex = $state<number>(-1); // -1 = initial state
  isPlaying = $state<boolean>(false);
  playbackSpeed = $state<number>(1.0); // 0.25, 0.5, 1, 2, 4
  private playbackTimeout: ReturnType<typeof setTimeout> | null = null;

  // "Watch Live" mode - hides timeline and result until end
  watchLive = $state<boolean>(false);

  // Loading/error
  isComputing = $state<boolean>(false);
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

  /** Whether timeline should be visible */
  get canShowTimeline(): boolean {
    // In watch live mode, only show timeline after reaching the end
    if (this.watchLive && this.phase !== "finished") return false;
    return true;
  }

  /** Whether result should be visible */
  get canShowResult(): boolean {
    // In watch live mode, only show result after reaching the end
    if (this.watchLive && this.phase !== "finished") return false;
    return true;
  }

  /** Current turn number */
  get currentTurn(): number {
    if (!this.match) return 0;
    if (this.currentActionIndex < 0) return this.match.initialState.turn;
    return this.match.actions[this.currentActionIndex].turn;
  }

  // ============================================================================
  // Actions
  // ============================================================================

  /** Load decks and bots for setup screen */
  async loadDecksAndBots() {
    this.error = null;
    try {
      const [decks, bots] = await Promise.all([
        api.listDecks(),
        api.listBots(),
      ]);
      this.decks = decks;
      this.bots = bots;
      this.phase = "setup";
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  /** Start computing a new spectator match */
  async startMatch(config: SpectatorConfig) {
    this.phase = "computing";
    this.isComputing = true;
    this.error = null;
    this.match = null;
    this.currentActionIndex = -1;

    try {
      const match = await api.computeSpectatorMatch(config);
      this.match = match;
      this.currentActionIndex = -1;
      this.phase = "watching";

      // Auto-start playback
      this.play();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      this.phase = "setup";
    } finally {
      this.isComputing = false;
    }
  }

  // ============================================================================
  // Playback Controls
  // ============================================================================

  /** Start/resume playback */
  play() {
    if (this.isAtEnd) {
      this.phase = "gameOver";
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
        this.phase = "gameOver";
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

    // Check if we reached the end - go to game over screen
    if (this.isAtEnd) {
      this.phase = "gameOver";
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
    // Reset phase if we stepped back from game over
    if (this.phase === "finished" || this.phase === "gameOver") {
      this.phase = "watching";
    }
  }

  /** Jump to the initial state */
  jumpToStart() {
    this.pause();
    this.currentActionIndex = -1;
    if (this.phase === "finished" || this.phase === "gameOver") {
      this.phase = "watching";
    }
  }

  /** Jump to the final state */
  jumpToEnd() {
    this.pause();
    this.currentActionIndex = this.totalActions - 1;
    this.phase = "gameOver";
  }

  /** Jump to a specific action index */
  jumpToAction(index: number) {
    if (index < -1 || index >= this.totalActions) return;
    this.pause();
    this.currentActionIndex = index;
    if (index >= this.totalActions - 1) {
      this.phase = "gameOver";
    } else if (this.phase === "finished" || this.phase === "gameOver") {
      this.phase = "watching";
    }
  }

  /** Set playback speed */
  setSpeed(speed: number) {
    this.playbackSpeed = speed;
  }

  /** Set watch live mode */
  setWatchLive(enabled: boolean) {
    this.watchLive = enabled;
  }

  /** Reset to setup screen */
  reset() {
    this.pause();
    this.phase = "setup";
    this.match = null;
    this.currentActionIndex = -1;
    this.isPlaying = false;
    this.error = null;
    this.watchLive = false;
  }

  /** Go back to main menu */
  backToMenu() {
    this.pause();
    this.phase = "setup";
    this.match = null;
    this.currentActionIndex = -1;
    this.isPlaying = false;
    this.error = null;
  }

  /** Show the game over screen with full results */
  showGameOver() {
    this.pause();
    this.phase = "gameOver";
  }

  /** Go back to watching from game over screen */
  backToWatching() {
    this.phase = "watching";
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
          // In spectator mode, player 1 is "player" side, player 2 is "opponent" side
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

export const spectatorStore = new SpectatorStore();
