<script lang="ts">
  import type { ActionInfo, GameEventDto } from "$lib/api/types";

  let {
    actions = [],
    events = [],
    maxItems = 10,
  }: {
    actions?: ActionInfo[];
    events?: GameEventDto[];
    maxItems?: number;
  } = $props();

  function getActionIcon(actionType: string): string {
    switch (actionType) {
      case "play_card": return "🃏";
      case "attack": return "⚔️";
      case "use_ability": return "✨";
      case "end_turn": return "⏭️";
      default: return "•";
    }
  }

  function getEventIcon(eventType: string): string {
    switch (eventType) {
      case "creature_spawned": return "🐣";
      case "creature_died": return "💀";
      case "life_changed": return "❤️";
      case "turn_started": return "🔄";
      case "turn_ended": return "⏸️";
      case "game_ended": return "🏆";
      default: return "📝";
    }
  }

  function formatEvent(event: GameEventDto): string {
    const data = event.data;
    switch (event.eventType) {
      case "creature_spawned":
        return `P${data.player} summoned creature at slot ${data.slot}`;
      case "creature_died":
        return `Creature died at P${data.player} slot ${data.slot}`;
      case "life_changed":
        return `P${data.player} life: ${data.old} → ${data.new}`;
      case "turn_started":
        return `Turn ${data.turn} - Player ${data.player}`;
      case "game_ended":
        return data.winner ? `Player ${data.winner} wins!` : "Draw!";
      default:
        return event.eventType;
    }
  }

  const recentActions = $derived(actions.slice(-maxItems).reverse());
</script>

<div class="bg-ui-panel/80 rounded-lg border border-gray-700 overflow-hidden">
  <div class="px-3 py-2 border-b border-gray-700 bg-gray-800/50">
    <h3 class="text-sm font-semibold text-ui-text">Action Log</h3>
  </div>

  <div class="max-h-48 overflow-y-auto">
    {#if recentActions.length === 0}
      <div class="px-3 py-4 text-ui-text-dim text-sm text-center">
        No actions yet
      </div>
    {:else}
      <div class="divide-y divide-gray-700/50">
        {#each recentActions as action, i}
          <div class="px-3 py-2 hover:bg-gray-700/30 transition-colors">
            <div class="flex items-start gap-2">
              <span class="text-base">{getActionIcon(action.actionType)}</span>
              <div class="flex-1 min-w-0">
                <div class="text-sm text-ui-text truncate">
                  {action.description}
                </div>
                <div class="text-[10px] text-ui-text-dim capitalize">
                  {action.actionType.replace("_", " ")}
                </div>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
