<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import { replayStore } from "$lib/stores/replayState.svelte";

  let isLoadingSpectator = $state(false);
  let isLoadingReplays = $state(false);

  async function startSpectatorMode() {
    isLoadingSpectator = true;
    await spectatorStore.loadDecksAndBots();
    isLoadingSpectator = false;
  }

  async function openReplayBrowser() {
    isLoadingReplays = true;
    await replayStore.loadReplayList();
    isLoadingReplays = false;
  }
</script>

<div class="min-h-screen flex flex-col items-center justify-center p-8">
  <div class="text-center">
    <h1 class="text-6xl font-bold text-ui-text mb-4">Essence Wars</h1>
    <p class="text-xl text-ui-text-dim mb-12">A Deterministic Card Game</p>

    <div class="flex flex-col gap-4 items-center">
      <button
        class="w-64 px-8 py-4 bg-ui-action text-white rounded-lg font-bold text-lg
               hover:bg-ui-action/80 transition-all hover:scale-105"
        onclick={() => gameStore.loadDecksAndBots()}
        disabled={gameStore.isLoading || isLoadingSpectator}
      >
        {#if gameStore.isLoading}
          Loading...
        {:else}
          Play vs AI
        {/if}
      </button>

      <button
        class="w-64 px-8 py-4 bg-ui-panel text-ui-text rounded-lg font-bold text-lg
               border border-gray-600 hover:border-ui-action hover:text-ui-action transition-all hover:scale-105"
        onclick={startSpectatorMode}
        disabled={gameStore.isLoading || isLoadingSpectator || isLoadingReplays}
      >
        {#if isLoadingSpectator}
          Loading...
        {:else}
          Watch AI vs AI
        {/if}
      </button>

      <button
        class="w-64 px-8 py-4 bg-ui-panel text-ui-text rounded-lg font-bold text-lg
               border border-gray-600 hover:border-ui-action hover:text-ui-action transition-all hover:scale-105"
        onclick={openReplayBrowser}
        disabled={gameStore.isLoading || isLoadingSpectator || isLoadingReplays}
      >
        {#if isLoadingReplays}
          Loading...
        {:else}
          Watch Replays
        {/if}
      </button>
    </div>

    {#if gameStore.error}
      <div class="mt-8 p-4 bg-damage/20 border border-damage rounded text-damage max-w-md">
        {gameStore.error}
      </div>
    {/if}

    <div class="mt-16 text-ui-text-dim text-sm">
      <p>v0.7.0 | Powered by Rust + Svelte + Tauri</p>
    </div>
  </div>
</div>
