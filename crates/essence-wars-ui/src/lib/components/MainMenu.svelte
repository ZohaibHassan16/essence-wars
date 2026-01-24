<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import { replayStore } from "$lib/stores/replayState.svelte";
  import { mcpSyncStore } from "$lib/stores/mcpSyncState.svelte";
  import { tutorialStore } from "$lib/stores/tutorialState.svelte";
  import { startTutorialGame } from "$lib/tutorial/tutorialGame";
  import { playSound } from "$lib/audio";

  let { onOpenSettings }: { onOpenSettings?: () => void } = $props();

  let isLoadingSpectator = $state(false);
  let isLoadingReplays = $state(false);
  let isLoadingTutorial = $state(false);

  function handleButtonHover() {
    playSound('buttonHover');
  }

  function handleButtonClick() {
    playSound('buttonClick');
  }

  async function startSpectatorMode() {
    handleButtonClick();
    isLoadingSpectator = true;
    await spectatorStore.loadDecksAndBots();
    isLoadingSpectator = false;
  }

  async function openReplayBrowser() {
    handleButtonClick();
    isLoadingReplays = true;
    await replayStore.loadReplayList();
    isLoadingReplays = false;
  }

  function startMcpSync() {
    handleButtonClick();
    mcpSyncStore.startWatching();
  }

  function startGame() {
    handleButtonClick();
    gameStore.loadDecksAndBots();
  }

  async function handleStartTutorial() {
    handleButtonClick();
    isLoadingTutorial = true;
    try {
      await startTutorialGame();
    } finally {
      isLoadingTutorial = false;
    }
  }
</script>

<div class="min-h-screen flex flex-col items-center justify-center p-8">
  <div class="text-center">
    <h1 class="text-6xl font-bold text-ui-text mb-4">Essence Wars</h1>
    <p class="text-xl text-ui-text-dim mb-12">A Deterministic Card Game</p>

    <div class="flex flex-col gap-4 items-center">
      <!-- Tutorial button - first for new players -->
      <button
        class="w-64 px-8 py-4 bg-health/20 text-health rounded-lg font-bold text-lg
               border border-health/50 hover:border-health hover:bg-health/30 transition-all hover:scale-105"
        onclick={handleStartTutorial}
        onmouseenter={handleButtonHover}
        disabled={gameStore.isLoading || isLoadingSpectator || isLoadingTutorial}
      >
        {#if isLoadingTutorial}
          Loading...
        {:else if tutorialStore.hasCompletedTutorial}
          Replay Tutorial
        {:else}
          Learn to Play
        {/if}
      </button>

      <button
        class="w-64 px-8 py-4 bg-ui-action text-white rounded-lg font-bold text-lg
               hover:bg-ui-action/80 transition-all hover:scale-105"
        onclick={startGame}
        onmouseenter={handleButtonHover}
        disabled={gameStore.isLoading || isLoadingSpectator || isLoadingTutorial}
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
        onmouseenter={handleButtonHover}
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
        onmouseenter={handleButtonHover}
        disabled={gameStore.isLoading || isLoadingSpectator || isLoadingReplays}
      >
        {#if isLoadingReplays}
          Loading...
        {:else}
          Watch Replays
        {/if}
      </button>

      <div class="h-px w-48 bg-gray-700 my-2"></div>

      <button
        class="w-64 px-8 py-4 bg-ui-panel text-ui-text rounded-lg font-bold text-lg
               border border-gray-600 hover:border-gray-500 hover:text-ui-text transition-all hover:scale-105"
        onclick={() => {
          handleButtonClick();
          onOpenSettings?.();
        }}
        onmouseenter={handleButtonHover}
      >
        Settings
      </button>

      <button
        class="w-64 px-8 py-4 bg-ui-panel text-ui-text rounded-lg font-bold text-lg
               border border-mana/50 hover:border-mana hover:text-mana transition-all hover:scale-105"
        onclick={startMcpSync}
        onmouseenter={handleButtonHover}
        disabled={gameStore.isLoading || isLoadingSpectator || isLoadingReplays}
      >
        MCP Sync View
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
