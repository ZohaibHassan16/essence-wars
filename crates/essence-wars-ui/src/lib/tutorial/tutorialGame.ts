// Tutorial game initialization logic
import { gameStore } from '$lib/stores/gameState.svelte';
import { tutorialStore } from '$lib/stores/tutorialState.svelte';
import {
  tutorialSteps,
  TUTORIAL_SEED,
  TUTORIAL_PLAYER_DECK,
  TUTORIAL_OPPONENT_DECK,
  TUTORIAL_BOT
} from './tutorialScript';

/**
 * Start a new tutorial game with fixed configuration.
 * Uses a known seed for reproducible gameplay.
 */
export async function startTutorialGame(): Promise<void> {
  // Load decks and bots if not already loaded
  if (gameStore.decks.length === 0) {
    await gameStore.loadDecksAndBots();
  }

  // Start game with tutorial configuration
  await gameStore.startGame(
    TUTORIAL_PLAYER_DECK,
    TUTORIAL_OPPONENT_DECK,
    TUTORIAL_BOT,
    true, // Player goes first
    TUTORIAL_SEED
  );

  // Initialize tutorial overlay
  tutorialStore.startTutorial(tutorialSteps);
}

/**
 * End the current tutorial and return to menu.
 */
export function endTutorial(): void {
  tutorialStore.endTutorial();
  gameStore.quitGame();
}
