<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";
  import MainMenu from "$lib/components/MainMenu.svelte";
  import SetupScreen from "$lib/components/SetupScreen.svelte";
  import GameBoard from "$lib/components/GameBoard.svelte";
  import GameOverScreen from "$lib/components/GameOverScreen.svelte";
</script>

{#if gameStore.phase === "menu"}
  <MainMenu />
{:else if gameStore.phase === "setup"}
  <SetupScreen />
{:else if gameStore.phase === "playing"}
  <GameBoard />
{:else if gameStore.phase === "gameOver"}
  <GameOverScreen />
{/if}
