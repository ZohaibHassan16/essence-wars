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
import {
  playSound,
  playCardSound,
  playFactionAttackSound,
  playFactionDeathSound,
  getFactionFromCardId,
  type Faction,
} from "$lib/audio";
import type { CommentaryEntry } from "$lib/commentary/types";
import { generateCommentaryForAction, resetCommentaryState } from "$lib/commentary/generator";

export type SpectatorPhase = "setup" | "computing" | "watching" | "finished" | "gameOver";

class SpectatorStore {
  // Phase management
  phase = $state<SpectatorPhase>("setup");

  // Setup data (shared with gameStore, loaded once)
  decks = $state<DeckInfo[]>([]);
  bots = $state<BotInfo[]>([]);

  // SFX mute (separate from global audio settings, spectator-specific)
  sfxMuted = $state<boolean>(false);

  // Match data (after computation)
  match = $state<SpectatorMatch | null>(null);

  // Playback state
  currentActionIndex = $state<number>(-1); // -1 = initial state
  isPlaying = $state<boolean>(false);
  playbackSpeed = $state<number>(1.0); // 0.25, 0.5, 1, 2, 4
  private playbackTimeout: ReturnType<typeof setTimeout> | null = null;

  // "Watch Live" mode - hides timeline and result until end
  watchLive = $state<boolean>(false);

  // Commentary state
  commentaryEnabled = $state<boolean>(false);
  showCommentaryOverlay = $state<boolean>(false);
  currentCommentary = $state<CommentaryEntry | null>(null);
  commentaryHistory = $state<CommentaryEntry[]>([]);

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
    // Bounds check: return last valid state if index out of bounds
    if (this.currentActionIndex >= this.match.actions.length) {
      return this.match.actions.length > 0
        ? this.match.actions[this.match.actions.length - 1].stateAfter
        : this.match.initialState;
    }
    return this.match.actions[this.currentActionIndex].stateAfter;
  }

  /** Current action (null if at initial state) */
  get currentAction(): SpectatorAction | null {
    if (!this.match || this.currentActionIndex < 0) return null;
    // Bounds check: return null if index out of bounds
    if (this.currentActionIndex >= this.match.actions.length) return null;
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

    // Capture previous state for commentary
    const prevState = this.currentState;

    this.currentActionIndex++;
    const action = this.currentAction;

    // Trigger animations if requested
    if (animate && action) {
      await this.processAnimations(action);
    }

    // Generate commentary for this action
    if (action) {
      this.generateCommentary(action, prevState);
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

  /** Toggle SFX mute */
  toggleSfxMute() {
    this.sfxMuted = !this.sfxMuted;
  }

  /** Set SFX mute state */
  setSfxMuted(muted: boolean) {
    this.sfxMuted = muted;
  }

  // ============================================================================
  // Commentary Controls
  // ============================================================================

  /** Enable/disable commentary */
  setCommentaryEnabled(enabled: boolean) {
    this.commentaryEnabled = enabled;
    if (!enabled) {
      this.showCommentaryOverlay = false;
    }
  }

  /** Generate commentary for an action */
  private generateCommentary(action: SpectatorAction, prevState: GameStateDto | null) {
    if (!this.commentaryEnabled) return;

    const entry = generateCommentaryForAction(action, prevState);
    this.currentCommentary = entry;
    this.commentaryHistory = [...this.commentaryHistory, entry];

    // Show overlay for key moments
    if (entry.isKeyMoment) {
      this.showCommentaryOverlay = true;
    }
  }

  /** Dismiss the commentary overlay */
  dismissCommentaryOverlay() {
    this.showCommentaryOverlay = false;
  }

  /** Reset commentary state */
  private resetCommentary() {
    this.currentCommentary = null;
    this.commentaryHistory = [];
    this.showCommentaryOverlay = false;
    resetCommentaryState();
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
    this.resetCommentary();
  }

  /** Clear current error */
  clearError() {
    this.error = null;
  }

  /** Go back to main menu */
  backToMenu() {
    this.pause();
    this.phase = "setup";
    this.match = null;
    this.currentActionIndex = -1;
    this.isPlaying = false;
    this.error = null;
    this.resetCommentary();
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
  // Animation & Sound Processing
  // ============================================================================

  /** Get faction from card ID in event data */
  private getFactionFromEvent(data: Record<string, unknown>): Faction {
    const cardId = (data.card_id as number) ?? (data.cardId as number);
    if (cardId) {
      return getFactionFromCardId(cardId);
    }
    return 'neutral';
  }

  /** Get creature faction from current state */
  private getCreatureFaction(slot: number, isPlayer1: boolean): Faction {
    const state = this.currentState;
    // In spectator mode: player = P1, opponent = P2
    const creatures = isPlayer1 ? state?.player.creatures : state?.opponent.creatures;
    const creature = creatures?.[slot];
    if (creature?.cardId) {
      return getFactionFromCardId(creature.cardId);
    }
    return 'neutral';
  }

  /** Process animations and sounds for a spectator action */
  private async processAnimations(action: SpectatorAction): Promise<void> {
    const playSfx = !this.sfxMuted;

    // Play card sound for play_card actions
    if (playSfx && action.action.actionType === "play_card") {
      const hasCreatureSpawn = action.events.some(e => e.eventType === "creature_spawned");
      const hasSupportPlayed = action.events.some(e => e.eventType === "support_played");
      const cardFaction = action.action.cardId ? getFactionFromCardId(action.action.cardId) : undefined;

      if (hasCreatureSpawn) {
        playCardSound('creature', cardFaction);
      } else if (hasSupportPlayed) {
        playCardSound('support');
      } else {
        playCardSound('spell');
      }
    }

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
          if (playSfx) {
            const faction = this.getFactionFromEvent(data);
            playFactionDeathSound(faction);
          }
          await triggerDeath(elementId);
          break;
        }

        case "life_changed": {
          const oldLife = data.old as number;
          const newLife = data.new as number;
          const diff = newLife - oldLife;
          if (playSfx) {
            if (diff > 0) {
              playSound('heal');
            } else if (diff < 0) {
              playSound('damage');
            }
          }
          break;
        }

        case "damage_dealt": {
          if (playSfx) {
            const amount = (data.amount as number) || 1;
            const attackerSlot = data.attacker_slot as number | undefined;
            if (attackerSlot !== undefined) {
              // Determine if attacker is P1 or P2
              const isPlayer1 = action.player === 1;
              const faction = this.getCreatureFaction(attackerSlot, isPlayer1);
              playFactionAttackSound(amount, faction);
            } else {
              playFactionAttackSound(amount, 'neutral');
            }
          }
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

      // Play attack sound if no damage_dealt event was processed
      if (playSfx && !action.events.some(e => e.eventType === "damage_dealt")) {
        const faction = this.getCreatureFaction(action.action.sourceSlot, action.player === 1);
        playFactionAttackSound(2, faction);
      }

      await triggerAttack(attackerId, defenderId, () => {
        triggerDamage(defenderId, 1);
      });
    }
  }
}

export const spectatorStore = new SpectatorStore();
