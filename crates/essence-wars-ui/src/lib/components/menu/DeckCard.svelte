<script lang="ts">
  import type { DeckInfo } from "$lib/api/types";
  import { playSound } from "$lib/audio";

  let {
    deck,
    isSelected = false,
    onSelect,
  }: {
    deck: DeckInfo;
    isSelected?: boolean;
    onSelect?: () => void;
  } = $props();

  // Faction-specific styling
  const factionStyles = $derived(() => {
    switch (deck.faction) {
      case "argentum":
        return {
          border: "border-argentum-gold",
          borderHover: "hover:border-argentum-gold/80",
          glow: "shadow-[0_0_20px_rgba(212,175,55,0.4)]",
          gradient: "from-argentum-brass/20 via-argentum-gold/10 to-argentum-brass/20",
          text: "text-argentum-gold",
          accent: "#D4AF37",
        };
      case "symbiote":
        return {
          border: "border-symbiote-glow",
          borderHover: "hover:border-symbiote-glow/80",
          glow: "shadow-[0_0_20px_rgba(127,255,0,0.4)]",
          gradient: "from-symbiote-primary/30 via-symbiote-purple/20 to-symbiote-primary/30",
          text: "text-symbiote-glow",
          accent: "#7FFF00",
        };
      case "obsidion":
        return {
          border: "border-obsidion-essence",
          borderHover: "hover:border-obsidion-essence/80",
          glow: "shadow-[0_0_20px_rgba(0,255,255,0.4)]",
          gradient: "from-obsidion-primary/25 via-obsidion-black/40 to-obsidion-primary/25",
          text: "text-obsidion-essence",
          accent: "#00FFFF",
        };
      default:
        return {
          border: "border-neutral-copper",
          borderHover: "hover:border-neutral-copper/80",
          glow: "shadow-[0_0_20px_rgba(184,115,51,0.4)]",
          gradient: "from-neutral-primary/25 via-neutral-tan/15 to-neutral-primary/25",
          text: "text-neutral-copper",
          accent: "#B87333",
        };
    }
  });

  function handleClick() {
    playSound("cardSelect");
    onSelect?.();
  }

  function handleMouseEnter() {
    playSound("cardHover");
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      handleClick();
    }
  }
</script>

<button
  class="deck-card relative flex flex-col rounded-xl border-2 overflow-hidden transition-all duration-300
         bg-gradient-to-b {factionStyles().gradient} bg-ui-panel/90
         {isSelected ? factionStyles().border + ' ' + factionStyles().glow : 'border-gray-600 ' + factionStyles().borderHover}
         hover:scale-[1.02] active:scale-[0.98]
         focus:outline-none focus:ring-2 focus:ring-ui-action focus:ring-offset-2 focus:ring-offset-ui-bg"
  onclick={handleClick}
  onmouseenter={handleMouseEnter}
  onkeydown={handleKeyDown}
>
  <!-- Commander Portrait -->
  <div class="relative w-full aspect-[4/3] overflow-hidden bg-gray-900">
    {#if deck.commander?.portraitPath}
      <img
        src={`/${deck.commander.portraitPath}`}
        alt={deck.commander.name}
        class="w-full h-full object-cover object-top transition-transform duration-300"
        style="object-position: center 20%;"
        loading="lazy"
      />
    {:else}
      <div class="w-full h-full flex items-center justify-center bg-gray-800">
        <span class="text-4xl text-gray-600">?</span>
      </div>
    {/if}

    <!-- Gradient overlay at bottom -->
    <div class="absolute inset-x-0 bottom-0 h-8 bg-gradient-to-t from-black/80 to-transparent"></div>

    <!-- Selected indicator overlay -->
    {#if isSelected}
      <div class="absolute inset-0 border-4 {factionStyles().border} rounded-t-lg opacity-60"></div>
    {/if}
  </div>

  <!-- Faction Divider -->
  <div
    class="h-0.5 w-full"
    style="background: linear-gradient(90deg, transparent, {factionStyles().accent}, transparent);"
  ></div>

  <!-- Card Info -->
  <div class="flex-1 flex flex-col p-3">
    <!-- Commander Name -->
    <h4 class="text-sm font-bold {factionStyles().text} truncate leading-tight">
      {deck.commander?.name ?? "Unknown Commander"}
    </h4>

    <!-- Deck Name -->
    <p class="text-xs text-ui-text mt-1 truncate">
      "{deck.name}"
    </p>

    <!-- Playstyle Tag -->
    {#if deck.playstyle}
      <div class="mt-2">
        <span
          class="inline-block px-2 py-0.5 text-xs rounded-full bg-gray-700/50 {factionStyles().text}"
        >
          {deck.playstyle}
        </span>
      </div>
    {/if}

    <!-- Selection Indicator -->
    <div class="mt-auto pt-2">
      {#if isSelected}
        <div class="flex items-center justify-center gap-1 text-xs font-semibold {factionStyles().text}">
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
          </svg>
          SELECTED
        </div>
      {:else}
        <div class="h-5"></div>
      {/if}
    </div>
  </div>
</button>

<style>
  .deck-card {
    cursor: pointer;
    width: 180px;
    will-change: transform;
  }

  .deck-card:hover img {
    transform: scale(1.05);
  }

  .deck-card img {
    will-change: transform;
  }

  /* Responsive: smaller cards on narrow screens */
  @media (max-width: 1200px) {
    .deck-card {
      width: 160px;
    }
  }

  @media (max-width: 1000px) {
    .deck-card {
      width: 140px;
    }
  }

  @media (max-width: 800px) {
    .deck-card {
      width: 160px; /* More room when preview panel is hidden */
    }
  }
</style>
