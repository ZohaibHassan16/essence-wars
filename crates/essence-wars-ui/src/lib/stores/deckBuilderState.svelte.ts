// Deck Builder state store using Svelte 5 runes

import type {
  BrowsableCard,
  CommanderDto,
  CustomDeck,
  CustomDeckInfo,
  DeckValidation,
  PlaystyleScore,
} from "$lib/api/types";
import * as api from "$lib/api/deckBuilder";
import { playSound } from "$lib/audio";

// Deck builder phase
export type DeckBuilderPhase = "closed" | "list" | "building";

// Filter state
export interface CardFilters {
  search: string;
  cost: number | null;
  cardType: string | null;
  keyword: string | null;
  rarity: string | null;
}

class DeckBuilderStore {
  // Core state
  phase = $state<DeckBuilderPhase>("closed");
  isLoading = $state(false);
  error = $state<string | null>(null);

  // Card browser state
  allCards = $state<BrowsableCard[]>([]);
  commanders = $state<CommanderDto[]>([]);

  // Filter state
  filters = $state<CardFilters>({
    search: "",
    cost: null,
    cardType: null,
    keyword: null,
    rarity: null,
  });

  // Commander selection
  selectedCommander = $state<CommanderDto | null>(null);

  // Deck building state
  currentDeckId = $state<string | null>(null);
  deckName = $state("New Deck");
  deckDescription = $state("");
  deckCards = $state<number[]>([]);

  // Validation state
  validation = $state<DeckValidation | null>(null);
  playstyle = $state<PlaystyleScore | null>(null);

  // Custom decks list
  savedDecks = $state<CustomDeckInfo[]>([]);

  // Unsaved changes tracking
  hasUnsavedChanges = $state(false);

  // =========================================================================
  // Computed Properties
  // =========================================================================

  /** Current deck's faction based on selected commander */
  get deckFaction(): string | null {
    return this.selectedCommander?.faction ?? null;
  }

  /** Number of cards in the deck */
  get cardCount(): number {
    return this.deckCards.length;
  }

  /** Cards filtered by current filters */
  get filteredCards(): BrowsableCard[] {
    let cards = this.allCards;

    // Filter by commander's faction (faction + neutral only)
    if (this.selectedCommander) {
      const faction = this.selectedCommander.faction;
      cards = cards.filter(
        (c) => c.faction === faction || c.faction === "neutral"
      );
    }

    // Apply search filter
    if (this.filters.search) {
      const search = this.filters.search.toLowerCase();
      cards = cards.filter((c) => c.name.toLowerCase().includes(search));
    }

    // Apply cost filter
    if (this.filters.cost !== null) {
      const cost = this.filters.cost;
      if (cost === 7) {
        // 7+ bucket
        cards = cards.filter((c) => c.cost >= 7);
      } else {
        cards = cards.filter((c) => c.cost === cost);
      }
    }

    // Apply card type filter
    if (this.filters.cardType) {
      cards = cards.filter((c) => c.cardType === this.filters.cardType);
    }

    // Apply keyword filter
    if (this.filters.keyword) {
      const keyword = this.filters.keyword;
      cards = cards.filter((c) => c.keywords.includes(keyword));
    }

    // Apply rarity filter
    if (this.filters.rarity) {
      cards = cards.filter((c) => c.rarity === this.filters.rarity);
    }

    // Sort by cost, then name
    return cards.sort((a, b) => {
      if (a.cost !== b.cost) return a.cost - b.cost;
      return a.name.localeCompare(b.name);
    });
  }

  /** Count of copies per card in deck */
  get cardCopyCounts(): Map<number, number> {
    const counts = new Map<number, number>();
    for (const cardId of this.deckCards) {
      counts.set(cardId, (counts.get(cardId) ?? 0) + 1);
    }
    return counts;
  }

  /** Mana curve data: { 0: count, 1: count, ..., 7: count (7+) } */
  get manaCurve(): Record<number, number> {
    const curve: Record<number, number> = {
      0: 0,
      1: 0,
      2: 0,
      3: 0,
      4: 0,
      5: 0,
      6: 0,
      7: 0,
    };
    for (const cardId of this.deckCards) {
      const card = this.allCards.find((c) => c.cardId === cardId);
      if (card) {
        const bucket = Math.min(card.cost, 7);
        curve[bucket]++;
      }
    }
    return curve;
  }

