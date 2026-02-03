<script lang="ts">
  import type { CreatureDto, CardDto } from "$lib/api/types";
  import CardPreview from "./CardPreview.svelte";
  import { animatable } from "$lib/animations/actions";
  import { playSound } from "$lib/audio";
  import { gameSettings } from "$lib/stores/gameSettings.svelte";

  let {
    creature = null,
    slot,
    isPlayerSide,
    isHighlighted = false,
    isSelected = false,
    isValidTarget = false,
    onClick,
    showKeyHint = false,
  }: {
    creature: CreatureDto | null;
    slot: number;
    isPlayerSide: boolean;
    isHighlighted?: boolean;
    isSelected?: boolean;
    isValidTarget?: boolean;
    onClick?: (event?: MouseEvent) => void;
    showKeyHint?: boolean;
  } = $props();

  let isHovered = $state(false);

  // Animation ID for this slot
  const animationId = $derived(`creature-${isPlayerSide ? "player" : "opponent"}-${slot}`);

  function getFactionBorder(faction: string): string {
    switch (faction) {
      case "argentum": return "border-argentum-gold";
      case "symbiote": return "border-symbiote-glow";
      case "obsidion": return "border-obsidion-essence";
      default: return "border-neutral-copper";
    }
  }

  function getFactionBg(faction: string): string {
    switch (faction) {
      case "argentum": return "bg-gradient-to-br from-gray-800/90 via-argentum-brass/10 to-gray-900/90";
      case "symbiote": return "bg-gradient-to-br from-symbiote-primary/20 via-gray-800/90 to-symbiote-purple/20";
      case "obsidion": return "bg-gradient-to-br from-obsidion-primary/20 via-gray-800/90 to-obsidion-black/40";
      default: return "bg-gradient-to-br from-neutral-tan/20 via-gray-800/90 to-neutral-primary/20";
    }
  }

  // Create a CardDto from CreatureDto for preview
  const cardForPreview = $derived<CardDto | null>(creature ? {
    cardId: creature.cardId,
    name: creature.name,
    cost: 0,
    cardType: "creature",
    faction: creature.faction,
    attack: creature.baseAttack,
    health: creature.maxHealth,
    keywords: creature.keywords,
    durability: undefined,
    artPath: creature.artPath,
  } : null);

  const showPreview = $derived(isHovered && creature !== null);
  const healthPercent = $derived(creature ? Math.max(0, (creature.health / creature.maxHealth) * 100) : 0);
  const isDamaged = $derived(creature ? creature.health < creature.maxHealth : false);
  const isBuffed = $derived(creature ? creature.attack > creature.baseAttack : false);
  const hasAbilities = $derived(creature?.abilities && creature.abilities.length > 0);

  // Track whether we've fallen back to generic art
  let usesFallbackArt = $state(false);

  // Reset fallback state when creature changes
  $effect(() => {
    if (creature) {
      usesFallbackArt = false;
    }
  });

  // Generate fallback art path based on faction
  const fallbackArtPath = $derived(
    creature ? `tokens/generic/${creature.faction || 'neutral'}.webp` : null
  );

  // Current art path to use (primary or fallback)
  const currentArtPath = $derived(
    usesFallbackArt ? fallbackArtPath : creature?.artPath
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
    use:animatable
    data-animate-id={animationId}
    data-tutorial-id="{isPlayerSide ? 'player' : 'opponent'}-slot-{slot}"
    class="creature-slot rounded-lg border-2 transition-all duration-150 flex flex-col items-center justify-between p-2
           no-select relative overflow-hidden
           {creature ? getFactionBorder(creature.faction) + ' ' + getFactionBg(creature.faction) : 'border-gray-600 border-dashed bg-ui-bg/30'}
           {isHighlighted ? 'ring-2 ring-health animate-pulse' : ''}
           {isSelected ? 'ring-2 ring-ui-action scale-105' : ''}
           {isValidTarget ? 'ring-2 ring-damage' : ''}
           {creature && isPlayerSide ? 'hover:scale-102 cursor-pointer' : creature ? 'cursor-pointer' : 'cursor-default'}
           disabled:cursor-not-allowed"
    style="width: var(--card-creature-width); height: var(--card-creature-height);"
    onclick={(e: MouseEvent) => {
      if (onClick) {
        if (creature) playSound('cardSelect');
        onClick(e);
      }
    }}
    onmouseenter={() => {
      isHovered = true;
      if (creature && onClick) playSound('cardHover');
    }}
    onmouseleave={() => isHovered = false}
    disabled={!onClick}
  >
    {#if creature}
      <!-- Creature art background (with fallback support) -->
      {#if currentArtPath}
        <div class="absolute inset-0 overflow-hidden rounded-lg">
          <img
            src="/{currentArtPath}"
            alt=""
            class="w-full h-full object-cover object-top opacity-70"
            loading="lazy"
            decoding="async"
            onerror={handleImageError}
          />
          <div class="absolute inset-0 bg-gradient-to-b from-black/10 via-transparent to-black/40"></div>
        </div>
      {/if}

      <!-- Health bar background -->
      <div class="absolute bottom-0 left-0 right-0 h-1 bg-gray-700">
        <div
          class="h-full transition-all duration-300
                 {healthPercent > 50 ? 'bg-health' : healthPercent > 25 ? 'bg-yellow-500' : 'bg-damage'}"
          style="width: {healthPercent}%"
        ></div>
      </div>

      <!-- Can attack indicator -->
      {#if creature.canAttack && isPlayerSide}
        <div class="absolute top-2 right-2 w-3 h-3 rounded-full bg-health animate-pulse"></div>
      {/if}

      <!-- Ability indicator (shows for player creatures with abilities) -->
      {#if hasAbilities && isPlayerSide}
        <div class="absolute top-2 right-7 w-4 h-4 rounded-full bg-purple-500/90 animate-pulse
                    flex items-center justify-center z-10 border border-purple-300/50"
             title="Has activated ability">
          <span class="text-white text-xs font-bold">✦</span>
        </div>
      {/if}

      <!-- Keyboard hint badge (only for player creatures that can attack) -->
      {#if showKeyHint && gameSettings.showKeyboardHints && isPlayerSide && creature.canAttack}
        <div class="absolute top-2 left-2 w-5 h-5 rounded bg-gray-800/90 flex items-center justify-center
                    text-ui-text-dim text-xs font-mono border border-gray-600 z-10">
          {slot + 1}
        </div>
      {/if}

      <!-- Exhausted overlay -->
      {#if creature.isExhausted}
        <div class="absolute inset-0 bg-black/30 rounded-xl flex items-center justify-center">
          <span class="text-xs text-yellow-400 font-semibold rotate-[-15deg]">EXHAUSTED</span>
        </div>
      {/if}

      <!-- Name -->
      <div class="text-sm font-semibold truncate w-full text-center text-ui-text leading-tight relative z-10"
           style="text-shadow: 0 1px 3px rgba(0,0,0,0.9), 0 0 8px rgba(0,0,0,0.7);">
        {creature.name}
      </div>

      <!-- Spacer to push stats to bottom -->
      <div class="flex-1"></div>

      <!-- Stats with Keywords -->
      <div class="flex items-center justify-center gap-1.5 mt-auto relative z-10">
        <!-- Keywords (left side) -->
        {#if creature.keywords.length > 0}
          <div class="flex flex-col gap-0.5">
            {#each creature.keywords.slice(0, 2) as keyword (keyword)}
              <span class="text-xs px-1 py-0.5 rounded bg-gray-900/80 text-ui-text-dim border border-gray-700 leading-tight"
                    style="text-shadow: 0 1px 2px rgba(0,0,0,0.8);">
                {keyword}
              </span>
            {/each}
            {#if creature.keywords.length > 2}
              <span class="text-xs text-ui-text-dim text-center" style="text-shadow: 0 1px 2px rgba(0,0,0,0.8);">+{creature.keywords.length - 2}</span>
            {/if}
          </div>
        {/if}

        <!-- ATK stat -->
        <div class="flex flex-col items-center">
          <span class="text-xl font-bold {isBuffed ? 'text-green-400' : 'text-damage'}"
                style="text-shadow: 0 1px 3px rgba(0,0,0,0.9), 0 0 6px rgba(0,0,0,0.6);">
            {creature.attack}
          </span>
          <span class="text-xs text-ui-text-dim font-medium" style="text-shadow: 0 1px 2px rgba(0,0,0,0.8);">ATK</span>
        </div>
        <div class="w-px h-5 bg-gray-600/80"></div>
        <!-- HP stat -->
        <div class="flex flex-col items-center">
          <span class="text-xl font-bold {isDamaged ? 'text-yellow-400' : 'text-health'}"
                style="text-shadow: 0 1px 3px rgba(0,0,0,0.9), 0 0 6px rgba(0,0,0,0.6);">
            {creature.health}
          </span>
          <span class="text-xs text-ui-text-dim font-medium" style="text-shadow: 0 1px 2px rgba(0,0,0,0.8);">HP</span>
        </div>
      </div>
    {:else}
      <!-- Empty slot -->
      <div class="flex-1 flex flex-col items-center justify-center">
        <div class="w-8 h-8 rounded-full border border-gray-600 flex items-center justify-center mb-1">
          <span class="text-gray-500 text-sm">{slot + 1}</span>
        </div>
        <span class="text-gray-500 text-xs">Empty</span>
      </div>
    {/if}
  </button>

  <!-- Card preview on hover -->
  {#if showPreview && cardForPreview}
    <CardPreview card={cardForPreview} creature={creature} position={isPlayerSide ? "top" : "bottom"} />
  {/if}
</div>
