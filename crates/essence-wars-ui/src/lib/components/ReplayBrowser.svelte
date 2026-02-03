<script lang="ts">
  import { replayStore } from "$lib/stores/replayState.svelte";
  import { comparisonStore } from "$lib/stores/comparisonState.svelte";
  import type { ReplayInfo } from "$lib/api/types";

  let sortBy = $state<"date" | "turns" | "deck">("date");
  let confirmDeletePath = $state<string | null>(null);

  const sortedReplays = $derived(() => {
    const replays = [...replayStore.replays];
    switch (sortBy) {
      case "date":
        return replays.sort((a, b) => b.timestamp - a.timestamp);
      case "turns":
        return replays.sort((a, b) => b.totalTurns - a.totalTurns);
      case "deck":
        return replays.sort((a, b) =>
          a.player1DeckName.localeCompare(b.player1DeckName)
        );
      default:
        return replays;
    }
  });

  function getWinnerText(replay: ReplayInfo): string {
    if (replay.winner === 1) return `${replay.player1Type} Won`;
    if (replay.winner === 2) return `${replay.player2Type} Won`;
    return "Draw";
  }

  function getWinnerColor(replay: ReplayInfo): string {
    if (replay.winner === 1) return "text-health";
    if (replay.winner === 2) return "text-damage";
    return "text-ui-text-dim";
  }

  async function handleDelete(path: string) {
    await replayStore.deleteReplay(path);
    confirmDeletePath = null;
  }

  function goBack() {
    replayStore.reset();
  }

  async function startComparison() {
    await comparisonStore.startComparison();
  }
</script>

<div class="min-h-screen flex flex-col items-center justify-center p-8">
  <h1 class="text-4xl font-bold text-ui-text mb-2">Saved Replays</h1>
  <p class="text-ui-text-dim mb-8">Watch your past games</p>

  <div class="max-w-4xl w-full bg-ui-panel rounded-xl p-8 shadow-2xl">
    <!-- Compare button at top -->
    {#if replayStore.replays.length >= 2}
      <div class="mb-6 flex justify-end">
        <button
          class="px-4 py-2 bg-purple-600/20 text-purple-400 rounded-lg font-semibold text-sm
                 border border-purple-500/50 hover:bg-purple-600 hover:text-white transition-colors flex items-center gap-2"
          onclick={startComparison}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
          </svg>
          Compare Matches
        </button>
      </div>
    {/if}
    {#if replayStore.error}
      <div class="mb-4 p-4 bg-damage/20 border border-damage rounded text-damage">
        {replayStore.error}
      </div>
    {/if}

    {#if replayStore.isLoading}
      <div class="flex flex-col items-center justify-center py-12">
        <div class="w-12 h-12 border-4 border-ui-action border-t-transparent rounded-full animate-spin mb-4"></div>
        <p class="text-ui-text-dim">Loading replays...</p>
      </div>
    {:else if replayStore.replays.length === 0}
      <div class="text-center py-12">
        <p class="text-xl text-ui-text-dim mb-4">No saved replays yet</p>
        <p class="text-sm text-ui-text-dim">
          Play a game vs AI and click "Save Replay" on the game over screen
        </p>
      </div>
    {:else}
      <!-- Sort Controls -->
      <div class="flex items-center gap-4 mb-4">
        <span class="text-sm text-ui-text-dim">Sort by:</span>
        <div class="flex gap-2">
          {#each [
            { value: "date", label: "Date" },
            { value: "turns", label: "Turns" },
            { value: "deck", label: "Deck" },
          ] as option (option.value)}
            <button
              class="px-3 py-1.5 rounded text-sm transition-colors
                     {sortBy === option.value
                       ? 'bg-ui-action text-white'
                       : 'bg-ui-bg border border-gray-600 text-ui-text-dim hover:border-ui-action hover:text-ui-action'}"
              onclick={() => (sortBy = option.value as "date" | "turns" | "deck")}
            >
              {option.label}
            </button>
          {/each}
        </div>
      </div>

      <!-- Replay List -->
      <div class="space-y-2 max-h-96 overflow-y-auto pr-2">
        {#each sortedReplays() as replay (replay.filename)}
          <div
            class="flex items-center gap-4 p-4 bg-ui-bg rounded-lg border border-gray-700 hover:border-gray-600 transition-colors"
          >
            <!-- Replay Info -->
            <button
              class="flex-1 text-left hover:bg-ui-panel/50 rounded p-2 -m-2 transition-colors"
              onclick={() => replayStore.loadReplay(replay.path)}
            >
              <div class="flex items-center gap-2 mb-1">
                <span class="font-semibold text-ui-text">
                  {replay.player1DeckName}
                </span>
                <span class="text-ui-text-dim">vs</span>
                <span class="font-semibold text-ui-text">
                  {replay.player2DeckName}
                </span>
              </div>
              <div class="flex items-center gap-4 text-sm">
                <span class="text-ui-text-dim">
                  {replay.player1Type} vs {replay.player2Type}
                </span>
                <span class="text-ui-text-dim">|</span>
                <span class={getWinnerColor(replay)}>
                  {getWinnerText(replay)}
                </span>
                <span class="text-ui-text-dim">|</span>
                <span class="text-ui-text-dim">
                  {replay.totalTurns} turns, {replay.totalActions} actions
                </span>
              </div>
              <div class="text-xs text-ui-text-dim mt-1">
                {replay.dateString}
              </div>
            </button>

            <!-- Actions -->
            <div class="flex items-center gap-2">
              <button
                class="px-3 py-2 bg-ui-action text-white rounded text-sm font-semibold
                       hover:bg-ui-action/80 transition-colors"
                onclick={() => replayStore.loadReplay(replay.path)}
              >
                Watch
              </button>

              {#if confirmDeletePath === replay.path}
                <div class="flex items-center gap-1">
                  <button
                    class="px-2 py-1 bg-damage text-white rounded text-xs font-semibold
                           hover:bg-damage/80 transition-colors"
                    onclick={() => handleDelete(replay.path)}
                  >
                    Confirm
                  </button>
                  <button
                    class="px-2 py-1 bg-ui-bg text-ui-text-dim rounded text-xs
                           border border-gray-600 hover:border-gray-500 transition-colors"
                    onclick={() => (confirmDeletePath = null)}
                  >
                    Cancel
                  </button>
                </div>
              {:else}
                <button
                  class="px-3 py-2 bg-ui-bg text-ui-text-dim rounded text-sm
                         border border-gray-600 hover:border-damage hover:text-damage transition-colors"
                  onclick={() => (confirmDeletePath = replay.path)}
                  title="Delete replay"
                >
                  Delete
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <!-- Back Button -->
    <div class="mt-8 flex justify-start">
      <button
        class="px-6 py-3 bg-ui-bg text-ui-text-dim rounded-lg font-semibold
               border border-gray-600 hover:border-gray-500 hover:text-ui-text transition-colors"
        onclick={goBack}
      >
        Back to Menu
      </button>
    </div>
  </div>
</div>
