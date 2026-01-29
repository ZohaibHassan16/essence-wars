<script lang="ts">
  import type { MatchStatistics } from "$lib/stats/types";

  interface Props {
    statistics: MatchStatistics;
  }

  let { statistics }: Props = $props();

  const ai = $derived(statistics.aiAnalysis);

  function formatMs(ms: number): string {
    if (ms < 1000) return `${Math.round(ms)}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  }

  function formatPercent(value: number): string {
    return `${Math.round(value * 100)}%`;
  }
</script>

<div class="space-y-6">
  <h3 class="text-lg font-bold text-ui-text">AI Analysis</h3>

  {#if !ai}
    <div class="bg-gray-800/50 rounded-lg p-8 text-center border border-gray-700">
      <div class="text-4xl mb-3">🤖</div>
      <div class="text-ui-text">No AI analysis data available.</div>
      <div class="text-sm text-ui-text-dim mt-2">
        AI analysis requires MCTS bots with thinking data enabled.
      </div>
    </div>
  {:else}
    <!-- MCTS Summary -->
    <div class="grid grid-cols-4 gap-4">
      <div class="bg-gray-800/50 rounded-lg p-4 text-center border border-gray-700">
        <div class="text-2xl font-bold text-ui-text">{ai.totalSimulations.toLocaleString()}</div>
        <div class="text-sm text-ui-text-dim">Total Simulations</div>
      </div>
      <div class="bg-gray-800/50 rounded-lg p-4 text-center border border-gray-700">
        <div class="text-2xl font-bold text-ui-text">{Math.round(ai.avgSimulationsPerMove).toLocaleString()}</div>
        <div class="text-sm text-ui-text-dim">Avg Sims/Move</div>
      </div>
      <div class="bg-gray-800/50 rounded-lg p-4 text-center border border-gray-700">
        <div class="text-2xl font-bold text-ui-text">{formatMs(ai.avgThinkingTimeMs)}</div>
        <div class="text-sm text-ui-text-dim">Avg Think Time</div>
      </div>
      <div class="bg-gray-800/50 rounded-lg p-4 text-center border border-gray-700">
        <div class="text-2xl font-bold text-ui-text">{formatMs(ai.maxThinkingTimeMs)}</div>
        <div class="text-sm text-ui-text-dim">Max Think Time</div>
      </div>
    </div>

    <!-- Win Probability Journey -->
    <div class="bg-gray-800/50 rounded-lg p-5 border border-gray-700">
      <h4 class="text-sm font-semibold text-ui-text-dim mb-4">Win Probability Journey (P1 Perspective)</h4>
      <div class="flex items-center justify-between mb-4">
        <div class="text-center">
          <div class="text-sm text-ui-text-dim mb-1">Start</div>
          <div class="text-2xl font-bold {ai.startingWinProb >= 0.5 ? 'text-health' : 'text-damage'}">
            {formatPercent(ai.startingWinProb)}
          </div>
        </div>
        <div class="flex-1 flex items-center justify-center px-4">
          <div class="h-2 flex-1 bg-gray-700 rounded-full relative">
            <!-- Starting position marker -->
            <div
              class="absolute top-1/2 -translate-y-1/2 w-3 h-3 rounded-full bg-gray-400 border-2 border-white"
              style="left: {ai.startingWinProb * 100}%"
            ></div>
            <!-- Ending position marker -->
            <div
              class="absolute top-1/2 -translate-y-1/2 w-3 h-3 rounded-full border-2 border-white
                     {ai.finalWinProb >= 0.5 ? 'bg-health' : 'bg-damage'}"
              style="left: {ai.finalWinProb * 100}%"
            ></div>
            <!-- 50% line -->
            <div class="absolute top-0 bottom-0 left-1/2 w-px bg-gray-500"></div>
          </div>
        </div>
        <div class="text-center">
          <div class="text-sm text-ui-text-dim mb-1">End</div>
          <div class="text-2xl font-bold {ai.finalWinProb >= 0.5 ? 'text-health' : 'text-damage'}">
            {formatPercent(ai.finalWinProb)}
          </div>
        </div>
      </div>
      <div class="flex items-center justify-center gap-6 text-sm">
        <div>
          <span class="text-ui-text-dim">Max Swing:</span>
          <span class="font-semibold text-gold ml-1">{formatPercent(ai.maxWinProbSwing)}</span>
        </div>
        <div>
          <span class="text-ui-text-dim">Avg Confidence:</span>
          <span class="font-semibold text-ui-text ml-1">{formatPercent(ai.avgMoveConfidence)}</span>
        </div>
      </div>
    </div>

    <!-- Critical Moments -->
    <div class="bg-gray-800/50 rounded-lg border border-gray-700 overflow-hidden">
      <div class="px-4 py-3 border-b border-gray-700 bg-gray-900/30 flex items-center justify-between">
        <h4 class="text-sm font-semibold text-ui-text">Critical Moments</h4>
        <span class="text-xs text-ui-text-dim">{ai.criticalMoments.length} moments</span>
      </div>

      {#if ai.criticalMoments.length === 0}
        <div class="p-6 text-center text-ui-text-dim">
          No critical moments detected (swings &lt; 15%).
        </div>
      {:else}
        <div class="max-h-[300px] overflow-auto">
          <table class="w-full">
            <thead class="sticky top-0 bg-gray-800">
              <tr class="border-b border-gray-700">
                <th class="px-4 py-2 text-left text-xs font-semibold text-ui-text-dim">Turn</th>
                <th class="px-4 py-2 text-left text-xs font-semibold text-ui-text-dim">Player</th>
                <th class="px-4 py-2 text-left text-xs font-semibold text-ui-text-dim">Action</th>
                <th class="px-4 py-2 text-center text-xs font-semibold text-ui-text-dim">Before</th>
                <th class="px-4 py-2 text-center text-xs font-semibold text-ui-text-dim">After</th>
                <th class="px-4 py-2 text-center text-xs font-semibold text-ui-text-dim">Swing</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-700/50">
              {#each ai.criticalMoments as moment}
                <tr class="hover:bg-gray-700/30">
                  <td class="px-4 py-2 text-sm font-medium text-ui-text">{moment.turn}</td>
                  <td class="px-4 py-2">
                    <span class="px-2 py-0.5 rounded text-xs font-medium
                                 {moment.player === 1 ? 'bg-health/20 text-health' : 'bg-damage/20 text-damage'}">
                      P{moment.player}
                    </span>
                  </td>
                  <td class="px-4 py-2 text-sm text-ui-text max-w-[200px] truncate" title={moment.actionDescription}>
                    {moment.actionDescription}
                  </td>
                  <td class="px-4 py-2 text-center text-sm {moment.winProbBefore >= 0.5 ? 'text-health' : 'text-damage'}">
                    {formatPercent(moment.winProbBefore)}
                  </td>
                  <td class="px-4 py-2 text-center text-sm {moment.winProbAfter >= 0.5 ? 'text-health' : 'text-damage'}">
                    {formatPercent(moment.winProbAfter)}
                  </td>
                  <td class="px-4 py-2 text-center">
                    <span class="px-2 py-0.5 rounded text-xs font-bold
                                 {moment.swing > 0 ? 'bg-health/20 text-health' : 'bg-damage/20 text-damage'}">
                      {moment.swing > 0 ? '+' : ''}{formatPercent(moment.swing)}
                    </span>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>

    <!-- AI Insight -->
    <div class="bg-gradient-to-r from-purple-900/20 to-gray-800/50 rounded-lg p-4 border border-purple-700/30">
      <div class="flex items-start gap-3">
        <span class="text-2xl">💡</span>
        <div>
          <div class="text-sm font-semibold text-purple-400 mb-1">AI Insight</div>
          <div class="text-sm text-ui-text">
            {#if ai.finalWinProb > 0.9}
              The game was decisively won, with the final position showing overwhelming advantage.
            {:else if ai.finalWinProb > 0.7}
              Clear advantage was established by the end of the game.
            {:else if ai.finalWinProb > 0.55}
              A close but controlled victory - marginal edges accumulated over time.
            {:else if ai.finalWinProb >= 0.45}
              An extremely close game that could have gone either way.
            {:else if ai.finalWinProb >= 0.3}
              The game swung against expectations - comeback potential was there but not realized.
            {:else}
              A dominant performance by the winner, shutting down options early.
            {/if}
            {#if ai.criticalMoments.length > 3}
              This was a volatile match with {ai.criticalMoments.length} critical turning points.
            {:else if ai.criticalMoments.length > 0}
              Key moments at turn{ai.criticalMoments.length > 1 ? 's' : ''} {ai.criticalMoments.map(m => m.turn).join(', ')} shaped the outcome.
            {:else}
              The game progressed steadily without major swings.
            {/if}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
