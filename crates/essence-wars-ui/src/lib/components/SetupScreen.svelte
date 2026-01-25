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
  <div class="fixed top-4 left-1/2 -translate-x-1/2 z-50 max-w-lg">
    <div class="p-4 bg-damage/90 border border-damage rounded-lg text-white shadow-xl">
      <div class="flex items-start gap-3">
        <div class="flex-1">
          <p class="font-semibold">Error</p>
          <p class="text-sm mt-1">{gameStore.error}</p>
        </div>
        <button
          class="px-3 py-1 bg-white/20 hover:bg-white/30 rounded text-sm font-medium transition-colors"
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
