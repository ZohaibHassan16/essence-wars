<script lang="ts">
  interface PlayerInfo {
    name: string;
    life: number;
    type: string; // "human", "ai", or bot name
  }

  interface Props {
    mode: "player" | "spectator" | "replay";
    winner: 1 | 2 | null;
    player1: PlayerInfo;
    player2: PlayerInfo;
    turn: number;
    reason?: string;
    onPlayAgain?: () => void;
    onMainMenu: () => void;
    onSaveReplay?: () => Promise<void>;
    onReviewMatch?: () => void;
    replaySaved?: boolean;
  }

  let {
    mode,
    winner,
    player1,
    player2,
    turn,
    reason,
    onPlayAgain,
    onMainMenu,
    onSaveReplay,
    onReviewMatch,
    replaySaved = false,
  }: Props = $props();

  // For player mode: winner=1 means player wins, winner=2 means AI wins
  // For spectator/replay: winner=1 means P1 wins, winner=2 means P2 wins
  const isPlayerMode = $derived(mode === "player");
  const didWin = $derived(isPlayerMode && winner === 1);
  const didLose = $derived(isPlayerMode && winner === 2);
  const isDraw = $derived(winner === null);

  // Save state (internal to component)
  let isSaving = $state(false);
  let saveSuccess = $state(false);
  let saveError = $state<string | null>(null);

  // Sync external replaySaved state when it changes
  $effect(() => {
    saveSuccess = replaySaved;
  });

  async function handleSaveReplay() {
    if (!onSaveReplay) return;

    isSaving = true;
    saveError = null;

    try {
      await onSaveReplay();
      saveSuccess = true;
    } catch (e) {
      saveError = e instanceof Error ? e.message : String(e);
    } finally {
      isSaving = false;
    }
  }

  function formatReason(reasonText: string | undefined): string {
    if (!reasonText) return "Game Over";
    // Convert snake_case or CamelCase to readable text
    return reasonText
      .replace(/([A-Z])/g, " $1")
      .replace(/_/g, " ")
      .replace(/^\s+/, "")
      .toLowerCase()
      .replace(/^./, (c) => c.toUpperCase());
  }

  // Determine colors and styling based on mode and winner
  function getResultColor(): string {
    if (isPlayerMode) {
      return didWin ? "text-health" : isDraw ? "text-gold" : "text-damage";
    }
    // Spectator/replay mode
    if (isDraw) return "text-gold";
    return winner === 1 ? "text-health" : "text-damage";
  }

  function getBgGlowColor(): string {
    if (isPlayerMode) {
      return didWin ? "bg-health" : isDraw ? "bg-gold" : "bg-damage";
    }
    if (isDraw) return "bg-gold";
    return winner === 1 ? "bg-health" : "bg-damage";
  }

  function getResultText(): string {
    if (isPlayerMode) {
      if (didWin) return "VICTORY";
      if (isDraw) return "DRAW";
      return "DEFEAT";
    }
    // Spectator/replay mode
    if (isDraw) return "DRAW";
    return winner === 1 ? "P1 WINS" : "P2 WINS";
  }

  function getResultIcon(): string {
    if (isPlayerMode) {
      if (didWin) return "🏆";
      if (isDraw) return "🤝";
      return "💀";
    }
    // Spectator/replay mode
    if (isDraw) return "🤝";
    return winner === 1 ? "🏆" : "🏆";
  }
</script>

