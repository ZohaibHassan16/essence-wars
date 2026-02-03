<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    stepNumber,
    totalSteps = 3,
    title,
    subtitle = "",
    children,
    footer,
  }: {
    stepNumber: number;
    totalSteps?: number;
    title: string;
    subtitle?: string;
    children: Snippet;
    footer?: Snippet;
  } = $props();
</script>

<div class="wizard-step flex flex-col h-full">
  <!-- Header -->
  <div class="text-center py-6 border-b border-gray-700/50">
    <!-- Step indicator -->
    <div class="flex items-center justify-center gap-2 mb-3">
      {#each Array(totalSteps) as _, i (i)}
        {@const stepNum = i + 1}
        {@const isActive = stepNum === stepNumber}
        {@const isComplete = stepNum < stepNumber}
        <div class="flex items-center">
          <!-- Step dot -->
          <div
            class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold transition-all duration-300
                   {isActive
                     ? 'bg-ui-action text-white scale-110'
                     : isComplete
                       ? 'bg-health text-white'
                       : 'bg-gray-700 text-ui-text-dim'}"
          >
            {#if isComplete}
              <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
            {:else}
              {stepNum}
            {/if}
          </div>

          <!-- Connector line (not after last step) -->
          {#if stepNum < totalSteps}
            <div
              class="w-12 h-0.5 mx-1 transition-all duration-300
                     {isComplete ? 'bg-health' : 'bg-gray-700'}"
            ></div>
          {/if}
        </div>
      {/each}
    </div>

    <!-- Title -->
    <h1 class="text-2xl font-bold text-ui-text">
      {title}
    </h1>

    <!-- Subtitle -->
    {#if subtitle}
      <p class="text-sm text-ui-text-dim mt-1">
        {subtitle}
      </p>
    {/if}
  </div>

  <!-- Content area -->
  <div class="flex-1 overflow-auto">
    {@render children()}
  </div>

  <!-- Footer (navigation buttons) -->
  {#if footer}
    <div class="border-t border-gray-700/50 p-4 bg-ui-panel/50">
      {@render footer()}
    </div>
  {/if}
</div>

<style>
  .wizard-step {
    animation: fadeIn 0.3s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
</style>
