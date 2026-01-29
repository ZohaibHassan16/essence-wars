<script lang="ts">
  import type { SpectatorMatch } from "$lib/api/types";
  import type { MatchStatistics } from "$lib/stats/types";

  interface Props {
    match: SpectatorMatch;
    statistics: MatchStatistics;
  }

  let { match, statistics }: Props = $props();

  const overview = $derived(statistics.overview);
  const winner = $derived(overview.winner);
  const isDraw = $derived(winner === null);

  function formatReason(reason: string): string {
    return reason
      .replace(/([A-Z])/g, " $1")
      .replace(/_/g, " ")
      .trim()
      .replace(/^./, (c) => c.toUpperCase());
  }

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    const seconds = Math.round(ms / 1000);
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${minutes}m ${secs}s`;
  }
</script>

<div class="space-y-6">
  <!-- Winner Banner -->
  <div class="text-center py-6 rounded-xl border
              {isDraw
                ? 'bg-gold/10 border-gold/30'
                : winner === 1
                  ? 'bg-health/10 border-health/30'
                  : 'bg-damage/10 border-damage/30'}">
    <div class="text-5xl mb-3">
      {isDraw ? "🤝" : "🏆"}
    </div>
    <h3 class="text-3xl font-bold mb-2
               {isDraw ? 'text-gold' : winner === 1 ? 'text-health' : 'text-damage'}">
      {#if isDraw}
        DRAW
      {:else if winner === 1}
        P1 WINS
      {:else}
        P2 WINS
      {/if}
    </h3>
    <p class="text-ui-text-dim">{formatReason(overview.reason)}</p>
  </div>

  <!-- Match Info Cards -->
  <div class="grid grid-cols-2 gap-4">
    <!-- P1 Info -->
    <div class="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
      <div class="flex items-center gap-2 mb-3">
        <span class="w-2 h-2 rounded-full bg-health"></span>
        <span class="text-sm font-semibold text-ui-text">Player 1</span>
      </div>
      <div class="space-y-2">
        <div class="text-lg font-bold text-ui-text">{match.player1DeckName}</div>
        <div class="text-sm text-ui-text-dim">{match.player1BotName}</div>
        <div class="flex items-center gap-2 mt-2">
          <span class="text-2xl font-bold {overview.player1FinalLife > 0 ? 'text-health' : 'text-damage'}">
            {overview.player1FinalLife}
          </span>
          <span class="text-sm text-ui-text-dim">final life</span>
        </div>
      </div>
    </div>

    <!-- P2 Info -->
    <div class="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
      <div class="flex items-center gap-2 mb-3">
        <span class="w-2 h-2 rounded-full bg-damage"></span>
        <span class="text-sm font-semibold text-ui-text">Player 2</span>
      </div>
      <div class="space-y-2">
        <div class="text-lg font-bold text-ui-text">{match.player2DeckName}</div>
        <div class="text-sm text-ui-text-dim">{match.player2BotName}</div>
        <div class="flex items-center gap-2 mt-2">
          <span class="text-2xl font-bold {overview.player2FinalLife > 0 ? 'text-health' : 'text-damage'}">
            {overview.player2FinalLife}
          </span>
          <span class="text-sm text-ui-text-dim">final life</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Quick Stats -->
  <div class="grid grid-cols-4 gap-4">
    <div class="bg-gray-800/50 rounded-lg p-4 text-center border border-gray-700">
      <div class="text-3xl font-bold text-ui-text">{overview.totalTurns}</div>
      <div class="text-sm text-ui-text-dim">Total Turns</div>
    </div>
    <div class="bg-gray-800/50 rounded-lg p-4 text-center border border-gray-700">
      <div class="text-3xl font-bold text-ui-text">{overview.totalActions}</div>
      <div class="text-sm text-ui-text-dim">Total Actions</div>
    </div>
    <div class="bg-gray-800/50 rounded-lg p-4 text-center border border-gray-700">
      <div class="text-3xl font-bold text-ui-text">{formatDuration(overview.gameDurationMs)}</div>
      <div class="text-sm text-ui-text-dim">Duration</div>
    </div>
    <div class="bg-gray-800/50 rounded-lg p-4 text-center border border-gray-700">
      <div class="text-3xl font-bold {overview.lifeDifferential > 0 ? 'text-health' : overview.lifeDifferential < 0 ? 'text-damage' : 'text-ui-text'}">
        {overview.lifeDifferential > 0 ? '+' : ''}{overview.lifeDifferential}
      </div>
      <div class="text-sm text-ui-text-dim">Life Differential</div>
    </div>
  </div>

  <!-- MVP Card (if available) -->
  {#if overview.mvpCard}
    <div class="bg-gradient-to-r from-gold/10 to-transparent rounded-lg p-4 border border-gold/30">
      <div class="flex items-center gap-3">
        <span class="text-2xl">⭐</span>
        <div>
          <div class="text-sm text-gold font-semibold">MVP Card</div>
          <div class="text-lg font-bold text-ui-text">{overview.mvpCard.name}</div>
          <div class="text-sm text-ui-text-dim">Impact Score: {overview.mvpCard.impact}</div>
        </div>
      </div>
    </div>
  {/if}
</div>
