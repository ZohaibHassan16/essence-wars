// Audio manager using preloaded sound files with random variation and faction support
import { audioSettings } from '$lib/stores/audioSettings.svelte';
import { assetUrl } from '$lib/utils/paths';

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
  // Ability sounds
  | 'abilityActivate'
  // Game state sounds
  | 'turnStartPlayer'
  | 'turnStartOpponent'
  | 'victory'
  | 'defeat';

// Faction types for faction-specific sounds
export type Faction = 'argentum' | 'symbiote' | 'obsidion' | 'neutral';

// Sound file paths - arrays allow random selection for variation
const SOUND_FILES: Record<SoundEffect, string[]> = {
  // UI Sounds
  buttonHover: ['/sounds/interface/tick_001.ogg'],
  buttonClick: [
    '/sounds/interface/click_001.ogg',
    '/sounds/interface/click_002.ogg',
    '/sounds/interface/click_003.ogg',
  ],
  cardHover: ['/sounds/casino/card-slide-1.ogg', '/sounds/casino/card-slide-2.ogg'],
  cardSelect: [
    '/sounds/interface/confirmation_001.ogg',
    '/sounds/interface/confirmation_002.ogg',
  ],
  menuOpen: ['/sounds/interface/open_001.ogg', '/sounds/interface/open_002.ogg'],
  menuClose: ['/sounds/interface/close_001.ogg', '/sounds/interface/close_002.ogg'],

  // Card Sounds
  cardDraw: [
    '/sounds/casino/card-slide-1.ogg',
    '/sounds/casino/card-slide-2.ogg',
    '/sounds/casino/card-slide-3.ogg',
    '/sounds/casino/card-slide-4.ogg',
    '/sounds/casino/card-slide-5.ogg',
    '/sounds/casino/card-slide-6.ogg',
    '/sounds/casino/card-slide-7.ogg',
    '/sounds/casino/card-slide-8.ogg',
  ],
  cardPlayCreature: [
    '/sounds/casino/card-place-1.ogg',
    '/sounds/casino/card-place-2.ogg',
    '/sounds/casino/card-place-3.ogg',
    '/sounds/casino/card-place-4.ogg',
  ],
  cardPlaySpell: [
    '/sounds/battle/buff_generic_01.ogg',
    '/sounds/battle/buff_generic_02.ogg',
  ],
  cardPlaySupport: [
    '/sounds/battle/block_generic_01.ogg',
    '/sounds/battle/block_generic_02.ogg',
  ],

  // Combat Sounds (generic/neutral) - AI-generated battle SFX
  attackLight: [
    '/sounds/battle/attack_neutral_light_01.ogg',
    '/sounds/battle/attack_neutral_light_02.ogg',
  ],
  attackMedium: [
    '/sounds/battle/attack_neutral_medium_01.ogg',
    '/sounds/battle/attack_neutral_medium_02.ogg',
  ],
  attackHeavy: [
    '/sounds/battle/attack_neutral_heavy_01.ogg',
    '/sounds/battle/attack_neutral_heavy_02.ogg',
    '/sounds/battle/attack_neutral_heavy_03.ogg',
  ],
  damage: [
    '/sounds/battle/damage_generic_01.ogg',
    '/sounds/battle/damage_generic_02.ogg',
    '/sounds/battle/damage_generic_03.ogg',
    '/sounds/battle/damage_generic_04.ogg',
    '/sounds/battle/damage_generic_05.ogg',
  ],
  creatureDeath: [
    '/sounds/battle/death_neutral_01.ogg',
    '/sounds/battle/death_neutral_02.ogg',
    '/sounds/battle/death_neutral_03.ogg',
  ],
  heal: [
    '/sounds/battle/heal_generic_01.ogg',
    '/sounds/battle/heal_generic_02.ogg',
    '/sounds/battle/heal_generic_03.ogg',
  ],

  // Ability Sounds - AI-generated battle SFX
  abilityActivate: [
    '/sounds/battle/buff_generic_01.ogg',
    '/sounds/battle/buff_generic_02.ogg',
  ],

  // Game State Sounds
  turnStartPlayer: ['/sounds/interface/confirmation_003.ogg'],
  turnStartOpponent: ['/sounds/interface/select_001.ogg'],
  // Victory/defeat use the music stings
  victory: ['/music/victory.ogg'],
  defeat: ['/music/defeat.ogg'],
};

