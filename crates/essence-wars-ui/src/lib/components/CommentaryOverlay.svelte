<script lang="ts">
  import { spectatorStore } from '$lib/stores/spectatorState.svelte';
  import { gsap } from 'gsap';
  import { onMount } from 'svelte';

  let containerEl: HTMLDivElement | undefined = $state();

  const show = $derived(spectatorStore.showCommentaryOverlay);
  const commentary = $derived(spectatorStore.currentCommentary);

  // Auto-dismiss after 3 seconds
  $effect(() => {
    if (show) {
      const timer = setTimeout(() => {
        spectatorStore.dismissCommentaryOverlay();
      }, 3000);
      return () => clearTimeout(timer);
    }
  });

  // Animate in when shown
  $effect(() => {
    if (show && containerEl) {
      gsap.fromTo(
        containerEl,
        { opacity: 0, y: -20, scale: 0.95 },
        { opacity: 1, y: 0, scale: 1, duration: 0.3, ease: 'back.out(1.5)' }
      );
    }
  });

  function getIcon(type: string | undefined): string {
    switch (type) {
      case 'game_start': return '🎮';
      case 'first_blood': return '💀';
      case 'board_swing': return '⚡';
      case 'lethal_threat': return '⚠️';
      case 'game_end': return '🏆';
      case 'commander_played': return '👑';
      default: return '📊';
    }
  }

  function getMomentTitle(type: string | undefined): string {
    switch (type) {
      case 'game_start': return 'MATCH START';
      case 'first_blood': return 'FIRST BLOOD';
      case 'board_swing': return 'BOARD SWING';
      case 'lethal_threat': return 'DANGER';
      case 'game_end': return 'GAME OVER';
      case 'commander_played': return 'COMMANDER';
      default: return 'KEY MOMENT';
    }
  }

  function getBorderClass(type: string | undefined): string {
    switch (type) {
      case 'game_start': return 'border-ui-action';
      case 'first_blood': return 'border-damage';
      case 'board_swing': return 'border-yellow-500';
      case 'lethal_threat': return 'border-damage';
      case 'game_end': return 'border-health';
      default: return 'border-ui-action';
    }
  }

  function handleClick() {
    spectatorStore.dismissCommentaryOverlay();
  }
</script>

{#if show && commentary}
  <div
    bind:this={containerEl}
    class="fixed inset-x-0 top-20 flex justify-center z-40 pointer-events-none"
  >
    <button
      class="bg-ui-panel border-2 {getBorderClass(commentary.momentType)} rounded-xl shadow-2xl
             px-6 py-4 max-w-md pointer-events-auto cursor-pointer
             hover:scale-105 transition-transform"
      onclick={handleClick}
    >
      <div class="flex items-center gap-4">
        <span class="text-4xl">{getIcon(commentary.momentType)}</span>
        <div class="text-left">
          <h3 class="text-lg font-bold text-ui-text tracking-wide">
            {getMomentTitle(commentary.momentType)}
          </h3>
          <p class="text-sm text-ui-text-dim mt-1 leading-relaxed">{commentary.text}</p>
        </div>
      </div>
      <div class="text-xs text-ui-text-dim text-center mt-2 opacity-60">
        Click to dismiss
      </div>
    </button>
  </div>
{/if}
