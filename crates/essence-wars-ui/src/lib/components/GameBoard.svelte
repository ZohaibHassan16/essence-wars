<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";
  import CreatureSlot from "./CreatureSlot.svelte";
  import SupportSlot from "./SupportSlot.svelte";
  import HandCard from "./HandCard.svelte";
  import ActionLog from "./ActionLog.svelte";
  import HintPanel from "./HintPanel.svelte";
  import TurnTransition from "./TurnTransition.svelte";

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

  function handlePlayerSlotClick(slot: number) {
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

  function handleOpponentSlotClick(slot: number) {
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

  function canCreatureAttack(slot: number): boolean {
    return gameStore.legalActions.some(
      a => a.actionType === "attack" && a.sourceSlot === slot
    );
  }

  // Check if an opponent slot is a valid attack target
  function isValidAttackTarget(slot: number): boolean {
    if (gameStore.selectedCreatureSlot === null) return false;
    return gameStore.highlightedSlots.includes(slot);
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
</script>

<!-- Turn transition overlay -->
<TurnTransition
  isYourTurn={isPlayerTurn}
  turnNumber={gameState?.turn ?? 1}
  visible={showTurnTransition}
/>

<div class="w-full h-full flex bg-ui-bg">
  <!-- Main game area -->
  <div class="flex-1 flex flex-col">
    <!-- Top bar: Opponent info -->
    <div class="h-14 bg-ui-panel flex items-center justify-between px-6 border-b border-gray-700">
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <div class="w-3 h-3 rounded-full {!isPlayerTurn ? 'bg-ui-action animate-pulse' : 'bg-gray-600'}"></div>
          <span class="text-ui-text font-semibold">Opponent</span>
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

    <!-- Opponent hand (hidden cards) -->
    <div class="h-16 flex items-center justify-center gap-1.5 bg-gray-900/30 px-4">
      {#each gameState?.opponent.hand ?? [] as card, i}
        <HandCard {card} index={i} />
      {/each}
      {#if (gameState?.opponent.hand.length ?? 0) === 0}
        <span class="text-ui-text-dim text-sm">Empty hand</span>
      {/if}
    </div>

    <!-- Main board area -->
    <div class="flex-1 flex flex-col justify-center gap-6 px-6 py-4">
      <!-- Opponent's board -->
      <div class="flex items-center justify-center gap-4">
        <!-- Opponent supports -->
        <div class="flex flex-col gap-2">
          {#each gameState?.opponent.supports ?? [null, null] as support, i}
            <SupportSlot {support} slot={i} />
          {/each}
        </div>

        <!-- Opponent creatures -->
        <div class="flex gap-2">
          {#each gameState?.opponent.creatures ?? [null, null, null, null, null] as creature, i}
            <CreatureSlot
              {creature}
              slot={i}
              isPlayerSide={false}
              isValidTarget={isValidAttackTarget(i)}
              onClick={() => handleOpponentSlotClick(i)}
            />
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
          <span class="text-sm font-semibold {isPlayerTurn ? 'text-health' : 'text-ui-action'}">
            {isPlayerTurn ? "Your Turn" : "Opponent's Turn"}
          </span>
        </div>
        <div class="h-px flex-1 bg-gradient-to-r from-transparent via-gray-600 to-transparent"></div>
      </div>

      <!-- Player's board -->
      <div class="flex items-center justify-center gap-4">
        <!-- Player supports -->
        <div class="flex flex-col gap-2">
          {#each gameState?.player.supports ?? [null, null] as support, i}
            <SupportSlot
              {support}
              slot={i}
              isHighlighted={gameStore.highlightedSlots.includes(i) && gameStore.selectedCardIndex !== null}
              onClick={() => handlePlayerSlotClick(i)}
            />
          {/each}
        </div>

        <!-- Player creatures -->
        <div class="flex gap-2">
          {#each gameState?.player.creatures ?? [null, null, null, null, null] as creature, i}
            <CreatureSlot
              {creature}
              slot={i}
              isPlayerSide={true}
              isHighlighted={gameStore.highlightedSlots.includes(i) && gameStore.selectedCardIndex !== null}
              isSelected={gameStore.selectedCreatureSlot === i}
              onClick={() => handlePlayerSlotClick(i)}
            />
          {/each}
        </div>
      </div>
    </div>

    <!-- Player hand -->
    <div class="h-36 flex items-center justify-center gap-1.5 bg-gray-900/30 px-4 py-2">
      {#each gameState?.player.hand ?? [] as card, i}
        <HandCard
          {card}
          index={i}
          isSelected={gameStore.selectedCardIndex === i}
          isPlayable={isCardPlayable(i)}
          onClick={() => handleCardClick(i)}
        />
      {/each}
      {#if (gameState?.player.hand.length ?? 0) === 0}
        <span class="text-ui-text-dim text-sm">Empty hand</span>
      {/if}
    </div>

    <!-- Bottom bar: Player info -->
    <div class="h-14 bg-ui-panel flex items-center justify-between px-6 border-t border-gray-700">
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <div class="w-3 h-3 rounded-full {isPlayerTurn ? 'bg-health animate-pulse' : 'bg-gray-600'}"></div>
          <span class="text-ui-text font-semibold">You</span>
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
      <div class="flex items-center gap-3">
        {#if isPlayerTurn}
          <button
            class="px-4 py-2 bg-amber-700 text-white rounded-lg font-semibold
                   hover:bg-amber-600 active:scale-95 transition-all
                   disabled:opacity-50 disabled:cursor-not-allowed"
            onclick={() => gameStore.undoAction()}
            disabled={gameStore.isLoading || gameStore.actionHistory.length === 0}
            title="Undo last action (dev mode)"
          >
            Undo
          </button>
          <button
            class="px-5 py-2 bg-ui-action text-white rounded-lg font-semibold
                   hover:bg-ui-action/80 active:scale-95 transition-all
                   disabled:opacity-50 disabled:cursor-not-allowed"
            onclick={() => gameStore.endTurn()}
            disabled={gameStore.isLoading}
          >
            End Turn
          </button>
        {:else}
          <div class="px-5 py-2 bg-gray-700 text-ui-text-dim rounded-lg font-semibold flex items-center gap-2">
            <div class="w-4 h-4 border-2 border-ui-text-dim border-t-transparent rounded-full animate-spin"></div>
            AI Thinking...
          </div>
        {/if}
        <button
          class="px-4 py-2 bg-gray-700 text-ui-text rounded-lg hover:bg-gray-600 transition-colors"
          onclick={() => gameStore.quitGame()}
        >
          Quit
        </button>
      </div>
    </div>
  </div>

  <!-- Right sidebar: Hint Panel + Action Log -->
  <div class="w-64 border-l border-gray-700 bg-ui-panel/50 flex flex-col">
    <!-- AI Hint Panel (only when player's turn) -->
    {#if isPlayerTurn}
      <div class="p-3 border-b border-gray-700">
        <HintPanel
          hint={gameStore.currentHint}
          isLoading={gameStore.isHintLoading}
          onRequestHint={() => gameStore.requestHint()}
          onApplyHint={(action) => gameStore.applyHint(action)}
        />
      </div>
    {/if}

    <!-- Action Log -->
    <div class="flex-1 p-3 overflow-hidden">
      <ActionLog actions={actionsForLog} />
    </div>
  </div>
</div>
