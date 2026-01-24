<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";
  import { gameSettings } from "$lib/stores/gameSettings.svelte";
  import BattlefieldRow from "./board/BattlefieldRow.svelte";
  import FanningHand from "./board/FanningHand.svelte";
  import PlayerInfoWidget from "./board/PlayerInfoWidget.svelte";
  import CollapsibleSidebar from "./board/CollapsibleSidebar.svelte";
  import ActionLog from "./ActionLog.svelte";
  import HintPanel from "./HintPanel.svelte";
  import TurnTransition from "./TurnTransition.svelte";
  import AudioControls from "./AudioControls.svelte";
  import { playSound } from "$lib/audio";

  const gameState = $derived(gameStore.gameState);
  const isPlayerTurn = $derived(gameStore.isPlayerTurn);

  // Track turn changes for transition animation
  let lastActivePlayer: number | null = $state(null);
  let showTurnTransition = $state(false);

  $effect(() => {
    const currentPlayer = gameState?.activePlayer;
    if (currentPlayer !== undefined && lastActivePlayer !== null && currentPlayer !== lastActivePlayer) {
      // Turn changed, show transition
      showTurnTransition = true;
      setTimeout(() => {
        showTurnTransition = false;
      }, 1500);
    }
    lastActivePlayer = currentPlayer ?? null;
  });

  function handlePlayerCreatureClick(slot: number) {
    if (!isPlayerTurn) return;

    const action = gameStore.getActionForTarget(slot);
    if (action) {
      gameStore.applyAction(action.index);
    } else if (gameStore.selectedCardIndex === null) {
      // Select creature for attack
      const creature = gameState?.player.creatures[slot];
      if (creature?.canAttack) {
        gameStore.selectCreature(slot);
      }
    }
  }

  function handlePlayerSupportClick(slot: number) {
    if (!isPlayerTurn) return;
    const action = gameStore.getActionForTarget(slot);
    if (action) {
      gameStore.applyAction(action.index);
    }
  }

  function handleOpponentCreatureClick(slot: number) {
    if (!isPlayerTurn) return;

    const action = gameStore.getActionForTarget(slot);
    if (action) {
      gameStore.applyAction(action.index);
    }
  }

  function handleCardClick(index: number) {
    if (!isPlayerTurn) return;
    gameStore.selectCard(index);
  }

  function isCardPlayable(index: number): boolean {
    return gameStore.legalActions.some(
      a => a.actionType === "play_card" && a.handIndex === index
    );
  }

  // Check if an opponent slot is a valid attack target
  function getValidAttackTargets(): number[] {
    if (gameStore.selectedCreatureSlot === null) return [];
    return gameStore.highlightedSlots;
  }

  // Get highlighted slots for card placement
  function getHighlightedSlots(): number[] {
    if (gameStore.selectedCardIndex === null) return [];
    return gameStore.highlightedSlots;
  }

  // Convert LoggedAction to ActionInfo for the log
  const actionsForLog = $derived(
    gameStore.actionHistory.map(a => ({
      index: a.index,
      actionType: a.actionType,
      description: `[P${a.player}] ${a.description}`,
      sourceSlot: a.sourceSlot,
      targetSlot: a.targetSlot,
      handIndex: a.handIndex,
      cardId: a.cardId,
    }))
  );

  // Derive player faction from first available card (hand or board)
  const playerFaction = $derived(() => {
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

  const boardBgClass = $derived(`board-bg-${playerFaction()}`);
</script>

<!-- Turn transition overlay -->
<TurnTransition
  isYourTurn={isPlayerTurn}
  turnNumber={gameState?.turn ?? 1}
  visible={showTurnTransition}
/>

<!-- Error banner -->
{#if gameStore.error}
  <div class="fixed top-4 left-1/2 -translate-x-1/2 z-50 max-w-lg">
    <div class="p-4 bg-damage/90 border border-damage rounded-lg text-white shadow-xl">
      <div class="flex items-start gap-3">
        <div class="flex-1">
          <p class="font-semibold">Error</p>
          <p class="text-sm mt-1">{gameStore.error}</p>
        </div>
        <div class="flex gap-2">
          {#if gameStore.aiTurnFailed}
            <button
              class="px-3 py-1 bg-white/20 hover:bg-white/30 rounded text-sm font-medium transition-colors"
              onclick={() => gameStore.retryAiTurn()}
            >
              Retry
            </button>
          {/if}
          <button
            class="px-3 py-1 bg-white/20 hover:bg-white/30 rounded text-sm font-medium transition-colors"
            onclick={() => gameStore.clearError()}
          >
            Dismiss
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<div class="w-full h-full flex {boardBgClass}">
  <!-- Main game area -->
  <div class="flex-1 flex flex-col min-w-0">
    <!-- Opponent info bar -->
    <PlayerInfoWidget
      name="Opponent"
      life={gameState?.opponent.life ?? 0}
      essence={gameState?.opponent.essence ?? 0}
      maxEssence={gameState?.opponent.maxEssence ?? 0}
      actionPoints={gameState?.opponent.actionPoints ?? 0}
      deckCount={gameState?.opponent.deckCount ?? 0}
      handCount={gameState?.opponent.hand.length ?? 0}
      isActive={!isPlayerTurn}
      isPlayer={false}
    />

    <!-- Opponent hand (hidden cards, compact) -->
    <div class="bg-gray-900/30 py-1">
      <FanningHand
        cards={gameState?.opponent.hand ?? []}
        compact={true}
        previewPosition="bottom"
      />
    </div>

    <!-- Main board area -->
    <div class="flex-1 flex flex-col justify-center px-4 py-2" style="gap: var(--board-gap);">
      <!-- Opponent's battlefield row -->
      <BattlefieldRow
        creatures={gameState?.opponent.creatures ?? [null, null, null, null, null]}
        supports={gameState?.opponent.supports ?? [null, null]}
        isPlayerSide={false}
        validAttackTargets={getValidAttackTargets()}
        onCreatureClick={handleOpponentCreatureClick}
      />

      <!-- Center turn indicator -->
      <div class="flex items-center justify-center gap-2">
        <div class="h-px flex-1 bg-gradient-to-r from-transparent via-gray-600 to-transparent"></div>
        <div class="px-4 py-1.5 rounded-full border border-gray-600 bg-ui-panel/80 flex items-center gap-2">
          <span class="text-ui-text-dim text-sm">Turn</span>
          <span class="text-ui-text font-bold text-lg">{gameState?.turn ?? 0}</span>
          <div class="w-px h-4 bg-gray-600"></div>
          <span class="text-sm font-semibold {isPlayerTurn ? 'text-health' : 'text-ui-action'}">
            {isPlayerTurn ? "Your Turn" : "P2 Turn"}
          </span>
        </div>
        <div class="h-px flex-1 bg-gradient-to-r from-transparent via-gray-600 to-transparent"></div>
      </div>

      <!-- Player's battlefield row -->
      <BattlefieldRow
        creatures={gameState?.player.creatures ?? [null, null, null, null, null]}
        supports={gameState?.player.supports ?? [null, null]}
        isPlayerSide={true}
        selectedCreatureSlot={gameStore.selectedCreatureSlot}
        highlightedSlots={getHighlightedSlots()}
        onCreatureClick={handlePlayerCreatureClick}
        onSupportClick={handlePlayerSupportClick}
        showKeyHints={isPlayerTurn}
      />
    </div>

    <!-- Player hand (interactive, with fanning) -->
    <div class="bg-gray-900/30 py-2">
      <FanningHand
        cards={gameState?.player.hand ?? []}
        selectedCardIndex={gameStore.selectedCardIndex}
        isInteractive={true}
        isPlayableCallback={isCardPlayable}
        onCardClick={handleCardClick}
        showKeyHints={isPlayerTurn}
      />
    </div>

    <!-- Player info bar with action buttons -->
    <PlayerInfoWidget
      name="You"
      life={gameState?.player.life ?? 0}
      essence={gameState?.player.essence ?? 0}
      maxEssence={gameState?.player.maxEssence ?? 0}
      actionPoints={gameState?.player.actionPoints ?? 0}
      deckCount={gameState?.player.deckCount ?? 0}
      isActive={isPlayerTurn}
      isPlayer={true}
    >
      {#snippet actions()}
        {#if isPlayerTurn}
          <button
            class="px-4 py-2 bg-amber-700 text-white rounded-lg font-semibold
                   hover:bg-amber-600 active:scale-95 transition-all
                   disabled:opacity-50 disabled:cursor-not-allowed"
            onclick={() => {
              playSound('buttonClick');
              gameStore.undoAction();
            }}
            onmouseenter={() => playSound('buttonHover')}
            disabled={gameStore.isLoading || gameStore.actionHistory.length === 0}
            title="Undo last action (dev mode)"
          >
            Undo
          </button>
          <button
            class="px-6 py-2 bg-ui-action text-white rounded-lg font-semibold
                   hover:bg-ui-action/80 active:scale-95 transition-all
                   disabled:opacity-50 disabled:cursor-not-allowed"
            onclick={() => {
              playSound('buttonClick');
              gameStore.endTurn();
            }}
            onmouseenter={() => playSound('buttonHover')}
            disabled={gameStore.isLoading}
          >
            End Turn{#if gameSettings.showKeyboardHints}<span class="ml-2 text-xs opacity-70">(Space)</span>{/if}
          </button>
        {:else}
          <div class="px-5 py-2 bg-gray-700 text-ui-text-dim rounded-lg font-semibold flex items-center gap-2">
            <div class="w-4 h-4 border-2 border-ui-text-dim border-t-transparent rounded-full animate-spin"></div>
            AI Thinking...
          </div>
        {/if}
        <button
          class="px-4 py-2 bg-gray-700 text-ui-text rounded-lg hover:bg-gray-600 transition-colors"
          onclick={() => {
            playSound('buttonClick');
            gameStore.quitGame();
          }}
          onmouseenter={() => playSound('buttonHover')}
        >
          Quit
        </button>
        <AudioControls />
      {/snippet}
    </PlayerInfoWidget>
  </div>

  <!-- Right sidebar: Hint Panel + Action Log -->
  <CollapsibleSidebar>
    <!-- AI Hint Panel (only when player's turn) -->
    {#if isPlayerTurn}
      <div class="p-2 border-b border-gray-700">
        <HintPanel
          hint={gameStore.currentHint}
          isLoading={gameStore.isHintLoading}
          onRequestHint={() => gameStore.requestHint()}
          onApplyHint={(action) => gameStore.applyHint(action)}
        />
      </div>
    {/if}

    <!-- Action Log -->
    <div class="flex-1 p-2 overflow-hidden">
      <ActionLog actions={actionsForLog} />
    </div>
  </CollapsibleSidebar>
</div>
