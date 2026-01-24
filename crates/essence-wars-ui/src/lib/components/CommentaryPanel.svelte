<script lang="ts">
  import { spectatorStore } from '$lib/stores/spectatorState.svelte';

  let isExpanded = $state(true);

  const commentary = $derived(spectatorStore.currentCommentary);
  const history = $derived(spectatorStore.commentaryHistory.slice(-5).reverse());
  const enabled = $derived(spectatorStore.commentaryEnabled);

  function getAdvantageText(advantage: number | undefined): string {
    if (advantage === undefined) return '';
    if (advantage > 10) return 'P1 leads';
    if (advantage < -10) return 'P2 leads';
    return 'Even';
  }

  function getAdvantageClass(advantage: number | undefined): string {
    if (advantage === undefined) return 'text-ui-text-dim';
    if (advantage > 10) return 'text-health';
    if (advantage < -10) return 'text-damage';
    return 'text-ui-text-dim';
  }
</script>

{#if enabled}
  <div class="text-sm">
    <!-- Header with toggle -->
    <button
      class="flex items-center justify-between w-full text-left"
      onclick={() => isExpanded = !isExpanded}
    >
      <span class="text-ui-text font-semibold flex items-center gap-2">
        <span class="text-base">📊</span> Commentary
      </span>
      <span class="text-ui-text-dim text-xs">{isExpanded ? '▼' : '▶'}</span>
    </button>

    {#if isExpanded}
      <div class="mt-2 space-y-2">
        <!-- Current commentary (highlighted) -->
        {#if commentary}
          <div class="p-2 rounded bg-ui-action/20 border border-ui-action/50">
            <p class="text-ui-text text-xs leading-relaxed">{commentary.text}</p>
            <div class="flex items-center justify-between mt-1.5 text-xs">
              {#if commentary.winProbability !== undefined}
                <span class="text-ui-text-dim">
                  Win rate: <span class="text-ui-action font-medium">{Math.round(commentary.winProbability * 100)}%</span>
                </span>
              {/if}
              {#if commentary.boardAdvantage !== undefined}
                <span class={getAdvantageClass(commentary.boardAdvantage)}>
                  {getAdvantageText(commentary.boardAdvantage)}
                  ({commentary.boardAdvantage > 0 ? '+' : ''}{commentary.boardAdvantage})
                </span>
              {/if}
            </div>
          </div>
        {:else}
          <div class="p-2 rounded bg-gray-800/50 text-ui-text-dim text-xs italic">
            Waiting for first action...
          </div>
        {/if}

        <!-- History -->
        {#if history.length > 1}
          <div class="space-y-1 max-h-20 overflow-y-auto">
            {#each history.slice(1) as entry (entry.id)}
              <div class="text-xs text-ui-text-dim p-1.5 rounded bg-gray-800/50 leading-relaxed">
                <span class="text-ui-text-dim opacity-70">T{entry.turn}:</span> {entry.text}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/if}
