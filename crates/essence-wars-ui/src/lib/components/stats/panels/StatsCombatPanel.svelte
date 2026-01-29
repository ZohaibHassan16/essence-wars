<script lang="ts">
  import type { MatchStatistics } from "$lib/stats/types";

  interface Props {
    statistics: MatchStatistics;
  }

  let { statistics }: Props = $props();

  const p1 = $derived(statistics.combat.player1);
  const p2 = $derived(statistics.combat.player2);

  function getAdvantage(v1: number, v2: number): 1 | 2 | 0 {
    if (v1 > v2) return 1;
    if (v2 > v1) return 2;
    return 0;
  }

  function formatRatio(value: number): string {
    if (!isFinite(value)) return "∞";
    return value.toFixed(2);
  }
</script>

<div class="space-y-6">
  <h3 class="text-lg font-bold text-ui-text">Combat Statistics</h3>

  <!-- Damage summary -->
  <div class="grid grid-cols-2 gap-4">
    <div class="bg-gray-800/50 rounded-lg p-5 border border-gray-700">
      <div class="flex items-center justify-between mb-3">
        <span class="text-sm font-semibold text-health">Player 1 Damage</span>
        <span class="text-3xl font-bold text-health">{p1.totalDamageDealt}</span>
      </div>
      <div class="flex gap-4 text-sm text-ui-text-dim">
        <span>To Creatures: {p1.damageToCreatures}</span>
        <span>To Face: {p1.damageToFace}</span>
      </div>
    </div>
    <div class="bg-gray-800/50 rounded-lg p-5 border border-gray-700">
      <div class="flex items-center justify-between mb-3">
        <span class="text-sm font-semibold text-damage">Player 2 Damage</span>
        <span class="text-3xl font-bold text-damage">{p2.totalDamageDealt}</span>
      </div>
      <div class="flex gap-4 text-sm text-ui-text-dim">
        <span>To Creatures: {p2.damageToCreatures}</span>
        <span>To Face: {p2.damageToFace}</span>
      </div>
    </div>
  </div>

  <!-- K/D Stats -->
  <div class="grid grid-cols-2 gap-4">
    <div class="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
      <div class="text-sm font-semibold text-ui-text-dim mb-2">Player 1 K/D</div>
      <div class="flex items-baseline gap-2">
        <span class="text-2xl font-bold text-health">{p1.creaturesKilled}</span>
        <span class="text-ui-text-dim">/</span>
        <span class="text-2xl font-bold text-damage">{p1.creaturesLost}</span>
        <span class="text-sm text-ui-text-dim ml-2">({formatRatio(p1.kdRatio)} ratio)</span>
      </div>
    </div>
    <div class="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
      <div class="text-sm font-semibold text-ui-text-dim mb-2">Player 2 K/D</div>
      <div class="flex items-baseline gap-2">
        <span class="text-2xl font-bold text-health">{p2.creaturesKilled}</span>
        <span class="text-ui-text-dim">/</span>
        <span class="text-2xl font-bold text-damage">{p2.creaturesLost}</span>
        <span class="text-sm text-ui-text-dim ml-2">({formatRatio(p2.kdRatio)} ratio)</span>
      </div>
    </div>
  </div>

  <!-- Trades Analysis -->
  <div class="bg-gray-800/50 rounded-lg border border-gray-700 overflow-hidden">
    <div class="px-4 py-3 border-b border-gray-700 bg-gray-900/30">
      <h4 class="text-sm font-semibold text-ui-text">Trade Analysis</h4>
    </div>
    <table class="w-full">
      <thead>
        <tr class="border-b border-gray-700">
          <th class="px-4 py-3 text-left text-sm font-semibold text-ui-text-dim">Trade Type</th>
          <th class="px-4 py-3 text-center text-sm font-semibold text-health">Player 1</th>
          <th class="px-4 py-3 text-center text-sm font-semibold text-damage">Player 2</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-700/50">
        <tr>
          <td class="px-4 py-3 text-sm text-ui-text">
            <span class="inline-flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-health"></span>
              Favorable Trades
            </span>
            <div class="text-xs text-ui-text-dim">Killed enemy without losing own creature</div>
          </td>
          <td class="px-4 py-3 text-center font-bold {getAdvantage(p1.favorableTrades, p2.favorableTrades) === 1 ? 'text-health' : 'text-ui-text'}">{p1.favorableTrades}</td>
          <td class="px-4 py-3 text-center font-bold {getAdvantage(p1.favorableTrades, p2.favorableTrades) === 2 ? 'text-damage' : 'text-ui-text'}">{p2.favorableTrades}</td>
        </tr>
        <tr>
          <td class="px-4 py-3 text-sm text-ui-text">
            <span class="inline-flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-gold"></span>
              Even Trades
            </span>
            <div class="text-xs text-ui-text-dim">Both creatures died</div>
          </td>
          <td class="px-4 py-3 text-center font-medium text-ui-text">{p1.evenTrades}</td>
          <td class="px-4 py-3 text-center font-medium text-ui-text">{p2.evenTrades}</td>
        </tr>
        <tr>
          <td class="px-4 py-3 text-sm text-ui-text">
            <span class="inline-flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-damage"></span>
              Unfavorable Trades
            </span>
            <div class="text-xs text-ui-text-dim">Lost creature without killing enemy</div>
          </td>
          <td class="px-4 py-3 text-center font-bold {getAdvantage(p2.unfavorableTrades, p1.unfavorableTrades) === 2 ? 'text-health' : 'text-ui-text'}">{p1.unfavorableTrades}</td>
          <td class="px-4 py-3 text-center font-bold {getAdvantage(p1.unfavorableTrades, p2.unfavorableTrades) === 1 ? 'text-damage' : 'text-ui-text'}">{p2.unfavorableTrades}</td>
        </tr>
      </tbody>
    </table>
  </div>

  <!-- Damage breakdown chart -->
  <div class="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
    <h4 class="text-sm font-semibold text-ui-text-dim mb-4">Damage Distribution</h4>
    <div class="space-y-4">
      <!-- P1 bar -->
      <div>
        <div class="flex items-center justify-between text-sm mb-1">
          <span class="text-health">Player 1</span>
          <span class="text-ui-text-dim">{p1.totalDamageDealt} total</span>
        </div>
        <div class="h-6 bg-gray-700 rounded-full overflow-hidden flex">
          {#if p1.totalDamageDealt > 0}
            <div
              class="h-full bg-health/80"
              style="width: {(p1.damageToCreatures / p1.totalDamageDealt) * 100}%"
              title="To Creatures: {p1.damageToCreatures}"
            ></div>
            <div
              class="h-full bg-health"
              style="width: {(p1.damageToFace / p1.totalDamageDealt) * 100}%"
              title="To Face: {p1.damageToFace}"
            ></div>
          {/if}
        </div>
      </div>

      <!-- P2 bar -->
      <div>
        <div class="flex items-center justify-between text-sm mb-1">
          <span class="text-damage">Player 2</span>
          <span class="text-ui-text-dim">{p2.totalDamageDealt} total</span>
        </div>
        <div class="h-6 bg-gray-700 rounded-full overflow-hidden flex">
          {#if p2.totalDamageDealt > 0}
            <div
              class="h-full bg-damage/80"
              style="width: {(p2.damageToCreatures / p2.totalDamageDealt) * 100}%"
              title="To Creatures: {p2.damageToCreatures}"
            ></div>
            <div
              class="h-full bg-damage"
              style="width: {(p2.damageToFace / p2.totalDamageDealt) * 100}%"
              title="To Face: {p2.damageToFace}"
            ></div>
          {/if}
        </div>
      </div>

      <div class="flex items-center gap-4 text-xs text-ui-text-dim justify-center mt-2">
        <span class="flex items-center gap-1">
          <span class="w-3 h-3 rounded bg-white/60"></span> To Creatures
        </span>
        <span class="flex items-center gap-1">
          <span class="w-3 h-3 rounded bg-white"></span> To Face
        </span>
      </div>
    </div>
  </div>
</div>
