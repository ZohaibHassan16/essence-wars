<script lang="ts">
  import type { Snippet } from "svelte";
  import { Flame } from "lucide-svelte";

  const ESSENCE_EXTRACTION_THRESHOLD = 50;

  let {
    name,
    life,
    essence,
    maxEssence,
    actionPoints,
    deckCount,
    handCount,
    essenceExtracted = 0,
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
    essenceExtracted?: number;
    isActive?: boolean;
    isPlayer?: boolean;
    actions?: Snippet;
  } = $props();

  // Calculate extraction progress percentage (capped at 100%)
  let extractionProgress = $derived(Math.min(100, (essenceExtracted / ESSENCE_EXTRACTION_THRESHOLD) * 100));

  // Color based on progress: green when close to winning
  let extractionColor = $derived(
    extractionProgress >= 80 ? 'text-health' :
    extractionProgress >= 50 ? 'text-gold' :
    'text-ui-text'
  );
</script>

<div class="bg-ui-panel flex items-center justify-between px-4 border-gray-700"
     style="height: var(--info-bar-height);"
     class:border-t={isPlayer}
     class:border-b={!isPlayer}>
  <div class="flex items-center gap-3">
    <!-- Active indicator + Name -->
    <div class="flex items-center gap-2">
      <div class="w-3 h-3 rounded-full transition-colors duration-300
                  {isActive ? (isPlayer ? 'bg-health' : 'bg-ui-action') + ' animate-pulse' : 'bg-gray-600'}">
      </div>
      <span class="text-ui-text font-semibold text-base">{name}</span>
    </div>

    <!-- Stats -->
    <div class="flex items-center gap-2">
      <!-- Life -->
      <div class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-{isPlayer ? 'health' : 'damage'}/20"
           data-tutorial-id="{isPlayer ? 'player' : 'opponent'}-life">
        <span class="font-bold text-lg {isPlayer ? 'text-health' : 'text-damage'}">{life}</span>
        <span class="{isPlayer ? 'text-health' : 'text-damage'}/60 text-sm font-medium">HP</span>
      </div>

      <!-- Essence -->
      <div class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-mana/20"
           data-tutorial-id="{isPlayer ? 'player' : 'opponent'}-essence">
        <span class="text-mana font-bold text-lg">{essence}</span>
        <span class="text-mana/60 text-sm font-medium">/ {maxEssence}</span>
      </div>

      <!-- Action Points -->
      <div class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-gold/20"
           data-tutorial-id="{isPlayer ? 'player' : 'opponent'}-ap">
        <span class="text-gold font-bold text-lg">{actionPoints}</span>
        <span class="text-gold/60 text-sm font-medium">AP</span>
      </div>

      <!-- Essence Extracted (VP progress toward 50) -->
      <div class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-orange-500/20 relative overflow-hidden"
           title="Essence Extracted: {essenceExtracted}/{ESSENCE_EXTRACTION_THRESHOLD} - Extract 50 to win!">
        <!-- Progress bar background -->
        <div class="absolute inset-0 bg-orange-500/30 transition-all duration-500"
             style="width: {extractionProgress}%"></div>
        <!-- Content -->
        <Flame size={16} class="text-orange-400 relative z-10" />
        <span class="{extractionColor} font-bold text-lg relative z-10">{essenceExtracted}</span>
        <span class="text-orange-400/60 text-sm font-medium relative z-10">/ {ESSENCE_EXTRACTION_THRESHOLD}</span>
      </div>
    </div>

    <!-- Deck count -->
    <div class="text-base text-ui-text-dim font-medium">
      Deck: {deckCount}
    </div>

    <!-- Hand count (for opponent) -->
    {#if handCount !== undefined}
      <div class="text-base text-ui-text-dim font-medium">
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
