<script lang="ts">
  import type { MatchStatistics } from "$lib/stats/types";

  interface Props {
    statistics: MatchStatistics;
  }

  let { statistics }: Props = $props();

  const p1 = $derived(statistics.actionEconomy.player1);
  const p2 = $derived(statistics.actionEconomy.player2);

  // Helper to determine who has the advantage
  function getAdvantage(v1: number, v2: number): 1 | 2 | 0 {
    if (v1 > v2) return 1;
    if (v2 > v1) return 2;
    return 0;
  }

  function formatPercent(value: number): string {
    return `${Math.round(value)}%`;
  }
</script>

<div class="space-y-6">
  <h3 class="text-lg font-bold text-ui-text">Action Economy</h3>

  <!-- Summary cards -->
  <div class="grid grid-cols-3 gap-4">
    <div class="bg-gray-800/50 rounded-lg p-4 text-center border border-gray-700">
      <div class="text-2xl font-bold text-ui-text">{p1.cardsPlayed + p2.cardsPlayed}</div>
      <div class="text-sm text-ui-text-dim">Total Cards Played</div>
    </div>
    <div class="bg-gray-800/50 rounded-lg p-4 text-center border border-gray-700">
      <div class="text-2xl font-bold text-ui-text">{p1.attacksMade + p2.attacksMade}</div>
      <div class="text-sm text-ui-text-dim">Total Attacks</div>
    </div>
    <div class="bg-gray-800/50 rounded-lg p-4 text-center border border-gray-700">
      <div class="text-2xl font-bold text-ui-text">{p1.abilitiesUsed + p2.abilitiesUsed}</div>
      <div class="text-sm text-ui-text-dim">Abilities Used</div>
    </div>
  </div>

  <!-- Comparison table -->
  <div class="bg-gray-800/50 rounded-lg border border-gray-700 overflow-hidden">
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
          <td class="px-4 py-2 text-sm text-ui-text">Cards Played</td>
          <td class="px-4 py-2 text-center font-medium {getAdvantage(p1.cardsPlayed, p2.cardsPlayed) === 1 ? 'text-health' : 'text-ui-text'}">{p1.cardsPlayed}</td>
          <td class="px-4 py-2 text-center font-medium {getAdvantage(p1.cardsPlayed, p2.cardsPlayed) === 2 ? 'text-damage' : 'text-ui-text'}">{p2.cardsPlayed}</td>
        </tr>
        <tr class="bg-gray-900/30">
          <td class="px-4 py-2 text-sm text-ui-text-dim pl-8">Creatures</td>
          <td class="px-4 py-2 text-center text-sm text-ui-text-dim">{p1.creaturesPlayed}</td>
          <td class="px-4 py-2 text-center text-sm text-ui-text-dim">{p2.creaturesPlayed}</td>
        </tr>
        <tr class="bg-gray-900/30">
          <td class="px-4 py-2 text-sm text-ui-text-dim pl-8">Spells</td>
          <td class="px-4 py-2 text-center text-sm text-ui-text-dim">{p1.spellsCast}</td>
          <td class="px-4 py-2 text-center text-sm text-ui-text-dim">{p2.spellsCast}</td>
        </tr>
        <tr class="bg-gray-900/30">
          <td class="px-4 py-2 text-sm text-ui-text-dim pl-8">Supports</td>
          <td class="px-4 py-2 text-center text-sm text-ui-text-dim">{p1.supportsPlaced}</td>
          <td class="px-4 py-2 text-center text-sm text-ui-text-dim">{p2.supportsPlaced}</td>
        </tr>
        <tr>
          <td class="px-4 py-2 text-sm text-ui-text">Attacks Made</td>
          <td class="px-4 py-2 text-center font-medium {getAdvantage(p1.attacksMade, p2.attacksMade) === 1 ? 'text-health' : 'text-ui-text'}">{p1.attacksMade}</td>
          <td class="px-4 py-2 text-center font-medium {getAdvantage(p1.attacksMade, p2.attacksMade) === 2 ? 'text-damage' : 'text-ui-text'}">{p2.attacksMade}</td>
        </tr>
        <tr class="bg-gray-900/30">
          <td class="px-4 py-2 text-sm text-ui-text-dim pl-8">Face Attacks</td>
          <td class="px-4 py-2 text-center text-sm text-ui-text-dim">{p1.faceAttacks}</td>
          <td class="px-4 py-2 text-center text-sm text-ui-text-dim">{p2.faceAttacks}</td>
        </tr>
        <tr class="bg-gray-900/30">
          <td class="px-4 py-2 text-sm text-ui-text-dim pl-8">Creature Attacks</td>
          <td class="px-4 py-2 text-center text-sm text-ui-text-dim">{p1.creatureAttacks}</td>
          <td class="px-4 py-2 text-center text-sm text-ui-text-dim">{p2.creatureAttacks}</td>
        </tr>
        <tr>
          <td class="px-4 py-2 text-sm text-ui-text">Abilities Used</td>
          <td class="px-4 py-2 text-center font-medium {getAdvantage(p1.abilitiesUsed, p2.abilitiesUsed) === 1 ? 'text-health' : 'text-ui-text'}">{p1.abilitiesUsed}</td>
          <td class="px-4 py-2 text-center font-medium {getAdvantage(p1.abilitiesUsed, p2.abilitiesUsed) === 2 ? 'text-damage' : 'text-ui-text'}">{p2.abilitiesUsed}</td>
        </tr>
        <tr>
          <td class="px-4 py-2 text-sm text-ui-text">Turns Played</td>
          <td class="px-4 py-2 text-center font-medium text-ui-text">{p1.turnsPlayed}</td>
          <td class="px-4 py-2 text-center font-medium text-ui-text">{p2.turnsPlayed}</td>
        </tr>
        <tr class="border-t border-gray-600">
          <td class="px-4 py-2 text-sm text-ui-text font-semibold">AP Spent</td>
          <td class="px-4 py-2 text-center font-bold {getAdvantage(p1.apSpent, p2.apSpent) === 1 ? 'text-health' : 'text-ui-text'}">{p1.apSpent}</td>
          <td class="px-4 py-2 text-center font-bold {getAdvantage(p1.apSpent, p2.apSpent) === 2 ? 'text-damage' : 'text-ui-text'}">{p2.apSpent}</td>
        </tr>
        <tr>
          <td class="px-4 py-2 text-sm text-ui-text font-semibold">AP Efficiency</td>
          <td class="px-4 py-2 text-center font-bold {getAdvantage(p1.apEfficiency, p2.apEfficiency) === 1 ? 'text-health' : 'text-ui-text'}">{formatPercent(p1.apEfficiency)}</td>
          <td class="px-4 py-2 text-center font-bold {getAdvantage(p1.apEfficiency, p2.apEfficiency) === 2 ? 'text-damage' : 'text-ui-text'}">{formatPercent(p2.apEfficiency)}</td>
        </tr>
      </tbody>
    </table>
  </div>
</div>