  /** Card type breakdown */
  get cardTypeBreakdown(): { creature: number; spell: number; support: number } {
    let creature = 0,
      spell = 0,
      support = 0;
    for (const cardId of this.deckCards) {
      const card = this.allCards.find((c) => c.cardId === cardId);
      if (card) {
        if (card.cardType === "creature") creature++;
        else if (card.cardType === "spell") spell++;
        else if (card.cardType === "support") support++;
      }
    }
    return { creature, spell, support };
  }

  /** All unique keywords in filtered cards (for keyword filter dropdown) */
  get availableKeywords(): string[] {
    const keywords = new Set<string>();
    for (const card of this.filteredCards) {
      for (const keyword of card.keywords) {
        keywords.add(keyword);
      }
    }
    return Array.from(keywords).sort();
  }

  /** Whether the current deck is valid */
  get isValid(): boolean {
    return this.validation?.isValid ?? false;
  }

  /** Alias for savedDecks (used by DeckListView) */
  get customDecks(): CustomDeckInfo[] {
    return this.savedDecks;
  }

  // =========================================================================
  // Initialization
  // =========================================================================

  async initialize() {
    this.isLoading = true;
    this.error = null;
    try {
      const [cards, commanders, savedDecks] = await Promise.all([
        api.listAllCards(),
        api.listCommanders(),
        api.listCustomDecks(),
      ]);
      this.allCards = cards;
      this.commanders = commanders;
      this.savedDecks = savedDecks;
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  // =========================================================================
  // Phase Management
  // =========================================================================

  /** Open the deck builder to the deck list */
  async open() {
    await this.initialize();
    this.phase = "list";
    playSound("menuOpen");
  }

  /** Close the deck builder */
  close() {
    if (this.hasUnsavedChanges) {
      // Could show confirmation dialog here
      console.warn("Closing with unsaved changes");
    }
    this.phase = "closed";
    this.resetBuilderState();
    playSound("menuClose");
  }

  /** Start building a new deck */
  startNewDeck() {
    this.resetBuilderState();
    this.phase = "building";
    playSound("buttonClick");
  }

  /** Alias for startNewDeck (used by DeckListView) */
  createNewDeck() {
    this.startNewDeck();
  }

  /** Return to deck list from builder */
  backToList() {
    if (this.hasUnsavedChanges) {
      console.warn("Going back with unsaved changes");
    }
    this.resetBuilderState();
    this.phase = "list";
    playSound("buttonClick");
  }

  // =========================================================================
  // Commander Selection
  // =========================================================================

  /** Select a commander (determines deck faction) */
  selectCommander(commander: CommanderDto) {
    const previousFaction = this.selectedCommander?.faction;

    this.selectedCommander = commander;
    playSound("cardSelect");

    // If faction changed, clear deck cards
    if (previousFaction && previousFaction !== commander.faction) {
      this.deckCards = [];
      this.validation = null;
      this.playstyle = null;
    }

    this.hasUnsavedChanges = true;
    this.updatePlaystyle();
  }

  // =========================================================================
  // Card Management
  // =========================================================================

  /** Add a card to the deck */
  addCard(cardId: number) {
    const card = this.allCards.find((c) => c.cardId === cardId);
    if (!card) return;

    // Check copy limit
    const currentCount = this.cardCopyCounts.get(cardId) ?? 0;
    if (currentCount >= card.copyLimit) {
      playSound("damage"); // Error sound
      return;
    }

    // Check max deck size
    if (this.deckCards.length >= 100) {
      playSound("damage");
      return;
    }

    this.deckCards = [...this.deckCards, cardId];
    this.hasUnsavedChanges = true;
    playSound("cardSelect");

    this.updatePlaystyle();
    this.updateValidation();
  }

  /** Remove one copy of a card from the deck */
  removeCard(cardId: number) {
    const index = this.deckCards.indexOf(cardId);
    if (index === -1) return;

    this.deckCards = [
      ...this.deckCards.slice(0, index),
      ...this.deckCards.slice(index + 1),
    ];
    this.hasUnsavedChanges = true;
    playSound("menuClose");

    this.updatePlaystyle();
    this.updateValidation();
  }

  /** Clear all cards from the deck */
  clearDeck() {
    this.deckCards = [];
    this.validation = null;
    this.playstyle = null;
    this.hasUnsavedChanges = true;
    playSound("buttonClick");
  }

  // =========================================================================
  // Filters
  // =========================================================================

  setSearchFilter(search: string) {
    this.filters = { ...this.filters, search };
  }

  setCostFilter(cost: number | null) {
    this.filters = { ...this.filters, cost };
  }

  setCardTypeFilter(cardType: string | null) {
    this.filters = { ...this.filters, cardType };
  }

  setKeywordFilter(keyword: string | null) {
    this.filters = { ...this.filters, keyword };
  }

  setRarityFilter(rarity: string | null) {
    this.filters = { ...this.filters, rarity };
  }

  clearFilters() {
    this.filters = {
      search: "",
      cost: null,
      cardType: null,
      keyword: null,
      rarity: null,
    };
  }

  // =========================================================================
  // Deck Persistence
  // =========================================================================

  /** Load an existing deck for editing */
  async loadDeck(deckId: string) {
    this.isLoading = true;
    this.error = null;
    try {
      const deck = await api.loadCustomDeck(deckId);

      // Find commander
      const commander = this.commanders.find((c) => c.id === deck.commander);
      if (!commander) {
        throw new Error(`Commander ${deck.commander} not found`);
      }

      this.currentDeckId = deck.id;
      this.deckName = deck.name;
      this.deckDescription = deck.description;
      this.deckCards = deck.cards;
      this.selectedCommander = commander;
      this.hasUnsavedChanges = false;
      this.phase = "building";

      await this.updatePlaystyle();
      await this.updateValidation();

      playSound("buttonClick");
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  /** Save the current deck */
  async saveDeck() {
    if (!this.selectedCommander) {
      this.error = "Please select a commander";
      return;
    }

    if (!this.deckName.trim()) {
      this.error = "Please enter a deck name";
      return;
    }

    this.isLoading = true;
    this.error = null;
    try {
      // Generate ID if new deck
      const id =
        this.currentDeckId ??
        `custom_${this.deckName.toLowerCase().replace(/\s+/g, "_")}_${Date.now()}`;

      const deck: CustomDeck = {
        id,
        name: this.deckName.trim(),
        commander: this.selectedCommander.id,
        cards: this.deckCards,
        description: this.deckDescription.trim(),
        tags: ["custom", this.selectedCommander.faction],
      };

      await api.saveCustomDeck(deck);
      this.currentDeckId = id;
      this.hasUnsavedChanges = false;

      // Refresh saved decks list
      this.savedDecks = await api.listCustomDecks();

      playSound("buttonClick");
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  /** Delete a custom deck */
  async deleteDeck(deckId: string) {
    this.isLoading = true;
    this.error = null;
    try {
      await api.deleteCustomDeck(deckId);

      // Refresh saved decks list
      this.savedDecks = await api.listCustomDecks();

      // If we were editing this deck, reset
      if (this.currentDeckId === deckId) {
        this.resetBuilderState();
        this.phase = "list";
      }

      playSound("buttonClick");
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  // =========================================================================
  // Validation & Playstyle
  // =========================================================================

  private async updateValidation() {
    if (!this.selectedCommander) {
      this.validation = null;
      return;
    }

    try {
      const deck: CustomDeck = {
        id: this.currentDeckId ?? "temp",
        name: this.deckName,
        commander: this.selectedCommander.id,
        cards: this.deckCards,
        description: this.deckDescription,
        tags: [],
      };
      this.validation = await api.validateCustomDeck(deck);
    } catch (e) {
      console.error("Validation failed:", e);
    }
  }

  private async updatePlaystyle() {
    if (!this.selectedCommander || this.deckCards.length === 0) {
      this.playstyle = null;
      return;
    }

    try {
      this.playstyle = await api.calculateDeckPlaystyle(
        this.deckCards,
        this.selectedCommander.id
      );
    } catch (e) {
      console.error("Playstyle calculation failed:", e);
    }
  }

  // =========================================================================
  // Private Helpers
  // =========================================================================

  private resetBuilderState() {
    this.currentDeckId = null;
    this.deckName = "New Deck";
    this.deckDescription = "";
    this.deckCards = [];
    this.selectedCommander = null;
    this.validation = null;
    this.playstyle = null;
    this.hasUnsavedChanges = false;
    this.clearFilters();
    this.error = null;
  }
}

export const deckBuilderStore = new DeckBuilderStore();
