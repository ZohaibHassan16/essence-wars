# Phase 5: AI vs AI Spectator Mode - Implementation Plan

**Status:** Ready for Implementation
**Date:** January 2026

---

## Overview

Implement AI vs AI spectator mode with pre-computed matches, playback controls, speed adjustment, and optional "Watch Live" mode that hides the game outcome.

### Key Design Decisions
- **Pre-compute approach**: Entire match computed before playback begins
- **"Watch Live" mode**: Hides timeline/outcome, plays at 1x without revealing end
- **Reuse existing components**: GameBoard.svelte works for both modes
- **MCTS introspection**: Capture thinking data during computation

---

## Implementation Tasks

### Task 1: Backend Data Structures

**File:** `crates/essence-wars-ui/src-tauri/src/state/spectator.rs` (new)

```rust
use serde::{Deserialize, Serialize};
use crate::state::serialization::*;

/// Configuration for starting a spectator match
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectatorConfig {
    pub player1_deck_id: String,
    pub player1_bot_type: String,
    pub player2_deck_id: String,
    pub player2_bot_type: String,
    pub seed: Option<u64>,  // Optional fixed seed for reproducibility
}

/// A single action in the spectator match with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectatorAction {
    pub turn: u16,
    pub player: u8,  // 1 or 2
    pub action: ActionInfo,
    pub state_after: GameStateDto,
    pub events: Vec<GameEventDto>,
    pub thinking: Option<MctsThinkingDto>,
    pub thinking_time_ms: u64,
}

/// MCTS thinking data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MctsThinkingDto {
    pub total_simulations: u32,
    pub top_moves: Vec<MctsMoveDto>,
    pub selected_win_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MctsMoveDto {
    pub action: ActionInfo,
    pub visits: u32,
    pub win_rate: f32,
}

/// Complete pre-computed spectator match
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectatorMatch {
    pub id: String,
    pub config: SpectatorConfig,
    pub initial_state: GameStateDto,
    pub actions: Vec<SpectatorAction>,
    pub result: SpectatorResult,
    pub total_turns: u16,
    pub player1_deck_name: String,
    pub player2_deck_name: String,
    pub player1_bot_name: String,
    pub player2_bot_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectatorResult {
    pub winner: Option<u8>,  // None = draw
    pub reason: String,
    pub player1_final_life: i16,
    pub player2_final_life: i16,
}

/// Progress update during match computation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeProgress {
    pub current_turn: u16,
    pub is_complete: bool,
    pub match_data: Option<SpectatorMatch>,
}
```

---

### Task 2: Backend IPC Commands

**File:** `crates/essence-wars-ui/src-tauri/src/commands/spectator.rs` (new)

```rust
use tauri::State;
use crate::state::GameManager;
use crate::state::spectator::*;

/// Start computing a spectator match (blocking for now, async later)
#[tauri::command]
pub fn compute_spectator_match(
    config: SpectatorConfig,
    state: State<'_, GameManager>,
) -> Result<SpectatorMatch, String> {
    state.compute_spectator_match(config)
}

/// Get available bot types for spectator mode
#[tauri::command]
pub fn list_spectator_bots(
    state: State<'_, GameManager>,
) -> Vec<BotInfo> {
    state.list_bots()
}
```

**GameManager additions** (`game_manager.rs`):

```rust
impl GameManager {
    /// Compute a full spectator match with all actions and thinking data
    pub fn compute_spectator_match(&self, config: SpectatorConfig) -> Result<SpectatorMatch, String> {
        // 1. Parse bot types
        // 2. Get decks
        // 3. Create GameClient with seed
        // 4. Loop until game over:
        //    a. Get current player's bot
        //    b. Select action (with introspection for MCTS)
        //    c. Apply action, capture events
        //    d. Store SpectatorAction with state snapshot
        // 5. Return complete SpectatorMatch
    }
}
```

---

### Task 3: Frontend Types

**File:** `crates/essence-wars-ui/src/lib/api/types.ts` (additions)

