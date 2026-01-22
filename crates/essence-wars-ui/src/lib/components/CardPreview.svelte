<script lang="ts">
  import type { CardDto, CreatureDto } from "$lib/api/types";

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
    class="absolute {positionClass} z-50 w-56 rounded-xl border-2 shadow-2xl pointer-events-none
           {getFactionBorder(card.faction)} {getFactionBg(card.faction)}"
    style="backdrop-filter: blur(8px);"
  >
    <!-- Card Header -->
    <div class="px-3 py-2 border-b border-gray-700">
      <div class="flex items-center justify-between">
        <span class="font-bold text-ui-text">{card.name}</span>
        <span class="w-7 h-7 rounded-full bg-mana flex items-center justify-center text-white text-sm font-bold">
          {card.cost}
        </span>
      </div>
      <div class="text-xs {getFactionAccent(card.faction)} capitalize mt-0.5">
        {card.faction} {card.cardType}
      </div>
    </div>

    <!-- Card Art Placeholder -->
    <div class="h-28 bg-gray-800/50 flex items-center justify-center border-b border-gray-700">
      <div class="text-ui-text-dim text-xs">[Card Art]</div>
    </div>

    <!-- Stats -->
    {#if card.cardType === "creature"}
      <div class="px-3 py-2 border-b border-gray-700">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="text-damage font-bold text-lg">{creature?.attack ?? card.attack}</span>
            <span class="text-ui-text-dim">/</span>
            <span class="text-health font-bold text-lg">{creature?.health ?? card.health}</span>
            {#if creature && creature.maxHealth !== creature.health}
              <span class="text-ui-text-dim text-sm">({creature.maxHealth} max)</span>
            {/if}
          </div>
        </div>
      </div>
    {:else if card.cardType === "support" && card.durability}
      <div class="px-3 py-2 border-b border-gray-700">
        <div class="flex items-center gap-2">
          <span class="text-mana font-bold text-lg">{card.durability}</span>
          <span class="text-ui-text-dim text-sm">Durability</span>
        </div>
      </div>
    {/if}

    <!-- Keywords -->
    {#if card.keywords && card.keywords.length > 0}
      <div class="px-3 py-2 border-b border-gray-700">
        <div class="flex flex-wrap gap-1">
          {#each card.keywords as keyword}
            <span class="px-2 py-0.5 rounded text-xs bg-ui-panel text-ui-text border border-gray-600">
              {keyword}
            </span>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Creature Status (if on board) -->
    {#if creature}
      <div class="px-3 py-2 border-b border-gray-700">
        <div class="flex items-center gap-2 text-xs">
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
    <div class="px-3 py-1.5 text-[10px] text-ui-text-dim">
      ID: {card.cardId}
    </div>
  </div>
{/if}
