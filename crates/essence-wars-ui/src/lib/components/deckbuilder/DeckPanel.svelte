<script lang="ts">
  import { deckBuilderStore } from "$lib/stores/deckBuilderState.svelte";
  import { playSound } from "$lib/audio";
  import DeckCardList from "./DeckCardList.svelte";

  function getPlaystyleIcon(playstyle: string): string {
    switch (playstyle) {
      case "Aggro":
        return "🔥";
      case "Control":
        return "🛡️";
      case "Tempo":
        return "⚡";
      case "Midrange":
        return "⚖️";
      default:
        return "❓";
    }
  }

  function getPlaystyleColor(playstyle: string): string {
    switch (playstyle) {
      case "Aggro":
        return "#ef4444";
      case "Control":
        return "#3b82f6";
      case "Tempo":
        return "#f59e0b";
      case "Midrange":
        return "#22c55e";
      default:
        return "#9ca3af";
    }
  }

  // Get max value for mana curve scaling
  const maxCurveValue = $derived(
    Math.max(...Object.values(deckBuilderStore.manaCurve), 1)
  );
</script>

<div class="deck-panel">
  <!-- Deck Name Input -->
  <div class="deck-name-section">
    <input
      type="text"
      class="deck-name-input"
      placeholder="Deck Name"
      value={deckBuilderStore.deckName}
      oninput={(e) => {
        deckBuilderStore.deckName = (e.target as HTMLInputElement).value;
      }}
    />
  </div>

  <!-- Mana Curve -->
  <div class="mana-curve-section">
    <h3 class="section-title">Mana Curve</h3>
    <div class="mana-curve">
      {#each Object.entries(deckBuilderStore.manaCurve) as [cost, count]}
        <div class="curve-bar-container">
          <div class="curve-count">{count}</div>
          <div
            class="curve-bar"
            style="height: {(count / maxCurveValue) * 100}%"
          ></div>
          <div class="curve-label">{cost === "7" ? "7+" : cost}</div>
        </div>
      {/each}
    </div>
  </div>

  <!-- Card Type Breakdown -->
  <div class="type-breakdown-section">
    <h3 class="section-title">Card Types</h3>
    <div class="type-breakdown">
      <div class="type-row">
        <span class="type-icon">⚔</span>
        <span class="type-name">Creatures</span>
        <span class="type-count">{deckBuilderStore.cardTypeBreakdown.creature}</span>
      </div>
      <div class="type-row">
        <span class="type-icon">✦</span>
        <span class="type-name">Spells</span>
        <span class="type-count">{deckBuilderStore.cardTypeBreakdown.spell}</span>
      </div>
      <div class="type-row">
        <span class="type-icon">◈</span>
        <span class="type-name">Supports</span>
        <span class="type-count">{deckBuilderStore.cardTypeBreakdown.support}</span>
      </div>
    </div>
  </div>

  <!-- Deck Card List -->
  <div class="deck-list-section">
    <h3 class="section-title">
      Cards ({deckBuilderStore.deckCards.length}/100)
    </h3>
    <DeckCardList />
  </div>

  <!-- Playstyle Indicator -->
  {#if deckBuilderStore.playstyle}
    <div
      class="playstyle-section"
      style="--playstyle-color: {getPlaystyleColor(deckBuilderStore.playstyle.primary)}"
    >
      <span class="playstyle-icon">{getPlaystyleIcon(deckBuilderStore.playstyle.primary)}</span>
      <span class="playstyle-name">{deckBuilderStore.playstyle.primary}</span>
    </div>
  {/if}

  <!-- Validation Status -->
  <div class="validation-section">
    {#if deckBuilderStore.validation}
      {#if deckBuilderStore.validation.isValid}
        <div class="validation-status valid">
          <span class="status-icon">✓</span>
          <span>Deck is valid</span>
        </div>
      {:else}
        <div class="validation-status invalid">
          <span class="status-icon">✗</span>
          <span>Deck has issues</span>
        </div>
        <ul class="validation-errors">
          {#each deckBuilderStore.validation.errors as error}
            <li class="error">{error}</li>
          {/each}
        </ul>
      {/if}
      {#if deckBuilderStore.validation.warnings.length > 0}
        <ul class="validation-warnings">
          {#each deckBuilderStore.validation.warnings as warning}
            <li class="warning">{warning}</li>
          {/each}
        </ul>
      {/if}
    {:else if !deckBuilderStore.selectedCommander}
      <div class="validation-status pending">
        <span>Select a commander to start building</span>
      </div>
    {:else if deckBuilderStore.deckCards.length < 29}
      <div class="validation-status pending">
        <span>Add {29 - deckBuilderStore.deckCards.length} more cards (minimum 29)</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .deck-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-ui-panel, #16213e);
    border-radius: 0.75rem;
    border: 1px solid rgba(255, 255, 255, 0.1);
    overflow: hidden;
  }

  .deck-name-section {
    padding: 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .deck-name-input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.5rem;
    color: var(--color-ui-text);
    font-size: 1rem;
    font-weight: 600;
  }

  .deck-name-input::placeholder {
    color: rgba(255, 255, 255, 0.4);
  }

  .deck-name-input:focus {
    outline: none;
    border-color: var(--color-ui-action, #e94560);
  }

  .section-title {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    color: rgba(255, 255, 255, 0.5);
    margin: 0 0 0.5rem 0;
  }

  .mana-curve-section {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .mana-curve {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    height: 80px;
    gap: 0.25rem;
  }

  .curve-bar-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100%;
  }

  .curve-count {
    font-size: 0.625rem;
    color: rgba(255, 255, 255, 0.7);
    margin-bottom: 0.25rem;
  }

  .curve-bar {
    width: 100%;
    background: var(--color-mana, #3b82f6);
    border-radius: 0.25rem 0.25rem 0 0;
    min-height: 2px;
    transition: height 0.2s ease;
  }

  .curve-label {
    font-size: 0.625rem;
    color: rgba(255, 255, 255, 0.5);
    margin-top: 0.25rem;
  }

  .type-breakdown-section {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .type-breakdown {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .type-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .type-icon {
    width: 1.25rem;
    text-align: center;
  }

  .type-name {
    flex: 1;
    color: rgba(255, 255, 255, 0.8);
  }

  .type-count {
    font-weight: 600;
    color: var(--color-ui-text);
  }

  .deck-list-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0.75rem 1rem;
  }

  .deck-list-section h3 {
    flex-shrink: 0;
  }

  .playstyle-section {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: rgba(0, 0, 0, 0.2);
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  .playstyle-icon {
    font-size: 1.25rem;
  }

  .playstyle-name {
    font-weight: 600;
    color: var(--playstyle-color);
  }

  .validation-section {
    padding: 0.75rem 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  .validation-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .validation-status.valid {
    color: #22c55e;
  }

  .validation-status.invalid {
    color: #ef4444;
  }

  .validation-status.pending {
    color: rgba(255, 255, 255, 0.5);
  }

  .status-icon {
    font-weight: 700;
  }

  .validation-errors,
  .validation-warnings {
    margin: 0.5rem 0 0 0;
    padding: 0 0 0 1.25rem;
    font-size: 0.75rem;
  }

  .validation-errors .error {
    color: #fca5a5;
  }

  .validation-warnings .warning {
    color: #fcd34d;
  }
</style>
