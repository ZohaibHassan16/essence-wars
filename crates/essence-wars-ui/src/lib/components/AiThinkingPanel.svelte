<script lang="ts">
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";

  let isExpanded = $state(true);

  // Get current action's thinking data
  const thinking = $derived(spectatorStore.currentAction?.thinking);
  const thinkingTimeMs = $derived(spectatorStore.currentAction?.thinkingTimeMs ?? 0);
  const currentPlayer = $derived(spectatorStore.currentAction?.player);
  const botName = $derived(
    currentPlayer === 1
      ? spectatorStore.match?.player1BotName
      : spectatorStore.match?.player2BotName
  );

  function formatWinRate(rate: number): string {
    return `${(rate * 100).toFixed(0)}%`;
  }

  function formatVisits(visits: number, total: number): number {
    return total > 0 ? (visits / total) * 100 : 0;
  }
</script>

<div class="bg-ui-panel rounded-lg shadow-lg overflow-hidden">
  <!-- Header -->
  <button
    class="w-full px-4 py-3 flex items-center justify-between bg-ui-bg/50 hover:bg-ui-bg/70 transition-colors"
    onclick={() => isExpanded = !isExpanded}
  >
    <div class="flex items-center gap-2">
      <span class="text-lg">&#129302;</span>
      <span class="font-semibold text-ui-text">AI Thinking</span>
      {#if currentPlayer}
        <span class="text-sm px-2 py-0.5 rounded {currentPlayer === 1 ? 'bg-health/20 text-health' : 'bg-damage/20 text-damage'}">
          P{currentPlayer}
        </span>
      {/if}
    </div>
    <span class="text-ui-text-dim">{isExpanded ? '▲' : '▼'}</span>
  </button>

  {#if isExpanded}
    <div class="p-4">
      {#if spectatorStore.currentActionIndex < 0}
        <!-- Initial state - no action yet -->
        <div class="text-center text-ui-text-dim py-4">
          No action selected. Press play or step forward to see AI decisions.
        </div>
      {:else if thinking}
        <!-- MCTS thinking data available -->
        <div class="space-y-4">
          <!-- Bot info -->
          <div class="flex items-center justify-between text-sm">
            <span class="text-ui-text">{botName}</span>
            <span class="text-ui-text-dim">{thinkingTimeMs}ms</span>
          </div>

          <!-- Simulations -->
          <div class="text-sm">
            <span class="text-ui-text-dim">Simulations:</span>
            <span class="text-ui-text font-semibold ml-1">{thinking.totalSimulations.toLocaleString()}</span>
          </div>

          <!-- Top moves -->
          <div class="space-y-2">
            <div class="text-sm text-ui-text-dim">Top Moves:</div>
            {#each thinking.topMoves as move, i}
              {@const isSelected = i === 0}
              <div class="p-2 rounded {isSelected ? 'bg-ui-action/10 border border-ui-action/30' : 'bg-ui-bg/50'}">
                <div class="flex items-center justify-between mb-1">
                  <div class="flex items-center gap-2">
                    {#if isSelected}
                      <span class="text-ui-action">&#10003;</span>
                    {/if}
                    <span class="text-sm text-ui-text">{move.action.description}</span>
                  </div>
                  <span class="text-sm font-semibold {move.winRate >= 0.5 ? 'text-health' : 'text-damage'}">
                    {formatWinRate(move.winRate)}
                  </span>
                </div>
                <!-- Visit bar -->
                <div class="h-2 bg-ui-bg rounded-full overflow-hidden">
                  <div
                    class="h-full {isSelected ? 'bg-ui-action' : 'bg-gray-500'} transition-all"
                    style="width: {formatVisits(move.visits, thinking.totalSimulations)}%"
                  ></div>
                </div>
                <div class="text-xs text-ui-text-dim mt-1">
                  {move.visits.toLocaleString()} visits
                </div>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <!-- No MCTS data (non-MCTS bot or data not captured) -->
        <div class="space-y-3">
          <!-- Bot info -->
          <div class="flex items-center justify-between text-sm">
            <span class="text-ui-text">{botName}</span>
            <span class="text-ui-text-dim">{thinkingTimeMs}ms</span>
          </div>

          <!-- Action taken -->
          {#if spectatorStore.currentAction}
            <div class="p-3 rounded bg-ui-bg/50">
              <div class="text-sm text-ui-text-dim mb-1">Action:</div>
              <div class="text-ui-text">{spectatorStore.currentAction.action.description}</div>
            </div>
          {/if}

          <div class="text-xs text-ui-text-dim text-center">
            Detailed thinking data not available for this bot type.
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
