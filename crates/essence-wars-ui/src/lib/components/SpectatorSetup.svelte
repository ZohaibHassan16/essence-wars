<script lang="ts">
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import type { SpectatorConfig } from "$lib/api/types";

  // Player 1 selections
  let player1Deck = $state("");
  let player1Bot = $state("greedy");

  // Player 2 selections
  let player2Deck = $state("");
  let player2Bot = $state("mcts");

  // Options
  let watchLive = $state(false);
  let showAdvanced = $state(false);
  let customSeed = $state("");

  // Bot configuration
  let mctsSimulations = $state(100);
  let alphabetaDepth = $state(4);

  // Check if any player is using MCTS or Alpha-Beta
  const usesMcts = $derived(player1Bot === "mcts" || player2Bot === "mcts");
  const usesAlphabeta = $derived(player1Bot === "alphabeta" || player2Bot === "alphabeta");

  // MCTS simulation presets
  const mctsPresets = [
    { value: 50, label: "50 (Fast)" },
    { value: 100, label: "100 (Default)" },
    { value: 250, label: "250 (Medium)" },
    { value: 500, label: "500 (Strong)" },
  ];

  // Alpha-Beta depth presets
  const alphabetaPresets = [
    { value: 2, label: "2 (Instant)" },
    { value: 4, label: "4 (Fast)" },
    { value: 6, label: "6 (Medium)" },
    { value: 8, label: "8 (Strong)" },
  ];

  function getFactionColor(faction: string): string {
    switch (faction) {
      case "argentum":
        return "border-argentum-gold text-argentum-gold";
      case "symbiote":
        return "border-symbiote-glow text-symbiote-glow";
      case "obsidion":
        return "border-obsidion-essence text-obsidion-essence";
      default:
        return "border-neutral-copper text-neutral-copper";
    }
  }

  async function startMatch() {
    if (!player1Deck || !player2Deck) return;

    const config: SpectatorConfig = {
      player1DeckId: player1Deck,
      player1BotType: player1Bot,
      player2DeckId: player2Deck,
      player2BotType: player2Bot,
      seed: customSeed ? parseInt(customSeed, 10) : undefined,
      mctsSimulations: usesMcts ? mctsSimulations : undefined,
      alphabetaDepth: usesAlphabeta ? alphabetaDepth : undefined,
    };

    spectatorStore.setWatchLive(watchLive);
    await spectatorStore.startMatch(config);
  }

  function goBack() {
    spectatorStore.phase = "setup";
    // Reset to menu by setting phase - the parent will handle navigation
    spectatorStore.reset();
  }

  // Group decks by faction
  const decksByFaction = $derived(() => {
    const grouped: Record<string, typeof spectatorStore.decks> = {};
    for (const deck of spectatorStore.decks) {
      if (!grouped[deck.faction]) {
        grouped[deck.faction] = [];
      }
      grouped[deck.faction].push(deck);
    }
    return grouped;
  });
</script>

