<script lang="ts">
  import type { SpectatorMatch } from "$lib/api/types";
  import type { MatchStatistics } from "$lib/stats/types";
  import { downloadJson, downloadCsv } from "$lib/stats/export";
  import { comparisonStore } from "$lib/stores/comparisonState.svelte";
  import StatsOverviewPanel from "./panels/StatsOverviewPanel.svelte";
  import StatsActionPanel from "./panels/StatsActionPanel.svelte";
  import StatsCombatPanel from "./panels/StatsCombatPanel.svelte";
  import StatsResourcePanel from "./panels/StatsResourcePanel.svelte";
  import StatsKeywordPanel from "./panels/StatsKeywordPanel.svelte";
  import StatsAiPanel from "./panels/StatsAiPanel.svelte";
  import StatsTimelineTabs from "./StatsTimelineTabs.svelte";

  interface Props {
    match: SpectatorMatch;
    statistics: MatchStatistics;
    onClose: () => void;
  }

  let { match, statistics, onClose }: Props = $props();

  // Tab state
  type TabId = "overview" | "action" | "combat" | "resources" | "keywords" | "ai" | "timeline";
  let activeTab = $state<TabId>("overview");

  const tabs: { id: TabId; label: string; icon: string }[] = [
    { id: "overview", label: "Overview", icon: "trophy" },
    { id: "action", label: "Actions", icon: "zap" },
    { id: "combat", label: "Combat", icon: "swords" },
    { id: "resources", label: "Resources", icon: "coins" },
    { id: "keywords", label: "Keywords", icon: "sparkles" },
    { id: "ai", label: "AI Analysis", icon: "brain" },
    { id: "timeline", label: "Timeline", icon: "chart" },
  ];

  // Export dropdown state
  let showExportMenu = $state(false);

  async function handleExportJson() {
    await downloadJson(match, statistics);
    showExportMenu = false;
  }

  async function handleExportCsv() {
    await downloadCsv(match, statistics);
    showExportMenu = false;
  }

  async function handleCompare() {
    // Load this match into slot 1 and start comparison mode
    comparisonStore.loadFromSpectatorMatch(1, match);
    await comparisonStore.startComparison();
    onClose();
  }

  // Handle escape key
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }

  // Close export menu when clicking outside
  function handleClickOutside(e: MouseEvent) {
    if (showExportMenu) {
      showExportMenu = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Modal backdrop -->
<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm"
  onclick={handleClickOutside}
>
  <!-- Modal content -->
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="bg-ui-panel border border-gray-700 rounded-xl shadow-2xl w-[90vw] max-w-5xl h-[85vh] flex flex-col"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-6 py-4 border-b border-gray-700">
      <div class="flex items-center gap-3">
        <span class="text-2xl">📊</span>
        <h2 class="text-xl font-bold text-ui-text">Match Statistics</h2>
      </div>

      <div class="flex items-center gap-3">
        <!-- Compare button -->
        <button
          class="px-4 py-2 bg-purple-600/20 text-purple-400 rounded-lg font-semibold text-sm
                 border border-purple-500/50 hover:bg-purple-600 hover:text-white transition-colors flex items-center gap-2"
          onclick={handleCompare}
          title="Compare with another match"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
          </svg>
          Compare
        </button>

        <!-- Export dropdown -->
        <div class="relative">
          <button
            class="px-4 py-2 bg-gray-700 text-ui-text rounded-lg font-semibold text-sm
                   hover:bg-gray-600 transition-colors flex items-center gap-2"
            onclick={(e) => {
              e.stopPropagation();
              showExportMenu = !showExportMenu;
            }}
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            Export
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>

          {#if showExportMenu}
            <div class="absolute right-0 mt-1 bg-gray-800 border border-gray-600 rounded-lg shadow-xl z-10 min-w-[140px]">
              <button
                class="w-full px-4 py-2 text-left text-sm text-ui-text hover:bg-gray-700 rounded-t-lg"
                onclick={handleExportJson}
              >
                JSON
              </button>
              <button
                class="w-full px-4 py-2 text-left text-sm text-ui-text hover:bg-gray-700 rounded-b-lg"
                onclick={handleExportCsv}
              >
                CSV
              </button>
            </div>
          {/if}
        </div>

        <!-- Close button -->
        <button
          class="p-2 text-ui-text-dim hover:text-ui-text hover:bg-gray-700 rounded-lg transition-colors"
          onclick={onClose}
          title="Close (Esc)"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Tab navigation + Content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Vertical tab sidebar -->
      <div class="w-40 border-r border-gray-700 bg-gray-900/50 flex flex-col py-2">
        {#each tabs as tab}
          <button
            class="px-4 py-3 text-left text-sm font-medium transition-colors
                   {activeTab === tab.id
                     ? 'bg-ui-action/20 text-ui-action border-r-2 border-ui-action'
                     : 'text-ui-text-dim hover:text-ui-text hover:bg-gray-800'}"
            onclick={() => (activeTab = tab.id)}
          >
            <span class="flex items-center gap-2">
              {#if tab.icon === "trophy"}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              {:else if tab.icon === "zap"}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
              {:else if tab.icon === "swords"}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
                </svg>
              {:else if tab.icon === "coins"}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              {:else if tab.icon === "sparkles"}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
                </svg>
              {:else if tab.icon === "brain"}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
                </svg>
              {:else if tab.icon === "chart"}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
              {/if}
              {tab.label}
            </span>
          </button>
        {/each}
      </div>

      <!-- Content area -->
      <div class="flex-1 overflow-auto p-6">
        {#if activeTab === "overview"}
          <StatsOverviewPanel {match} {statistics} />
        {:else if activeTab === "action"}
          <StatsActionPanel {statistics} />
        {:else if activeTab === "combat"}
          <StatsCombatPanel {statistics} />
        {:else if activeTab === "resources"}
          <StatsResourcePanel {statistics} />
        {:else if activeTab === "keywords"}
          <StatsKeywordPanel {statistics} />
        {:else if activeTab === "ai"}
          <StatsAiPanel {statistics} />
        {:else if activeTab === "timeline"}
          <StatsTimelineTabs {statistics} />
        {/if}
      </div>
    </div>
  </div>
</div>
