<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";

  const state = $derived(gameStore.gameState);
  const didWin = $derived(state?.winner === 1);
</script>

<div class="min-h-screen flex flex-col items-center justify-center p-8 bg-ui-bg">
  <div class="text-center">
    <h1 class="text-6xl font-bold mb-4
               {didWin ? 'text-health' : state?.winner === 2 ? 'text-damage' : 'text-gold'}">
      {#if didWin}
        Victory!
      {:else if state?.winner === 2}
        Defeat
      {:else}
        Draw
      {/if}
    </h1>

    <div class="text-xl text-ui-text-dim mb-8">
      {state?.gameOverReason ?? "Game Over"}
    </div>

    <div class="bg-ui-panel rounded-xl p-6 mb-8 inline-block">
      <div class="grid grid-cols-2 gap-8 text-lg">
        <div>
          <div class="text-ui-text-dim mb-2">Your Final Life</div>
          <div class="text-3xl font-bold text-health">{state?.player.life ?? 0}</div>
        </div>
        <div>
          <div class="text-ui-text-dim mb-2">Opponent Final Life</div>
          <div class="text-3xl font-bold text-damage">{state?.opponent.life ?? 0}</div>
        </div>
      </div>
      <div class="mt-4 pt-4 border-t border-gray-700">
        <div class="text-ui-text-dim">Final Turn</div>
        <div class="text-2xl font-bold text-ui-text">{state?.turn ?? 0}</div>
      </div>
    </div>

    <div class="flex gap-4 justify-center">
      <button
        class="px-8 py-4 bg-ui-action text-white rounded-lg font-bold text-lg
               hover:bg-ui-action/80 transition-colors"
        onclick={() => {
          gameStore.quitGame();
          gameStore.loadDecksAndBots();
        }}
      >
        Play Again
      </button>
      <button
        class="px-8 py-4 bg-gray-600 text-white rounded-lg font-bold text-lg
               hover:bg-gray-500 transition-colors"
        onclick={() => gameStore.quitGame()}
      >
        Main Menu
      </button>
    </div>
  </div>
</div>
