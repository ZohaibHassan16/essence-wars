<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import { replayStore } from "$lib/stores/replayState.svelte";
  import { mcpSyncStore } from "$lib/stores/mcpSyncState.svelte";
  import { tutorialStore } from "$lib/stores/tutorialState.svelte";
  import { deckBuilderStore } from "$lib/stores/deckBuilderState.svelte";
  import { startTutorialGame } from "$lib/tutorial/tutorialGame";
  import { playSound, playMusic } from "$lib/audio";
  import { assetUrl } from "$lib/utils/paths";

  let { onOpenSettings, onOpenRules, onOpenLore }: { onOpenSettings?: () => void; onOpenRules?: () => void; onOpenLore?: () => void } = $props();

  let isLoadingSpectator = $state(false);
  let isLoadingReplays = $state(false);
  let isLoadingTutorial = $state(false);
  let isLoadingDeckBuilder = $state(false);

  // Dynamically load all background images from the backgrounds folder
  // This uses Vite's import.meta.glob to automatically discover all .webp files
  const backgroundModules = import.meta.glob('/static/backgrounds/*.webp', { eager: true, query: '?url', import: 'default' });
  const menuBackgrounds = Object.values(backgroundModules) as string[];

  // Select random background on component creation (fallback to solid color if none available)
  const currentBackground = menuBackgrounds.length > 0
    ? menuBackgrounds[Math.floor(Math.random() * menuBackgrounds.length)]
    : null;

  // Play menu music when component mounts
  $effect(() => {
    playMusic('menu');
  });

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

  async function openDeckBuilder() {
    handleButtonClick();
    isLoadingDeckBuilder = true;
    try {
      await deckBuilderStore.open();
    } finally {
      isLoadingDeckBuilder = false;
    }
  }
</script>

<div
  class="min-h-screen bg-cover bg-center bg-no-repeat"
  style={currentBackground ? `background-image: url('${currentBackground}')` : 'background-color: #1A1A2E'}
>
  <!-- Dark overlay for readability -->
  <div class="min-h-screen flex flex-col items-center justify-center p-8 bg-black/60 backdrop-blur-[2px]">
    <div class="text-center">
      <!-- Banner image -->
      <img
        src={assetUrl("/ui/essence_wars_banner.webp")}
        alt="Essence Wars"
        class="h-32 md:h-40 lg:h-48 mx-auto mb-2 drop-shadow-2xl"
      />
      <p class="text-xl text-ui-text-dim mb-12 drop-shadow-lg">A Deterministic Card Game</p>

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

      <button
        class="w-64 px-8 py-4 bg-ui-panel text-ui-text rounded-lg font-bold text-lg
               border border-gray-600 hover:border-amber-500 hover:text-amber-500 transition-all hover:scale-105"
        onclick={openDeckBuilder}
        onmouseenter={handleButtonHover}
        disabled={gameStore.isLoading || isLoadingSpectator || isLoadingDeckBuilder}
      >
        {#if isLoadingDeckBuilder}
          Loading...
        {:else}
          Deck Builder
        {/if}
      </button>

      <button
        class="w-64 px-8 py-4 bg-ui-panel text-ui-text rounded-lg font-bold text-lg
               border border-gray-600 hover:border-mana hover:text-mana transition-all hover:scale-105"
        onclick={() => {
          handleButtonClick();
          onOpenRules?.();
        }}
        onmouseenter={handleButtonHover}
      >
        Rules & Guide
      </button>

      <button
        class="w-64 px-8 py-4 bg-ui-panel text-ui-text rounded-lg font-bold text-lg
               border border-gray-600 hover:border-gold hover:text-gold transition-all hover:scale-105"
        onclick={() => {
          handleButtonClick();
          onOpenLore?.();
        }}
        onmouseenter={handleButtonHover}
      >
        Lore & World
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

      <!-- MCP Sync View - only visible in development mode -->
      {#if import.meta.env.DEV}
        <button
          class="w-64 px-8 py-4 bg-ui-panel text-ui-text rounded-lg font-bold text-lg
                 border border-mana/50 hover:border-mana hover:text-mana transition-all hover:scale-105"
          onclick={startMcpSync}
          onmouseenter={handleButtonHover}
          disabled={gameStore.isLoading || isLoadingSpectator || isLoadingReplays}
        >
          MCP Sync View
        </button>
      {/if}
    </div>

    {#if gameStore.error}
      <div class="mt-8 p-4 bg-damage/20 border border-damage rounded text-damage max-w-md">
        {gameStore.error}
      </div>
    {/if}

    <div class="mt-16 text-ui-text-dim text-sm drop-shadow-lg">
      <p>v0.7.0 | Powered by Rust + Svelte + Tauri</p>
    </div>
  </div>
  </div>
</div>
