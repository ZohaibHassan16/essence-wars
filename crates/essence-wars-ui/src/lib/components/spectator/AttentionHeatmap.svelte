<script lang="ts">
  import type { AttentionWeightsDto } from "$lib/api/types";

  let {
    attention = null,
    selectedLayer = 0,
    selectedHead = 0,
  }: {
    attention: AttentionWeightsDto[] | null;
    selectedLayer?: number;
    selectedHead?: number;
  } = $props();

  // Get unique layers and heads from attention data
  const layers = $derived(() => {
    if (!attention) return [];
    return [...new Set(attention.map((a) => a.layer))].sort((a, b) => a - b);
  });

  const heads = $derived(() => {
    if (!attention) return [];
    return [...new Set(attention.filter((a) => a.layer === selectedLayer).map((a) => a.head))].sort(
      (a, b) => a - b
    );
  });

  // Get current attention weights
  const currentWeights = $derived(() => {
    if (!attention) return null;
    return attention.find((a) => a.layer === selectedLayer && a.head === selectedHead);
  });

  // Color scale for attention weights (white to red)
  function getWeightColor(weight: number): string {
    const intensity = Math.floor(weight * 255);
    return `rgb(255, ${255 - intensity}, ${255 - intensity})`;
  }
</script>

{#if attention && attention.length > 0}
  <div class="attention-heatmap space-y-2">
    <!-- Layer/Head selector -->
    <div class="flex items-center gap-2 text-xs">
      <label class="text-ui-text-dim">
        Layer:
        <select
          bind:value={selectedLayer}
          class="ml-1 bg-ui-bg border border-gray-600 rounded px-1 py-0.5 text-ui-text"
        >
          {#each layers() as layer (layer)}
            <option value={layer}>L{layer}</option>
          {/each}
        </select>
      </label>

      <label class="text-ui-text-dim">
        Head:
        <select
          bind:value={selectedHead}
          class="ml-1 bg-ui-bg border border-gray-600 rounded px-1 py-0.5 text-ui-text"
        >
          {#each heads() as head (head)}
            <option value={head}>H{head}</option>
          {/each}
        </select>
      </label>
    </div>

    <!-- Attention visualization -->
    {#if currentWeights()}
      <div class="grid gap-0.5" style="grid-template-columns: repeat(auto-fill, minmax(24px, 1fr))">
        {#each currentWeights()!.weights as entry (entry.sourceId)}
          <div
            class="w-6 h-6 rounded text-[8px] flex items-center justify-center font-mono"
            style="background-color: {getWeightColor(entry.weight)}"
            title="{entry.sourceType} {entry.sourceId}: {(entry.weight * 100).toFixed(1)}%"
          >
            {entry.sourceId}
          </div>
        {/each}
      </div>

      <!-- Legend -->
      <div class="flex items-center gap-2 text-xs text-ui-text-dim">
        <span>Low</span>
        <div
          class="h-2 w-16 rounded"
          style="background: linear-gradient(to right, white, red)"
        ></div>
        <span>High</span>
      </div>
    {:else}
      <div class="text-xs text-ui-text-dim text-center py-2">
        No attention data for selected layer/head
      </div>
    {/if}
  </div>
{:else}
  <div class="text-xs text-ui-text-dim text-center py-4">
    <div class="mb-2">Attention Heatmap</div>
    <div class="text-ui-text-dim/70">
      Available when using neural network agents (future feature)
    </div>
  </div>
{/if}

<style>
  .attention-heatmap {
    font-family: ui-monospace, monospace;
  }
</style>
