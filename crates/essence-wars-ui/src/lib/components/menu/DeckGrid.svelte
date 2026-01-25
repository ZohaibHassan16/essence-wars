<script lang="ts">
  import type { DeckInfo } from "$lib/api/types";
  import DeckCard from "./DeckCard.svelte";

  let {
    decks,
    selectedFaction = "argentum",
    selectedDeckId = null,
    onSelectDeck,
  }: {
    decks: DeckInfo[];
    selectedFaction?: string;
    selectedDeckId?: string | null;
    onSelectDeck?: (deckId: string) => void;
  } = $props();

  // Filter decks by selected faction
  const filteredDecks = $derived(
    decks.filter((deck) => deck.faction === selectedFaction)
  );
</script>

<div class="deck-grid flex flex-wrap justify-center gap-4 p-4">
  {#each filteredDecks as deck (deck.id)}
    <div class="deck-card-wrapper">
      <DeckCard
        {deck}
        isSelected={selectedDeckId === deck.id}
        onSelect={() => onSelectDeck?.(deck.id)}
      />
    </div>
  {:else}
    <div class="text-ui-text-dim text-center py-8">
      No decks available for this faction.
    </div>
  {/each}
</div>

<style>
  .deck-card-wrapper {
    animation: fadeIn 0.3s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* Stagger animation for each card */
  .deck-card-wrapper:nth-child(1) { animation-delay: 0ms; }
  .deck-card-wrapper:nth-child(2) { animation-delay: 50ms; }
  .deck-card-wrapper:nth-child(3) { animation-delay: 100ms; }
  .deck-card-wrapper:nth-child(4) { animation-delay: 150ms; }
  .deck-card-wrapper:nth-child(5) { animation-delay: 200ms; }
  .deck-card-wrapper:nth-child(6) { animation-delay: 250ms; }
</style>
