<script lang="ts">
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import type { MoveScoreDto } from "$lib/api/types";
  import EvalTimeline from "./EvalTimeline.svelte";
  import SearchTreeView from "./SearchTreeView.svelte";
  import SpectatorControls from "../SpectatorControls.svelte";

  const match = $derived(spectatorStore.match);
  const gameState = $derived(spectatorStore.currentState);
  const insights = $derived(spectatorStore.currentInsights);
  const currentPlayer = $derived(spectatorStore.currentAction?.player);
  const isP1Turn = $derived(gameState?.activePlayer === 1);
  const isFinished = $derived(spectatorStore.phase === "finished");

  // Get top moves from insights
  const topMoves = $derived(insights?.moveScores.slice(0, 8) ?? []);

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

  function formatScore(score: number): string {
    const sign = score > 0 ? "+" : "";
    return `${sign}${score.toFixed(1)}`;
  }

  function formatProbability(prob: number): string {
    return `${Math.round(prob * 100)}%`;
  }

  function getProbabilityColor(prob: number): string {
    if (prob >= 0.5) return "text-health";
    if (prob >= 0.2) return "text-gold";
    return "text-ui-text-dim";
  }
</script>

<div class="analysis-dashboard w-full h-full flex flex-col bg-ui-bg overflow-hidden">
  <!-- Header -->
  <div class="flex items-center justify-between px-4 py-3 bg-ui-panel/80 border-b border-gray-700">
    <!-- Left: Back to Watch Mode -->
    <button
      class="flex items-center gap-2 px-3 py-1.5 text-sm rounded transition-colors
             bg-ui-bg hover:bg-gray-700 text-ui-text border border-gray-600"
      onclick={() => spectatorStore.setViewMode("watch")}
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
      </svg>
      Watch Mode
      <span class="text-ui-text-dim text-xs">[A]</span>
    </button>

    <!-- Center: Match info -->
    <div class="flex items-center gap-4">
      <span class="text-ui-text font-semibold">
        {match?.player1DeckName ?? "P1"} vs {match?.player2DeckName ?? "P2"}
      </span>
      <div class="flex items-center gap-2">
        <span class="px-2 py-0.5 rounded text-sm {isP1Turn ? 'bg-health/20 text-health' : 'bg-damage/20 text-damage'}">
          Turn {gameState?.turn ?? 0}
        </span>
        <span class="text-ui-text-dim text-sm">
          {isP1Turn ? "P1's Turn" : "P2's Turn"}
        </span>
      </div>
    </div>

    <!-- Right: Bot names + Navigation buttons -->
    <div class="flex items-center gap-3">
      <div class="text-ui-text-dim text-sm">
        {match?.player1BotName} vs {match?.player2BotName}
      </div>
      {#if isFinished && spectatorStore.matchStatistics}
        <button
          class="flex items-center gap-2 px-3 py-1.5 text-sm rounded transition-colors
                 bg-purple-600/20 hover:bg-purple-600 text-purple-400 hover:text-white border border-purple-500/50"
          onclick={() => spectatorStore.openStatsSummary()}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
          </svg>
          View Stats
        </button>
        <button
          class="flex items-center gap-2 px-3 py-1.5 text-sm rounded transition-colors
                 bg-damage/20 hover:bg-damage/40 text-damage border border-damage/50"
          onclick={() => spectatorStore.backToMenu()}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
          </svg>
          Quit
        </button>
      {/if}
    </div>
  </div>

  <!-- Main content area -->
  <div class="flex-1 overflow-auto p-4">
    <div class="max-w-7xl mx-auto space-y-4">
      <!-- Row 1: AI Decision + Position Evaluation -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <!-- AI Decision Panel -->
        <div class="analysis-card">
          <div class="card-header">
            <span class="card-icon">&#129302;</span>
            <span>AI Decision</span>
            {#if currentPlayer}
              <span class="ml-2 px-1.5 py-0.5 rounded text-xs {currentPlayer === 1 ? 'bg-health/20 text-health' : 'bg-damage/20 text-damage'}">
                P{currentPlayer}
              </span>
            {/if}
            {#if insights}
              <span class="ml-auto px-2 py-0.5 rounded text-xs bg-blue-500/20 text-blue-400">
                {insights.searchStats.algorithm}
              </span>
            {/if}
          </div>

          <div class="card-content">
            {#if insights}
              <!-- Confidence -->
              <div class="flex items-center justify-between text-sm mb-3">
                <span class="text-ui-text-dim">Confidence:</span>
                <div class="flex items-center gap-2">
                  <div class="w-24 h-2 bg-ui-bg rounded-full overflow-hidden">
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

              <!-- Search Stats -->
              <div class="flex gap-4 text-xs text-ui-text-dim mb-3">
                <span>{insights.searchStats.numActions} legal moves</span>
                {#if insights.searchStats.simulations}
                  <span>{insights.searchStats.simulations} sims</span>
                {/if}
                {#if insights.searchStats.depth}
                  <span>Depth {insights.searchStats.depth}</span>
                {/if}
              </div>

              <!-- Top Moves -->
              <div class="space-y-1.5">
                <div class="text-xs text-ui-text-dim font-medium">Top Moves:</div>
                {#each topMoves as move, i (i)}
                  <div class="p-2 rounded {move.isChosen ? 'bg-health/10 border border-health/30' : 'bg-ui-bg/50'}">
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-2">
                        <span class="text-xs text-ui-text-dim w-5">#{i + 1}</span>
                        {#if move.isChosen}
                          <span class="text-health text-xs">&#10003;</span>
                        {/if}
                        <span class="text-sm text-ui-text">{getActionDescription(move)}</span>
                      </div>
                      <div class="flex items-center gap-3 text-sm">
                        <span class={getProbabilityColor(move.probability)}>
                          {formatProbability(move.probability)}
                        </span>
                        <span class="text-ui-text-dim">
                          {formatScore(move.score)}
                        </span>
                      </div>
                    </div>
                    <div class="h-1.5 mt-1.5 bg-ui-bg rounded-full overflow-hidden">
                      <div
                        class="h-full {move.isChosen ? 'bg-health' : 'bg-gray-500'} transition-all"
                        style="width: {move.probability * 100}%"
                      ></div>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-center text-ui-text-dim py-8">
                No AI insights available for this action.
              </div>
            {/if}
          </div>
        </div>

        <!-- Position Evaluation Panel -->
        <div class="analysis-card">
          <div class="card-header">
            <span class="card-icon">&#128202;</span>
            <span>Position Evaluation</span>
            {#if insights?.evalBreakdown}
              <span class="ml-auto font-semibold {insights.evalBreakdown.totalScore > 0 ? 'text-health' : 'text-damage'}">
                {formatScore(insights.evalBreakdown.totalScore)}
              </span>
            {/if}
          </div>

          <div class="card-content">
            {#if insights?.evalBreakdown}
              <div class="space-y-2">
                {#each insights.evalBreakdown.factors as factor (factor.name)}
                  <div class="flex items-center gap-3 text-sm">
                    <span class="text-ui-text-dim w-24 truncate" title={factor.name}>{factor.name}</span>
                    <div class="flex-1 flex items-center gap-1">
                      <!-- P1 bar (left) -->
                      <div class="flex-1 h-4 bg-ui-bg rounded-l flex justify-end">
                        <div
                          class="h-full bg-health/60 rounded-l"
                          style="width: {Math.min(100, Math.abs(factor.p1Value) / 2)}%"
                        ></div>
                      </div>
                      <!-- P2 bar (right) -->
                      <div class="flex-1 h-4 bg-ui-bg rounded-r flex justify-start">
                        <div
                          class="h-full bg-damage/60 rounded-r"
                          style="width: {Math.min(100, Math.abs(factor.p2Value) / 2)}%"
                        ></div>
                      </div>
                    </div>
                    <span class="w-16 text-right {factor.contribution > 0 ? 'text-health' : factor.contribution < 0 ? 'text-damage' : 'text-ui-text-dim'}">
                      {formatScore(factor.contribution)}
                    </span>
                  </div>
                {/each}
              </div>

              <!-- Legend -->
              <div class="flex justify-center gap-6 mt-4 text-xs text-ui-text-dim">
                <span class="flex items-center gap-1">
                  <span class="w-3 h-3 rounded bg-health/60"></span> P1
                </span>
                <span class="flex items-center gap-1">
                  <span class="w-3 h-3 rounded bg-damage/60"></span> P2
                </span>
              </div>
            {:else}
              <div class="text-center text-ui-text-dim py-8">
                Position evaluation not available.
              </div>
            {/if}
          </div>
        </div>
      </div>

      <!-- Row 2: Large Evaluation Timeline -->
      <div class="analysis-card">
        <div class="card-header">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
          </svg>
          <span>Evaluation Timeline</span>
        </div>
        <div class="card-content">
          {#if (match?.evalHistory?.length ?? 0) > 0}
            <EvalTimeline height={180} />
          {:else}
            <div class="text-center text-ui-text-dim py-8">
              Timeline data not available.
            </div>
          {/if}
        </div>
      </div>

      <!-- Row 3: Search Tree + Commentary/Log -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <!-- Search Tree -->
        <div class="analysis-card">
          <div class="card-content p-0">
            {#if insights?.treeRoot}
              <SearchTreeView
                treeRoot={insights.treeRoot}
                algorithm={insights.searchStats.algorithm}
                totalSimulations={insights.searchStats.simulations ?? undefined}
              />
            {:else}
              <div class="p-4">
                <div class="card-header border-0 px-0 pt-0">
                  <span class="card-icon">&#128065;</span>
                  <span>Search Tree</span>
                </div>
                <div class="text-center text-ui-text-dim py-8">
                  Search tree not available for this bot type.
                </div>
              </div>
            {/if}
          </div>
        </div>

        <!-- Action Log & Commentary -->
        <div class="analysis-card">
          <div class="card-header">
            <span class="card-icon">📋</span>
            <span>Action Log</span>
            <!-- Commentary overlay toggle -->
            <button
              class="ml-auto text-xs px-2 py-0.5 rounded transition-colors
                     {spectatorStore.commentaryEnabled
                       ? 'bg-ui-action/20 text-ui-action border border-ui-action/50'
                       : 'bg-gray-700 text-ui-text-dim border border-gray-600'}"
              onclick={() => spectatorStore.setCommentaryEnabled(!spectatorStore.commentaryEnabled)}
              title="Toggle commentary overlay for key moments"
            >
              Overlay {spectatorStore.commentaryEnabled ? 'ON' : 'OFF'}
            </button>
          </div>
          <div class="card-content space-y-3">
            <!-- Description -->
            <p class="text-sm text-ui-text-dim">
              View the complete action log with integrated commentary, AI insights, and export options.
            </p>

            <!-- Action Log Button -->
            <button
              class="w-full flex items-center justify-center gap-2 px-4 py-3 text-sm rounded-lg transition-colors
                     bg-purple-600/20 text-purple-400 hover:bg-purple-600 hover:text-white
                     border border-purple-500/50 font-medium"
              onclick={() => spectatorStore.openActionLog()}
            >
              <span>📋</span>
              <span>Open Action Log</span>
              <span class="text-xs opacity-60">[L]</span>
            </button>

            <!-- Quick stats -->
            {#if match}
              <div class="flex items-center justify-center gap-4 text-xs text-ui-text-dim">
                <span>{match.actions.length} total actions</span>
                <span>•</span>
                <span>{spectatorStore.commentaryHistory.length} commentary entries</span>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Playback Controls (always at bottom) -->
  <div class="border-t border-gray-700">
    <SpectatorControls />
  </div>
</div>

<style>
  .analysis-card {
    background: rgba(30, 30, 40, 0.6);
    border: 1px solid rgba(75, 85, 99, 0.5);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: rgba(0, 0, 0, 0.2);
    border-bottom: 1px solid rgba(75, 85, 99, 0.3);
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-ui-text);
  }

  .card-icon {
    font-size: 1rem;
  }

  .card-content {
    padding: 1rem;
  }
</style>