// Faction-specific attack sounds - AI-generated battle SFX
const FACTION_ATTACK_SOUNDS: Record<Faction, Record<'light' | 'medium' | 'heavy', string[]>> = {
  argentum: {
    light: [
      '/sounds/battle/attack_argentum_light_01.ogg',
      '/sounds/battle/attack_argentum_light_02.ogg',
    ],
    medium: [
      '/sounds/battle/attack_argentum_medium_01.ogg',
      '/sounds/battle/attack_argentum_medium_02.ogg',
    ],
    heavy: [
      '/sounds/battle/attack_argentum_heavy_01.ogg',
      '/sounds/battle/attack_argentum_heavy_02.ogg',
      '/sounds/battle/attack_argentum_heavy_03.ogg',
    ],
  },
  symbiote: {
    light: [
      '/sounds/battle/attack_symbiote_light_01.ogg',
      '/sounds/battle/attack_symbiote_light_02.ogg',
    ],
    medium: [
      '/sounds/battle/attack_symbiote_medium_01.ogg',
      '/sounds/battle/attack_symbiote_medium_02.ogg',
    ],
    heavy: [
      '/sounds/battle/attack_symbiote_heavy_01.ogg',
      '/sounds/battle/attack_symbiote_heavy_02.ogg',
      '/sounds/battle/attack_symbiote_heavy_03.ogg',
    ],
  },
  obsidion: {
    light: [
      '/sounds/battle/attack_obsidion_light_01.ogg',
      '/sounds/battle/attack_obsidion_light_02.ogg',
    ],
    medium: [
      '/sounds/battle/attack_obsidion_medium_01.ogg',
      '/sounds/battle/attack_obsidion_medium_02.ogg',
    ],
    heavy: [
      '/sounds/battle/attack_obsidion_heavy_01.ogg',
      '/sounds/battle/attack_obsidion_heavy_02.ogg',
      '/sounds/battle/attack_obsidion_heavy_03.ogg',
    ],
  },
  neutral: {
    light: [
      '/sounds/battle/attack_neutral_light_01.ogg',
      '/sounds/battle/attack_neutral_light_02.ogg',
    ],
    medium: [
      '/sounds/battle/attack_neutral_medium_01.ogg',
      '/sounds/battle/attack_neutral_medium_02.ogg',
    ],
    heavy: [
      '/sounds/battle/attack_neutral_heavy_01.ogg',
      '/sounds/battle/attack_neutral_heavy_02.ogg',
      '/sounds/battle/attack_neutral_heavy_03.ogg',
    ],
  },
};

// Faction-specific summon sounds - AI-generated battle SFX
const FACTION_SUMMON_SOUNDS: Record<Faction, string[]> = {
  argentum: [
    '/sounds/battle/summon_argentum_01.ogg',
    '/sounds/battle/summon_argentum_02.ogg',
    '/sounds/battle/summon_argentum_03.ogg',
  ],
  symbiote: [
    '/sounds/battle/summon_symbiote_01.ogg',
    '/sounds/battle/summon_symbiote_02.ogg',
    '/sounds/battle/summon_symbiote_03.ogg',
  ],
  obsidion: [
    '/sounds/battle/summon_obsidion_01.ogg',
    '/sounds/battle/summon_obsidion_02.ogg',
    '/sounds/battle/summon_obsidion_03.ogg',
  ],
  neutral: [
    '/sounds/battle/summon_neutral_01.ogg',
    '/sounds/battle/summon_neutral_02.ogg',
    '/sounds/battle/summon_neutral_03.ogg',
  ],
};

// Faction-specific death sounds - AI-generated battle SFX
const FACTION_DEATH_SOUNDS: Record<Faction, string[]> = {
  argentum: [
    '/sounds/battle/death_argentum_01.ogg',
    '/sounds/battle/death_argentum_02.ogg',
    '/sounds/battle/death_argentum_03.ogg',
  ],
  symbiote: [
    '/sounds/battle/death_symbiote_01.ogg',
    '/sounds/battle/death_symbiote_02.ogg',
    '/sounds/battle/death_symbiote_03.ogg',
  ],
  obsidion: [
    '/sounds/battle/death_obsidion_01.ogg',
    '/sounds/battle/death_obsidion_02.ogg',
    '/sounds/battle/death_obsidion_03.ogg',
  ],
  neutral: [
    '/sounds/battle/death_neutral_01.ogg',
    '/sounds/battle/death_neutral_02.ogg',
    '/sounds/battle/death_neutral_03.ogg',
  ],
};

// Cache for preloaded audio elements
const audioCache = new Map<string, HTMLAudioElement>();

