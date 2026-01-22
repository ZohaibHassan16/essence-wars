// Game state store using Svelte 5 runes

import type { GameStateDto, ActionInfo, DeckInfo, BotInfo, GameEventDto, AiHintResponse } from "$lib/api/types";
import * as api from "$lib/api/game";
import { triggerDamage, triggerHeal, triggerDeath, triggerSpawn, triggerAttack } from "$lib/animations/actions";

// Game state type for our reactive store
export type GamePhase = "menu" | "setup" | "playing" | "gameOver";

// Action with metadata for the log
export interface LoggedAction extends ActionInfo {
  player: 1 | 2;
  turn: number;
  timestamp: number;
}

class GameStore {
  // Core state
  phase = $state<GamePhase>("menu");
  gameState = $state<GameStateDto | null>(null);
  legalActions = $state<ActionInfo[]>([]);
  isLoading = $state(false);
  error = $state<string | null>(null);

  // Setup state
  decks = $state<DeckInfo[]>([]);
  bots = $state<BotInfo[]>([]);

  // Selection state
  selectedCardIndex = $state<number | null>(null);
  selectedCreatureSlot = $state<number | null>(null);
  highlightedSlots = $state<number[]>([]);

  // Action history
  actionHistory = $state<LoggedAction[]>([]);
  eventHistory = $state<GameEventDto[]>([]);

  // AI Hint state
  currentHint = $state<AiHintResponse | null>(null);
  isHintLoading = $state(false);

  // Computed
  get isPlayerTurn() {
    return this.gameState?.activePlayer === 1;
  }

  get gameId() {
    return this.gameState?.id ?? null;
  }

