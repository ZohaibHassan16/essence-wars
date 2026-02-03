<script lang="ts">
  import { deckBuilderStore } from "$lib/stores/deckBuilderState.svelte";
  import { playSound } from "$lib/audio";
  import CommanderSection from "./CommanderSection.svelte";
  import CardBrowser from "./CardBrowser.svelte";
  import DeckPanel from "./DeckPanel.svelte";
  import CommanderPickerModal from "./CommanderPickerModal.svelte";
  import SaveDeckModal from "./SaveDeckModal.svelte";
  import DeckListView from "./DeckListView.svelte";

  let showCommanderPicker = $state(false);
  let showSaveModal = $state(false);
  let saveButtonState = $state<"idle" | "saved">("idle");

  // Whether this is a new deck (not yet saved)
  const isNewDeck = $derived(!deckBuilderStore.currentDeckId);

  function handleBack() {
    if (deckBuilderStore.phase === "building") {
      deckBuilderStore.backToList();
    } else {
      deckBuilderStore.close();
    }
  }

  function handleSave() {
    if (isNewDeck) {
      // New deck: show save modal to get name/description
      showSaveModal = true;
      playSound("menuOpen");
    } else {
      // Existing deck: save directly with feedback
      saveExistingDeck();
    }
  }

  async function saveExistingDeck() {
    await deckBuilderStore.saveDeck();
    if (!deckBuilderStore.error) {
      showSavedFeedback();
    }
  }

  function showSavedFeedback() {
    saveButtonState = "saved";
    playSound("cardSelect");
    setTimeout(() => {
      saveButtonState = "idle";
    }, 2000);
  }

  function closeSaveModal() {
    showSaveModal = false;
    playSound("menuClose");
  }

  function handleSaveModalSaved() {
    showSavedFeedback();
  }

  function openCommanderPicker() {
    showCommanderPicker = true;
    playSound("menuOpen");
  }

  function closeCommanderPicker() {
    showCommanderPicker = false;
    playSound("menuClose");
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      if (showSaveModal) {
        closeSaveModal();
      } else if (showCommanderPicker) {
        closeCommanderPicker();
      } else {
        handleBack();
      }
    } else if (event.key === "s" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      handleSave();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="deck-builder-screen">
  <!-- Header -->
  <header class="deck-builder-header">
    <button class="back-button" onclick={handleBack}>
      <span class="icon">←</span>
      <span>Back</span>
    </button>

    <h1 class="title">
      {#if deckBuilderStore.phase === "list"}
        Custom Decks
      {:else}
        Deck Builder
      {/if}
    </h1>

    <div class="header-actions">
      {#if deckBuilderStore.phase === "building"}
        <button
          class="save-button"
          class:saved={saveButtonState === "saved"}
          onclick={handleSave}
          disabled={deckBuilderStore.isLoading || !deckBuilderStore.selectedCommander || saveButtonState === "saved"}
        >
          {#if deckBuilderStore.isLoading}
            Saving...
          {:else if saveButtonState === "saved"}
            ✓ Saved!
          {:else}
            Save Deck
          {/if}
        </button>
      {/if}
    </div>
  </header>

  <!-- Error display -->
  {#if deckBuilderStore.error}
    <div class="error-banner">
      <span>{deckBuilderStore.error}</span>
      <button onclick={() => (deckBuilderStore.error = null)}>×</button>
    </div>
  {/if}

  <!-- Main content -->
  <main class="deck-builder-content">
    {#if deckBuilderStore.phase === "list"}
      <DeckListView />
    {:else if deckBuilderStore.phase === "building"}
      <!-- Commander Section -->
      <CommanderSection
        commander={deckBuilderStore.selectedCommander}
        onChangeCommander={openCommanderPicker}
      />

      <!-- Main Builder Area -->
      <div class="builder-main">
        <!-- Card Browser (left) -->
        <div class="card-browser-section">
          <CardBrowser />
        </div>

        <!-- Deck Panel (right) -->
        <div class="deck-panel-section">
          <DeckPanel />
        </div>
      </div>
    {/if}
  </main>

  <!-- Commander Picker Modal -->
  {#if showCommanderPicker}
    <CommanderPickerModal onClose={closeCommanderPicker} />
  {/if}

  <!-- Save Deck Modal -->
  {#if showSaveModal}
    <SaveDeckModal onClose={closeSaveModal} onSaved={handleSaveModalSaved} />
  {/if}
</div>

<style>
  .deck-builder-screen {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--color-ui-bg, #1a1a2e);
    color: var(--color-ui-text, #f6f6f6);
  }

  .deck-builder-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    background: var(--color-ui-panel, #16213e);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .back-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.5rem;
    color: var(--color-ui-text);
    cursor: pointer;
    transition: all 0.2s;
  }

  .back-button:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .title {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 0;
  }

  .header-actions {
    display: flex;
    gap: 0.5rem;
  }

  .save-button {
    padding: 0.5rem 1.5rem;
    background: var(--color-ui-action, #e94560);
    border: none;
    border-radius: 0.5rem;
    color: white;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .save-button:hover:not(:disabled) {
    background: #ff5a75;
    transform: translateY(-1px);
  }

  .save-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .save-button.saved {
    background: #22c55e;
    opacity: 1;
  }

  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1.5rem;
    background: rgba(239, 68, 68, 0.2);
    border-bottom: 1px solid rgba(239, 68, 68, 0.3);
    color: #fca5a5;
  }

  .error-banner button {
    background: transparent;
    border: none;
    color: #fca5a5;
    font-size: 1.25rem;
    cursor: pointer;
    padding: 0 0.5rem;
  }

  .deck-builder-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 1rem;
    gap: 1rem;
  }

  .builder-main {
    flex: 1;
    display: flex;
    gap: 1rem;
    overflow: hidden;
  }

  .card-browser-section {
    flex: 3;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .deck-panel-section {
    flex: 2;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
</style>
