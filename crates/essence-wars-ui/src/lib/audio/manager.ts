// Audio manager using preloaded sound files with random variation and faction support
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

// Faction types for faction-specific sounds
export type Faction = 'argentum' | 'symbiote' | 'obsidion' | 'neutral';

// Sound file paths - arrays allow random selection for variation
const SOUND_FILES: Record<SoundEffect, string[]> = {
  // UI Sounds
  buttonHover: ['/sounds/interface/tick_001.ogg', '/sounds/interface/tick_002.ogg'],
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
  cardPlaySpell: ['/sounds/rpg/spell_01.ogg', '/sounds/rpg/spell_02.ogg'],
  cardPlaySupport: ['/sounds/rpg/metal_01.ogg', '/sounds/rpg/metal_02.ogg'],

  // Combat Sounds (generic/neutral)
  attackLight: ['/sounds/rpg/blade_01.ogg'],
  attackMedium: ['/sounds/rpg/blade_02.ogg'],
  attackHeavy: ['/sounds/rpg/blade_03.ogg'],
  damage: [
    '/sounds/creatures/hurt_01.ogg',
    '/sounds/creatures/hurt_02.ogg',
    '/sounds/creatures/hurt_03.ogg',
    '/sounds/creatures/hurt_04.ogg',
    '/sounds/creatures/hurt_05.ogg',
  ],
  creatureDeath: ['/sounds/rpg/creature_die_01.ogg'],
  heal: [
    '/sounds/rpg/item_gem_01.ogg',
    '/sounds/rpg/item_gem_02.ogg',
    '/sounds/rpg/item_gem_03.ogg',
  ],

  // Game State Sounds
  turnStartPlayer: ['/sounds/interface/confirmation_003.ogg'],
  turnStartOpponent: ['/sounds/interface/select_001.ogg'],
  // Victory/defeat use the music stings
  victory: ['/music/victory.wav'],
  defeat: ['/music/defeat.mp3'],
};

// Faction-specific attack sounds
const FACTION_ATTACK_SOUNDS: Record<Faction, Record<'light' | 'medium' | 'heavy', string[]>> = {
  argentum: {
    light: ['/sounds/rpg/metal_01.ogg'],
    medium: ['/sounds/rpg/metal_02.ogg', '/sounds/rpg/chain_01.ogg'],
    heavy: ['/sounds/rpg/metal_03.ogg', '/sounds/rpg/chain_02.ogg', '/sounds/rpg/chain_03.ogg'],
  },
  symbiote: {
    light: ['/sounds/creatures/spit_01.ogg', '/sounds/creatures/spit_02.ogg'],
    medium: ['/sounds/rpg/creature_slime_01.ogg', '/sounds/rpg/creature_slime_02.ogg'],
    heavy: ['/sounds/rpg/creature_slime_03.ogg', '/sounds/rpg/creature_slime_04.ogg'],
  },
  obsidion: {
    light: ['/sounds/rpg/spell_fire_01.ogg', '/sounds/rpg/spell_fire_02.ogg'],
    medium: ['/sounds/rpg/spell_fire_03.ogg', '/sounds/rpg/spell_fire_04.ogg'],
    heavy: ['/sounds/rpg/spell_fire_05.ogg', '/sounds/rpg/spell_fire_06.ogg', '/sounds/rpg/spell_fire_07.ogg'],
  },
  neutral: {
    light: ['/sounds/rpg/blade_01.ogg'],
    medium: ['/sounds/rpg/blade_02.ogg'],
    heavy: ['/sounds/rpg/blade_03.ogg'],
  },
};

// Faction-specific summon sounds
const FACTION_SUMMON_SOUNDS: Record<Faction, string[]> = {
  argentum: [
    '/sounds/rpg/lock_01.ogg',
    '/sounds/rpg/lock_02.ogg',
    '/sounds/rpg/stones_01.ogg',
    '/sounds/rpg/stones_02.ogg',
  ],
  symbiote: [
    '/sounds/creatures/burble_01.ogg',
    '/sounds/creatures/burble_02.ogg',
    '/sounds/creatures/bug_01.ogg',
    '/sounds/creatures/bug_02.ogg',
  ],
  obsidion: [
    '/sounds/rpg/creature_roar_01.ogg',
    '/sounds/rpg/creature_roar_02.ogg',
    '/sounds/rpg/creature_roar_03.ogg',
  ],
  neutral: [
    '/sounds/casino/card-place-1.ogg',
    '/sounds/casino/card-place-2.ogg',
    '/sounds/casino/card-place-3.ogg',
  ],
};

// Faction-specific death sounds
const FACTION_DEATH_SOUNDS: Record<Faction, string[]> = {
  argentum: [
    '/sounds/rpg/metal_03.ogg',
    '/sounds/rpg/stones_03.ogg',
    '/sounds/rpg/stones_04.ogg',
  ],
  symbiote: [
    '/sounds/creatures/alien_01.ogg',
    '/sounds/creatures/alien_02.ogg',
    '/sounds/creatures/weird_01.ogg',
    '/sounds/creatures/weird_02.ogg',
  ],
  obsidion: [
    '/sounds/creatures/scream_01.ogg',
    '/sounds/creatures/scream_02.ogg',
    '/sounds/creatures/monster_01.ogg',
  ],
  neutral: [
    '/sounds/rpg/creature_die_01.ogg',
    '/sounds/creatures/hurt_05.ogg',
  ],
};

// Cache for preloaded audio elements
const audioCache = new Map<string, HTMLAudioElement>();

// Preload a single audio file
async function preloadAudio(path: string): Promise<HTMLAudioElement | null> {
  if (audioCache.has(path)) {
    return audioCache.get(path)!;
  }

  try {
    const audio = new Audio(path);
    await new Promise<void>((resolve, reject) => {
      audio.addEventListener('canplaythrough', () => resolve(), { once: true });
      audio.addEventListener('error', () => reject(new Error(`Failed to load: ${path}`)), { once: true });
      audio.load();
    });
    audioCache.set(path, audio);
    return audio;
  } catch (e) {
    console.warn(`Failed to preload audio: ${path}`, e);
    return null;
  }
}

// Get a random element from an array
function randomChoice<T>(arr: T[]): T {
  return arr[Math.floor(Math.random() * arr.length)];
}

// Play a sound file with volume control
function playSoundFile(path: string): void {
  const volume = audioSettings.effectiveVolume;
  if (volume <= 0) return;

  // Try to use cached audio, or create new
  const cached = audioCache.get(path);
  if (cached) {
    // Clone the audio for overlapping sounds
    const audio = cached.cloneNode() as HTMLAudioElement;
    audio.volume = volume;
    audio.play().catch((e) => console.warn(`Failed to play: ${path}`, e));
  } else {
    // Fallback: create and play directly
    const audio = new Audio(path);
    audio.volume = volume;
    audio.play().catch((e) => console.warn(`Failed to play: ${path}`, e));
  }
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

// Play attack sound based on damage amount (generic)
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
