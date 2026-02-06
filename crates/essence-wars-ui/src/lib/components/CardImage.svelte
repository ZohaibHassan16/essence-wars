<script lang="ts">
  import { assetLoader } from "$lib/assets";

  interface Props {
    cardId: number;
    cardType?: "creature" | "spell" | "support";
    alt?: string;
    class?: string;
    /** If true, load immediately without intersection observer */
    eager?: boolean;
  }

  let {
    cardId,
    cardType = "creature",
    alt = "Card art",
    class: className = "",
    eager = false,
  }: Props = $props();

  let loaded = $state(false);
  let error = $state(false);
  let actualSrc = $state("");
  let containerElement: HTMLElement | undefined = $state();

  // Track if image is in viewport (starts as eager if prop is true)
  let inViewport = $state(false);

  // Initialize inViewport based on eager prop
  $effect(() => {
    if (eager) {
      inViewport = true;
    }
  });

  // Derive the card art path based on card type
  const cardPaths = $derived.by(() => {
    const basePaths = {
      creature: `/cards/creatures/${cardId}.webp`,
      spell: `/cards/spells/${cardId}.webp`,
      support: `/cards/supports/${cardId}.webp`,
    };

    // Return paths in order of priority
    if (cardType === "creature") {
      return [basePaths.creature, basePaths.spell, basePaths.support];
    } else if (cardType === "spell") {
      return [basePaths.spell, basePaths.creature, basePaths.support];
    } else {
      return [basePaths.support, basePaths.creature, basePaths.spell];
    }
  });

  $effect(() => {
    if (!containerElement || inViewport) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) {
          inViewport = true;
          observer.disconnect();
        }
      },
      {
        rootMargin: "100px",
        threshold: 0.1,
      }
    );

    observer.observe(containerElement);

    return () => observer.disconnect();
  });

  // Load image when in viewport, trying each path in order
  $effect(() => {
    if (!inViewport) return;

    async function tryLoadPaths() {
      for (const path of cardPaths) {
        try {
          await assetLoader.loadImage(path);
          actualSrc = path;
          loaded = true;
          return;
        } catch {
          // Try next path
          continue;
        }
      }
      // All paths failed
      error = true;
    }

    tryLoadPaths();
  });
</script>

<div bind:this={containerElement} class="card-image-container {className}">
  {#if loaded && actualSrc}
    <img
      src={actualSrc}
      {alt}
      class="card-image loaded"
    />
  {:else if error}
    <div class="card-image-placeholder error">
      <span class="text-ui-muted text-xs">?</span>
    </div>
  {:else}
    <div class="card-image-placeholder loading">
      <div class="shimmer"></div>
    </div>
  {/if}
</div>

<style>
  .card-image-container {
    position: relative;
    overflow: hidden;
    background: var(--ui-surface, #1a1a2e);
    border-radius: 4px;
  }

  .card-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: opacity 0.3s ease-in-out;
  }

  .card-image.loaded {
    opacity: 1;
  }

  .card-image-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    min-height: 60px;
    background: linear-gradient(135deg, #1a1a2e 0%, #2a2a4e 100%);
  }

  .card-image-placeholder.loading {
    position: relative;
    overflow: hidden;
  }

  .card-image-placeholder.error {
    background: var(--ui-border, #333);
  }

  .shimmer {
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(
      90deg,
      transparent 0%,
      rgba(255, 255, 255, 0.05) 50%,
      transparent 100%
    );
    animation: shimmer 1.5s infinite;
  }

  @keyframes shimmer {
    0% {
      left: -100%;
    }
    100% {
      left: 100%;
    }
  }
</style>
