<script lang="ts">
  import type { CardDto, CreatureDto } from "$lib/api/types";
  import KeywordIcon from "./KeywordIcon.svelte";

  let {
    card,
    creature = null,
    position = "right",
  }: {
    card: CardDto | null;
    creature?: CreatureDto | null;
    position?: "left" | "right" | "top" | "bottom";
  } = $props();

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
      case "argentum": return "bg-gradient-to-b from-gray-800 via-gray-900 to-gray-800";
      case "symbiote": return "bg-gradient-to-b from-green-950 via-gray-900 to-purple-950";
      case "obsidion": return "bg-gradient-to-b from-red-950 via-gray-900 to-gray-950";
      default: return "bg-gradient-to-b from-amber-950 via-gray-900 to-stone-900";
    }
  }

  function getFactionAccent(faction: string): string {
    switch (faction) {
      case "argentum": return "text-argentum-gold";
      case "symbiote": return "text-symbiote-glow";
      case "obsidion": return "text-obsidion-essence";
      default: return "text-neutral-copper";
    }
  }

  const positionClass = $derived({
    right: "left-full ml-4",
    left: "right-full mr-4",
    top: "bottom-full mb-4",
    bottom: "top-full mt-4",
  }[position]);
</script>

{#if card}
  <div
    class="absolute {positionClass} z-50 rounded-xl border-2 shadow-2xl pointer-events-none
           {getFactionBorder(card.faction)} {getFactionBg(card.faction)}"
    style="width: var(--card-preview-width); backdrop-filter: blur(8px);"
  >
    <!-- Card Header -->
    <div class="px-4 py-3 border-b border-gray-700">
      <div class="flex items-center justify-between">
        <span class="font-bold text-ui-text text-lg">{card.name}</span>
        <span class="w-8 h-8 rounded-full bg-mana flex items-center justify-center text-white text-base font-bold">
          {card.cost}
        </span>
      </div>
      <div class="text-sm {getFactionAccent(card.faction)} capitalize mt-1">
        {card.faction} {card.cardType}
      </div>
    </div>

    <!-- Card Art -->
    <div class="h-36 bg-gray-800/50 overflow-hidden border-b border-gray-700 relative">
      {#if card.artPath}
        <img
          src="/{card.artPath}"
          alt={card.name}
          class="w-full h-full object-cover object-top"
          onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
        />
      {/if}
      <div class="absolute inset-0 flex items-center justify-center text-ui-text-dim text-sm pointer-events-none"
           class:hidden={card.artPath}>
        [No Art]
      </div>
    </div>

    <!-- Stats -->
    {#if card.cardType === "creature"}
      <div class="px-4 py-3 border-b border-gray-700">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span class="text-damage font-bold text-xl">{creature?.attack ?? card.attack}</span>
            <span class="text-ui-text-dim text-lg">/</span>
            <span class="text-health font-bold text-xl">{creature?.health ?? card.health}</span>
            {#if creature && creature.maxHealth !== creature.health}
              <span class="text-ui-text-dim text-base">({creature.maxHealth} max)</span>
            {/if}
          </div>
        </div>
      </div>
    {:else if card.cardType === "support" && card.durability}
      <div class="px-4 py-3 border-b border-gray-700">
        <div class="flex items-center gap-3">
          <span class="text-mana font-bold text-xl">{card.durability}</span>
          <span class="text-ui-text-dim text-base">Durability</span>
        </div>
      </div>
    {/if}

    <!-- Keywords -->
    {#if card.keywords && card.keywords.length > 0}
      <div class="px-4 py-3 border-b border-gray-700">
        <div class="flex flex-wrap gap-1.5">
          {#each card.keywords as keyword}
            <KeywordIcon {keyword} size={14} />
          {/each}
        </div>
      </div>
    {/if}

    <!-- Creature Status (if on board) -->
    {#if creature}
      <div class="px-4 py-3 border-b border-gray-700">
        <div class="flex items-center gap-2 text-sm">
          {#if creature.canAttack}
            <span class="text-health">Ready to attack</span>
          {:else if creature.isExhausted}
            <span class="text-yellow-500">Exhausted</span>
          {:else}
            <span class="text-ui-text-dim">Summoning sickness</span>
          {/if}
        </div>
      </div>
    {/if}

    <!-- Card ID (debug info) -->
    <div class="px-4 py-2 text-xs text-ui-text-dim">
      ID: {card.cardId}
    </div>
  </div>
{/if}