```typescript
// Spectator mode types
export interface SpectatorConfig {
  player1DeckId: string;
  player1BotType: string;
  player2DeckId: string;
  player2BotType: string;
  seed?: number;
}

export interface SpectatorAction {
  turn: number;
  player: 1 | 2;
  action: ActionInfo;
  stateAfter: GameStateDto;
  events: GameEventDto[];
  thinking: MctsThinkingDto | null;
  thinkingTimeMs: number;
}

export interface MctsThinkingDto {
  totalSimulations: number;
  topMoves: MctsMoveDto[];
  selectedWinRate: number;
}

export interface MctsMoveDto {
  action: ActionInfo;
  visits: number;
  winRate: number;
}

export interface SpectatorMatch {
  id: string;
  config: SpectatorConfig;
  initialState: GameStateDto;
  actions: SpectatorAction[];
  result: SpectatorResult;
  totalTurns: number;
  player1DeckName: string;
  player2DeckName: string;
  player1BotName: string;
  player2BotName: string;
}

export interface SpectatorResult {
  winner: 1 | 2 | null;
  reason: string;
  player1FinalLife: number;
  player2FinalLife: number;
}
```

---

### Task 4: Spectator Store

**File:** `crates/essence-wars-ui/src/lib/stores/spectatorState.svelte.ts` (new)

```typescript
import type { SpectatorMatch, SpectatorAction, GameStateDto, DeckInfo, BotInfo } from "$lib/api/types";
import * as api from "$lib/api/game";

export type SpectatorPhase = "setup" | "computing" | "watching" | "finished";

class SpectatorStore {
  // Phase management
  phase = $state<SpectatorPhase>("setup");

  // Setup data
  decks = $state<DeckInfo[]>([]);
  bots = $state<BotInfo[]>([]);

  // Match data (after computation)
  match = $state<SpectatorMatch | null>(null);

  // Playback state
  currentActionIndex = $state<number>(-1);  // -1 = initial state
  isPlaying = $state<boolean>(false);
  playbackSpeed = $state<number>(1.0);  // 0.25, 0.5, 1, 2, 4

  // "Watch Live" mode
  watchLive = $state<boolean>(false);

  // Loading/error
  isComputing = $state<boolean>(false);
  computeProgress = $state<number>(0);
  error = $state<string | null>(null);

  // Computed properties
  get currentState(): GameStateDto | null {
    if (!this.match) return null;
    if (this.currentActionIndex < 0) return this.match.initialState;
    return this.match.actions[this.currentActionIndex].stateAfter;
  }

  get currentAction(): SpectatorAction | null {
    if (!this.match || this.currentActionIndex < 0) return null;
    return this.match.actions[this.currentActionIndex];
  }

  get totalActions(): number {
    return this.match?.actions.length ?? 0;
  }

  get isAtStart(): boolean {
    return this.currentActionIndex < 0;
  }

  get isAtEnd(): boolean {
    return this.currentActionIndex >= this.totalActions - 1;
  }

  get canShowTimeline(): boolean {
    // In watch live mode, only show timeline after reaching the end
    if (this.watchLive && !this.isAtEnd) return false;
    return true;
  }

  get canShowResult(): boolean {
    // In watch live mode, only show result after reaching the end
    if (this.watchLive && !this.isAtEnd) return false;
    return true;
  }

  // Actions
  async loadDecksAndBots() { ... }

  async startMatch(config: SpectatorConfig) {
    this.phase = "computing";
    this.isComputing = true;
    this.error = null;

    try {
      const match = await api.computeSpectatorMatch(config);
      this.match = match;
      this.currentActionIndex = -1;
      this.phase = "watching";

      // Auto-start playback
      this.play();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      this.phase = "setup";
    } finally {
      this.isComputing = false;
    }
  }

  // Playback controls
  play() {
    if (this.isAtEnd) return;
    this.isPlaying = true;
    this.playNextAction();
  }

  pause() {
    this.isPlaying = false;
  }

  async playNextAction() {
    if (!this.isPlaying || this.isAtEnd) {
      this.isPlaying = false;
      if (this.isAtEnd) this.phase = "finished";
      return;
    }

    this.currentActionIndex++;
    const action = this.currentAction;

    // Trigger animations
    if (action) {
      await this.processAnimations(action);
    }

    // Schedule next action based on speed
    const baseDelay = 800;  // ms between actions at 1x
    const delay = baseDelay / this.playbackSpeed;

    setTimeout(() => this.playNextAction(), delay);
  }

  stepForward() {
    if (this.isAtEnd) return;
    this.pause();
    this.currentActionIndex++;
  }

  stepBackward() {
    if (this.isAtStart) return;
    this.pause();
    this.currentActionIndex--;
  }

  jumpToStart() {
    this.pause();
    this.currentActionIndex = -1;
  }

  jumpToEnd() {
    this.pause();
    this.currentActionIndex = this.totalActions - 1;
    this.phase = "finished";
  }

  jumpToAction(index: number) {
    if (index < -1 || index >= this.totalActions) return;
    this.pause();
    this.currentActionIndex = index;
  }

  setSpeed(speed: number) {
    this.playbackSpeed = speed;
  }

  setWatchLive(enabled: boolean) {
    this.watchLive = enabled;
  }

  reset() {
    this.phase = "setup";
    this.match = null;
    this.currentActionIndex = -1;
    this.isPlaying = false;
    this.error = null;
  }

  async processAnimations(action: SpectatorAction) {
    // Similar to gameStore but for spectator view
    // Handle creature_spawned, creature_died, attacks, etc.
  }
}

export const spectatorStore = new SpectatorStore();
```

