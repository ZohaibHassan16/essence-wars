<script lang="ts">
  import type { DeckInfo, BotInfo } from "$lib/api/types";
  import { playSound } from "$lib/audio";
  import WizardStep from "./WizardStep.svelte";
  import FactionTabs from "./FactionTabs.svelte";
  import DeckGrid from "./DeckGrid.svelte";
  import DeckPreview from "./DeckPreview.svelte";

  let {
    mode = "human-vs-ai",
    decks,
    bots,
    onStart,
    onBack,
  }: {
    mode?: "human-vs-ai" | "ai-vs-ai";
    decks: DeckInfo[];
    bots: BotInfo[];
    onStart?: (config: WizardConfig) => void;
    onBack?: () => void;
  } = $props();

  // Wizard configuration output
  export interface WizardConfig {
    playerDeckId: string;
    opponentDeckId: string;
    playerBotType?: string; // Only for ai-vs-ai
    opponentBotType: string;
    playerGoesFirst: boolean;
    // Spectator options (ai-vs-ai only)
    watchLive?: boolean;
    enableCommentary?: boolean;
    mctsSimulations?: number;
    alphabetaDepth?: number;
    customSeed?: number;
  }

  // Wizard state
  let currentStep = $state(1);
  let selectedFaction1 = $state("argentum");
  let selectedFaction2 = $state("argentum");
  let selectedPlayerDeckId = $state<string | null>(null);
  let selectedOpponentDeckId = $state<string | null>(null);
  let selectedPlayerBot = $state("greedy");
  let selectedOpponentBot = $state("mcts");
  let playerGoesFirst = $state(true);

  // Spectator options (ai-vs-ai mode)
  let watchLive = $state(false);
  let enableCommentary = $state(true);
  let showAdvanced = $state(false);
  let customSeedStr = $state("");
  let mctsSimulations = $state(100);
  let alphabetaDepth = $state(4);

  // Check if any player is using MCTS or Alpha-Beta
  const usesMcts = $derived(selectedPlayerBot === "mcts" || selectedOpponentBot === "mcts");
  const usesAlphabeta = $derived(selectedPlayerBot === "alphabeta" || selectedOpponentBot === "alphabeta");

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

  // Direction for animation
  let slideDirection = $state<"left" | "right">("left");

  // Derived values
  const selectedPlayerDeck = $derived(
    decks.find((d) => d.id === selectedPlayerDeckId) ?? null
  );
  const selectedOpponentDeck = $derived(
    decks.find((d) => d.id === selectedOpponentDeckId) ?? null
  );

  const totalSteps = $derived(mode === "ai-vs-ai" ? 3 : 3);

  // Step titles based on mode
  const step1Title = $derived(mode === "human-vs-ai" ? "Choose Your Commander" : "Choose Player 1");
  const step2Title = $derived(mode === "human-vs-ai" ? "Choose Your Opponent" : "Choose Player 2");
  const step3Title = "Game Options";

  // Navigation functions
  function nextStep() {
    if (currentStep < totalSteps) {
      playSound("buttonClick");
      slideDirection = "left";
      currentStep++;
    }
  }

  function prevStep() {
    if (currentStep > 1) {
      playSound("buttonClick");
      slideDirection = "right";
      currentStep--;
    } else {
      onBack?.();
    }
  }

  function handleStart() {
    if (!selectedPlayerDeckId || !selectedOpponentDeckId) return;

    playSound("buttonClick");

    const config: WizardConfig = {
      playerDeckId: selectedPlayerDeckId,
      opponentDeckId: selectedOpponentDeckId,
      opponentBotType: selectedOpponentBot,
      playerGoesFirst,
    };

    if (mode === "ai-vs-ai") {
      config.playerBotType = selectedPlayerBot;
      // Include spectator options
      config.watchLive = watchLive;
      config.enableCommentary = enableCommentary;
      if (usesMcts) {
        config.mctsSimulations = mctsSimulations;
      }
      if (usesAlphabeta) {
        config.alphabetaDepth = alphabetaDepth;
      }
      if (customSeedStr) {
        const seed = parseInt(customSeedStr, 10);
        if (!isNaN(seed)) {
          config.customSeed = seed;
        }
      }
    }

    onStart?.(config);
  }

  // Can proceed to next step?
  const canProceedStep1 = $derived(selectedPlayerDeckId !== null);
  const canProceedStep2 = $derived(selectedOpponentDeckId !== null);
  const canStart = $derived(canProceedStep1 && canProceedStep2);

  // Faction color helper for selected deck display
  function getFactionTextColor(faction: string): string {
    switch (faction) {
      case "argentum": return "text-argentum-gold";
      case "symbiote": return "text-symbiote-glow";
      case "obsidion": return "text-obsidion-essence";
      default: return "text-neutral-copper";
    }
  }
