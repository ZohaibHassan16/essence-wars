# Essence Wars Desktop Client - Design Document

**Version:** 1.0 (Planning)
**Date:** January 2026
**Status:** Pre-Development

---

## 1. Overview

### 1.1 Vision
A polished desktop client for Essence Wars that brings the card game engine to life with beautiful visuals, satisfying animations, and intelligent AI opponents. Inspired by Hearthstone's polish and MTG Online's depth.

### 1.2 Game Modes
1. **Human vs AI** - Play against various AI opponents with optional AI hints
2. **AI vs AI Spectator** - Watch AI matches with speed control and thinking visualization

### 1.3 Target Platforms
- Windows 10/11
- Linux (Ubuntu 22.04+, other distros)

---

## 2. Technical Architecture

### 2.1 Tech Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Framework | Tauri 2.0 | Desktop app shell, Rust backend |
| Frontend | Svelte 5 + TypeScript | Reactive UI components |
| Styling | Tailwind CSS | Rapid UI development |
| Animations | GSAP | Professional card/combat animations |
| Audio | Howler.js | Cross-platform sound management |
| State | Svelte stores | Reactive game state |
| Build | Vite | Fast dev server and bundling |

### 2.2 Project Structure

```
ai-cardgame/
├── crates/
│   ├── cardgame/                    # Existing game engine
│   └── essence-wars-ui/             # Tauri desktop client
│       ├── Cargo.toml
│       ├── src/                     # Rust backend
│       │   ├── main.rs              # Tauri entry point
│       │   ├── lib.rs               # Library root
│       │   ├── commands/            # IPC command handlers
│       │   │   ├── mod.rs
│       │   │   ├── game.rs          # Game lifecycle commands
│       │   │   ├── action.rs        # Player action commands
│       │   │   ├── ai.rs            # AI-related commands
│       │   │   └── replay.rs        # Replay system commands
│       │   ├── state/               # Game state management
│       │   │   ├── mod.rs
│       │   │   ├── game_manager.rs  # Active game tracking
│       │   │   └── serialization.rs # State → JSON conversion
│       │   └── ai/                  # AI interface layer
│       │       ├── mod.rs
│       │       ├── hint.rs          # AI hint generation
│       │       └── thinking.rs      # MCTS visualization data
│       ├── src-ui/                  # Svelte frontend
│       │   ├── app.html
│       │   ├── app.css
│       │   ├── lib/
│       │   │   ├── components/
│       │   │   │   ├── board/       # Game board components
│       │   │   │   │   ├── Board.svelte
│       │   │   │   │   ├── CreatureSlot.svelte
│       │   │   │   │   ├── SupportSlot.svelte
│       │   │   │   │   └── PlayerArea.svelte
│       │   │   │   ├── card/        # Card components
│       │   │   │   │   ├── Card.svelte
│       │   │   │   │   ├── CardFrame.svelte
│       │   │   │   │   ├── CardPreview.svelte
│       │   │   │   │   └── Hand.svelte
│       │   │   │   ├── ui/          # General UI
│       │   │   │   │   ├── Button.svelte
│       │   │   │   │   ├── HealthBar.svelte
│       │   │   │   │   ├── ActionPoints.svelte
│       │   │   │   │   └── TurnIndicator.svelte
│       │   │   │   ├── ai/          # AI visualization
│       │   │   │   │   ├── AiHintPanel.svelte
│       │   │   │   │   ├── AiThinkingOverlay.svelte
│       │   │   │   │   └── MctsTree.svelte
│       │   │   │   └── screens/     # Full screens
│       │   │   │       ├── MainMenu.svelte
│       │   │   │       ├── DeckSelect.svelte
│       │   │   │       ├── GameScreen.svelte
│       │   │   │       ├── SpectatorScreen.svelte
│       │   │   │       └── Settings.svelte
│       │   │   ├── stores/          # Svelte stores
│       │   │   │   ├── game.ts      # Current game state
│       │   │   │   ├── ui.ts        # UI state (modals, etc)
│       │   │   │   ├── settings.ts  # User preferences
│       │   │   │   └── audio.ts     # Audio state
│       │   │   ├── animations/      # GSAP animation configs
│       │   │   │   ├── card.ts
│       │   │   │   ├── combat.ts
│       │   │   │   └── effects.ts
│       │   │   ├── audio/           # Sound management
│       │   │   │   ├── manager.ts
│       │   │   │   └── tracks.ts
│       │   │   ├── types/           # TypeScript types
│       │   │   │   ├── game.ts
│       │   │   │   ├── card.ts
│       │   │   │   └── ipc.ts
│       │   │   └── utils/           # Utilities
│       │   │       ├── tauri.ts     # IPC wrappers
│       │   │       └── format.ts    # Display formatting
│       │   └── routes/
│       │       └── +page.svelte     # Main entry
│       ├── static/                  # Static assets
│       │   ├── cards/               # Card art (symlinked or copied)
│       │   ├── frames/              # Card frame images
│       │   ├── icons/               # Keyword & UI icons
│       │   ├── backgrounds/         # Board backgrounds
│       │   ├── sounds/              # Sound effects
│       │   └── music/               # Background music
│       └── src-tauri/
│           ├── tauri.conf.json
│           ├── capabilities/
│           └── icons/
```