<div class="min-h-screen flex flex-col items-center justify-center p-8">
  <h1 class="text-4xl font-bold text-ui-text mb-2">AI vs AI Spectator</h1>
  <p class="text-ui-text-dim mb-8">Watch two AIs battle it out</p>

  <div class="max-w-5xl w-full bg-ui-panel rounded-xl p-8 shadow-2xl">
    {#if spectatorStore.error}
      <div class="mb-4 p-4 bg-damage/20 border border-damage rounded text-damage">
        {spectatorStore.error}
      </div>
    {/if}

    <div class="grid grid-cols-2 gap-8">
      <!-- Player 1 Column -->
      <div class="border-r border-gray-700 pr-8">
        <h2 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
          <span class="w-8 h-8 rounded-full bg-health/20 text-health flex items-center justify-center text-sm font-bold">
            P1
          </span>
          Player 1
        </h2>

        <!-- Player 1 Deck Selection -->
        <div class="mb-4">
          <h3 class="text-sm font-semibold text-ui-text-dim mb-2">Deck</h3>
          <div class="space-y-1 max-h-48 overflow-y-auto pr-2">
            {#each Object.entries(decksByFaction()) as [faction, decks]}
              <div class="mb-2">
                <div class="text-xs text-ui-text-dim mb-1 capitalize">{faction}</div>
                {#each decks as deck}
                  <button
                    class="w-full text-left p-2 rounded border-2 mb-1 transition-all text-sm
                           {player1Deck === deck.id
                             ? getFactionColor(deck.faction) + ' bg-ui-bg'
                             : 'border-gray-600 hover:border-gray-500 bg-ui-bg/50'}"
                    onclick={() => (player1Deck = deck.id)}
                  >
                    <div class="font-semibold">{deck.name}</div>
                  </button>
                {/each}
              </div>
            {/each}
          </div>
        </div>

        <!-- Player 1 Bot Selection -->
        <div>
          <h3 class="text-sm font-semibold text-ui-text-dim mb-2">AI Bot</h3>
          <div class="space-y-1">
            {#each spectatorStore.bots as bot}
              <button
                class="w-full text-left p-2 rounded border-2 transition-all text-sm
                       {player1Bot === bot.id
                         ? 'border-health bg-health/10'
                         : 'border-gray-600 hover:border-gray-500 bg-ui-bg/50'}"
                onclick={() => (player1Bot = bot.id)}
              >
                <div class="font-semibold">{bot.name}</div>
                <div class="text-xs text-ui-text-dim">{bot.description}</div>
              </button>
            {/each}
          </div>
        </div>
      </div>

      <!-- Player 2 Column -->
      <div class="pl-0">
        <h2 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
          <span class="w-8 h-8 rounded-full bg-damage/20 text-damage flex items-center justify-center text-sm font-bold">
            P2
          </span>
          Player 2
        </h2>

        <!-- Player 2 Deck Selection -->
        <div class="mb-4">
          <h3 class="text-sm font-semibold text-ui-text-dim mb-2">Deck</h3>
          <div class="space-y-1 max-h-48 overflow-y-auto pr-2">
            {#each Object.entries(decksByFaction()) as [faction, decks]}
              <div class="mb-2">
                <div class="text-xs text-ui-text-dim mb-1 capitalize">{faction}</div>
                {#each decks as deck}
                  <button
                    class="w-full text-left p-2 rounded border-2 mb-1 transition-all text-sm
                           {player2Deck === deck.id
                             ? getFactionColor(deck.faction) + ' bg-ui-bg'
                             : 'border-gray-600 hover:border-gray-500 bg-ui-bg/50'}"
                    onclick={() => (player2Deck = deck.id)}
                  >
                    <div class="font-semibold">{deck.name}</div>
                  </button>
                {/each}
              </div>
            {/each}
          </div>
        </div>

        <!-- Player 2 Bot Selection -->
        <div>
          <h3 class="text-sm font-semibold text-ui-text-dim mb-2">AI Bot</h3>
          <div class="space-y-1">
            {#each spectatorStore.bots as bot}
              <button
                class="w-full text-left p-2 rounded border-2 transition-all text-sm
                       {player2Bot === bot.id
                         ? 'border-damage bg-damage/10'
                         : 'border-gray-600 hover:border-gray-500 bg-ui-bg/50'}"
                onclick={() => (player2Bot = bot.id)}
              >
                <div class="font-semibold">{bot.name}</div>
                <div class="text-xs text-ui-text-dim">{bot.description}</div>
              </button>
            {/each}
          </div>
        </div>
      </div>
    </div>

    <!-- Options -->
    <div class="mt-6 pt-6 border-t border-gray-700">
      <div class="flex items-center gap-6">
        <!-- Watch Live Toggle -->
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={watchLive}
            class="w-4 h-4 rounded border-gray-600 bg-ui-bg text-ui-action focus:ring-ui-action"
          />
          <span class="text-ui-text">Watch Live</span>
          <span class="text-xs text-ui-text-dim">(hide outcome until end)</span>
        </label>

        <!-- Advanced Options Toggle -->
        <button
          class="text-sm text-ui-text-dim hover:text-ui-text flex items-center gap-1"
          onclick={() => (showAdvanced = !showAdvanced)}
        >
          <span>{showAdvanced ? "Hide" : "Show"} Advanced</span>
          <span class="text-xs">{showAdvanced ? "▲" : "▼"}</span>
        </button>
      </div>

      {#if showAdvanced}
        <div class="mt-4 p-4 bg-ui-bg/50 rounded space-y-4">
          <!-- Bot Configuration -->
          <div class="grid grid-cols-2 gap-4">
            <!-- MCTS Simulations (only shown when MCTS is selected) -->
            {#if usesMcts}
              <div>
                <span class="text-sm text-ui-text-dim block mb-2">MCTS Simulations</span>
                <div class="flex flex-wrap gap-1">
                  {#each mctsPresets as preset}
                    <button
                      class="px-3 py-1.5 rounded text-sm transition-colors
                             {mctsSimulations === preset.value
                               ? 'bg-ui-action text-white'
                               : 'bg-ui-bg border border-gray-600 text-ui-text-dim hover:border-ui-action hover:text-ui-action'}"
                      onclick={() => (mctsSimulations = preset.value)}
                    >
                      {preset.label}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Alpha-Beta Depth (only shown when Alpha-Beta is selected) -->
            {#if usesAlphabeta}
              <div>
                <span class="text-sm text-ui-text-dim block mb-2">Alpha-Beta Depth</span>
                <div class="flex flex-wrap gap-1">
                  {#each alphabetaPresets as preset}
                    <button
                      class="px-3 py-1.5 rounded text-sm transition-colors
                             {alphabetaDepth === preset.value
                               ? 'bg-ui-action text-white'
                               : 'bg-ui-bg border border-gray-600 text-ui-text-dim hover:border-ui-action hover:text-ui-action'}"
                      onclick={() => (alphabetaDepth = preset.value)}
                    >
                      {preset.label}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <!-- Custom Seed -->
          <label class="block">
            <span class="text-sm text-ui-text-dim">Custom Seed (for reproducibility)</span>
            <input
              type="text"
              bind:value={customSeed}
              placeholder="Leave empty for random"
              class="mt-1 block w-full px-3 py-2 bg-ui-bg border border-gray-600 rounded
                     text-ui-text placeholder-ui-text-dim focus:border-ui-action focus:outline-none"
            />
          </label>
        </div>
      {/if}
    </div>

    <!-- Action Buttons -->
    <div class="mt-8 flex justify-between items-center">
      <button
        class="px-6 py-3 bg-ui-bg text-ui-text-dim rounded-lg font-semibold
               border border-gray-600 hover:border-gray-500 hover:text-ui-text transition-colors"
        onclick={goBack}
      >
        Back
      </button>

      <button
        class="px-8 py-4 bg-ui-action text-white rounded-lg font-bold text-lg
               hover:bg-ui-action/80 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        onclick={startMatch}
        disabled={!player1Deck || !player2Deck || spectatorStore.isComputing}
      >
        Start Match
      </button>
    </div>
  </div>
</div>
