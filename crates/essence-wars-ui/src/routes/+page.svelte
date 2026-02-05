<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { gameStore } from "$lib/stores/gameState.svelte";
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import { replayStore } from "$lib/stores/replayState.svelte";
  import { mcpSyncStore } from "$lib/stores/mcpSyncState.svelte";
  import { deckBuilderStore } from "$lib/stores/deckBuilderState.svelte";
  import * as api from "$lib/api/game";
  import { preloadSounds } from "$lib/audio";
  import MainMenu from "$lib/components/MainMenu.svelte";
  import SetupScreen from "$lib/components/SetupScreen.svelte";
  import GameBoard from "$lib/components/GameBoard.svelte";
  import GameOverScreen from "$lib/components/GameOverScreen.svelte";
  import SpectatorSetup from "$lib/components/SpectatorSetup.svelte";
  import ComputingMatch from "$lib/components/ComputingMatch.svelte";
  import SpectatorPlayback from "$lib/components/SpectatorPlayback.svelte";
  import ReplayBrowser from "$lib/components/ReplayBrowser.svelte";
  import ReplayPlayback from "$lib/components/ReplayPlayback.svelte";
  import McpSyncView from "$lib/components/McpSyncView.svelte";
  import SettingsScreen from "$lib/components/SettingsScreen.svelte";
  import RulesScreen from "$lib/components/RulesScreen.svelte";
  import LoreScreen from "$lib/components/LoreScreen.svelte";
  import MatchStatsSummary from "$lib/components/stats/MatchStatsSummary.svelte";
  import MatchCompareView from "$lib/components/stats/MatchCompareView.svelte";
  import { DeckBuilderScreen } from "$lib/components/deckbuilder";
  import { comparisonStore } from "$lib/stores/comparisonState.svelte";
  import { audioSettings } from "$lib/stores/audioSettings.svelte";

  // Overlay screen states
  let showSettings = $state(false);
  let showRules = $state(false);
  let showLore = $state(false);

  // Global keyboard handler
  function handleKeydown(event: KeyboardEvent) {
    // Skip if user is in an input field
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
      return;
    }

    // Global shortcuts (work anywhere)
    switch (event.key.toLowerCase()) {
      case 'm':
        event.preventDefault();
        audioSettings.toggleMute();
        return;
    }

    // Gameplay shortcuts (only during playing phase, player's turn, not loading)
    if (gameStore.phase === "playing" && gameStore.isPlayerTurn && !gameStore.isLoading) {
      switch (event.key.toLowerCase()) {
        case ' ':
          event.preventDefault();
          gameStore.endTurn();
          return;
        case 'escape':
          event.preventDefault();
          gameStore.clearSelection();
          return;
        case 'h':
          event.preventDefault();
          gameStore.requestHint();
          return;
        case 'i':
          event.preventDefault();
          if (gameStore.insightAvailable) {
            gameStore.applyCommanderInsight();
          }
          return;
        // Number keys 1-5 for selecting player creatures
        case '1':
        case '2':
        case '3':
        case '4':
        case '5': {
          event.preventDefault();
          const slot = parseInt(event.key) - 1;
          const creature = gameStore.gameState?.player.creatures[slot];
          if (creature?.canAttack) {
            gameStore.selectCreature(slot);
          }
          return;
        }
        // Q-U for selecting cards in hand (positions 0-6)
        case 'q':
        case 'w':
        case 'e':
        case 'r':
        case 't':
        case 'y':
        case 'u': {
          event.preventDefault();
          const keyMap: Record<string, number> = { q: 0, w: 1, e: 2, r: 3, t: 4, y: 5, u: 6 };
          const cardIndex = keyMap[event.key.toLowerCase()];
          const hand = gameStore.gameState?.player.hand ?? [];
          if (cardIndex < hand.length) {
            gameStore.selectCard(cardIndex);
          }
          return;
        }
      }
    }
  }

  // Initialize on app mount
  onMount(() => {
    preloadSounds();
    // Start MCP auto-watch for auto-switching when MCP syncs state
    mcpSyncStore.startAutoWatch();
    // Add global keyboard listener
    window.addEventListener('keydown', handleKeydown);
  });

  // Cleanup on app unmount
  onDestroy(() => {
    mcpSyncStore.cleanup();
    window.removeEventListener('keydown', handleKeydown);
  });

  // Helper functions to build props for GameOverScreen

  function getPlayerGameOverProps() {
    const state = gameStore.gameState;
    // Cast winner to the expected type (1 | 2 | null)
    const winner = state?.winner === 1 ? 1 : state?.winner === 2 ? 2 : null;
    return {
      mode: "player" as const,
      winner: winner as 1 | 2 | null,
      player1: {
        name: "You",
        life: state?.player.life ?? 0,
        type: "human",
      },
      player2: {
        name: "Opponent",
        life: state?.opponent.life ?? 0,
        type: "ai",
      },
      turn: state?.turn ?? 0,
      reason: state?.gameOverReason,
      onPlayAgain: () => {
        gameStore.quitGame();
        gameStore.loadDecksAndBots();
      },
      onMainMenu: () => gameStore.quitGame(),
      onSaveReplay: state?.id ? async () => {
        await api.saveReplay(state.id);
      } : undefined,
      // Pass action and event history for detailed stats
      actionHistory: gameStore.actionHistory,
      eventHistory: gameStore.eventHistory,
    };
  }

  function getSpectatorGameOverProps() {
    const match = spectatorStore.match;
    const finalState = match?.actions[match.actions.length - 1]?.stateAfter ?? match?.initialState;
    return {
      mode: "spectator" as const,
      winner: match?.result.winner ?? null,
      player1: {
        name: match?.player1DeckName ?? "Player 1",
        life: finalState?.player.life ?? 0,
        type: match?.player1BotName ?? "AI",
      },
      player2: {
        name: match?.player2DeckName ?? "Player 2",
        life: finalState?.opponent.life ?? 0,
        type: match?.player2BotName ?? "AI",
      },
      turn: finalState?.turn ?? 0,
      reason: match?.result.reason,
      onPlayAgain: () => {
        spectatorStore.reset();
      },
      onMainMenu: () => spectatorStore.backToMenu(),
      onSaveReplay: match ? async () => {
        await api.saveSpectatorReplay(match);
      } : undefined,
      onReviewMatch: () => spectatorStore.backToWatching(),
      onViewStatistics: spectatorStore.matchStatistics ? () => spectatorStore.openStatsSummary() : undefined,
    };
  }

  function getReplayGameOverProps() {
    const match = replayStore.match;
    const finalState = match?.actions[match.actions.length - 1]?.stateAfter ?? match?.initialState;
    return {
      mode: "replay" as const,
      winner: match?.result.winner ?? null,
      player1: {
        name: match?.player1DeckName ?? "Player 1",
        life: finalState?.player.life ?? 0,
        type: match?.player1BotName ?? "Human",
      },
      player2: {
        name: match?.player2DeckName ?? "Player 2",
        life: finalState?.opponent.life ?? 0,
        type: match?.player2BotName ?? "AI",
      },
      turn: finalState?.turn ?? 0,
      reason: match?.result.reason,
      onPlayAgain: () => {
        replayStore.backToBrowser();
      },
      onMainMenu: () => replayStore.reset(),
      onReviewMatch: () => replayStore.backToWatching(),
      // No save replay for replays - they're already saved
    };
  }
