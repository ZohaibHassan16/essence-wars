// Game settings store using Svelte 5 runes with localStorage persistence

const STORAGE_KEY = 'essence-wars-game-settings';

interface GameSettingsData {
  animationSpeed: number;       // 0.5-2.0 multiplier
  aiTurnDelay: number;          // 100-1000ms delay between AI actions
  showKeyboardHints: boolean;   // Show keyboard shortcut hints in UI
}

function loadSettings(): GameSettingsData {
  if (typeof localStorage === 'undefined') {
    return { animationSpeed: 1.0, aiTurnDelay: 150, showKeyboardHints: true };
  }
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return {
        animationSpeed: typeof parsed.animationSpeed === 'number' ? parsed.animationSpeed : 1.0,
        aiTurnDelay: typeof parsed.aiTurnDelay === 'number' ? parsed.aiTurnDelay : 150,
        showKeyboardHints: typeof parsed.showKeyboardHints === 'boolean' ? parsed.showKeyboardHints : true,
      };
    }
  } catch (e) {
    console.warn('Failed to load game settings:', e);
  }
  return { animationSpeed: 1.0, aiTurnDelay: 150, showKeyboardHints: true };
}

function saveSettings(settings: GameSettingsData) {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch (e) {
    console.warn('Failed to save game settings:', e);
  }
}

class GameSettingsStore {
  private _animationSpeed = $state(1.0);
  private _aiTurnDelay = $state(150);
  private _showKeyboardHints = $state(true);

  constructor() {
    // Load settings on construction (will be called client-side)
    if (typeof window !== 'undefined') {
      const settings = loadSettings();
      this._animationSpeed = settings.animationSpeed;
      this._aiTurnDelay = settings.aiTurnDelay;
      this._showKeyboardHints = settings.showKeyboardHints;
    }
  }

  get animationSpeed() {
    return this._animationSpeed;
  }

  set animationSpeed(value: number) {
    this._animationSpeed = Math.max(0.5, Math.min(2.0, value));
    this.persist();
  }

  get aiTurnDelay() {
    return this._aiTurnDelay;
  }

  set aiTurnDelay(value: number) {
    this._aiTurnDelay = Math.max(50, Math.min(1000, value));
    this.persist();
  }

  get showKeyboardHints() {
    return this._showKeyboardHints;
  }

  set showKeyboardHints(value: boolean) {
    this._showKeyboardHints = value;
    this.persist();
  }

  /** Get animation duration adjusted by speed multiplier */
  getAdjustedDuration(baseDurationMs: number): number {
    return baseDurationMs / this._animationSpeed;
  }

  /** Reset all settings to defaults */
  resetToDefaults() {
    this._animationSpeed = 1.0;
    this._aiTurnDelay = 150;
    this._showKeyboardHints = true;
    this.persist();
  }

  private persist() {
    saveSettings({
      animationSpeed: this._animationSpeed,
      aiTurnDelay: this._aiTurnDelay,
      showKeyboardHints: this._showKeyboardHints,
    });
  }
}

export const gameSettings = new GameSettingsStore();
