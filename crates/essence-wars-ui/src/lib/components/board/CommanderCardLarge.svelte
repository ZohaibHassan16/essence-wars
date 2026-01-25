<script lang="ts">
  import type { CommanderDto } from "$lib/api/types";

  let {
    commander,
    life,
    maxLife,
    essence = 0,
    maxEssence = 0,
    isActive = false,
    isPlayer = true,
  }: {
    commander: CommanderDto | null;
    life: number;
    maxLife: number;
    essence?: number;
    maxEssence?: number;
    isActive?: boolean;
    isPlayer?: boolean;
  } = $props();

  // Track life changes for animation
  let previousLife = $state<number | null>(null);
  let lifeChangeClass = $state("");

  $effect(() => {
    if (previousLife !== null && life !== previousLife) {
      lifeChangeClass = life < previousLife ? "life-decreased" : life > previousLife ? "life-increased" : "";
      // Clear animation class after animation completes
      setTimeout(() => {
        lifeChangeClass = "";
      }, 600);
    }
    previousLife = life;
  });

  // Hover state for popup
  let showPopup = $state(false);

  // Get faction-specific styling
  const factionColors = $derived(() => {
    if (!commander) return {
      gradient: "from-gray-800 to-gray-900",
      border: "border-gray-600",
      glow: "",
      text: "text-gray-400",
      accent: "#6b7280",
    };

    switch (commander.faction) {
      case "argentum":
        return {
          gradient: "from-argentum-brass/20 via-argentum-gold/10 to-argentum-brass/20",
          border: "border-argentum-gold/60",
          glow: isActive ? "shadow-[0_0_25px_rgba(212,175,55,0.5)]" : "",
          text: "text-argentum-gold",
          accent: "#D4AF37",
        };
      case "symbiote":
        return {
          gradient: "from-symbiote-primary/30 via-symbiote-purple/20 to-symbiote-primary/30",
          border: "border-symbiote-glow/50",
          glow: isActive ? "shadow-[0_0_25px_rgba(127,255,0,0.4)]" : "",
          text: "text-symbiote-glow",
          accent: "#7FFF00",
        };
      case "obsidion":
        return {
          gradient: "from-obsidion-primary/25 via-obsidion-black/40 to-obsidion-primary/25",
          border: "border-obsidion-essence/50",
          glow: isActive ? "shadow-[0_0_25px_rgba(0,255,255,0.4)]" : "",
          text: "text-obsidion-essence",
          accent: "#00FFFF",
        };
      case "neutral":
        return {
          gradient: "from-neutral-primary/25 via-neutral-tan/15 to-neutral-primary/25",
          border: "border-neutral-copper/50",
          glow: isActive ? "shadow-[0_0_25px_rgba(184,115,51,0.4)]" : "",
          text: "text-neutral-copper",
          accent: "#B87333",
        };
      default:
        return {
          gradient: "from-gray-800 to-gray-900",
          border: "border-gray-600",
          glow: "",
          text: "text-gray-400",
          accent: "#6b7280",
        };
    }
  });

  // Life percentage for color coding
  const lifePercent = $derived(maxLife > 0 ? (life / maxLife) * 100 : 0);
  const lifeColor = $derived(() => {
    if (lifePercent > 66) return "text-health";
    if (lifePercent > 33) return "text-yellow-400";
    return "text-damage";
  });

  // Parse ability text to highlight keywords
  const keywordDefinitions: Record<string, string> = {
    "Rush": "Can attack immediately when played",
    "Guard": "Enemies must attack this creature first",
    "Ranged": "Can attack any enemy creature",
    "Piercing": "Excess damage hits the enemy player",
    "Lifesteal": "Heals your commander for damage dealt",
    "Lethal": "Destroys any creature it damages",
    "Shield": "Blocks the first instance of damage",
    "Quick": "Attacks before the defender",
    "Ephemeral": "Dies at end of turn",
    "Regenerate": "Heals to full at start of turn",
    "Stealth": "Cannot be targeted until it attacks",
    "Charge": "Gains +1 Attack when attacking",
    "Frenzy": "Attacks twice each combat",
    "Volatile": "Deals damage to adjacent enemies on death",
  };

  // Find keywords in ability text
  const highlightedKeywords = $derived(() => {
    if (!commander) return [];
    const text = commander.abilityDescription;
    return Object.keys(keywordDefinitions).filter(kw =>
      text.toLowerCase().includes(kw.toLowerCase())
    );
  });

  // Faction display name
  const factionDisplayName = $derived(() => {
    if (!commander) return "";
    switch (commander.faction) {
      case "argentum": return "Argentum Combine";
      case "symbiote": return "Symbiote Circles";
      case "obsidion": return "Obsidion Syndicate";
      case "neutral": return "Free-Walkers";
      default: return commander.faction;
    }
  });
</script>

<div
  class="commander-card relative flex flex-col rounded-xl border-2 bg-gradient-to-b overflow-hidden transition-all duration-300
         {factionColors().gradient} {factionColors().border} {factionColors().glow}"
  class:opacity-60={!commander}
  class:active-pulse={isActive}
  style="width: var(--commander-card-width, 250px);"
  onmouseenter={() => showPopup = true}
  onmouseleave={() => showPopup = false}
  role="region"
  aria-label={commander ? `${commander.name} - ${life} HP` : "No commander"}
