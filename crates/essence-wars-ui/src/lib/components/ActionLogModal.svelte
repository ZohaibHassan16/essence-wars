<script lang="ts">
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import type { SpectatorAction } from "$lib/api/types";
  import type { CommentaryEntry } from "$lib/commentary/types";

  const match = $derived(spectatorStore.match);
  const currentActionIndex = $derived(spectatorStore.currentActionIndex);
  const commentaryHistory = $derived(spectatorStore.commentaryHistory);

  // Get all actions (full log) or up to current position based on toggle
  let showFullLog = $state(true);

  // Export feedback toast
  let exportToast = $state<{ message: string; type: "success" | "error" } | null>(null);
  let toastTimeout: ReturnType<typeof setTimeout> | null = null;

  function showToast(message: string, type: "success" | "error" = "success") {
    if (toastTimeout) clearTimeout(toastTimeout);
    exportToast = { message, type };
    toastTimeout = setTimeout(() => {
      exportToast = null;
    }, 3000);
  }
  const actions = $derived(
    showFullLog
      ? (match?.actions ?? [])
      : (match?.actions.slice(0, currentActionIndex + 1) ?? [])
  );

  // Create a map of action index -> commentary entries
  const commentaryByActionIndex = $derived(() => {
    const map: Record<number, CommentaryEntry[]> = {};
    for (const entry of commentaryHistory) {
      // Match commentary to the action that generated it
      // Commentary entries are indexed by their order in history
      const historyIndex = commentaryHistory.indexOf(entry);
      if (!map[historyIndex]) {
        map[historyIndex] = [];
      }
      map[historyIndex].push(entry);
    }
    return map;
  });

  // Group actions by turn (returns array of [turn, actions] for iteration)
  const actionsByTurn = $derived(() => {
    const grouped: [number, SpectatorAction[]][] = [];
    const turnMap: Record<number, SpectatorAction[]> = {};

    for (const action of actions) {
      const turn = action.turn;
      if (!turnMap[turn]) {
        turnMap[turn] = [];
        grouped.push([turn, turnMap[turn]]);
      }
      turnMap[turn].push(action);
    }
    return grouped;
  });

  // Get commentary for an action (by matching action index to commentary history index)
  function getCommentaryForAction(actionIndex: number): CommentaryEntry | undefined {
    // Commentary history is indexed by action - find entry matching this action
    return commentaryHistory.find((_, i) => i === actionIndex);
  }

  function getActionIcon(actionType: string): string {
    switch (actionType) {
      case "play_card": return "🃏";
      case "attack": return "⚔️";
      case "use_ability": return "✨";
      case "commander_insight": return "💡";
      case "end_turn": return "⏭️";
      default: return "•";
    }
  }

  function getPlayerBadgeClass(player: number): string {
    return player === 1
      ? "bg-health/20 text-health"
      : "bg-damage/20 text-damage";
  }

  function getMomentTypeIcon(momentType: string | undefined): string {
    switch (momentType) {
      case "game_start": return "🎬";
      case "first_blood": return "🩸";
      case "board_swing": return "⚡";
      case "commander_played": return "👑";
      case "lethal_threat": return "💀";
      case "game_end": return "🏆";
      default: return "💬";
    }
  }

  function handleClose() {
    spectatorStore.closeActionLog();
  }

  function jumpToAction(index: number) {
    spectatorStore.jumpToAction(index);
  }

  // Export functions
  async function exportAsText(toClipboard = false) {
    if (!match) return;

    let output = `# Essence Wars Match Log\n`;
    output += `${match.player1DeckName} (${match.player1BotName}) vs ${match.player2DeckName} (${match.player2BotName})\n`;
    output += `Result: ${match.result.winner === 1 ? "P1 Wins" : match.result.winner === 2 ? "P2 Wins" : "Draw"}\n`;
    output += `Total Actions: ${match.actions.length}\n\n`;

    for (const [turn, turnActions] of actionsByTurn()) {
      output += `== Turn ${turn} ==\n`;
      for (const action of turnActions) {
        const actionIdx = match.actions.indexOf(action);
        const commentary = getCommentaryForAction(actionIdx);
        output += `  [P${action.player}] ${action.action.description}`;
        if (action.thinkingTimeMs) output += ` (${action.thinkingTimeMs}ms)`;
        output += `\n`;
        if (action.events.length > 0) {
          output += `    Events: ${action.events.map(e => e.eventType.replace(/_/g, " ")).join(", ")}\n`;
        }
        if (commentary) {
          output += `    💬 ${commentary.text}\n`;
          if (commentary.analysis) output += `    📊 ${commentary.analysis}\n`;
        }
      }
      output += `\n`;
    }

    if (toClipboard) {
      const success = await copyToClipboard(output);
      showToast(success ? "Copied to clipboard!" : "Failed to copy", success ? "success" : "error");
    } else {
      const success = downloadFile(output, `match-log-${Date.now()}.txt`, "text/plain");
      showToast(success ? "Downloaded match-log.txt" : "Download failed", success ? "success" : "error");
    }
  }

  function exportAsJson() {
    if (!match) return;

    const exportData = {
      metadata: {
        player1Deck: match.player1DeckName,
        player1Bot: match.player1BotName,
        player2Deck: match.player2DeckName,
        player2Bot: match.player2BotName,
        result: match.result,
        totalActions: match.actions.length,
        exportedAt: new Date().toISOString(),
      },
      actions: match.actions.map((action, idx) => ({
        index: action.action.index,
        turn: action.turn,
        player: action.player,
        type: action.action.actionType,
        description: action.action.description,
        thinkingTimeMs: action.thinkingTimeMs,
        events: action.events.map(e => ({
          type: e.eventType,
          data: e.data,
        })),
        insights: action.insights ? {
          confidence: action.insights.confidence,
          confidenceLevel: action.insights.confidenceLevel,
          algorithm: action.insights.searchStats.algorithm,
          topMoves: action.insights.moveScores.slice(0, 5).map(m => ({
            action: m.action.description,
            probability: m.probability,
            score: m.score,
            isChosen: m.isChosen,
          })),
        } : null,
        commentary: getCommentaryForAction(idx) ?? null,
      })),
      evalHistory: match.evalHistory,
    };

    const success = downloadFile(
      JSON.stringify(exportData, null, 2),
      `match-log-${Date.now()}.json`,
      "application/json"
    );
    showToast(success ? "Downloaded match-log.json" : "Download failed", success ? "success" : "error");
  }

  function downloadFile(content: string, filename: string, mimeType: string): boolean {
    try {
      const blob = new Blob([content], { type: mimeType });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      return true;
    } catch (e) {
      console.error("Download failed:", e);
      return false;
    }
  }

  async function copyToClipboard(content: string): Promise<boolean> {
    try {
      await navigator.clipboard.writeText(content);
      return true;
    } catch (e) {
      console.error("Clipboard copy failed:", e);
      return false;
    }
  }

  // Keyboard handler for closing
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      handleClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if spectatorStore.showActionLog}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4"
    onclick={handleClose}
    onkeydown={(e) => e.key === "Enter" && handleClose()}
    role="button"
    tabindex="0"
  >
    <!-- Modal -->
    <div
      class="bg-ui-panel border border-gray-700 rounded-lg shadow-2xl w-full max-w-3xl max-h-[80vh] flex flex-col relative"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="action-log-title"
      tabindex="-1"
    >
      <!-- Toast notification -->
      {#if exportToast}
        <div
          class="absolute top-4 right-4 z-10 px-4 py-2 rounded-lg shadow-lg animate-fade-in
                 {exportToast.type === 'success' ? 'bg-health/90 text-white' : 'bg-damage/90 text-white'}"
        >
          <div class="flex items-center gap-2">
            <span>{exportToast.type === 'success' ? '✓' : '✗'}</span>
            <span class="text-sm font-medium">{exportToast.message}</span>
          </div>
        </div>
      {/if}
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-gray-700">
        <div class="flex items-center gap-3">
          <span class="text-lg">📋</span>
          <h2 id="action-log-title" class="text-lg font-semibold text-ui-text">Action Log</h2>
          <span class="text-sm text-ui-text-dim">
            {actions.length} actions
          </span>
          <!-- Toggle for full log vs current position -->
          <button
            class="text-xs px-2 py-0.5 rounded transition-colors
                   {showFullLog
                     ? 'bg-ui-action/20 text-ui-action border border-ui-action/50'
                     : 'bg-gray-700 text-ui-text-dim border border-gray-600'}"
            onclick={() => showFullLog = !showFullLog}
          >
            {showFullLog ? 'Full Log' : 'Up to Current'}
          </button>
        </div>
        <button
          class="text-ui-text-dim hover:text-ui-text transition-colors p-1"
          onclick={handleClose}
          aria-label="Close"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-4">
        {#if actions.length === 0}
          <div class="text-center text-ui-text-dim py-8">
            No actions yet. Press play to start the replay.
          </div>
        {:else}
          <div class="space-y-4">
            {#each actionsByTurn().toReversed() as [turn, turnActions] (turn)}
              <div class="space-y-1">
                <!-- Turn header -->
                <div class="flex items-center gap-2 mb-2">
                  <div class="h-px flex-1 bg-gray-700"></div>
                  <span class="text-xs font-semibold text-ui-text-dim px-2">Turn {turn}</span>
                  <div class="h-px flex-1 bg-gray-700"></div>
                </div>

                <!-- Actions in this turn -->
                {#each [...turnActions].reverse() as action, i (action.action.index)}
                  {@const actionIdx = match?.actions.indexOf(action) ?? -1}
                  {@const isCurrentAction = actionIdx === currentActionIndex}
                  {@const commentary = getCommentaryForAction(actionIdx)}
                  <button
                    class="w-full text-left p-3 rounded-lg transition-colors
                           {isCurrentAction
                             ? 'bg-ui-action/20 border border-ui-action/50'
                             : 'bg-ui-bg/50 hover:bg-ui-bg/80 border border-transparent'}"
                    onclick={() => jumpToAction(actionIdx)}
                  >
                    <div class="flex items-start gap-3">
                      <!-- Icon -->
                      <span class="text-lg">{getActionIcon(action.action.actionType)}</span>

                      <!-- Content -->
                      <div class="flex-1 min-w-0">
                        <div class="flex items-center gap-2">
                          <span class="px-1.5 py-0.5 rounded text-xs font-medium {getPlayerBadgeClass(action.player)}">
                            P{action.player}
                          </span>
                          <span class="text-sm text-ui-text font-medium">
                            {action.action.description}
                          </span>
                        </div>

                        <!-- Additional details -->
                        <div class="flex items-center gap-3 mt-1 text-xs text-ui-text-dim">
                          <span>Action #{action.action.index}</span>
                          {#if action.thinkingTimeMs}
                            <span>{action.thinkingTimeMs}ms</span>
                          {/if}
                          {#if action.insights}
                            <span class="text-purple-400">
                              {Math.round(action.insights.confidence * 100)}% confidence
                            </span>
                          {/if}
                        </div>

                        <!-- Events summary -->
                        {#if action.events.length > 0}
                          <div class="flex flex-wrap gap-1 mt-2">
                            {#each action.events.slice(0, 5) as event (event.eventType + JSON.stringify(event.data))}
                              <span class="px-1.5 py-0.5 rounded text-xs bg-gray-700 text-ui-text-dim">
                                {event.eventType.replace(/_/g, " ")}
                              </span>
                            {/each}
                            {#if action.events.length > 5}
                              <span class="px-1.5 py-0.5 rounded text-xs bg-gray-700 text-ui-text-dim">
                                +{action.events.length - 5} more
                              </span>
                            {/if}
                          </div>
                        {/if}

                        <!-- Commentary (if available) -->
                        {#if commentary}
                          <div class="mt-2 p-2 rounded bg-purple-900/30 border border-purple-700/50">
                            <div class="flex items-start gap-2">
                              <span class="text-sm">{getMomentTypeIcon(commentary.momentType)}</span>
                              <div class="flex-1">
                                <p class="text-xs text-ui-text leading-relaxed">{commentary.text}</p>
                                {#if commentary.analysis}
                                  <p class="text-xs text-ui-text-dim mt-1 italic">{commentary.analysis}</p>
                                {/if}
                                <div class="flex items-center gap-3 mt-1 text-xs text-ui-text-dim">
                                  {#if commentary.winProbability !== undefined}
                                    <span>Win: <span class="text-purple-400">{Math.round(commentary.winProbability * 100)}%</span></span>
                                  {/if}
                                  {#if commentary.boardAdvantage !== undefined}
                                    <span class="{commentary.boardAdvantage > 0 ? 'text-health' : commentary.boardAdvantage < 0 ? 'text-damage' : 'text-ui-text-dim'}">
                                      Adv: {commentary.boardAdvantage > 0 ? '+' : ''}{commentary.boardAdvantage}
                                    </span>
                                  {/if}
                                  {#if commentary.isKeyMoment}
                                    <span class="text-gold">Key Moment</span>
                                  {/if}
                                </div>
                              </div>
                            </div>
                          </div>
                        {/if}
                      </div>

                      <!-- Current indicator -->
                      {#if isCurrentAction}
                        <span class="text-ui-action text-xs font-medium">Current</span>
                      {/if}
                    </div>
                  </button>
                {/each}
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-4 py-3 border-t border-gray-700 flex items-center justify-between">
        <div class="text-xs text-ui-text-dim">
          Click an action to jump to that point in the replay
        </div>
        <div class="flex items-center gap-2">
          <!-- Export buttons -->
          <div class="flex items-center gap-1 mr-2">
            <button
              class="px-3 py-1.5 text-sm bg-gray-600/50 text-ui-text-dim rounded-lg
                     hover:bg-gray-600 hover:text-ui-text transition-colors
                     border border-gray-500/50"
              onclick={() => exportAsText(true)}
              title="Copy as plain text to clipboard"
            >
              📋 Copy
            </button>
            <button
              class="px-3 py-1.5 text-sm bg-purple-600/20 text-purple-400 rounded-lg
                     hover:bg-purple-600 hover:text-white transition-colors
                     border border-purple-500/50"
              onclick={() => exportAsText(false)}
              title="Download as plain text file"
            >
              📄 Text
            </button>
            <button
              class="px-3 py-1.5 text-sm bg-blue-600/20 text-blue-400 rounded-lg
                     hover:bg-blue-600 hover:text-white transition-colors
                     border border-blue-500/50"
              onclick={exportAsJson}
              title="Download as JSON (includes AI insights)"
            >
              📊 JSON
            </button>
          </div>
          <button
            class="px-4 py-2 bg-gray-700 text-ui-text rounded-lg hover:bg-gray-600 transition-colors"
            onclick={handleClose}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  :global(.animate-fade-in) {
    animation: fade-in 0.2s ease-out;
  }
</style>