### 2.3 IPC Commands

#### Game Lifecycle
```rust
#[tauri::command]
fn list_decks() -> Vec<DeckInfo>;

#[tauri::command]
fn list_bots() -> Vec<BotInfo>;

#[tauri::command]
fn new_game(config: GameConfig) -> Result<GameId, GameError>;

#[tauri::command]
fn get_game_state(game_id: GameId) -> Result<GameStateDto, GameError>;

#[tauri::command]
fn end_game(game_id: GameId) -> Result<GameResult, GameError>;
```

#### Player Actions
```rust
#[tauri::command]
fn get_legal_actions(game_id: GameId) -> Result<Vec<ActionInfo>, GameError>;

#[tauri::command]
fn apply_action(game_id: GameId, action_index: u8) -> Result<GameStateUpdate, GameError>;

#[tauri::command]
fn undo_action(game_id: GameId) -> Result<GameStateUpdate, GameError>;  // Dev mode only
```

#### AI System
```rust
#[tauri::command]
fn get_ai_hint(game_id: GameId, depth: HintDepth) -> Result<AiHintResponse, GameError>;

#[tauri::command]
fn get_ai_move(game_id: GameId) -> Result<AiMoveResponse, GameError>;

#[tauri::command]
async fn compute_ai_thinking(game_id: GameId) -> Result<AiThinkingData, GameError>;
```

#### AI vs AI Spectator
```rust
#[tauri::command]
fn start_spectator_match(config: SpectatorConfig) -> Result<MatchId, GameError>;

#[tauri::command]
fn step_match(match_id: MatchId) -> Result<MatchStepResult, GameError>;

#[tauri::command]
fn set_match_speed(match_id: MatchId, speed: f32) -> Result<(), GameError>;

#[tauri::command]
fn pause_match(match_id: MatchId) -> Result<(), GameError>;

#[tauri::command]
fn resume_match(match_id: MatchId) -> Result<(), GameError>;
```

#### Replay System
```rust
#[tauri::command]
fn save_replay(game_id: GameId, name: Option<String>) -> Result<ReplayPath, GameError>;

#[tauri::command]
fn list_replays() -> Result<Vec<ReplayInfo>, GameError>;

#[tauri::command]
fn load_replay(path: String) -> Result<Replay, GameError>;

#[tauri::command]
fn delete_replay(path: String) -> Result<(), GameError>;
```

### 2.4 Data Transfer Objects

