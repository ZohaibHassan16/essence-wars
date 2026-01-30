<script lang="ts">
  import { onMount } from "svelte";
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";
  import type { TimelineData } from "$lib/stats/types";

  interface Props {
    timeline: TimelineData;
  }

  let { timeline }: Props = $props();

  let container = $state<HTMLDivElement>(undefined!);
  let chart: uPlot | null = null;

  const P1_COLOR = "#4ade80";
  const P2_COLOR = "#f87171";

  function createChart() {
    if (!container || timeline.turns.length === 0) return;

    if (chart) {
      chart.destroy();
      chart = null;
    }

    const data: uPlot.AlignedData = [
      timeline.turns,
      timeline.player1EssenceSpent,
      timeline.player2EssenceSpent,
    ];

    // Calculate max for y-axis
    const maxEssence = Math.max(
      ...timeline.player1EssenceSpent,
      ...timeline.player2EssenceSpent,
      10
    );

    const opts: uPlot.Options = {
      width: container.clientWidth,
      height: 350,
      title: "Cumulative Essence Spent",
      class: "uplot-dark",
      scales: {
        x: { time: false },
        y: {
          auto: true,
          range: [0, Math.ceil(maxEssence * 1.1)],
        },
      },
      axes: [
        {
          label: "Turn",
          stroke: "#9ca3af",
          grid: { stroke: "#374151", width: 1 },
          ticks: { stroke: "#374151" },
        },
        {
          label: "Essence",
          stroke: "#9ca3af",
          grid: { stroke: "#374151", width: 1 },
          ticks: { stroke: "#374151" },
        },
      ],
      series: [
        {},
        {
          label: "Player 1",
          stroke: P1_COLOR,
          width: 2,
          fill: `${P1_COLOR}15`,
          points: { show: false },
        },
        {
          label: "Player 2",
          stroke: P2_COLOR,
          width: 2,
          fill: `${P2_COLOR}15`,
          points: { show: false },
        },
      ],
      legend: { show: true },
      cursor: { drag: { x: false, y: false } },
    };

    chart = new uPlot(opts, data, container);
  }

  onMount(() => {
    createChart();

    const resizeObserver = new ResizeObserver(() => {
      if (chart && container) {
        chart.setSize({ width: container.clientWidth, height: 350 });
      }
    });
    resizeObserver.observe(container);

    return () => {
      resizeObserver.disconnect();
      if (chart) chart.destroy();
    };
  });

  $effect(() => {
    if (timeline && container) {
      createChart();
    }
  });
</script>

<div bind:this={container} class="w-full"></div>

<div class="mt-2 text-xs text-center text-ui-text-dim">
  Cumulative essence spent on cards throughout the game
</div>

<style>
  :global(.uplot-dark) {
    --uplot-bg: transparent;
  }
  :global(.uplot-dark .u-title) {
    color: #e5e7eb;
    font-size: 14px;
    font-weight: 600;
  }
  :global(.uplot-dark .u-legend) {
    font-size: 12px;
  }
  :global(.uplot-dark .u-legend .u-series) {
    padding: 4px 8px;
  }
  :global(.uplot-dark .u-legend .u-label) {
    color: #9ca3af;
  }
  :global(.uplot-dark .u-legend .u-value) {
    color: #e5e7eb;
    font-weight: 500;
  }
</style>
