<script lang="ts">
  import { comparisonStore } from "$lib/stores/comparisonState.svelte";
  import type { ReplayInfo } from "$lib/api/types";
  import ComparisonPanel from "./ComparisonPanel.svelte";

  // Selection mode
  const isSelecting = $derived(comparisonStore.phase === "selecting");
  const isComparing = $derived(comparisonStore.phase === "comparing");
  const selectingSlot = $derived(comparisonStore.selectingSlot);

  function handleReplaySelect(replay: ReplayInfo) {
    if (selectingSlot) {
      comparisonStore.loadFromReplay(selectingSlot, replay.path);
    }
  }

  function getWinnerText(replay: ReplayInfo): string {
    if (replay.winner === 1) return "P1 Won";
    if (replay.winner === 2) return "P2 Won";
    return "Draw";
  }

  function getWinnerColor(replay: ReplayInfo): string {
    if (replay.winner === 1) return "text-health";
    if (replay.winner === 2) return "text-damage";
    return "text-gold";
  }
</script>

<div class="min-h-screen flex flex-col bg-ui-bg">
  <!-- Header -->
  <div class="bg-ui-panel border-b border-gray-700 px-6 py-4">
    <div class="flex items-center justify-between max-w-7xl mx-auto">
      <div class="flex items-center gap-3">
        <span class="text-2xl">⚖️</span>
        <h1 class="text-xl font-bold text-ui-text">Match Comparison</h1>
      </div>
      <button
        class="px-4 py-2 bg-gray-700 text-ui-text rounded-lg font-semibold text-sm
               hover:bg-gray-600 transition-colors"
        onclick={() => comparisonStore.reset()}
      >
        Close
      </button>
    </div>
  </div>

  {#if comparisonStore.error}
    <div class="max-w-7xl mx-auto w-full px-6 py-4">
      <div class="p-4 bg-damage/20 border border-damage rounded-lg text-damage flex items-center justify-between">
        <span>{comparisonStore.error}</span>
        <button
          class="text-sm underline hover:no-underline"
          onclick={() => comparisonStore.clearError()}
        >
          Dismiss
        </button>
      </div>
    </div>
  {/if}

  <!-- Content -->
  <div class="flex-1 overflow-auto">
    {#if isSelecting}
      <!-- Selection Mode -->
      <div class="max-w-7xl mx-auto p-6">
        <!-- Slot Cards -->
        <div class="grid grid-cols-2 gap-6 mb-8">
          <!-- Slot 1 -->
          <div class="bg-ui-panel rounded-xl border border-gray-700 p-6">
            <div class="flex items-center justify-between mb-4">
              <h2 class="text-lg font-semibold text-health">Match 1</h2>
              {#if comparisonStore.hasSlot1}
                <button
                  class="text-sm text-ui-text-dim hover:text-damage transition-colors"
                  onclick={() => comparisonStore.clearSlot(1)}
                >
                  Clear
                </button>
              {/if}
            </div>

            {#if comparisonStore.slot1.match}
              <div class="bg-gray-800/50 rounded-lg p-4">
                <div class="font-semibold text-ui-text mb-1">{comparisonStore.slot1.label}</div>
                <div class="text-sm text-ui-text-dim">
                  {comparisonStore.slot1.match.player1BotName} vs {comparisonStore.slot1.match.player2BotName}
                </div>
                <div class="text-sm mt-2">
                  <span class="{comparisonStore.slot1.match.result.winner === 1 ? 'text-health' : comparisonStore.slot1.match.result.winner === 2 ? 'text-damage' : 'text-gold'}">
                    {comparisonStore.slot1.match.result.winner === 1 ? 'P1 Won' : comparisonStore.slot1.match.result.winner === 2 ? 'P2 Won' : 'Draw'}
                  </span>
                  <span class="text-ui-text-dim ml-2">
                    Turn {comparisonStore.slot1.statistics?.overview.totalTurns ?? '?'}
                  </span>
                </div>
              </div>
            {:else}
              <button
                class="w-full py-8 border-2 border-dashed border-gray-600 rounded-lg text-ui-text-dim
                       hover:border-health hover:text-health transition-colors
                       {selectingSlot === 1 ? 'border-health text-health bg-health/10' : ''}"
                onclick={() => comparisonStore.selectSlot(1)}
              >
                {selectingSlot === 1 ? 'Select from list below...' : 'Click to select a match'}
              </button>
            {/if}
          </div>

          <!-- Slot 2 -->
          <div class="bg-ui-panel rounded-xl border border-gray-700 p-6">
            <div class="flex items-center justify-between mb-4">
              <h2 class="text-lg font-semibold text-damage">Match 2</h2>
              {#if comparisonStore.hasSlot2}
                <button
                  class="text-sm text-ui-text-dim hover:text-damage transition-colors"
                  onclick={() => comparisonStore.clearSlot(2)}
                >
                  Clear
                </button>
              {/if}
            </div>

            {#if comparisonStore.slot2.match}
              <div class="bg-gray-800/50 rounded-lg p-4">
                <div class="font-semibold text-ui-text mb-1">{comparisonStore.slot2.label}</div>
                <div class="text-sm text-ui-text-dim">
                  {comparisonStore.slot2.match.player1BotName} vs {comparisonStore.slot2.match.player2BotName}
                </div>
                <div class="text-sm mt-2">
                  <span class="{comparisonStore.slot2.match.result.winner === 1 ? 'text-health' : comparisonStore.slot2.match.result.winner === 2 ? 'text-damage' : 'text-gold'}">
                    {comparisonStore.slot2.match.result.winner === 1 ? 'P1 Won' : comparisonStore.slot2.match.result.winner === 2 ? 'P2 Won' : 'Draw'}
                  </span>
                  <span class="text-ui-text-dim ml-2">
                    Turn {comparisonStore.slot2.statistics?.overview.totalTurns ?? '?'}
                  </span>
                </div>
              </div>
            {:else}
              <button
                class="w-full py-8 border-2 border-dashed border-gray-600 rounded-lg text-ui-text-dim
                       hover:border-damage hover:text-damage transition-colors
                       {selectingSlot === 2 ? 'border-damage text-damage bg-damage/10' : ''}"
                onclick={() => comparisonStore.selectSlot(2)}
              >
                {selectingSlot === 2 ? 'Select from list below...' : 'Click to select a match'}
              </button>
            {/if}
          </div>
        </div>

        <!-- Compare Button -->
        {#if comparisonStore.canCompare}
          <div class="text-center mb-8">
            <button
              class="px-8 py-3 bg-ui-action text-white rounded-xl font-bold text-lg
                     hover:bg-ui-action/80 transition-all transform hover:scale-105"
              onclick={() => comparisonStore.phase = "comparing"}
            >
              Compare Matches
            </button>
          </div>
        {/if}

        <!-- Replay List for Selection -->
        {#if selectingSlot}
          <div class="bg-ui-panel rounded-xl border border-gray-700 p-6">
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-lg font-semibold text-ui-text">
                Select a Match for Slot {selectingSlot}
              </h3>
              <button
                class="text-sm text-ui-text-dim hover:text-ui-text transition-colors"
                onclick={() => comparisonStore.cancelSelection()}
              >
                Cancel
              </button>
            </div>

            {#if comparisonStore.isLoadingReplays}
              <div class="flex items-center justify-center py-8">
                <div class="w-8 h-8 border-4 border-ui-action border-t-transparent rounded-full animate-spin"></div>
              </div>
            {:else if comparisonStore.replays.length === 0}
              <div class="text-center py-8 text-ui-text-dim">
                No saved replays found. Play some matches first!
              </div>
            {:else}
              <div class="space-y-2 max-h-96 overflow-y-auto">
                {#each comparisonStore.replays as replay}
                  <button
                    class="w-full flex items-center gap-4 p-4 bg-gray-800/50 rounded-lg border border-gray-700
                           hover:border-ui-action hover:bg-gray-800 transition-colors text-left"
                    onclick={() => handleReplaySelect(replay)}
                    disabled={comparisonStore.isLoading}
                  >
                    <div class="flex-1">
                      <div class="flex items-center gap-2 mb-1">
                        <span class="font-semibold text-ui-text">{replay.player1DeckName}</span>
                        <span class="text-ui-text-dim">vs</span>
                        <span class="font-semibold text-ui-text">{replay.player2DeckName}</span>
                      </div>
                      <div class="flex items-center gap-3 text-sm">
                        <span class="text-ui-text-dim">{replay.player1Type} vs {replay.player2Type}</span>
                        <span class={getWinnerColor(replay)}>{getWinnerText(replay)}</span>
                        <span class="text-ui-text-dim">{replay.totalTurns} turns</span>
                      </div>
                      <div class="text-xs text-ui-text-dim mt-1">{replay.dateString}</div>
                    </div>
                    <div class="text-ui-action">
                      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                      </svg>
                    </div>
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {:else}
          <!-- Instructions -->
          <div class="text-center text-ui-text-dim">
            <p>Select two matches to compare their statistics side by side.</p>
            <p class="text-sm mt-2">Click on a slot above to choose from your saved replays.</p>
          </div>
        {/if}
      </div>
    {:else if isComparing && comparisonStore.slot1.statistics && comparisonStore.slot2.statistics}
      <!-- Comparison View -->
      <div class="max-w-7xl mx-auto p-6">
        <!-- Back button and swap -->
        <div class="flex items-center justify-between mb-6">
          <button
            class="px-4 py-2 bg-gray-700 text-ui-text-dim rounded-lg text-sm
                   hover:bg-gray-600 hover:text-ui-text transition-colors flex items-center gap-2"
            onclick={() => comparisonStore.backToSelection()}
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
            Change Matches
          </button>
          <button
            class="px-4 py-2 bg-gray-700 text-ui-text-dim rounded-lg text-sm
                   hover:bg-gray-600 hover:text-ui-text transition-colors flex items-center gap-2"
            onclick={() => comparisonStore.swapSlots()}
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
            </svg>
            Swap
          </button>
        </div>

        <!-- Match Headers -->
        <div class="grid grid-cols-2 gap-6 mb-6">
          <div class="bg-health/10 border border-health/30 rounded-xl p-4">
            <div class="font-bold text-health text-lg mb-1">{comparisonStore.slot1.label}</div>
            <div class="text-sm text-ui-text-dim">
              {comparisonStore.slot1.match?.player1BotName} vs {comparisonStore.slot1.match?.player2BotName}
            </div>
            <div class="mt-2">
              <span class="{comparisonStore.slot1.match?.result.winner === 1 ? 'text-health' : comparisonStore.slot1.match?.result.winner === 2 ? 'text-damage' : 'text-gold'} font-semibold">
                {comparisonStore.slot1.match?.result.winner === 1 ? 'P1 Won' : comparisonStore.slot1.match?.result.winner === 2 ? 'P2 Won' : 'Draw'}
              </span>
              <span class="text-ui-text-dim ml-2">
                in {comparisonStore.slot1.statistics.overview.totalTurns} turns
              </span>
            </div>
          </div>
          <div class="bg-damage/10 border border-damage/30 rounded-xl p-4">
            <div class="font-bold text-damage text-lg mb-1">{comparisonStore.slot2.label}</div>
            <div class="text-sm text-ui-text-dim">
              {comparisonStore.slot2.match?.player1BotName} vs {comparisonStore.slot2.match?.player2BotName}
            </div>
            <div class="mt-2">
              <span class="{comparisonStore.slot2.match?.result.winner === 1 ? 'text-health' : comparisonStore.slot2.match?.result.winner === 2 ? 'text-damage' : 'text-gold'} font-semibold">
                {comparisonStore.slot2.match?.result.winner === 1 ? 'P1 Won' : comparisonStore.slot2.match?.result.winner === 2 ? 'P2 Won' : 'Draw'}
              </span>
              <span class="text-ui-text-dim ml-2">
                in {comparisonStore.slot2.statistics.overview.totalTurns} turns
              </span>
            </div>
          </div>
        </div>

        <!-- Comparison Panels -->
        <ComparisonPanel
          stats1={comparisonStore.slot1.statistics}
          stats2={comparisonStore.slot2.statistics}
        />
      </div>
    {/if}
  </div>
</div>
