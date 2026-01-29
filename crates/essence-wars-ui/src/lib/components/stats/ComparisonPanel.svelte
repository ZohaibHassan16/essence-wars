<script lang="ts">
  import type { MatchStatistics } from "$lib/stats/types";

  interface Props {
    stats1: MatchStatistics;
    stats2: MatchStatistics;
  }

  let { stats1, stats2 }: Props = $props();

  // Tab state
  type TabId = "overview" | "combat" | "action" | "resources";
  let activeTab = $state<TabId>("overview");

  const tabs: { id: TabId; label: string }[] = [
    { id: "overview", label: "Overview" },
    { id: "combat", label: "Combat" },
    { id: "action", label: "Actions" },
    { id: "resources", label: "Resources" },
  ];

  // Helper to determine which value is "better" (higher)
  // Returns: 1 = match1 better, 2 = match2 better, 0 = equal
  function compareValues(v1: number, v2: number, higherIsBetter: boolean = true): 1 | 2 | 0 {
    if (v1 === v2) return 0;
    if (higherIsBetter) {
      return v1 > v2 ? 1 : 2;
    } else {
      return v1 < v2 ? 1 : 2;
    }
  }

  // Format percentage
  function formatPct(value: number): string {
    return `${Math.round(value)}%`;
  }

  // Format decimal
  function formatDec(value: number, decimals: number = 1): string {
    return value.toFixed(decimals);
  }

  // Get diff indicator class
  function getDiffClass(comparison: 1 | 2 | 0, forMatch: 1 | 2): string {
    if (comparison === 0) return "text-ui-text";
    if (comparison === forMatch) return forMatch === 1 ? "text-health font-bold" : "text-damage font-bold";
    return "text-ui-text-dim";
  }

  // Get diff arrow
  function getDiffArrow(comparison: 1 | 2 | 0, forMatch: 1 | 2): string {
    if (comparison === 0) return "";
    if (comparison === forMatch) return " ▲";
    return " ▼";
  }

  // Derived comparisons for Overview tab
  const overviewCmp = $derived({
    turns: compareValues(stats1.overview.totalTurns, stats2.overview.totalTurns, false),
    actions: compareValues(stats1.overview.totalActions, stats2.overview.totalActions),
    duration: compareValues(stats1.overview.gameDurationMs, stats2.overview.gameDurationMs, false),
    p1Life: compareValues(stats1.overview.player1FinalLife, stats2.overview.player1FinalLife),
    p2Life: compareValues(stats1.overview.player2FinalLife, stats2.overview.player2FinalLife),
  });

  // Derived comparisons for Combat tab - Player 1
  const combatP1Stats1 = $derived(stats1.combat.player1);
  const combatP1Stats2 = $derived(stats2.combat.player1);
  const combatP1Cmp = $derived({
    totalDmg: compareValues(combatP1Stats1.totalDamageDealt, combatP1Stats2.totalDamageDealt),
    faceDmg: compareValues(combatP1Stats1.damageToFace, combatP1Stats2.damageToFace),
    kills: compareValues(combatP1Stats1.creaturesKilled, combatP1Stats2.creaturesKilled),
    lost: compareValues(combatP1Stats1.creaturesLost, combatP1Stats2.creaturesLost, false),
    kd: compareValues(combatP1Stats1.kdRatio, combatP1Stats2.kdRatio),
    favorable: compareValues(combatP1Stats1.favorableTrades, combatP1Stats2.favorableTrades),
  });

  // Derived comparisons for Combat tab - Player 2
  const combatP2Stats1 = $derived(stats1.combat.player2);
  const combatP2Stats2 = $derived(stats2.combat.player2);
  const combatP2Cmp = $derived({
    totalDmg: compareValues(combatP2Stats1.totalDamageDealt, combatP2Stats2.totalDamageDealt),
    faceDmg: compareValues(combatP2Stats1.damageToFace, combatP2Stats2.damageToFace),
    kills: compareValues(combatP2Stats1.creaturesKilled, combatP2Stats2.creaturesKilled),
    kd: compareValues(combatP2Stats1.kdRatio, combatP2Stats2.kdRatio),
  });

  // Derived comparisons for Action tab
  const actionStats1 = $derived(stats1.actionEconomy.player1);
  const actionStats2 = $derived(stats2.actionEconomy.player1);
  const actionCmp = $derived({
    cards: compareValues(actionStats1.cardsPlayed, actionStats2.cardsPlayed),
    creatures: compareValues(actionStats1.creaturesPlayed, actionStats2.creaturesPlayed),
    spells: compareValues(actionStats1.spellsCast, actionStats2.spellsCast),
    attacks: compareValues(actionStats1.attacksMade, actionStats2.attacksMade),
    apSpent: compareValues(actionStats1.apSpent, actionStats2.apSpent),
    apEff: compareValues(actionStats1.apEfficiency, actionStats2.apEfficiency),
  });

  // Derived comparisons for Resources tab
  const resourceStats1 = $derived(stats1.resources.player1);
  const resourceStats2 = $derived(stats2.resources.player1);
  const resourceCmp = $derived({
    essence: compareValues(resourceStats1.totalEssenceSpent, resourceStats2.totalEssenceSpent),
    avgEssence: compareValues(resourceStats1.avgEssencePerTurn, resourceStats2.avgEssencePerTurn),
    drawn: compareValues(resourceStats1.cardsDrawn, resourceStats2.cardsDrawn),
    avgHand: compareValues(resourceStats1.averageHandSize, resourceStats2.averageHandSize),
    emptyHand: compareValues(resourceStats1.emptyHandTurns, resourceStats2.emptyHandTurns, false),
  });
