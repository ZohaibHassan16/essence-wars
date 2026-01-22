<script lang="ts">
  import { replayStore } from "$lib/stores/replayState.svelte";

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
        Turn <span class="text-ui-text font-semibold">{replayStore.currentTurn}</span>
      </div>
      <div class="text-ui-text-dim">
        Action <span class="text-ui-text font-semibold">{replayStore.currentActionIndex + 1}</span>
        <span class="text-ui-text-dim">/ {replayStore.totalActions}</span>
      </div>
      {#if replayStore.currentAction}
        <div class="text-ui-text-dim">
          Player <span class="font-semibold {replayStore.currentAction.player === 1 ? 'text-health' : 'text-damage'}">
            {replayStore.currentAction.player}
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
        onclick={() => replayStore.jumpToStart()}
        disabled={replayStore.isAtStart}
        title="Jump to start"
      >
        &#9198;
      </button>

      <!-- Step backward -->
      <button
        class="w-8 h-8 rounded bg-ui-bg border border-gray-600 text-ui-text text-sm
               hover:border-ui-action hover:text-ui-action transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed"
        onclick={() => replayStore.stepBackward()}
        disabled={replayStore.isAtStart}
        title="Step backward"
      >
        &#9194;
      </button>

      <!-- Play/Pause -->
      <button
        class="w-10 h-10 rounded-lg bg-ui-action text-white text-lg
               hover:bg-ui-action/80 transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed"
        onclick={() => replayStore.isPlaying ? replayStore.pause() : replayStore.play()}
        disabled={replayStore.isAtEnd && !replayStore.isPlaying}
        title={replayStore.isPlaying ? "Pause" : "Play"}
      >
        {#if replayStore.isPlaying}
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
        onclick={() => replayStore.stepForward()}
        disabled={replayStore.isAtEnd}
        title="Step forward"
      >
        &#9193;
      </button>

      <!-- Jump to end -->
      <button
        class="w-8 h-8 rounded bg-ui-bg border border-gray-600 text-ui-text text-sm
               hover:border-ui-action hover:text-ui-action transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed"
        onclick={() => replayStore.jumpToEnd()}
        disabled={replayStore.isAtEnd}
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
                   {replayStore.playbackSpeed === speed
                     ? 'bg-ui-action text-white'
                     : 'bg-ui-bg border border-gray-600 text-ui-text-dim hover:border-ui-action hover:text-ui-action'}"
            onclick={() => replayStore.setSpeed(speed)}
          >
            {formatSpeed(speed)}
          </button>
        {/each}
      </div>

      <!-- Timeline scrubber -->
      <input
        type="range"
        min="-1"
        max={replayStore.totalActions - 1}
        value={replayStore.currentActionIndex}
        oninput={(e) => replayStore.jumpToAction(parseInt(e.currentTarget.value))}
        class="w-32 h-1.5 bg-ui-bg rounded-lg appearance-none cursor-pointer
               [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3
               [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-ui-action
               [&::-webkit-slider-thumb]:cursor-pointer [&::-webkit-slider-thumb]:hover:bg-ui-action/80"
      />
    </div>
  </div>
</div>
