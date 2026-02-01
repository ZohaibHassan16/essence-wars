<script lang="ts">
  import type { SupportDto, CardDto } from "$lib/api/types";
  import CardPreview from "./CardPreview.svelte";

  let {
    support = null,
    slot,
    isHighlighted = false,
    isPlayerSide = true,
    onClick,
  }: {
    support: SupportDto | null;
    slot: number;
    isHighlighted?: boolean;
    isPlayerSide?: boolean;
    onClick?: () => void;
  } = $props();

  let isHovered = $state(false);

  function getFactionColor(faction: string): string {
    switch (faction) {
      case "argentum": return "border-argentum-gold";
      case "symbiote": return "border-symbiote-glow";
      case "obsidion": return "border-obsidion-essence";
      default: return "border-neutral-copper";
    }
  }

  // Create a CardDto from SupportDto for preview
  const cardForPreview = $derived<CardDto | null>(support ? {
    cardId: support.cardId,
    name: support.name,
    cost: 0, // We don't have cost info in SupportDto
    cardType: "support",
    faction: support.faction,
    attack: undefined,
    health: undefined,
    keywords: [], // SupportDto doesn't have keywords
    durability: support.durability,
    artPath: support.artPath,
  } : null);

  const showPreview = $derived(isHovered && support !== null);

  // Track whether we've fallen back to generic art
  let usesFallbackArt = $state(false);

  // Reset fallback state when support changes
  $effect(() => {
    if (support) {
      usesFallbackArt = false;
    }
  });

  // Generate fallback art path based on faction
  const fallbackArtPath = $derived(
    support ? `tokens/generic/${support.faction || 'neutral'}.webp` : null
  );

  // Current art path to use (primary or fallback)
  const currentArtPath = $derived(
    usesFallbackArt ? fallbackArtPath : support?.artPath
  );

  // Handle image load error - try fallback, then hide
  function handleImageError(e: Event) {
    const img = e.currentTarget as HTMLImageElement;
    if (!usesFallbackArt && fallbackArtPath) {
      // First failure - try the fallback
      usesFallbackArt = true;
    } else {
      // Fallback also failed - hide the image
      img.style.display = 'none';
    }
  }
</script>

<div class="relative">
  <button
    class="support-slot rounded-md border-2 transition-all duration-150 flex flex-col items-center justify-between p-1.5
           relative overflow-hidden
           {support ? getFactionColor(support.faction) : 'border-gray-600 border-dashed'}
           {isHighlighted ? 'glow-blue scale-105' : ''}
           {support ? 'bg-ui-panel' : 'bg-ui-bg/50'}
           hover:border-opacity-100"
    style="width: var(--card-support-width); height: var(--card-support-height);"
    onclick={onClick}
    onmouseenter={() => isHovered = true}
    onmouseleave={() => isHovered = false}
    disabled={!onClick}
    data-tutorial-id="{isPlayerSide ? 'player' : 'opponent'}-support-{slot}"
  >
    {#if support}
      <!-- Support art background (with fallback support) -->
      {#if currentArtPath}
        <div class="absolute inset-0 overflow-hidden rounded-md">
          <img
            src="/{currentArtPath}"
            alt=""
            class="w-full h-full object-cover object-top opacity-40"
            onerror={handleImageError}
          />
          <div class="absolute inset-0 bg-gradient-to-b from-transparent via-black/30 to-black/70"></div>
        </div>
      {/if}

      <!-- Support name -->
      <div class="relative z-10 text-xs font-semibold truncate w-full px-0.5 text-center mt-0.5 drop-shadow-md">
        {support.name}
      </div>

      <!-- Spacer -->
      <div class="flex-1"></div>

      <!-- Durability badge -->
      <div class="relative z-10 flex items-center justify-center mb-0.5">
        <div class="w-6 h-6 rounded-full bg-mana/30 border border-mana/50 flex items-center justify-center">
          <span class="text-mana font-bold text-sm">{support.durability}</span>
        </div>
      </div>
    {:else}
      <div class="flex-1 flex flex-col items-center justify-center">
        <div class="w-6 h-6 rounded border border-gray-600 flex items-center justify-center mb-0.5">
          <span class="text-ui-text-dim text-xs">S{slot + 1}</span>
        </div>
        <span class="text-gray-500 text-xs">Support</span>
      </div>
    {/if}
  </button>

  <!-- Card preview on hover - positioned above since supports are at the bottom of the screen -->
  {#if showPreview && cardForPreview}
    <CardPreview card={cardForPreview} support={support} position="top" />
  {/if}
</div>
