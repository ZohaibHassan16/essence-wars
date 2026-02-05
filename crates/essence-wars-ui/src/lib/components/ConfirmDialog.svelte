<script lang="ts">
  import { playSound } from "$lib/audio";

  interface Props {
    show: boolean;
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    confirmVariant?: "danger" | "primary";
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    show,
    title,
    message,
    confirmText = "Confirm",
    cancelText = "Cancel",
    confirmVariant = "primary",
    onConfirm,
    onCancel,
  }: Props = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onCancel();
    } else if (event.key === "Enter") {
      onConfirm();
    }
  }

  const confirmButtonClass = $derived(
    confirmVariant === "danger"
      ? "bg-damage hover:bg-damage/80"
      : "bg-ui-action hover:bg-ui-action/80"
  );
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
      aria-labelledby="confirm-dialog-title"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="px-5 py-4 border-b border-gray-700">
        <h2 id="confirm-dialog-title" class="text-lg font-semibold text-ui-text">{title}</h2>
      </div>

      <!-- Content -->
      <div class="px-5 py-4">
        <p class="text-ui-text-dim">{message}</p>
      </div>

      <!-- Actions -->
      <div class="px-5 py-4 border-t border-gray-700 flex items-center justify-end gap-3">
        <button
          class="px-4 py-2 bg-gray-700 text-ui-text rounded-lg font-semibold
                 hover:bg-gray-600 transition-colors"
          onclick={() => {
            playSound('buttonClick');
            onCancel();
          }}
        >
          {cancelText}
        </button>
        <button
          class="px-4 py-2 {confirmButtonClass} text-white rounded-lg font-semibold transition-colors"
          onclick={() => {
            playSound('buttonClick');
            onConfirm();
          }}
        >
          {confirmText}
        </button>
      </div>
    </div>
  </div>
{/if}
