<script lang="ts">
  import type { SupportDto, CardDto } from "$lib/api/types";
  import CardPreview from "./CardPreview.svelte";

  let {
    support = null,
    slot,
    isHighlighted = false,
    onClick,
  }: {
    support: SupportDto | null;
    slot: number;
    isHighlighted?: boolean;
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
</script>

<div class="relative">
  <button
    class="support-slot rounded-lg border-2 transition-all duration-150 flex flex-col items-center justify-between p-2
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
  >
    {#if support}
      <!-- Support art background -->
      {#if support.artPath}
        <div class="absolute inset-0 overflow-hidden rounded-md">
          <img
            src="/{support.artPath}"
            alt=""
            class="w-full h-full object-cover object-top opacity-40"
            onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
          />
          <div class="absolute inset-0 bg-gradient-to-b from-transparent via-black/30 to-black/70"></div>
        </div>
      {/if}

      <!-- Support name -->
      <div class="relative z-10 text-sm font-semibold truncate w-full px-1 text-center mt-1">
        {support.name}
      </div>

      <!-- Spacer -->
      <div class="flex-1"></div>

      <!-- Durability badge -->
      <div class="relative z-10 flex items-center justify-center mb-1">
        <div class="w-7 h-7 rounded-full bg-mana/30 border border-mana/50 flex items-center justify-center">
          <span class="text-mana font-bold text-base">{support.durability}</span>
        </div>
      </div>
    {:else}
      <div class="flex-1 flex flex-col items-center justify-center">
        <div class="w-7 h-7 rounded border border-gray-600 flex items-center justify-center mb-1">
          <span class="text-ui-text-dim text-sm">S{slot + 1}</span>
        </div>
        <span class="text-gray-500 text-xs">Support</span>
      </div>
    {/if}
  </button>

  <!-- Card preview on hover -->
  {#if showPreview && cardForPreview}
    <CardPreview card={cardForPreview} position="right" />
  {/if}
</div>
