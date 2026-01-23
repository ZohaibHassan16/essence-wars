<script lang="ts">
  import { audioSettings } from "$lib/stores/audioSettings.svelte";
  import { playSound } from "$lib/audio";
  import { Volume2, VolumeX } from "lucide-svelte";

  let isExpanded = $state(false);

  function handleMasterChange(e: Event) {
    const target = e.target as HTMLInputElement;
    audioSettings.masterVolume = parseFloat(target.value);
  }

  function handleSfxChange(e: Event) {
    const target = e.target as HTMLInputElement;
    audioSettings.sfxVolume = parseFloat(target.value);
    // Play a test sound when adjusting
    playSound('buttonClick');
  }

  function toggleMute() {
    audioSettings.toggleMute();
    if (!audioSettings.muted) {
      playSound('buttonClick');
    }
  }

  function toggleExpand() {
    isExpanded = !isExpanded;
    if (isExpanded) {
      playSound('menuOpen');
    }
  }
</script>

<div class="relative">
  <!-- Compact toggle button -->
  <button
    class="p-2 rounded-lg bg-ui-panel/80 hover:bg-ui-panel border border-gray-600
           hover:border-ui-action transition-all"
    onclick={toggleExpand}
    title={audioSettings.muted ? "Audio muted" : "Audio settings"}
  >
    {#if audioSettings.muted}
      <VolumeX class="w-5 h-5 text-ui-text-dim" />
    {:else}
      <Volume2 class="w-5 h-5 text-ui-text" />
    {/if}
  </button>

  <!-- Expanded controls panel -->
  {#if isExpanded}
    <div class="absolute bottom-full right-0 mb-2 p-4 rounded-lg bg-ui-panel border border-gray-600
                shadow-xl min-w-[200px] z-50">
      <div class="flex flex-col gap-4">
        <div class="flex items-center justify-between">
          <span class="text-sm text-ui-text font-semibold">Audio</span>
          <button
            class="p-1.5 rounded hover:bg-gray-700 transition-colors"
            onclick={toggleMute}
            title={audioSettings.muted ? "Unmute" : "Mute"}
          >
            {#if audioSettings.muted}
              <VolumeX class="w-4 h-4 text-damage" />
            {:else}
              <Volume2 class="w-4 h-4 text-health" />
            {/if}
          </button>
        </div>

        <!-- Master Volume -->
        <div class="flex flex-col gap-1">
          <div class="flex justify-between text-xs text-ui-text-dim">
            <span>Master</span>
            <span>{Math.round(audioSettings.masterVolume * 100)}%</span>
          </div>
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            value={audioSettings.masterVolume}
            oninput={handleMasterChange}
            class="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer
                   [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4
                   [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:rounded-full
                   [&::-webkit-slider-thumb]:bg-ui-action [&::-webkit-slider-thumb]:cursor-pointer
                   [&::-moz-range-thumb]:w-4 [&::-moz-range-thumb]:h-4
                   [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:bg-ui-action
                   [&::-moz-range-thumb]:border-0 [&::-moz-range-thumb]:cursor-pointer"
          />
        </div>

        <!-- SFX Volume -->
        <div class="flex flex-col gap-1">
          <div class="flex justify-between text-xs text-ui-text-dim">
            <span>SFX</span>
            <span>{Math.round(audioSettings.sfxVolume * 100)}%</span>
          </div>
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            value={audioSettings.sfxVolume}
            oninput={handleSfxChange}
            class="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer
                   [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4
                   [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:rounded-full
                   [&::-webkit-slider-thumb]:bg-ui-action [&::-webkit-slider-thumb]:cursor-pointer
                   [&::-moz-range-thumb]:w-4 [&::-moz-range-thumb]:h-4
                   [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:bg-ui-action
                   [&::-moz-range-thumb]:border-0 [&::-moz-range-thumb]:cursor-pointer"
          />
        </div>
      </div>
    </div>
  {/if}
</div>
