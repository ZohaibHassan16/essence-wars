<script lang="ts">
  import type { CreatureDto, CardDto } from "$lib/api/types";
  import CardPreview from "./CardPreview.svelte";
  import { animationRegistry } from "$lib/animations/actions";
  import { onMount, onDestroy } from "svelte";

  let {
    creature = null,
    slot,
    isPlayerSide,
    isHighlighted = false,
    isSelected = false,
    isValidTarget = false,
    onClick,
  }: {
    creature: CreatureDto | null;
    slot: number;
    isPlayerSide: boolean;
    isHighlighted?: boolean;
    isSelected?: boolean;
    isValidTarget?: boolean;
    onClick?: () => void;
  } = $props();

  let isHovered = $state(false);
  let slotElement: HTMLButtonElement;

  // Animation ID for this slot
  const animationId = $derived(`creature-${isPlayerSide ? "player" : "opponent"}-${slot}`);

  // Register/unregister with animation system
  $effect(() => {
    if (slotElement) {
      animationRegistry.set(animationId, slotElement);
      return () => {
        animationRegistry.delete(animationId);
      };
    }
  });

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
    bind:this={slotElement}
    class="w-28 h-36 rounded-xl border-2 transition-all duration-150 flex flex-col items-center justify-between p-2
           no-select relative overflow-hidden
           {creature ? getFactionBorder(creature.faction) + ' ' + getFactionBg(creature.faction) : 'border-gray-600 border-dashed bg-ui-bg/30'}
           {isHighlighted ? 'ring-2 ring-health animate-pulse' : ''}
           {isSelected ? 'ring-2 ring-ui-action scale-105' : ''}
           {isValidTarget ? 'ring-2 ring-damage' : ''}
           {creature && isPlayerSide ? 'hover:scale-102 cursor-pointer' : creature ? 'cursor-pointer' : 'cursor-default'}
           disabled:cursor-not-allowed"
    onclick={onClick}
    onmouseenter={() => isHovered = true}
    onmouseleave={() => isHovered = false}
    disabled={!onClick}
  >
    {#if creature}
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
        <div class="absolute top-1 right-1 w-2 h-2 rounded-full bg-health animate-pulse"></div>
      {/if}

      <!-- Exhausted overlay -->
      {#if creature.isExhausted}
        <div class="absolute inset-0 bg-black/30 rounded-xl flex items-center justify-center">
          <span class="text-[10px] text-yellow-400 font-semibold rotate-[-15deg]">EXHAUSTED</span>
        </div>
      {/if}

      <!-- Name -->
      <div class="text-xs font-semibold truncate w-full text-center text-ui-text leading-tight">
        {creature.name}
      </div>

      <!-- Keywords -->
      {#if creature.keywords.length > 0}
        <div class="flex flex-wrap justify-center gap-0.5 my-1">
          {#each creature.keywords.slice(0, 3) as keyword}
            <span class="text-[8px] px-1 py-0.5 rounded bg-gray-900/60 text-ui-text-dim border border-gray-700">
              {keyword}
            </span>
          {/each}
          {#if creature.keywords.length > 3}
            <span class="text-[8px] text-ui-text-dim">+{creature.keywords.length - 3}</span>
          {/if}
        </div>
      {:else}
        <div class="flex-1"></div>
      {/if}

      <!-- Stats -->
      <div class="flex items-center gap-3 mt-auto">
        <div class="flex flex-col items-center">
          <span class="text-lg font-bold {isBuffed ? 'text-green-400' : 'text-damage'}">
            {creature.attack}
          </span>
          <span class="text-[8px] text-ui-text-dim">ATK</span>
        </div>
        <div class="w-px h-6 bg-gray-600"></div>
        <div class="flex flex-col items-center">
          <span class="text-lg font-bold {isDamaged ? 'text-yellow-400' : 'text-health'}">
            {creature.health}
          </span>
          <span class="text-[8px] text-ui-text-dim">HP</span>
        </div>
      </div>
    {:else}
      <!-- Empty slot -->
      <div class="flex-1 flex flex-col items-center justify-center">
        <div class="w-8 h-8 rounded-full border border-gray-600 flex items-center justify-center mb-2">
          <span class="text-gray-500 text-sm">{slot + 1}</span>
        </div>
        <span class="text-gray-500 text-[10px]">Empty</span>
      </div>
    {/if}
  </button>

  <!-- Card preview on hover -->
  {#if showPreview && cardForPreview}
    <CardPreview card={cardForPreview} creature={creature} position={isPlayerSide ? "top" : "bottom"} />
  {/if}
</div>
