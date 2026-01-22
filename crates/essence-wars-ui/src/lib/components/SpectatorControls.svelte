<script lang="ts">
  import { spectatorStore } from "$lib/stores/spectatorState.svelte";

  const speeds = [0.25, 0.5, 1, 2, 4];

  function formatSpeed(speed: number): string {
    return speed === 1 ? "1x" : `${speed}x`;
  }
</script>

<div class="bg-ui-panel px-4 py-3">
  <div class="flex items-center justify-between gap-4">
    <!-- Left: Turn/Action info -->
    <div class="flex items-center gap-4 text-sm">
      <div class="text-ui-text-dim">
        Turn <span class="text-ui-text font-semibold">{spectatorStore.currentTurn}</span>
      </div>
      <div class="text-ui-text-dim">
        Action <span class="text-ui-text font-semibold">{spectatorStore.currentActionIndex + 1}</span>
        <span class="text-ui-text-dim">/ {spectatorStore.totalActions}</span>
      </div>
      {#if spectatorStore.currentAction}
        <div class="text-ui-text-dim">
          Player <span class="font-semibold {spectatorStore.currentAction.player === 1 ? 'text-health' : 'text-damage'}">
            {spectatorStore.currentAction.player}
          </span>
        </div>
      {/if}
    </div>

    <!-- Center: Playback controls -->
    <div class="flex items-center gap-1">
      <!-- Jump to start -->
      <button
        class="w-8 h-8 rounded bg-ui-bg border border-gray-600 text-ui-text text-sm
               hover:border-ui-action hover:text-ui-action transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed"
        onclick={() => spectatorStore.jumpToStart()}
        disabled={spectatorStore.isAtStart}
        title="Jump to start"
      >
        &#9198;
      </button>

      <!-- Step backward -->
      <button
        class="w-8 h-8 rounded bg-ui-bg border border-gray-600 text-ui-text text-sm
               hover:border-ui-action hover:text-ui-action transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed"
        onclick={() => spectatorStore.stepBackward()}
        disabled={spectatorStore.isAtStart}
        title="Step backward"
      >
        &#9194;
      </button>

      <!-- Play/Pause -->
      <button
        class="w-10 h-10 rounded-lg bg-ui-action text-white text-lg
               hover:bg-ui-action/80 transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed"
        onclick={() => spectatorStore.isPlaying ? spectatorStore.pause() : spectatorStore.play()}
        disabled={spectatorStore.isAtEnd && !spectatorStore.isPlaying}
        title={spectatorStore.isPlaying ? "Pause" : "Play"}
      >
        {#if spectatorStore.isPlaying}
          &#9208;
        {:else}
          &#9654;
        {/if}
      </button>

      <!-- Step forward -->
      <button
        class="w-8 h-8 rounded bg-ui-bg border border-gray-600 text-ui-text text-sm
               hover:border-ui-action hover:text-ui-action transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed"
        onclick={() => spectatorStore.stepForward()}
        disabled={spectatorStore.isAtEnd}
        title="Step forward"
      >
        &#9193;
      </button>

      <!-- Jump to end -->
      <button
        class="w-8 h-8 rounded bg-ui-bg border border-gray-600 text-ui-text text-sm
               hover:border-ui-action hover:text-ui-action transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed"
        onclick={() => spectatorStore.jumpToEnd()}
        disabled={spectatorStore.isAtEnd}
        title="Jump to end"
      >
        &#9197;
      </button>
    </div>

    <!-- Right: Speed controls + Timeline -->
    <div class="flex items-center gap-3">
      <!-- Speed buttons -->
      <div class="flex items-center gap-1">
        <span class="text-xs text-ui-text-dim mr-1">Speed:</span>
        {#each speeds as speed}
          <button
            class="px-2 py-0.5 rounded text-xs font-semibold transition-colors
                   {spectatorStore.playbackSpeed === speed
                     ? 'bg-ui-action text-white'
                     : 'bg-ui-bg border border-gray-600 text-ui-text-dim hover:border-ui-action hover:text-ui-action'}"
            onclick={() => spectatorStore.setSpeed(speed)}
          >
            {formatSpeed(speed)}
          </button>
        {/each}
      </div>

      <!-- Timeline scrubber (only when allowed) -->
      {#if spectatorStore.canShowTimeline}
        <input
          type="range"
          min="-1"
          max={spectatorStore.totalActions - 1}
          value={spectatorStore.currentActionIndex}
          oninput={(e) => spectatorStore.jumpToAction(parseInt(e.currentTarget.value))}
          class="w-32 h-1.5 bg-ui-bg rounded-lg appearance-none cursor-pointer
                 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3
                 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-ui-action
                 [&::-webkit-slider-thumb]:cursor-pointer [&::-webkit-slider-thumb]:hover:bg-ui-action/80"
        />
      {:else}
        <span class="text-xs text-ui-text-dim">(Watch Live)</span>
      {/if}
    </div>
  </div>
</div>
