<script lang="ts">
  import type { CardDto } from "$lib/api/types";
  import HandCard from "../HandCard.svelte";

  let {
    cards,
    selectedCardIndex = null,
    isInteractive = false,
    isPlayableCallback,
    onCardClick,
    compact = false,
  }: {
    cards: CardDto[];
    selectedCardIndex?: number | null;
    isInteractive?: boolean;
    isPlayableCallback?: (index: number) => boolean;
    onCardClick?: (index: number) => void;
    compact?: boolean;
  } = $props();

  // Calculate rotation for fanning effect
  function getCardRotation(index: number, total: number): number {
    if (total <= 1) return 0;
    const maxAngle = compact ? 15 : 25; // degrees
    const angleStep = Math.min(maxAngle * 2 / (total - 1), 8);
    const centerIndex = (total - 1) / 2;
    return (index - centerIndex) * angleStep;
  }

  // Calculate vertical offset for arc effect
  function getCardOffset(index: number, total: number): number {
    if (total <= 1) return 0;
    const centerIndex = (total - 1) / 2;
    const distance = Math.abs(index - centerIndex);
    const maxOffset = compact ? 8 : 15;
    return distance * distance * (maxOffset / Math.pow(centerIndex || 1, 2));
  }
</script>

<div class="hand-container relative" class:py-2={!compact} class:py-1={compact}>
  {#if cards.length === 0}
    <span class="text-ui-text-dim text-sm">Empty hand</span>
  {:else}
    {#each cards as card, i}
      {@const rotation = getCardRotation(i, cards.length)}
      {@const yOffset = getCardOffset(i, cards.length)}
      {@const isPlayable = isPlayableCallback ? isPlayableCallback(i) : false}
      <div
        class="hand-card relative"
        style="
          transform: rotate({rotation}deg) translateY({yOffset}px);
          z-index: {i};
        "
        style:--hover-z-index={cards.length + 10}
      >
        <HandCard
          {card}
          index={i}
          isSelected={selectedCardIndex === i}
          {isPlayable}
          onClick={isInteractive && onCardClick ? () => onCardClick(i) : undefined}
        />
      </div>
    {/each}
  {/if}
</div>

<style>
  .hand-card:hover {
    z-index: var(--hover-z-index, 100) !important;
  }
</style>
