<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import MainMenu from "$lib/components/MainMenu.svelte";
  import SetupScreen from "$lib/components/SetupScreen.svelte";
  import GameBoard from "$lib/components/GameBoard.svelte";
  import GameOverScreen from "$lib/components/GameOverScreen.svelte";
  import SpectatorSetup from "$lib/components/SpectatorSetup.svelte";
  import ComputingMatch from "$lib/components/ComputingMatch.svelte";
  import SpectatorPlayback from "$lib/components/SpectatorPlayback.svelte";
</script>

<!-- Spectator mode takes precedence when active -->
{#if spectatorStore.phase === "setup" && spectatorStore.decks.length > 0}
  <SpectatorSetup />
{:else if spectatorStore.phase === "computing"}
  <ComputingMatch />
{:else if spectatorStore.phase === "watching" || spectatorStore.phase === "finished"}
  <SpectatorPlayback />
{:else if gameStore.phase === "menu"}
  <MainMenu />
{:else if gameStore.phase === "setup"}
  <SetupScreen />
{:else if gameStore.phase === "playing"}
  <GameBoard />
{:else if gameStore.phase === "gameOver"}
  <GameOverScreen />
{/if}
