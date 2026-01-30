<script lang="ts">
  import { onMount } from "svelte";
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";
  import type { TimelineData } from "$lib/stats/types";

  interface Props {
    timeline: TimelineData;
    hasAiData: boolean;
  }

  let { timeline, hasAiData }: Props = $props();

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

    // Check if we have meaningful win probability data
    const hasWinProbData = timeline.player1WinProb.some((v) => v !== 0.5 && v !== undefined);

    if (!hasWinProbData && !hasAiData) {
      return; // Don't create chart without data
    }

    const data: uPlot.AlignedData = [
      timeline.turns,
      timeline.player1WinProb.map((v) => (v ?? 0.5) * 100),
      timeline.player2WinProb.map((v) => (v ?? 0.5) * 100),
    ];

    const opts: uPlot.Options = {
      width: container.clientWidth,
      height: 350,
      title: "Win Probability Over Time",
      class: "uplot-dark",
      scales: {
        x: { time: false },
        y: {
          auto: false,
          range: [0, 100],
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
          label: "Win %",
          stroke: "#9ca3af",
          grid: { stroke: "#374151", width: 1 },
          ticks: { stroke: "#374151" },
          values: (_, ticks) => ticks.map((v) => `${v}%`),
        },
      ],
      series: [
        {},
        {
          label: "Player 1",
          stroke: P1_COLOR,
          width: 2,
          fill: `${P1_COLOR}20`,
          points: { show: false },
        },
        {
          label: "Player 2",
          stroke: P2_COLOR,
          width: 2,
          fill: `${P2_COLOR}20`,
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

  const showNoDataMessage = $derived(
    !hasAiData || !timeline.player1WinProb.some((v) => v !== 0.5 && v !== undefined)
  );
</script>

{#if showNoDataMessage}
  <div class="flex flex-col items-center justify-center h-[350px] text-center">
    <div class="text-4xl mb-3">📊</div>
    <div class="text-ui-text font-medium">No Win Probability Data</div>
    <div class="text-sm text-ui-text-dim mt-2 max-w-sm">
      Win probability tracking requires MCTS bots with thinking data.
      Run a match with MCTS bots to see this graph.
    </div>
  </div>
{:else}
  <div bind:this={container} class="w-full"></div>
{/if}

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