</script>

<!-- Overlay screens (Settings, Rules, and Lore) -->
{#if showSettings}
  <SettingsScreen onBack={() => showSettings = false} />
{:else if showRules}
  <RulesScreen onBack={() => showRules = false} />
{:else if showLore}
  <LoreScreen onBack={() => showLore = false} />
<!-- Comparison mode -->
{:else if comparisonStore.phase !== "idle"}
  <MatchCompareView />
<!-- Deck Builder mode -->
{:else if deckBuilderStore.phase !== "closed"}
  <DeckBuilderScreen />
<!-- MCP Sync mode takes top precedence when active -->
{:else if mcpSyncStore.phase === "watching" || mcpSyncStore.phase === "disconnected"}
  <McpSyncView />
<!-- Replay mode when active -->
{:else if replayStore.phase === "browser" || replayStore.phase === "loading"}
  <ReplayBrowser />
{:else if replayStore.phase === "watching" || replayStore.phase === "finished"}
  <ReplayPlayback />
{:else if replayStore.phase === "gameOver"}
  <GameOverScreen {...getReplayGameOverProps()} />
<!-- Spectator mode when active -->
{:else if spectatorStore.phase === "setup" && spectatorStore.decks.length > 0}
  <SpectatorSetup />
{:else if spectatorStore.phase === "computing"}
  <ComputingMatch />
{:else if spectatorStore.phase === "watching" || spectatorStore.phase === "finished"}
  <SpectatorPlayback />
{:else if spectatorStore.phase === "gameOver"}
  <GameOverScreen {...getSpectatorGameOverProps()} />
<!-- Game mode -->
{:else if gameStore.phase === "menu"}
  <MainMenu onOpenSettings={() => showSettings = true} onOpenRules={() => showRules = true} onOpenLore={() => showLore = true} />
{:else if gameStore.phase === "setup"}
  <SetupScreen />
{:else if gameStore.phase === "playing"}
  <GameBoard />
{:else if gameStore.phase === "gameOver"}
  <GameOverScreen {...getPlayerGameOverProps()} />
{/if}

<!-- Statistics modal (overlay for spectator mode) -->
{#if spectatorStore.showStatsSummary && spectatorStore.match && spectatorStore.matchStatistics}
  <MatchStatsSummary
    match={spectatorStore.match}
    statistics={spectatorStore.matchStatistics}
    onClose={() => spectatorStore.closeStatsSummary()}
  />
{/if}
