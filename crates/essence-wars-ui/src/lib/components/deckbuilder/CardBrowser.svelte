<script lang="ts">
  import { deckBuilderStore } from "$lib/stores/deckBuilderState.svelte";
  import CardFilterBar from "./CardFilterBar.svelte";
  import BrowsableCardItem from "./BrowsableCardItem.svelte";
</script>

<div class="card-browser">
  <CardFilterBar />

  <div class="card-grid-container">
    {#if !deckBuilderStore.selectedCommander}
      <div class="no-commander-message">
        <p>Select a commander to see available cards</p>
      </div>
    {:else if deckBuilderStore.filteredCards.length === 0}
      <div class="no-cards-message">
        <p>No cards match your filters</p>
        <button onclick={() => deckBuilderStore.clearFilters()}>
          Clear Filters
        </button>
      </div>
    {:else}
      <div class="card-grid">
        {#each deckBuilderStore.filteredCards as card (card.cardId)}
          <BrowsableCardItem
            {card}
            copiesInDeck={deckBuilderStore.cardCopyCounts.get(card.cardId) ?? 0}
            onAddCard={() => deckBuilderStore.addCard(card.cardId)}
          />
        {/each}
      </div>
    {/if}
  </div>

  <div class="card-count-info">
    Showing {deckBuilderStore.filteredCards.length} cards
    {#if deckBuilderStore.selectedCommander}
      <span class="faction-indicator">
        ({deckBuilderStore.selectedCommander.faction} + neutral)
      </span>
    {/if}
  </div>
</div>

<style>
  .card-browser {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-ui-panel, #16213e);
    border-radius: 0.75rem;
    border: 1px solid rgba(255, 255, 255, 0.1);
    overflow: hidden;
  }

  .card-grid-container {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 0.75rem;
  }

  .no-commander-message,
  .no-cards-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: rgba(255, 255, 255, 0.5);
    text-align: center;
  }

  .no-cards-message button {
    margin-top: 1rem;
    padding: 0.5rem 1rem;
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 0.5rem;
    color: rgba(255, 255, 255, 0.7);
    cursor: pointer;
    transition: all 0.2s;
  }

  .no-cards-message button:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.5);
  }

  .card-count-info {
    padding: 0.75rem 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.6);
  }

  .faction-indicator {
    color: rgba(255, 255, 255, 0.4);
  }
</style>
