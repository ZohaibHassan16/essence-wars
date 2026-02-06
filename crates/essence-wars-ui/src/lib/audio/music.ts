// Music manager for background music playback with track variety
import { audioSettings } from '$lib/stores/audioSettings.svelte';
import { assetUrl } from '$lib/utils/paths';

export type MusicTrack = 'menu' | 'battle' | 'spectator';
export type MusicSting = 'victory' | 'defeat';

// Track file paths - arrays for variety (OGG Vorbis for optimal size)
// Note: These are relative paths that get prefixed with base path at runtime
const MUSIC_TRACKS: Record<MusicTrack, string[]> = {
  menu: [
    '/music/menu_1.ogg',  // Town Theme RPG
    '/music/menu_2.ogg',  // Field of Dreams
    '/music/menu_3.ogg',  // Forest Ambience
    '/music/menu_4.ogg',  // Invention in B Minor
  ],
  battle: [
    '/music/battle_1.ogg',  // Battle Theme A
    '/music/battle_2.ogg',  // Boss Battle
    '/music/battle_3.ogg',  // Determined Pursuit
    '/music/battle_4.ogg',  // Battle Theme B
  ],
  spectator: [
    '/music/spectator_1.ogg',  // The Bard's Tale
    '/music/spectator_2.ogg',  // The Old Tower Inn
    '/music/spectator_3.ogg',  // Minstrel Dance
    '/music/spectator_4.ogg',  // King's Feast
  ],
};

const MUSIC_STINGS: Record<MusicSting, string> = {
  victory: '/music/victory.ogg',
  defeat: '/music/defeat.ogg',
};

// Current state
let currentTrack: MusicTrack | null = null;
let currentFilePath: string | null = null;
let currentAudio: HTMLAudioElement | null = null;
let stingAudio: HTMLAudioElement | null = null;
let fadeInterval: ReturnType<typeof setInterval> | null = null;

// Get a random track file, optionally excluding the current one for variety
function getRandomTrackFile(track: MusicTrack, excludeCurrent: boolean = false): string {
  const files = MUSIC_TRACKS[track];
  if (files.length === 1) return files[0];

  if (excludeCurrent && currentFilePath && files.length > 1) {
    const otherFiles = files.filter(f => f !== currentFilePath);
    return otherFiles[Math.floor(Math.random() * otherFiles.length)];
  }

  return files[Math.floor(Math.random() * files.length)];
}

// Start playing a music track (with optional fade-in)
export function playMusic(track: MusicTrack, fadeInMs: number = 1000): void {
  // Already playing this track type
  if (currentTrack === track && currentAudio && !currentAudio.paused) {
    return;
  }

  // Stop any existing music first
  if (currentAudio) {
    stopMusic(500);
  }

  // Clear any pending fade
  if (fadeInterval) {
    clearInterval(fadeInterval);
    fadeInterval = null;
  }

  currentTrack = track;
  currentFilePath = getRandomTrackFile(track);
  currentAudio = new Audio(assetUrl(currentFilePath));
  currentAudio.loop = false; // Don't loop - we'll play a different track when it ends
  currentAudio.volume = 0;

  // When track ends, play another random track from the same category
  currentAudio.onended = () => {
    if (currentTrack === track) {
      // Pick a different track for variety
      currentFilePath = getRandomTrackFile(track, true);
      currentAudio = new Audio(assetUrl(currentFilePath));
      currentAudio.loop = false;
      currentAudio.volume = audioSettings.effectiveMusicVolume;
      // onended handler is already set
      currentAudio.play().catch((e) => {
        console.warn(`Failed to play next track:`, e);
      });

      // Re-attach the onended handler to the new audio element
      const audio = currentAudio;
      audio.onended = () => {
        if (currentTrack === track && audio === currentAudio) {
          playNextTrack(track);
        }
      };
    }
  };

  // Start playback
  currentAudio.play().catch((e) => {
    console.warn(`Failed to play music "${track}":`, e);
  });

  // Fade in
  const targetVolume = audioSettings.effectiveMusicVolume;
  const steps = 20;
  const stepMs = fadeInMs / steps;
  const volumeStep = targetVolume / steps;
  let currentStep = 0;

  fadeInterval = setInterval(() => {
    currentStep++;
    if (currentAudio) {
      currentAudio.volume = Math.min(volumeStep * currentStep, targetVolume);
    }
    if (currentStep >= steps) {
      if (fadeInterval) {
        clearInterval(fadeInterval);
        fadeInterval = null;
      }
    }
  }, stepMs);
}

