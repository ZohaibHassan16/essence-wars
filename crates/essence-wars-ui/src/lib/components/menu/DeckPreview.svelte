<script lang="ts">
  import type { DeckInfo } from "$lib/api/types";

  let {
    deck,
  }: {
    deck: DeckInfo | null;
  } = $props();

  // Faction-specific styling
  const factionStyles = $derived(() => {
    if (!deck) return {
      border: "border-gray-600",
      gradient: "from-gray-800 to-gray-900",
      text: "text-gray-400",
      accent: "#6b7280",
      name: "Unknown",
    };

    switch (deck.faction) {
      case "argentum":
        return {
          border: "border-argentum-gold/60",
          gradient: "from-argentum-brass/20 via-argentum-gold/10 to-argentum-brass/20",
          text: "text-argentum-gold",
          accent: "#D4AF37",
          name: "Argentum Combine",
        };
      case "symbiote":
        return {
          border: "border-symbiote-glow/50",
          gradient: "from-symbiote-primary/30 via-symbiote-purple/20 to-symbiote-primary/30",
          text: "text-symbiote-glow",
          accent: "#7FFF00",
          name: "Symbiote Circles",
        };
      case "obsidion":
        return {
          border: "border-obsidion-essence/50",
          gradient: "from-obsidion-primary/25 via-obsidion-black/40 to-obsidion-primary/25",
          text: "text-obsidion-essence",
          accent: "#00FFFF",
          name: "Obsidion Syndicate",
        };
      default:
        return {
          border: "border-neutral-copper/50",
          gradient: "from-neutral-primary/25 via-neutral-tan/15 to-neutral-primary/25",
          text: "text-neutral-copper",
          accent: "#B87333",
          name: "Free-Walkers",
        };
    }
  });

  // Faction emblem path
  const factionEmblem = $derived(() => {
    if (!deck) return null;
    return `/ui/decorations/emblems/emblem_${deck.faction}.png`;
  });
</script>

<div
  class="deck-preview rounded-xl border-2 bg-gradient-to-b overflow-hidden transition-all duration-300
         {factionStyles().border} {factionStyles().gradient} bg-ui-panel/90"
  style="width: 320px; min-height: 400px;"
>
  {#if deck}
    <!-- Commander Portrait (larger) -->
    <div class="relative w-full aspect-[4/3] overflow-hidden bg-gray-900">
      {#if deck.commander?.portraitPath}
        <img
          src={`/${deck.commander.portraitPath}`}
          alt={deck.commander.name}
          class="w-full h-full object-cover object-top"
          style="object-position: center 20%;"
          loading="lazy"
        />
      {:else}
        <div class="w-full h-full flex items-center justify-center bg-gray-800">
          <span class="text-6xl text-gray-600">?</span>
        </div>
      {/if}

      <!-- Gradient overlay at bottom -->
      <div class="absolute inset-x-0 bottom-0 h-16 bg-gradient-to-t from-black/90 to-transparent"></div>
    </div>

    <!-- Faction Divider -->
    <div
      class="h-1 w-full"
      style="background: linear-gradient(90deg, transparent, {factionStyles().accent}, transparent);"
    ></div>

    <!-- Deck Information -->
    <div class="p-4">
      <!-- Commander Name -->
      <h3 class="text-lg font-bold {factionStyles().text}">
        {deck.commander?.name ?? "Unknown Commander"}
      </h3>

      <!-- Deck Name -->
      <p class="text-sm text-ui-text mt-1">
        "{deck.name}"
      </p>

      <!-- Playstyle Tag -->
      {#if deck.playstyle}
        <div class="mt-2">
          <span
            class="inline-block px-3 py-1 text-xs rounded-full bg-gray-700/50 {factionStyles().text} font-medium"
          >
            {deck.playstyle}
          </span>
        </div>
      {/if}

      <!-- Divider -->
      <div
        class="my-4 h-px"
        style="background: linear-gradient(90deg, transparent, {factionStyles().accent}40, transparent);"
      ></div>

      <!-- Description -->
      <p class="text-sm text-ui-text-dim leading-relaxed">
        {deck.description || "No description available."}
      </p>

      <!-- Commander Ability -->
      {#if deck.commander?.abilityDescription}
        <div class="mt-4">
          <div class="text-xs font-semibold text-ui-text-dim uppercase tracking-wide mb-1">
            Commander Ability
          </div>
          <p class="text-sm text-ui-text leading-relaxed">
            {deck.commander.abilityDescription}
          </p>
        </div>
      {/if}

      <!-- Stats Footer -->
      <div class="mt-4 pt-3 border-t border-gray-700/50">
        <div class="flex justify-between items-center text-xs">
          <span class="text-ui-text-dim">
            <span class="font-semibold text-ui-text">{deck.cardCount}</span> cards
          </span>
          <div class="flex items-center gap-1.5">
            {#if factionEmblem()}
              <img
                src={factionEmblem()}
                alt={factionStyles().name}
                class="w-5 h-5 object-contain drop-shadow-sm"
              />
            {/if}
            <span class="{factionStyles().text} font-medium uppercase tracking-wider">
              {factionStyles().name}
            </span>
          </div>
        </div>
      </div>
    </div>
  {:else}
    <!-- Empty state -->
    <div class="flex flex-col items-center justify-center h-full py-16 px-8">
      <div class="w-20 h-20 rounded-full bg-gray-700/50 border border-gray-600 flex items-center justify-center mb-4">
        <svg class="w-10 h-10 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
      </div>
      <p class="text-ui-text-dim text-center">
        Select a deck to see details
      </p>
    </div>
  {/if}
</div>

<style>
  .deck-preview {
    animation: slideIn 0.3s ease-out;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateX(20px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }
</style>
