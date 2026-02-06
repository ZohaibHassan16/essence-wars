# Essence Wars Web Demo - Implementation Plan

**Status:** Planning
**Author:** Chris + Claude
**Created:** 2026-02-06

## Overview

This document outlines the implementation plan for a web-playable version of Essence Wars, accessible via GitHub Pages. The goal is to provide the same gameplay experience as the desktop app, with minimal code duplication.

### Design Principle

> **"One Svelte App, Two Backends"**

The Svelte 5 UI remains unchanged. Only the backend communication layer differs:
- **Desktop:** Tauri IPC → Rust backend
- **Web:** Direct WASM → Rust compiled to WebAssembly

---

## Current State Analysis

### Assets (Already Optimized)
| Type | Count | Format | Size |
|------|-------|--------|------|
| Card art | 349 | WebP | ~61 MB |
| Audio | 234 | OGG | ~44 MB |
| Other | 7 | PNG | <1 MB |
| **Total** | 590 | - | **~106 MB** |

### Existing WASM Support
The `cardgame` crate already has WASM feature flags:
```toml
wasm = ["dep:wasm-bindgen", "dep:js-sys", "dep:web-sys", "dep:console_error_panic_hook"]
web = ["wasm"]
```

### API Surface
All Tauri commands are centralized in two files:
- `src/lib/api/game.ts` (20 functions)
- `src/lib/api/deckBuilder.ts` (7 functions)

---

## Architecture

### Directory Structure (Changes)

```
crates/essence-wars-ui/
├── src/
│   ├── lib/
│   │   ├── api/
│   │   │   ├── types.ts              # Unchanged - shared types
│   │   │   ├── interface.ts          # NEW: GameBackend interface
│   │   │   ├── backends/
│   │   │   │   ├── tauri.ts          # Tauri IPC implementation
│   │   │   │   ├── wasm.ts           # WASM implementation
│   │   │   │   └── index.ts          # Runtime detection + factory
│   │   │   ├── game.ts               # Modified: uses backend
│   │   │   └── deckBuilder.ts        # Modified: uses backend
│   │   ├── assets/
│   │   │   ├── loader.svelte.ts      # NEW: Progressive asset loader
│   │   │   └── preload.ts            # NEW: Essential asset preloading
│   │   └── storage/
│   │       ├── interface.ts          # NEW: Storage abstraction
│   │       ├── filesystem.ts         # Tauri filesystem storage
│   │       └── indexeddb.ts          # Web IndexedDB storage
│   └── ...
├── static/                           # Unchanged
├── src-tauri/                        # Unchanged
├── src-wasm/                         # NEW: WASM-specific Rust bindings
│   ├── Cargo.toml
│   └── src/lib.rs
└── scripts/
    └── build-web.sh                  # NEW: Web build script
```

### Backend Interface

```typescript
// src/lib/api/interface.ts
export interface GameBackend {
  // Platform identification
  readonly platform: 'tauri' | 'web';

  // Initialization
  init(): Promise<void>;

  // Core game operations
  listDecks(): Promise<DeckInfo[]>;
  listBots(): Promise<BotInfo[]>;
  newGame(config: GameConfig): Promise<GameStateDto>;
  getGameState(gameId: string): Promise<GameStateDto>;
  getLegalActions(gameId: string): Promise<ActionInfo[]>;
  applyAction(gameId: string, actionIndex: number): Promise<GameStateUpdate>;
  getAiMove(gameId: string): Promise<ActionInfo>;
  getAiHint(gameId: string): Promise<AiHintResponse>;
  endGame(gameId: string): Promise<GameResultDto>;
  undoAction(gameId: string): Promise<GameStateDto>;
  canUndo(gameId: string): Promise<boolean>;
  getDeckCards(deckId: string): Promise<CardDto[]>;

  // Spectator mode
  computeSpectatorMatch(config: SpectatorConfig): Promise<SpectatorMatch>;

  // Deck builder
  listAllCards(faction?: string): Promise<BrowsableCard[]>;
  listCommanders(): Promise<CommanderDto[]>;
  validateCustomDeck(deck: CustomDeck): Promise<DeckValidation>;
  calculateDeckPlaystyle(cards: number[], commanderId: number): Promise<PlaystyleScore>;
}

export interface StorageBackend {
  // Replays
  saveReplay(match: SpectatorMatch, name?: string): Promise<string>;
  listReplays(): Promise<ReplayInfo[]>;
  loadReplay(id: string): Promise<SpectatorMatch>;
  deleteReplay(id: string): Promise<void>;

  // Custom decks
  saveCustomDeck(deck: CustomDeck): Promise<string>;
  listCustomDecks(): Promise<CustomDeckInfo[]>;
  loadCustomDeck(id: string): Promise<CustomDeck>;
  deleteCustomDeck(id: string): Promise<void>;
}
```

