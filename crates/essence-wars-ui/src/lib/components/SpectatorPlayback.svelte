<script lang="ts">
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import * as api from "$lib/api/game";
  import BattlefieldRow from "./board/BattlefieldRow.svelte";
  import FanningHand from "./board/FanningHand.svelte";
  import CommanderCardLarge from "./board/CommanderCardLarge.svelte";
  import EssenceBar from "./board/EssenceBar.svelte";
  import CollapsibleSidebar from "./board/CollapsibleSidebar.svelte";
  import AiThinkingPanel from "./AiThinkingPanel.svelte";
  import SpectatorControls from "./SpectatorControls.svelte";
  import CommentaryOverlay from "./CommentaryOverlay.svelte";
  import BottomTimelinePanel from "./spectator/BottomTimelinePanel.svelte";
  import AnalysisDashboard from "./spectator/AnalysisDashboard.svelte";
  import ActionLogModal from "./ActionLogModal.svelte";
  import { playMusic } from "$lib/audio";

  // View mode
  const viewMode = $derived(spectatorStore.viewMode);

  // Keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
      return;
    }
    switch (event.key.toLowerCase()) {
      case "a":
        spectatorStore.toggleViewMode();
        break;
      case "l":
        if (spectatorStore.showActionLog) {
          spectatorStore.closeActionLog();
        } else {
          spectatorStore.openActionLog();
        }
        break;
    }
  }

  // Play spectator music when component mounts
  $effect(() => {
    playMusic('spectator');
  });

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


  // Derive player 1 faction from first available card
  const player1Faction = $derived(() => {
    const state = gameState;
    if (!state) return "default";

    // Check hand first
    for (const card of state.player.hand) {
      if (card.faction && card.faction !== "unknown") {
        return card.faction;
      }
    }
    // Check creatures
    for (const creature of state.player.creatures) {
      if (creature?.faction && creature.faction !== "neutral") {
        return creature.faction;
      }
    }
    return "default";
  });

  const boardBgClass = $derived(`board-bg-${player1Faction()}`);
</script>

<svelte:window onkeydown={handleKeydown} />

