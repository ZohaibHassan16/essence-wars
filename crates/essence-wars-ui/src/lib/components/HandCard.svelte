<script lang="ts">
  import type { CardDto } from "$lib/api/types";
  import CardPreview from "./CardPreview.svelte";
  import KeywordIcon from "./KeywordIcon.svelte";
  import { playSound } from "$lib/audio";
  import { gameSettings } from "$lib/stores/gameSettings.svelte";

  let {
    card,
    index,
    isSelected = false,
    isPlayable = false,
    onClick,
    showKeyHint = false,
  }: {
    card: CardDto;
    index: number;
    isSelected?: boolean;
    isPlayable?: boolean;
    onClick?: () => void;
    showKeyHint?: boolean;
  } = $props();

  // Keyboard hint keys for positions 0-6
  const keyHints = ['Q', 'W', 'E', 'R', 'T', 'Y', 'U'];

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
    class="hand-card-btn rounded-lg border-2 transition-all duration-150 flex flex-col relative
           no-select overflow-hidden
           {isHidden ? 'bg-ui-panel border-gray-600' : getFactionBg(card.faction) + ' ' + getFactionBorder(card.faction)}
           {isSelected ? 'ring-2 ring-ui-action scale-110 -translate-y-4 z-20 shadow-lg ' + getFactionGlow(card.faction) : ''}
           {isPlayable && !isSelected ? 'cursor-pointer' : ''}
           {!isPlayable && !isHidden ? 'opacity-50 grayscale-[30%]' : ''}
           disabled:cursor-not-allowed"
    style="width: var(--card-hand-width); height: var(--card-hand-height);"
    onclick={() => {
      if (onClick && !isHidden) {
        playSound('cardSelect');
        onClick();
      }
    }}
    onmouseenter={() => {
      isHovered = true;
      if (!isHidden && isPlayable) playSound('cardHover');
    }}
    onmouseleave={() => isHovered = false}
    disabled={!onClick || isHidden}
  >
    {#if isHidden}
      <!-- Hidden card (opponent's hand) - CSS card back design -->
      <div class="w-full h-full flex items-center justify-center relative overflow-hidden
                  bg-gradient-to-br from-slate-800 via-slate-900 to-slate-950">
        <!-- Decorative border pattern -->
        <div class="absolute inset-2 border border-amber-900/30 rounded"></div>
        <div class="absolute inset-3 border border-amber-800/20 rounded"></div>

        <!-- Central emblem -->
        <div class="w-14 h-14 rounded-full bg-gradient-to-br from-amber-900/40 to-amber-950/60
                    border border-amber-700/50 flex items-center justify-center shadow-inner">
          <div class="w-9 h-9 rounded-full bg-gradient-to-br from-amber-600/30 to-amber-800/40
                      border border-amber-600/40 flex items-center justify-center">
            <span class="text-amber-500/70 text-sm font-bold">E</span>
          </div>
        </div>

        <!-- Corner accents -->
        <div class="absolute top-3 left-3 w-3 h-3 border-l border-t border-amber-700/30"></div>
        <div class="absolute top-3 right-3 w-3 h-3 border-r border-t border-amber-700/30"></div>
        <div class="absolute bottom-3 left-3 w-3 h-3 border-l border-b border-amber-700/30"></div>
        <div class="absolute bottom-3 right-3 w-3 h-3 border-r border-b border-amber-700/30"></div>
      </div>
    {:else}
      <!-- Card art background -->
      {#if card.artPath}
        <div class="absolute inset-0 overflow-hidden rounded-md">
          <img
            src="/{card.artPath}"
            alt=""
            class="w-full h-full object-cover object-top opacity-40"
            loading="lazy"
            decoding="async"
            onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
          />
          <div class="absolute inset-0 bg-gradient-to-b from-transparent via-black/40 to-black/70"></div>
        </div>
      {/if}

      <!-- Cost badge -->
      <div class="absolute -top-2 -right-2 w-8 h-8 rounded-full bg-mana flex items-center justify-center
                  text-white text-sm font-bold shadow-md border-2 border-blue-400 z-10">
        {card.cost}
      </div>

      <!-- Keyboard hint badge -->
      {#if showKeyHint && gameSettings.showKeyboardHints && index < keyHints.length}
        <div class="absolute -top-1 -left-1 w-5 h-5 rounded bg-gray-800/90 flex items-center justify-center
                    text-ui-text-dim text-xs font-mono border border-gray-600 z-10">
          {keyHints[index]}
        </div>
      {/if}

      <!-- Playable indicator glow -->
      {#if isPlayable && !isSelected}
        <div class="absolute inset-0 rounded-lg animate-pulse opacity-30 pointer-events-none
                    {getFactionBorder(card.faction).replace('border-', 'bg-')}"></div>
      {/if}

      <!-- Card content -->
      <div class="flex-1 flex flex-col p-1.5 pt-2 relative z-0">
        <!-- Name -->
        <div class="text-sm font-semibold truncate w-full text-center leading-tight text-ui-text px-0.5">
          {card.name}
        </div>

        <!-- Type -->
        <div class="text-xs text-ui-text-dim capitalize mt-0.5 text-center">
          {card.cardType}
        </div>

        <!-- Keywords preview (icons only, first 3) -->
        {#if card.keywords && card.keywords.length > 0}
          <div class="flex flex-wrap justify-center gap-0.5 mt-1">
            {#each card.keywords.slice(0, 3) as keyword}
              <KeywordIcon {keyword} size={14} showLabel={false} />
            {/each}
            {#if card.keywords.length > 3}
              <span class="text-xs text-ui-text-dim">+{card.keywords.length - 3}</span>
            {/if}
          </div>
        {/if}

        <!-- Stats -->
        {#if card.cardType === "creature" && card.attack !== undefined && card.health !== undefined}
          <div class="flex justify-center items-center gap-1.5 mt-auto mb-1">
            <span class="w-6 h-6 rounded bg-damage/20 flex items-center justify-center text-damage text-sm font-bold">
              {card.attack}
            </span>
            <span class="w-6 h-6 rounded bg-health/20 flex items-center justify-center text-health text-sm font-bold">
              {card.health}
            </span>
          </div>
        {:else if card.cardType === "support" && card.durability !== undefined}
          <div class="flex justify-center mt-auto mb-1">
            <span class="w-6 h-6 rounded bg-mana/20 flex items-center justify-center text-mana text-sm font-bold">
              {card.durability}
            </span>
          </div>
        {:else if card.cardType === "spell"}
          <div class="flex-1 flex items-center justify-center">
            <span class="text-xs text-ui-text-dim">Spell</span>
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
