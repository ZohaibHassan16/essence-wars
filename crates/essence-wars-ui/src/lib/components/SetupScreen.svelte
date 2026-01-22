<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";

  let selectedPlayerDeck = $state("");
  let selectedOpponentDeck = $state("");
  let selectedBot = $state("greedy");
  let playerGoesFirst = $state(true);

  function getFactionColor(faction: string): string {
    switch (faction) {
      case "argentum": return "border-argentum-gold text-argentum-gold";
      case "symbiote": return "border-symbiote-glow text-symbiote-glow";
      case "obsidion": return "border-obsidion-essence text-obsidion-essence";
      default: return "border-neutral-copper text-neutral-copper";
    }
  }

  async function startGame() {
    if (!selectedPlayerDeck || !selectedOpponentDeck) return;
    await gameStore.startGame(
      selectedPlayerDeck,
      selectedOpponentDeck,
      selectedBot,
      playerGoesFirst
    );
  }

  // Group decks by faction
  const decksByFaction = $derived(() => {
    const grouped: Record<string, typeof gameStore.decks> = {};
    for (const deck of gameStore.decks) {
      if (!grouped[deck.faction]) {
        grouped[deck.faction] = [];
      }
      grouped[deck.faction].push(deck);
    }
    return grouped;
  });
</script>

<div class="min-h-screen flex flex-col items-center justify-center p-8">
  <h1 class="text-4xl font-bold text-ui-text mb-8">Essence Wars</h1>

  <div class="max-w-4xl w-full bg-ui-panel rounded-xl p-8 shadow-2xl">
    <h2 class="text-2xl font-semibold text-ui-text mb-6">New Game Setup</h2>

    {#if gameStore.error}
      <div class="mb-4 p-4 bg-damage/20 border border-damage rounded text-damage">
        {gameStore.error}
      </div>
    {/if}

    <div class="grid grid-cols-2 gap-8">
      <!-- Player Deck Selection -->
      <div>
        <h3 class="text-lg font-semibold text-ui-text mb-3">Your Deck</h3>
        <div class="space-y-2 max-h-64 overflow-y-auto pr-2">
          {#each Object.entries(decksByFaction()) as [faction, decks]}
            <div class="mb-3">
              <div class="text-sm text-ui-text-dim mb-1 capitalize">{faction}</div>
              {#each decks as deck}
                <button
                  class="w-full text-left p-3 rounded border-2 mb-1 transition-all
                         {selectedPlayerDeck === deck.id
                           ? getFactionColor(deck.faction) + ' bg-ui-bg'
                           : 'border-gray-600 hover:border-gray-500 bg-ui-bg/50'}"
                  onclick={() => selectedPlayerDeck = deck.id}
                >
                  <div class="font-semibold">{deck.name}</div>
                  <div class="text-xs text-ui-text-dim">{deck.cardCount} cards</div>
                </button>
              {/each}
            </div>
          {/each}
        </div>
      </div>

      <!-- Opponent Deck Selection -->
      <div>
        <h3 class="text-lg font-semibold text-ui-text mb-3">Opponent Deck</h3>
        <div class="space-y-2 max-h-64 overflow-y-auto pr-2">
          {#each Object.entries(decksByFaction()) as [faction, decks]}
            <div class="mb-3">
              <div class="text-sm text-ui-text-dim mb-1 capitalize">{faction}</div>
              {#each decks as deck}
                <button
                  class="w-full text-left p-3 rounded border-2 mb-1 transition-all
                         {selectedOpponentDeck === deck.id
                           ? getFactionColor(deck.faction) + ' bg-ui-bg'
                           : 'border-gray-600 hover:border-gray-500 bg-ui-bg/50'}"
                  onclick={() => selectedOpponentDeck = deck.id}
                >
                  <div class="font-semibold">{deck.name}</div>
                  <div class="text-xs text-ui-text-dim">{deck.cardCount} cards</div>
                </button>
              {/each}
            </div>
          {/each}
        </div>
      </div>
    </div>

    <!-- Bot Selection -->
    <div class="mt-6">
      <h3 class="text-lg font-semibold text-ui-text mb-3">Opponent AI</h3>
      <div class="flex gap-4">
        {#each gameStore.bots as bot}
          <button
            class="flex-1 p-4 rounded border-2 transition-all text-left
                   {selectedBot === bot.id
                     ? 'border-ui-action bg-ui-action/10'
                     : 'border-gray-600 hover:border-gray-500 bg-ui-bg/50'}"
            onclick={() => selectedBot = bot.id}
          >
            <div class="font-semibold">{bot.name}</div>
            <div class="text-xs text-ui-text-dim mt-1">{bot.description}</div>
          </button>
        {/each}
      </div>
    </div>

    <!-- First Turn Selection -->
    <div class="mt-6">
      <h3 class="text-lg font-semibold text-ui-text mb-3">Who Goes First?</h3>
      <div class="flex gap-4">
        <button
          class="flex-1 p-4 rounded border-2 transition-all
                 {playerGoesFirst
                   ? 'border-health bg-health/10'
                   : 'border-gray-600 hover:border-gray-500 bg-ui-bg/50'}"
          onclick={() => playerGoesFirst = true}
        >
          You
        </button>
        <button
          class="flex-1 p-4 rounded border-2 transition-all
                 {!playerGoesFirst
                   ? 'border-damage bg-damage/10'
                   : 'border-gray-600 hover:border-gray-500 bg-ui-bg/50'}"
          onclick={() => playerGoesFirst = false}
        >
          Opponent
        </button>
      </div>
    </div>

    <!-- Start Button -->
    <div class="mt-8 flex justify-center">
      <button
        class="px-8 py-4 bg-ui-action text-white rounded-lg font-bold text-lg
               hover:bg-ui-action/80 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        onclick={startGame}
        disabled={!selectedPlayerDeck || !selectedOpponentDeck || gameStore.isLoading}
      >
        {#if gameStore.isLoading}
          Starting...
        {:else}
          Start Game
        {/if}
      </button>
    </div>
  </div>
</div>
