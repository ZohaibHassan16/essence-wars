<script lang="ts">
  import type { MatchStatistics, KeywordStats } from "$lib/stats/types";

  interface Props {
    statistics: MatchStatistics;
  }

  let { statistics }: Props = $props();

  const p1 = $derived(statistics.keywords.player1);
  const p2 = $derived(statistics.keywords.player2);

  // Keyword data with descriptions and icons
  const keywordData: {
    key: keyof KeywordStats;
    name: string;
    description: string;
    color: string;
  }[] = [
    { key: "rushAttacks", name: "Rush", description: "Attacks made on play turn", color: "text-orange-400" },
    { key: "guardBlocks", name: "Guard", description: "Attacks redirected", color: "text-blue-400" },
    { key: "lethalKills", name: "Lethal", description: "One-shot kills", color: "text-purple-400" },
    { key: "lifestealHealing", name: "Lifesteal", description: "Health healed", color: "text-green-400" },
    { key: "piercingDamage", name: "Piercing", description: "Overflow damage to face", color: "text-amber-400" },
    { key: "shieldAbsorbed", name: "Shield", description: "Damage absorbed", color: "text-cyan-400" },
    { key: "quickStrikes", name: "Quick", description: "First strike activations", color: "text-yellow-400" },
    { key: "fortifyReduced", name: "Fortify", description: "Damage reduced", color: "text-stone-400" },
    { key: "wardBlocks", name: "Ward", description: "Spells/abilities blocked", color: "text-violet-400" },
    { key: "rangedAttacks", name: "Ranged", description: "No counter-damage attacks", color: "text-lime-400" },
    { key: "stealthEvades", name: "Stealth", description: "Attacks evaded", color: "text-gray-400" },
    { key: "regenerateHealing", name: "Regenerate", description: "Health regenerated", color: "text-emerald-400" },
    { key: "volatileDamage", name: "Volatile", description: "Splash damage on death", color: "text-red-400" },
    { key: "frenzyStacks", name: "Frenzy", description: "Bonus attack gained", color: "text-rose-400" },
    { key: "chargeBonus", name: "Charge", description: "Times Charge triggered", color: "text-indigo-400" },
  ];

  function getAdvantage(v1: number, v2: number): 1 | 2 | 0 {
    if (v1 > v2) return 1;
    if (v2 > v1) return 2;
    return 0;
  }

  // Filter to show only keywords that were used
  const activeKeywords = $derived(
    keywordData.filter((kw) => p1[kw.key] > 0 || p2[kw.key] > 0)
  );

  const inactiveKeywords = $derived(
    keywordData.filter((kw) => p1[kw.key] === 0 && p2[kw.key] === 0)
  );
</script>

<div class="space-y-6">
  <h3 class="text-lg font-bold text-ui-text">Keyword Statistics</h3>

  {#if activeKeywords.length === 0}
    <div class="bg-gray-800/50 rounded-lg p-8 text-center border border-gray-700">
      <div class="text-4xl mb-3">🔮</div>
      <div class="text-ui-text-dim">No keyword activations were recorded in this match.</div>
    </div>
  {:else}
    <!-- Active keywords table -->
    <div class="bg-gray-800/50 rounded-lg border border-gray-700 overflow-hidden">
      <div class="px-4 py-3 border-b border-gray-700 bg-gray-900/30">
        <h4 class="text-sm font-semibold text-ui-text">Active Keywords ({activeKeywords.length})</h4>
      </div>
      <table class="w-full">
        <thead>
          <tr class="border-b border-gray-700">
            <th class="px-4 py-3 text-left text-sm font-semibold text-ui-text-dim">Keyword</th>
            <th class="px-4 py-3 text-center text-sm font-semibold text-health">Player 1</th>
            <th class="px-4 py-3 text-center text-sm font-semibold text-damage">Player 2</th>
            <th class="px-4 py-3 text-center text-sm font-semibold text-ui-text-dim">Total</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-700/50">
          {#each activeKeywords as kw}
            {@const v1 = p1[kw.key]}
            {@const v2 = p2[kw.key]}
            {@const adv = getAdvantage(v1, v2)}
            <tr>
              <td class="px-4 py-3">
                <div class="flex items-center gap-2">
                  <span class="font-semibold {kw.color}">{kw.name}</span>
                </div>
                <div class="text-xs text-ui-text-dim">{kw.description}</div>
              </td>
              <td class="px-4 py-3 text-center font-bold {adv === 1 ? 'text-health' : 'text-ui-text'}">{v1}</td>
              <td class="px-4 py-3 text-center font-bold {adv === 2 ? 'text-damage' : 'text-ui-text'}">{v2}</td>
              <td class="px-4 py-3 text-center text-ui-text-dim">{v1 + v2}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- Keyword comparison visual -->
    <div class="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
      <h4 class="text-sm font-semibold text-ui-text-dim mb-4">Keyword Impact Comparison</h4>
      <div class="space-y-3">
        {#each activeKeywords as kw}
          {@const v1 = p1[kw.key]}
          {@const v2 = p2[kw.key]}
          {@const total = v1 + v2}
          {#if total > 0}
            <div>
              <div class="flex items-center justify-between text-xs mb-1">
                <span class="{kw.color} font-medium">{kw.name}</span>
                <span class="text-ui-text-dim">{total}</span>
              </div>
              <div class="h-4 bg-gray-700 rounded-full overflow-hidden flex">
                <div
                  class="h-full bg-health transition-all"
                  style="width: {(v1 / total) * 100}%"
                ></div>
                <div
                  class="h-full bg-damage transition-all"
                  style="width: {(v2 / total) * 100}%"
                ></div>
              </div>
            </div>
          {/if}
        {/each}
      </div>
      <div class="flex items-center gap-4 text-xs text-ui-text-dim justify-center mt-4">
        <span class="flex items-center gap-1">
          <span class="w-3 h-3 rounded bg-health"></span> Player 1
        </span>
        <span class="flex items-center gap-1">
          <span class="w-3 h-3 rounded bg-damage"></span> Player 2
        </span>
      </div>
    </div>
  {/if}

  <!-- Inactive keywords (collapsed) -->
  {#if inactiveKeywords.length > 0}
    <details class="bg-gray-800/30 rounded-lg border border-gray-700/50">
      <summary class="px-4 py-3 text-sm text-ui-text-dim cursor-pointer hover:text-ui-text transition-colors">
        Unused Keywords ({inactiveKeywords.length})
      </summary>
      <div class="px-4 pb-3 flex flex-wrap gap-2">
        {#each inactiveKeywords as kw}
          <span class="px-2 py-1 rounded text-xs bg-gray-800 text-ui-text-dim">{kw.name}</span>
        {/each}
      </div>
    </details>
  {/if}
</div>
