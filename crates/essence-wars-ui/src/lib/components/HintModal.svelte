<script lang="ts">
  import type { AiHintResponse, ActionInfo } from "$lib/api/types";
  import { gameStore } from "$lib/stores/gameState.svelte";
  import { playSound } from "$lib/audio";

  const hint = $derived(gameStore.currentHint);
  const isLoading = $derived(gameStore.isHintLoading);

  function formatActionDescription(action: ActionInfo): string {
    switch (action.actionType) {
      case "play_card":
        return `Play card ${action.handIndex !== undefined ? `#${action.handIndex + 1}` : ""} to slot ${action.targetSlot}`;
      case "attack":
        return `Attack with slot ${action.sourceSlot} → slot ${action.targetSlot}`;
      case "use_ability":
        return `Use ability from slot ${action.sourceSlot}${action.targetSlot !== undefined ? ` on slot ${action.targetSlot}` : ""}`;
      case "end_turn":
        return "End turn";
      case "commander_insight":
        return "Use Commander's Insight";
      default:
        return action.description;
    }
  }

  function getActionIcon(actionType: string): string {
    switch (actionType) {
      case "play_card": return "🃏";
      case "attack": return "⚔️";
      case "use_ability": return "✨";
      case "end_turn": return "⏭️";
      case "commander_insight": return "💡";
      default: return "❓";
    }
  }

  function formatScore(score: number): string {
    if (score > 0) return `+${score.toFixed(1)}`;
    return score.toFixed(1);
  }

  function handleClose() {
    gameStore.closeHintModal();
  }

  function handleRequestHint() {
    gameStore.requestHint();
  }

  function handleApplyHint(action: ActionInfo) {
    playSound('buttonClick');
    gameStore.applyHint(action);
    handleClose();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      handleClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if gameStore.showHintModal}
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
      class="bg-ui-panel border border-gray-700 rounded-lg shadow-2xl w-full max-w-md flex flex-col"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="hint-modal-title"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-gray-700">
        <div class="flex items-center gap-3">
          <span class="text-lg">💡</span>
          <h2 id="hint-modal-title" class="text-lg font-semibold text-ui-text">AI Hint</h2>
        </div>
        <div class="flex items-center gap-2">
          <button
            class="px-3 py-1.5 text-sm rounded-lg transition-colors
                   bg-purple-600 text-white hover:bg-purple-500
                   disabled:opacity-50 disabled:cursor-not-allowed
                   flex items-center gap-2"
            onclick={handleRequestHint}
            disabled={isLoading}
          >
            {#if isLoading}
              <span class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
              Thinking...
            {:else}
              Get Hint
            {/if}
          </button>
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
      </div>

      <!-- Content -->
      <div class="p-4">
        {#if hint}
          <div class="space-y-4">
            <!-- Recommended action -->
            <div>
              <div class="text-xs text-ui-text-dim uppercase tracking-wide mb-2">Recommended</div>
              <button
                class="w-full p-3 rounded-lg transition-colors text-left
                       bg-purple-600/20 border border-purple-500/50
                       hover:bg-purple-600/30 hover:border-purple-500"
                onclick={() => handleApplyHint(hint.recommendedAction)}
              >
                <div class="flex items-center gap-3">
                  <span class="text-xl">{getActionIcon(hint.recommendedAction.actionType)}</span>
                  <div class="flex-1">
                    <div class="text-ui-text font-medium">
                      {formatActionDescription(hint.recommendedAction)}
                    </div>
                    <div class="text-xs text-ui-text-dim mt-0.5">
                      Click to apply this action
                    </div>
                  </div>
                  <div class="px-2 py-1 rounded bg-health/20 text-health text-sm font-mono">
                    {formatScore(hint.score)}
                  </div>
                </div>
              </button>
            </div>

            <!-- Alternatives -->
            {#if hint.alternatives.length > 0}
              <div>
                <div class="text-xs text-ui-text-dim uppercase tracking-wide mb-2">Alternatives</div>
                <div class="space-y-2">
                  {#each hint.alternatives.slice(0, 4) as alt, i (i)}
                    <button
                      class="w-full p-2.5 rounded-lg transition-colors text-left
                             bg-ui-bg/50 border border-gray-700
                             hover:bg-ui-bg hover:border-gray-600"
                      onclick={() => handleApplyHint(alt.action)}
                    >
                      <div class="flex items-center gap-3">
                        <span class="text-lg">{getActionIcon(alt.action.actionType)}</span>
                        <div class="flex-1">
                          <div class="text-sm text-ui-text">
                            {formatActionDescription(alt.action)}
                          </div>
                        </div>
                        <div class="text-right">
                          <div class="text-sm font-mono {alt.score > 0 ? 'text-health' : alt.score < 0 ? 'text-damage' : 'text-ui-text-dim'}">
                            {formatScore(alt.score)}
                          </div>
                          <div class="text-xs font-mono {alt.scoreDelta < 0 ? 'text-damage' : 'text-ui-text-dim'}">
                            ({formatScore(alt.scoreDelta)})
                          </div>
                        </div>
                      </div>
                    </button>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Meta info -->
            <div class="text-xs text-ui-text-dim text-right">
              Computed in {hint.thinkingTimeMs}ms
            </div>
          </div>
        {:else if isLoading}
          <div class="flex flex-col items-center justify-center py-8">
            <div class="w-8 h-8 border-3 border-purple-500/30 border-t-purple-500 rounded-full animate-spin mb-3"></div>
            <div class="text-ui-text-dim">Analyzing position...</div>
          </div>
        {:else}
          <div class="text-center py-8">
            <div class="text-4xl mb-3">🤔</div>
            <div class="text-ui-text-dim mb-4">
              Need help deciding your next move?
            </div>
            <button
              class="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-500 transition-colors"
              onclick={handleRequestHint}
            >
              Get AI Suggestion
            </button>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-4 py-3 border-t border-gray-700 flex items-center justify-between">
        <div class="text-xs text-ui-text-dim">
          Click an action to apply it automatically
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
