# Phase 9.6 Final Art Asset Sprint - Implementation Plan

## Overview

This document outlines the remaining work for Phase 9.6 of the Essence Wars desktop client. The sprint focuses on adding polish through art assets, audio, and a rules reference system.

**Status**: In Progress
**Quick Fixes Completed**:
- [x] Hide console window in release mode (`main.rs`)
- [x] Hide MCP Sync View from production menu (dev-only via `import.meta.env.DEV`)

---

## ✅ Sub-Phase A: Rules Submenu [DONE]

**Goal**: Create a standalone rules reference accessible from the main menu that explains the game without requiring users to play the tutorial.

### A.1 Content Structure

The rules screen should cover:

1. **Overview Tab**
   - Game objective (reduce opponent life to 0)
   - Win/loss conditions (life zero, deck out, 30-turn limit tiebreaker)
   - Board layout overview (5 creature slots, 2 support slots)

2. **Turn Structure Tab**
   - Action Points (3 per turn)
   - Actions: Play cards, Attack, Use abilities, End turn
   - Turn phases and flow

3. **Card Types Tab**
   - **Creatures**: Stats (attack/health), summoning sickness, combat
   - **Spells**: One-time effects, targeting
   - **Supports**: Persistent effects, durability

4. **Keywords Tab** (14 keywords)
   | Keyword | Effect |
   |---------|--------|
   | Rush | Can attack immediately |
   | Ranged | Can attack any enemy creature |
   | Piercing | Excess damage hits face |
   | Guard | Must be attacked first |
   | Lifesteal | Heal for damage dealt |
   | Lethal | Kills any creature it damages |
   | Shield | Blocks first damage instance |
   | Quick | Attacks before being attacked |
   | Ephemeral | Dies at end of turn |
   | Regenerate | Heals to full at turn start |
   | Stealth | Cannot be targeted until attacks |
   | Charge | Gains +1 attack each turn |
   | Frenzy | Attacks twice |
   | Volatile | Deals damage when killed |

5. **Combat Tab**
   - Attack resolution
   - Creature vs creature combat
   - Direct attacks (face damage)
   - Guard mechanics

6. **Factions Tab** (Optional - for lore/flavor)
   - Argentum Combine: Defensive constructs
   - Symbiote Circles: Aggressive swarm
   - Obsidion Syndicate: Lifesteal/burst
   - Free-Walkers: Utility neutral cards

### A.2 Technical Implementation

**New Files**:
- `src/lib/components/RulesScreen.svelte` - Main rules component
- `src/lib/components/rules/` - Sub-components for each tab
  - `RulesOverview.svelte`
  - `RulesTurnStructure.svelte`
  - `RulesCardTypes.svelte`
  - `RulesKeywords.svelte`
  - `RulesCombat.svelte`
  - `RulesFactions.svelte` (optional)

**Menu Integration**:
- Add "Rules" button to `MainMenu.svelte` between "Watch Replays" and "Settings"
- New app state for showing rules screen

**Visual Design**:
- Tabbed navigation (horizontal tabs at top)
- Each section uses existing card components for examples
- Keyword icons displayed with descriptions
- Consistent with existing UI theme

### A.3 Art Assets for Rules

- Keyword icons (already exist in `/static/icons/`)
- Example card renders (can use existing card components)
- Optional: Section header illustrations

### A.4 Acceptance Criteria

- [x] Rules accessible from main menu
- [x] All 6 content sections implemented
- [x] All 14 keywords documented with icons
- [x] Navigation between sections smooth
- [x] Back button returns to main menu
- [x] Consistent visual style with rest of app

### A.5 Implementation Complete

**Files Created:**
- `src/lib/components/RulesScreen.svelte` - Main tabbed container
- `src/lib/components/rules/RulesOverview.svelte` - Game overview, victory conditions, board layout
- `src/lib/components/rules/RulesTurnStructure.svelte` - Turn phases, actions, AP system
- `src/lib/components/rules/RulesCardTypes.svelte` - Creatures, spells, supports explained
- `src/lib/components/rules/RulesKeywords.svelte` - All 14 keywords with icons and details
- `src/lib/components/rules/RulesCombat.svelte` - Combat mechanics, guard, direct damage
- `src/lib/components/rules/RulesFactions.svelte` - Faction lore and playstyles

**Files Modified:**
- `src/lib/components/MainMenu.svelte` - Added "Rules & Guide" button
- `src/routes/+page.svelte` - Added routing for RulesScreen

---

## ✅ Sub-Phase B: Main Menu Backgrounds [DONE]

**Goal**: Replace the plain dark gradient with 20-30 rotating atmospheric artwork pieces.

### B.1 Art Requirements

