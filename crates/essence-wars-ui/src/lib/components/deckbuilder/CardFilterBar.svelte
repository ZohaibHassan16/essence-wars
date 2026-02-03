<script lang="ts">
  import { deckBuilderStore } from "$lib/stores/deckBuilderState.svelte";

  const costOptions = [
    { value: null, label: "All Costs" },
    { value: 0, label: "0" },
    { value: 1, label: "1" },
    { value: 2, label: "2" },
    { value: 3, label: "3" },
    { value: 4, label: "4" },
    { value: 5, label: "5" },
    { value: 6, label: "6" },
    { value: 7, label: "7+" },
  ];

  const typeOptions = [
    { value: null, label: "All Types" },
    { value: "creature", label: "Creature" },
    { value: "spell", label: "Spell" },
    { value: "support", label: "Support" },
  ];

  const rarityOptions = [
    { value: null, label: "All Rarities" },
    { value: "Common", label: "Common" },
    { value: "Uncommon", label: "Uncommon" },
    { value: "Rare", label: "Rare" },
    { value: "Legendary", label: "Legendary" },
  ];

  function handleSearchInput(event: Event) {
    const target = event.target as HTMLInputElement;
    deckBuilderStore.setSearchFilter(target.value);
  }

  function handleCostChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const value = target.value === "" ? null : parseInt(target.value);
    deckBuilderStore.setCostFilter(value);
  }

  function handleTypeChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const value = target.value === "" ? null : target.value;
    deckBuilderStore.setCardTypeFilter(value);
  }

  function handleKeywordChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const value = target.value === "" ? null : target.value;
    deckBuilderStore.setKeywordFilter(value);
  }

  function handleRarityChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const value = target.value === "" ? null : target.value;
    deckBuilderStore.setRarityFilter(value);
  }
</script>

<div class="filter-bar">
  <div class="search-container">
    <input
      type="text"
      placeholder="Search cards..."
      value={deckBuilderStore.filters.search}
      oninput={handleSearchInput}
      class="search-input"
    />
  </div>

  <div class="filter-group">
    <select
      value={deckBuilderStore.filters.cost ?? ""}
      onchange={handleCostChange}
      class="filter-select"
    >
      {#each costOptions as option (option.value ?? option.label)}
        <option value={option.value ?? ""}>{option.label}</option>
      {/each}
    </select>

    <select
      value={deckBuilderStore.filters.cardType ?? ""}
      onchange={handleTypeChange}
      class="filter-select"
    >
      {#each typeOptions as option (option.value ?? option.label)}
        <option value={option.value ?? ""}>{option.label}</option>
      {/each}
    </select>

    <select
      value={deckBuilderStore.filters.keyword ?? ""}
      onchange={handleKeywordChange}
      class="filter-select"
    >
      <option value="">All Keywords</option>
      {#each deckBuilderStore.availableKeywords as keyword (keyword)}
        <option value={keyword}>{keyword}</option>
      {/each}
    </select>

    <select
      value={deckBuilderStore.filters.rarity ?? ""}
      onchange={handleRarityChange}
      class="filter-select"
    >
      {#each rarityOptions as option (option.value ?? option.label)}
        <option value={option.value ?? ""}>{option.label}</option>
      {/each}
    </select>
  </div>
</div>

<style>
  .filter-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    padding: 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(0, 0, 0, 0.2);
  }

  .search-container {
    flex: 1;
    min-width: 200px;
  }

  .search-input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.5rem;
    color: var(--color-ui-text);
    font-size: 0.875rem;
  }

  .search-input::placeholder {
    color: rgba(255, 255, 255, 0.4);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--color-ui-action, #e94560);
  }

  .filter-group {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .filter-select {
    padding: 0.5rem 0.75rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.5rem;
    color: var(--color-ui-text);
    font-size: 0.875rem;
    cursor: pointer;
    min-width: 120px;
  }

  .filter-select:focus {
    outline: none;
    border-color: var(--color-ui-action, #e94560);
  }

  .filter-select option {
    background: var(--color-ui-panel, #16213e);
    color: var(--color-ui-text);
  }
</style>
