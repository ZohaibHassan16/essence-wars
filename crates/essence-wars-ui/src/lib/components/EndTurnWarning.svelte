<script lang="ts">
  import { playSound } from "$lib/audio";

  interface Props {
    show: boolean;
    actionPoints: number;
    playableCards: number;
    onConfirm: () => void;
    onCancel: () => void;
    onDisableWarning: () => void;
  }

  let {
    show,
    actionPoints,
    playableCards,
    onConfirm,
    onCancel,
    onDisableWarning,
  }: Props = $props();

  let dontAskAgain = $state(false);

  function handleConfirm() {
    if (dontAskAgain) {
      onDisableWarning();
    }
    onConfirm();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onCancel();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if show}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4"
    onclick={onCancel}
    role="button"
    tabindex="0"
    onkeydown={(e) => e.key === "Enter" && onCancel()}
  >
    <!-- Dialog -->
    <div
      class="bg-ui-panel border border-gray-700 rounded-lg shadow-2xl w-full max-w-sm"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="end-turn-warning-title"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="px-5 py-4 border-b border-gray-700 flex items-center gap-3">
        <span class="text-2xl">⚠️</span>
        <h2 id="end-turn-warning-title" class="text-lg font-semibold text-ui-text">End Turn?</h2>
      </div>

      <!-- Content -->
      <div class="px-5 py-4">
        <p class="text-ui-text-dim mb-4">You still have actions available:</p>

        <div class="space-y-2">
          {#if actionPoints > 0}
            <div class="flex items-center gap-3 p-3 rounded-lg bg-gold/10 border border-gold/30">
              <span class="text-gold text-lg">⚡</span>
              <div>
                <div class="text-ui-text font-medium">{actionPoints} Action Point{actionPoints > 1 ? 's' : ''}</div>
                <div class="text-xs text-ui-text-dim">Unused AP will be lost</div>
              </div>
            </div>
          {/if}

          {#if playableCards > 0}
            <div class="flex items-center gap-3 p-3 rounded-lg bg-mana/10 border border-mana/30">
              <span class="text-mana text-lg">🃏</span>
              <div>
                <div class="text-ui-text font-medium">{playableCards} Playable Card{playableCards > 1 ? 's' : ''}</div>
                <div class="text-xs text-ui-text-dim">You can still play cards</div>
              </div>
            </div>
          {/if}
        </div>

        <!-- Don't ask again checkbox -->
        <label class="flex items-center gap-2 mt-4 cursor-pointer group">
          <input
            type="checkbox"
            bind:checked={dontAskAgain}
            class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-ui-action focus:ring-ui-action cursor-pointer"
          />
          <span class="text-sm text-ui-text-dim group-hover:text-ui-text transition-colors">
            Don't show this warning again (this game)
          </span>
        </label>
      </div>

      <!-- Actions -->
      <div class="px-5 py-4 border-t border-gray-700 flex items-center justify-end gap-3">
        <button
          class="px-4 py-2 bg-ui-action text-white rounded-lg font-semibold
                 hover:bg-ui-action/80 transition-colors"
          onclick={() => {
            playSound('buttonClick');
            onCancel();
          }}
        >
          Keep Playing
        </button>
        <button
          class="px-4 py-2 bg-gray-700 text-ui-text rounded-lg font-semibold
                 hover:bg-gray-600 transition-colors"
          onclick={() => {
            playSound('buttonClick');
            handleConfirm();
          }}
        >
          End Turn Anyway
        </button>
      </div>
    </div>
  </div>
{/if}
