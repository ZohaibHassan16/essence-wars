<script lang="ts">
  import { replayStore } from "$lib/stores/replayState.svelte";
  import CreatureSlot from "./CreatureSlot.svelte";
  import SupportSlot from "./SupportSlot.svelte";
  import HandCard from "./HandCard.svelte";
  import ActionLog from "./ActionLog.svelte";
  import ReplayControls from "./ReplayControls.svelte";

  const gameState = $derived(replayStore.currentState);
  const match = $derived(replayStore.match);
  const isP1Turn = $derived(gameState?.activePlayer === 1);

  // Convert replay actions to ActionInfo format for the log
  const actionsForLog = $derived(
    replayStore.match?.actions.slice(0, replayStore.currentActionIndex + 1).map(a => ({
      index: a.action.index,
      actionType: a.action.actionType,
      description: `[P${a.player}] ${a.action.description}`,
      sourceSlot: a.action.sourceSlot,
      targetSlot: a.action.targetSlot,
      handIndex: a.action.handIndex,
      cardId: a.action.cardId,
    })) ?? []
  );
</script>

<div class="w-full h-full flex bg-ui-bg">
  <!-- Main game area -->
  <div class="flex-1 flex flex-col">
    <!-- Top header bar -->
    <div class="h-10 bg-ui-panel/80 flex items-center justify-between px-4 border-b border-gray-700">
      <div class="flex items-center gap-4">
        <button
          class="text-ui-text-dim hover:text-ui-text transition-colors text-sm"
          onclick={() => replayStore.backToBrowser()}
        >
          &larr; Back
        </button>
        <span class="px-2 py-0.5 bg-ui-action/20 text-ui-action rounded text-xs font-semibold">
          REPLAY
        </span>
        <div class="text-ui-text font-semibold text-sm">
          {match?.player1DeckName ?? "Player 1"} vs {match?.player2DeckName ?? "Player 2"}
        </div>
      </div>
      <div class="text-ui-text-dim text-xs">
        {match?.player1BotName} vs {match?.player2BotName}
      </div>
    </div>

    <!-- Top bar: Player 2 (opponent) info -->
    <div class="h-14 bg-ui-panel flex items-center justify-between px-6 border-b border-gray-700">
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <div class="w-3 h-3 rounded-full {!isP1Turn ? 'bg-damage animate-pulse' : 'bg-gray-600'}"></div>
          <span class="text-ui-text font-semibold">P2 - {match?.player2DeckName ?? "Player 2"}</span>
          <span class="text-xs text-ui-text-dim">({match?.player2BotName})</span>
        </div>
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-1 px-2 py-1 rounded bg-damage/20">
            <span class="text-damage font-bold">{gameState?.opponent.life ?? 0}</span>
            <span class="text-damage/60 text-xs">HP</span>
          </div>
          <div class="flex items-center gap-1 px-2 py-1 rounded bg-mana/20">
            <span class="text-mana font-bold">{gameState?.opponent.essence ?? 0}</span>
            <span class="text-mana/60 text-xs">/ {gameState?.opponent.maxEssence ?? 0}</span>
          </div>
          <div class="flex items-center gap-1 px-2 py-1 rounded bg-gold/20">
            <span class="text-gold font-bold">{gameState?.opponent.actionPoints ?? 0}</span>
            <span class="text-gold/60 text-xs">AP</span>
          </div>
        </div>
      </div>
      <div class="flex items-center gap-3 text-sm text-ui-text-dim">
        <span>Deck: {gameState?.opponent.deckCount ?? 0}</span>
        <span>Hand: {gameState?.opponent.hand.length ?? 0}</span>
      </div>
    </div>

    <!-- Player 2 hand (visible in replay mode) -->
    <div class="h-32 flex items-center justify-center gap-1.5 bg-gray-900/30 px-4 py-2">
      {#each gameState?.opponent.hand ?? [] as card, i}
        <HandCard {card} index={i} isPlayable={false} />
      {/each}
      {#if (gameState?.opponent.hand.length ?? 0) === 0}
        <span class="text-ui-text-dim text-sm">Empty hand</span>
      {/if}
    </div>

    <!-- Main board area -->
    <div class="flex-1 flex flex-col justify-center gap-6 px-6 py-4">
      <!-- Player 2's board (opponent) -->
      <div class="flex items-center justify-center gap-4">
        <!-- Player 2 supports -->
        <div class="flex flex-col gap-2">
          {#each gameState?.opponent.supports ?? [null, null] as support, i}
            <SupportSlot {support} slot={i} />
          {/each}
        </div>

        <!-- Player 2 creatures -->
        <div class="flex gap-2">
          {#each gameState?.opponent.creatures ?? [null, null, null, null, null] as creature, i}
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
          <span class="text-ui-text font-bold text-lg">{gameState?.turn ?? 0}</span>
          <div class="w-px h-4 bg-gray-600"></div>
          <span class="text-sm font-semibold {isP1Turn ? 'text-health' : 'text-damage'}">
            {isP1Turn ? "P1 Turn" : "P2 Turn"}
          </span>
        </div>
        <div class="h-px flex-1 bg-gradient-to-r from-transparent via-gray-600 to-transparent"></div>
      </div>

      <!-- Player 1's board -->
      <div class="flex items-center justify-center gap-4">
        <!-- Player 1 supports -->
        <div class="flex flex-col gap-2">
          {#each gameState?.player.supports ?? [null, null] as support, i}
            <SupportSlot {support} slot={i} />
          {/each}
        </div>

        <!-- Player 1 creatures -->
        <div class="flex gap-2">
          {#each gameState?.player.creatures ?? [null, null, null, null, null] as creature, i}
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

    <!-- Player 1 hand (visible in replay mode) -->
    <div class="h-32 flex items-center justify-center gap-1.5 bg-gray-900/30 px-4 py-2">
      {#each gameState?.player.hand ?? [] as card, i}
        <HandCard {card} index={i} isPlayable={false} />
      {/each}
      {#if (gameState?.player.hand.length ?? 0) === 0}
        <span class="text-ui-text-dim text-sm">Empty hand</span>
      {/if}
    </div>

    <!-- Bottom bar: Player 1 info -->
    <div class="h-14 bg-ui-panel flex items-center justify-between px-6 border-t border-gray-700">
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <div class="w-3 h-3 rounded-full {isP1Turn ? 'bg-health animate-pulse' : 'bg-gray-600'}"></div>
          <span class="text-ui-text font-semibold">P1 - {match?.player1DeckName ?? "Player 1"}</span>
          <span class="text-xs text-ui-text-dim">({match?.player1BotName})</span>
        </div>
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-1 px-2 py-1 rounded bg-health/20">
            <span class="text-health font-bold">{gameState?.player.life ?? 0}</span>
            <span class="text-health/60 text-xs">HP</span>
          </div>
          <div class="flex items-center gap-1 px-2 py-1 rounded bg-mana/20">
            <span class="text-mana font-bold">{gameState?.player.essence ?? 0}</span>
            <span class="text-mana/60 text-xs">/ {gameState?.player.maxEssence ?? 0}</span>
          </div>
          <div class="flex items-center gap-1 px-2 py-1 rounded bg-gold/20">
            <span class="text-gold font-bold">{gameState?.player.actionPoints ?? 0}</span>
            <span class="text-gold/60 text-xs">AP</span>
          </div>
        </div>
        <div class="text-sm text-ui-text-dim">
          Deck: {gameState?.player.deckCount ?? 0}
        </div>
      </div>
      <!-- Result badge and buttons when game is finished -->
      {#if replayStore.phase === "finished"}
        <div class="flex items-center gap-3">
          {#if match?.result.winner === 1}
            <span class="px-3 py-1 bg-health/20 text-health rounded-full text-sm font-semibold">
              P1 Wins!
            </span>
          {:else if match?.result.winner === 2}
            <span class="px-3 py-1 bg-damage/20 text-damage rounded-full text-sm font-semibold">
              P2 Wins!
            </span>
          {:else}
            <span class="px-3 py-1 bg-gray-600/50 text-ui-text rounded-full text-sm font-semibold">
              Draw
            </span>
          {/if}

          <!-- View Results button -->
          <button
            class="px-3 py-1 bg-ui-action/20 text-ui-action rounded text-sm font-semibold
                   border border-ui-action/50 hover:bg-ui-action hover:text-white transition-all"
            onclick={() => replayStore.showGameOver()}
          >
            View Summary
          </button>
        </div>
      {/if}
    </div>

    <!-- Playback controls -->
    <div class="border-t border-gray-700">
      <ReplayControls />
    </div>
  </div>

  <!-- Right sidebar: Action Log only (no AI thinking panel for replays) -->
  <div class="w-72 border-l border-gray-700 bg-ui-panel/50 flex flex-col">
    <!-- Action Log -->
    <div class="flex-1 p-3 overflow-hidden">
      <ActionLog actions={actionsForLog} maxItems={20} />
    </div>
  </div>
</div>