  // Actions
  async loadDecksAndBots() {
    this.isLoading = true;
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
    } finally {
      this.isLoading = false;
    }
  }

  async startGame(
    playerDeckId: string,
    opponentDeckId: string,
    opponentBotType: string,
    playerGoesFirst: boolean = true
  ) {
    this.isLoading = true;
    this.error = null;
    try {
      const state = await api.newGame({
        playerDeckId,
        opponentDeckId,
        opponentBotType,
        playerGoesFirst,
      });
      this.gameState = state;
      this.phase = "playing";
      this.actionHistory = [];
      this.eventHistory = [];
      await this.refreshLegalActions();

      // If AI goes first, do their turn
      if (!playerGoesFirst && state.activePlayer === 2) {
        await this.doAiTurn();
      }
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  async refreshLegalActions() {
    if (!this.gameId) return;
    try {
      this.legalActions = await api.getLegalActions(this.gameId);
    } catch (e) {
      console.error("Failed to get legal actions:", e);
    }
  }

  async applyAction(actionIndex: number, isPlayerAction: boolean = true) {
    if (!this.gameId) return;
    this.isLoading = true;
    try {
      const prevState = this.gameState;
      const update = await api.applyAction(this.gameId, actionIndex);
      this.gameState = update.state;
      this.clearSelection();

      // Log the action
      if (update.lastAction) {
        const loggedAction: LoggedAction = {
          ...update.lastAction,
          player: isPlayerAction ? 1 : 2,
          turn: prevState?.turn ?? 1,
          timestamp: Date.now(),
        };
        this.actionHistory = [...this.actionHistory, loggedAction];
      }

      // Log events
      if (update.events.length > 0) {
        this.eventHistory = [...this.eventHistory, ...update.events];
      }

      // Process animations for this action
      await this.processEventsForAnimations(update.events, update.lastAction);

      if (update.state.isGameOver) {
        this.phase = "gameOver";
      } else {
        await this.refreshLegalActions();

        // If it's now opponent's turn, get AI move
        if (update.state.activePlayer === 2 && isPlayerAction) {
          await this.doAiTurn();
        }
      }
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  async doAiTurn() {
    if (!this.gameId || this.gameState?.isGameOver) return;

    // Keep making AI moves until it's player's turn or game ends
    while (this.gameState && this.gameState.activePlayer === 2 && !this.gameState.isGameOver) {
      try {
        const aiAction = await api.getAiMove(this.gameId);

        // Apply action and log as AI action
        const prevState = this.gameState;
        const update = await api.applyAction(this.gameId, aiAction.index);
        this.gameState = update.state;

        // Log the AI action
        if (update.lastAction) {
          const loggedAction: LoggedAction = {
            ...update.lastAction,
            player: 2,
            turn: prevState?.turn ?? 1,
            timestamp: Date.now(),
          };
          this.actionHistory = [...this.actionHistory, loggedAction];
        }

        // Log events
        if (update.events.length > 0) {
          this.eventHistory = [...this.eventHistory, ...update.events];
        }

        // Process animations for AI actions
        await this.processEventsForAnimationsAi(update.events, update.lastAction);

        // Small delay between AI actions for visibility
        await new Promise(r => setTimeout(r, 300));
      } catch (e) {
        console.error("AI turn error:", e);
        break;
      }
    }

    if (this.gameState?.isGameOver) {
      this.phase = "gameOver";
    } else {
      await this.refreshLegalActions();
    }
  }

  selectCard(handIndex: number) {
    if (this.selectedCardIndex === handIndex) {
      this.clearSelection();
    } else {
      this.selectedCardIndex = handIndex;
      this.selectedCreatureSlot = null;
      this.updateHighlights();
    }
  }

  selectCreature(slot: number) {
    if (this.selectedCreatureSlot === slot) {
      this.clearSelection();
    } else {
      this.selectedCreatureSlot = slot;
      this.selectedCardIndex = null;
      this.updateHighlights();
    }
  }

  clearSelection() {
    this.selectedCardIndex = null;
    this.selectedCreatureSlot = null;
    this.highlightedSlots = [];
  }

  updateHighlights() {
    const slots: number[] = [];

    if (this.selectedCardIndex !== null) {
      // Find play_card actions for this hand index
      for (const action of this.legalActions) {
        if (action.actionType === "play_card" && action.handIndex === this.selectedCardIndex) {
          if (action.targetSlot !== undefined) {
            slots.push(action.targetSlot);
          }
        }
      }
    } else if (this.selectedCreatureSlot !== null) {
      // Find attack actions for this creature
      for (const action of this.legalActions) {
        if (action.actionType === "attack" && action.sourceSlot === this.selectedCreatureSlot) {
          if (action.targetSlot !== undefined) {
            slots.push(action.targetSlot);
          }
        }
      }
    }

    this.highlightedSlots = slots;
  }

  getActionForTarget(targetSlot: number): ActionInfo | null {
    if (this.selectedCardIndex !== null) {
      return this.legalActions.find(
        a => a.actionType === "play_card" &&
             a.handIndex === this.selectedCardIndex &&
             a.targetSlot === targetSlot
      ) ?? null;
    } else if (this.selectedCreatureSlot !== null) {
      return this.legalActions.find(
        a => a.actionType === "attack" &&
             a.sourceSlot === this.selectedCreatureSlot &&
             a.targetSlot === targetSlot
      ) ?? null;
    }
    return null;
  }

  async endTurn() {
    const endTurnAction = this.legalActions.find(a => a.actionType === "end_turn");
    if (endTurnAction) {
      await this.applyAction(endTurnAction.index);
    }
  }

  async requestHint() {
    if (!this.gameId || !this.isPlayerTurn) return;
    this.isHintLoading = true;
    try {
      this.currentHint = await api.getAiHint(this.gameId);
    } catch (e) {
      console.error("Failed to get AI hint:", e);
      this.currentHint = null;
    } finally {
      this.isHintLoading = false;
    }
  }

  async applyHint(action: ActionInfo) {
    this.currentHint = null;
    await this.applyAction(action.index);
  }

  clearHint() {
    this.currentHint = null;
  }

  async undoAction() {
    if (!this.gameId) return;
    this.isLoading = true;
    try {
      const newState = await api.undoAction(this.gameId);
      this.gameState = newState;
      this.currentHint = null;
      // Remove the last action from local history too
      if (this.actionHistory.length > 0) {
        this.actionHistory = this.actionHistory.slice(0, -1);
      }
      await this.refreshLegalActions();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  async checkCanUndo(): Promise<boolean> {
    if (!this.gameId) return false;
    try {
      return await api.canUndo(this.gameId);
    } catch {
      return false;
    }
  }

  /**
   * Process game events and trigger animations
   */
  async processEventsForAnimations(events: GameEventDto[], lastAction?: ActionInfo): Promise<void> {
    for (const event of events) {
      const data = event.data as Record<string, unknown>;

      switch (event.eventType) {
        case "creature_spawned": {
          const player = data.player as number;
          const slot = data.slot as number;
          const elementId = `creature-${player === 1 ? "player" : "opponent"}-${slot}`;
          // Small delay to let the DOM update
          await new Promise(r => setTimeout(r, 50));
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
          const player = data.player as number;
          const oldLife = data.old as number;
          const newLife = data.new as number;
          const diff = newLife - oldLife;

          // For now, we don't have a player avatar element to animate
          // This could be enhanced later
          break;
        }
      }
    }

    // Handle attack animations based on last action (player attacking)
    if (lastAction?.actionType === "attack" && lastAction.sourceSlot !== undefined && lastAction.targetSlot !== undefined) {
      const attackerId = `creature-player-${lastAction.sourceSlot}`;
      const defenderId = `creature-opponent-${lastAction.targetSlot}`;

      await triggerAttack(attackerId, defenderId, () => {
        triggerDamage(defenderId, 1);
      });
    }
  }

  /**
   * Process game events for AI actions (swapped sides)
   */
  async processEventsForAnimationsAi(events: GameEventDto[], lastAction?: ActionInfo): Promise<void> {
    for (const event of events) {
      const data = event.data as Record<string, unknown>;

      switch (event.eventType) {
        case "creature_spawned": {
          const player = data.player as number;
          const slot = data.slot as number;
          // AI is player 2, so their creatures are on "opponent" side visually
          const elementId = `creature-${player === 2 ? "opponent" : "player"}-${slot}`;
          await new Promise(r => setTimeout(r, 50));
          await triggerSpawn(elementId);
          break;
        }

        case "creature_died": {
          const player = data.player as number;
          const slot = data.slot as number;
          const elementId = `creature-${player === 2 ? "opponent" : "player"}-${slot}`;
          await triggerDeath(elementId);
          break;
        }
      }
    }

    // Handle attack animations for AI (opponent attacking player)
    if (lastAction?.actionType === "attack" && lastAction.sourceSlot !== undefined && lastAction.targetSlot !== undefined) {
      const attackerId = `creature-opponent-${lastAction.sourceSlot}`;
      const defenderId = `creature-player-${lastAction.targetSlot}`;

      await triggerAttack(attackerId, defenderId, () => {
        triggerDamage(defenderId, 1);
      });
    }
  }

  async quitGame() {
    if (this.gameId) {
      try {
        await api.endGame(this.gameId);
      } catch (e) {
        console.error("Failed to end game:", e);
      }
    }
    this.gameState = null;
    this.legalActions = [];
    this.clearSelection();
    this.phase = "menu";
  }
}

export const gameStore = new GameStore();