</script>

<div class="space-y-6">
  <!-- Tab navigation -->
  <div class="flex gap-2 border-b border-gray-700 pb-2">
    {#each tabs as tab}
      <button
        class="px-4 py-2 rounded-t-lg text-sm font-medium transition-colors
               {activeTab === tab.id
                 ? 'bg-ui-action/20 text-ui-action border-b-2 border-ui-action'
                 : 'text-ui-text-dim hover:text-ui-text hover:bg-gray-800'}"
        onclick={() => (activeTab = tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <!-- Content -->
  <div class="bg-ui-panel rounded-xl border border-gray-700 overflow-hidden">
    {#if activeTab === "overview"}
      <!-- Overview Comparison -->
      <table class="w-full">
        <thead>
          <tr class="border-b border-gray-700 bg-gray-900/50">
            <th class="px-6 py-3 text-left text-sm font-semibold text-ui-text-dim">Metric</th>
            <th class="px-6 py-3 text-center text-sm font-semibold text-health">Match 1</th>
            <th class="px-6 py-3 text-center text-sm font-semibold text-damage">Match 2</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-700/50">
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Winner</td>
            <td class="px-6 py-3 text-center font-semibold {stats1.overview.winner === 1 ? 'text-health' : stats1.overview.winner === 2 ? 'text-damage' : 'text-gold'}">
              {stats1.overview.winner === 1 ? 'P1 Won' : stats1.overview.winner === 2 ? 'P2 Won' : 'Draw'}
            </td>
            <td class="px-6 py-3 text-center font-semibold {stats2.overview.winner === 1 ? 'text-health' : stats2.overview.winner === 2 ? 'text-damage' : 'text-gold'}">
              {stats2.overview.winner === 1 ? 'P1 Won' : stats2.overview.winner === 2 ? 'P2 Won' : 'Draw'}
            </td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Total Turns</td>
            <td class="px-6 py-3 text-center {getDiffClass(overviewCmp.turns, 1)}">{stats1.overview.totalTurns}{getDiffArrow(overviewCmp.turns, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(overviewCmp.turns, 2)}">{stats2.overview.totalTurns}{getDiffArrow(overviewCmp.turns, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Total Actions</td>
            <td class="px-6 py-3 text-center {getDiffClass(overviewCmp.actions, 1)}">{stats1.overview.totalActions}{getDiffArrow(overviewCmp.actions, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(overviewCmp.actions, 2)}">{stats2.overview.totalActions}{getDiffArrow(overviewCmp.actions, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">P1 Final Life</td>
            <td class="px-6 py-3 text-center {getDiffClass(overviewCmp.p1Life, 1)}">{stats1.overview.player1FinalLife}{getDiffArrow(overviewCmp.p1Life, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(overviewCmp.p1Life, 2)}">{stats2.overview.player1FinalLife}{getDiffArrow(overviewCmp.p1Life, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">P2 Final Life</td>
            <td class="px-6 py-3 text-center {getDiffClass(overviewCmp.p2Life, 1)}">{stats1.overview.player2FinalLife}{getDiffArrow(overviewCmp.p2Life, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(overviewCmp.p2Life, 2)}">{stats2.overview.player2FinalLife}{getDiffArrow(overviewCmp.p2Life, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Life Differential</td>
            <td class="px-6 py-3 text-center text-ui-text">{stats1.overview.lifeDifferential > 0 ? '+' : ''}{stats1.overview.lifeDifferential}</td>
            <td class="px-6 py-3 text-center text-ui-text">{stats2.overview.lifeDifferential > 0 ? '+' : ''}{stats2.overview.lifeDifferential}</td>
          </tr>
        </tbody>
      </table>

    {:else if activeTab === "combat"}
      <!-- Combat Comparison -->
      <table class="w-full">
        <thead>
          <tr class="border-b border-gray-700 bg-gray-900/50">
            <th class="px-6 py-3 text-left text-sm font-semibold text-ui-text-dim">Metric (P1)</th>
            <th class="px-6 py-3 text-center text-sm font-semibold text-health">Match 1</th>
            <th class="px-6 py-3 text-center text-sm font-semibold text-damage">Match 2</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-700/50">
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Total Damage</td>
            <td class="px-6 py-3 text-center {getDiffClass(combatP1Cmp.totalDmg, 1)}">{combatP1Stats1.totalDamageDealt}{getDiffArrow(combatP1Cmp.totalDmg, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(combatP1Cmp.totalDmg, 2)}">{combatP1Stats2.totalDamageDealt}{getDiffArrow(combatP1Cmp.totalDmg, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Face Damage</td>
            <td class="px-6 py-3 text-center {getDiffClass(combatP1Cmp.faceDmg, 1)}">{combatP1Stats1.damageToFace}{getDiffArrow(combatP1Cmp.faceDmg, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(combatP1Cmp.faceDmg, 2)}">{combatP1Stats2.damageToFace}{getDiffArrow(combatP1Cmp.faceDmg, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Creatures Killed</td>
            <td class="px-6 py-3 text-center {getDiffClass(combatP1Cmp.kills, 1)}">{combatP1Stats1.creaturesKilled}{getDiffArrow(combatP1Cmp.kills, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(combatP1Cmp.kills, 2)}">{combatP1Stats2.creaturesKilled}{getDiffArrow(combatP1Cmp.kills, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Creatures Lost</td>
            <td class="px-6 py-3 text-center {getDiffClass(combatP1Cmp.lost, 1)}">{combatP1Stats1.creaturesLost}{getDiffArrow(combatP1Cmp.lost, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(combatP1Cmp.lost, 2)}">{combatP1Stats2.creaturesLost}{getDiffArrow(combatP1Cmp.lost, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">K/D Ratio</td>
            <td class="px-6 py-3 text-center {getDiffClass(combatP1Cmp.kd, 1)}">{formatDec(combatP1Stats1.kdRatio, 2)}{getDiffArrow(combatP1Cmp.kd, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(combatP1Cmp.kd, 2)}">{formatDec(combatP1Stats2.kdRatio, 2)}{getDiffArrow(combatP1Cmp.kd, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Favorable Trades</td>
            <td class="px-6 py-3 text-center {getDiffClass(combatP1Cmp.favorable, 1)}">{combatP1Stats1.favorableTrades}{getDiffArrow(combatP1Cmp.favorable, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(combatP1Cmp.favorable, 2)}">{combatP1Stats2.favorableTrades}{getDiffArrow(combatP1Cmp.favorable, 2)}</td>
          </tr>
        </tbody>
      </table>

      <!-- P2 Combat Stats -->
      <div class="border-t border-gray-700 mt-4">
        <table class="w-full">
          <thead>
            <tr class="border-b border-gray-700 bg-gray-900/50">
              <th class="px-6 py-3 text-left text-sm font-semibold text-ui-text-dim">Metric (P2)</th>
              <th class="px-6 py-3 text-center text-sm font-semibold text-health">Match 1</th>
              <th class="px-6 py-3 text-center text-sm font-semibold text-damage">Match 2</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-700/50">
            <tr>
              <td class="px-6 py-3 text-sm text-ui-text">Total Damage</td>
              <td class="px-6 py-3 text-center {getDiffClass(combatP2Cmp.totalDmg, 1)}">{combatP2Stats1.totalDamageDealt}{getDiffArrow(combatP2Cmp.totalDmg, 1)}</td>
              <td class="px-6 py-3 text-center {getDiffClass(combatP2Cmp.totalDmg, 2)}">{combatP2Stats2.totalDamageDealt}{getDiffArrow(combatP2Cmp.totalDmg, 2)}</td>
            </tr>
            <tr>
              <td class="px-6 py-3 text-sm text-ui-text">Face Damage</td>
              <td class="px-6 py-3 text-center {getDiffClass(combatP2Cmp.faceDmg, 1)}">{combatP2Stats1.damageToFace}{getDiffArrow(combatP2Cmp.faceDmg, 1)}</td>
              <td class="px-6 py-3 text-center {getDiffClass(combatP2Cmp.faceDmg, 2)}">{combatP2Stats2.damageToFace}{getDiffArrow(combatP2Cmp.faceDmg, 2)}</td>
            </tr>
            <tr>
              <td class="px-6 py-3 text-sm text-ui-text">Creatures Killed</td>
              <td class="px-6 py-3 text-center {getDiffClass(combatP2Cmp.kills, 1)}">{combatP2Stats1.creaturesKilled}{getDiffArrow(combatP2Cmp.kills, 1)}</td>
              <td class="px-6 py-3 text-center {getDiffClass(combatP2Cmp.kills, 2)}">{combatP2Stats2.creaturesKilled}{getDiffArrow(combatP2Cmp.kills, 2)}</td>
            </tr>
            <tr>
              <td class="px-6 py-3 text-sm text-ui-text">K/D Ratio</td>
              <td class="px-6 py-3 text-center {getDiffClass(combatP2Cmp.kd, 1)}">{formatDec(combatP2Stats1.kdRatio, 2)}{getDiffArrow(combatP2Cmp.kd, 1)}</td>
              <td class="px-6 py-3 text-center {getDiffClass(combatP2Cmp.kd, 2)}">{formatDec(combatP2Stats2.kdRatio, 2)}{getDiffArrow(combatP2Cmp.kd, 2)}</td>
            </tr>
          </tbody>
        </table>
      </div>

    {:else if activeTab === "action"}
      <!-- Action Economy Comparison -->
      <table class="w-full">
        <thead>
          <tr class="border-b border-gray-700 bg-gray-900/50">
            <th class="px-6 py-3 text-left text-sm font-semibold text-ui-text-dim">Metric (P1)</th>
            <th class="px-6 py-3 text-center text-sm font-semibold text-health">Match 1</th>
            <th class="px-6 py-3 text-center text-sm font-semibold text-damage">Match 2</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-700/50">
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Cards Played</td>
            <td class="px-6 py-3 text-center {getDiffClass(actionCmp.cards, 1)}">{actionStats1.cardsPlayed}{getDiffArrow(actionCmp.cards, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(actionCmp.cards, 2)}">{actionStats2.cardsPlayed}{getDiffArrow(actionCmp.cards, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Creatures</td>
            <td class="px-6 py-3 text-center {getDiffClass(actionCmp.creatures, 1)}">{actionStats1.creaturesPlayed}{getDiffArrow(actionCmp.creatures, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(actionCmp.creatures, 2)}">{actionStats2.creaturesPlayed}{getDiffArrow(actionCmp.creatures, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Spells</td>
            <td class="px-6 py-3 text-center {getDiffClass(actionCmp.spells, 1)}">{actionStats1.spellsCast}{getDiffArrow(actionCmp.spells, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(actionCmp.spells, 2)}">{actionStats2.spellsCast}{getDiffArrow(actionCmp.spells, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Attacks Made</td>
            <td class="px-6 py-3 text-center {getDiffClass(actionCmp.attacks, 1)}">{actionStats1.attacksMade}{getDiffArrow(actionCmp.attacks, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(actionCmp.attacks, 2)}">{actionStats2.attacksMade}{getDiffArrow(actionCmp.attacks, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">AP Spent</td>
            <td class="px-6 py-3 text-center {getDiffClass(actionCmp.apSpent, 1)}">{actionStats1.apSpent}{getDiffArrow(actionCmp.apSpent, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(actionCmp.apSpent, 2)}">{actionStats2.apSpent}{getDiffArrow(actionCmp.apSpent, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">AP Efficiency</td>
            <td class="px-6 py-3 text-center {getDiffClass(actionCmp.apEff, 1)}">{formatPct(actionStats1.apEfficiency)}{getDiffArrow(actionCmp.apEff, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(actionCmp.apEff, 2)}">{formatPct(actionStats2.apEfficiency)}{getDiffArrow(actionCmp.apEff, 2)}</td>
          </tr>
        </tbody>
      </table>

    {:else if activeTab === "resources"}
      <!-- Resource Comparison -->
      <table class="w-full">
        <thead>
          <tr class="border-b border-gray-700 bg-gray-900/50">
            <th class="px-6 py-3 text-left text-sm font-semibold text-ui-text-dim">Metric (P1)</th>
            <th class="px-6 py-3 text-center text-sm font-semibold text-health">Match 1</th>
            <th class="px-6 py-3 text-center text-sm font-semibold text-damage">Match 2</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-700/50">
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Total Essence</td>
            <td class="px-6 py-3 text-center {getDiffClass(resourceCmp.essence, 1)}">{resourceStats1.totalEssenceSpent}{getDiffArrow(resourceCmp.essence, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(resourceCmp.essence, 2)}">{resourceStats2.totalEssenceSpent}{getDiffArrow(resourceCmp.essence, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Avg Essence/Turn</td>
            <td class="px-6 py-3 text-center {getDiffClass(resourceCmp.avgEssence, 1)}">{formatDec(resourceStats1.avgEssencePerTurn)}{getDiffArrow(resourceCmp.avgEssence, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(resourceCmp.avgEssence, 2)}">{formatDec(resourceStats2.avgEssencePerTurn)}{getDiffArrow(resourceCmp.avgEssence, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Cards Drawn</td>
            <td class="px-6 py-3 text-center {getDiffClass(resourceCmp.drawn, 1)}">{resourceStats1.cardsDrawn}{getDiffArrow(resourceCmp.drawn, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(resourceCmp.drawn, 2)}">{resourceStats2.cardsDrawn}{getDiffArrow(resourceCmp.drawn, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Avg Hand Size</td>
            <td class="px-6 py-3 text-center {getDiffClass(resourceCmp.avgHand, 1)}">{formatDec(resourceStats1.averageHandSize)}{getDiffArrow(resourceCmp.avgHand, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(resourceCmp.avgHand, 2)}">{formatDec(resourceStats2.averageHandSize)}{getDiffArrow(resourceCmp.avgHand, 2)}</td>
          </tr>
          <tr>
            <td class="px-6 py-3 text-sm text-ui-text">Empty Hand Turns</td>
            <td class="px-6 py-3 text-center {getDiffClass(resourceCmp.emptyHand, 1)}">{resourceStats1.emptyHandTurns}{getDiffArrow(resourceCmp.emptyHand, 1)}</td>
            <td class="px-6 py-3 text-center {getDiffClass(resourceCmp.emptyHand, 2)}">{resourceStats2.emptyHandTurns}{getDiffArrow(resourceCmp.emptyHand, 2)}</td>
          </tr>
        </tbody>
      </table>
    {/if}
  </div>

  <!-- Legend -->
  <div class="flex items-center justify-center gap-6 text-sm text-ui-text-dim">
    <span class="flex items-center gap-2">
      <span class="text-health font-bold">▲</span> Better in Match 1
    </span>
    <span class="flex items-center gap-2">
      <span class="text-damage font-bold">▲</span> Better in Match 2
    </span>
    <span class="flex items-center gap-2">
      <span class="text-ui-text-dim">▼</span> Lower value
    </span>
  </div>
</div>
