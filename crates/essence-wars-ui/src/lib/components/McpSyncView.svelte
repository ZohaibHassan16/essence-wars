<script lang="ts">
  import { mcpSyncStore } from "$lib/stores/mcpSyncState.svelte";
  import { onMount, onDestroy } from "svelte";
  import CreatureSlot from "./CreatureSlot.svelte";
  import SupportSlot from "./SupportSlot.svelte";
  import HandCard from "./HandCard.svelte";
  import { padCreatures, padSupports } from "$lib/utils/arrays";

  const gameState = $derived(mcpSyncStore.gameState);
  const isP1Turn = $derived(gameState?.activePlayer === 1);

  // Ensure arrays always have exactly the expected number of elements
  const opponentCreatures = $derived(padCreatures(gameState?.opponent.creatures));
  const opponentSupports = $derived(padSupports(gameState?.opponent.supports));
  const playerCreatures = $derived(padCreatures(gameState?.player.creatures));
  const playerSupports = $derived(padSupports(gameState?.player.supports));

  // Start polling when component mounts
  onMount(() => {
    mcpSyncStore.startWatching();
  });

  // Stop polling when component unmounts
  onDestroy(() => {
    mcpSyncStore.stopWatching();
  });
</script>

<div class="w-full h-full flex bg-ui-bg">
  <!-- Main game area -->
  <div class="flex-1 flex flex-col">
    <!-- Top header bar -->
    <div class="h-10 bg-ui-panel/80 flex items-center justify-between px-4 border-b border-gray-700">
      <div class="flex items-center gap-4">
        <button
          class="text-ui-text-dim hover:text-ui-text transition-colors text-sm"
          onclick={() => mcpSyncStore.reset()}
        >
          &larr; Back
        </button>
        <div class="text-ui-text font-semibold text-sm">
          MCP Game Sync
        </div>
      </div>
      <div class="flex items-center gap-2">
        {#if mcpSyncStore.phase === "watching" && !mcpSyncStore.isStale}
          <span class="w-2 h-2 rounded-full bg-health animate-pulse"></span>
          <span class="text-health text-xs">Live</span>
        {:else if mcpSyncStore.phase === "disconnected" || mcpSyncStore.isStale}
          <span class="w-2 h-2 rounded-full bg-damage"></span>
          <span class="text-damage text-xs">Stale ({Math.round(mcpSyncStore.stateAgeMs / 1000)}s)</span>
        {:else}
          <span class="w-2 h-2 rounded-full bg-gray-500"></span>
          <span class="text-ui-text-dim text-xs">Waiting for sync...</span>
        {/if}
      </div>
    </div>

    {#if gameState}
      <!-- Top bar: Player 2 (opponent) info -->
      <div class="h-14 bg-ui-panel flex items-center justify-between px-6 border-b border-gray-700">
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <div class="w-3 h-3 rounded-full {!isP1Turn ? 'bg-damage animate-pulse' : 'bg-gray-600'}"></div>
            <span class="text-ui-text font-semibold">Opponent (P2)</span>
          </div>
          <div class="flex items-center gap-3">
            <div class="flex items-center gap-1 px-2 py-1 rounded bg-damage/20">
              <span class="text-damage font-bold">{gameState.opponent.life}</span>
              <span class="text-damage/60 text-xs">HP</span>
            </div>
            <div class="flex items-center gap-1 px-2 py-1 rounded bg-mana/20">
              <span class="text-mana font-bold">{gameState.opponent.essence}</span>
              <span class="text-mana/60 text-xs">/ {gameState.opponent.maxEssence}</span>
            </div>
            <div class="flex items-center gap-1 px-2 py-1 rounded bg-gold/20">
              <span class="text-gold font-bold">{gameState.opponent.actionPoints}</span>
              <span class="text-gold/60 text-xs">AP</span>
            </div>
          </div>
        </div>
        <div class="flex items-center gap-3 text-sm text-ui-text-dim">
          <span>Deck: {gameState.opponent.deckCount}</span>
          <span>Hand: {gameState.opponent.hand.length}</span>
        </div>
      </div>

      <!-- Player 2 hand -->
      <div class="h-32 flex items-center justify-center gap-1.5 bg-gray-900/30 px-4 py-2">
        {#each gameState.opponent.hand as card, i (i)}
          <HandCard {card} index={i} isPlayable={false} />
        {/each}
        {#if gameState.opponent.hand.length === 0}
          <span class="text-ui-text-dim text-sm">Empty hand</span>
        {/if}
      </div>

      <!-- Main board area -->
      <div class="flex-1 flex flex-col justify-center gap-6 px-6 py-4">
        <!-- Player 2's board (opponent) -->
        <div class="flex items-center justify-center gap-4">
          <!-- Player 2 supports -->
          <div class="flex flex-col gap-2">
            {#each opponentSupports as support, i (i)}
              <SupportSlot {support} slot={i} />
            {/each}
          </div>

          <!-- Player 2 creatures -->
          <div class="flex gap-2">
            {#each opponentCreatures as creature, i (i)}
              <div id="creature-opponent-{i}">
                <CreatureSlot
                  {creature}
                  slot={i}
                  isPlayerSide={false}
                />
              </div>
            {/each}
          </div>
        </div>

        <!-- Center divider with turn info -->
        <div class="flex items-center justify-center gap-4">
          <div class="h-px flex-1 bg-gradient-to-r from-transparent via-gray-600 to-transparent"></div>
          <div class="px-6 py-2 rounded-full border border-gray-600 bg-ui-panel/80 flex items-center gap-3">
            <span class="text-ui-text-dim text-sm">Turn</span>
            <span class="text-ui-text font-bold text-lg">{gameState.turn}</span>
            <div class="w-px h-4 bg-gray-600"></div>
            <span class="text-sm font-semibold {isP1Turn ? 'text-health' : 'text-damage'}">
              {isP1Turn ? "Your Turn" : "Opponent Turn"}
            </span>
            {#if gameState.isGameOver}
              <div class="w-px h-4 bg-gray-600"></div>
              <span class="text-sm font-semibold text-gold">
                {#if gameState.winner === 1}
                  You Win!
                {:else if gameState.winner === 2}
                  Opponent Wins!
                {:else}
                  Draw!
                {/if}
              </span>
            {/if}
          </div>
          <div class="h-px flex-1 bg-gradient-to-r from-transparent via-gray-600 to-transparent"></div>
        </div>

        <!-- Player 1's board -->
        <div class="flex items-center justify-center gap-4">
          <!-- Player 1 supports -->
          <div class="flex flex-col gap-2">
            {#each playerSupports as support, i (i)}
              <SupportSlot {support} slot={i} />
            {/each}
          </div>

          <!-- Player 1 creatures -->
          <div class="flex gap-2">
            {#each playerCreatures as creature, i (i)}
              <div id="creature-player-{i}">
                <CreatureSlot
                  {creature}
                  slot={i}
                  isPlayerSide={true}
                />
              </div>
            {/each}
          </div>
        </div>
      </div>

      <!-- Player 1 hand -->
      <div class="h-32 flex items-center justify-center gap-1.5 bg-gray-900/30 px-4 py-2">
        {#each gameState.player.hand as card, i (i)}
          <HandCard {card} index={i} isPlayable={false} />
        {/each}
        {#if gameState.player.hand.length === 0}
          <span class="text-ui-text-dim text-sm">Empty hand</span>
        {/if}
      </div>

      <!-- Bottom bar: Player 1 info -->
      <div class="h-14 bg-ui-panel flex items-center justify-between px-6 border-t border-gray-700">
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <div class="w-3 h-3 rounded-full {isP1Turn ? 'bg-health animate-pulse' : 'bg-gray-600'}"></div>
            <span class="text-ui-text font-semibold">You (P1)</span>
          </div>
          <div class="flex items-center gap-3">
            <div class="flex items-center gap-1 px-2 py-1 rounded bg-health/20">
              <span class="text-health font-bold">{gameState.player.life}</span>
              <span class="text-health/60 text-xs">HP</span>
            </div>
            <div class="flex items-center gap-1 px-2 py-1 rounded bg-mana/20">
              <span class="text-mana font-bold">{gameState.player.essence}</span>
              <span class="text-mana/60 text-xs">/ {gameState.player.maxEssence}</span>
            </div>
            <div class="flex items-center gap-1 px-2 py-1 rounded bg-gold/20">
              <span class="text-gold font-bold">{gameState.player.actionPoints}</span>
              <span class="text-gold/60 text-xs">AP</span>
            </div>
          </div>
          <div class="text-sm text-ui-text-dim">
            Deck: {gameState.player.deckCount}
          </div>
        </div>
        <div class="text-sm text-ui-text-dim">
          Game ID: {gameState.id.slice(0, 8)}...
        </div>
      </div>
    {:else}
      <!-- No game state yet -->
      <div class="flex-1 flex flex-col items-center justify-center gap-4">
        <div class="text-ui-text-dim text-lg">Waiting for MCP game sync...</div>
        <div class="text-ui-text-dim text-sm">
          Start a game via MCP tools and it will appear here.
        </div>
        {#if mcpSyncStore.error}
          <div class="mt-4 p-4 bg-damage/20 border border-damage rounded text-damage max-w-md">
            {mcpSyncStore.error}
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
