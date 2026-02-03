<script lang="ts">
  import type { CommanderDto } from "$lib/api/types";

  let { commander, onChangeCommander }: {
    commander: CommanderDto | null;
    onChangeCommander: () => void;
  } = $props();

  function getFactionColor(faction: string): string {
    switch (faction) {
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
</script>

<div class="commander-section" style="--faction-color: {commander ? getFactionColor(commander.faction) : '#888'}">
  {#if commander}
    <div class="commander-display">
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
      <div class="commander-info">
        <h2 class="commander-name">{commander.name}</h2>
        <span class="faction-badge">{commander.faction}</span>
        <p class="ability-description">{commander.abilityDescription}</p>
      </div>
      <button class="change-button" onclick={onChangeCommander}>
        Change Commander
      </button>
    </div>
  {:else}
    <button class="select-commander-button" onclick={onChangeCommander}>
      <span class="icon">+</span>
      <span>Select Commander</span>
    </button>
  {/if}
</div>

<style>
  .commander-section {
    background: var(--color-ui-panel, #16213e);
    border-radius: 0.75rem;
    border: 1px solid rgba(255, 255, 255, 0.1);
    padding: 1rem;
  }

  .commander-display {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .portrait-container {
    width: 80px;
    height: 80px;
    border-radius: 0.5rem;
    overflow: hidden;
    border: 2px solid var(--faction-color);
    box-shadow: 0 0 10px var(--faction-color);
    flex-shrink: 0;
  }

  .portrait {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .commander-info {
    flex: 1;
    min-width: 0;
  }

  .commander-name {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0 0 0.25rem 0;
    color: var(--faction-color);
  }

  .faction-badge {
    display: inline-block;
    padding: 0.125rem 0.5rem;
    background: var(--faction-color);
    color: #000;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    margin-bottom: 0.5rem;
  }

  .ability-description {
    margin: 0;
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.7);
    line-height: 1.4;
  }

  .change-button {
    padding: 0.5rem 1rem;
    background: transparent;
    border: 1px solid var(--faction-color);
    border-radius: 0.5rem;
    color: var(--faction-color);
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .change-button:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .select-commander-button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    width: 100%;
    padding: 1.5rem;
    background: transparent;
    border: 2px dashed rgba(255, 255, 255, 0.3);
    border-radius: 0.5rem;
    color: rgba(255, 255, 255, 0.7);
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .select-commander-button:hover {
    border-color: var(--color-ui-action, #e94560);
    color: var(--color-ui-action);
    background: rgba(233, 69, 96, 0.1);
  }

  .select-commander-button .icon {
    font-size: 1.5rem;
    font-weight: 300;
  }
</style>