```typescript
// Game state sent to frontend
interface GameStateDto {
  id: string;
  turn: number;
  phase: Phase;
  activePlayer: 1 | 2;

  player1: PlayerStateDto;
  player2: PlayerStateDto;

  isGameOver: boolean;
  winner?: 1 | 2 | null;  // null = draw
}

interface PlayerStateDto {
  life: number;
  maxLife: number;
  actionPoints: number;
  maxActionPoints: number;
  deckCount: number;

  hand: CardDto[];           // Full info for human, hidden for AI
  creatures: (CreatureDto | null)[];  // 5 slots
  supports: (SupportDto | null)[];    // 2 slots
}

interface CardDto {
  instanceId: string;        // Unique instance ID
  cardId: number;            // Card definition ID (e.g., 1001)
  name: string;
  cost: number;
  cardType: 'creature' | 'spell' | 'support';
  faction: Faction;

  // Creature stats (if applicable)
  attack?: number;
  health?: number;
  maxHealth?: number;
  keywords?: Keyword[];

  // Support stats (if applicable)
  durability?: number;
  maxDurability?: number;

  // Art
  artPath?: string;

  // Ability text (shown on hover)
  abilityText?: string;
}

interface ActionInfo {
  index: number;             // Action space index (0-255)
  type: ActionType;
  description: string;       // Human-readable: "Play Steam Knight to slot 2"

  // For targeting visualization
  sourceSlot?: number;
  targetSlot?: number;
  cardId?: number;
}

interface AiHintResponse {
  recommendedAction: ActionInfo;
  score: number;
  reasoning: string[];

  alternatives: {
    action: ActionInfo;
    score: number;
    scoreDelta: number;
    whyWorse: string;
  }[];

  mctsData?: {
    totalSimulations: number;
    thinkingTimeMs: number;
    topMoves: {
      action: ActionInfo;
      visits: number;
      winRate: number;
    }[];
  };
}
```

---

## 3. Visual Design

### 3.1 Card Frames

Each faction has a distinct hand-painted frame style:

#### Argentum Combine
- **Shape:** Geometric, Art Deco inspired with precise angles
- **Colors:** White marble, gold filigree, brass accents
- **Motifs:** Gear teeth along edges, sunburst patterns, riveted corners
- **Texture:** Polished metal with subtle reflection

#### Symbiote Circles
- **Shape:** Organic, flowing curves with irregular edges
- **Colors:** Deep forest green, purple bioluminescence, bone white
- **Motifs:** Vine tendrils, chitin plates, spore clusters, breathing holes
- **Texture:** Living tissue with subtle pulsing glow

#### Obsidion Syndicate
- **Shape:** Gothic Victorian, ornate with sharp points
- **Colors:** Crimson velvet, obsidian black, neon blue essence
- **Motifs:** Wrought iron filigree, blood drops, arcane runes, crystal shards
- **Texture:** Dark velvet with metallic inlay

#### Free-Walkers (Neutral)
- **Shape:** Practical, weathered, utilitarian
- **Colors:** Brown leather, tarnished copper, sand tan
- **Motifs:** Leather straps, worn buckles, guild emblems, road dust
- **Texture:** Aged leather with metal reinforcement

### 3.2 Card Layout

```
┌─────────────────────────────┐
│ [Cost]     Card Name        │  ← Name banner
├─────────────────────────────┤
│                             │
│                             │
│         Card Art            │  ← Art window (main focus)
│                             │
│                             │
├─────────────────────────────┤
│  [Keyword Icons]            │  ← Below art
├─────────────────────────────┤
│  Creature / Spell / Support │  ← Type bar
├─────────────────────────────┤
│ [ATK]               [HP]    │  ← Stats (creatures only)
└─────────────────────────────┘
```

Card dimensions: **280 x 400 pixels** (0.7 aspect ratio, matches generation)

### 3.3 Board Layout

