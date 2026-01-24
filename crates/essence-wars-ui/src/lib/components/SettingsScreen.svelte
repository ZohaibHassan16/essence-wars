<script lang="ts">
  import { gameSettings } from "$lib/stores/gameSettings.svelte";
  import { audioSettings } from "$lib/stores/audioSettings.svelte";
  import { playSound, setMusicVolume } from "$lib/audio";

  let { onBack }: { onBack: () => void } = $props();

  function handleButtonHover() {
    playSound('buttonHover');
  }

  function handleButtonClick() {
    playSound('buttonClick');
  }

  function setAnimationSpeed(speed: number) {
    handleButtonClick();
    gameSettings.animationSpeed = speed;
  }

  function setAiTurnDelay(delay: number) {
    handleButtonClick();
    gameSettings.aiTurnDelay = delay;
  }
</script>

<div class="min-h-screen flex flex-col items-center justify-center p-8">
  <div class="max-w-2xl w-full bg-ui-panel rounded-xl p-8 shadow-2xl">
    <h1 class="text-3xl font-bold text-ui-text mb-8">Settings</h1>

    <!-- Gameplay Section -->
    <section class="mb-8">
      <h2 class="text-xl font-semibold text-ui-text mb-4 border-b border-gray-700 pb-2">Gameplay</h2>

      <!-- Animation Speed -->
      <fieldset class="mb-6 border-0 p-0 m-0">
        <legend class="block text-sm text-ui-text-dim mb-2">Animation Speed</legend>
        <div class="flex gap-2">
          {#each [0.5, 0.75, 1.0, 1.5, 2.0] as speed}
            <button
              class="flex-1 px-4 py-2 rounded-lg font-semibold transition-all
                     {gameSettings.animationSpeed === speed
                       ? 'bg-ui-action text-white'
                       : 'bg-ui-bg border border-gray-600 text-ui-text hover:border-gray-500'}"
              onclick={() => setAnimationSpeed(speed)}
              onmouseenter={handleButtonHover}
            >
              {speed}x
            </button>
          {/each}
        </div>
      </fieldset>

      <!-- AI Turn Delay -->
      <div class="mb-6">
        <label for="ai-turn-delay" class="block text-sm text-ui-text-dim mb-2">
          AI Turn Delay: {gameSettings.aiTurnDelay}ms
        </label>
        <input
          id="ai-turn-delay"
          type="range"
          min="50"
          max="500"
          step="50"
          value={gameSettings.aiTurnDelay}
          oninput={(e) => gameSettings.aiTurnDelay = Number((e.target as HTMLInputElement).value)}
          class="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer
                 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
                 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-ui-action"
        />
        <div class="flex justify-between text-xs text-ui-text-dim mt-1">
          <span>Fast</span>
          <span>Slow</span>
        </div>
      </div>
    </section>

    <!-- Audio Section -->
    <section class="mb-8">
      <h2 class="text-xl font-semibold text-ui-text mb-4 border-b border-gray-700 pb-2">Audio</h2>

      <!-- Master Volume -->
      <div class="mb-4">
        <label for="master-volume" class="block text-sm text-ui-text-dim mb-2">
          Master Volume: {Math.round(audioSettings.masterVolume * 100)}%
        </label>
        <input
          id="master-volume"
          type="range"
          min="0"
          max="100"
          value={audioSettings.masterVolume * 100}
          oninput={(e) => {
            audioSettings.masterVolume = Number((e.target as HTMLInputElement).value) / 100;
            setMusicVolume(audioSettings.effectiveMusicVolume);
          }}
          class="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer
                 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
                 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-ui-action"
        />
      </div>

      <!-- SFX Volume -->
      <div class="mb-4">
        <label for="sfx-volume" class="block text-sm text-ui-text-dim mb-2">
          SFX Volume: {Math.round(audioSettings.sfxVolume * 100)}%
        </label>
        <input
          id="sfx-volume"
          type="range"
          min="0"
          max="100"
          value={audioSettings.sfxVolume * 100}
          oninput={(e) => audioSettings.sfxVolume = Number((e.target as HTMLInputElement).value) / 100}
          class="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer
                 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
                 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-ui-action"
        />
      </div>

      <!-- Music Volume -->
      <div class="mb-4">
        <label for="music-volume" class="block text-sm text-ui-text-dim mb-2">
          Music Volume: {Math.round(audioSettings.musicVolume * 100)}%
        </label>
        <input
          id="music-volume"
          type="range"
          min="0"
          max="100"
          value={audioSettings.musicVolume * 100}
          oninput={(e) => {
            audioSettings.musicVolume = Number((e.target as HTMLInputElement).value) / 100;
            setMusicVolume(audioSettings.effectiveMusicVolume);
          }}
          class="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer
                 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
                 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-ui-action"
        />
      </div>

      <!-- Mute Toggle -->
      <button
        class="px-4 py-2 rounded-lg font-semibold transition-all
               {audioSettings.muted
                 ? 'bg-damage/20 border border-damage text-damage'
                 : 'bg-ui-bg border border-gray-600 text-ui-text hover:border-gray-500'}"
        onclick={() => {
          handleButtonClick();
          audioSettings.toggleMute();
          setMusicVolume(audioSettings.effectiveMusicVolume);
        }}
        onmouseenter={handleButtonHover}
      >
        {audioSettings.muted ? 'Unmute' : 'Mute All'}
      </button>
    </section>

    <!-- Accessibility Section -->
    <section class="mb-8">
      <h2 class="text-xl font-semibold text-ui-text mb-4 border-b border-gray-700 pb-2">Accessibility</h2>

      <!-- Keyboard Hints Toggle -->
      <label class="flex items-center gap-3 cursor-pointer">
        <input
          type="checkbox"
          checked={gameSettings.showKeyboardHints}
          onchange={(e) => gameSettings.showKeyboardHints = (e.target as HTMLInputElement).checked}
          class="w-5 h-5 rounded bg-ui-bg border-gray-600 text-ui-action
                 focus:ring-ui-action focus:ring-offset-0"
        />
        <span class="text-ui-text">Show keyboard shortcut hints</span>
      </label>
    </section>

    <!-- Action Buttons -->
    <div class="flex justify-between items-center pt-4 border-t border-gray-700">
      <button
        class="px-4 py-2 bg-ui-bg text-ui-text-dim rounded-lg font-semibold
               border border-gray-600 hover:border-gray-500 hover:text-ui-text transition-colors"
        onclick={() => {
          handleButtonClick();
          gameSettings.resetToDefaults();
        }}
        onmouseenter={handleButtonHover}
      >
        Reset to Defaults
      </button>

      <button
        class="px-6 py-2 bg-ui-action text-white rounded-lg font-semibold
               hover:bg-ui-action/80 transition-colors"
        onclick={() => {
          handleButtonClick();
          onBack();
        }}
        onmouseenter={handleButtonHover}
      >
        Back
      </button>
    </div>
  </div>
</div>