// Play the next track in rotation (called when a track ends)
function playNextTrack(track: MusicTrack): void {
  if (currentTrack !== track) return;

  currentFilePath = getRandomTrackFile(track, true);
  currentAudio = new Audio(assetUrl(currentFilePath));
  currentAudio.loop = false;
  currentAudio.volume = audioSettings.effectiveMusicVolume;

  const audio = currentAudio;
  audio.onended = () => {
    if (currentTrack === track && audio === currentAudio) {
      playNextTrack(track);
    }
  };

  audio.play().catch((e) => {
    console.warn(`Failed to play next track:`, e);
  });
}

// Stop the current music (with optional fade-out)
export function stopMusic(fadeOutMs: number = 500): void {
  if (!currentAudio) return;

  // Clear any pending fade
  if (fadeInterval) {
    clearInterval(fadeInterval);
    fadeInterval = null;
  }

  const audioToStop = currentAudio;
  const startVolume = audioToStop.volume;
  currentAudio = null;
  currentTrack = null;
  currentFilePath = null;

  // Remove onended handler to prevent it from playing next track
  audioToStop.onended = null;

  if (fadeOutMs <= 0) {
    audioToStop.pause();
    audioToStop.src = '';
    return;
  }

  // Fade out
  const steps = 10;
  const stepMs = fadeOutMs / steps;
  const volumeStep = startVolume / steps;
  let currentStep = 0;

  const fadeOut = setInterval(() => {
    currentStep++;
    audioToStop.volume = Math.max(startVolume - volumeStep * currentStep, 0);
    if (currentStep >= steps) {
      clearInterval(fadeOut);
      audioToStop.pause();
      audioToStop.src = '';
    }
  }, stepMs);
}

// Fade to a different track
export function fadeToTrack(track: MusicTrack, crossfadeMs: number = 1000): void {
  if (currentTrack === track) return;

  stopMusic(crossfadeMs / 2);
  setTimeout(() => {
    playMusic(track, crossfadeMs / 2);
  }, crossfadeMs / 2);
}

// Play a one-shot sting (victory/defeat) - doesn't interrupt current music but lowers its volume
export function playSting(sting: MusicSting): void {
  // Lower current music volume temporarily
  const previousVolume = currentAudio?.volume ?? 0;
  if (currentAudio) {
    currentAudio.volume = previousVolume * 0.3;
  }

  // Play the sting
  stingAudio = new Audio(assetUrl(MUSIC_STINGS[sting]));
  stingAudio.volume = audioSettings.effectiveMusicVolume;

  stingAudio.onended = () => {
    // Restore music volume
    if (currentAudio) {
      currentAudio.volume = audioSettings.effectiveMusicVolume;
    }
    stingAudio = null;
  };

  stingAudio.play().catch((e) => {
    console.warn(`Failed to play sting "${sting}":`, e);
    // Restore music volume on error too
    if (currentAudio) {
      currentAudio.volume = previousVolume;
    }
  });
}

// Set music volume (called when settings change)
export function setMusicVolume(volume: number): void {
  if (currentAudio) {
    currentAudio.volume = volume;
  }
  if (stingAudio) {
    stingAudio.volume = volume;
  }
}

// Pause music (e.g., when app loses focus)
export function pauseMusic(): void {
  if (currentAudio && !currentAudio.paused) {
    currentAudio.pause();
  }
}

// Resume music
export function resumeMusic(): void {
  if (currentAudio && currentAudio.paused && currentTrack) {
    currentAudio.play().catch((e) => {
      console.warn('Failed to resume music:', e);
    });
  }
}

// Get current track type (for UI display)
export function getCurrentTrack(): MusicTrack | null {
  return currentTrack;
}

// Get current file path (for debugging)
export function getCurrentFilePath(): string | null {
  return currentFilePath;
}

// Check if music is playing
export function isMusicPlaying(): boolean {
  return currentAudio !== null && !currentAudio.paused;
}
