<script lang="ts">
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import * as api from "$lib/api/game";
  import BattlefieldRow from "./board/BattlefieldRow.svelte";
  import FanningHand from "./board/FanningHand.svelte";
  import PlayerInfoWidget from "./board/PlayerInfoWidget.svelte";
  import CollapsibleSidebar from "./board/CollapsibleSidebar.svelte";
  import ActionLog from "./ActionLog.svelte";
  import AiThinkingPanel from "./AiThinkingPanel.svelte";
  import SpectatorControls from "./SpectatorControls.svelte";

  const gameState = $derived(spectatorStore.currentState);
  const match = $derived(spectatorStore.match);
  const isP1Turn = $derived(gameState?.activePlayer === 1);

  // Save replay state
  let isSaving = $state(false);
  let saveSuccess = $state(false);
  let saveError = $state<string | null>(null);

  async function handleSaveReplay() {
    if (!match) return;

    isSaving = true;
    saveError = null;
    saveSuccess = false;

    try {
      await api.saveSpectatorReplay(match);
      saveSuccess = true;
    } catch (e) {
      saveError = e instanceof Error ? e.message : String(e);
    } finally {
      isSaving = false;
    }
  }

  // Convert spectator actions to ActionInfo format for the log
  const actionsForLog = $derived(
    spectatorStore.match?.actions.slice(0, spectatorStore.currentActionIndex + 1).map(a => ({
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
  <div class="flex-1 flex flex-col min-w-0">
    <!-- Top header bar with match info -->
    <div class="bg-ui-panel/80 flex items-center justify-between px-4 border-b border-gray-700"
         style="height: var(--info-bar-height);">
      <div class="flex items-center gap-3">
        <button
          class="text-ui-text-dim hover:text-ui-text transition-colors text-sm flex items-center gap-1"
          onclick={() => spectatorStore.backToMenu()}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
          Back
        </button>
        <div class="text-ui-text font-semibold text-sm">
          {match?.player1DeckName ?? "Player 1"} vs {match?.player2DeckName ?? "Player 2"}
        </div>
      </div>
      <div class="text-ui-text-dim text-xs">
        {match?.player1BotName} vs {match?.player2BotName}
      </div>
    </div>

    <!-- Player 2 (opponent) info bar -->
    <PlayerInfoWidget
      name="P2 - {match?.player2DeckName ?? 'Player 2'}"
      life={gameState?.opponent.life ?? 0}
      essence={gameState?.opponent.essence ?? 0}
      maxEssence={gameState?.opponent.maxEssence ?? 0}
      actionPoints={gameState?.opponent.actionPoints ?? 0}
      deckCount={gameState?.opponent.deckCount ?? 0}
      handCount={gameState?.opponent.hand.length ?? 0}
      isActive={!isP1Turn}
      isPlayer={false}
    />

    <!-- Player 2 hand (visible in spectator mode) -->
    <div class="bg-gray-900/30 py-1">
      <FanningHand
        cards={gameState?.opponent.hand ?? []}
        previewPosition="bottom"
        tutorialId="opponent-hand"
      />
    </div>

    <!-- Main board area -->
    <div class="flex-1 flex flex-col justify-center px-4 py-2" style="gap: var(--board-gap);">
      <!-- Player 2's battlefield row -->
      <BattlefieldRow
        creatures={gameState?.opponent.creatures ?? [null, null, null, null, null]}
        supports={gameState?.opponent.supports ?? [null, null]}
        isPlayerSide={false}
      />

      <!-- Center turn indicator -->
      <div class="flex items-center justify-center gap-2">
        <div class="h-px flex-1 bg-gradient-to-r from-transparent via-gray-600 to-transparent"></div>
        <div class="px-4 py-1.5 rounded-full border border-gray-600 bg-ui-panel/80 flex items-center gap-2">
          <span class="text-ui-text-dim text-sm">Turn</span>
          <span class="text-ui-text font-bold text-lg">{gameState?.turn ?? 0}</span>
          <div class="w-px h-4 bg-gray-600"></div>
          <span class="text-sm font-semibold {isP1Turn ? 'text-health' : 'text-damage'}">
            {isP1Turn ? "P1 Turn" : "P2 Turn"}
          </span>
        </div>
        <div class="h-px flex-1 bg-gradient-to-r from-transparent via-gray-600 to-transparent"></div>
      </div>

      <!-- Player 1's battlefield row -->
      <BattlefieldRow
        creatures={gameState?.player.creatures ?? [null, null, null, null, null]}
        supports={gameState?.player.supports ?? [null, null]}
        isPlayerSide={true}
      />
    </div>

    <!-- Player 1 hand (visible in spectator mode) -->
    <div class="bg-gray-900/30 py-2">
      <FanningHand
        cards={gameState?.player.hand ?? []}
        tutorialId="player-hand"
      />
    </div>

    <!-- Player 1 info bar with result badge -->
    <PlayerInfoWidget
      name="P1 - {match?.player1DeckName ?? 'Player 1'}"
      life={gameState?.player.life ?? 0}
      essence={gameState?.player.essence ?? 0}
      maxEssence={gameState?.player.maxEssence ?? 0}
      actionPoints={gameState?.player.actionPoints ?? 0}
      deckCount={gameState?.player.deckCount ?? 0}
      isActive={isP1Turn}
      isPlayer={true}
    >
      {#snippet actions()}
        <!-- Result badge and buttons when game is finished -->
        {#if spectatorStore.phase === "finished" && spectatorStore.canShowResult}
          {#if match?.result.winner === 1}
            <span class="px-4 py-2 bg-health/20 text-health rounded-full font-semibold">
              P1 Wins!
            </span>
          {:else if match?.result.winner === 2}
            <span class="px-4 py-2 bg-damage/20 text-damage rounded-full font-semibold">
              P2 Wins!
            </span>
          {:else}
            <span class="px-4 py-2 bg-gray-600/50 text-ui-text rounded-full font-semibold">
              Draw
            </span>
          {/if}

          <!-- View Results button -->
          <button
            class="px-4 py-2 bg-ui-action/20 text-ui-action rounded-lg font-semibold
                   border border-ui-action/50 hover:bg-ui-action hover:text-white transition-all"
            onclick={() => spectatorStore.showGameOver()}
          >
            View Summary
          </button>

          <!-- Save Replay button -->
          <div class="flex items-center gap-2">
            {#if saveSuccess}
              <span class="text-health text-xs">Saved!</span>
            {:else if saveError}
              <span class="text-damage text-xs" title={saveError}>Error</span>
            {/if}
            <button
              class="px-4 py-2 bg-ui-bg text-ui-text-dim rounded-lg font-semibold
                     border border-gray-600 hover:border-ui-action hover:text-ui-action transition-all
                     disabled:opacity-50 disabled:cursor-not-allowed"
              onclick={handleSaveReplay}
              disabled={isSaving || saveSuccess}
            >
              {#if isSaving}
                Saving...
              {:else if saveSuccess}
                Saved
              {:else}
                Save Replay
              {/if}
            </button>
          </div>
        {/if}
      {/snippet}
    </PlayerInfoWidget>

    <!-- Playback controls -->
    <div class="border-t border-gray-700">
      <SpectatorControls />
    </div>
  </div>

  <!-- Right sidebar: AI Thinking + Action Log -->
  <CollapsibleSidebar>
    <!-- AI Thinking Panel -->
    <div class="p-2 border-b border-gray-700">
      <AiThinkingPanel />
    </div>

    <!-- Action Log -->
    <div class="flex-1 p-2 overflow-hidden">
      <ActionLog actions={actionsForLog} maxItems={10} />
    </div>
  </CollapsibleSidebar>
</div>
