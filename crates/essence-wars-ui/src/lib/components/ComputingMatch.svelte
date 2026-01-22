<script lang="ts">
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";

  function cancel() {
    spectatorStore.reset();
  }
</script>

<div class="min-h-screen flex flex-col items-center justify-center p-8 bg-ui-bg">
  <div class="text-center max-w-md">
    <!-- Spinning gear animation -->
    <div class="mb-8">
      <div class="inline-block animate-spin text-6xl text-ui-action">
        &#9881;
      </div>
    </div>

    <h2 class="text-3xl font-bold text-ui-text mb-4">Computing Match</h2>

    <p class="text-ui-text-dim mb-6">
      The AI bots are playing their game. This may take a moment for MCTS bots...
    </p>

    <!-- Bot matchup display -->
    <div class="bg-ui-panel rounded-lg p-4 mb-8">
      <div class="flex items-center justify-center gap-4">
        <div class="text-center">
          <div class="w-12 h-12 rounded-full bg-health/20 text-health flex items-center justify-center text-lg font-bold mx-auto mb-2">
            P1
          </div>
          <div class="text-sm text-ui-text-dim">Bot 1</div>
        </div>

        <div class="text-2xl text-ui-text-dim">vs</div>

        <div class="text-center">
          <div class="w-12 h-12 rounded-full bg-damage/20 text-damage flex items-center justify-center text-lg font-bold mx-auto mb-2">
            P2
          </div>
          <div class="text-sm text-ui-text-dim">Bot 2</div>
        </div>
      </div>
    </div>

    <!-- Progress dots animation -->
    <div class="flex justify-center gap-2 mb-8">
      <div class="w-2 h-2 rounded-full bg-ui-action animate-pulse" style="animation-delay: 0ms;"></div>
      <div class="w-2 h-2 rounded-full bg-ui-action animate-pulse" style="animation-delay: 150ms;"></div>
      <div class="w-2 h-2 rounded-full bg-ui-action animate-pulse" style="animation-delay: 300ms;"></div>
    </div>

    <button
      class="px-6 py-3 bg-ui-panel text-ui-text-dim rounded-lg font-semibold
             border border-gray-600 hover:border-damage hover:text-damage transition-colors"
      onclick={cancel}
    >
      Cancel
    </button>
  </div>
</div>

<style>
  @keyframes pulse {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 1; }
  }
</style>
