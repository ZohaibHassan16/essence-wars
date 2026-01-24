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

<div class="h-12 bg-ui-panel flex items-center justify-between px-4 border-gray-700"
     class:border-t={isPlayer}
     class:border-b={!isPlayer}>
  <div class="flex items-center gap-3">
    <!-- Active indicator + Name -->
    <div class="flex items-center gap-2">
      <div class="w-2.5 h-2.5 rounded-full transition-colors duration-300
                  {isActive ? (isPlayer ? 'bg-health' : 'bg-ui-action') + ' animate-pulse' : 'bg-gray-600'}">
      </div>
      <span class="text-ui-text font-semibold text-base">{name}</span>
    </div>

    <!-- Stats -->
    <div class="flex items-center gap-2">
      <!-- Life -->
      <div class="flex items-center gap-1 px-2 py-0.5 rounded bg-{isPlayer ? 'health' : 'damage'}/20">
        <span class="font-bold text-base {isPlayer ? 'text-health' : 'text-damage'}">{life}</span>
        <span class="{isPlayer ? 'text-health' : 'text-damage'}/60 text-sm">HP</span>
      </div>

      <!-- Essence -->
      <div class="flex items-center gap-1 px-2 py-0.5 rounded bg-mana/20">
        <span class="text-mana font-bold text-base">{essence}</span>
        <span class="text-mana/60 text-sm">/ {maxEssence}</span>
      </div>

      <!-- Action Points -->
      <div class="flex items-center gap-1 px-2 py-0.5 rounded bg-gold/20">
        <span class="text-gold font-bold text-base">{actionPoints}</span>
        <span class="text-gold/60 text-sm">AP</span>
      </div>
    </div>

    <!-- Deck count -->
    <div class="text-base text-ui-text-dim">
      Deck: {deckCount}
    </div>

    <!-- Hand count (for opponent) -->
    {#if handCount !== undefined}
      <div class="text-base text-ui-text-dim">
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
