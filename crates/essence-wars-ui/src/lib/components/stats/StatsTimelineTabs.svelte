<script lang="ts">
  import type { MatchStatistics } from "$lib/stats/types";
  import LifeGraph from "./graphs/LifeGraph.svelte";
  import WinProbGraph from "./graphs/WinProbGraph.svelte";
  import BoardGraph from "./graphs/BoardGraph.svelte";
  import EssenceGraph from "./graphs/EssenceGraph.svelte";

  interface Props {
    statistics: MatchStatistics;
  }

  let { statistics }: Props = $props();

  type GraphTab = "life" | "winprob" | "board" | "essence";
  let activeGraph = $state<GraphTab>("life");

  const graphTabs: { id: GraphTab; label: string; description: string }[] = [
    { id: "life", label: "Life Totals", description: "Commander health over time" },
    { id: "winprob", label: "Win Probability", description: "MCTS evaluation over time" },
    { id: "board", label: "Board Presence", description: "Creature strength on board" },
    { id: "essence", label: "Essence Spent", description: "Cumulative resource usage" },
  ];
</script>

<div class="space-y-4">
  <h3 class="text-lg font-bold text-ui-text">Timeline Graphs</h3>

  <!-- Graph tab buttons -->
  <div class="flex gap-2 border-b border-gray-700 pb-2">
    {#each graphTabs as tab}
      <button
        class="px-4 py-2 rounded-t-lg text-sm font-medium transition-colors
               {activeGraph === tab.id
                 ? 'bg-ui-action/20 text-ui-action border-b-2 border-ui-action'
                 : 'text-ui-text-dim hover:text-ui-text hover:bg-gray-800'}"
        onclick={() => (activeGraph = tab.id)}
        title={tab.description}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <!-- Graph container -->
  <div class="bg-gray-800/50 rounded-lg border border-gray-700 p-4 min-h-[400px]">
    {#if activeGraph === "life"}
      <LifeGraph timeline={statistics.timeline} />
    {:else if activeGraph === "winprob"}
      <WinProbGraph timeline={statistics.timeline} hasAiData={!!statistics.aiAnalysis} />
    {:else if activeGraph === "board"}
      <BoardGraph timeline={statistics.timeline} />
    {:else if activeGraph === "essence"}
      <EssenceGraph timeline={statistics.timeline} />
    {/if}
  </div>

  <!-- Legend -->
  <div class="flex items-center justify-center gap-6 text-sm">
    <span class="flex items-center gap-2">
      <span class="w-4 h-0.5 bg-health"></span>
      <span class="text-ui-text-dim">Player 1</span>
    </span>
    <span class="flex items-center gap-2">
      <span class="w-4 h-0.5 bg-damage"></span>
      <span class="text-ui-text-dim">Player 2</span>
    </span>
  </div>
</div>
