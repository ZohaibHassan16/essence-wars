<script lang="ts">
  interface Props {
    show: boolean;
    mode: "spectator" | "game";
    onClose: () => void;
  }

  let { show, mode, onClose }: Props = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" || event.key === "?") {
      onClose();
    }
  }

  interface ShortcutGroup {
    title: string;
    shortcuts: { key: string; description: string }[];
  }

  const spectatorShortcuts: ShortcutGroup[] = [
    {
      title: "Playback",
      shortcuts: [
        { key: "Space", description: "Play / Pause" },
        { key: "Arrow Left", description: "Step backward" },
        { key: "Arrow Right", description: "Step forward" },
        { key: "Home", description: "Jump to start" },
        { key: "End", description: "Jump to end" },
      ],
    },
    {
      title: "Views & Panels",
      shortcuts: [
        { key: "A", description: "Toggle Analysis mode" },
        { key: "L", description: "Open Action Log" },
        { key: "D", description: "Open Deck Library" },
      ],
    },
    {
      title: "General",
      shortcuts: [
        { key: "?", description: "Show this help" },
        { key: "Esc", description: "Close modals / Go back" },
      ],
    },
  ];

  const gameShortcuts: ShortcutGroup[] = [
    {
      title: "Gameplay",
      shortcuts: [
        { key: "Space", description: "End turn" },
        { key: "1-9", description: "Select card from hand" },
        { key: "Esc", description: "Cancel selection" },
      ],
    },
    {
      title: "Panels",
      shortcuts: [
        { key: "D", description: "Open Deck Library" },
      ],
    },
    {
      title: "General",
      shortcuts: [
        { key: "?", description: "Show this help" },
        { key: "Esc", description: "Close modals" },
      ],
    },
  ];

  const shortcuts = $derived(mode === "spectator" ? spectatorShortcuts : gameShortcuts);
</script>

<svelte:window onkeydown={handleKeydown} />

{#if show}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black/80 z-50 flex items-center justify-center p-4"
    onclick={onClose}
    onkeydown={(e) => e.key === "Enter" && onClose()}
    role="button"
    tabindex="0"
  >
    <!-- Modal -->
    <div
      class="bg-ui-panel border border-gray-700 rounded-lg shadow-2xl w-full max-w-md"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="shortcuts-title"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-gray-700">
        <div class="flex items-center gap-3">
          <span class="text-lg">⌨️</span>
          <h2 id="shortcuts-title" class="text-lg font-semibold text-ui-text">Keyboard Shortcuts</h2>
        </div>
        <button
          class="text-ui-text-dim hover:text-ui-text transition-colors p-1"
          onclick={onClose}
          aria-label="Close"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="p-4 space-y-4 max-h-[60vh] overflow-y-auto">
        {#each shortcuts as group}
          <div>
            <h3 class="text-xs font-semibold text-ui-text-dim uppercase tracking-wide mb-2">
              {group.title}
            </h3>
            <div class="space-y-1.5">
              {#each group.shortcuts as shortcut}
                <div class="flex items-center justify-between py-1">
                  <span class="text-sm text-ui-text">{shortcut.description}</span>
                  <kbd class="px-2 py-1 bg-gray-700 border border-gray-600 rounded text-xs text-ui-text font-mono min-w-[2rem] text-center">
                    {shortcut.key}
                  </kbd>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>

      <!-- Footer -->
      <div class="px-4 py-3 border-t border-gray-700 flex items-center justify-between">
        <div class="text-xs text-ui-text-dim">
          Press <kbd class="px-1.5 py-0.5 bg-gray-700 rounded text-ui-text">?</kbd> anytime to toggle
        </div>
        <button
          class="px-4 py-2 bg-gray-700 text-ui-text rounded-lg hover:bg-gray-600 transition-colors"
          onclick={onClose}
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