**Specifications**:
- Resolution: 2560 x 1440 (16:9, scales down gracefully)
- Format: WebP or PNG (WebP preferred for size)
- Style: Fantasy card game atmosphere, painterly
- Themes: Mix of faction-themed and general fantasy

**Suggested Themes** (20-30 images):
1. **Argentum (5-7 images)**
   - Steam-powered factory interior
   - Clockwork army marching
   - Brass and gold throne room
   - Gear-filled laboratory
   - Sunrise over metal spires

2. **Symbiote (5-7 images)**
   - Bioluminescent forest
   - Hive mind emergence
   - Spore-filled cavern
   - Jungle canopy at dusk
   - Creature swarm gathering

3. **Obsidion (5-7 images)**
   - Gothic cathedral with essence crystals
   - Blood moon ritual
   - Vampire court
   - Neon-lit dark alley
   - Crimson throne room

4. **Neutral/General (5-7 images)**
   - Trading post at crossroads
   - Desert battlefield aftermath
   - Essence storm
   - Card table with floating cards
   - Arena entrance

### B.2 Technical Implementation

**Asset Location**: `/static/backgrounds/menu/`

**Implementation Changes**:

```svelte
// MainMenu.svelte additions
<script>
  // Preload background list
  const menuBackgrounds = [
    '/backgrounds/menu/argentum_factory.webp',
    '/backgrounds/menu/symbiote_forest.webp',
    // ... etc
  ];

  // Random selection on mount
  let currentBackground = $state('');

  $effect(() => {
    const idx = Math.floor(Math.random() * menuBackgrounds.length);
    currentBackground = menuBackgrounds[idx];
  });
</script>

<div
  class="min-h-screen bg-cover bg-center"
  style="background-image: url({currentBackground})"
>
  <!-- Overlay for readability -->
  <div class="min-h-screen bg-black/60 backdrop-blur-sm">
    <!-- existing menu content -->
  </div>
</div>
```

**Optional Enhancements**:
- Fade transition on load
- Background rotation on timer (every 30s)
- Preload next background for smooth transitions

### B.3 Art Generation Approach

**Tools**:
- FLUX via stable-diffusion.cpp (local)
- Gemini Image / Nanon Banana 3 for collages
- Resolution: Generate at 1280x720, upscale to 2560x1440

**Example FLUX Prompts**:

```
# Argentum Factory
A vast steampunk factory interior with brass gears and golden
machinery, steam rising from pipes, warm amber lighting, oil painting
style, highly detailed, fantasy card game art, no text, no characters

# Symbiote Forest
A bioluminescent alien forest at twilight, purple and green glowing
mushrooms, organic architecture, misty atmosphere, fantasy painting
style, rich colors, card game background art

# Obsidion Cathedral
A gothic cathedral interior with floating crimson crystals, dark
atmosphere with neon blue essence flowing through channels, dramatic
lighting, dark fantasy art style, painterly
```

### B.4 Acceptance Criteria

- [x] 20-30 background images created (22 backgrounds + 1 banner = 23 total)
- [x] Images properly optimized (WebP format)
- [x] Random selection on menu load
- [x] Readable text overlay (60% black + slight blur)
- [x] Smooth loading (no flash of unstyled content)

### B.5 Implementation Complete

**Assets Created (23 total):**
- `essence_wars_banner.webp` - Title banner replacing text
- 4x Argentum backgrounds
- 4x Symbiote backgrounds
- 4x Obsidion backgrounds
- 6x Neutral backgrounds
- 4x Mixed/world backgrounds

**Files Modified:**
- `src/lib/components/MainMenu.svelte` - Background rotation + banner image

---

## ✅ Sub-Phase C: Music Integration [DONE]

**Goal**: Add background music for menu, battle, and spectator modes.

### ✅ C.1 Music Requirements

| Context | Style | Duration | Loop |
|---------|-------|----------|------|
| Main Menu | Epic orchestral, building anticipation | 2-4 min | Yes |
| Battle (Human vs AI) | Tense, strategic, medium tempo | 3-5 min | Yes |
| Spectator Mode | Analytical, observational, calmer | 3-5 min | Yes |
| Victory | Triumphant fanfare | 10-15 sec | No |
| Defeat | Somber, reflective | 10-15 sec | No |

### C.2 Sourcing Options