>
  {#if commander}
    <!-- Portrait Section -->
    <div class="relative" style="height: var(--commander-portrait-height, 150px);">
      <img
        src={`/${commander.portraitPath}`}
        alt={commander.name}
        class="w-full h-full object-cover object-top"
        style="object-position: center 20%;"
      />
      <!-- Gradient overlay at bottom of portrait -->
      <div class="absolute inset-x-0 bottom-0 h-12 bg-gradient-to-t from-black/80 to-transparent"></div>

      <!-- Active turn indicator overlay -->
      {#if isActive}
        <div class="absolute inset-0 border-4 border-white/30 rounded-t-lg animate-pulse pointer-events-none"></div>
      {/if}
    </div>

    <!-- Faction Divider -->
    <div class="h-1 w-full" style="background: linear-gradient(90deg, transparent, {factionColors().accent}, transparent);"></div>

    <!-- Commander Info Section -->
    <div class="flex-1 flex flex-col p-3 bg-ui-panel/90">
      <!-- Name -->
      <h3 class="text-base font-bold {factionColors().text} truncate leading-tight">
        {commander.name}
      </h3>

      <!-- Life Display -->
      <div class="flex items-center justify-center my-2">
        <div
          class="relative px-4 py-2 rounded-lg bg-black/40 border {factionColors().border} {lifeChangeClass}"
        >
          <div class="flex items-baseline gap-1">
            <span class="text-2xl font-bold {lifeColor()}">{life}</span>
            <span class="text-sm text-ui-text-dim">/ {maxLife}</span>
          </div>
          <div class="text-xs text-center text-ui-text-dim mt-0.5">HP</div>
        </div>
      </div>

      <!-- Essence Display (compact) -->
      {#if maxEssence > 0}
        <div class="flex items-center justify-center gap-1 mb-2">
          {#each Array(maxEssence) as _, i}
            <div
              class="w-3 h-3 rounded-full border transition-all duration-200"
              class:bg-mana={i < essence}
              class:border-mana={i < essence}
              class:bg-transparent={i >= essence}
              class:border-gray-600={i >= essence}
            ></div>
          {/each}
        </div>
      {/if}

      <!-- Ability Text -->
      <div class="text-xs text-ui-text leading-relaxed line-clamp-3">
        {commander.abilityDescription}
      </div>

      <!-- Faction Badge -->
      <div class="mt-auto pt-2">
        <div class="h-px w-full mb-2" style="background: linear-gradient(90deg, transparent, {factionColors().accent}40, transparent);"></div>
        <div class="text-xs font-semibold {factionColors().text} text-center uppercase tracking-wider">
          {factionDisplayName()}
        </div>
      </div>
    </div>
  {:else}
    <!-- Placeholder when no commander -->
    <div class="flex-1 flex flex-col items-center justify-center p-4 min-h-[300px]">
      <div class="w-16 h-16 rounded-full bg-gray-700/50 border border-gray-600 flex items-center justify-center mb-3">
        <span class="text-2xl text-gray-500">?</span>
      </div>
      <span class="text-sm text-ui-text-dim">No Commander</span>
    </div>
  {/if}

  <!-- Hover Popup -->
  {#if showPopup && commander}
    <div
      class="absolute z-50 p-4 rounded-xl border bg-ui-panel/95 backdrop-blur-sm shadow-2xl
             {factionColors().border}"
      class:left-full={isPlayer}
      class:right-full={!isPlayer}
      class:ml-3={isPlayer}
      class:mr-3={!isPlayer}
      style="top: 50%; transform: translateY(-50%); width: 280px;"
    >
      <!-- Popup Header -->
      <div class="flex items-center gap-3 mb-3">
        <img
          src={`/${commander.portraitPath}`}
          alt={commander.name}
          class="w-12 h-12 rounded-lg object-cover border {factionColors().border}"
        />
        <div>
          <h4 class="font-bold {factionColors().text}">{commander.name}</h4>
          <div class="text-xs text-ui-text-dim">{factionDisplayName()}</div>
        </div>
      </div>

      <!-- Full Ability Text -->
      <div class="mb-3">
        <div class="text-xs font-semibold text-ui-text-dim uppercase tracking-wide mb-1">Ability</div>
        <p class="text-sm text-ui-text leading-relaxed">{commander.abilityDescription}</p>
      </div>

      <!-- Keyword Definitions -->
      {#if highlightedKeywords().length > 0}
        <div class="border-t border-gray-700 pt-3">
          <div class="text-xs font-semibold text-ui-text-dim uppercase tracking-wide mb-2">Keywords</div>
          <div class="space-y-1.5">
            {#each highlightedKeywords() as keyword}
              <div class="text-xs">
                <span class="font-semibold {factionColors().text}">{keyword}:</span>
                <span class="text-ui-text-dim">{keywordDefinitions[keyword]}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Stats Summary -->
      <div class="border-t border-gray-700 pt-3 mt-3">
        <div class="flex justify-between text-sm">
          <span class="text-ui-text-dim">Current HP:</span>
          <span class="{lifeColor()} font-bold">{life} / {maxLife}</span>
        </div>
        {#if maxEssence > 0}
          <div class="flex justify-between text-sm mt-1">
            <span class="text-ui-text-dim">Essence:</span>
            <span class="text-mana font-bold">{essence} / {maxEssence}</span>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  /* Life change animations */
  .life-decreased {
    animation: life-flash-red 0.6s ease-out;
  }

  .life-increased {
    animation: life-flash-green 0.6s ease-out;
  }

  @keyframes life-flash-red {
    0%, 100% { background-color: rgba(0, 0, 0, 0.4); }
    50% { background-color: rgba(239, 68, 68, 0.4); }
  }

  @keyframes life-flash-green {
    0%, 100% { background-color: rgba(0, 0, 0, 0.4); }
    50% { background-color: rgba(34, 197, 94, 0.4); }
  }

  /* Active turn pulse animation */
  .active-pulse {
    animation: commander-pulse 2s ease-in-out infinite;
  }

  @keyframes commander-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.9; }
  }
</style>
