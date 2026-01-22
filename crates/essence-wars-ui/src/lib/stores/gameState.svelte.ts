// Game state store using Svelte 5 runes

import type { GameStateDto, ActionInfo, DeckInfo, BotInfo } from "$lib/api/types";
import * as api from "$lib/api/game";

// Game state type for our reactive store
export type GamePhase = "menu" | "setup" | "playing" | "gameOver";

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
      await this.refreshLegalActions();
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

  async applyAction(actionIndex: number) {
    if (!this.gameId) return;
    this.isLoading = true;
    try {
      const update = await api.applyAction(this.gameId, actionIndex);
      this.gameState = update.state;
      this.clearSelection();

      if (update.state.isGameOver) {
        this.phase = "gameOver";
      } else {
        await this.refreshLegalActions();

        // If it's now opponent's turn, get AI move
        if (update.state.activePlayer === 2) {
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
        const update = await api.applyAction(this.gameId, aiAction.index);
        this.gameState = update.state;

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