**Free/CC0 Sources**:
- [OpenGameArt.org](https://opengameart.org/) - CC0 game music
- [Pixabay Music](https://pixabay.com/music/) - Royalty-free
- [FreePD](https://freepd.com/) - Public domain
- [Incompetech](https://incompetech.com/) - CC-BY (requires attribution)

**Paid Sources** (if budget allows):
- [Epidemic Sound](https://www.epidemicsound.com/) - Subscription
- [Artlist](https://artlist.io/) - Subscription
- [AudioJungle](https://audiojungle.net/) - Per-track purchase

### C.3 Technical Implementation

**Asset Location**: `/static/music/`

**New Files**:
- `src/lib/audio/music.ts` - Music manager
- Update `src/lib/stores/audioSettings.svelte.ts` - Add music volume

**Music Manager API**:
```typescript
// music.ts
export function playMusic(track: MusicTrack): void;
export function stopMusic(): void;
export function fadeToTrack(track: MusicTrack, duration: number): void;
export function setMusicVolume(volume: number): void;

type MusicTrack = 'menu' | 'battle' | 'spectator' | 'victory' | 'defeat';
```

**Integration Points**:
- `MainMenu.svelte` - Start menu music
- `GameBoard.svelte` - Switch to battle music
- `SpectatorPlayback.svelte` - Switch to spectator music
- `GameOverScreen.svelte` - Play victory/defeat sting

**Settings Addition**:
- Music Volume slider (separate from SFX)
- Music mute toggle

### C.4 Acceptance Criteria

- [x] Music tracks sourced (5 tracks minimum)
- [x] Music manager implemented
- [x] Smooth transitions between tracks
- [x] Volume controls in settings
- [x] Music persists correctly across screens
- [x] Proper attribution if required by license (all CC0/Pixabay)

### C.5 Implementation Complete

**Assets Created (5 tracks):**
- `/static/music/menu.mp3` - Epic orchestral menu music (CC0)
- `/static/music/battle.mp3` - Tense battle music (CC0)
- `/static/music/spectator.mp3` - Calm analytical spectator music (CC0)
- `/static/music/victory.wav` - Triumphant victory sting (CC0)
- `/static/music/defeat.mp3` - Somber defeat sting (CC0)

**Files Created:**
- `src/lib/audio/music.ts` - Music manager with fade-in/out, sting support

**Files Modified:**
- `src/lib/audio/index.ts` - Added music exports
- `src/lib/stores/audioSettings.svelte.ts` - Added musicVolume property
- `src/lib/components/MainMenu.svelte` - Plays menu track on mount
- `src/lib/components/GameBoard.svelte` - Plays battle track on mount
- `src/lib/components/SpectatorPlayback.svelte` - Plays spectator track on mount
- `src/lib/components/GameOverScreen.svelte` - Plays victory/defeat stings
- `src/lib/components/SettingsScreen.svelte` - Added Music Volume slider

---

## ✅ Sub-Phase D: SFX Replacement [DONE]

**Goal**: Replace procedural jsfxr sounds with high-quality recorded sound effects.

### D.1 Sound Categories

**UI Sounds** (6):
- `buttonHover` - Subtle hover feedback
- `buttonClick` - Satisfying click
- `cardHover` - Paper/card rustle
- `cardSelect` - Selection chime
- `menuOpen` - Whoosh up
- `menuClose` - Whoosh down

**Card Sounds** (4):
- `cardDraw` - Card slide
- `cardPlayCreature` - Thud/placement
- `cardPlaySpell` - Magic whoosh
- `cardPlaySupport` - Mechanical placement

**Combat Sounds** (6):
- `attackLight` - Quick swipe (1-2 damage)
- `attackMedium` - Slash (3-4 damage)
- `attackHeavy` - Heavy impact (5+ damage)
- `damage` - Impact/hurt
- `creatureDeath` - Death sound
- `heal` - Restoration sound

**Game State Sounds** (4):
- `turnStartPlayer` - Your turn chime
- `turnStartOpponent` - Opponent turn indicator
- `victory` - Win fanfare
- `defeat` - Loss sound

**Total**: ~20 sound effects + faction-specific variants

### D.2 Sourcing Options

**Free Sources**:
- [Freesound.org](https://freesound.org/) - CC0/CC-BY sounds
- [OpenGameArt.org](https://opengameart.org/art-search-advanced?field_art_type_tid[]=13) - Game SFX
- [Sonniss GDC Bundles](https://sonniss.com/gameaudiogdc) - Free annual packs
- [Zapsplat](https://www.zapsplat.com/) - Free with attribution

**Paid Sources**:
- [Epidemic Sound](https://www.epidemicsound.com/) - Subscription includes SFX
- [Artlist](https://artlist.io/) - Subscription includes SFX
- [BOOM Library](https://www.boomlibrary.com/) - Professional game SFX

### D.3 Technical Implementation

**Asset Location**: `/static/sounds/`

**File Format**:
- Format: MP3 or OGG (OGG preferred for web)
- Sample rate: 44.1kHz
- Bit depth: 16-bit
- Duration: 0.1s - 2s (short, snappy)

**Implementation Changes**:

```typescript
// audio/manager.ts changes
// Replace jsfxr generation with preloaded audio files

const soundFiles: Record<SoundEffect, string> = {
  buttonHover: '/sounds/ui/button_hover.ogg',
  buttonClick: '/sounds/ui/button_click.ogg',
  // ... etc
};

// Preload all sounds on init
export async function preloadSounds(): Promise<void> {
  for (const [effect, path] of Object.entries(soundFiles)) {
    audioCache[effect] = new Audio(path);
  }
}
```

**Fallback Option**:
- Keep jsfxr as fallback if audio file fails to load
- Graceful degradation

### D.4 Acceptance Criteria

- [x] All 20+ sound effects sourced (314 files from 4 CC0 packs)
- [x] Proper licensing/attribution documented (all CC0)
- [x] Sounds feel cohesive (similar style/quality)
- [x] Random variation for replay value
- [x] Faction-specific sounds implemented
- [x] File sizes reasonable (OGG format)

### D.5 Implementation Complete

**Sound Packs Downloaded (CC0):**
- Kenney Casino Audio - Card slides, places, shuffles (53 files)
- Kenney Interface Sounds - UI clicks, chimes (99 files)
- 80 CC0 RPG SFX - Combat, spells, impacts (79 files)
- 80 CC0 Creature SFX - Creature voices, deaths (80 files)

**Asset Location**: `/static/sounds/` with subdirectories:
- `casino/` - Card game sounds
- `interface/` - UI sounds
- `rpg/` - Combat and spell sounds
- `creatures/` - Creature sounds

**Files Modified:**
- `src/lib/audio/manager.ts` - Complete rewrite: file-based audio with random variation and faction support
- `src/lib/audio/index.ts` - Added faction sound exports
- `src/lib/stores/gameState.svelte.ts` - Integrated faction-specific sounds for attacks, deaths, summons

**Faction-Specific Sounds:**

| Faction | Attack Sounds | Summon Sounds | Death Sounds |
|---------|--------------|---------------|--------------|
| Argentum | metal, chain | lock, stones | metal crash |
| Symbiote | spit, slime | burble, bug | alien, weird |
| Obsidion | spell_fire | creature_roar | scream, monster |
| Neutral | blade | card-place | creature_die |

**New API Functions:**
- `playFactionAttackSound(damage, faction)` - Faction-specific attack based on damage
- `playFactionSummonSound(faction)` - Faction-specific creature summon
- `playFactionDeathSound(faction)` - Faction-specific creature death
- `getFactionFromCardId(cardId)` - Utility to determine faction from card ID

---

## Implementation Order

### Recommended Sequence

1. **Sub-Phase A: Rules Submenu** (Code-focused)
   - Can be done independently
   - Uses existing components
   - Provides immediate user value

2. **Sub-Phase B: Main Menu Backgrounds** (Art-focused)
   - Chris generates art assets
   - Claude provides prompts and integration code
   - Parallel work possible

3. **Sub-Phase C: Music Integration** (Audio-focused)
   - Requires sourcing decisions first
   - Code implementation straightforward
   - Can start sourcing while doing A/B

4. **Sub-Phase D: SFX Replacement** (Audio-focused)
   - Similar sourcing process to music
   - Can batch with music sourcing
   - Lower priority (procedural sounds work)

### Parallel Work Opportunities

```
Week 1:
├── Claude: Rules Submenu implementation
└── Chris: Background art generation + Music/SFX sourcing research

Week 2:
├── Claude: Background integration + Music system
└── Chris: Continue art + Finalize audio sourcing

Week 3:
├── Claude: SFX integration + Polish
└── Chris: Review and testing
```

---

## Decisions Made

1. **Music Budget**: Free sources only (CC0, Pixabay, OpenGameArt, FreePD)
2. **SFX Budget**: Free sources only (Freesound.org, OpenGameArt, Sonniss GDC bundles)
3. **Background Art Style**: Match existing card art style from `docs/lore.md`, `docs/flux-guide.md`, `data/art/style_guide.yaml`
   - Focus on the 12 Commanders and world of Omyra
   - Use existing FLUX prompts as reference
   - Chris will use Gemini with existing card art for style consistency
4. **Factions Tab**: Include lore and flavor (small touch)
5. **Priority Order**: Confirmed as A → B → C → D

---

## File Locations Summary

| Asset Type | Location |
|------------|----------|
| Menu backgrounds | `/static/backgrounds/menu/` |
| Music tracks | `/static/music/` |
| Sound effects | `/static/sounds/` |
| Rules components | `/src/lib/components/rules/` |

---

*Last Updated: January 2026*
