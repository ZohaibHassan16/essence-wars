<script lang="ts">
  import type { MatchStatistics } from "$lib/stats/types";

  interface Props {
    statistics: MatchStatistics;
  }

  let { statistics }: Props = $props();

  const p1 = $derived(statistics.resources.player1);
  const p2 = $derived(statistics.resources.player2);

  function getAdvantage(v1: number, v2: number): 1 | 2 | 0 {
    if (v1 > v2) return 1;
    if (v2 > v1) return 2;
    return 0;
  }

  function formatDecimal(value: number): string {
    return value.toFixed(1);
  }
</script>

<div class="space-y-6">
  <h3 class="text-lg font-bold text-ui-text">Resource Economy</h3>

  <!-- Essence summary -->
  <div class="grid grid-cols-2 gap-4">
    <div class="bg-gradient-to-br from-purple-900/30 to-gray-800/50 rounded-lg p-5 border border-purple-700/30">
      <div class="flex items-center justify-between mb-3">
        <span class="text-sm font-semibold text-health">P1 Essence Spent</span>
        <span class="text-3xl font-bold text-purple-400">{p1.totalEssenceSpent}</span>
      </div>
      <div class="text-sm text-ui-text-dim">
        Avg per turn: {formatDecimal(p1.avgEssencePerTurn)}
      </div>
    </div>
    <div class="bg-gradient-to-br from-purple-900/30 to-gray-800/50 rounded-lg p-5 border border-purple-700/30">
      <div class="flex items-center justify-between mb-3">
        <span class="text-sm font-semibold text-damage">P2 Essence Spent</span>
        <span class="text-3xl font-bold text-purple-400">{p2.totalEssenceSpent}</span>
      </div>
      <div class="text-sm text-ui-text-dim">
        Avg per turn: {formatDecimal(p2.avgEssencePerTurn)}
      </div>
    </div>
  </div>

  <!-- Card flow -->
  <div class="bg-gray-800/50 rounded-lg border border-gray-700 overflow-hidden">
    <div class="px-4 py-3 border-b border-gray-700 bg-gray-900/30">
      <h4 class="text-sm font-semibold text-ui-text">Card Flow</h4>
    </div>
    <table class="w-full">
      <thead>
        <tr class="border-b border-gray-700">
          <th class="px-4 py-3 text-left text-sm font-semibold text-ui-text-dim">Metric</th>
          <th class="px-4 py-3 text-center text-sm font-semibold text-health">Player 1</th>
          <th class="px-4 py-3 text-center text-sm font-semibold text-damage">Player 2</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-700/50">
        <tr>
          <td class="px-4 py-3 text-sm text-ui-text">Cards Drawn</td>
          <td class="px-4 py-3 text-center font-medium {getAdvantage(p1.cardsDrawn, p2.cardsDrawn) === 1 ? 'text-health' : 'text-ui-text'}">{p1.cardsDrawn}</td>
          <td class="px-4 py-3 text-center font-medium {getAdvantage(p1.cardsDrawn, p2.cardsDrawn) === 2 ? 'text-damage' : 'text-ui-text'}">{p2.cardsDrawn}</td>
        </tr>
        <tr>
          <td class="px-4 py-3 text-sm text-ui-text">Cards Discarded</td>
          <td class="px-4 py-3 text-center font-medium text-ui-text">{p1.cardsDiscarded}</td>
          <td class="px-4 py-3 text-center font-medium text-ui-text">{p2.cardsDiscarded}</td>
        </tr>
        <tr>
          <td class="px-4 py-3 text-sm text-ui-text">Average Hand Size</td>
          <td class="px-4 py-3 text-center font-medium {getAdvantage(p1.averageHandSize, p2.averageHandSize) === 1 ? 'text-health' : 'text-ui-text'}">{formatDecimal(p1.averageHandSize)}</td>
          <td class="px-4 py-3 text-center font-medium {getAdvantage(p1.averageHandSize, p2.averageHandSize) === 2 ? 'text-damage' : 'text-ui-text'}">{formatDecimal(p2.averageHandSize)}</td>
        </tr>
        <tr>
          <td class="px-4 py-3 text-sm text-ui-text">Max Hand Size</td>
          <td class="px-4 py-3 text-center font-medium text-ui-text">{p1.maxHandSize}</td>
          <td class="px-4 py-3 text-center font-medium text-ui-text">{p2.maxHandSize}</td>
        </tr>
        <tr>
          <td class="px-4 py-3 text-sm text-ui-text">
            Empty Hand Turns
            <div class="text-xs text-ui-text-dim">Turns with no cards in hand</div>
          </td>
          <td class="px-4 py-3 text-center font-medium {p1.emptyHandTurns > 0 ? 'text-damage' : 'text-ui-text'}">{p1.emptyHandTurns}</td>
          <td class="px-4 py-3 text-center font-medium {p2.emptyHandTurns > 0 ? 'text-damage' : 'text-ui-text'}">{p2.emptyHandTurns}</td>
        </tr>
      </tbody>
    </table>
  </div>

  <!-- Resource efficiency insight -->
  <div class="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
    <h4 class="text-sm font-semibold text-ui-text-dim mb-3">Resource Efficiency</h4>
    <div class="space-y-3">
      <!-- P1 essence per card -->
      <div class="flex items-center justify-between">
        <span class="text-sm text-ui-text">P1 Essence per Card Played</span>
        <span class="font-medium text-purple-400">
          {statistics.actionEconomy.player1.cardsPlayed > 0
            ? formatDecimal(p1.totalEssenceSpent / statistics.actionEconomy.player1.cardsPlayed)
            : "-"}
        </span>
      </div>
      <!-- P2 essence per card -->
      <div class="flex items-center justify-between">
        <span class="text-sm text-ui-text">P2 Essence per Card Played</span>
        <span class="font-medium text-purple-400">
          {statistics.actionEconomy.player2.cardsPlayed > 0
            ? formatDecimal(p2.totalEssenceSpent / statistics.actionEconomy.player2.cardsPlayed)
            : "-"}
        </span>
      </div>
      <!-- P1 cards per turn -->
      <div class="flex items-center justify-between">
        <span class="text-sm text-ui-text">P1 Cards Played per Turn</span>
        <span class="font-medium text-ui-text">
          {statistics.actionEconomy.player1.turnsPlayed > 0
            ? formatDecimal(statistics.actionEconomy.player1.cardsPlayed / statistics.actionEconomy.player1.turnsPlayed)
            : "-"}
        </span>
      </div>
      <!-- P2 cards per turn -->
      <div class="flex items-center justify-between">
        <span class="text-sm text-ui-text">P2 Cards Played per Turn</span>
        <span class="font-medium text-ui-text">
          {statistics.actionEconomy.player2.turnsPlayed > 0
            ? formatDecimal(statistics.actionEconomy.player2.cardsPlayed / statistics.actionEconomy.player2.turnsPlayed)
            : "-"}
        </span>
      </div>
    </div>
  </div>
</div>
