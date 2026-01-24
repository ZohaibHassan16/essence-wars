// Audio module exports
export {
  playSound,
  preloadSounds,
  clearAudioCache,
  playAttackSound,
  playCardSound,
  playFactionAttackSound,
  playFactionSummonSound,
  playFactionDeathSound,
  getFactionFromCardId,
  type SoundEffect,
  type Faction,
} from './manager';

export {
  playMusic,
  stopMusic,
  fadeToTrack,
  playSting,
  setMusicVolume,
  pauseMusic,
  resumeMusic,
  getCurrentTrack,
  isMusicPlaying,
  type MusicTrack,
  type MusicSting,
} from './music';
