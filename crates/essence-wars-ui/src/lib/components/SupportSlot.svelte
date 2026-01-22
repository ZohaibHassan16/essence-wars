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
    class="w-16 h-20 rounded border-2 transition-all duration-150 flex flex-col items-center justify-center
           {support ? getFactionColor(support.faction) : 'border-gray-600 border-dashed'}
           {isHighlighted ? 'glow-blue scale-105' : ''}
           {support ? 'bg-ui-panel' : 'bg-ui-bg/50'}
           hover:border-opacity-100"
    onclick={onClick}
    onmouseenter={() => isHovered = true}
    onmouseleave={() => isHovered = false}
    disabled={!onClick}
  >
    {#if support}
      <div class="text-[10px] font-semibold truncate w-full px-1 text-center">
        {support.name}
      </div>
      <div class="text-xs text-mana font-bold mt-1">
        {support.durability}
      </div>
    {:else}
      <div class="text-ui-text-dim text-[10px]">S{slot + 1}</div>
    {/if}
  </button>

  <!-- Card preview on hover -->
  {#if showPreview && cardForPreview}
    <CardPreview card={cardForPreview} position="right" />
  {/if}
</div>
