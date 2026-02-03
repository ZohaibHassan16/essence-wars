<script lang="ts">
  import type { CommanderDto } from "$lib/api/types";

  let {
    commander,
    isPlayer: _isPlayer = false,
    isActive = false,
    tutorialId = undefined,
  }: {
    commander: CommanderDto | null;
    isPlayer?: boolean;
    isActive?: boolean;
    tutorialId?: string;
  } = $props();

  // Get faction-specific styling classes
  const factionGradient = $derived(() => {
    if (!commander) return 'from-gray-800 to-gray-900';
    switch (commander.faction) {
      case 'argentum':
        return 'from-argentum-brass/30 to-argentum-gold/10';
      case 'symbiote':
        return 'from-symbiote-primary/40 to-symbiote-purple/20';
      case 'obsidion':
        return 'from-obsidion-primary/30 to-obsidion-black/40';
      case 'neutral':
        return 'from-neutral-primary/30 to-neutral-tan/10';
      default:
        return 'from-gray-800 to-gray-900';
    }
  });

  const factionBorder = $derived(() => {
    if (!commander) return 'border-gray-600';
    switch (commander.faction) {
      case 'argentum':
        return 'border-argentum-gold/60';
      case 'symbiote':
        return 'border-symbiote-glow/40';
      case 'obsidion':
        return 'border-obsidion-essence/40';
      case 'neutral':
        return 'border-neutral-copper/50';
      default:
        return 'border-gray-600';
    }
  });

  const factionGlow = $derived(() => {
    if (!commander || !isActive) return '';
    switch (commander.faction) {
      case 'argentum':
        return 'shadow-[0_0_15px_rgba(212,175,55,0.4)]';
      case 'symbiote':
        return 'shadow-[0_0_15px_rgba(127,255,0,0.3)]';
      case 'obsidion':
        return 'shadow-[0_0_15px_rgba(0,255,255,0.3)]';
      case 'neutral':
        return 'shadow-[0_0_15px_rgba(184,115,51,0.3)]';
      default:
        return '';
    }
  });
</script>

<div
  class="flex items-center gap-3 px-3 py-2 rounded-lg border bg-gradient-to-r transition-all duration-300
         {factionGradient()} {factionBorder()} {factionGlow()}"
  class:opacity-50={!commander}
  title={commander?.abilityDescription ?? 'No commander'}
  data-tutorial-id={tutorialId}
>
  {#if commander}
    <!-- Commander Portrait -->
    <div class="relative w-12 h-12 rounded-lg overflow-hidden border-2 {factionBorder()} shrink-0">
      <img
        src={`/${commander.portraitPath}`}
        alt={commander.name}
        class="w-full h-full object-cover object-top"
        style="object-position: center 15%;"
      />
      {#if isActive}
        <div class="absolute inset-0 ring-2 ring-white/30 rounded-lg animate-pulse"></div>
      {/if}
    </div>

    <!-- Commander Info -->
    <div class="flex flex-col min-w-0">
      <span class="text-sm font-semibold text-ui-text truncate">
        {commander.name}
      </span>
      <span class="text-xs text-ui-text-dim truncate max-w-[180px]" title={commander.abilityDescription}>
        {commander.abilityDescription}
      </span>
    </div>
  {:else}
    <!-- Placeholder when no commander -->
    <div class="w-12 h-12 rounded-lg bg-gray-700/50 border border-gray-600 shrink-0 flex items-center justify-center">
      <span class="text-gray-500 text-xs">?</span>
    </div>
    <div class="flex flex-col">
      <span class="text-sm text-ui-text-dim">No Commander</span>
    </div>
  {/if}
</div>
