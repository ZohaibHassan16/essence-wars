<script lang="ts">
  import type { CreatureDto, SupportDto } from "$lib/api/types";
  import CreatureSlot from "../CreatureSlot.svelte";
  import SupportSlot from "../SupportSlot.svelte";
  import { padCreatures, padSupports } from "$lib/utils/arrays";

  let {
    creatures,
    supports,
    isPlayerSide,
    selectedCreatureSlot = null,
    highlightedSlots = [],
    validAttackTargets = [],
    onCreatureClick,
    onSupportClick,
    showKeyHints = false,
  }: {
    creatures: (CreatureDto | null)[];
    supports: (SupportDto | null)[];
    isPlayerSide: boolean;
    selectedCreatureSlot?: number | null;
    highlightedSlots?: number[];
    validAttackTargets?: number[];
    onCreatureClick?: (slot: number) => void;
    onSupportClick?: (slot: number) => void;
    showKeyHints?: boolean;
  } = $props();

  // Ensure arrays always have exactly the expected number of elements
  const paddedCreatures = $derived(padCreatures(creatures));
  const paddedSupports = $derived(padSupports(supports));
</script>

<div class="battlefield-row">
  <!-- Left support (S1) -->
  <div class="flex-shrink-0">
    <SupportSlot
      support={paddedSupports[0]}
      slot={0}
      isHighlighted={highlightedSlots.includes(0)}
      onClick={onSupportClick ? () => onSupportClick(0) : undefined}
    />
  </div>

  <!-- Creatures row (C1-C5) -->
  <div class="creature-row">
    {#each paddedCreatures as creature, i}
      <CreatureSlot
        {creature}
        slot={i}
        {isPlayerSide}
        isHighlighted={highlightedSlots.includes(i)}
        isSelected={selectedCreatureSlot === i}
        isValidTarget={validAttackTargets.includes(i)}
        onClick={onCreatureClick ? () => onCreatureClick(i) : undefined}
        showKeyHint={showKeyHints}
      />
    {/each}
  </div>

  <!-- Right support (S2) -->
  <div class="flex-shrink-0">
    <SupportSlot
      support={paddedSupports[1]}
      slot={1}
      isHighlighted={highlightedSlots.includes(1)}
      onClick={onSupportClick ? () => onSupportClick(1) : undefined}
    />
  </div>
</div>
