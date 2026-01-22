<script lang="ts">
  import type { SupportDto } from "$lib/api/types";

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

  function getFactionColor(faction: string): string {
    switch (faction) {
      case "argentum": return "border-argentum-gold";
      case "symbiote": return "border-symbiote-glow";
      case "obsidion": return "border-obsidion-essence";
      default: return "border-neutral-copper";
    }
  }
</script>

<button
  class="w-16 h-20 rounded border-2 transition-all duration-150 flex flex-col items-center justify-center
         {support ? getFactionColor(support.faction) : 'border-gray-600 border-dashed'}
         {isHighlighted ? 'glow-blue scale-105' : ''}
         {support ? 'bg-ui-panel' : 'bg-ui-bg/50'}
         hover:border-opacity-100"
  onclick={onClick}
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
