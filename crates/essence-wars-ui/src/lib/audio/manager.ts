// Audio manager using jsfxr for procedural sound generation
import { sfxr } from 'jsfxr';
import { audioSettings } from '$lib/stores/audioSettings.svelte';

// Sound effect types available in the game
export type SoundEffect =
  // UI sounds
  | 'buttonHover'
  | 'buttonClick'
  | 'cardHover'
  | 'cardSelect'
  | 'menuOpen'
  | 'menuClose'
  // Card sounds
  | 'cardDraw'
  | 'cardPlayCreature'
  | 'cardPlaySpell'
  | 'cardPlaySupport'
  // Combat sounds
  | 'attackLight'
  | 'attackMedium'
  | 'attackHeavy'
  | 'damage'
  | 'creatureDeath'
  | 'heal'
  // Game state sounds
  | 'turnStartPlayer'
  | 'turnStartOpponent'
  | 'victory'
  | 'defeat';

// Pre-defined sound parameters for consistent, non-random sounds
// These are base58-encoded jsfxr parameter strings for reproducibility
const SOUND_DEFINITIONS: Record<SoundEffect, string> = {
  // UI Sounds - subtle, quick
  buttonHover: '5EoyNBo9K7FnfQCkuZVhXNPoZ9mfqXRbsmGVnPfAE4SqaH3nWBBP4mfA7',
  buttonClick: '111112EMn5sQf6YJoeMHcUGHi1sqeVWLYaMVfrTjDhZ8F8KZ',
  cardHover: '111112EMn5sQf6YJoeMHcUGHi1sqeVWLYaMVfrTjDhZ8F8KZ',
  cardSelect: '111119XG3HQ5ZUUh4cAq1K3KNTXQXDvYo3tvB5hn9Zxzj3Gg',
  menuOpen: '34T91cGm3VbM5eHQ1QqV8uYwxqz9Gvq2BK4cCDsYBjBXqxDSNjnKbRBP4',
  menuClose: '34T91cGm3VbM5eHQ1QqV8uYwxqz9Gvq2BK4cCDsYBjBXqxDSNjnKbRBP4',

  // Card Sounds
  cardDraw: '34T91cGm3XAEf6aDT3xS2Z8qxWiMDBcuqdwC4YXaKcMYXhkZKrnhSLB',
  cardPlayCreature: '34T91cHVNMjBX6qJR6qJR4kGf6rwJEaRUaJf13sBMkrpDiEU',
  cardPlaySpell: '5EoyNBRYB4bWjp7j1H9ENBRuC2h7xdF1B5qMFrQ1fxk9nxkZKrnhSLB',
  cardPlaySupport: '34T91cHVNMjBX6qJR6qJR4kGf6rwJEaRUaJf13sBMkrpDiEU',

  // Combat Sounds
  attackLight: '34T91cC3c3GhKXEG1i9xfD2kMvpUaWGHKxvdxCDsYBjBXqxDS',
  attackMedium: '34T91cC3c5AuVZ8m1LV2LkZWn2fAhVq9Xr8c4YXaKcMYXhkZ',
  attackHeavy: '7BMHBBxZc78GFzMp18KBMpLqVoK7L2hVfKiVpZ1exexqxAm7Z',
  damage: '34T91cC3c3GhKXEG1i9xfD2kMvpUaWGHKxvdxCDsYBjBXqxDS',
  creatureDeath: '7BMHBBxZc78GFzMp18KBMpLqVoK7L2hVfKiVpZ1exexqxAm7Z',
  heal: '34T91cGm3VbM5eHQ1QqV8uYwxqz9Gvq2BK4cCDsYBjBXqxDSNjnKb',

  // Game State Sounds
  turnStartPlayer: '111119XG3HQ5ZUUh4cAq1K3KNTXQXDvYo3tvB5hn9Zxzj3Gg',
  turnStartOpponent: '5EoyNBo9K7FnfQCkuZVhXNPoZ9mfqXRbsmGVnPfAE4SqaH3nW',
  victory: '34T91cGm3VbM5eHQ1QqV8uYwxqz9Gvq2BK4cCDsYBjBXqxDSNjnKbRBP4',
  defeat: '7BMHBBxZc78GFzMp18KBMpLqVoK7L2hVfKiVpZ1exexqxAm7Z',
};