---

### Task 5: Spectator Setup Screen

**File:** `crates/essence-wars-ui/src/lib/components/screens/SpectatorSetup.svelte` (new)

**Features:**
- Two-column layout: Player 1 (left) vs Player 2 (right)
- Deck dropdown for each player (grouped by faction)
- Bot type dropdown for each player
- "Watch Live" checkbox with tooltip
- Optional seed input (collapsed by default)
- "Start Match" button

**Wireframe:**
```
┌──────────────────────────────────────────────────────────┐
│                 AI vs AI Spectator                        │
├────────────────────────┬─────────────────────────────────┤
│       PLAYER 1         │         PLAYER 2                │
├────────────────────────┼─────────────────────────────────┤
│  Deck: [Dropdown    ▼] │  Deck: [Dropdown    ▼]          │
│  Bot:  [Dropdown    ▼] │  Bot:  [Dropdown    ▼]          │
├────────────────────────┴─────────────────────────────────┤
│  ☑ Watch Live (hide outcome until end)                   │
│  ▶ Advanced: Seed [________]                             │
├──────────────────────────────────────────────────────────┤
│              [Back]              [Start Match]           │
└──────────────────────────────────────────────────────────┘
```

---

### Task 6: Computing Screen

**File:** `crates/essence-wars-ui/src/lib/components/screens/ComputingMatch.svelte` (new)

**Features:**
- Centered loading spinner
- "Computing match..." text
- Progress indicator (turn X)
- Bot names displayed
- Cancel button

