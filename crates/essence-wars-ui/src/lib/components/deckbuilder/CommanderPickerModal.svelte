<script lang="ts">
  import { deckBuilderStore } from "$lib/stores/deckBuilderState.svelte";
  import { playSound } from "$lib/audio";
  import type { CommanderDto } from "$lib/api/types";

  let { onClose }: { onClose: () => void } = $props();

  let selectedFaction = $state<string>("argentum");
  let hoveredCommander = $state<CommanderDto | null>(null);

  const factions = ["argentum", "symbiote", "obsidion"];

  const filteredCommanders = $derived(
    deckBuilderStore.commanders.filter((c) => c.faction.toLowerCase() === selectedFaction)
  );

  function selectCommander(commander: CommanderDto) {
    deckBuilderStore.selectCommander(commander);
    playSound("cardSelect");
    onClose();
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    }
  }

  function getFactionColor(faction: string): string {
    switch (faction.toLowerCase()) {
      case "argentum":
        return "var(--color-argentum-gold, #D4AF37)";
      case "symbiote":
        return "var(--color-symbiote-glow, #7FFF00)";
      case "obsidion":
        return "var(--color-obsidion-essence, #00FFFF)";
      default:
        return "var(--color-neutral-copper, #B87333)";
    }
  }

  function getFactionBgColor(faction: string): string {
    switch (faction.toLowerCase()) {
      case "argentum":
        return "rgba(212, 175, 55, 0.1)";
      case "symbiote":
        return "rgba(127, 255, 0, 0.1)";
      case "obsidion":
        return "rgba(0, 255, 255, 0.1)";
      default:
        return "rgba(184, 115, 51, 0.1)";
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="modal-backdrop"
  onclick={handleBackdropClick}
  role="dialog"
  aria-modal="true"
  aria-labelledby="commander-picker-title"
>
  <div class="modal-content" style="--faction-color: {getFactionColor(selectedFaction)}">
    <header class="modal-header">
      <h2 id="commander-picker-title">Choose Your Commander</h2>
      <button class="close-button" onclick={onClose} aria-label="Close">
        ×
      </button>
    </header>

    <!-- Faction Tabs -->
    <nav class="faction-tabs">
      {#each factions as faction}
        <button
          class="faction-tab"
          class:active={selectedFaction === faction}
          onclick={() => {
            selectedFaction = faction;
            playSound("buttonClick");
          }}
          onmouseenter={() => playSound("buttonHover")}
          style="--tab-color: {getFactionColor(faction)}"
        >
          {faction.charAt(0).toUpperCase() + faction.slice(1)}
        </button>
      {/each}
    </nav>

    <div class="modal-body">
      <!-- Commander Grid -->
      <div class="commander-grid">
        {#each filteredCommanders as commander (commander.id)}
          <button
            class="commander-card"
            class:selected={deckBuilderStore.selectedCommander?.id === commander.id}
            onclick={() => selectCommander(commander)}
            onmouseenter={() => {
              hoveredCommander = commander;
              playSound("cardHover");
            }}
            onmouseleave={() => (hoveredCommander = null)}
            style="--faction-color: {getFactionColor(commander.faction)}"
          >
            <div class="portrait-container">
              <img
                src={commander.portraitPath}
                alt={commander.name}
                class="portrait"
                onerror={(e) => {
                  (e.target as HTMLImageElement).src = "portrait/placeholder.webp";
                }}
              />
            </div>
            <div class="commander-name">{commander.name}</div>
          </button>
        {/each}
      </div>

      <!-- Commander Preview -->
      <div
        class="commander-preview"
        style="--faction-color: {hoveredCommander ? getFactionColor(hoveredCommander.faction) : getFactionColor(selectedFaction)}; --faction-bg: {hoveredCommander ? getFactionBgColor(hoveredCommander.faction) : getFactionBgColor(selectedFaction)}"
      >
        {#if hoveredCommander}
          <div class="preview-portrait">
            <img
              src={hoveredCommander.portraitPath}
              alt={hoveredCommander.name}
              onerror={(e) => {
                (e.target as HTMLImageElement).src = "portrait/placeholder.webp";
              }}
            />
          </div>
          <div class="preview-info">
            <h3 class="preview-name">{hoveredCommander.name}</h3>
            <span class="preview-faction">{hoveredCommander.faction}</span>
            <p class="preview-ability">{hoveredCommander.abilityDescription}</p>
          </div>
        {:else}
          <div class="preview-placeholder">
            <p>Hover over a commander to see details</p>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 2rem;
  }

  .modal-content {
    background: var(--color-ui-bg, #1a1a2e);
    border-radius: 1rem;
    border: 1px solid rgba(255, 255, 255, 0.1);
    max-width: 900px;
    width: 100%;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .close-button {
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.6);
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    line-height: 1;
    transition: color 0.2s;
  }

  .close-button:hover {
    color: white;
  }

  .faction-tabs {
    display: flex;
    gap: 0.5rem;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .faction-tab {
    flex: 1;
    padding: 0.75rem 1rem;
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.5rem;
    color: rgba(255, 255, 255, 0.7);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .faction-tab:hover {
    border-color: var(--tab-color);
    color: var(--tab-color);
  }

  .faction-tab.active {
    background: var(--tab-color);
    border-color: var(--tab-color);
    color: #000;
  }

  .modal-body {
    display: flex;
    flex: 1;
    overflow: hidden;
    padding: 1.5rem;
    gap: 1.5rem;
  }

  .commander-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
    flex: 1;
    overflow-y: auto;
    align-content: start;
  }

  .commander-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1rem;
    background: rgba(0, 0, 0, 0.3);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.75rem;
    cursor: pointer;
    transition: all 0.2s;
    text-align: center;
  }

  .commander-card:hover {
    border-color: var(--faction-color);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }

  .commander-card.selected {
    border-color: var(--faction-color);
    background: rgba(255, 255, 255, 0.05);
    box-shadow: 0 0 20px color-mix(in srgb, var(--faction-color) 30%, transparent);
  }

  .portrait-container {
    width: 80px;
    height: 80px;
    border-radius: 0.5rem;
    overflow: hidden;
    border: 2px solid var(--faction-color);
    margin-bottom: 0.75rem;
  }

  .portrait {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .commander-name {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-ui-text);
  }

  .commander-preview {
    width: 280px;
    flex-shrink: 0;
    background: var(--faction-bg);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.75rem;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .preview-portrait {
    width: 120px;
    height: 120px;
    border-radius: 0.75rem;
    overflow: hidden;
    border: 3px solid var(--faction-color);
    box-shadow: 0 0 20px color-mix(in srgb, var(--faction-color) 40%, transparent);
    margin-bottom: 1rem;
  }

  .preview-portrait img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .preview-info {
    text-align: center;
    width: 100%;
  }

  .preview-name {
    margin: 0 0 0.5rem 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--faction-color);
  }

  .preview-faction {
    display: inline-block;
    padding: 0.125rem 0.5rem;
    background: var(--faction-color);
    color: #000;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    margin-bottom: 1rem;
  }

  .preview-ability {
    margin: 0 0 1rem 0;
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.8);
    line-height: 1.5;
  }


  .preview-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: rgba(255, 255, 255, 0.4);
    text-align: center;
  }
</style>