// Cache for generated audio objects
const audioCache = new Map<SoundEffect, ReturnType<typeof sfxr.toAudio>>();

// Generate fresh audio from preset parameters
function generateSound(effect: SoundEffect): ReturnType<typeof sfxr.toAudio> {
  // Use preset-based generation for more reliable sounds
  const presets: Record<SoundEffect, () => ReturnType<typeof sfxr.generate>> = {
    // UI - subtle blips
    buttonHover: () => {
      const p = sfxr.generate('blipSelect');
      p.p_env_sustain = 0.05;
      p.p_env_decay = 0.05;
      p.p_base_freq = 0.6;
      p.sound_vol = 0.15;
      return p;
    },
    buttonClick: () => {
      const p = sfxr.generate('blipSelect');
      p.p_env_sustain = 0.08;
      p.p_env_decay = 0.1;
      p.p_base_freq = 0.4;
      p.sound_vol = 0.2;
      return p;
    },
    cardHover: () => {
      const p = sfxr.generate('blipSelect');
      p.p_env_sustain = 0.03;
      p.p_env_decay = 0.05;
      p.p_base_freq = 0.7;
      p.sound_vol = 0.1;
      return p;
    },
    cardSelect: () => {
      const p = sfxr.generate('blipSelect');
      p.p_env_sustain = 0.1;
      p.p_env_decay = 0.15;
      p.sound_vol = 0.25;
      return p;
    },
    menuOpen: () => {
      const p = sfxr.generate('powerUp');
      p.p_env_sustain = 0.1;
      p.p_env_decay = 0.2;
      p.p_freq_ramp = 0.2;
      p.sound_vol = 0.2;
      return p;
    },
    menuClose: () => {
      const p = sfxr.generate('powerUp');
      p.p_env_sustain = 0.1;
      p.p_env_decay = 0.15;
      p.p_freq_ramp = -0.2;
      p.sound_vol = 0.2;
      return p;
    },

    // Card sounds
    cardDraw: () => {
      const p = sfxr.generate('pickupCoin');
      p.p_env_sustain = 0.1;
      p.p_env_decay = 0.15;
      p.p_base_freq = 0.5;
      p.sound_vol = 0.2;
      return p;
    },
    cardPlayCreature: () => {
      const p = sfxr.generate('hitHurt');
      p.p_env_sustain = 0.15;
      p.p_env_decay = 0.2;
      p.p_base_freq = 0.3;
      p.p_freq_ramp = -0.15;
      p.sound_vol = 0.3;
      return p;
    },
    cardPlaySpell: () => {
      const p = sfxr.generate('laserShoot');
      p.p_env_sustain = 0.15;
      p.p_env_decay = 0.25;
      p.sound_vol = 0.25;
      return p;
    },
    cardPlaySupport: () => {
      const p = sfxr.generate('hitHurt');
      p.p_env_sustain = 0.2;
      p.p_env_decay = 0.15;
      p.p_base_freq = 0.25;
      p.sound_vol = 0.25;
      return p;
    },

    // Combat sounds
    attackLight: () => {
      const p = sfxr.generate('hitHurt');
      p.p_env_sustain = 0.05;
      p.p_env_decay = 0.1;
      p.p_base_freq = 0.5;
      p.sound_vol = 0.25;
      return p;
    },
    attackMedium: () => {
      const p = sfxr.generate('hitHurt');
      p.p_env_sustain = 0.1;
      p.p_env_decay = 0.15;
      p.p_base_freq = 0.4;
      p.sound_vol = 0.3;
      return p;
    },
    attackHeavy: () => {
      const p = sfxr.generate('explosion');
      p.p_env_sustain = 0.15;
      p.p_env_decay = 0.3;
      p.p_base_freq = 0.3;
      p.sound_vol = 0.35;
      return p;
    },
    damage: () => {
      const p = sfxr.generate('hitHurt');
      p.p_env_sustain = 0.08;
      p.p_env_decay = 0.12;
      p.p_base_freq = 0.35;
      p.sound_vol = 0.3;
      return p;
    },
    creatureDeath: () => {
      const p = sfxr.generate('explosion');
      p.p_env_sustain = 0.2;
      p.p_env_decay = 0.4;
      p.p_base_freq = 0.2;
      p.sound_vol = 0.35;
      return p;
    },
    heal: () => {
      const p = sfxr.generate('powerUp');
      p.p_env_sustain = 0.15;
      p.p_env_decay = 0.3;
      p.p_freq_ramp = 0.15;
      p.sound_vol = 0.25;
      return p;
    },

    // Game state sounds
    turnStartPlayer: () => {
      const p = sfxr.generate('powerUp');
      p.p_env_sustain = 0.15;
      p.p_env_decay = 0.25;
      p.p_base_freq = 0.5;
      p.p_freq_ramp = 0.1;
      p.sound_vol = 0.3;
      return p;
    },
    turnStartOpponent: () => {
      const p = sfxr.generate('blipSelect');
      p.p_env_sustain = 0.12;
      p.p_env_decay = 0.2;
      p.p_base_freq = 0.35;
      p.sound_vol = 0.2;
      return p;
    },
    victory: () => {
      const p = sfxr.generate('powerUp');
      p.p_env_sustain = 0.3;
      p.p_env_decay = 0.5;
      p.p_freq_ramp = 0.25;
      p.p_arp_mod = 0.5;
      p.p_arp_speed = 0.6;
      p.sound_vol = 0.4;
      return p;
    },
    defeat: () => {
      const p = sfxr.generate('explosion');
      p.p_env_sustain = 0.3;
      p.p_env_decay = 0.6;
      p.p_base_freq = 0.15;
      p.p_freq_ramp = -0.1;
      p.sound_vol = 0.35;
      return p;
    },
  };

  const params = presets[effect]();
  return sfxr.toAudio(params);
}

