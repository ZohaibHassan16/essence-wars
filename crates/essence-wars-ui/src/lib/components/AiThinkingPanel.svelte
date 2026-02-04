<script lang="ts">
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import type { MoveScoreDto } from "$lib/api/types";
  import SearchTreeView from "./spectator/SearchTreeView.svelte";

  let isExpanded = $state(true);
  let showEvalBreakdown = $state(false);
  let showSearchTree = $state(false);

  // Get current action's data
  const insights = $derived(spectatorStore.currentInsights);
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

  function formatProbability(prob: number): string {
    return `${Math.round(prob * 100)}%`;
  }

  function formatScore(score: number): string {
    const sign = score > 0 ? "+" : "";
    return `${sign}${score.toFixed(1)}`;
  }

  function getActionDescription(move: MoveScoreDto): string {
    const action = move.action;
    switch (action.actionType) {
      case "play_card":
        return `Play card ${action.handIndex ?? "?"} → slot ${action.targetSlot}`;
      case "attack":
        return `Attack ${action.sourceSlot} → ${action.targetSlot}`;
      case "end_turn":
        return "End Turn";
      case "use_ability":
        return `Ability ${action.abilityIndex}`;
      case "commander_insight":
        return "Commander's Insight";
      default:
        return action.description;
    }
  }

  function getProbabilityColor(prob: number): string {
    if (prob >= 0.5) return "text-health";
    if (prob >= 0.2) return "text-gold";
    return "text-ui-text-dim";
  }

  // Get top moves from insights (sorted by score)
  const topMoves = $derived(
    insights?.moveScores.slice(0, 5) ?? []
  );

  // Get the chosen move rank
  const chosenRank = $derived(() => {
    if (!insights) return null;
    const idx = insights.moveScores.findIndex(m => m.isChosen);
    return idx >= 0 ? idx + 1 : null;
  });

  // Keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    // Ignore if typing in an input
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
      return;
    }

    switch (event.key.toLowerCase()) {
      case "t":
        // Toggle search tree (only if available)
        if (insights?.treeRoot) {
          showSearchTree = !showSearchTree;
        }
        break;
      case "e":
        // Toggle eval breakdown (only if available)
        if (insights?.evalBreakdown) {
          showEvalBreakdown = !showEvalBreakdown;
        }
        break;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

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
      {:else if insights}
        <!-- New insights data available -->
        <div class="space-y-2">
          <!-- Bot info and stats -->
          <div class="flex items-center justify-between text-xs">
            <span class="px-1.5 py-0.5 rounded bg-blue-500/20 text-blue-400 font-medium">
              {insights.searchStats.algorithm}
            </span>
            <div class="flex items-center gap-2 text-ui-text-dim">
              <span>{insights.searchStats.numActions} moves</span>
              <span>{thinkingTimeMs}ms</span>
            </div>
          </div>

          <!-- Search parameters if available -->
          {#if insights.searchStats.simulations || insights.searchStats.depth}
            <div class="text-xs text-ui-text-dim flex gap-2">
              {#if insights.searchStats.simulations}
                <span>Sims: {insights.searchStats.simulations}</span>
              {/if}
              {#if insights.searchStats.depth}
                <span>Depth: {insights.searchStats.depth}</span>
              {/if}
            </div>
          {/if}

          <!-- Confidence indicator -->
          <div class="flex items-center justify-between text-xs">
            <span class="text-ui-text-dim">Confidence:</span>
            <div class="flex items-center gap-2">
              <div class="w-16 h-1.5 bg-ui-bg rounded-full overflow-hidden">
                <div
                  class="h-full rounded-full transition-all {
                    insights.confidenceLevel === 'high' ? 'bg-health' :
                    insights.confidenceLevel === 'medium' ? 'bg-gold' : 'bg-damage'
                  }"
                  style="width: {insights.confidence * 100}%"
                ></div>
              </div>
              <span class="{
                insights.confidenceLevel === 'high' ? 'text-health' :
                insights.confidenceLevel === 'medium' ? 'text-gold' : 'text-damage'
              } font-medium">
                {Math.round(insights.confidence * 100)}%
              </span>
            </div>
          </div>

          <!-- Top moves -->
          <div class="space-y-1">
            <div class="flex items-center justify-between text-xs">
              <span class="text-ui-text-dim">Top Moves:</span>
              {#if chosenRank()}
                <span class="text-health">Chose #{chosenRank()}</span>
              {/if}
            </div>
            {#each topMoves as move, i (i)}
              <div class="p-1.5 rounded {move.isChosen ? 'bg-health/10 border border-health/30' : 'bg-ui-bg/50'}">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-1.5">
                    <span class="text-xs text-ui-text-dim">#{i + 1}</span>
                    {#if move.isChosen}
                      <span class="text-health text-xs">&#10003;</span>
                    {/if}
                    <span class="text-xs text-ui-text truncate max-w-[110px]">{getActionDescription(move)}</span>
                  </div>
                  <div class="flex items-center gap-2 text-xs">
                    <span class={getProbabilityColor(move.probability)} title="Probability">
                      {formatProbability(move.probability)}
                    </span>
                    <span class="text-ui-text-dim" title="Score">
                      {formatScore(move.score)}
                    </span>
                  </div>
                </div>
                <!-- Probability bar -->
                <div class="h-1 mt-1 bg-ui-bg rounded-full overflow-hidden">
                  <div
                    class="h-full {move.isChosen ? 'bg-health' : 'bg-gray-500'} transition-all"
                    style="width: {move.probability * 100}%"
                  ></div>
                </div>
              </div>
            {/each}
          </div>

          <!-- Eval breakdown toggle -->
          {#if insights.evalBreakdown}
            <button
              class="w-full text-xs px-2 py-1 rounded transition-colors
                     {showEvalBreakdown
                       ? 'bg-purple-500/20 text-purple-400 border border-purple-500/30'
                       : 'bg-ui-bg/50 text-ui-text-dim hover:bg-ui-bg/70'}"
              onclick={() => showEvalBreakdown = !showEvalBreakdown}
              title="Press E to toggle"
            >
              {showEvalBreakdown ? '▼' : '▶'} Position Evaluation
              <span class="ml-1 font-semibold {insights.evalBreakdown.totalScore > 0 ? 'text-health' : 'text-damage'}">
                {formatScore(insights.evalBreakdown.totalScore)}
              </span>
              <span class="ml-1 text-ui-text-dim/50">[E]</span>
            </button>

            {#if showEvalBreakdown}
              <div class="space-y-0.5 text-xs pl-2">
                {#each insights.evalBreakdown.factors as factor (factor.name)}
                  <div class="flex items-center justify-between">
                    <span class="text-ui-text-dim">{factor.name}</span>
                    <div class="flex items-center gap-2">
                      <span class="text-ui-text-dim">
                        {factor.p1Value.toFixed(0)}|{factor.p2Value.toFixed(0)}
                      </span>
                      <span class="{factor.contribution > 0 ? 'text-health' : 'text-damage'}">
                        {formatScore(factor.contribution)}
                      </span>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          {/if}

          <!-- Search Tree toggle -->
          {#if insights.treeRoot}
            <button
              class="w-full text-xs px-2 py-1 rounded transition-colors
                     {showSearchTree
                       ? 'bg-blue-500/20 text-blue-400 border border-blue-500/30'
                       : 'bg-ui-bg/50 text-ui-text-dim hover:bg-ui-bg/70'}"
              onclick={() => showSearchTree = !showSearchTree}
              title="Press T to toggle"
            >
              {showSearchTree ? '▼' : '▶'} Search Tree
              <span class="ml-1 text-ui-text-dim">
                {insights.treeRoot.children.length} moves
              </span>
              <span class="ml-1 text-ui-text-dim/50">[T]</span>
            </button>

            {#if showSearchTree}
              <SearchTreeView
                treeRoot={insights.treeRoot}
                algorithm={insights.searchStats.algorithm}
                totalSimulations={insights.searchStats.simulations ?? undefined}
              />
            {/if}
          {/if}
        </div>
      {:else if thinking}
        <!-- Legacy MCTS thinking data (fallback) -->
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
        <!-- No data available -->
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
            Detailed thinking data not available.
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
