<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    initialCollapsed = false,
    children,
  }: {
    initialCollapsed?: boolean;
    children: Snippet;
  } = $props();

  let isCollapsed = $state(initialCollapsed);

  function toggle() {
    isCollapsed = !isCollapsed;
  }
</script>

<div class="sidebar border-l border-gray-700 bg-ui-panel/50 flex flex-col h-full relative"
     class:collapsed={isCollapsed}
     class:expanded={!isCollapsed}>
  <!-- Toggle button -->
  <button
    class="absolute -left-3 top-1/2 -translate-y-1/2 w-6 h-12 bg-ui-panel border border-gray-600
           rounded-l-md flex items-center justify-center hover:bg-gray-700 transition-colors z-10"
    onclick={toggle}
    title={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
  >
    <svg
      class="w-4 h-4 text-ui-text-dim transition-transform duration-300"
      class:rotate-180={isCollapsed}
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
    </svg>
  </button>

  <!-- Collapsed icon bar -->
  {#if isCollapsed}
    <div class="flex flex-col items-center py-4 gap-4">
      <!-- Hint icon -->
      <button
        class="w-8 h-8 rounded bg-ui-panel hover:bg-gray-700 flex items-center justify-center
               text-ui-text-dim hover:text-ui-text transition-colors"
        onclick={toggle}
        title="Show hints"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
        </svg>
      </button>
      <!-- Log icon -->
      <button
        class="w-8 h-8 rounded bg-ui-panel hover:bg-gray-700 flex items-center justify-center
               text-ui-text-dim hover:text-ui-text transition-colors"
        onclick={toggle}
        title="Show action log"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
        </svg>
      </button>
    </div>
  {/if}

  <!-- Sidebar content -->
  <div class="sidebar-content flex flex-col h-full overflow-hidden">
    {@render children()}
  </div>
</div>
