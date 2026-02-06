<script lang="ts">
  import { playSound } from "$lib/audio";

  let {
    selectedFaction = "argentum",
    onSelect,
  }: {
    selectedFaction?: string;
    onSelect?: (faction: string) => void;
  } = $props();

  const factions = [
    {
      id: "argentum",
      name: "Argentum Combine",
      shortName: "Argentum",
      emblem: "/ui/decorations/emblems/emblem_argentum.png",
      color: "argentum-gold",
      bgColor: "bg-argentum-gold/20",
      borderColor: "border-argentum-gold",
      textColor: "text-argentum-gold",
      description: "Masters of construct warfare",
    },
    {
      id: "symbiote",
      name: "Symbiote Circles",
      shortName: "Symbiote",
      emblem: "/ui/decorations/emblems/emblem_symbiote.png",
      color: "symbiote-glow",
      bgColor: "bg-symbiote-glow/20",
      borderColor: "border-symbiote-glow",
      textColor: "text-symbiote-glow",
      description: "Primal pack hunters",
    },
    {
      id: "obsidion",
      name: "Obsidion Syndicate",
      shortName: "Obsidion",
      emblem: "/ui/decorations/emblems/emblem_obsidion.png",
      color: "obsidion-essence",
      bgColor: "bg-obsidion-essence/20",
      borderColor: "border-obsidion-essence",
      textColor: "text-obsidion-essence",
      description: "Masters of shadow and burst",
    },
  ];

  function handleSelect(factionId: string) {
    playSound("buttonClick");
    onSelect?.(factionId);
  }

  function handleMouseEnter() {
    playSound("buttonHover");
  }
</script>

<div class="flex justify-center gap-2">
  {#each factions as faction (faction.id)}
    {@const isSelected = selectedFaction === faction.id}
    <button
      class="faction-tab relative flex flex-col items-center px-6 py-3 rounded-lg border-2 transition-all duration-200
             focus:outline-none focus:ring-2 focus:ring-ui-action focus:ring-offset-2 focus:ring-offset-ui-bg
             {isSelected
               ? `${faction.bgColor} ${faction.borderColor} ${faction.textColor}`
               : 'bg-ui-panel/50 border-gray-600 text-ui-text-dim hover:border-gray-500 hover:text-ui-text'}"
      onclick={() => handleSelect(faction.id)}
      onmouseenter={handleMouseEnter}
    >
      <!-- Faction Emblem -->
      <div class="w-10 h-10 mb-1 flex items-center justify-center">
        <img
          src={faction.emblem}
          alt={faction.name}
          class="w-full h-full object-contain drop-shadow-lg"
          loading="lazy"
        />
      </div>

      <!-- Faction Name -->
      <span class="text-sm font-semibold">{faction.shortName}</span>

      <!-- Selection Indicator -->
      {#if isSelected}
        <div
          class="absolute -bottom-0.5 left-1/2 -translate-x-1/2 w-8 h-1 rounded-full"
          style="background-color: var(--color-{faction.color});"
        ></div>
      {/if}
    </button>
  {/each}
</div>

<style>
  .faction-tab {
    min-width: 120px;
  }

  .faction-tab:hover {
    transform: translateY(-2px);
  }

  .faction-tab:active {
    transform: translateY(0);
  }
</style>
