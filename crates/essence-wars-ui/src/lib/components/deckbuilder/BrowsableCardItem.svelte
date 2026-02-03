<script lang="ts">
  import type { BrowsableCard } from "$lib/api/types";
  import { playSound } from "$lib/audio";

  let { card, copiesInDeck, onAddCard }: {
    card: BrowsableCard;
    copiesInDeck: number;
    onAddCard: () => void;
  } = $props();

  const isMaxed = $derived(copiesInDeck >= card.copyLimit);

  function handleClick() {
    if (!isMaxed) {
      onAddCard();
    } else {
      playSound("damage");
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      handleClick();
    }
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

  function getCardTypeIcon(type: string): string {
    switch (type) {
      case "creature":
        return "⚔";
      case "spell":
        return "✦";
      case "support":
        return "◈";
      default:
        return "•";
    }
  }
</script>

<button
  class="browsable-card"
  class:maxed={isMaxed}
  onclick={handleClick}
  onkeydown={handleKeydown}
  onmouseenter={() => playSound("buttonHover")}
  style="--rarity-color: {getRarityColor(card.rarity)}"
>
  <div class="card-cost">{card.cost}</div>

  <div class="card-art">
    <img
      src={card.artPath}
      alt={card.name}
      onerror={(e) => {
        (e.target as HTMLImageElement).style.display = "none";
      }}
    />
    <div class="card-type-icon">{getCardTypeIcon(card.cardType)}</div>
  </div>

  <div class="card-name">{card.name}</div>

  {#if card.cardType === "creature" && card.attack !== undefined && card.health !== undefined}
    <div class="card-stats">
      <span class="attack">{card.attack}</span>
      <span class="separator">/</span>
      <span class="health">{card.health}</span>
    </div>
  {/if}

  <div class="copy-indicator" class:maxed={isMaxed}>
    {copiesInDeck}/{card.copyLimit}
  </div>

  {#if card.keywords.length > 0}
    <div class="keywords-preview">
      {card.keywords.slice(0, 2).join(", ")}
      {#if card.keywords.length > 2}...{/if}
    </div>
  {/if}
</button>

<style>
  .browsable-card {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0.5rem;
    background: rgba(0, 0, 0, 0.3);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.5rem;
    cursor: pointer;
    transition: all 0.2s;
    text-align: center;
  }

  .browsable-card:hover:not(.maxed) {
    transform: translateY(-2px);
    border-color: var(--rarity-color);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }

  .browsable-card.maxed {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .card-cost {
    position: absolute;
    top: -8px;
    left: -8px;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-mana, #3b82f6);
    border-radius: 50%;
    font-weight: 700;
    font-size: 0.875rem;
    color: white;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  .card-art {
    position: relative;
    width: 100%;
    aspect-ratio: 1;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 0.25rem;
    overflow: hidden;
    margin-bottom: 0.5rem;
  }

  .card-art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .card-type-icon {
    position: absolute;
    bottom: 4px;
    right: 4px;
    font-size: 0.875rem;
    background: rgba(0, 0, 0, 0.6);
    padding: 0.125rem 0.25rem;
    border-radius: 0.25rem;
  }

  .card-name {
    font-size: 0.75rem;
    font-weight: 600;
    line-height: 1.2;
    margin-bottom: 0.25rem;
    color: var(--color-ui-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    width: 100%;
  }

  .card-stats {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.875rem;
    font-weight: 700;
  }

  .attack {
    color: #ef4444;
  }

  .separator {
    color: rgba(255, 255, 255, 0.4);
  }

  .health {
    color: #22c55e;
  }

  .copy-indicator {
    position: absolute;
    top: -8px;
    right: -8px;
    padding: 0.125rem 0.375rem;
    background: rgba(0, 0, 0, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.25rem;
    font-size: 0.625rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.8);
  }

  .copy-indicator.maxed {
    background: rgba(239, 68, 68, 0.7);
    border-color: #ef4444;
  }

  .keywords-preview {
    font-size: 0.625rem;
    color: rgba(255, 255, 255, 0.5);
    margin-top: 0.25rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    width: 100%;
  }
</style>
