<script lang="ts">
  let {
    current = 0,
    max = 0,
    isPlayer = true,
    tutorialId = undefined,
  }: {
    current: number;
    max: number;
    isPlayer?: boolean;
    tutorialId?: string;
  } = $props();

  // Total slots to show (always 10 for consistency)
  const totalSlots = 10;

  // Determine state for each gem
  function getGemState(index: number): 'available' | 'spent' | 'locked' {
    if (index < current) return 'available';
    if (index < max) return 'spent';
    return 'locked';
  }
</script>

<div
  class="flex flex-col gap-1 py-2"
  class:flex-col-reverse={!isPlayer}
  title="{current} / {max} Essence"
  data-tutorial-id={tutorialId}
>
  {#each Array(totalSlots) as _, i (i)}
    {@const state = getGemState(isPlayer ? i : totalSlots - 1 - i)}
    <div
      class="w-4 h-4 rounded-sm transition-all duration-200 relative"
      class:essence-available={state === 'available'}
      class:essence-spent={state === 'spent'}
      class:essence-locked={state === 'locked'}
    >
      <!-- Diamond/gem shape using CSS -->
      <div
        class="absolute inset-0.5 rotate-45 rounded-sm transition-all duration-200
               {state === 'available' ? 'bg-mana border border-mana shadow-[0_0_8px_rgba(59,130,246,0.8)]' : ''}
               {state === 'spent' ? 'bg-mana/20 border border-mana/40' : ''}
               {state === 'locked' ? 'bg-transparent border border-gray-700/30' : ''}"
      ></div>
    </div>
  {/each}
</div>

<style>
  /* Pulse animation for available essence */
  .essence-available {
    animation: essence-pulse 2s ease-in-out infinite;
  }

  @keyframes essence-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.85; }
  }
</style>
