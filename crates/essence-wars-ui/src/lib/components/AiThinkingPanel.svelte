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
    class="w-full px-3 py-2 flex items-center justify-between bg-ui-bg/50 hover:bg-ui-bg/70 transition-colors"
    onclick={() => isExpanded = !isExpanded}
  >
    <div class="flex items-center gap-2">
      <span class="text-sm">&#129302;</span>
      <span class="font-semibold text-sm text-ui-text">AI Thinking</span>
      {#if currentPlayer}
        <span class="text-xs px-1.5 py-0.5 rounded {currentPlayer === 1 ? 'bg-health/20 text-health' : 'bg-damage/20 text-damage'}">
          P{currentPlayer}
        </span>
      {/if}
    </div>
    <span class="text-ui-text-dim text-xs">{isExpanded ? '▲' : '▼'}</span>
  </button>

  {#if isExpanded}
    <div class="p-2">
      {#if spectatorStore.currentActionIndex < 0}
        <!-- Initial state - no action yet -->
        <div class="text-center text-ui-text-dim py-2 text-xs">
          No action selected. Press play to see AI decisions.
        </div>
      {:else if thinking}
        <!-- MCTS thinking data available -->
        <div class="space-y-2">
          <!-- Bot info -->
          <div class="flex items-center justify-between text-xs">
            <span class="text-ui-text">{botName}</span>
            <span class="text-ui-text-dim">{thinkingTimeMs}ms</span>
          </div>

          <!-- Simulations -->
          <div class="text-xs">
            <span class="text-ui-text-dim">Sims:</span>
            <span class="text-ui-text font-semibold ml-1">{thinking.totalSimulations.toLocaleString()}</span>
          </div>

          <!-- Top moves -->
          <div class="space-y-1">
            <div class="text-xs text-ui-text-dim">Top Moves:</div>
            {#each thinking.topMoves.slice(0, 3) as move, i (i)}
              {@const isSelected = i === 0}
              <div class="p-1.5 rounded {isSelected ? 'bg-ui-action/10 border border-ui-action/30' : 'bg-ui-bg/50'}">
                <div class="flex items-center justify-between mb-0.5">
                  <div class="flex items-center gap-1">
                    {#if isSelected}
                      <span class="text-ui-action text-xs">&#10003;</span>
                    {/if}
                    <span class="text-xs text-ui-text truncate max-w-[140px]">{move.action.description}</span>
                  </div>
                  <span class="text-xs font-semibold {move.winRate >= 0.5 ? 'text-health' : 'text-damage'}">
                    {formatWinRate(move.winRate)}
                  </span>
                </div>
                <!-- Visit bar -->
                <div class="h-1.5 bg-ui-bg rounded-full overflow-hidden">
                  <div
                    class="h-full {isSelected ? 'bg-ui-action' : 'bg-gray-500'} transition-all"
                    style="width: {formatVisits(move.visits, thinking.totalSimulations)}%"
                  ></div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <!-- No MCTS data (non-MCTS bot or data not captured) -->
        <div class="space-y-2">
          <!-- Bot info -->
          <div class="flex items-center justify-between text-xs">
            <span class="text-ui-text">{botName}</span>
            <span class="text-ui-text-dim">{thinkingTimeMs}ms</span>
          </div>

          <!-- Action taken -->
          {#if spectatorStore.currentAction}
            <div class="p-2 rounded bg-ui-bg/50">
              <div class="text-xs text-ui-text-dim mb-0.5">Action:</div>
              <div class="text-sm text-ui-text">{spectatorStore.currentAction.action.description}</div>
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
