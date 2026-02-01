<script lang="ts">
  import type { CardDto, CreatureDto, SupportDto } from "$lib/api/types";
  import KeywordIcon from "./KeywordIcon.svelte";

  let {
    card,
    creature = null,
    support = null,
    position = "right",
  }: {
    card: CardDto | null;
    creature?: CreatureDto | null;
    support?: SupportDto | null;
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

  // Keyword descriptions for the preview
  const keywordDescriptions: Record<string, string> = {
    Rush: "Can attack immediately when played",
    Ranged: "Can attack any enemy creature",
    Piercing: "Excess damage hits the enemy player",
    Guard: "Must be attacked before other creatures",
    Lifesteal: "Heals your hero equal to damage dealt",
    Lethal: "Destroys any creature it damages",
    Shield: "Blocks the first damage taken",
    Quick: "Can attack twice per turn",
    Ephemeral: "Dies at end of turn",
    Regenerate: "Heals 1 HP at start of your turn",
    Stealth: "Cannot be targeted until it attacks",
    Charge: "Gains +1 attack each turn",
    Frenzy: "Gains +1 attack when damaged",
    Volatile: "Deals damage to all when it dies",
  };

  function getKeywordDescription(keyword: string): string {
    return keywordDescriptions[keyword] ?? "Unknown keyword";
  }

  const positionClass = $derived({
    right: "left-full ml-4",
    left: "right-full mr-4",
    top: "bottom-full mb-4",
    bottom: "top-full mt-4",
  }[position]);

  // Track whether we've fallen back to generic art
  let usesFallbackArt = $state(false);

  // Reset fallback state when card changes
  $effect(() => {
    if (card) {
      usesFallbackArt = false;
    }
  });

  // Generate fallback art path based on faction
  const fallbackArtPath = $derived(
    card ? `tokens/generic/${card.faction || 'neutral'}.webp` : null
  );

  // Current art path to use (primary or fallback)
  const currentArtPath = $derived(
    usesFallbackArt ? fallbackArtPath : card?.artPath
  );

  // Handle image load error - try fallback, then show placeholder
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

    <!-- Card Art (with fallback support) -->
    <div class="h-48 bg-gray-800/50 overflow-hidden border-b border-gray-700 relative">
      {#if currentArtPath}
        <img
          src="/{currentArtPath}"
          alt={card.name}
          class="w-full h-full object-cover object-center"
          onerror={handleImageError}
        />
      {/if}
      <div class="absolute inset-0 flex items-center justify-center text-ui-text-dim text-sm pointer-events-none"
           class:hidden={currentArtPath}>
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
          <span class="text-mana font-bold text-xl">{support?.durability ?? card.durability}</span>
          <span class="text-ui-text-dim text-base">Durability</span>
        </div>
      </div>
    {/if}

    <!-- Support Effect Description (from card or support DTO) -->
    {#if card.effectDescription || support?.effectDescription}
      <div class="px-4 py-3 border-b border-gray-700">
        <div class="text-xs font-semibold text-ui-text-dim uppercase tracking-wide mb-1">Effect</div>
        <p class="text-sm text-ui-text leading-relaxed">{card.effectDescription ?? support?.effectDescription}</p>
      </div>
    {/if}

    <!-- Keywords with descriptions -->
    {#if card.keywords && card.keywords.length > 0}
      <div class="px-4 py-3 border-b border-gray-700">
        <div class="space-y-1.5">
          {#each card.keywords as keyword}
            <div class="flex items-start gap-2">
              <KeywordIcon {keyword} size={14} showLabel={false} />
              <div class="flex-1 min-w-0">
                <span class="text-sm font-medium text-ui-text">{keyword}</span>
                <p class="text-xs text-ui-text-dim leading-tight">{getKeywordDescription(keyword)}</p>
              </div>
            </div>
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
