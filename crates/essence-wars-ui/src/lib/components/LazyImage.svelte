<script lang="ts">
  import { assetLoader } from "$lib/assets";

  interface Props {
    src: string;
    alt: string;
    class?: string;
    placeholder?: string;
    /** If true, load immediately without intersection observer */
    eager?: boolean;
  }

  let {
    src,
    alt,
    class: className = "",
    placeholder = "",
    eager = false,
  }: Props = $props();

  let loaded = $state(false);
  let error = $state(false);
  let containerElement: HTMLElement | undefined = $state();

  // Track if image is in viewport
  let inViewport = $state(false);

  // Initialize inViewport based on eager prop
  $effect(() => {
    if (eager) {
      inViewport = true;
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

  // Load image when in viewport
  $effect(() => {
    if (!inViewport) return;

    assetLoader
      .loadImage(src)
      .then(() => {
        loaded = true;
      })
      .catch(() => {
        error = true;
      });
  });
</script>

<div bind:this={containerElement} class="lazy-image-container {className}">
  {#if loaded}
    <img
      {src}
      {alt}
      class="lazy-image loaded"
    />
  {:else if error}
    <div class="lazy-image-error">
      <span class="text-ui-muted text-xs">Failed to load</span>
    </div>
  {:else if placeholder}
    <img
      src={placeholder}
      {alt}
      class="lazy-image loading"
    />
  {:else}
    <div class="lazy-image-placeholder">
      <div class="shimmer"></div>
    </div>
  {/if}
</div>

<style>
  .lazy-image-container {
    position: relative;
    overflow: hidden;
  }

  .lazy-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: opacity 0.3s ease-in-out;
  }

  .lazy-image.loading {
    opacity: 0.3;
    filter: blur(4px);
  }

  .lazy-image.loaded {
    opacity: 1;
    filter: none;
  }

  .lazy-image-error {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    background: var(--ui-border, #333);
  }

  .lazy-image-placeholder {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 60px;
    background: linear-gradient(135deg, #1a1a2e 0%, #2a2a4e 100%);
    overflow: hidden;
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
