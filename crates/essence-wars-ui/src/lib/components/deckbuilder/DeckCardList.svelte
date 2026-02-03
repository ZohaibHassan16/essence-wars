<script lang="ts">
  import { deckBuilderStore } from "$lib/stores/deckBuilderState.svelte";
  import { playSound } from "$lib/audio";

  // Group cards by ID and get card info
  const groupedCards = $derived(() => {
    const groups = new Map<number, { card: typeof deckBuilderStore.allCards[0]; count: number }>();

    for (const cardId of deckBuilderStore.deckCards) {
      const card = deckBuilderStore.allCards.find(c => c.cardId === cardId);
      if (card) {
        const existing = groups.get(cardId);
        if (existing) {
          existing.count++;
        } else {
          groups.set(cardId, { card, count: 1 });
        }
      }
    }

    // Sort by cost, then by name
    return Array.from(groups.values()).sort((a, b) => {
      if (a.card.cost !== b.card.cost) {
        return a.card.cost - b.card.cost;
      }
      return a.card.name.localeCompare(b.card.name);
    });
  });

  function handleRemoveCard(cardId: number) {
    deckBuilderStore.removeCard(cardId);
    playSound("menuClose");
  }

  function getRarityColor(rarity: string): string {
    switch (rarity) {
      case "Common":
        return "#9ca3af";
      case "Uncommon":
        return "#22c55e";
      case "Rare":
        return "#3b82f6";
      case "Legendary":
        return "#f59e0b";
      default:
        return "#9ca3af";
    }
  }
</script>

<div class="deck-card-list">
  {#if groupedCards().length === 0}
    <div class="empty-state">
      <p>No cards in deck</p>
      <p class="hint">Click cards in the browser to add them</p>
    </div>
  {:else}
    {#each groupedCards() as { card, count } (card.cardId)}
      <button
        class="deck-card-row"
        onclick={() => handleRemoveCard(card.cardId)}
        onmouseenter={() => playSound("buttonHover")}
        style="--rarity-color: {getRarityColor(card.rarity)}"
      >
        <span class="card-cost">{card.cost}</span>
        <span class="card-name">{card.name}</span>
        <span class="card-count">×{count}</span>
      </button>
    {/each}
  {/if}
</div>

<style>
  .deck-card-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: rgba(255, 255, 255, 0.4);
    text-align: center;
  }

  .empty-state p {
    margin: 0;
  }

  .empty-state .hint {
    font-size: 0.75rem;
    margin-top: 0.5rem;
  }

  .deck-card-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.5rem;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-left: 3px solid var(--rarity-color);
    border-radius: 0.25rem;
    cursor: pointer;
    transition: all 0.15s;
    text-align: left;
    width: 100%;
    color: var(--color-ui-text);
  }

  .deck-card-row:hover {
    background: rgba(239, 68, 68, 0.2);
    border-color: rgba(239, 68, 68, 0.3);
  }

  .card-cost {
    width: 1.25rem;
    height: 1.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-mana, #3b82f6);
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 700;
    color: white;
    flex-shrink: 0;
  }

  .card-name {
    flex: 1;
    font-size: 0.8125rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-count {
    font-size: 0.75rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.6);
    flex-shrink: 0;
  }
</style>
