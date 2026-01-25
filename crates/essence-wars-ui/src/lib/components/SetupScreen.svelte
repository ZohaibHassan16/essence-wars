<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";
  import DeckSelectionWizard from "./menu/DeckSelectionWizard.svelte";
  import type { WizardConfig } from "./menu/DeckSelectionWizard.svelte";

  async function handleStart(config: WizardConfig) {
    await gameStore.startGame(
      config.playerDeckId,
      config.opponentDeckId,
      config.opponentBotType,
      config.playerGoesFirst
    );
  }

  function handleBack() {
    gameStore.phase = "menu";
  }
</script>

<!-- Error banner -->
{#if gameStore.error}
  <div class="fixed top-4 left-1/2 -translate-x-1/2 z-50 max-w-lg animate-slide-down">
    <div class="p-4 bg-damage/90 border border-damage rounded-lg text-white shadow-xl backdrop-blur-sm">
      <div class="flex items-start gap-3">
        <div class="text-damage-light">
          <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="flex-1">
          <p class="font-semibold">Error</p>
          <p class="text-sm mt-1 opacity-90">{gameStore.error}</p>
        </div>
        <button
          class="px-3 py-1 bg-white/20 hover:bg-white/30 rounded text-sm font-medium transition-colors
                 focus:outline-none focus:ring-2 focus:ring-white/50"
          onclick={() => gameStore.clearError()}
        >
          Dismiss
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Loading overlay -->
{#if gameStore.isLoading}
  <div class="fixed inset-0 z-40 bg-black/50 flex items-center justify-center">
    <div class="bg-ui-panel rounded-xl p-8 shadow-2xl flex flex-col items-center gap-4">
      <div class="w-12 h-12 border-4 border-ui-action border-t-transparent rounded-full animate-spin"></div>
      <p class="text-ui-text font-semibold">Starting game...</p>
    </div>
  </div>
{/if}

<div class="w-full h-full">
  <DeckSelectionWizard
    mode="human-vs-ai"
    decks={gameStore.decks}
    bots={gameStore.bots}
    onStart={handleStart}
    onBack={handleBack}
  />
</div>

<style>
  @keyframes slide-down {
    from {
      opacity: 0;
      transform: translate(-50%, -100%);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0);
    }
  }

  :global(.animate-slide-down) {
    animation: slide-down 0.3s ease-out;
  }
</style>
