<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { initBackends, getPlatform } from "$lib/api/backends";
  import {
    preloadEssentialAssets,
    preloadDeferredAssets,
    getLoadingProgress,
  } from "$lib/assets";

  let { children } = $props();
  let ready = $state(false);
  let error = $state<string | null>(null);
  let loadingStage = $state<"backend" | "assets" | "ready">("backend");
  let loadingProgress = $state(0);

  onMount(async () => {
    try {
      // Stage 1: Initialize Tauri backends
      loadingStage = "backend";
      await initBackends();

      // Stage 2: Preload essential assets
      loadingStage = "assets";
      await preloadEssentialAssets();

      // Ready!
      loadingStage = "ready";
      ready = true;

      // Stage 3: Load deferred assets in background (non-blocking)
      preloadDeferredAssets();
    } catch (e) {
      console.error("Failed to initialize:", e);
      error = e instanceof Error ? e.message : String(e);
    }
  });

  // Update loading progress periodically during asset loading
  $effect(() => {
    if (loadingStage === "assets") {
      const interval = setInterval(() => {
        loadingProgress = getLoadingProgress().percentage;
      }, 100);
      return () => clearInterval(interval);
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
        <p class="mt-4 text-ui-muted">
          {#if loadingStage === "backend"}
            Initializing game engine...
          {:else if loadingStage === "assets"}
            Loading assets... {loadingProgress}%
          {:else}
            Starting...
          {/if}
        </p>
        {#if loadingStage === "assets"}
          <div class="mt-2 h-1 w-48 mx-auto bg-ui-border rounded-full overflow-hidden">
            <div
              class="h-full bg-faction-argentum-accent transition-all duration-200"
              style="width: {loadingProgress}%"
            ></div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
