<script lang="ts">
  import {
    Zap,
    Target,
    Swords,
    Shield,
    Droplet,
    Skull,
    ShieldCheck,
    ChevronsRight,
    Ghost,
    HeartPulse,
    EyeOff,
    BatteryCharging,
    Flame,
    Bomb
  } from "lucide-svelte";

  interface Props {
    keyword: string;
    size?: number;
    showLabel?: boolean;
  }

  let { keyword, size = 14, showLabel = true }: Props = $props();

  // Map keywords to icons and colors
  const keywordConfig: Record<string, { icon: typeof Zap; color: string; description: string }> = {
    Rush: {
      icon: Zap,
      color: "text-yellow-400",
      description: "Can attack immediately when played"
    },
    Ranged: {
      icon: Target,
      color: "text-blue-400",
      description: "Can attack any enemy creature"
    },
    Piercing: {
      icon: Swords,
      color: "text-orange-400",
      description: "Excess damage hits the enemy player"
    },
    Guard: {
      icon: Shield,
      color: "text-cyan-400",
      description: "Must be attacked before other creatures"
    },
    Lifesteal: {
      icon: Droplet,
      color: "text-red-400",
      description: "Heals your hero equal to damage dealt"
    },
    Lethal: {
      icon: Skull,
      color: "text-purple-400",
      description: "Destroys any creature it damages"
    },
    Shield: {
      icon: ShieldCheck,
      color: "text-blue-300",
      description: "Blocks the first damage taken"
    },
    Quick: {
      icon: ChevronsRight,
      color: "text-green-400",
      description: "Can attack twice per turn"
    },
    Ephemeral: {
      icon: Ghost,
      color: "text-gray-400",
      description: "Dies at end of turn"
    },
    Regenerate: {
      icon: HeartPulse,
      color: "text-emerald-400",
      description: "Heals 1 HP at start of your turn"
    },
    Stealth: {
      icon: EyeOff,
      color: "text-violet-400",
      description: "Cannot be targeted until it attacks"
    },
    Charge: {
      icon: BatteryCharging,
      color: "text-amber-400",
      description: "Gains +1 attack each turn"
    },
    Frenzy: {
      icon: Flame,
      color: "text-orange-500",
      description: "Gains +1 attack when damaged"
    },
    Volatile: {
      icon: Bomb,
      color: "text-red-500",
      description: "Deals damage to all when it dies"
    }
  };

  const config = $derived(keywordConfig[keyword] || { icon: Zap, color: "text-gray-400", description: "Unknown keyword" });
  const IconComponent = $derived(config.icon);
</script>

<span
  class="inline-flex items-center gap-1 rounded text-xs bg-ui-panel border border-gray-600 {config.color} {showLabel ? 'px-1.5 py-0.5' : 'p-0.5'}"
  title="{keyword}: {config.description}"
>
  <IconComponent {size} />
  {#if showLabel}
    <span class="text-ui-text">{keyword}</span>
  {/if}
</span>