### Runtime Detection

```typescript
// src/lib/api/backends/index.ts
export function detectPlatform(): 'tauri' | 'web' {
  return typeof window !== 'undefined' && '__TAURI__' in window
    ? 'tauri'
    : 'web';
}

let _backend: GameBackend | null = null;

export async function getBackend(): Promise<GameBackend> {
  if (_backend) return _backend;

  if (detectPlatform() === 'tauri') {
    const { TauriBackend } = await import('./tauri');
    _backend = new TauriBackend();
  } else {
    const { WasmBackend } = await import('./wasm');
    _backend = new WasmBackend();
  }

  await _backend.init();
  return _backend;
}
```

---

## Implementation Phases

### Phase 1: Backend Abstraction Layer
**Goal:** Refactor existing code to use abstraction without changing behavior.

#### Tasks
1. [ ] Create `src/lib/api/interface.ts` with `GameBackend` interface
2. [ ] Create `src/lib/api/backends/tauri.ts` (extract from current game.ts)
3. [ ] Create `src/lib/api/backends/index.ts` with detection logic
4. [ ] Create stub `src/lib/api/backends/wasm.ts` (throws "not implemented")
5. [ ] Update `game.ts` to use `getBackend()`
6. [ ] Update `deckBuilder.ts` to use `getBackend()`
7. [ ] Test that desktop app still works identically

#### Files Changed
- `src/lib/api/game.ts` (modified)
- `src/lib/api/deckBuilder.ts` (modified)
- `src/lib/api/interface.ts` (new)
- `src/lib/api/backends/tauri.ts` (new)
- `src/lib/api/backends/wasm.ts` (new, stub)
- `src/lib/api/backends/index.ts` (new)

#### Verification
```bash
pnpm tauri:dev  # Desktop app works as before
pnpm run check  # No type errors
```

---

### Phase 2: Storage Abstraction
**Goal:** Abstract replay and custom deck storage for cross-platform support.

#### Tasks
1. [ ] Create `src/lib/storage/interface.ts` with `StorageBackend`
2. [ ] Create `src/lib/storage/filesystem.ts` (Tauri implementation)
3. [ ] Create `src/lib/storage/indexeddb.ts` (Web implementation)
4. [ ] Update replay-related functions to use storage abstraction
5. [ ] Update custom deck functions to use storage abstraction
6. [ ] Test desktop replay save/load still works

#### Web Storage Design
```typescript
// IndexedDB schema
const DB_NAME = 'essence-wars';
const DB_VERSION = 1;

interface EssenceWarsDB {
  replays: {
    key: string;
    value: {
      id: string;
      name: string;
      timestamp: number;
      data: SpectatorMatch;
    };
  };
  customDecks: {
    key: string;
    value: CustomDeck;
  };
  settings: {
    key: string;
    value: unknown;
  };
}
```

---

### Phase 3: WASM Engine Bindings
**Goal:** Create complete WASM bindings for the game engine.

#### Tasks
1. [ ] Create `src-wasm/` crate with wasm-bindgen
2. [ ] Implement `WasmGameManager` mirroring Tauri's `GameManager`
3. [ ] Expose all required functions via `#[wasm_bindgen]`
4. [ ] Handle game state management in WASM
5. [ ] Implement AI with reduced simulation count for web
6. [ ] Build and test WASM module locally