**Wireframe:**
```
┌──────────────────────────────────────────────────────────┐
│                                                          │
│                    ⟳ (spinning)                          │
│                                                          │
│               Computing Match...                         │
│                                                          │
│           MCTS Bot vs Greedy Bot                         │
│                                                          │
│                   [Cancel]                               │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

---

### Task 7: Spectator Controls Component

**File:** `crates/essence-wars-ui/src/lib/components/SpectatorControls.svelte` (new)

**Features:**
- Playback buttons: ⏮ ⏪ ⏸/▶ ⏩ ⏭
- Speed dropdown/buttons: 0.25x, 0.5x, 1x, 2x, 4x
- Timeline scrubber (hidden in Watch Live until end)
- Current action / total actions counter
- Turn indicator

**Wireframe:**
```
┌──────────────────────────────────────────────────────────┐
│  Turn 5  │  Action 23/87                                 │
├──────────────────────────────────────────────────────────┤
│  [⏮] [⏪] [⏸] [⏩] [⏭]    Speed: [0.5x][1x][2x][4x]    │
├──────────────────────────────────────────────────────────┤
│  ═══════════════●══════════════════════════════════════  │  ← Timeline (if visible)
│  0              23                                   87  │
└──────────────────────────────────────────────────────────┘
```

---

### Task 8: AI Thinking Panel

**File:** `crates/essence-wars-ui/src/lib/components/AiThinkingPanel.svelte` (new)

**Features:**
- Collapsible panel
- Current player indicator (P1 bot or P2 bot)
- Total simulations count
- Top 3-5 moves with:
  - Action description
  - Visit count (horizontal bar)
  - Win rate percentage
- Selected move highlighted
- Thinking time display

**Wireframe:**
```
┌─────────────────────────────────────┐
│ 🤖 MCTS Bot (Player 1)              │
├─────────────────────────────────────┤
│ Simulations: 1,024                  │
│ Time: 245ms                         │
├─────────────────────────────────────┤
│ ✓ Attack Slot 2 → Face              │
│   ████████████░░░░ 612 (58%)        │
│                                     │
│   Play "Reinforce"                  │
│   ████████░░░░░░░░ 298 (51%)        │
│                                     │
│   End Turn                          │
│   ███░░░░░░░░░░░░░ 114 (34%)        │
└─────────────────────────────────────┘
```

---

### Task 9: Integration

**Updates to existing files:**

1. **`lib.rs`**: Register new commands
   ```rust
   .invoke_handler(tauri::generate_handler![
       // ... existing commands ...
       compute_spectator_match,
   ])
   ```

2. **`MainMenu.svelte`**: Add "Watch AI Match" button
   ```svelte
   <button on:click={() => spectatorStore.loadDecksAndBots()}>
     Watch AI Match
   </button>
   ```

3. **`+page.svelte`**: Add spectator phases
   ```svelte
   {#if gameStore.phase === "menu"}
     <MainMenu />
   {:else if spectatorStore.phase !== "setup"}
     <!-- Spectator mode screens -->
   {:else if gameStore.phase === "setup"}
     <SetupScreen />
   ...
   ```

4. **`api/game.ts`**: Add new API functions
   ```typescript
   export async function computeSpectatorMatch(config: SpectatorConfig): Promise<SpectatorMatch> {
     return invoke("compute_spectator_match", { config });
   }
   ```

---

## File Summary

| File | Type | Description |
|------|------|-------------|
| `src-tauri/src/state/spectator.rs` | New | Backend data structures |
| `src-tauri/src/commands/spectator.rs` | New | IPC command handlers |
| `src-tauri/src/state/game_manager.rs` | Modify | Add `compute_spectator_match()` |
| `src-tauri/src/state/mod.rs` | Modify | Export spectator module |
| `src-tauri/src/commands/mod.rs` | Modify | Export spectator commands |
| `src-tauri/src/lib.rs` | Modify | Register new commands |
| `src/lib/api/types.ts` | Modify | Add spectator types |
| `src/lib/api/game.ts` | Modify | Add spectator API functions |
| `src/lib/stores/spectatorState.svelte.ts` | New | Spectator store |
| `src/lib/components/screens/SpectatorSetup.svelte` | New | Setup screen |
| `src/lib/components/screens/ComputingMatch.svelte` | New | Loading screen |
| `src/lib/components/SpectatorControls.svelte` | New | Playback controls |
| `src/lib/components/AiThinkingPanel.svelte` | New | MCTS visualization |
| `src/lib/components/screens/MainMenu.svelte` | Modify | Add spectator button |
| `src/routes/+page.svelte` | Modify | Route spectator phases |

---

## Implementation Order

1. **Backend first** (Tasks 1-2): Data structures and match computation
2. **Frontend types** (Task 3): TypeScript interfaces
3. **Store** (Task 4): Spectator state management
4. **Setup screen** (Task 5): Entry point for spectator mode
5. **Computing screen** (Task 6): Loading state
6. **Controls** (Task 7): Playback functionality
7. **AI panel** (Task 8): Thinking visualization
8. **Integration** (Task 9): Wire everything together

---

## Testing Checklist

- [ ] Random vs Random match computes successfully
- [ ] MCTS vs Greedy match captures thinking data
- [ ] Playback controls work (play, pause, step)
- [ ] Speed control affects playback timing
- [ ] Timeline scrubber jumps to correct action
- [ ] "Watch Live" hides timeline until end
- [ ] Animations play correctly during playback
- [ ] Back button returns to menu
- [ ] Error handling for failed computation

---

## Future Enhancements (Not in scope)

- Async match computation with progress events
- Save spectator match as replay file
- Share match via seed
- Commentary/annotations
