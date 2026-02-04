<script lang="ts">
  import type { DecisionInsightsDto, MoveScoreDto } from "$lib/api/types";

  let {
    insights,
    maxMovesToShow = 5,
    showEvalBreakdown = false,
  }: {
    insights: DecisionInsightsDto | null;
    maxMovesToShow?: number;
    showEvalBreakdown?: boolean;
  } = $props();

  // Format probability as percentage
  function formatProbability(prob: number): string {
    return `${Math.round(prob * 100)}%`;
  }

  // Format score with sign
  function formatScore(score: number): string {
    const sign = score > 0 ? "+" : "";
    return `${sign}${score.toFixed(1)}`;
  }

  // Get action description
  function getActionDescription(move: MoveScoreDto): string {
    const action = move.action;
    switch (action.actionType) {
      case "play_card":
        return `Play card to slot ${action.targetSlot}`;
      case "attack":
        return `Attack slot ${action.targetSlot}`;
      case "end_turn":
        return "End Turn";
      case "use_ability":
        return `Use ability ${action.abilityIndex}`;
      case "commander_insight":
        return "Commander's Insight";
      default:
        return action.description;
    }
  }

  // Get color class based on probability
  function getProbabilityColor(prob: number): string {
    if (prob >= 0.5) return "text-green-400";
    if (prob >= 0.2) return "text-yellow-400";
    return "text-gray-400";
  }

  // Get top moves
  const topMoves = $derived(
    insights?.moveScores.slice(0, maxMovesToShow) ?? []
  );

  // Get the chosen move
  const chosenMove = $derived(
    insights?.moveScores.find(m => m.isChosen) ?? null
  );

  // Get chosen move rank
  const chosenRank = $derived(
    chosenMove ? insights!.moveScores.findIndex(m => m.isChosen) + 1 : null
  );
</script>

{#if insights}
  <div class="ai-stats-bar">
    <!-- Search Stats Header -->
    <div class="stats-header">
      <span class="algorithm-badge">
        {insights.searchStats.algorithm}
      </span>
      <span class="stats-detail">
        {insights.searchStats.numActions} moves analyzed
      </span>
      {#if insights.searchStats.timeMs > 0}
        <span class="stats-detail">
          {insights.searchStats.timeMs}ms
        </span>
      {/if}
      {#if insights.searchStats.simulations}
        <span class="stats-detail">
          {insights.searchStats.simulations} sims
        </span>
      {/if}
      {#if insights.searchStats.depth}
        <span class="stats-detail">
          depth {insights.searchStats.depth}
        </span>
      {/if}
    </div>

    <!-- Move Rankings -->
    <div class="move-rankings">
      <div class="rankings-header">
        <span class="header-label">Top Moves</span>
        {#if chosenMove && chosenRank}
          <span class="chosen-indicator">
            Chose #{chosenRank}
          </span>
        {/if}
      </div>

      <div class="moves-list">
        {#each topMoves as move, i (i)}
          <div
            class="move-item"
            class:is-chosen={move.isChosen}
          >
            <span class="rank">#{i + 1}</span>
            <span class="action-desc">{getActionDescription(move)}</span>
            <span class={`probability ${getProbabilityColor(move.probability)}`}>
              {formatProbability(move.probability)}
            </span>
            <span class="score">{formatScore(move.score)}</span>
          </div>
        {/each}
      </div>
    </div>

    <!-- Evaluation Breakdown (optional) -->
    {#if showEvalBreakdown && insights.evalBreakdown}
      <div class="eval-breakdown">
        <div class="eval-header">
          <span class="header-label">Position Evaluation</span>
          <span class="total-score" class:positive={insights.evalBreakdown.totalScore > 0}>
            {formatScore(insights.evalBreakdown.totalScore)}
          </span>
        </div>

        <div class="factors-list">
          {#each insights.evalBreakdown.factors as factor (factor.name)}
            <div class="factor-item">
              <span class="factor-name">{factor.name}</span>
              <span class="factor-values">
                P1: {factor.p1Value.toFixed(0)} | P2: {factor.p2Value.toFixed(0)}
              </span>
              <span class="factor-contribution" class:positive={factor.contribution > 0}>
                {formatScore(factor.contribution)}
              </span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .ai-stats-bar {
    background: rgba(0, 0, 0, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.5rem;
    padding: 0.75rem;
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    color: #e0e0e0;
  }

  .stats-header {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    margin-bottom: 0.5rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .algorithm-badge {
    background: rgba(59, 130, 246, 0.3);
    color: #60a5fa;
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    font-weight: 600;
  }

  .stats-detail {
    color: #9ca3af;
  }

  .move-rankings {
    margin-bottom: 0.5rem;
  }

  .rankings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.25rem;
  }

  .header-label {
    color: #9ca3af;
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .chosen-indicator {
    color: #34d399;
    font-size: 0.65rem;
  }

  .moves-list {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .move-item {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    padding: 0.25rem 0.375rem;
    border-radius: 0.25rem;
    background: rgba(255, 255, 255, 0.02);
  }

  .move-item.is-chosen {
    background: rgba(52, 211, 153, 0.1);
    border: 1px solid rgba(52, 211, 153, 0.3);
  }

  .rank {
    color: #6b7280;
    width: 1.5rem;
  }

  .action-desc {
    flex: 1;
    color: #d1d5db;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .probability {
    font-weight: 600;
    width: 2.5rem;
    text-align: right;
  }

  .score {
    color: #9ca3af;
    width: 3rem;
    text-align: right;
  }

  .eval-breakdown {
    margin-top: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  .eval-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.25rem;
  }

  .total-score {
    font-weight: 700;
    font-size: 0.875rem;
    color: #ef4444;
  }

  .total-score.positive {
    color: #34d399;
  }

  .factors-list {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .factor-item {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    font-size: 0.65rem;
  }

  .factor-name {
    flex: 1;
    color: #9ca3af;
  }

  .factor-values {
    color: #6b7280;
    width: 6rem;
    text-align: right;
  }

  .factor-contribution {
    width: 3rem;
    text-align: right;
    color: #ef4444;
  }

  .factor-contribution.positive {
    color: #34d399;
  }
</style>