<div class="min-h-screen flex flex-col items-center justify-center p-8 bg-ui-bg overflow-hidden">
  <!-- Animated background glow -->
  <div class="absolute inset-0 overflow-hidden pointer-events-none">
    <div
      class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] rounded-full blur-[120px] animate-pulse opacity-30
             {getBgGlowColor()}"
    ></div>
  </div>

  <div class="text-center relative z-10 animate-fade-in">
    <!-- Mode badge for spectator/replay -->
    {#if mode === "spectator"}
      <div class="mb-4">
        <span class="px-3 py-1 bg-purple-600/30 text-purple-400 rounded-full text-sm font-semibold border border-purple-500/50">
          AI vs AI Match
        </span>
      </div>
    {:else if mode === "replay"}
      <div class="mb-4">
        <span class="px-3 py-1 bg-amber-600/30 text-amber-400 rounded-full text-sm font-semibold border border-amber-500/50">
          Replay
        </span>
      </div>
    {/if}

    <!-- Large icon -->
    <div class="text-8xl mb-6 animate-bounce-slow">
      {getResultIcon()}
    </div>

    <h1 class="text-7xl font-black mb-2 tracking-tight {getResultColor()}">
      {getResultText()}
    </h1>

    <div class="text-xl text-ui-text-dim mb-10">
      {formatReason(reason)}
    </div>

    <!-- Stats card -->
    <div class="bg-ui-panel/80 backdrop-blur-sm rounded-2xl p-8 mb-10 inline-block border border-gray-700 shadow-2xl">
      <div class="grid grid-cols-3 gap-10 text-lg">
        <div class="text-center">
          <div class="text-ui-text-dim text-sm uppercase tracking-wider mb-2">
            {isPlayerMode ? "You" : player1.name}
          </div>
          <div class="text-4xl font-bold {player1.life > 0 ? 'text-health' : 'text-damage'}">
            {player1.life}
          </div>
          <div class="text-ui-text-dim text-xs mt-1">
            {isPlayerMode ? "Life" : player1.type}
          </div>
        </div>
        <div class="text-center border-x border-gray-700 px-8">
          <div class="text-ui-text-dim text-sm uppercase tracking-wider mb-2">Turn</div>
          <div class="text-4xl font-bold text-ui-text">{turn}</div>
          <div class="text-ui-text-dim text-xs mt-1">Final</div>
        </div>
        <div class="text-center">
          <div class="text-ui-text-dim text-sm uppercase tracking-wider mb-2">
            {isPlayerMode ? "Opponent" : player2.name}
          </div>
          <div class="text-4xl font-bold {player2.life > 0 ? 'text-health' : 'text-damage'}">
            {player2.life}
          </div>
          <div class="text-ui-text-dim text-xs mt-1">
            {isPlayerMode ? "Life" : player2.type}
          </div>
        </div>
      </div>
    </div>

    <!-- Action buttons -->
    <div class="flex flex-col items-center gap-4">
      <div class="flex gap-6 justify-center">
        {#if onPlayAgain}
          <button
            class="group px-10 py-4 bg-gradient-to-r from-ui-action to-purple-600 text-white rounded-xl font-bold text-lg
                   hover:from-ui-action/90 hover:to-purple-500 transition-all transform hover:scale-105 hover:shadow-lg hover:shadow-ui-action/30"
            onclick={onPlayAgain}
          >
            <span class="flex items-center gap-2">
              <span>{isPlayerMode ? "Play Again" : "Watch Another"}</span>
              <span class="group-hover:translate-x-1 transition-transform">→</span>
            </span>
          </button>
        {/if}
        <button
          class="px-10 py-4 bg-gray-700 text-white rounded-xl font-bold text-lg
                 hover:bg-gray-600 transition-all transform hover:scale-105"
          onclick={onMainMenu}
        >
          Main Menu
        </button>
      </div>

      <!-- Secondary actions row -->
      <div class="flex gap-4 justify-center">
        <!-- Save Replay button -->
        {#if onSaveReplay}
          <div class="flex flex-col items-center gap-1">
            <button
              class="px-6 py-2 bg-ui-panel text-ui-text rounded-lg font-semibold text-sm
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
            {#if saveSuccess}
              <div class="text-health text-xs">Replay saved!</div>
            {:else if saveError}
              <div class="text-damage text-xs">{saveError}</div>
            {/if}
          </div>
        {/if}

        <!-- Review Match button (for spectator/replay modes) -->
        {#if onReviewMatch}
          <button
            class="px-6 py-2 bg-ui-panel text-ui-text rounded-lg font-semibold text-sm
                   border border-gray-600 hover:border-ui-action hover:text-ui-action transition-all"
            onclick={onReviewMatch}
          >
            Review Match
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @keyframes bounce-slow {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-10px);
    }
  }

  .animate-fade-in {
    animation: fade-in 0.6s ease-out forwards;
  }

  .animate-bounce-slow {
    animation: bounce-slow 2s ease-in-out infinite;
  }
</style>