```
┌────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  [Deck]  [Life: 30]  [AP: ●●●]     [Support1][Support2]  │  │  ← AI Info Bar
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
│     ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐              │
│     │  1  │  │  2  │  │  3  │  │  4  │  │  5  │              │  ← AI Creatures
│     └─────┘  └─────┘  └─────┘  └─────┘  └─────┘              │
│                                                                │
│  ═══════════════════════════════════════════════════════════  │  ← Divider
│                                                                │
│     ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐              │
│     │  1  │  │  2  │  │  3  │  │  4  │  │  5  │              │  ← Player Creatures
│     └─────┘  └─────┘  └─────┘  └─────┘  └─────┘              │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  [Support1][Support2]    [AP: ●●●]  [Life: 30]  [Deck]   │  │  ← Player Info Bar
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                     Player Hand                           │  │  ← Hand (scrollable)
│  │   [Card][Card][Card][Card][Card][Card]...                 │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
│  [End Turn]  [AI Hint]  [Menu]                                │  ← Action Bar
└────────────────────────────────────────────────────────────────┘
```

### 3.4 Board Backgrounds

Subtle animated backgrounds for each faction matchup:

| Background | Theme | Subtle Animation |
|------------|-------|------------------|
| Argentum | White marble factory floor | Steam wisps, gear shadows rotating |
| Symbiote | Bioluminescent jungle clearing | Floating spores, vine sway |
| Obsidion | Gothic cathedral interior | Candle flicker, essence flow |
| Neutral | Desert trading post | Dust motes, torch flicker |
| Mixed | Contested borderlands | Combines elements from both factions |

### 3.5 Keyword Icons

14 distinct icons needed:

| Keyword | Icon Concept |
|---------|--------------|
| Rush | Lightning bolt / charging horse |
| Ranged | Crosshair / bow |
| Piercing | Armor-breaking arrow |
| Guard | Shield |
| Lifesteal | Dripping blood drop |
| Lethal | Skull |
| Shield | Bubble / energy barrier |
| Quick | Double arrows |
| Ephemeral | Fading ghost |
| Regenerate | Green heart with plus |
| Stealth | Eye with slash |
| Charge | Stacked lightning |
| Frenzy | Spiral rage marks |
| Volatile | Explosion / unstable energy |

---

## 4. Animation System

### 4.1 Card Animations

| Animation | Duration | Description |
|-----------|----------|-------------|
| Card Draw | 400ms | Card slides from deck, flips to reveal |
| Card Hover | 150ms | Slight lift, scale to 1.1x |
| Card Select | 200ms | Glow outline, lift higher |
| Card Play | 500ms | Arc from hand to board slot |
| Card Return | 300ms | Shrink back to hand if cancelled |

### 4.2 Combat Animations

| Animation | Duration | Description |
|-----------|----------|-------------|
| Attack Declare | 200ms | Attacker glows, pulses |
| Attack Execute | 400ms | Attacker lunges toward target |
| Damage Dealt | 300ms | Target shakes, damage number pops |
| Creature Death | 500ms | Fade out with particle burst |
| Lifesteal | 400ms | Green particles flow to attacker |

### 4.3 Spell Effects

| Effect Type | Visual |
|-------------|--------|
| Damage | Red impact particles |
| Heal | Green rising particles |
| Buff | Golden shimmer on target |
| Debuff | Purple drain effect |
| Draw | Card silhouettes flying to hand |
| Summon | Creature materializes with glow |

### 4.4 Keyword Triggers

| Keyword | Visual Trigger |
|---------|----------------|
| Guard | Shield icon flashes when protecting |
| Rush | Speed lines on summon |
| Lifesteal | Blood droplets on damage |
| Shield | Bubble pop when broken |
| Regenerate | Green pulse at turn start |
| Stealth | Creature partially transparent |

---

## 5. Audio Design

### 5.1 Music System

**Dynamic Orchestral Track:**
- Base layer: Epic orchestral theme (looping, ~3-5 minutes)
- Intensity layers that crossfade based on game state:
  - **Calm:** Early game, building board
  - **Tension:** Mid-game, both players healthy
  - **Climax:** Either player below 10 life
  - **Victory:** Triumphant swell on win
  - **Defeat:** Somber fade on loss

**Faction Ambient Undertones:**
- Argentum: Industrial hum, distant machinery, steam hisses
- Symbiote: Jungle sounds, creature calls, organic squelches
- Obsidion: Gothic choir whispers, ethereal drones, crystal tones
- Mixed based on player faction vs opponent faction

