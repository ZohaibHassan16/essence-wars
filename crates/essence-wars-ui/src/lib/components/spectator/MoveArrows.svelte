<script lang="ts">
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import type { MoveScoreDto } from "$lib/api/types";

  let {
    showThreshold = 0.05,
    maxArrows = 5,
  }: {
    showThreshold?: number;
    maxArrows?: number;
  } = $props();

  // Get current insights
  const insights = $derived(spectatorStore.currentInsights);
  const currentPlayer = $derived(spectatorStore.currentAction?.player ?? 1);

  // Filter to attack moves above threshold
  const attackMoves = $derived(() => {
    if (!insights) return [];
    return insights.moveScores
      .filter((m) => m.action.actionType === "attack" && m.probability >= showThreshold)
      .slice(0, maxArrows);
  });

  // Slot positions (relative percentages for SVG viewBox)
  // These map to the 5 creature slots on each side
  const slotPositions = {
    // Player side (bottom) - slots 0-4
    player: [
      { x: 15, y: 75 },
      { x: 30, y: 75 },
      { x: 50, y: 75 },
      { x: 70, y: 75 },
      { x: 85, y: 75 },
    ],
    // Opponent side (top) - slots 0-4
    opponent: [
      { x: 15, y: 25 },
      { x: 30, y: 25 },
      { x: 50, y: 25 },
      { x: 70, y: 25 },
      { x: 85, y: 25 },
    ],
    // Face/commander positions
    playerFace: { x: 5, y: 50 },
    opponentFace: { x: 95, y: 50 },
  };

  // Get position for a slot
  function getSlotPosition(slot: number | undefined, isSource: boolean): { x: number; y: number } {
    if (slot === undefined || slot < 0 || slot > 4) {
      // Face attack
      return isSource
        ? currentPlayer === 1
          ? slotPositions.player[2]
          : slotPositions.opponent[2]
        : currentPlayer === 1
          ? slotPositions.opponentFace
          : slotPositions.playerFace;
    }

    // Creature slot
    if (isSource) {
      return currentPlayer === 1 ? slotPositions.player[slot] : slotPositions.opponent[slot];
    } else {
      return currentPlayer === 1 ? slotPositions.opponent[slot] : slotPositions.player[slot];
    }
  }

  // Calculate arrow properties
  function getArrowProps(move: MoveScoreDto) {
    const source = getSlotPosition(move.action.sourceSlot, true);
    const target = getSlotPosition(move.action.targetSlot, false);

    // Arrow thickness based on probability (2-8 pixels)
    const thickness = 2 + move.probability * 6;

    // Opacity based on probability
    const opacity = 0.3 + move.probability * 0.5;

    // Color: gold for chosen, white for others
    const color = move.isChosen ? "#fbbf24" : "#ffffff";

    return { source, target, thickness, opacity, color };
  }

  // Calculate arrow path with curve
  function getArrowPath(
    source: { x: number; y: number },
    target: { x: number; y: number }
  ): string {
    // Control point for curve (offset to the right)
    const midX = (source.x + target.x) / 2;
    const midY = (source.y + target.y) / 2;
    const dx = target.x - source.x;
    const dy = target.y - source.y;

    // Perpendicular offset for curve
    const perpX = -dy * 0.2;
    const perpY = dx * 0.2;

    const ctrlX = midX + perpX;
    const ctrlY = midY + perpY;

    return `M ${source.x} ${source.y} Q ${ctrlX} ${ctrlY} ${target.x} ${target.y}`;
  }
</script>

{#if attackMoves().length > 0}
  <svg
    class="move-arrows-overlay"
    viewBox="0 0 100 100"
    preserveAspectRatio="none"
  >
    <!-- Arrow marker definition -->
    <defs>
      <marker
        id="arrowhead"
        markerWidth="10"
        markerHeight="7"
        refX="9"
        refY="3.5"
        orient="auto"
      >
        <polygon points="0 0, 10 3.5, 0 7" fill="currentColor" />
      </marker>
      <marker
        id="arrowhead-gold"
        markerWidth="10"
        markerHeight="7"
        refX="9"
        refY="3.5"
        orient="auto"
      >
        <polygon points="0 0, 10 3.5, 0 7" fill="#fbbf24" />
      </marker>
    </defs>

    <!-- Render arrows -->
    {#each attackMoves() as move}
      {@const { source, target, thickness, opacity, color } = getArrowProps(move)}
      {@const midX = (source.x + target.x) / 2}
      {@const midY = (source.y + target.y) / 2}
      <g class="arrow-group" style="--arrow-color: {color}">
        <!-- Glow effect for chosen move -->
        {#if move.isChosen}
          <path
            d={getArrowPath(source, target)}
            fill="none"
            stroke="#fbbf24"
            stroke-width={thickness + 4}
            stroke-opacity={opacity * 0.3}
            stroke-linecap="round"
          />
        {/if}

        <!-- Main arrow -->
        <path
          d={getArrowPath(source, target)}
          fill="none"
          stroke={color}
          stroke-width={thickness}
          stroke-opacity={opacity}
          stroke-linecap="round"
          marker-end={move.isChosen ? "url(#arrowhead-gold)" : "url(#arrowhead)"}
          class="arrow-path"
        />

        <!-- Probability label -->
        <text
          x={midX}
          y={midY - 3}
          text-anchor="middle"
          font-size="4"
          fill={color}
          fill-opacity={opacity + 0.2}
          class="arrow-label"
        >
          {Math.round(move.probability * 100)}%
        </text>
      </g>
    {/each}
  </svg>
{/if}

<style>
  .move-arrows-overlay {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 10;
  }

  .arrow-path {
    transition: stroke-width 0.2s, stroke-opacity 0.2s;
  }

  .arrow-group:hover .arrow-path {
    stroke-opacity: 1;
  }

  .arrow-label {
    font-family: ui-monospace, monospace;
    font-weight: 600;
  }
</style>
