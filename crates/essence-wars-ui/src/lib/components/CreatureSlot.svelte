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
    onClick?: () => void;
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
</script>

<div class="relative">
  <button
    use:animatable
    data-animate-id={animationId}
    class="creature-slot rounded-lg border-2 transition-all duration-150 flex flex-col items-center justify-between p-2
           no-select relative overflow-hidden
           {creature ? getFactionBorder(creature.faction) + ' ' + getFactionBg(creature.faction) : 'border-gray-600 border-dashed bg-ui-bg/30'}
           {isHighlighted ? 'ring-2 ring-health animate-pulse' : ''}
           {isSelected ? 'ring-2 ring-ui-action scale-105' : ''}
           {isValidTarget ? 'ring-2 ring-damage' : ''}
           {creature && isPlayerSide ? 'hover:scale-102 cursor-pointer' : creature ? 'cursor-pointer' : 'cursor-default'}
           disabled:cursor-not-allowed"
    style="width: var(--card-creature-width); height: var(--card-creature-height);"
    onclick={() => {
      if (onClick) {
        if (creature) playSound('cardSelect');
        onClick();
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
      <!-- Creature art background -->
      {#if creature.artPath}
        <div class="absolute inset-0 overflow-hidden rounded-lg">
          <img
            src="/{creature.artPath}"
            alt=""
            class="w-full h-full object-cover object-top opacity-30"
            loading="lazy"
            decoding="async"
            onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
          />
          <div class="absolute inset-0 bg-gradient-to-b from-transparent via-black/30 to-black/60"></div>
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
      <div class="text-sm font-semibold truncate w-full text-center text-ui-text leading-tight drop-shadow-md">
        {creature.name}
      </div>

      <!-- Keywords -->
      {#if creature.keywords.length > 0}
        <div class="flex flex-wrap justify-center gap-0.5 my-0.5">
          {#each creature.keywords.slice(0, 2) as keyword}
            <span class="text-xs px-1 py-0.5 rounded bg-gray-900/70 text-ui-text-dim border border-gray-700">
              {keyword}
            </span>
          {/each}
          {#if creature.keywords.length > 2}
            <span class="text-xs text-ui-text-dim">+{creature.keywords.length - 2}</span>
          {/if}
        </div>
      {:else}
        <div class="flex-1"></div>
      {/if}

      <!-- Stats -->
      <div class="flex items-center gap-2 mt-auto">
        <div class="flex flex-col items-center">
          <span class="text-xl font-bold {isBuffed ? 'text-green-400' : 'text-damage'} drop-shadow-md">
            {creature.attack}
          </span>
          <span class="text-xs text-ui-text-dim font-medium">ATK</span>
        </div>
        <div class="w-px h-5 bg-gray-600"></div>
        <div class="flex flex-col items-center">
          <span class="text-xl font-bold {isDamaged ? 'text-yellow-400' : 'text-health'} drop-shadow-md">
            {creature.health}
          </span>
          <span class="text-xs text-ui-text-dim font-medium">HP</span>
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