### 5.2 Sound Effects

#### UI Sounds
| Event | Sound |
|-------|-------|
| Button hover | Soft tick |
| Button click | Satisfying click |
| Card hover | Paper rustle |
| Menu open | Whoosh |
| Menu close | Reverse whoosh |

#### Card Sounds
| Event | Sound |
|-------|-------|
| Card draw | Card slide + flip |
| Card play (creature) | Thud + faction-specific |
| Card play (spell) | Magic whoosh |
| Card play (support) | Mechanical clunk |
| Card select | Subtle chime |

#### Combat Sounds
| Event | Sound |
|-------|-------|
| Attack (light, 1-2 dmg) | Quick swipe |
| Attack (medium, 3-4 dmg) | Heavy slash |
| Attack (heavy, 5+ dmg) | Crushing impact |
| Creature death | Faction-specific death cry |
| Lethal trigger | Sinister sting |
| Lifesteal trigger | Slurping drain |

#### Game State Sounds
| Event | Sound |
|-------|-------|
| Turn start (yours) | Bell chime |
| Turn start (opponent) | Lower bell |
| Low health warning | Heartbeat |
| Victory | Triumphant fanfare |
| Defeat | Somber sting |

### 5.3 Volume Controls

Settings panel with separate sliders:
- Master Volume
- Music Volume
- SFX Volume
- Ambient Volume
- Mute All toggle

---

## 6. Game Modes

### 6.1 Human vs AI

#### Deck Selection Screen
- Grid of available decks (12 total, 4 per faction)
- Deck preview: Commander card, deck name, description
- Hover shows full deck list
- Filter by faction

#### AI Selection
- Bot type dropdown: Greedy, MCTS, Agent (faction specialists)
- Difficulty hints: "Greedy: Fast but predictable", "MCTS: Strong tactical play"

#### Gameplay Flow
1. Player sees their hand, board, and AI's board (not AI's hand)
2. On player turn:
   - Legal actions highlighted (playable cards glow, valid targets pulse)
   - Click card → click target/slot to play
   - Click creature → click enemy to attack
   - "End Turn" button always visible
   - "AI Hint" button shows recommended move
3. On AI turn:
   - Brief thinking indicator
   - AI actions animate with slight delay for readability
   - Actions appear in log

#### AI Hint Panel
When "AI Hint" clicked:
```
┌─────────────────────────────────────────┐
│ 💡 AI Recommendation                    │
├─────────────────────────────────────────┤
│ Play "Steam Knight" to Slot 2           │
│                                         │
│ Score: +2.3                             │
│                                         │
│ Reasoning:                              │
│ • Develops board presence (+1.2)        │
│ • Guards against lethal next turn (+0.8)│
│ • Good stat efficiency for cost (+0.3)  │
├─────────────────────────────────────────┤
│ Alternatives:                           │
│ • Attack with Broodling → Face (-0.5)   │
│   "Trades poorly, loses tempo"          │
│ • End Turn (-1.8)                       │
│   "Wastes action points"                │
├─────────────────────────────────────────┤
│ MCTS: 1,247 simulations in 340ms        │
│ Win rate for this move: 62%             │
└─────────────────────────────────────────┘
```

### 6.2 AI vs AI Spectator

#### Match Setup
- Select Deck 1 + Bot 1 (any combination)
- Select Deck 2 + Bot 2 (any combination)
- Optional: Set random seed for reproducibility

#### Spectator Controls
```
┌────────────────────────────────────────┐
│  ⏮  ⏪  ⏸/▶  ⏩  ⏭                    │
│  [|<] [<] [||/▶] [>] [>|]              │
│                                        │
│  Speed: [====●=====] 1.0x              │
│         0.25x            4.0x          │
│                                        │
│  ☑ Show AI Thinking                    │
│  ☑ Auto-pause on Combat                │
└────────────────────────────────────────┘
```

