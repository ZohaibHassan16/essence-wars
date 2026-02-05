<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";

  const actionHistory = $derived(gameStore.actionHistory);
  const currentTurn = $derived(gameStore.gameState?.turn ?? 1);

  // Group actions by turn
  const actionsByTurn = $derived(() => {
    const grouped: [number, typeof actionHistory][] = [];
    const turnMap: Record<number, typeof actionHistory> = {};

    for (const action of actionHistory) {
      const turn = action.turn;
      if (!turnMap[turn]) {
        turnMap[turn] = [];
        grouped.push([turn, turnMap[turn]]);
      }
      turnMap[turn].push(action);
    }
    return grouped;
  });

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

  function handleClose() {
    gameStore.closeActionLogModal();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      handleClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if gameStore.showActionLogModal}
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
      class="bg-ui-panel border border-gray-700 rounded-lg shadow-2xl w-full max-w-2xl max-h-[70vh] flex flex-col"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="action-log-title"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-gray-700">
        <div class="flex items-center gap-3">
          <span class="text-lg">📋</span>
          <h2 id="action-log-title" class="text-lg font-semibold text-ui-text">Action Log</h2>
          <span class="text-sm text-ui-text-dim">
            {actionHistory.length} actions • Turn {currentTurn}
          </span>
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
        {#if actionHistory.length === 0}
          <div class="text-center text-ui-text-dim py-8">
            <div class="text-4xl mb-3">📝</div>
            <div>No actions yet. Make your first move!</div>
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
                {#each [...turnActions].reverse() as action (action.index)}
                  <div
                    class="p-3 rounded-lg bg-ui-bg/50 border border-transparent
                           hover:bg-ui-bg/80 transition-colors"
                  >
                    <div class="flex items-start gap-3">
                      <!-- Icon -->
                      <span class="text-lg">{getActionIcon(action.actionType)}</span>

                      <!-- Content -->
                      <div class="flex-1 min-w-0">
                        <div class="flex items-center gap-2">
                          <span class="px-1.5 py-0.5 rounded text-xs font-medium {getPlayerBadgeClass(action.player)}">
                            {action.player === 1 ? "You" : "AI"}
                          </span>
                          <span class="text-sm text-ui-text font-medium">
                            {action.description}
                          </span>
                        </div>

                        <!-- Additional details -->
                        <div class="flex items-center gap-3 mt-1 text-xs text-ui-text-dim">
                          <span>Action #{action.index}</span>
                          <span class="capitalize">{action.actionType.replace(/_/g, " ")}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                {/each}
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-4 py-3 border-t border-gray-700 flex items-center justify-between">
        <div class="text-xs text-ui-text-dim">
          Review the game's action history
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
{/if}