{#if viewMode === "analysis"}
  <!-- ANALYSIS MODE: Full-width analysis dashboard -->
  <AnalysisDashboard />
{:else}
  <!-- WATCH MODE: Game board with bottom timeline -->
  <div class="w-full h-full flex flex-col">
    <div class="flex-1 flex min-h-0 {boardBgClass}">
  <!-- LEFT COLUMN: Commander Cards -->
  <div class="flex flex-col justify-between p-2 bg-ui-panel/30 border-r border-gray-700/50 overflow-visible"
       style="width: var(--commander-card-width, 250px);">
    <!-- P2 Commander (top) -->
    <div class="flex flex-col items-center overflow-visible">
      <div class="flex items-stretch gap-2">
        <!-- P2 Essence Bar -->
        <EssenceBar
          current={gameState?.opponent.essence ?? 0}
          max={gameState?.opponent.maxEssence ?? 0}
          isPlayer={false}
        />
        <CommanderCardLarge
          commander={gameState?.opponent.commander ?? null}
          life={gameState?.opponent.life ?? 0}
          maxLife={gameState?.opponent.maxLife ?? 30}
          essence={gameState?.opponent.essence ?? 0}
          maxEssence={gameState?.opponent.maxEssence ?? 0}
          essenceExtracted={gameState?.opponent.essenceExtracted ?? 0}
          isActive={!isP1Turn}
          isPlayer={false}
        />
      </div>
      <!-- P2 compact stats below commander -->
      <div class="mt-2 flex items-center gap-3 text-xs text-ui-text-dim">
        <span title="Cards in hand">
          <svg class="w-3.5 h-3.5 inline mr-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          {gameState?.opponent.hand.length ?? 0}
        </span>
        <span title="Cards in deck">
          <svg class="w-3.5 h-3.5 inline mr-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          {gameState?.opponent.deckCount ?? 0}
        </span>
        <span title="Action Points" class="text-gold">
          AP: {gameState?.opponent.actionPoints ?? 0}
        </span>
      </div>
      <!-- P2 label -->
      <div class="mt-1 text-xs text-ui-text-dim truncate max-w-full px-2">
        P2 - {match?.player2DeckName ?? "Player 2"}
      </div>
    </div>

    <!-- P1 Commander (bottom) -->
    <div class="flex flex-col items-center overflow-visible">
      <!-- P1 label -->
      <div class="mb-1 text-xs text-ui-text-dim truncate max-w-full px-2">
        P1 - {match?.player1DeckName ?? "Player 1"}
      </div>
      <!-- P1 compact stats above commander -->
      <div class="mb-2 flex items-center gap-3 text-xs text-ui-text-dim">
        <span title="Cards in deck">
          <svg class="w-3.5 h-3.5 inline mr-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          {gameState?.player.deckCount ?? 0}
        </span>
        <span title="Action Points" class="text-gold">
          AP: {gameState?.player.actionPoints ?? 0}
        </span>
      </div>
      <div class="flex items-stretch gap-2">
        <!-- P1 Essence Bar -->
        <EssenceBar
          current={gameState?.player.essence ?? 0}
          max={gameState?.player.maxEssence ?? 0}
          isPlayer={true}
        />
        <CommanderCardLarge
          commander={gameState?.player.commander ?? null}
          life={gameState?.player.life ?? 0}
          maxLife={gameState?.player.maxLife ?? 30}
          essence={gameState?.player.essence ?? 0}
          maxEssence={gameState?.player.maxEssence ?? 0}
          essenceExtracted={gameState?.player.essenceExtracted ?? 0}
          isActive={isP1Turn}
          isPlayer={true}
        />
      </div>
    </div>
  </div>

  <!-- CENTER COLUMN: Main Game Area -->
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

    <!-- P2 hand (visible in spectator mode) -->
    <div class="bg-gray-900/30 py-1 border-b border-gray-700/30">
      <FanningHand
        cards={gameState?.opponent.hand ?? []}
        previewPosition="bottom"
        tutorialId="opponent-hand"
      />
    </div>

    <!-- Main board area -->
    <div class="flex-1 flex flex-col justify-center px-4 py-2" style="gap: var(--board-gap);">
      <!-- P2's battlefield row -->
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

      <!-- P1's battlefield row -->
      <BattlefieldRow
        creatures={gameState?.player.creatures ?? [null, null, null, null, null]}
        supports={gameState?.player.supports ?? [null, null]}
        isPlayerSide={true}
      />
    </div>

    <!-- P1 hand (visible in spectator mode) -->
    <div class="bg-gray-900/30 py-2 border-t border-gray-700/30">
      <FanningHand
        cards={gameState?.player.hand ?? []}
        tutorialId="player-hand"
      />
    </div>

    <!-- Action bar with result badges -->
    <div class="flex items-center justify-center gap-3 px-4 py-2 bg-ui-panel/50 border-t border-gray-700">
      {#if spectatorStore.phase === "finished" && spectatorStore.canShowResult}
        <!-- Result badge -->
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

        <!-- View Statistics button -->
        {#if spectatorStore.matchStatistics}
          <button
            class="px-4 py-2 bg-purple-600/20 text-purple-400 rounded-lg font-semibold
                   border border-purple-500/50 hover:bg-purple-600 hover:text-white transition-all"
            onclick={() => spectatorStore.openStatsSummary()}
          >
            View Statistics
          </button>
        {/if}

        <!-- Save Replay button -->
        <div class="flex items-center gap-2">
          {#if saveSuccess}
            <span class="text-health text-xs">Saved!</span>
          {:else if saveError}
            <span class="text-damage text-xs" title={saveError}>Error</span>
          {/if}
          <button
            class="px-4 py-2 bg-gray-700 text-ui-text rounded-lg font-semibold
                   hover:bg-gray-600 transition-colors
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

        <!-- Quit to Menu button -->
        <button
          class="px-4 py-2 bg-damage/20 text-damage rounded-lg font-semibold
                 border border-damage/50 hover:bg-damage hover:text-white transition-all"
          onclick={() => spectatorStore.backToMenu()}
        >
          Quit to Menu
        </button>
      {:else}
        <!-- Empty state when game not finished -->
        <div class="text-ui-text-dim text-sm">
          Watching replay...
        </div>
      {/if}
    </div>

    </div>

    <!-- RIGHT COLUMN: Simplified Sidebar (Watch Mode) -->
    <CollapsibleSidebar>
      <!-- AI Thinking Panel -->
      <div class="p-2 border-b border-gray-700">
        <AiThinkingPanel />
      </div>

      <!-- Action Log Button -->
      <div class="p-2">
        <button
          class="w-full flex items-center justify-center gap-2 px-3 py-2 text-sm rounded-lg transition-colors
                 bg-gray-700/50 text-ui-text-dim hover:bg-gray-700 hover:text-ui-text border border-gray-600"
          onclick={() => spectatorStore.openActionLog()}
        >
          <span>📋</span>
          <span>Action Log</span>
          <span class="text-xs text-ui-text-dim/60">[L]</span>
        </button>
      </div>
    </CollapsibleSidebar>
    </div>

    <!-- Bottom Timeline Panel (full width) -->
    <BottomTimelinePanel />

    <!-- Playback controls -->
    <div class="border-t border-gray-700 flex-shrink-0">
      <SpectatorControls />
    </div>
  </div>
{/if}

<!-- Commentary overlay for key moments -->
<CommentaryOverlay />

<!-- Action Log Modal -->
<ActionLogModal />
