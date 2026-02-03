<script lang="ts">
  import { deckBuilderStore } from "$lib/stores/deckBuilderState.svelte";
  import { playSound } from "$lib/audio";

  let deleteConfirmId = $state<string | null>(null);

  function handleNewDeck() {
    deckBuilderStore.createNewDeck();
    playSound("menuOpen");
  }

  async function handleEditDeck(deckId: string) {
    await deckBuilderStore.loadDeck(deckId);
    playSound("cardSelect");
  }

  function handleDeleteClick(deckId: string) {
    deleteConfirmId = deckId;
    playSound("buttonClick");
  }

  async function confirmDelete(deckId: string) {
    await deckBuilderStore.deleteDeck(deckId);
    deleteConfirmId = null;
    playSound("menuClose");
  }

  function cancelDelete() {
    deleteConfirmId = null;
    playSound("buttonClick");
  }

  function getPlaystyleIcon(playstyle: string): string {
    switch (playstyle.toLowerCase()) {
      case "aggro":
        return "🔥";
      case "control":
        return "🛡️";
      case "tempo":
        return "⚡";
      case "midrange":
        return "⚖️";
      default:
        return "❓";
    }
  }

  /** Capitalize first letter for display */
  function formatPlaystyle(playstyle: string): string {
    return playstyle.charAt(0).toUpperCase() + playstyle.slice(1);
  }

  function getFactionColor(faction: string): string {
    switch (faction?.toLowerCase()) {
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

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  }
</script>

<div class="deck-list-view">
  <!-- Header with New Deck button -->
  <div class="list-header">
    <p class="list-description">
      Create and manage your custom decks. Custom decks can be used in matches against AI opponents.
    </p>
    <button class="new-deck-button" onclick={handleNewDeck}>
      <span class="icon">+</span>
      <span>Create New Deck</span>
    </button>
  </div>

  <!-- Deck Grid -->
  <div class="deck-grid-container">
    {#if deckBuilderStore.customDecks.length === 0}
      <div class="empty-state">
        <div class="empty-icon">📚</div>
        <h3>No Custom Decks Yet</h3>
        <p>Create your first deck to get started!</p>
      </div>
    {:else}
      <div class="deck-grid">
        {#each deckBuilderStore.customDecks as deck (deck.id)}
          <div
            class="deck-card"
            style="--faction-color: {getFactionColor(deck.faction)}"
          >
            {#if deleteConfirmId === deck.id}
              <!-- Delete Confirmation -->
              <div class="delete-confirm">
                <p>Delete "{deck.name}"?</p>
                <div class="confirm-actions">
                  <button
                    class="confirm-delete"
                    onclick={() => confirmDelete(deck.id)}
                  >
                    Delete
                  </button>
                  <button class="cancel-delete" onclick={cancelDelete}>
                    Cancel
                  </button>
                </div>
              </div>
            {:else}
              <!-- Deck Info -->
              <div class="deck-header">
                <h3 class="deck-name">{deck.name}</h3>
                <span class="deck-faction">{deck.faction}</span>
              </div>

              <div class="deck-meta">
                <span class="commander-name">{deck.commanderName}</span>
                <span class="card-count">{deck.cardCount} cards</span>
              </div>

              {#if deck.playstyle}
                <div class="deck-playstyle">
                  <span class="playstyle-icon">{getPlaystyleIcon(deck.playstyle.primary)}</span>
                  <span class="playstyle-name">{formatPlaystyle(deck.playstyle.primary)}</span>
                </div>
              {/if}

              {#if deck.description}
                <p class="deck-description">{deck.description}</p>
              {/if}

              {#if deck.modifiedAt}
                <div class="deck-date">
                  Modified: {formatDate(deck.modifiedAt)}
                </div>
              {/if}

              <div class="deck-actions">
                <button
                  class="edit-button"
                  onclick={() => handleEditDeck(deck.id)}
                  onmouseenter={() => playSound("buttonHover")}
                >
                  Edit
                </button>
                <button
                  class="delete-button"
                  onclick={() => handleDeleteClick(deck.id)}
                  onmouseenter={() => playSound("buttonHover")}
                >
                  Delete
                </button>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .deck-list-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: 1.5rem;
  }

  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .list-description {
    margin: 0;
    color: rgba(255, 255, 255, 0.6);
    flex: 1;
    min-width: 200px;
  }

  .new-deck-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.5rem;
    background: var(--color-ui-action, #e94560);
    border: none;
    border-radius: 0.5rem;
    color: white;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .new-deck-button:hover {
    background: #ff5a75;
    transform: translateY(-1px);
  }

  .new-deck-button .icon {
    font-size: 1.25rem;
    font-weight: 300;
  }

  .deck-grid-container {
    flex: 1;
    overflow-y: auto;
  }

  .deck-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 300px;
    text-align: center;
    color: rgba(255, 255, 255, 0.5);
  }

  .empty-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }

  .empty-state h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
    color: rgba(255, 255, 255, 0.7);
  }

  .empty-state p {
    margin: 0;
  }

  .deck-card {
    background: var(--color-ui-panel, #16213e);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-left: 4px solid var(--faction-color);
    border-radius: 0.75rem;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    transition: all 0.2s;
  }

  .deck-card:hover {
    border-color: rgba(255, 255, 255, 0.2);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .deck-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .deck-name {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-ui-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .deck-faction {
    padding: 0.125rem 0.5rem;
    background: var(--faction-color);
    color: #000;
    border-radius: 0.25rem;
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .deck-meta {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 0.875rem;
  }

  .commander-name {
    color: var(--faction-color);
    font-weight: 500;
  }

  .card-count {
    color: rgba(255, 255, 255, 0.5);
  }

  .deck-playstyle {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.875rem;
  }

  .playstyle-icon {
    font-size: 0.875rem;
  }

  .playstyle-name {
    color: rgba(255, 255, 255, 0.7);
  }

  .deck-description {
    margin: 0;
    font-size: 0.8125rem;
    color: rgba(255, 255, 255, 0.5);
    line-height: 1.4;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    line-clamp: 2;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .deck-date {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.4);
  }

  .deck-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .edit-button,
  .delete-button {
    flex: 1;
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }

  .edit-button {
    background: transparent;
    border: 1px solid var(--faction-color);
    color: var(--faction-color);
  }

  .edit-button:hover {
    background: color-mix(in srgb, var(--faction-color) 15%, transparent);
  }

  .delete-button {
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: rgba(255, 255, 255, 0.6);
  }

  .delete-button:hover {
    border-color: #ef4444;
    color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
  }

  .delete-confirm {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-height: 120px;
    text-align: center;
  }

  .delete-confirm p {
    margin: 0 0 1rem 0;
    color: rgba(255, 255, 255, 0.8);
  }

  .confirm-actions {
    display: flex;
    gap: 0.5rem;
  }

  .confirm-delete {
    padding: 0.5rem 1rem;
    background: #ef4444;
    border: none;
    border-radius: 0.375rem;
    color: white;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s;
  }

  .confirm-delete:hover {
    background: #dc2626;
  }

  .cancel-delete {
    padding: 0.5rem 1rem;
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.375rem;
    color: rgba(255, 255, 255, 0.7);
    cursor: pointer;
    transition: all 0.15s;
  }

  .cancel-delete:hover {
    border-color: rgba(255, 255, 255, 0.4);
    color: white;
  }
</style>
