<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { initBackends, isInitialized } from "$lib/api/backends";

  let { children } = $props();
  let ready = $state(false);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      await initBackends();
      ready = true;
    } catch (e) {
      console.error("Failed to initialize backends:", e);
      error = e instanceof Error ? e.message : String(e);
    }
  });
</script>

<div class="h-screen w-screen overflow-hidden bg-ui-bg text-ui-text">
  {#if error}
    <div class="flex h-full items-center justify-center">
      <div class="text-center">
        <h1 class="text-2xl font-bold text-red-500">Initialization Error</h1>
        <p class="mt-2 text-ui-muted">{error}</p>
      </div>
    </div>
  {:else if ready}
    {@render children()}
  {:else}
    <div class="flex h-full items-center justify-center">
      <div class="text-center">
        <div class="inline-block h-8 w-8 animate-spin rounded-full border-4 border-faction-argentum-accent border-t-transparent"></div>
        <p class="mt-4 text-ui-muted">Initializing...</p>
      </div>
    </div>
  {/if}
</div>
