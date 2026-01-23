<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    name,
    life,
    essence,
    maxEssence,
    actionPoints,
    deckCount,
    handCount,
    isActive = false,
    isPlayer = false,
    actions,
  }: {
    name: string;
    life: number;
    essence: number;
    maxEssence: number;
    actionPoints: number;
    deckCount: number;
    handCount?: number;
    isActive?: boolean;
    isPlayer?: boolean;
    actions?: Snippet;
  } = $props();
</script>

<div class="h-14 bg-ui-panel flex items-center justify-between px-6 border-gray-700"
     class:border-t={isPlayer}
     class:border-b={!isPlayer}>
  <div class="flex items-center gap-4">
    <!-- Active indicator + Name -->
    <div class="flex items-center gap-2">
      <div class="w-3 h-3 rounded-full transition-colors duration-300
                  {isActive ? (isPlayer ? 'bg-health' : 'bg-ui-action') + ' animate-pulse' : 'bg-gray-600'}">
      </div>
      <span class="text-ui-text font-semibold">{name}</span>
    </div>

    <!-- Stats -->
    <div class="flex items-center gap-3">
      <!-- Life -->
      <div class="flex items-center gap-1 px-2 py-1 rounded bg-{isPlayer ? 'health' : 'damage'}/20">
        <span class="font-bold {isPlayer ? 'text-health' : 'text-damage'}">{life}</span>
        <span class="{isPlayer ? 'text-health' : 'text-damage'}/60 text-xs">HP</span>
      </div>

      <!-- Essence -->
      <div class="flex items-center gap-1 px-2 py-1 rounded bg-mana/20">
        <span class="text-mana font-bold">{essence}</span>
        <span class="text-mana/60 text-xs">/ {maxEssence}</span>
      </div>

      <!-- Action Points -->
      <div class="flex items-center gap-1 px-2 py-1 rounded bg-gold/20">
        <span class="text-gold font-bold">{actionPoints}</span>
        <span class="text-gold/60 text-xs">AP</span>
      </div>
    </div>

    <!-- Deck count -->
    <div class="text-sm text-ui-text-dim">
      Deck: {deckCount}
    </div>

    <!-- Hand count (for opponent) -->
    {#if handCount !== undefined}
      <div class="text-sm text-ui-text-dim">
        Hand: {handCount}
      </div>
    {/if}
  </div>

  <!-- Action buttons slot -->
  {#if actions}
    <div class="flex items-center gap-3">
      {@render actions()}
    </div>
  {/if}
</div>