Controls:
- **⏮** Jump to start
- **⏪** Previous action (step back)
- **⏸/▶** Pause / Resume
- **⏩** Next action (step forward)
- **⏭** Jump to end
- **Speed slider**: 0.25x to 4x playback

#### AI Thinking Visualization
When "Show AI Thinking" enabled:
- Sidebar shows current player's evaluation
- Top 3-5 considered moves with visit counts
- Win rate bars for each option
- Brief explanation of chosen move

```
┌─────────────────────────────┐
│ 🤖 MCTS Thinking (Player 1) │
├─────────────────────────────┤
│ Simulations: 2,048          │
│                             │
│ 1. Attack Slot2 → Face      │
│    ████████░░ 847 visits    │
│    Win: 58%                 │
│                             │
│ 2. Play "Reinforce"         │
│    █████░░░░░ 512 visits    │
│    Win: 51%                 │
│                             │
│ 3. End Turn                 │
│    ██░░░░░░░░ 203 visits    │
│    Win: 34%                 │
│                             │
│ ✓ Chose: Attack → Face      │
│   "Threatens lethal"        │
└─────────────────────────────┘
```

---

## 7. Replay System

### 7.1 File Format

```json
{
  "version": "1.0",
  "timestamp": "2026-01-21T15:30:00Z",
  "gameId": "uuid-here",

  "config": {
    "player1": {
      "deckId": "broodmother_swarm",
      "deckName": "The Broodmother",
      "isHuman": true
    },
    "player2": {
      "deckId": "sovereign_lifesteal",
      "deckName": "The Blood Sovereign",
      "botType": "mcts"
    }
  },

  "initialSeed": 12345,

  "actions": [
    {
      "turn": 1,
      "player": 1,
      "actionIndex": 42,
      "actionDescription": "Play Broodling to Slot 1",
      "timestampMs": 3420,
      "stateAfter": { /* snapshot */ },
      "aiThinking": null
    },
    {
      "turn": 1,
      "player": 2,
      "actionIndex": 15,
      "actionDescription": "Play Blood Acolyte to Slot 2",
      "timestampMs": 850,
      "stateAfter": { /* snapshot */ },
      "aiThinking": {
        "simulations": 1024,
        "topMoves": [...]
      }
    }
  ],

  "result": {
    "winner": 1,
    "reason": "opponent_life_zero",
    "finalTurn": 12
  }
}
```

### 7.2 Storage Location

Default: `~/.essence-wars/replays/`

Naming: `{timestamp}_{deck1}_vs_{deck2}.replay.json`

Example: `2026-01-21_1530_broodmother_vs_sovereign.replay.json`

### 7.3 Replay Viewer Features

- Full playback with same animation system
- Step forward/backward through any action
- Jump to any turn
- Speed control
- Show/hide AI thinking data
- Export to shareable format

---

## 8. Development Phases

### Phase 1: Foundation (Week 1-2)
- [x] Initialize Tauri 2.0 project in `crates/essence-wars-ui/`
- [x] Set up Svelte + Vite + Tailwind
- [x] Integrate `cardgame` crate as dependency
- [x] Implement basic IPC: `list_decks`, `new_game`, `get_game_state`
- [x] Create minimal board layout (slots, placeholders)
- [x] Test game creation and state retrieval

### Phase 2: Core Gameplay (Week 3-4)
- [ ] Implement full IPC command set
- [ ] Card component with basic styling
- [ ] Hand display with card selection
- [ ] Legal action highlighting
- [ ] Play card to slot interaction
- [ ] Attack creature interaction
- [ ] End turn flow
- [ ] Basic turn indicator and life display

### Phase 3: AI Integration (Week 5)
- [ ] AI move execution
- [ ] AI hint system with reasoning
- [ ] Turn flow (human → AI → human)
- [ ] Game over detection and display

### Phase 4: Visual Polish (Week 6-7)
- [ ] Card frame designs (all 4 factions)
- [ ] GSAP animation integration
- [ ] Card play animations
- [ ] Combat animations
- [ ] Damage/heal effects
- [ ] Death animations