#### WASM Crate Structure
```rust
// src-wasm/src/lib.rs
use wasm_bindgen::prelude::*;
use cardgame::{Game, Bot, DeckManager};

#[wasm_bindgen]
pub struct WasmGameManager {
    games: HashMap<String, Game>,
    deck_manager: DeckManager,
}

#[wasm_bindgen]
impl WasmGameManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmGameManager, JsError> {
        console_error_panic_hook::set_once();
        Ok(Self {
            games: HashMap::new(),
            deck_manager: DeckManager::new()?,
        })
    }

    pub fn list_decks(&self) -> JsValue {
        // Return JSON-serialized deck list
    }

    pub fn new_game(&mut self, config: JsValue) -> Result<JsValue, JsError> {
        // Create new game, return initial state
    }

    // ... etc
}
```

#### Web AI Configuration
```typescript
// Reduced settings for WASM performance
const WEB_AI_CONFIG = {
  mctsSimulations: 200,      // vs 1000 desktop
  alphabetaDepth: 4,         // vs 6 desktop
  greedyBotOnly: false,      // greedy is fast, keep all bots
};
```

---

### Phase 4: WASM Backend Implementation
**Goal:** Connect Svelte UI to WASM engine.

#### Tasks
1. [ ] Implement `WasmBackend` class in `backends/wasm.ts`
2. [ ] Handle WASM module initialization
3. [ ] Convert between JS objects and WASM types
4. [ ] Add loading state for WASM initialization
5. [ ] Test all game functions in browser

#### WASM Backend Implementation
```typescript
// src/lib/api/backends/wasm.ts
import init, { WasmGameManager } from '../../wasm/essence_wars_wasm';

export class WasmBackend implements GameBackend {
  readonly platform = 'web' as const;
  private manager: WasmGameManager | null = null;

  async init(): Promise<void> {
    await init();  // Initialize WASM module
    this.manager = new WasmGameManager();
  }

  async listDecks(): Promise<DeckInfo[]> {
    return JSON.parse(this.manager!.list_decks());
  }

  async newGame(config: GameConfig): Promise<GameStateDto> {
    return JSON.parse(this.manager!.new_game(JSON.stringify(config)));
  }

  // ... etc
}
```

---

### Phase 5: Progressive Asset Loading
**Goal:** Implement lazy loading for optimal web performance.

#### Tasks
1. [ ] Create `src/lib/assets/loader.svelte.ts` using Svelte 5 runes
2. [ ] Implement essential asset preloading
3. [ ] Add intersection observer for card art lazy loading
4. [ ] Add background audio loading
5. [ ] Create loading indicators for components
6. [ ] Update card components to use loader

#### Asset Loading Strategy
```
Initial Load (~3-5 MB):
├── WASM bundle (~1-2 MB gzipped)
├── UI assets (~500 KB)
├── Commander portraits (12 × ~50 KB = 600 KB)
└── Essential SFX (~500 KB)

On-Demand (as needed):
├── Card art (loaded when viewing deck/hand)
├── Music (loads during gameplay)
└── Additional SFX (loads in background)
```

#### Loader Implementation
```typescript
// src/lib/assets/loader.svelte.ts
import { SvelteMap } from 'svelte/reactivity';

class AssetLoader {
  private loaded = new SvelteMap<string, boolean>();
  private loading = new SvelteMap<string, Promise<void>>();

  isLoaded(path: string): boolean {
    return this.loaded.get(path) ?? false;
  }

  async loadCard(cardId: number): Promise<string> {
    const path = `/cards/${cardId}.webp`;
    if (!this.loaded.has(path)) {
      await this.loadImage(path);
    }
    return path;
  }

  private async loadImage(path: string): Promise<void> {
    if (this.loading.has(path)) {
      return this.loading.get(path);
    }

    const promise = new Promise<void>((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        this.loaded.set(path, true);
        resolve();
      };
      img.onerror = reject;
      img.src = path;
    });

    this.loading.set(path, promise);
    return promise;
  }
}

export const assetLoader = new AssetLoader();
```

---

### Phase 6: SvelteKit Static Build
**Goal:** Configure SvelteKit for static site generation.

#### Tasks
1. [ ] Update `svelte.config.js` for static adapter
2. [ ] Create `routes/+layout.ts` with `prerender = true`
3. [ ] Handle dynamic routes appropriately
4. [ ] Configure base path for GitHub Pages
5. [ ] Test static build locally

