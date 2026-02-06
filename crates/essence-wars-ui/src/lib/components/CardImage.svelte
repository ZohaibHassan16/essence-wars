<script lang="ts">
  import { assetLoader } from "$lib/assets";
  import { cardArtUrl } from "$lib/utils/paths";

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

  // All cards are in /cards/core_set/{id}.webp - use cardArtUrl for base path
  const cardPath = $derived(cardArtUrl(cardId));

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

  // Load image when in viewport
  $effect(() => {
    if (!inViewport) return;

    async function loadCard() {
      try {
        await assetLoader.loadImage(cardPath);
        actualSrc = cardPath;
        loaded = true;
      } catch {
        error = true;
      }
    }

    loadCard();
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
