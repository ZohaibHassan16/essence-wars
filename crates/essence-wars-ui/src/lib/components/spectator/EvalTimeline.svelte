<script lang="ts">
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";
  import type { EvalPoint, KeyMoment } from "$lib/api/types";

  let {
    height = 100,
    showKeyMoments = true,
  }: {
    height?: number;
    showKeyMoments?: boolean;
  } = $props();

  let canvasElement: HTMLCanvasElement | null = $state(null);
  let containerWidth = $state(0);

  // Get data from store
  const evalHistory = $derived(spectatorStore.match?.evalHistory ?? []);
  const keyMoments = $derived(spectatorStore.match?.keyMoments ?? []);
  const currentActionIndex = $derived(spectatorStore.currentActionIndex);
  const totalActions = $derived(spectatorStore.totalActions);

  // Compute Y-axis range (symmetric around 0)
  const yRange = $derived(() => {
    if (evalHistory.length === 0) return { min: -10, max: 10 };
    const maxAbs = Math.max(
      ...evalHistory.map((p) => Math.abs(p.evalScore)),
      5 // Minimum range
    );
    // Round up to nice number
    const rounded = Math.ceil(maxAbs / 5) * 5;
    return { min: -rounded, max: rounded };
  });

  // Draw the chart
  function drawChart() {
    if (!canvasElement || containerWidth === 0) return;

    const ctx = canvasElement.getContext("2d");
    if (!ctx) return;

    const width = containerWidth;
    const dpr = window.devicePixelRatio || 1;

    // Set canvas size with device pixel ratio for sharp rendering
    canvasElement.width = width * dpr;
    canvasElement.height = height * dpr;
    canvasElement.style.width = `${width}px`;
    canvasElement.style.height = `${height}px`;
    ctx.scale(dpr, dpr);

    // Clear
    ctx.clearRect(0, 0, width, height);

    // Padding
    const padding = { left: 30, right: 10, top: 10, bottom: 20 };
    const chartWidth = width - padding.left - padding.right;
    const chartHeight = height - padding.top - padding.bottom;

    // Background
    ctx.fillStyle = "rgba(0, 0, 0, 0.3)";
    ctx.fillRect(padding.left, padding.top, chartWidth, chartHeight);

    // Draw grid and axis
    const { min: yMin, max: yMax } = yRange();
    const yScale = (val: number) =>
      padding.top + chartHeight * (1 - (val - yMin) / (yMax - yMin));
    const xScale = (idx: number) =>
      padding.left + (chartWidth * (idx + 1)) / Math.max(totalActions, 1);

    // Horizontal center line (y = 0)
    ctx.strokeStyle = "rgba(255, 255, 255, 0.3)";
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(padding.left, yScale(0));
    ctx.lineTo(padding.left + chartWidth, yScale(0));
    ctx.stroke();

    // Y-axis labels
    ctx.fillStyle = "rgba(255, 255, 255, 0.5)";
    ctx.font = "10px monospace";
    ctx.textAlign = "right";
    ctx.fillText(`+${yMax}`, padding.left - 4, padding.top + 10);
    ctx.fillText(`${yMin}`, padding.left - 4, padding.top + chartHeight);
    ctx.fillText("0", padding.left - 4, yScale(0) + 3);

    // X-axis label
    ctx.textAlign = "center";
    ctx.fillText("Actions", padding.left + chartWidth / 2, height - 2);

    // No data case
    if (evalHistory.length === 0) {
      ctx.fillStyle = "rgba(255, 255, 255, 0.3)";
      ctx.textAlign = "center";
      ctx.fillText("No evaluation data", width / 2, height / 2);
      return;
    }

    // Draw key moments (vertical lines) behind the main line
    if (showKeyMoments) {
      for (const moment of keyMoments) {
        const x = xScale(moment.actionIndex);
        ctx.strokeStyle = getMomentColor(moment.momentType, 0.3);
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(x, padding.top);
        ctx.lineTo(x, padding.top + chartHeight);
        ctx.stroke();
      }
    }

    // Draw the evaluation line
    ctx.strokeStyle = "#60a5fa"; // Blue
    ctx.lineWidth = 2;
    ctx.beginPath();

    // Start from initial state (assumed 0 eval)
    ctx.moveTo(padding.left, yScale(0));

    for (const point of evalHistory) {
      const x = xScale(point.actionIndex);
      const y = yScale(point.evalScore);
      ctx.lineTo(x, y);
    }
    ctx.stroke();

    // Fill areas above/below 0
    // P1 advantage area (above 0)
    ctx.fillStyle = "rgba(52, 211, 153, 0.15)"; // Green
    ctx.beginPath();
    ctx.moveTo(padding.left, yScale(0));
    for (const point of evalHistory) {
      const x = xScale(point.actionIndex);
      const y = yScale(Math.max(point.evalScore, 0));
      ctx.lineTo(x, y);
    }
    ctx.lineTo(xScale(evalHistory.length - 1), yScale(0));
    ctx.closePath();
    ctx.fill();

    // P2 advantage area (below 0)
    ctx.fillStyle = "rgba(239, 68, 68, 0.15)"; // Red
    ctx.beginPath();
    ctx.moveTo(padding.left, yScale(0));
    for (const point of evalHistory) {
      const x = xScale(point.actionIndex);
      const y = yScale(Math.min(point.evalScore, 0));
      ctx.lineTo(x, y);
    }
    ctx.lineTo(xScale(evalHistory.length - 1), yScale(0));
    ctx.closePath();
    ctx.fill();

    // Draw key moment markers
    if (showKeyMoments) {
      for (const moment of keyMoments) {
        const x = xScale(moment.actionIndex);
        // Find the eval score at this action
        const point = evalHistory.find((p) => p.actionIndex === moment.actionIndex);
        const y = point ? yScale(point.evalScore) : yScale(0);

        // Draw marker dot
        ctx.fillStyle = getMomentColor(moment.momentType, 1);
        ctx.beginPath();
        ctx.arc(x, y, 5, 0, Math.PI * 2);
        ctx.fill();

        // Draw outline
        ctx.strokeStyle = "white";
        ctx.lineWidth = 1;
        ctx.stroke();
      }
    }

    // Draw current position indicator
    if (currentActionIndex >= 0) {
      const x = xScale(currentActionIndex);
      ctx.strokeStyle = "rgba(255, 255, 255, 0.8)";
      ctx.lineWidth = 2;
      ctx.setLineDash([4, 4]);
      ctx.beginPath();
      ctx.moveTo(x, padding.top);
      ctx.lineTo(x, padding.top + chartHeight);
      ctx.stroke();
      ctx.setLineDash([]);

      // Draw position dot
      const point = evalHistory.find((p) => p.actionIndex === currentActionIndex);
      if (point) {
        const y = yScale(point.evalScore);
        ctx.fillStyle = "white";
        ctx.beginPath();
        ctx.arc(x, y, 4, 0, Math.PI * 2);
        ctx.fill();
      }
    }
  }

  function getMomentColor(momentType: string, alpha: number): string {
    switch (momentType) {
      case "p1Surge":
        return `rgba(52, 211, 153, ${alpha})`; // Green
      case "p2Surge":
        return `rgba(239, 68, 68, ${alpha})`; // Red
      case "leadChange":
        return `rgba(251, 191, 36, ${alpha})`; // Yellow
      case "decisive":
        return `rgba(168, 85, 247, ${alpha})`; // Purple
      default:
        return `rgba(255, 255, 255, ${alpha})`;
    }
  }

  // Handle click to jump to action
  function handleClick(event: MouseEvent) {
    if (!canvasElement || totalActions === 0) return;

    const rect = canvasElement.getBoundingClientRect();
    const x = event.clientX - rect.left;

    const padding = { left: 30, right: 10 };
    const chartWidth = containerWidth - padding.left - padding.right;

    // Convert x to action index
    const relativeX = x - padding.left;
    if (relativeX < 0 || relativeX > chartWidth) return;

    const actionIndex = Math.round((relativeX / chartWidth) * totalActions) - 1;
    const clampedIndex = Math.max(-1, Math.min(actionIndex, totalActions - 1));

    spectatorStore.jumpToAction(clampedIndex);
  }

  // Redraw when data changes
  $effect(() => {
    // Track dependencies
    evalHistory;
    keyMoments;
    currentActionIndex;
    containerWidth;

    drawChart();
  });
