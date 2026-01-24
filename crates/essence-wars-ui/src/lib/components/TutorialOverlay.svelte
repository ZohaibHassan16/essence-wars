<script lang="ts">
  import { tutorialStore } from '$lib/stores/tutorialState.svelte';
  import { gsap } from 'gsap';
  import { onMount } from 'svelte';

  // Spotlight target rectangle
  let spotlightRect = $state<DOMRect | null>(null);
  let containerEl: HTMLDivElement | undefined = $state();
  let tooltipEl: HTMLDivElement | undefined = $state();

  // Track and update spotlight position
  $effect(() => {
    const step = tutorialStore.currentStep;
    if (step?.targetElementId) {
      updateSpotlightPosition(step.targetElementId);
    } else {
      spotlightRect = null;
    }
  });

  function updateSpotlightPosition(targetId: string) {
    const el = document.querySelector(`[data-tutorial-id="${targetId}"]`);
    if (el) {
      spotlightRect = el.getBoundingClientRect();
    } else {
      spotlightRect = null;
    }
  }

  // Handle window resize
  onMount(() => {
    const handleResize = () => {
      const step = tutorialStore.currentStep;
      if (step?.targetElementId) {
        updateSpotlightPosition(step.targetElementId);
      }
    };

    window.addEventListener('resize', handleResize);

    // Animate in
    if (containerEl) {
      gsap.fromTo(
        containerEl,
        { opacity: 0 },
        { opacity: 1, duration: 0.3 }
      );
    }

    return () => {
      window.removeEventListener('resize', handleResize);
    };
  });

  // Compute tooltip position based on arrow direction and spotlight
  function getTooltipStyle(): string {
    const step = tutorialStore.currentStep;
    const tooltipWidth = 380;
    const tooltipHeight = 200;
    const padding = 24;

    if (!spotlightRect || !step) {
      // No target element - position at bottom center so board is visible
      // This is important for action-based steps where player needs to see the board
      const viewportHeight = window.innerHeight;
      return `left: 50%; top: ${viewportHeight - tooltipHeight - padding - 80}px; transform: translateX(-50%);`;
    }

    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    switch (step.arrowDirection) {
      case 'top':
        // Tooltip above the target
        return `
          left: ${Math.max(padding, Math.min(viewportWidth - tooltipWidth - padding, spotlightRect.x + spotlightRect.width / 2 - tooltipWidth / 2))}px;
          top: ${Math.max(padding, spotlightRect.y - tooltipHeight - padding)}px;
        `;
      case 'bottom':
        // Tooltip below the target
        return `
          left: ${Math.max(padding, Math.min(viewportWidth - tooltipWidth - padding, spotlightRect.x + spotlightRect.width / 2 - tooltipWidth / 2))}px;
          top: ${Math.min(viewportHeight - tooltipHeight - padding, spotlightRect.y + spotlightRect.height + padding)}px;
        `;
      case 'left':
        // Tooltip to the left
        return `
          left: ${Math.max(padding, spotlightRect.x - tooltipWidth - padding)}px;
          top: ${Math.max(padding, Math.min(viewportHeight - tooltipHeight - padding, spotlightRect.y + spotlightRect.height / 2 - tooltipHeight / 2))}px;
        `;
      case 'right':
        // Tooltip to the right
        return `
          left: ${Math.min(viewportWidth - tooltipWidth - padding, spotlightRect.x + spotlightRect.width + padding)}px;
          top: ${Math.max(padding, Math.min(viewportHeight - tooltipHeight - padding, spotlightRect.y + spotlightRect.height / 2 - tooltipHeight / 2))}px;
        `;
      default:
        // Default: position based on available space
        if (spotlightRect.y > viewportHeight / 2) {
          // Target is in lower half, show tooltip above
          return `
            left: ${Math.max(padding, Math.min(viewportWidth - tooltipWidth - padding, spotlightRect.x + spotlightRect.width / 2 - tooltipWidth / 2))}px;
            top: ${Math.max(padding, spotlightRect.y - tooltipHeight - padding)}px;
          `;
        } else {
          // Target is in upper half, show tooltip below
          return `
            left: ${Math.max(padding, Math.min(viewportWidth - tooltipWidth - padding, spotlightRect.x + spotlightRect.width / 2 - tooltipWidth / 2))}px;
            top: ${Math.min(viewportHeight - tooltipHeight - padding, spotlightRect.y + spotlightRect.height + padding)}px;
          `;
        }
    }
  }

  function getConditionHint(): string {
    const step = tutorialStore.currentStep;
    if (!step) return '';

    switch (step.advanceCondition.type) {
      case 'card_played':
        return 'Play a card to continue...';
      case 'creature_attacked':
        return 'Attack with a creature to continue...';
      case 'turn_ended':
        return 'End your turn to continue...';
      case 'action_performed':
        return `Perform an action to continue...`;
      case 'auto':
        return 'Please wait...';
      default:
        return '';
    }
  }

  function handleSkip() {
    tutorialStore.skipTutorial();
  }

  function handleNext() {
    tutorialStore.advanceStep();
  }
