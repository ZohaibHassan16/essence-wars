<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";
  import CreatureSlot from "./CreatureSlot.svelte";
  import SupportSlot from "./SupportSlot.svelte";
  import HandCard from "./HandCard.svelte";

  const state = $derived(gameStore.gameState);
  const isPlayerTurn = $derived(gameStore.isPlayerTurn);

  function handlePlayerSlotClick(slot: number) {
    if (!isPlayerTurn) return;

    const action = gameStore.getActionForTarget(slot);
    if (action) {
      gameStore.applyAction(action.index);
    } else if (gameStore.selectedCardIndex === null) {
      // Select creature for attack
      const creature = state?.player.creatures[slot];
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
</script>

<div class="w-full h-full flex flex-col bg-ui-bg">
  <!-- Top bar: Opponent info -->
  <div class="h-16 bg-ui-panel flex items-center justify-between px-6 border-b border-gray-700">
    <div class="flex items-center gap-4">
      <div class="text-ui-text font-semibold">Opponent</div>
      <div class="flex gap-2 text-sm">
        <span class="text-health">{state?.opponent.life ?? 0} HP</span>
        <span class="text-mana">{state?.opponent.essence ?? 0}/{state?.opponent.maxEssence ?? 0} Essence</span>
        <span class="text-gold">{state?.opponent.actionPoints ?? 0} AP</span>
      </div>
    </div>
    <div class="flex items-center gap-4">
      <div class="text-sm text-ui-text-dim">
        Deck: {state?.opponent.deckCount ?? 0} | Hand: {state?.opponent.hand.length ?? 0}
      </div>
    </div>
  </div>

  <!-- Opponent hand (hidden cards) -->
  <div class="h-20 flex items-center justify-center gap-2 bg-ui-bg/50">
    {#each state?.opponent.hand ?? [] as card, i}
      <HandCard {card} index={i} />
    {/each}
  </div>

  <!-- Main board area -->
  <div class="flex-1 flex flex-col justify-center gap-8 px-8">
    <!-- Opponent's board -->
    <div class="flex items-center justify-center gap-6">
      <!-- Opponent supports -->
      <div class="flex flex-col gap-2">
        {#each state?.opponent.supports ?? [null, null] as support, i}
          <SupportSlot {support} slot={i} />
        {/each}
      </div>

      <!-- Opponent creatures -->
      <div class="flex gap-3">
        {#each state?.opponent.creatures ?? [null, null, null, null, null] as creature, i}
          <CreatureSlot
            {creature}
            slot={i}
            isPlayerSide={false}
            isHighlighted={gameStore.highlightedSlots.includes(i) && gameStore.selectedCreatureSlot !== null}
            onClick={() => handleOpponentSlotClick(i)}
          />
        {/each}
      </div>
    </div>

    <!-- Center divider with turn info -->
    <div class="flex items-center justify-center gap-4">
      <div class="h-px flex-1 bg-gray-700"></div>
      <div class="px-4 py-2 rounded bg-ui-panel text-sm">
        Turn {state?.turn ?? 0} - {isPlayerTurn ? "Your Turn" : "Opponent's Turn"}
      </div>
      <div class="h-px flex-1 bg-gray-700"></div>
    </div>

    <!-- Player's board -->
    <div class="flex items-center justify-center gap-6">
      <!-- Player supports -->
      <div class="flex flex-col gap-2">
        {#each state?.player.supports ?? [null, null] as support, i}
          <SupportSlot
            {support}
            slot={i}
            isHighlighted={gameStore.highlightedSlots.includes(i) && gameStore.selectedCardIndex !== null}
            onClick={() => handlePlayerSlotClick(i)}
          />
        {/each}
      </div>

      <!-- Player creatures -->
      <div class="flex gap-3">
        {#each state?.player.creatures ?? [null, null, null, null, null] as creature, i}
          <CreatureSlot
            {creature}
            slot={i}
            isPlayerSide={true}
            isHighlighted={gameStore.highlightedSlots.includes(i)}
            isSelected={gameStore.selectedCreatureSlot === i}
            onClick={() => handlePlayerSlotClick(i)}
          />
        {/each}
      </div>
    </div>
  </div>

  <!-- Player hand -->
  <div class="h-36 flex items-center justify-center gap-2 bg-ui-bg/50 px-4">
    {#each state?.player.hand ?? [] as card, i}
      <HandCard
        {card}
        index={i}
        isSelected={gameStore.selectedCardIndex === i}
        isPlayable={isCardPlayable(i)}
        onClick={() => handleCardClick(i)}
      />
    {/each}
  </div>

  <!-- Bottom bar: Player info -->
  <div class="h-16 bg-ui-panel flex items-center justify-between px-6 border-t border-gray-700">
    <div class="flex items-center gap-4">
      <div class="text-ui-text font-semibold">You</div>
      <div class="flex gap-2 text-sm">
        <span class="text-health">{state?.player.life ?? 0} HP</span>
        <span class="text-mana">{state?.player.essence ?? 0}/{state?.player.maxEssence ?? 0} Essence</span>
        <span class="text-gold">{state?.player.actionPoints ?? 0} AP</span>
      </div>
    </div>
    <div class="flex items-center gap-4">
      <div class="text-sm text-ui-text-dim">
        Deck: {state?.player.deckCount ?? 0}
      </div>
      {#if isPlayerTurn}
        <button
          class="px-4 py-2 bg-ui-action text-white rounded font-semibold hover:bg-ui-action/80 transition-colors"
          onclick={() => gameStore.endTurn()}
          disabled={gameStore.isLoading}
        >
          End Turn
        </button>
      {/if}
      <button
        class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 transition-colors"
        onclick={() => gameStore.quitGame()}
      >
        Quit
      </button>
    </div>
  </div>
</div>
