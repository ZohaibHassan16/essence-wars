<script lang="ts">
  import type { CardDto } from "$lib/api/types";
  import CardPreview from "./CardPreview.svelte";

  let {
    card,
    index,
    isSelected = false,
    isPlayable = false,
    onClick,
  }: {
    card: CardDto;
    index: number;
    isSelected?: boolean;
    isPlayable?: boolean;
    onClick?: () => void;
  } = $props();

  let isHovered = $state(false);

  function getFactionBg(faction: string): string {
    switch (faction) {
      case "argentum": return "bg-gradient-to-b from-argentum-primary/20 via-argentum-brass/10 to-argentum-gold/20";
      case "symbiote": return "bg-gradient-to-b from-symbiote-primary/30 via-symbiote-purple/15 to-symbiote-glow/10";
      case "obsidion": return "bg-gradient-to-b from-obsidion-primary/30 via-obsidion-black/20 to-obsidion-essence/10";
      default: return "bg-gradient-to-b from-neutral-tan/20 via-neutral-primary/15 to-neutral-copper/20";
    }
  }

  function getFactionBorder(faction: string): string {
    switch (faction) {
      case "argentum": return "border-argentum-gold";
      case "symbiote": return "border-symbiote-glow";
      case "obsidion": return "border-obsidion-essence";
      default: return "border-neutral-copper";
    }
  }

  function getFactionGlow(faction: string): string {
    switch (faction) {
      case "argentum": return "shadow-argentum-gold/40";
      case "symbiote": return "shadow-symbiote-glow/40";
      case "obsidion": return "shadow-obsidion-essence/40";
      default: return "shadow-neutral-copper/40";
    }
  }

  const isHidden = $derived(card.cardId === 0);
  const showPreview = $derived(isHovered && !isHidden && !isSelected);
</script>

<div class="relative">
  <button
    class="w-20 h-28 rounded-lg border-2 transition-all duration-150 flex flex-col relative
           no-select overflow-hidden
           {isHidden ? 'bg-ui-panel border-gray-600' : getFactionBg(card.faction) + ' ' + getFactionBorder(card.faction)}
           {isSelected ? 'ring-2 ring-ui-action scale-110 -translate-y-4 z-20 shadow-lg ' + getFactionGlow(card.faction) : ''}
           {isPlayable && !isSelected ? 'hover:scale-105 hover:-translate-y-2 hover:shadow-md cursor-pointer' : ''}
           {!isPlayable && !isHidden ? 'opacity-50 grayscale-[30%]' : ''}
           disabled:cursor-not-allowed"
    onclick={onClick}
    onmouseenter={() => isHovered = true}
    onmouseleave={() => isHovered = false}
    disabled={!onClick || isHidden}
  >
    {#if isHidden}
      <!-- Hidden card (opponent's hand) -->
      <div class="w-full h-full flex items-center justify-center bg-gradient-to-br from-gray-700 to-gray-800">
        <div class="w-10 h-10 rounded-full border-2 border-gray-500 flex items-center justify-center">
          <span class="text-gray-400 text-lg font-bold">?</span>
        </div>
      </div>
    {:else}
      <!-- Cost badge -->
      <div class="absolute -top-1.5 -right-1.5 w-6 h-6 rounded-full bg-mana flex items-center justify-center
                  text-white text-xs font-bold shadow-md border border-blue-400 z-10">
        {card.cost}
      </div>

      <!-- Playable indicator glow -->
      {#if isPlayable && !isSelected}
        <div class="absolute inset-0 rounded-lg animate-pulse opacity-30 pointer-events-none
                    {getFactionBorder(card.faction).replace('border-', 'bg-')}"></div>
      {/if}

      <!-- Card content -->
      <div class="flex-1 flex flex-col p-1 pt-2 relative z-0">
        <!-- Name -->
        <div class="text-[10px] font-semibold truncate w-full text-center leading-tight text-ui-text px-0.5">
          {card.name}
        </div>

        <!-- Type -->
        <div class="text-[8px] text-ui-text-dim capitalize mt-0.5 text-center">
          {card.cardType}
        </div>

        <!-- Keywords preview (first 2) -->
        {#if card.keywords && card.keywords.length > 0}
          <div class="flex flex-wrap justify-center gap-0.5 mt-1">
            {#each card.keywords.slice(0, 2) as keyword}
              <span class="text-[7px] px-1 py-0.5 rounded bg-gray-800/60 text-ui-text-dim">
                {keyword}
              </span>
            {/each}
            {#if card.keywords.length > 2}
              <span class="text-[7px] text-ui-text-dim">+{card.keywords.length - 2}</span>
            {/if}
          </div>
        {/if}

        <!-- Stats -->
        {#if card.cardType === "creature" && card.attack !== undefined && card.health !== undefined}
          <div class="flex justify-center items-center gap-1 mt-auto mb-1">
            <span class="w-5 h-5 rounded bg-damage/20 flex items-center justify-center text-damage text-xs font-bold">
              {card.attack}
            </span>
            <span class="w-5 h-5 rounded bg-health/20 flex items-center justify-center text-health text-xs font-bold">
              {card.health}
            </span>
          </div>
        {:else if card.cardType === "support" && card.durability !== undefined}
          <div class="flex justify-center mt-auto mb-1">
            <span class="w-5 h-5 rounded bg-mana/20 flex items-center justify-center text-mana text-xs font-bold">
              {card.durability}
            </span>
          </div>
        {:else if card.cardType === "spell"}
          <div class="flex-1 flex items-center justify-center">
            <span class="text-[8px] text-ui-text-dim">Spell</span>
          </div>
        {:else}
          <div class="flex-1"></div>
        {/if}
      </div>
    {/if}
  </button>

  <!-- Card preview on hover -->
  {#if showPreview}
    <CardPreview {card} position="top" />
  {/if}
</div>
