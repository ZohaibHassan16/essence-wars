<script lang="ts">
  import type { CommanderDto } from "$lib/api/types";
  import { playSound } from "$lib/audio";
  import { assetUrl } from "$lib/utils/paths";

  // Essence crystal icon
  const essenceCrystalIcon = assetUrl("/ui/decorations/icons/essence_icon_crystal.png");

  const ESSENCE_EXTRACTION_THRESHOLD = 50;

  let {
    commander,
    life,
    maxLife,
    essence = 0,
    maxEssence = 0,
    essenceExtracted = 0,
    isActive = false,
    isPlayer = true,
    insightAvailable = false,
    insightIndicator = false,
    onInsightClick,
    tutorialId = undefined,
    /** Whether this commander is a valid target for face-targeting abilities */
    isValidFaceTarget = false,
    /** Called when commander is clicked as a face target */
    onFaceTargetClick,
  }: {
    commander: CommanderDto | null;
    life: number;
    maxLife: number;
    essence?: number;
    maxEssence?: number;
    essenceExtracted?: number;
    isActive?: boolean;
    isPlayer?: boolean;
    insightAvailable?: boolean;
    insightIndicator?: boolean;
    onInsightClick?: () => void;
    tutorialId?: string;
    isValidFaceTarget?: boolean;
    onFaceTargetClick?: () => void;
  } = $props();

  // Calculate extraction progress percentage (capped at 100%)
  let extractionProgress = $derived(Math.min(100, (essenceExtracted / ESSENCE_EXTRACTION_THRESHOLD) * 100));

  // Color based on progress: green when close to winning, yellow at midpoint
  let extractionColor = $derived(
    extractionProgress >= 80 ? 'text-health' :
    extractionProgress >= 50 ? 'text-gold' :
    'text-orange-400'
  );

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

  // Handle face click for keyboard accessibility
  function handleFaceClick() {
    if (isValidFaceTarget && onFaceTargetClick) {
      playSound('abilityActivate');
      onFaceTargetClick();
    }
  }
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

  // Faction emblem path
  const factionEmblem = $derived(() => {
    if (!commander) return null;
    return assetUrl(`/ui/decorations/emblems/emblem_${commander.faction}.png`);
  });
</script>

<div
  class="commander-card relative flex flex-col rounded-xl border-2 bg-gradient-to-b transition-all duration-300
         {factionColors().gradient} {factionColors().border} {factionColors().glow}
         {isValidFaceTarget ? 'ring-2 ring-damage ring-offset-2 ring-offset-black cursor-pointer animate-pulse' : ''}"
  class:opacity-60={!commander}
  class:active-pulse={isActive && !isValidFaceTarget}
  style="width: var(--commander-card-width, 250px); overflow: visible;"
  role="button"
  tabindex="0"
  aria-label={commander ? `${commander.name} - ${life} HP` : "No commander"}
  data-tutorial-id={tutorialId}
  onkeydown={(e) => { if (isValidFaceTarget && (e.key === 'Enter' || e.key === ' ')) { e.preventDefault(); handleFaceClick(); } }}
  onmouseenter={() => showPopup = true}
  onmouseleave={() => showPopup = false}
  onclick={() => {
    if (isValidFaceTarget && onFaceTargetClick) {
      playSound('abilityActivate');
      onFaceTargetClick();
    }
  }}
>
  {#if commander}
    <!-- Portrait Section -->
    <div class="relative overflow-hidden rounded-t-lg" style="height: var(--commander-portrait-height, 150px);">
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

      <!-- Essence Extracted (VP Progress) -->
      <div class="flex items-center justify-center mb-2"
           title="Essence Extracted: {essenceExtracted}/{ESSENCE_EXTRACTION_THRESHOLD} - Extract 50 to win!">
        <div class="relative flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-black/40 border border-orange-500/40 overflow-hidden w-full max-w-[140px]">
          <!-- Progress bar background -->
          <div class="absolute inset-0 bg-orange-500/20 transition-all duration-500"
               style="width: {extractionProgress}%"></div>
          <!-- Content -->
          <img src={essenceCrystalIcon} alt="" class="w-4 h-4 object-contain relative z-10 flex-shrink-0 drop-shadow-sm" />
          <span class="{extractionColor} font-bold text-base relative z-10">{essenceExtracted}</span>
          <span class="text-orange-400/60 text-xs font-medium relative z-10">/ {ESSENCE_EXTRACTION_THRESHOLD}</span>
        </div>
      </div>

      <!-- Essence Display (crystal icons) -->
      {#if maxEssence > 0}
        <div class="flex items-center justify-center gap-0.5 mb-2">
          {#each Array(maxEssence) as _, i (i)}
            <img
              src={essenceCrystalIcon}
              alt=""
              class="w-5 h-5 object-contain transition-all duration-200 drop-shadow-sm
                     {i < essence ? 'opacity-100' : 'opacity-25 grayscale'}"
            />
          {/each}
        </div>
      {/if}

      <!-- Ability Text -->
      <div class="text-xs text-ui-text leading-relaxed line-clamp-3">
        {commander.abilityDescription}
      </div>

      <!-- Faction Badge with Emblem -->
      <div class="mt-auto pt-2">
        <div class="h-px w-full mb-2" style="background: linear-gradient(90deg, transparent, {factionColors().accent}40, transparent);"></div>
        <div class="flex items-center justify-center gap-2">
          {#if factionEmblem()}
            <img
              src={factionEmblem()}
              alt={factionDisplayName()}
              class="w-5 h-5 object-contain drop-shadow-sm"
            />
          {/if}
          <span class="text-xs font-semibold {factionColors().text} uppercase tracking-wider">
            {factionDisplayName()}
          </span>
        </div>
      </div>

      <!-- Commander's Insight Button (player side only) -->
      {#if isPlayer && (insightAvailable || insightIndicator)}
        <button
          class="w-full mt-3 px-3 py-2 rounded-lg font-semibold text-sm
                 flex items-center justify-center gap-2
                 transition-all duration-200
                 {insightAvailable
                   ? `bg-black/40 border-2 ${factionColors().border} ${factionColors().text} hover:bg-black/60 cursor-pointer`
                   : 'bg-black/20 border border-gray-600/50 text-gray-500 cursor-not-allowed'}"
          class:animate-pulse={insightAvailable}
          onclick={() => {
            if (insightAvailable && onInsightClick) {
              playSound('buttonClick');
              onInsightClick();
            }
          }}
          onmouseenter={() => insightAvailable && playSound('buttonHover')}
          disabled={!insightAvailable}
          title={insightAvailable
            ? "Draw a card for 4 Essence (I)"
            : "Available Turn 10+ when behind with low hand"}
        >
          <span class="text-lg">💡</span>
          <span>Insight</span>
          <span class="text-xs opacity-70">4⟡</span>
          {#if insightAvailable}
            <kbd class="ml-1 px-1 py-0.5 text-xs bg-black/30 rounded">I</kbd>
          {/if}
        </button>
      {/if}
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
            {#each highlightedKeywords() as keyword (keyword)}
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
        <div class="flex justify-between text-sm mt-1">
          <span class="text-ui-text-dim flex items-center gap-1">
            <img src={essenceCrystalIcon} alt="" class="w-3 h-3 object-contain" />
            Extracted:
          </span>
          <span class="{extractionColor} font-bold">{essenceExtracted} / {ESSENCE_EXTRACTION_THRESHOLD}</span>
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
