<script lang="ts">
  import type { CreatureDto, AbilityDto } from "$lib/api/types";
  import { playSound } from "$lib/audio";
  import { gameStore } from "$lib/stores/gameState.svelte";

  let {
    creature,
    position,
  }: {
    creature: CreatureDto;
    position: { x: number; y: number };
  } = $props();

  // Check which actions are available
  const canAttack = $derived(
    creature && gameStore.selectedCreatureSlot !== null
      ? gameStore.hasAttackActions(gameStore.selectedCreatureSlot)
      : false
  );

  // Get usable abilities (those that have legal actions available)
  const usableAbilities = $derived.by((): AbilityDto[] => {
    if (!creature || gameStore.selectedCreatureSlot === null) return [];
    const slot = gameStore.selectedCreatureSlot;
    const abilityActions = gameStore.getAbilityActions(slot);
    const usableIndices = new Set(
      abilityActions.map(a => a.abilityIndex).filter(i => i !== undefined)
    );
    return creature.abilities.filter(a => usableIndices.has(a.index));
  });

  function handleAttack() {
    playSound('buttonClick');
    gameStore.selectAttackMode();
  }

  function handleAbility(abilityIndex: number) {
    playSound('buttonClick');
    gameStore.selectAbility(abilityIndex);
  }

  function handleClose() {
    playSound('menuClose');
    gameStore.clearSelection();
  }

  // Close menu when clicking outside
  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      handleClose();
    }
  }

  // Keyboard handler
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      handleClose();
    }
  }

  // Calculate menu position (keep it on screen)
  const menuStyle = $derived.by(() => {
    const menuWidth = 200;
    const menuHeight = 150;
    let x = position.x;
    let y = position.y;

    // Adjust if too close to right edge
    if (typeof window !== 'undefined' && x + menuWidth > window.innerWidth - 20) {
      x = window.innerWidth - menuWidth - 20;
    }
    // Adjust if too close to bottom
    if (typeof window !== 'undefined' && y + menuHeight > window.innerHeight - 20) {
      y = y - menuHeight - 10;
    }

    return `left: ${x}px; top: ${y}px;`;
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Backdrop to catch clicks outside menu -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-40"
  onclick={handleBackdropClick}
>
  <!-- Menu -->
  <div
    class="fixed z-50 bg-gray-900/95 border border-gray-600 rounded-lg shadow-2xl
           min-w-[180px] overflow-hidden backdrop-blur-sm"
    style={menuStyle}
  >
    <!-- Header -->
    <div class="px-3 py-2 bg-gray-800/50 border-b border-gray-700 flex items-center justify-between">
      <span class="text-sm font-semibold text-ui-text truncate">{creature.name}</span>
      <button
        class="text-gray-400 hover:text-white transition-colors"
        onclick={handleClose}
        title="Close (Esc)"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <!-- Actions -->
    <div class="p-1">
      <!-- Attack option -->
      {#if canAttack}
        <button
          class="w-full text-left px-3 py-2 rounded-md hover:bg-gray-700/50 flex items-center gap-3
                 transition-colors group"
          onclick={handleAttack}
        >
          <span class="text-red-400 group-hover:scale-110 transition-transform text-lg">⚔</span>
          <div class="flex-1">
            <div class="text-sm font-medium text-ui-text">Attack</div>
            <div class="text-xs text-gray-400">Deal combat damage</div>
          </div>
        </button>
      {/if}

      <!-- Divider if both attack and abilities available -->
      {#if canAttack && usableAbilities.length > 0}
        <div class="h-px bg-gray-700 mx-2 my-1"></div>
      {/if}

      <!-- Ability options -->
      {#each usableAbilities as ability}
        <button
          class="w-full text-left px-3 py-2 rounded-md hover:bg-purple-900/30 flex items-center gap-3
                 transition-colors group"
          onclick={() => handleAbility(ability.index)}
        >
          <span class="text-purple-400 group-hover:scale-110 transition-transform text-lg">✦</span>
          <div class="flex-1 min-w-0">
            <div class="text-sm font-medium text-ui-text truncate">{ability.name}</div>
            <div class="text-xs text-gray-400 truncate">{ability.description}</div>
          </div>
          {#if ability.essenceCost > 0}
            <div class="flex items-center gap-0.5 text-blue-400 text-sm font-medium shrink-0">
              <span>{ability.essenceCost}</span>
              <span class="text-xs">⬡</span>
            </div>
          {/if}
        </button>
      {/each}

      <!-- No actions message -->
      {#if !canAttack && usableAbilities.length === 0}
        <div class="px-3 py-2 text-gray-500 text-sm text-center">
          No actions available
        </div>
      {/if}
    </div>
  </div>
</div>