### Phase 5: Art Asset Sprint (Week 8-9)
- [ ] Generate remaining card art via FLUX
- [ ] Create board backgrounds
- [ ] Design keyword icons
- [ ] UI element graphics
- [ ] Card back design

### Phase 6: Audio Implementation (Week 10)
- [ ] Howler.js integration
- [ ] Source/create sound effects
- [ ] Music system with dynamic layers
- [ ] Faction ambient sounds
- [ ] Volume controls

### Phase 7: AI vs AI Mode (Week 11)
- [ ] Spectator match setup screen
- [ ] Speed slider implementation
- [ ] Step-through controls
- [ ] AI thinking visualization panel
- [ ] Pause/resume functionality

### Phase 8: Replay System (Week 12)
- [ ] Action recording during gameplay
- [ ] Save replay to file
- [ ] Replay browser/list
- [ ] Replay playback with controls
- [ ] State reconstruction from actions

### Phase 9: Polish & Testing (Week 13-14)
- [ ] Settings screen
- [ ] Keyboard shortcuts
- [ ] Window resize handling
- [ ] Error handling and recovery
- [ ] Performance optimization
- [ ] Bug fixing

### Phase 10: Release Prep (Week 15)
- [ ] Build for Windows
- [ ] Build for Linux
- [ ] Installer/package creation
- [ ] Final testing on both platforms
- [ ] Documentation

---

## 9. Asset Inventory

### 9.1 Card Art Status

| Faction | Total Cards | Art Complete | Remaining |
|---------|-------------|--------------|-----------|
| Argentum | 75 | 22 | 53 |
| Symbiote | 75 | 15 | 60 |
| Obsidion | 75 | 19 | 56 |
| Neutral | 75 | 13 | 62 |
| **Total** | **300** | **69** | **231** |

### 9.2 Art Generation Pipeline

Using FLUX Dev model via stable-diffusion.cpp:
- Resolution: 704 x 896 (portrait, matches card aspect)
- Steps: 20 (high quality)
- LoRAs: classical-painting, frazetta, rutkowski (faction-appropriate)

See `docs/flux-guide.md` for full command templates.

### 9.3 Other Assets Needed

| Category | Items | Status |
|----------|-------|--------|
| Card frames | 4 faction frames | Not started |
| Card backs | 4 faction backs + 1 universal | Not started |
| Keyword icons | 14 icons | Not started |
| Board backgrounds | 5 backgrounds | Not started |
| UI elements | Buttons, bars, panels | Not started |
| Sound effects | ~30 sounds | Not started |
| Music | 1-3 tracks + ambient | Not started |

---

## 10. Open Questions / Future Considerations

### Deferred Features (v2.0+)
- Deck builder UI
- Online multiplayer (peer-to-peer or server)
- Tournament mode
- Achievements/statistics tracking
- Card collection/unlock system
- Custom AI training interface

### Technical Considerations
- Card art caching strategy (load all vs lazy load)
- Animation performance on lower-end machines
- Replay file compression for long games
- Accessibility (colorblind modes, screen reader support)

---

## Appendix A: Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Space | End turn / Confirm |
| Escape | Cancel / Back |
| H | Show AI hint |
| 1-5 | Select creature slot |
| Q-U | Select card in hand (left to right) |
| M | Toggle music |
| S | Toggle sound effects |

## Appendix B: Color Palette

| Faction | Primary | Secondary | Accent |
|---------|---------|-----------|--------|
| Argentum | #F5F5F5 (white) | #D4AF37 (gold) | #B87333 (brass) |
| Symbiote | #1A472A (forest) | #4A0080 (purple) | #7FFF00 (biolum) |
| Obsidion | #8B0000 (crimson) | #0D0D0D (black) | #00FFFF (essence) |
| Neutral | #8B4513 (brown) | #D2B48C (tan) | #B87333 (copper) |
| UI | #1A1A2E (bg) | #16213E (panel) | #E94560 (action) |
