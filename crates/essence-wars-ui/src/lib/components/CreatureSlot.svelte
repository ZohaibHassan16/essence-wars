<script lang="ts">
  import type { CreatureDto } from "$lib/api/types";

  let {
    creature = null,
    slot,
    isPlayerSide,
    isHighlighted = false,
    isSelected = false,
    onClick,
  }: {
    creature: CreatureDto | null;
    slot: number;
    isPlayerSide: boolean;
    isHighlighted?: boolean;
    isSelected?: boolean;
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
  class="w-24 h-32 rounded-lg border-2 transition-all duration-150 flex flex-col items-center justify-center
         {creature ? getFactionColor(creature.faction) : 'border-gray-600 border-dashed'}
         {isHighlighted ? 'glow-green scale-105' : ''}
         {isSelected ? 'ring-2 ring-ui-action' : ''}
         {creature ? 'bg-ui-panel' : 'bg-ui-bg/50'}
         hover:border-opacity-100 active:scale-95"
  onclick={onClick}
  disabled={!onClick}
>
  {#if creature}
    <div class="text-xs font-semibold truncate w-full px-1 text-center">
      {creature.name}
    </div>
    <div class="flex gap-2 mt-2">
      <span class="text-damage font-bold">{creature.attack}</span>
      <span class="text-ui-text-dim">/</span>
      <span class="text-health font-bold">{creature.health}</span>
    </div>
    {#if creature.keywords.length > 0}
      <div class="text-[10px] text-ui-text-dim mt-1 truncate w-full px-1 text-center">
        {creature.keywords.slice(0, 2).join(", ")}
      </div>
    {/if}
    {#if creature.isExhausted}
      <div class="text-[10px] text-yellow-500 mt-1">Exhausted</div>
    {:else if creature.canAttack && isPlayerSide}
      <div class="text-[10px] text-health mt-1">Ready</div>
    {/if}
  {:else}
    <div class="text-ui-text-dim text-xs">Slot {slot + 1}</div>
  {/if}
</button>