// Get or create cached audio for an effect
function getAudio(effect: SoundEffect): ReturnType<typeof sfxr.toAudio> {
  let audio = audioCache.get(effect);
  if (!audio) {
    audio = generateSound(effect);
    audioCache.set(effect, audio);
  }
  return audio;
}

// Play a sound effect
export function playSound(effect: SoundEffect): void {
  // Skip if muted or volume is 0
  const volume = audioSettings.effectiveVolume;
  if (volume <= 0) return;

  try {
    const audio = getAudio(effect);
    audio.setVolume(volume);
    audio.play();
  } catch (e) {
    console.warn(`Failed to play sound "${effect}":`, e);
  }
}

// Preload all sounds (call on app init for smoother first plays)
export function preloadSounds(): void {
  const effects: SoundEffect[] = [
    'buttonHover',
    'buttonClick',
    'cardHover',
    'cardSelect',
    'menuOpen',
    'menuClose',
    'cardDraw',
    'cardPlayCreature',
    'cardPlaySpell',
    'cardPlaySupport',
    'attackLight',
    'attackMedium',
    'attackHeavy',
    'damage',
    'creatureDeath',
    'heal',
    'turnStartPlayer',
    'turnStartOpponent',
    'victory',
    'defeat',
  ];

  for (const effect of effects) {
    getAudio(effect);
  }
}

// Clear the audio cache (useful for memory management)
export function clearAudioCache(): void {
  audioCache.clear();
}

// Play attack sound based on damage amount
export function playAttackSound(damage: number): void {
  if (damage >= 5) {
    playSound('attackHeavy');
  } else if (damage >= 3) {
    playSound('attackMedium');
  } else {
    playSound('attackLight');
  }
}

// Play card play sound based on card type
export function playCardSound(cardType: 'creature' | 'spell' | 'support'): void {
  switch (cardType) {
    case 'creature':
      playSound('cardPlayCreature');
      break;
    case 'spell':
      playSound('cardPlaySpell');
      break;
    case 'support':
      playSound('cardPlaySupport');
      break;
  }
}
