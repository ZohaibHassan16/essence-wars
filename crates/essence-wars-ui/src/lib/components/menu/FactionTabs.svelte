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
      icon: "gear", // Represents mechanical/construct theme
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
      icon: "leaf", // Represents organic/nature theme
      color: "symbiote-glow",
      bgColor: "bg-symbiote-glow/20",
      borderColor: "border-symbiote-glow",
      textColor: "text-symbiote-glow",
      description: "Aggressive organic swarms",
    },
    {
      id: "obsidion",
      name: "Obsidion Syndicate",
      shortName: "Obsidion",
      icon: "skull", // Represents dark/shadowy theme
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
  {#each factions as faction}
    {@const isSelected = selectedFaction === faction.id}
    <button
      class="faction-tab relative flex flex-col items-center px-6 py-3 rounded-lg border-2 transition-all duration-200
             {isSelected
               ? `${faction.bgColor} ${faction.borderColor} ${faction.textColor}`
               : 'bg-ui-panel/50 border-gray-600 text-ui-text-dim hover:border-gray-500 hover:text-ui-text'}"
      onclick={() => handleSelect(faction.id)}
      onmouseenter={handleMouseEnter}
    >
      <!-- Faction Icon -->
      <div class="w-8 h-8 mb-1 flex items-center justify-center">
        {#if faction.icon === "gear"}
          <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clip-rule="evenodd" />
          </svg>
        {:else if faction.icon === "leaf"}
          <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M4.382 11.018C4.18 9.83 4.43 8.49 5.225 7.314a6.97 6.97 0 012.033-1.945C9.193 4.267 11.585 4.25 14 4.25c0 2.415-.017 4.807-1.119 6.742a6.97 6.97 0 01-1.945 2.033c-1.176.795-2.516 1.045-3.704.843-.237.582-.637 1.28-1.285 1.928l-.636-.636a.75.75 0 10-1.06 1.06l.634.636c-1.016 1.016-2.372 1.494-3.635 1.394a1 1 0 01-.931-.931c-.1-1.263.378-2.62 1.394-3.635l.636.634a.75.75 0 101.06-1.06l-.636-.636c.648-.648 1.346-1.048 1.928-1.285h.001z" clip-rule="evenodd" />
          </svg>
        {:else if faction.icon === "skull"}
          <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 2a8 8 0 00-8 8v1.5a.5.5 0 00.5.5h1a.5.5 0 00.5-.5V10a6 6 0 1112 0v1.5a.5.5 0 00.5.5h1a.5.5 0 00.5-.5V10a8 8 0 00-8-8zM6 12a2 2 0 114 0 2 2 0 01-4 0zm8 0a2 2 0 11-4 0 2 2 0 014 0zM8 16a1 1 0 011-1h2a1 1 0 110 2H9a1 1 0 01-1-1z" clip-rule="evenodd" />
          </svg>
        {/if}
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