</script>

{#if tutorialStore.isActive && tutorialStore.isOverlayVisible}
  <div
    bind:this={containerEl}
    class="fixed inset-0 z-50 pointer-events-none"
  >
    <!-- SVG mask for spotlight effect -->
    <svg class="w-full h-full absolute inset-0">
      <defs>
        <mask id="tutorial-spotlight-mask">
          <!-- White = visible (dimmed) -->
          <rect width="100%" height="100%" fill="white" />
          <!-- Black = transparent (spotlight) -->
          {#if spotlightRect}
            <rect
              x={spotlightRect.x - 12}
              y={spotlightRect.y - 12}
              width={spotlightRect.width + 24}
              height={spotlightRect.height + 24}
              rx="12"
              fill="black"
            />
          {/if}
        </mask>
      </defs>
      <rect
        width="100%"
        height="100%"
        fill="rgba(0, 0, 0, 0.8)"
        mask="url(#tutorial-spotlight-mask)"
      />
    </svg>

    <!-- Spotlight border glow -->
    {#if spotlightRect}
      <div
        class="absolute rounded-xl ring-4 ring-ui-action animate-pulse pointer-events-none"
        style="
          left: {spotlightRect.x - 12}px;
          top: {spotlightRect.y - 12}px;
          width: {spotlightRect.width + 24}px;
          height: {spotlightRect.height + 24}px;
          box-shadow: 0 0 30px rgba(168, 85, 247, 0.5);
        "
      ></div>
    {/if}

    <!-- Tooltip panel -->
    <div
      bind:this={tooltipEl}
      class="absolute z-50 w-96 bg-ui-panel border-2 border-ui-action rounded-xl shadow-2xl pointer-events-auto"
      style={getTooltipStyle()}
    >
      <!-- Header with step indicator and skip -->
      <div class="flex items-center justify-between px-4 py-2 border-b border-gray-700">
        <span class="text-xs text-ui-text-dim font-medium">
          Step {tutorialStore.currentStepIndex + 1} of {tutorialStore.steps.length}
        </span>
        <button
          class="text-xs text-ui-text-dim hover:text-ui-text transition-colors"
          onclick={handleSkip}
        >
          Skip Tutorial
        </button>
      </div>

      <!-- Content -->
      <div class="p-4">
        <h3 class="text-lg font-bold text-ui-text mb-2">
          {tutorialStore.currentStep?.title}
        </h3>
        <p class="text-sm text-ui-text-dim leading-relaxed">
          {tutorialStore.currentStep?.message}
        </p>
      </div>

      <!-- Footer with progress and navigation -->
      <div class="flex items-center justify-between px-4 py-3 border-t border-gray-700 bg-ui-bg/30 rounded-b-xl">
        <!-- Progress dots -->
        <div class="flex gap-1.5">
          {#each tutorialStore.steps.slice(0, Math.min(12, tutorialStore.steps.length)) as _, i}
            <div
              class="w-2 h-2 rounded-full transition-colors {i < tutorialStore.currentStepIndex
                ? 'bg-ui-action'
                : i === tutorialStore.currentStepIndex
                  ? 'bg-ui-action animate-pulse'
                  : 'bg-gray-600'}"
            ></div>
          {/each}
          {#if tutorialStore.steps.length > 12}
            <span class="text-xs text-ui-text-dim">+{tutorialStore.steps.length - 12}</span>
          {/if}
        </div>

        <!-- Navigation -->
        {#if tutorialStore.currentStep?.advanceCondition.type === 'click_next'}
          <button
            class="px-5 py-2 bg-ui-action text-white rounded-lg font-semibold text-sm
                   hover:bg-ui-action/80 transition-all hover:scale-105 active:scale-95"
            onclick={handleNext}
          >
            {tutorialStore.currentStepIndex === tutorialStore.steps.length - 1 ? 'Finish' : 'Next'}
          </button>
        {:else}
          <span class="text-xs text-ui-text-dim italic">
            {getConditionHint()}
          </span>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  /* Ensure smooth animations */
  .animate-pulse {
    animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.7;
    }
  }
</style>
