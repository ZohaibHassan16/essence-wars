// Audio settings store using Svelte 5 runes with localStorage persistence

const STORAGE_KEY = 'essence-wars-audio-settings';

interface AudioSettingsData {
  masterVolume: number;
  sfxVolume: number;
  muted: boolean;
}

function loadSettings(): AudioSettingsData {
  if (typeof localStorage === 'undefined') {
    return { masterVolume: 0.7, sfxVolume: 0.8, muted: false };
  }
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return {
        masterVolume: typeof parsed.masterVolume === 'number' ? parsed.masterVolume : 0.7,
        sfxVolume: typeof parsed.sfxVolume === 'number' ? parsed.sfxVolume : 0.8,
        muted: typeof parsed.muted === 'boolean' ? parsed.muted : false,
      };
    }
  } catch (e) {
    console.warn('Failed to load audio settings:', e);
  }
  return { masterVolume: 0.7, sfxVolume: 0.8, muted: false };
}

function saveSettings(settings: AudioSettingsData) {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch (e) {
    console.warn('Failed to save audio settings:', e);
  }
}

class AudioSettingsStore {
  private _masterVolume = $state(0.7);
  private _sfxVolume = $state(0.8);
  private _muted = $state(false);

  constructor() {
    // Load settings on construction (will be called client-side)
    if (typeof window !== 'undefined') {
      const settings = loadSettings();
      this._masterVolume = settings.masterVolume;
      this._sfxVolume = settings.sfxVolume;
      this._muted = settings.muted;
    }
  }

  get masterVolume() {
    return this._masterVolume;
  }

  set masterVolume(value: number) {
    this._masterVolume = Math.max(0, Math.min(1, value));
    this.persist();
  }

  get sfxVolume() {
    return this._sfxVolume;
  }

  set sfxVolume(value: number) {
    this._sfxVolume = Math.max(0, Math.min(1, value));
    this.persist();
  }

  get muted() {
    return this._muted;
  }

  set muted(value: boolean) {
    this._muted = value;
    this.persist();
  }

  // Computed effective volume (combines master, sfx, and mute)
  get effectiveVolume() {
    if (this._muted) return 0;
    return this._masterVolume * this._sfxVolume;
  }

  toggleMute() {
    this._muted = !this._muted;
    this.persist();
  }

  private persist() {
    saveSettings({
      masterVolume: this._masterVolume,
      sfxVolume: this._sfxVolume,
      muted: this._muted,
    });
  }
}

export const audioSettings = new AudioSettingsStore();
