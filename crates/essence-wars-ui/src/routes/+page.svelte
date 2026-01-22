<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import { replayStore } from "$lib/stores/replayState.svelte";
  import { mcpSyncStore } from "$lib/stores/mcpSyncState.svelte";
  import * as api from "$lib/api/game";
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

<!-- MCP Sync mode takes top precedence when active -->
{#if mcpSyncStore.phase === "watching" || mcpSyncStore.phase === "disconnected"}
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
  <MainMenu />
{:else if gameStore.phase === "setup"}
  <SetupScreen />
{:else if gameStore.phase === "playing"}
  <GameBoard />
{:else if gameStore.phase === "gameOver"}
  <GameOverScreen {...getPlayerGameOverProps()} />
{/if}
