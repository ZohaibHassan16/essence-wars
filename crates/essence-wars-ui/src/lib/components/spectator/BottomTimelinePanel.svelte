<script lang="ts">
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import EvalTimeline from "./EvalTimeline.svelte";

  const match = $derived(spectatorStore.match);
  const canShowTimeline = $derived(spectatorStore.canShowTimeline);
  const hasEvalHistory = $derived((match?.evalHistory?.length ?? 0) > 0);
</script>

{#if canShowTimeline && hasEvalHistory}
  <div class="bottom-timeline-panel bg-ui-panel/80 border-t border-gray-700">
    <!-- Header with mode toggle -->
    <div class="flex items-center justify-between px-3 py-1.5 border-b border-gray-700/50">
      <div class="flex items-center gap-2 text-xs text-ui-text-dim">
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
        </svg>
        <span>Evaluation Timeline</span>
      </div>

      <!-- Analysis mode toggle -->
      <button
        class="flex items-center gap-1.5 px-2 py-1 text-xs rounded transition-colors
               bg-purple-500/20 text-purple-400 hover:bg-purple-500/30 border border-purple-500/30"
        onclick={() => spectatorStore.setViewMode("analysis")}
        title="Switch to Analysis Mode [A]"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
        </svg>
        Analysis
        <span class="text-purple-400/60">[A]</span>
      </button>
    </div>

    <!-- Timeline graph -->
    <div class="px-3 py-2">
      <EvalTimeline height={100} />
    </div>
  </div>
{/if}

<style>
  .bottom-timeline-panel {
    /* Ensure it stays at bottom and doesn't shrink */
    flex-shrink: 0;
  }
</style>
