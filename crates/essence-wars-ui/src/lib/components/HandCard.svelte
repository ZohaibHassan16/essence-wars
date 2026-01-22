<script lang="ts">
  import type { CardDto } from "$lib/api/types";

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

  function getFactionBg(faction: string): string {
    switch (faction) {
      case "argentum": return "bg-gradient-to-b from-argentum-primary/20 to-argentum-brass/20";
      case "symbiote": return "bg-gradient-to-b from-symbiote-primary/30 to-symbiote-purple/20";
      case "obsidion": return "bg-gradient-to-b from-obsidion-primary/30 to-obsidion-black/30";
      default: return "bg-gradient-to-b from-neutral-tan/20 to-neutral-primary/20";
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

  const isHidden = $derived(card.cardId === 0);
</script>

<button
  class="w-20 h-28 rounded-lg border-2 transition-all duration-150 flex flex-col relative
         card-hover no-select
         {isHidden ? 'bg-ui-panel border-gray-600' : getFactionBg(card.faction) + ' ' + getFactionBorder(card.faction)}
         {isSelected ? 'ring-2 ring-ui-action scale-110 -translate-y-2 z-10' : ''}
         {isPlayable && !isSelected ? 'opacity-100' : isPlayable ? 'opacity-100' : 'opacity-60'}
         disabled:cursor-not-allowed"
  onclick={onClick}
  disabled={!onClick || isHidden}
>
  {#if isHidden}
    <div class="w-full h-full flex items-center justify-center">
      <div class="text-ui-text-dim text-2xl">?</div>
    </div>
  {:else}
    <!-- Cost badge -->
    <div class="absolute -top-2 -right-2 w-6 h-6 rounded-full bg-mana flex items-center justify-center text-white text-xs font-bold shadow">
      {card.cost}
    </div>

    <!-- Card content -->
    <div class="flex-1 flex flex-col p-1 pt-2">
      <div class="text-[10px] font-semibold truncate w-full text-center leading-tight">
        {card.name}
      </div>

      <div class="text-[8px] text-ui-text-dim capitalize mt-0.5">
        {card.cardType}
      </div>

      {#if card.cardType === "creature" && card.attack !== undefined && card.health !== undefined}
        <div class="flex justify-center gap-1 mt-auto mb-1">
          <span class="text-damage text-sm font-bold">{card.attack}</span>
          <span class="text-ui-text-dim text-sm">/</span>
          <span class="text-health text-sm font-bold">{card.health}</span>
        </div>
      {:else if card.cardType === "support" && card.durability !== undefined}
        <div class="flex justify-center mt-auto mb-1">
          <span class="text-mana text-sm font-bold">{card.durability}</span>
        </div>
      {:else}
        <div class="flex-1"></div>
      {/if}
    </div>
  {/if}
</button>