</script>

<div class="deck-selection-wizard w-full h-full bg-ui-bg overflow-hidden">
  {#if currentStep === 1}
    <!-- Step 1: Player/P1 Deck Selection -->
    <div class="step-container" class:slide-in-left={slideDirection === "left"} class:slide-in-right={slideDirection === "right"}>
      <WizardStep
        stepNumber={1}
        {totalSteps}
        title={step1Title}
        subtitle="Step 1 of {totalSteps}"
      >
        <div class="flex h-full">
          <!-- Main selection area -->
          <div class="flex-1 flex flex-col">
            <!-- Faction tabs -->
            <div class="py-4">
              <FactionTabs
                selectedFaction={selectedFaction1}
                onSelect={(f) => selectedFaction1 = f}
              />
            </div>

            <!-- Deck grid -->
            <div class="flex-1 overflow-auto">
              <DeckGrid
                {decks}
                selectedFaction={selectedFaction1}
                selectedDeckId={selectedPlayerDeckId}
                onSelectDeck={(id) => selectedPlayerDeckId = id}
              />
            </div>
          </div>

          <!-- Preview panel -->
          <div class="w-[340px] border-l border-gray-700/50 p-4 bg-ui-panel/30 overflow-auto">
            <DeckPreview deck={selectedPlayerDeck} />
          </div>
        </div>

        {#snippet footer()}
          <div class="flex justify-between items-center">
            <button
              class="px-6 py-3 bg-ui-bg text-ui-text-dim rounded-lg font-semibold
                     border border-gray-600 hover:border-gray-500 hover:text-ui-text transition-all"
              onclick={prevStep}
              onmouseenter={() => playSound("buttonHover")}
            >
              <span class="flex items-center gap-2">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                </svg>
                Back
              </span>
            </button>

            <button
              class="px-8 py-3 bg-ui-action text-white rounded-lg font-bold text-lg
                     hover:bg-ui-action/80 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              onclick={nextStep}
              onmouseenter={() => playSound("buttonHover")}
              disabled={!canProceedStep1}
            >
              <span class="flex items-center gap-2">
                Next: Choose Opponent
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </span>
            </button>
          </div>
        {/snippet}
      </WizardStep>
    </div>

  {:else if currentStep === 2}
    <!-- Step 2: Opponent/P2 Deck Selection -->
    <div class="step-container" class:slide-in-left={slideDirection === "left"} class:slide-in-right={slideDirection === "right"}>
      <WizardStep
        stepNumber={2}
        {totalSteps}
        title={step2Title}
        subtitle="Step 2 of {totalSteps}"
      >
        <div class="flex h-full">
          <!-- Main selection area -->
          <div class="flex-1 flex flex-col">
            <!-- Previous selection summary -->
            {#if selectedPlayerDeck}
              <div class="px-4 py-2 bg-ui-panel/50 border-b border-gray-700/50">
                <span class="text-sm text-ui-text-dim">
                  {mode === "human-vs-ai" ? "You selected" : "Player 1"}:
                </span>
                <span class="ml-2 font-semibold {getFactionTextColor(selectedPlayerDeck.faction)}">
                  {selectedPlayerDeck.commander?.name ?? selectedPlayerDeck.name}
                </span>
              </div>
            {/if}

            <!-- Faction tabs -->
            <div class="py-4">
              <FactionTabs
                selectedFaction={selectedFaction2}
                onSelect={(f) => selectedFaction2 = f}
              />
            </div>

            <!-- Deck grid -->
            <div class="flex-1 overflow-auto">
              <DeckGrid
                {decks}
                selectedFaction={selectedFaction2}
                selectedDeckId={selectedOpponentDeckId}
                onSelectDeck={(id) => selectedOpponentDeckId = id}
              />
            </div>
          </div>

          <!-- Preview panel -->
          <div class="w-[340px] border-l border-gray-700/50 p-4 bg-ui-panel/30 overflow-auto">
            <DeckPreview deck={selectedOpponentDeck} />
          </div>
        </div>

        {#snippet footer()}
          <div class="flex justify-between items-center">
            <button
              class="px-6 py-3 bg-ui-bg text-ui-text-dim rounded-lg font-semibold
                     border border-gray-600 hover:border-gray-500 hover:text-ui-text transition-all"
              onclick={prevStep}
              onmouseenter={() => playSound("buttonHover")}
            >
              <span class="flex items-center gap-2">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                </svg>
                Back
              </span>
            </button>

            <button
              class="px-8 py-3 bg-ui-action text-white rounded-lg font-bold text-lg
                     hover:bg-ui-action/80 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              onclick={nextStep}
              onmouseenter={() => playSound("buttonHover")}
              disabled={!canProceedStep2}
            >
              <span class="flex items-center gap-2">
                Next: Game Options
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </span>
            </button>
          </div>
        {/snippet}
      </WizardStep>
    </div>

  {:else if currentStep === 3}
    <!-- Step 3: Game Options -->
    <div class="step-container" class:slide-in-left={slideDirection === "left"} class:slide-in-right={slideDirection === "right"}>
      <WizardStep
        stepNumber={3}
        {totalSteps}
        title={step3Title}
        subtitle="Step 3 of {totalSteps}"
      >
        <div class="max-w-2xl mx-auto p-8">
          <!-- Match Summary -->
          <div class="mb-8 p-6 rounded-xl bg-ui-panel/50 border border-gray-700">
            <h3 class="text-lg font-semibold text-ui-text mb-4">Match Summary</h3>
            <div class="flex items-center justify-center gap-8">
              <!-- Player 1 -->
              <div class="text-center">
                {#if selectedPlayerDeck?.commander?.portraitPath}
                  <img
                    src={`/${selectedPlayerDeck.commander.portraitPath}`}
                    alt={selectedPlayerDeck.commander.name}
                    class="w-20 h-20 rounded-lg object-cover mx-auto mb-2 border-2 {getFactionTextColor(selectedPlayerDeck.faction).replace('text-', 'border-')}"
                  />
                {/if}
                <div class="text-sm font-semibold {getFactionTextColor(selectedPlayerDeck?.faction ?? 'neutral')}">
                  {selectedPlayerDeck?.commander?.name ?? "Unknown"}
                </div>
                <div class="text-xs text-ui-text-dim mt-1">
                  {mode === "human-vs-ai" ? "You" : "Player 1"}
                </div>
              </div>

              <!-- VS -->
              <div class="text-2xl font-bold text-ui-text-dim">VS</div>

              <!-- Player 2 / Opponent -->
              <div class="text-center">
                {#if selectedOpponentDeck?.commander?.portraitPath}
                  <img
                    src={`/${selectedOpponentDeck.commander.portraitPath}`}
                    alt={selectedOpponentDeck.commander.name}
                    class="w-20 h-20 rounded-lg object-cover mx-auto mb-2 border-2 {getFactionTextColor(selectedOpponentDeck.faction).replace('text-', 'border-')}"
                  />
                {/if}
                <div class="text-sm font-semibold {getFactionTextColor(selectedOpponentDeck?.faction ?? 'neutral')}">
                  {selectedOpponentDeck?.commander?.name ?? "Unknown"}
                </div>
                <div class="text-xs text-ui-text-dim mt-1">
                  {mode === "human-vs-ai" ? "Opponent" : "Player 2"}
                </div>
              </div>
            </div>
          </div>

          <!-- Bot Selection -->
          <div class="mb-6">
            <h3 class="text-lg font-semibold text-ui-text mb-4">
              {mode === "ai-vs-ai" ? "AI Configuration" : "Opponent AI"}
            </h3>

            {#if mode === "ai-vs-ai"}
              <!-- Player 1 Bot -->
              <div class="mb-4">
                <div class="text-sm text-ui-text-dim mb-2">Player 1 Bot</div>
                <div class="grid grid-cols-2 gap-2">
                  {#each bots as bot}
                    <button
                      class="p-3 rounded-lg border-2 text-left transition-all
                             {selectedPlayerBot === bot.id
                               ? 'border-health bg-health/10 text-ui-text'
                               : 'border-gray-600 bg-ui-bg/50 text-ui-text-dim hover:border-gray-500'}"
                      onclick={() => { selectedPlayerBot = bot.id; playSound("buttonClick"); }}
                      onmouseenter={() => playSound("buttonHover")}
                    >
                      <div class="font-semibold text-sm">{bot.name}</div>
                      <div class="text-xs opacity-70 mt-0.5">{bot.description}</div>
                    </button>
                  {/each}
                </div>
              </div>

              <!-- Player 2 Bot -->
              <div>
                <div class="text-sm text-ui-text-dim mb-2">Player 2 Bot</div>
                <div class="grid grid-cols-2 gap-2">
                  {#each bots as bot}
                    <button
                      class="p-3 rounded-lg border-2 text-left transition-all
                             {selectedOpponentBot === bot.id
                               ? 'border-damage bg-damage/10 text-ui-text'
                               : 'border-gray-600 bg-ui-bg/50 text-ui-text-dim hover:border-gray-500'}"
                      onclick={() => { selectedOpponentBot = bot.id; playSound("buttonClick"); }}
                      onmouseenter={() => playSound("buttonHover")}
                    >
                      <div class="font-semibold text-sm">{bot.name}</div>
                      <div class="text-xs opacity-70 mt-0.5">{bot.description}</div>
                    </button>
                  {/each}
                </div>
              </div>
            {:else}
              <!-- Single bot selection for human-vs-ai -->
              <div class="grid grid-cols-2 gap-3">
                {#each bots as bot}
                  <button
                    class="p-4 rounded-lg border-2 text-left transition-all
                           {selectedOpponentBot === bot.id
                             ? 'border-ui-action bg-ui-action/10 text-ui-text'
                             : 'border-gray-600 bg-ui-bg/50 text-ui-text-dim hover:border-gray-500'}"
                    onclick={() => { selectedOpponentBot = bot.id; playSound("buttonClick"); }}
                    onmouseenter={() => playSound("buttonHover")}
                  >
                    <div class="font-semibold">{bot.name}</div>
                    <div class="text-xs opacity-70 mt-1">{bot.description}</div>
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Turn Order (human-vs-ai only) -->
          {#if mode === "human-vs-ai"}
            <div class="mb-6">
              <h3 class="text-lg font-semibold text-ui-text mb-4">Who Goes First?</h3>
              <div class="flex gap-4">
                <button
                  class="flex-1 p-4 rounded-lg border-2 transition-all text-center
                         {playerGoesFirst
                           ? 'border-health bg-health/10 text-ui-text'
                           : 'border-gray-600 bg-ui-bg/50 text-ui-text-dim hover:border-gray-500'}"
                  onclick={() => { playerGoesFirst = true; playSound("buttonClick"); }}
                  onmouseenter={() => playSound("buttonHover")}
                >
                  <div class="font-semibold">You</div>
                  <div class="text-xs opacity-70 mt-1">Go first</div>
                </button>
                <button
                  class="flex-1 p-4 rounded-lg border-2 transition-all text-center
                         {!playerGoesFirst
                           ? 'border-damage bg-damage/10 text-ui-text'
                           : 'border-gray-600 bg-ui-bg/50 text-ui-text-dim hover:border-gray-500'}"
                  onclick={() => { playerGoesFirst = false; playSound("buttonClick"); }}
                  onmouseenter={() => playSound("buttonHover")}
                >
                  <div class="font-semibold">Opponent</div>
                  <div class="text-xs opacity-70 mt-1">Goes first</div>
                </button>
              </div>
            </div>
          {/if}

          <!-- Spectator Options (ai-vs-ai only) -->
          {#if mode === "ai-vs-ai"}
            <div class="space-y-4">
              <!-- Watch Options -->
              <div class="flex items-center gap-6">
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    bind:checked={watchLive}
                    class="w-4 h-4 rounded border-gray-600 bg-ui-bg text-ui-action focus:ring-ui-action"
                  />
                  <span class="text-ui-text">Watch Live</span>
                  <span class="text-xs text-ui-text-dim">(hide outcome until end)</span>
                </label>

                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    bind:checked={enableCommentary}
                    class="w-4 h-4 rounded border-gray-600 bg-ui-bg text-ui-action focus:ring-ui-action"
                  />
                  <span class="text-ui-text">AI Commentary</span>
                  <span class="text-xs text-ui-text-dim">(analytical insights)</span>
                </label>
              </div>

              <!-- Advanced Options Toggle -->
              <button
                class="text-sm text-ui-text-dim hover:text-ui-text flex items-center gap-1"
                onclick={() => (showAdvanced = !showAdvanced)}
              >
                <span>{showAdvanced ? "Hide" : "Show"} Advanced Options</span>
                <span class="text-xs">{showAdvanced ? "▲" : "▼"}</span>
              </button>

              {#if showAdvanced}
                <div class="p-4 bg-ui-bg/50 rounded-lg space-y-4 border border-gray-700/50">
                  <!-- Bot Configuration -->
                  <div class="grid grid-cols-2 gap-4">
                    <!-- MCTS Simulations -->
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

                    <!-- Alpha-Beta Depth -->
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
                      bind:value={customSeedStr}
                      placeholder="Leave empty for random"
                      class="mt-1 block w-full px-3 py-2 bg-ui-bg border border-gray-600 rounded
                             text-ui-text placeholder-ui-text-dim focus:border-ui-action focus:outline-none"
                    />
                  </label>
                </div>
              {/if}
            </div>
          {/if}
        </div>

        {#snippet footer()}
          <div class="flex justify-between items-center">
            <button
              class="px-6 py-3 bg-ui-bg text-ui-text-dim rounded-lg font-semibold
                     border border-gray-600 hover:border-gray-500 hover:text-ui-text transition-all"
              onclick={prevStep}
              onmouseenter={() => playSound("buttonHover")}
            >
              <span class="flex items-center gap-2">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                </svg>
                Back
              </span>
            </button>

            <button
              class="px-10 py-4 bg-ui-action text-white rounded-lg font-bold text-xl
                     hover:bg-ui-action/80 hover:scale-105 transition-all disabled:opacity-50 disabled:cursor-not-allowed
                     shadow-lg shadow-ui-action/30"
              onclick={handleStart}
              onmouseenter={() => playSound("buttonHover")}
              disabled={!canStart}
            >
              {mode === "ai-vs-ai" ? "Start Match" : "Start Game"}
            </button>
          </div>
        {/snippet}
      </WizardStep>
    </div>
  {/if}
</div>

<style>
  .step-container {
    height: 100%;
  }

  .slide-in-left {
    animation: slideInLeft 0.3s ease-out;
  }

  .slide-in-right {
    animation: slideInRight 0.3s ease-out;
  }

  @keyframes slideInLeft {
    from {
      opacity: 0;
      transform: translateX(30px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  @keyframes slideInRight {
    from {
      opacity: 0;
      transform: translateX(-30px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }
</style>
