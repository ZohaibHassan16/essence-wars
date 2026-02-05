<script lang="ts">
  import type { CardDto } from "$lib/api/types";
  import { getDeckCards } from "$lib/api/game";

  interface Props {
    show: boolean;
    player1DeckId: string;
    player2DeckId: string;
    player1DeckName: string;
    player2DeckName: string;
    /** If true, show deck order view. If false, only show organized view (for human vs AI to avoid spoilers) */
    allowDeckOrder?: boolean;
    onClose: () => void;
  }

  let {
    show,
    player1DeckId,
    player2DeckId,
    player1DeckName,
    player2DeckName,
    allowDeckOrder = true,
    onClose
  }: Props = $props();

  type ViewMode = "deck_order" | "organized";
  let viewMode = $state<ViewMode>("organized");

  let player1Cards = $state<CardDto[]>([]);
  let player2Cards = $state<CardDto[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  // Initialize view mode based on allowDeckOrder and reset when it changes
  $effect(() => {
    viewMode = allowDeckOrder ? "deck_order" : "organized";
  });

  // Load deck cards when modal opens
  $effect(() => {
    if (show && player1DeckId && player2DeckId) {
      loadDecks();
    }
  });

  async function loadDecks() {
    loading = true;
    error = null;
    try {
      const [p1Cards, p2Cards] = await Promise.all([
        getDeckCards(player1DeckId),
        getDeckCards(player2DeckId),
      ]);
      player1Cards = p1Cards;
      player2Cards = p2Cards;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load deck cards";
    } finally {
      loading = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    }
  }

  // Organize cards by type and cost
  function organizeCards(cards: CardDto[]): { type: string; cards: CardDto[] }[] {
    const groups: Record<string, CardDto[]> = {
      creatures: [],
      supports: [],
      spells: [],
    };

    for (const card of cards) {
      const type = card.cardType.toLowerCase();
      if (type === "creature") {
        groups.creatures.push(card);
      } else if (type === "support") {
        groups.supports.push(card);
      } else {
        groups.spells.push(card);
      }
    }

    // Sort each group by cost, then by name
    for (const key of Object.keys(groups)) {
      groups[key].sort((a, b) => {
        if (a.cost !== b.cost) return a.cost - b.cost;
        return a.name.localeCompare(b.name);
      });
    }

    const result: { type: string; cards: CardDto[] }[] = [];
    if (groups.creatures.length > 0) result.push({ type: "Creatures", cards: groups.creatures });
    if (groups.supports.length > 0) result.push({ type: "Supports", cards: groups.supports });
    if (groups.spells.length > 0) result.push({ type: "Spells", cards: groups.spells });

    return result;
  }

  function getCardTypeIcon(cardType: string): string {
    switch (cardType.toLowerCase()) {
      case "creature": return "";
      case "support": return "";
      case "spell": return "";
      default: return "";
    }
  }

  function getFactionColor(faction: string): string {
    switch (faction.toLowerCase()) {
      case "argentum": return "border-argentum-accent/50";
      case "symbiote": return "border-symbiote-accent/50";
      case "obsidion": return "border-obsidion-accent/50";
      default: return "border-gray-600";
    }
  }

  function getKeywordBadges(keywords: string[]): string[] {
    return keywords.slice(0, 3); // Limit to 3 keywords for space
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if show}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black/80 z-50 flex items-center justify-center p-4"
    onclick={onClose}
    onkeydown={(e) => e.key === "Enter" && onClose()}
    role="button"
    tabindex="0"
  >
    <!-- Modal -->
    <div
      class="bg-ui-panel border border-gray-700 rounded-lg shadow-2xl w-full max-w-6xl max-h-[85vh] flex flex-col"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="deck-library-title"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-gray-700">
        <div class="flex items-center gap-3">
          <span class="text-lg">📚</span>
          <h2 id="deck-library-title" class="text-lg font-semibold text-ui-text">Deck Library</h2>
          <span class="text-sm text-ui-text-dim">
            {player1Cards.length + player2Cards.length} cards total
          </span>
        </div>
        <div class="flex items-center gap-3">
          <!-- View toggle -->
          {#if allowDeckOrder}
            <div class="flex items-center bg-ui-bg rounded-lg p-0.5">
              <button
                class="px-3 py-1.5 text-sm rounded-md transition-colors {viewMode === 'deck_order'
                  ? 'bg-purple-600 text-white'
                  : 'text-ui-text-dim hover:text-ui-text'}"
                onclick={() => (viewMode = "deck_order")}
              >
                Draw Order
              </button>
              <button
                class="px-3 py-1.5 text-sm rounded-md transition-colors {viewMode === 'organized'
                  ? 'bg-purple-600 text-white'
                  : 'text-ui-text-dim hover:text-ui-text'}"
                onclick={() => (viewMode = "organized")}
              >
                By Type
              </button>
            </div>
          {/if}
          <button
            class="text-ui-text-dim hover:text-ui-text transition-colors p-1"
            onclick={onClose}
            aria-label="Close"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-hidden p-4">
        {#if loading}
          <div class="flex items-center justify-center h-full">
            <div class="flex flex-col items-center gap-3">
              <div class="w-8 h-8 border-3 border-purple-500/30 border-t-purple-500 rounded-full animate-spin"></div>
              <span class="text-ui-text-dim">Loading deck cards...</span>
            </div>
          </div>
        {:else if error}
          <div class="flex items-center justify-center h-full">
            <div class="text-center text-damage">
              <div class="text-4xl mb-2">⚠️</div>
              <div>{error}</div>
            </div>
          </div>
        {:else}
          <!-- Dual column layout -->
          <div class="grid grid-cols-2 gap-4 h-full">
            <!-- Player 1 Deck -->
            <div class="flex flex-col h-full overflow-hidden">
              <div class="flex items-center gap-2 mb-3">
                <span class="px-2 py-0.5 rounded text-xs font-medium bg-health/20 text-health">P1</span>
                <span class="text-sm font-semibold text-ui-text">{player1DeckName}</span>
                <span class="text-xs text-ui-text-dim">({player1Cards.length} cards)</span>
              </div>
              <div class="flex-1 overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-transparent">
                {#if viewMode === "deck_order"}
                  <div class="space-y-1">
                    {#each player1Cards as card, i (i)}
                      <div class="flex items-center gap-2 p-2 rounded-lg bg-ui-bg/50 border {getFactionColor(card.faction)} hover:bg-ui-bg/80 transition-colors">
                        <span class="text-xs text-ui-text-dim w-6 text-right">#{i + 1}</span>
                        <span class="w-5 h-5 flex items-center justify-center rounded bg-gray-700 text-xs font-bold">{card.cost}</span>
                        <span class="flex-1 text-sm text-ui-text truncate">{card.name}</span>
                        {#if card.attack !== undefined && card.health !== undefined}
                          <span class="text-xs text-ui-text-dim">{card.attack}/{card.health}</span>
                        {/if}
                        {#each getKeywordBadges(card.keywords) as keyword}
                          <span class="px-1 py-0.5 text-[10px] bg-gray-700 text-ui-text-dim rounded">{keyword}</span>
                        {/each}
                      </div>
                    {/each}
                  </div>
                {:else}
                  {#each organizeCards(player1Cards) as group}
                    <div class="mb-4">
                      <div class="text-xs font-semibold text-ui-text-dim uppercase tracking-wide mb-2 flex items-center gap-2">
                        {group.type} ({group.cards.length})
                      </div>
                      <div class="space-y-1">
                        {#each group.cards as card (card.cardId)}
                          <div class="flex items-center gap-2 p-2 rounded-lg bg-ui-bg/50 border {getFactionColor(card.faction)} hover:bg-ui-bg/80 transition-colors">
                            <span class="w-5 h-5 flex items-center justify-center rounded bg-gray-700 text-xs font-bold">{card.cost}</span>
                            <span class="flex-1 text-sm text-ui-text truncate">{card.name}</span>
                            {#if card.attack !== undefined && card.health !== undefined}
                              <span class="text-xs text-ui-text-dim">{card.attack}/{card.health}</span>
                            {/if}
                            {#each getKeywordBadges(card.keywords) as keyword}
                              <span class="px-1 py-0.5 text-[10px] bg-gray-700 text-ui-text-dim rounded">{keyword}</span>
                            {/each}
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/each}
                {/if}
              </div>
            </div>

            <!-- Divider -->
            <div class="absolute left-1/2 top-0 bottom-0 w-px bg-gray-700/50 -translate-x-1/2 hidden"></div>

            <!-- Player 2 Deck -->
            <div class="flex flex-col h-full overflow-hidden">
              <div class="flex items-center gap-2 mb-3">
                <span class="px-2 py-0.5 rounded text-xs font-medium bg-damage/20 text-damage">P2</span>
                <span class="text-sm font-semibold text-ui-text">{player2DeckName}</span>
                <span class="text-xs text-ui-text-dim">({player2Cards.length} cards)</span>
              </div>
              <div class="flex-1 overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-transparent">
                {#if viewMode === "deck_order"}
                  <div class="space-y-1">
                    {#each player2Cards as card, i (i)}
                      <div class="flex items-center gap-2 p-2 rounded-lg bg-ui-bg/50 border {getFactionColor(card.faction)} hover:bg-ui-bg/80 transition-colors">
                        <span class="text-xs text-ui-text-dim w-6 text-right">#{i + 1}</span>
                        <span class="w-5 h-5 flex items-center justify-center rounded bg-gray-700 text-xs font-bold">{card.cost}</span>
                        <span class="flex-1 text-sm text-ui-text truncate">{card.name}</span>
                        {#if card.attack !== undefined && card.health !== undefined}
                          <span class="text-xs text-ui-text-dim">{card.attack}/{card.health}</span>
                        {/if}
                        {#each getKeywordBadges(card.keywords) as keyword}
                          <span class="px-1 py-0.5 text-[10px] bg-gray-700 text-ui-text-dim rounded">{keyword}</span>
                        {/each}
                      </div>
                    {/each}
                  </div>
                {:else}
                  {#each organizeCards(player2Cards) as group}
                    <div class="mb-4">
                      <div class="text-xs font-semibold text-ui-text-dim uppercase tracking-wide mb-2 flex items-center gap-2">
                        {group.type} ({group.cards.length})
                      </div>
                      <div class="space-y-1">
                        {#each group.cards as card (card.cardId)}
                          <div class="flex items-center gap-2 p-2 rounded-lg bg-ui-bg/50 border {getFactionColor(card.faction)} hover:bg-ui-bg/80 transition-colors">
                            <span class="w-5 h-5 flex items-center justify-center rounded bg-gray-700 text-xs font-bold">{card.cost}</span>
                            <span class="flex-1 text-sm text-ui-text truncate">{card.name}</span>
                            {#if card.attack !== undefined && card.health !== undefined}
                              <span class="text-xs text-ui-text-dim">{card.attack}/{card.health}</span>
                            {/if}
                            {#each getKeywordBadges(card.keywords) as keyword}
                              <span class="px-1 py-0.5 text-[10px] bg-gray-700 text-ui-text-dim rounded">{keyword}</span>
                            {/each}
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/each}
                {/if}
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-4 py-3 border-t border-gray-700 flex items-center justify-between">
        <div class="text-xs text-ui-text-dim">
          {#if viewMode === "deck_order"}
            Cards shown in draw order (top = next draw)
          {:else}
            Cards organized by type and cost
          {/if}
        </div>
        <div class="flex items-center gap-3">
          <span class="text-xs text-ui-text-dim">Press <kbd class="px-1.5 py-0.5 bg-gray-700 rounded text-ui-text">D</kbd> or <kbd class="px-1.5 py-0.5 bg-gray-700 rounded text-ui-text">Esc</kbd> to close</span>
          <button
            class="px-4 py-2 bg-gray-700 text-ui-text rounded-lg hover:bg-gray-600 transition-colors"
            onclick={onClose}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