#### SvelteKit Configuration
```javascript
// svelte.config.js
import adapter from '@sveltejs/adapter-static';

export default {
  kit: {
    adapter: adapter({
      pages: 'build-web',
      assets: 'build-web',
      fallback: 'index.html',  // SPA mode
    }),
    paths: {
      base: process.env.BASE_PATH || '',  // '/essence-wars' for GH Pages
    },
  },
};
```

```typescript
// src/routes/+layout.ts
export const prerender = true;
export const ssr = false;  // Client-side only
```

---

### Phase 7: GitHub Pages Deployment
**Goal:** Automated deployment via GitHub Actions.

#### Tasks
1. [ ] Create `.github/workflows/deploy-web.yml`
2. [ ] Build WASM module in CI
3. [ ] Build SvelteKit static site
4. [ ] Deploy to `gh-pages` branch
5. [ ] Configure custom domain (optional)
6. [ ] Add deployment status badge

#### Workflow
```yaml
# .github/workflows/deploy-web.yml
name: Deploy Web Demo

on:
  push:
    branches: [master]
    paths:
      - 'crates/cardgame/**'
      - 'crates/essence-wars-ui/**'
      - 'data/**'
  workflow_dispatch:

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      - name: Build WASM
        run: |
          cd crates/essence-wars-ui/src-wasm
          wasm-pack build --target web --release

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Setup pnpm
        uses: pnpm/action-setup@v3
        with:
          version: 9

      - name: Install dependencies
        working-directory: crates/essence-wars-ui
        run: pnpm install

      - name: Build static site
        working-directory: crates/essence-wars-ui
        run: pnpm build:web
        env:
          BASE_PATH: '/essence-wars'

      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v4
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: crates/essence-wars-ui/build-web
```

---

## Feature Comparison

| Feature | Desktop | Web | Implementation Notes |
|---------|---------|-----|---------------------|
| Play vs AI | ✅ | ✅ | Same experience |
| Spectator Mode | ✅ | ✅ | Same experience |
| AI Hint | ✅ | ✅ | Reduced simulations on web |
| Deck Builder | ✅ | ✅ | Same experience |
| Custom Decks | ✅ | ✅ | IndexedDB on web |
| Save Replays | ✅ | ✅ | IndexedDB on web |
| Undo/Redo | ✅ | ✅ | Same experience |
| Music/SFX | ✅ | ✅ | Lazy-loaded on web |
| MCP Sync | ✅ | ❌ | Desktop-only (not applicable) |
| Offline Play | ✅ | ✅ | Service Worker caches assets |
| Auto-updates | ✅ | ✅ | Web always latest |

---

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Initial load | <5 MB | WASM + UI + essential assets |
| Time to interactive | <3s | On 4G connection |
| Card art lazy load | <100ms | Per card on-demand |
| AI move (MCTS 200) | <2s | WASM is ~2-3x slower |
| AI move (Greedy) | <50ms | Near-instant |
| Full asset cache | ~106 MB | After all assets loaded |

---

## Testing Strategy

### Unit Tests
- Backend abstraction interface compliance
- WASM bindings correctness
- Storage backend operations

### Integration Tests
- Full game flow in browser
- Replay save/load cycle
- Custom deck persistence

### E2E Tests (Optional)
- Playwright tests for critical paths
- Cross-browser testing (Chrome, Firefox, Safari)

---

## Rollout Plan

1. **Alpha** (internal): Deploy to `https://christianwissmann85.github.io/essence-wars/`
2. **Beta** (limited): Share with playtesters, gather feedback
3. **Public**: Add link from main README, announce

---

## Decisions

1. **Service Worker**: Deferred - add offline support in a future phase
2. **Analytics**: Yes - add basic usage tracking for insights
3. **URL**: Use default GitHub Pages URL (`christianwissmann85.github.io/essence-wars`)

---

## Success Criteria

- [ ] Web demo loads in <5s on average connection
- [ ] All game modes work (Play, Spectator, Replay)
- [ ] Replays persist across browser sessions
- [ ] No visual differences from desktop version
- [ ] Works in Chrome, Firefox, Safari, Edge