// Preload a single audio file
async function preloadAudio(path: string): Promise<HTMLAudioElement | null> {
  // Apply base path for web deployment
  const fullPath = assetUrl(path);

  if (audioCache.has(fullPath)) {
    return audioCache.get(fullPath)!;
  }

  try {
    const audio = new Audio(fullPath);
    await new Promise<void>((resolve, reject) => {
      audio.addEventListener('canplaythrough', () => resolve(), { once: true });
      audio.addEventListener('error', () => reject(new Error(`Failed to load: ${fullPath}`)), { once: true });
      audio.load();
    });
    audioCache.set(fullPath, audio);
    return audio;
  } catch (e) {
    console.warn(`Failed to preload audio: ${fullPath}`, e);
    return null;
  }
}

// Get a random element from an array
function randomChoice<T>(arr: T[]): T {
  return arr[Math.floor(Math.random() * arr.length)];
}

// Play a sound file with explicit volume
function playSoundFileWithVolume(path: string, volume: number): void {
  if (volume <= 0) return;

  // Apply base path for web deployment
  const fullPath = assetUrl(path);

  // Try to use cached audio, or create new
  const cached = audioCache.get(fullPath);
  if (cached) {
    // Clone the audio for overlapping sounds
    const audio = cached.cloneNode() as HTMLAudioElement;
    audio.volume = volume;
    audio.play().catch((e) => console.warn(`Failed to play: ${fullPath}`, e));
  } else {
    // Fallback: create and play directly
    const audio = new Audio(fullPath);
    audio.volume = volume;
    audio.play().catch((e) => console.warn(`Failed to play: ${fullPath}`, e));
  }
}

// Play a sound file using the SFX volume setting
function playSoundFile(path: string): void {
  playSoundFileWithVolume(path, audioSettings.effectiveVolume);
}

// Play a sound effect (random selection from available files)
export function playSound(effect: SoundEffect): void {
  const volume = audioSettings.effectiveVolume;
  if (volume <= 0) return;

  const files = SOUND_FILES[effect];
  if (!files || files.length === 0) {
    console.warn(`No sound files for effect: ${effect}`);
    return;
  }

  const path = randomChoice(files);
  playSoundFile(path);
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

// Play faction-specific attack sound
export function playFactionAttackSound(damage: number, faction: Faction): void {
  const volume = audioSettings.effectiveVolume;
  if (volume <= 0) return;

  const intensity = damage >= 5 ? 'heavy' : damage >= 3 ? 'medium' : 'light';
  const files = FACTION_ATTACK_SOUNDS[faction][intensity];
  const path = randomChoice(files);
  playSoundFile(path);
}

// Play faction-specific summon sound
export function playFactionSummonSound(faction: Faction): void {
  const volume = audioSettings.effectiveVolume;
  if (volume <= 0) return;

  const files = FACTION_SUMMON_SOUNDS[faction];
  const path = randomChoice(files);
  playSoundFile(path);
}

// Play faction-specific death sound
export function playFactionDeathSound(faction: Faction): void {
  const volume = audioSettings.effectiveVolume;
  if (volume <= 0) return;

  const files = FACTION_DEATH_SOUNDS[faction];
  const path = randomChoice(files);
  playSoundFile(path);
}

// Play card play sound based on card type (with optional faction for creatures)
export function playCardSound(cardType: 'creature' | 'spell' | 'support', faction?: Faction): void {
  if (cardType === 'creature' && faction) {
    playFactionSummonSound(faction);
  } else {
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
}

// Preload all sounds for smoother playback
export async function preloadSounds(): Promise<void> {
  const allPaths = new Set<string>();

  // Collect all sound paths
  for (const files of Object.values(SOUND_FILES)) {
    for (const path of files) {
      allPaths.add(path);
    }
  }
  for (const factionSounds of Object.values(FACTION_ATTACK_SOUNDS)) {
    for (const files of Object.values(factionSounds)) {
      for (const path of files) {
        allPaths.add(path);
      }
    }
  }
  for (const files of Object.values(FACTION_SUMMON_SOUNDS)) {
    for (const path of files) {
      allPaths.add(path);
    }
  }
  for (const files of Object.values(FACTION_DEATH_SOUNDS)) {
    for (const path of files) {
      allPaths.add(path);
    }
  }

  // Preload all in parallel
  await Promise.all(Array.from(allPaths).map(preloadAudio));
  console.log(`Preloaded ${audioCache.size} sound files`);
}

// Clear the audio cache
export function clearAudioCache(): void {
  audioCache.clear();
}

// Utility: Get faction from card ID (based on ID ranges from CLAUDE.md)
export function getFactionFromCardId(cardId: number): Faction {
  if (cardId >= 1000 && cardId < 2000) return 'argentum';
  if (cardId >= 2000 && cardId < 3000) return 'symbiote';
  if (cardId >= 3000 && cardId < 4000) return 'obsidion';
  return 'neutral';
}