</script>

<div
  class="eval-timeline"
  bind:clientWidth={containerWidth}
>
  <canvas
    bind:this={canvasElement}
    onclick={handleClick}
    class="cursor-pointer"
  ></canvas>

  {#if showKeyMoments && keyMoments.length > 0}
    <div class="key-moments-legend">
      <span class="legend-label">Key Moments:</span>
      {#each keyMoments.slice(0, 3) as moment}
        <button
          class="moment-chip"
          style="--moment-color: {getMomentColor(moment.momentType, 1)}"
          onclick={() => spectatorStore.jumpToAction(moment.actionIndex)}
          title={moment.actionDescription}
        >
          T{moment.turn}
        </button>
      {/each}
      {#if keyMoments.length > 3}
        <span class="more-moments">+{keyMoments.length - 3} more</span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .eval-timeline {
    width: 100%;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 0.5rem;
    padding: 0.5rem;
  }

  canvas {
    display: block;
    width: 100%;
  }

  .key-moments-legend {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    font-size: 0.7rem;
    flex-wrap: wrap;
  }

  .legend-label {
    color: rgba(255, 255, 255, 0.5);
  }

  .moment-chip {
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    background: var(--moment-color, rgba(255, 255, 255, 0.2));
    color: white;
    font-size: 0.65rem;
    font-weight: 600;
    border: none;
    cursor: pointer;
    transition: transform 0.1s, opacity 0.1s;
  }

  .moment-chip:hover {
    transform: scale(1.05);
    opacity: 0.9;
  }

  .more-moments {
    color: rgba(255, 255, 255, 0.4);
    font-size: 0.65rem;
  }
</style>
