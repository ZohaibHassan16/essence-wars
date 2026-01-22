<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";

  const state = $derived(gameStore.gameState);
  const didWin = $derived(state?.winner === 1);
  const isDraw = $derived(state?.winner === null || state?.winner === undefined);

  function formatReason(reason: string | undefined): string {
    if (!reason) return "Game Over";
    // Convert snake_case or CamelCase to readable text
    return reason
      .replace(/([A-Z])/g, " $1")
      .replace(/_/g, " ")
      .replace(/^\s+/, "")
      .toLowerCase()
      .replace(/^./, (c) => c.toUpperCase());
  }
</script>

<div class="min-h-screen flex flex-col items-center justify-center p-8 bg-ui-bg overflow-hidden">
  <!-- Animated background glow -->
  <div class="absolute inset-0 overflow-hidden pointer-events-none">
    <div
      class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] rounded-full blur-[120px] animate-pulse opacity-30
             {didWin ? 'bg-health' : isDraw ? 'bg-gold' : 'bg-damage'}"
    ></div>
  </div>

  <div class="text-center relative z-10 animate-fade-in">
    <!-- Large icon -->
    <div class="text-8xl mb-6 animate-bounce-slow">
      {#if didWin}
        🏆
      {:else if isDraw}
        🤝
      {:else}
        💀
      {/if}
    </div>

    <h1
      class="text-7xl font-black mb-2 tracking-tight
             {didWin ? 'text-health' : isDraw ? 'text-gold' : 'text-damage'}"
    >
      {#if didWin}
        VICTORY
      {:else if isDraw}
        DRAW
      {:else}
        DEFEAT
      {/if}
    </h1>

    <div class="text-xl text-ui-text-dim mb-10">
      {formatReason(state?.gameOverReason)}
    </div>

    <!-- Stats card -->
    <div class="bg-ui-panel/80 backdrop-blur-sm rounded-2xl p-8 mb-10 inline-block border border-gray-700 shadow-2xl">
      <div class="grid grid-cols-3 gap-10 text-lg">
        <div class="text-center">
          <div class="text-ui-text-dim text-sm uppercase tracking-wider mb-2">You</div>
          <div class="text-4xl font-bold {state?.player.life && state.player.life > 0 ? 'text-health' : 'text-damage'}">
            {state?.player.life ?? 0}
          </div>
          <div class="text-ui-text-dim text-xs mt-1">Life</div>
        </div>
        <div class="text-center border-x border-gray-700 px-8">
          <div class="text-ui-text-dim text-sm uppercase tracking-wider mb-2">Turn</div>
          <div class="text-4xl font-bold text-ui-text">{state?.turn ?? 0}</div>
          <div class="text-ui-text-dim text-xs mt-1">Final</div>
        </div>
        <div class="text-center">
          <div class="text-ui-text-dim text-sm uppercase tracking-wider mb-2">Opponent</div>
          <div class="text-4xl font-bold {state?.opponent.life && state.opponent.life > 0 ? 'text-health' : 'text-damage'}">
            {state?.opponent.life ?? 0}
          </div>
          <div class="text-ui-text-dim text-xs mt-1">Life</div>
        </div>
      </div>
    </div>

    <!-- Action buttons -->
    <div class="flex gap-6 justify-center">
      <button
        class="group px-10 py-4 bg-gradient-to-r from-ui-action to-purple-600 text-white rounded-xl font-bold text-lg
               hover:from-ui-action/90 hover:to-purple-500 transition-all transform hover:scale-105 hover:shadow-lg hover:shadow-ui-action/30"
        onclick={() => {
          gameStore.quitGame();
          gameStore.loadDecksAndBots();
        }}
      >
        <span class="flex items-center gap-2">
          <span>Play Again</span>
          <span class="group-hover:translate-x-1 transition-transform">→</span>
        </span>
      </button>
      <button
        class="px-10 py-4 bg-gray-700 text-white rounded-xl font-bold text-lg
               hover:bg-gray-600 transition-all transform hover:scale-105"
        onclick={() => gameStore.quitGame()}
      >
        Main Menu
      </button>
    </div>
  </div>
</div>

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @keyframes bounce-slow {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-10px);
    }
  }

  .animate-fade-in {
    animation: fade-in 0.6s ease-out forwards;
  }

  .animate-bounce-slow {
    animation: bounce-slow 2s ease-in-out infinite;
  }
</style>
