# JRPG Integration Architecture

> **Status**: Phase 2 Planning Document  
> **Target Implementation**: Phase 4 (Prototype), Phase 5A/5B (Full Product)  
> **Last Updated**: 2026-01-16

## Executive Summary

This document defines the architectural boundaries and integration strategy for transforming Essence Wars from a pure ML research engine into a playable JRPG with story campaigns, while preserving the engine's modularity for continued research and web deployment.

**Key Principle**: The card game engine remains a pure, standalone library. All game clients (3D JRPG, web app, headless trainer) depend on it via clean API boundaries.

---

## 1. Architecture Overview

### 1.1 Module Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│                     Game Clients Layer                       │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   3D JRPG    │  │  Web Client  │  │   Headless   │     │
│  │ (Bevy Game)  │  │ (WASM/UI)    │  │   Trainer    │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                  │                  │              │
└─────────┼──────────────────┼──────────────────┼─────────────┘
          │                  │                  │
          └──────────────────┼──────────────────┘
                             │
┌────────────────────────────┴─────────────────────────────────┐
│                   Game Engine API Layer                       │
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Game Client │  │  Analytics   │  │  Replay &    │      │
│  │     API      │  │     API      │  │  Logging     │      │
│  │ (Phase 5A)   │  │ (Glassbox)   │  │  (Phase 5A)  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└──────────────────────────────────────────────────────────────┘
                             │
┌────────────────────────────┴─────────────────────────────────┐
│                   Core Engine (Current)                       │
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  GameEngine  │  │  GameState   │  │    Action    │      │
│  │    (Exec)    │  │  (326 floats)│  │    Space     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │     Bot      │  │  CardDB &    │  │    Arena     │      │
│  │    Trait     │  │   Effects    │  │   (Match)    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└──────────────────────────────────────────────────────────────┘
```

---

## 2. Core Engine (Current State)

### 2.1 Public API Surface

**Already Excellent for Integration:**

```rust
// src/lib.rs - Clean module exports
pub mod core;      // Game logic
pub mod tensor;    // ML interface
pub mod bots;      // AI agents
pub mod arena;     // Match execution
pub mod decks;     // Deck definitions
```

**Key Traits for Clients:**

```rust
// GameEnvironment - Generic AI interface
trait GameEnvironment {
    type State;
    type Action;
    
    fn get_state(&self) -> &Self::State;
    fn get_legal_actions(&self) -> Vec<Self::Action>;
    fn apply_action(&mut self, action: Self::Action) -> Result<(), String>;
    fn is_terminal(&self) -> bool;
    fn clone_for_search(&self) -> Self;  // MCTS tree search
}

// Bot - AI agent interface
trait Bot: Send {
    fn select_action(
        &mut self,
        state_tensor: &[f32; 326],
        legal_mask: &[f32; 256],
        legal_actions: &[Action]
    ) -> Action;
    
    fn select_action_with_engine(&mut self, engine: &GameEngine) -> Action;
}
```

**State Representation:**

```rust
// GameState - Complete game state (cloneable for search)
pub struct GameState {
    pub players: [PlayerState; 2],
    pub active_player: PlayerId,
    pub current_turn: u16,
    pub phase: GamePhase,
    pub result: Option<GameResult>,
    pub rng_state: u64,  // Deterministic
}

// Action - 256-index fixed action space
pub enum Action {
    PlayCard { hand_index: u8, slot: Slot },
    Attack { attacker: Slot, defender: Slot },
    UseAbility { slot: Slot, ability_index: u8, target: Target },
    EndTurn,
}
```

### 2.2 What Makes This Architecture Perfect for JRPG

1. **Deterministic**: Same seed → same game outcome (perfect for replays, debugging)
2. **Cloneable**: `GameEngine::fork()` enables tree search without affecting main game
3. **State-Action Separation**: Pure state + pure actions = easy serialization
4. **No Hidden Dependencies**: Engine only needs `&CardDatabase` reference
5. **Headless Native**: No graphics coupling, can run anywhere

---

## 3. New API Layer (Phase 3-4)

### 3.1 Game Client API

**Purpose**: High-level interface for interactive play (human vs AI, AI vs AI)

**Location**: `src/client_api/mod.rs` (new module)

**Core Types:**

```rust
/// High-level game controller for client applications
pub struct GameClient {
    engine: GameEngine<'static>,
    card_db: &'static CardDatabase,
    event_log: Vec<GameEvent>,
}

/// Events emitted by the game (for UI reactivity)
pub enum GameEvent {
    GameStarted { seed: u64 },
    TurnStarted { player: PlayerId, turn: u16 },
    ActionTaken { player: PlayerId, action: Action },
    CreatureSpawned { player: PlayerId, slot: Slot, card_id: CardId },
    CreatureDied { player: PlayerId, slot: Slot },
    CombatResolved { attacker: Slot, defender: Slot, result: CombatResult },
    EffectTriggered { source: EffectSource, effect_type: String },
    GameEnded { result: GameResult },
}

impl GameClient {
    pub fn new(card_db: &'static CardDatabase) -> Self;
    
    /// Start a new game with decks
    pub fn start_game(&mut self, deck1: Vec<CardId>, deck2: Vec<CardId>, seed: u64);
    
    /// Execute action and return resulting events
    pub fn apply_action(&mut self, action: Action) -> Result<Vec<GameEvent>, String>;
    
    /// Get current game state for rendering
    pub fn get_state(&self) -> &GameState;
    
    /// Get legal actions for current player
    pub fn get_legal_actions(&self) -> Vec<Action>;
    
    /// Get AI's suggested action (for hint system)
    pub fn get_ai_hint(&mut self, bot: &mut dyn Bot) -> Action;
    
    /// Subscribe to events (for reactive UI)
    pub fn drain_events(&mut self) -> Vec<GameEvent>;
}
```

**Usage Example (Web Client):**

```rust
let mut client = GameClient::new(&CARD_DB);
client.start_game(player_deck, ai_deck, 42);

loop {
    // Get legal actions for UI
    let legal_actions = client.get_legal_actions();
    
    // Wait for player input
    let action = ui.wait_for_player_action(&legal_actions);
    
    // Apply action and get events
    match client.apply_action(action) {
        Ok(events) => {
            for event in events {
                ui.animate_event(&event);  // Play animations
            }
        }
        Err(e) => ui.show_error(e),
    }
    
    if client.get_state().result.is_some() {
        break;
    }
}
```

---

### 3.2 Bot Introspection System (Glassbox Mode)

**Purpose**: Expose AI decision-making internals for visualization

**Location**: `src/bots/introspection.rs` (implemented)

**Core Types:**

```rust
use cardgame::bots::{MctsNodeStats, MctsTreeSnapshot, BotDecision, IntrospectionConfig};

/// Per-action statistics from MCTS tree search
pub struct MctsNodeStats {
    pub action: Action,
    pub visits: u32,
    pub win_rate: f32,
}

/// Complete MCTS state at decision time
pub struct MctsTreeSnapshot {
    pub total_simulations: u32,
    pub children: Vec<MctsNodeStats>,
    pub selected_action: Action,
    pub root_value: f32,
}

/// Complete decision record from any bot
pub struct BotDecision {
    pub mcts_snapshot: Option<MctsTreeSnapshot>,
    // Future: policy network outputs, value estimates, etc.
}

/// Configuration for what introspection data to collect
pub struct IntrospectionConfig {
    pub capture_mcts_tree: bool,
    pub max_children_to_report: usize,
    // Future: capture_policy_output, capture_value_estimate
}
```

**Integration with MctsBot:**

The `MctsBot` automatically captures tree statistics when introspection is enabled:

```rust
impl MctsBot {
    /// Enable introspection with configuration
    pub fn with_introspection(mut self, config: IntrospectionConfig) -> Self;

    /// Get the last decision's introspection data
    pub fn last_decision(&self) -> Option<&BotDecision>;
}
```

**Bevy Client Integration (GameBridge):**

The 3D JRPG client accesses MCTS introspection via the `GameBridge` resource:

```rust
// In crates/essence-wars-3d/src/game/bridge.rs

#[derive(Resource)]
pub struct GameBridge {
    // ... game state fields ...
    last_decision: Option<BotDecision>,
}

impl GameBridge {
    /// Get MCTS tree snapshot from the last AI decision
    pub fn get_mcts_snapshot(&self) -> Option<&MctsTreeSnapshot> {
        self.last_decision.as_ref()
            .and_then(|d| d.mcts_snapshot.as_ref())
    }
}
```

**Glassbox UI Flow:**

```rust
// In Bevy system for AI turn visualization
fn visualize_ai_decision(
    bridge: Res<GameBridge>,
    mut ui_state: ResMut<GlassboxUI>,
) {
    if let Some(snapshot) = bridge.get_mcts_snapshot() {
        // Display total simulations run
        ui_state.show_simulation_count(snapshot.total_simulations);

        // Show root value estimate
        ui_state.show_value_bar(snapshot.root_value);

        // Render action distribution chart
        for node in &snapshot.children {
            ui_state.add_action_bar(
                &node.action,
                node.visits,
                node.win_rate,
            );
        }

        // Highlight selected action
        ui_state.highlight_selected(&snapshot.selected_action);
    }
}
```

**Data Available for Visualization:**

| Field | Type | Description |
|-------|------|-------------|
| `total_simulations` | u32 | Number of MCTS rollouts performed |
| `root_value` | f32 | Estimated win probability at root (-1.0 to 1.0) |
| `children[].action` | Action | The action this node represents |
| `children[].visits` | u32 | How many times this action was explored |
| `children[].win_rate` | f32 | Win rate from simulations through this action |
| `selected_action` | Action | The action the bot ultimately chose |

---

### 3.3 Replay System

**Purpose**: Save and replay games with full state history

**Current Foundation**: Already exists in `arena::ActionLogger`

**Enhancements Needed:**

```rust
// src/replay/mod.rs (new module)

/// Serializable replay format
#[derive(Serialize, Deserialize)]
pub struct GameReplay {
    pub version: String,  // Engine version
    pub seed: u64,
    pub deck1: Vec<CardId>,
    pub deck2: Vec<CardId>,
    pub actions: Vec<Action>,
    pub analytics: Option<AnalyticsData>,  // For Glassbox mode
}

impl GameReplay {
    /// Save to disk
    pub fn save(&self, path: &Path) -> io::Result<()>;
    
    /// Load from disk
    pub fn load(path: &Path) -> io::Result<Self>;
    
    /// Replay game step-by-step
    pub fn replay(&self, card_db: &CardDatabase) -> ReplayIterator;
}

/// Iterator for stepping through replay
pub struct ReplayIterator<'a> {
    engine: GameEngine<'a>,
    actions: &'a [Action],
    current_index: usize,
}

impl<'a> ReplayIterator<'a> {
    pub fn step(&mut self) -> Option<ReplayStep> {
        // Execute next action, return state + events
    }
    
    pub fn seek(&mut self, turn: u16) {
        // Fast-forward to specific turn
    }
}
```

---

## 4. Glassbox AI Visualization

### 4.1 Overview

Glassbox mode provides real-time AI decision transparency in the Bevy 3D client. When enabled, players can observe the AI's thought process as it plays, including MCTS tree exploration, action probabilities, and position evaluation. This feature serves both as an educational tool for players learning the game and as a debugging aid during development.

### 4.2 UI Components

**MCTS Panel (Right Sidebar)**

The MCTS panel displays the top 10 actions by visit count from the most recent AI decision:

- **Progress Bars**: Visual representation of visit count relative to total simulations
- **Win Rate Display**: Percentage showing expected win probability for each action
- **Action Labels**: Human-readable description of each action (e.g., "Play Brass Sentinel to slot 2")
- **Selection Highlight**: The action the AI ultimately chose is highlighted with a distinct border

**Action Probabilities (Bottom-Left)**

Shows the top 5 actions ranked by win rate with color-coded bars:

| Win Rate | Color |
|----------|-------|
| > 60%    | Green |
| 40-60%   | Yellow |
| < 40%    | Red |

This panel helps players understand which moves the AI considers strongest regardless of exploration frequency.

**Value Gauge (Top-Left)**

A vertical gauge displaying the AI's position evaluation:

- **Range**: -1.0 (P2 winning) to +1.0 (P1 winning)
- **Center**: 0.0 represents an even position
- **Color Gradient**: Red (P2 advantage) through Yellow (even) to Green (P1 advantage)
- **Label**: Shows numeric value (e.g., "+0.35")

### 4.3 Data Flow

```
MctsBot → BotDecision (with MctsTreeSnapshot) → GameBridge.last_decision → Glassbox panels query and render
```

**Step-by-Step:**

1. `MctsBot::select_action_with_engine()` performs tree search
2. After search completes, MCTS statistics are captured into `MctsTreeSnapshot`
3. The snapshot is wrapped in `BotDecision` and stored in the bot
4. `GameBridge` copies the decision when processing the AI turn
5. Glassbox UI systems query `GameBridge::get_mcts_snapshot()` each frame
6. Panels render the data using Bevy UI components

### 4.4 Key Types

From `cardgame::bots::introspection`:

```rust
/// Statistics for a single action node in the MCTS tree
pub struct MctsNodeStats {
    pub action: Action,      // The action this node represents
    pub visits: u32,         // Number of times this action was explored
    pub win_rate: f32,       // Win rate from simulations (0.0 to 1.0)
}

/// Complete snapshot of MCTS tree state at decision time
pub struct MctsTreeSnapshot {
    pub total_simulations: u32,      // Total rollouts performed
    pub children: Vec<MctsNodeStats>, // Stats for each explored action
    pub selected_action: Action,      // The action chosen by the bot
    pub root_value: f32,              // Position evaluation (-1.0 to 1.0)
}

/// Decision record from any bot (extensible for future bot types)
pub struct BotDecision {
    pub mcts_snapshot: Option<MctsTreeSnapshot>,
    // Future: policy network outputs, value estimates, etc.
}
```

### 4.5 Toggle Control

**Keyboard Shortcut**: Press **'G'** key to show/hide all Glassbox panels

The toggle affects all three panels simultaneously. When hidden, no performance overhead is incurred from rendering the visualization UI.

**Implementation:**

```rust
fn toggle_glassbox(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut glassbox_state: ResMut<GlassboxState>,
) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        glassbox_state.visible = !glassbox_state.visible;
    }
}
```

---

## 5. 3D JRPG Integration (Phase 5B)

### 5.1 Bevy Project Structure

The 3D client is implemented as a separate crate in the workspace:

```
crates/essence-wars-3d/
├── Cargo.toml             # Depends on: essence-wars (workspace), bevy
└── src/
    ├── main.rs            # App entry point, plugin registration
    │
    ├── game/              # Core game integration
    │   ├── mod.rs         # Game module exports
    │   ├── bridge.rs      # GameBridge resource wrapping GameClient
    │   └── state.rs       # AppState enum (Menu, Playing, GameOver)
    │
    ├── rendering/         # 3D visualization
    │   ├── mod.rs         # Rendering module exports
    │   ├── board.rs       # 3D board with creature slots
    │   ├── creatures.rs   # Creature3D component and spawning
    │   ├── camera.rs      # Camera controller
    │   └── lighting.rs    # Scene lighting setup
    │
    ├── ui/                # User interface
    │   ├── mod.rs         # UI module exports
    │   ├── hud.rs         # Game HUD (life, essence, turn)
    │   ├── hand.rs        # Player hand display
    │   └── menu.rs        # Main menu and game over screens
    │
    └── glassbox/          # AI visualization (Glassbox Mode)
        ├── mod.rs         # Glassbox state and plugin
        ├── mcts_panel.rs  # MCTS tree visualization
        ├── action_probs.rs # Action probability bars
        └── value_gauge.rs # Value estimate gauge
```

**Note**: The `overworld/` and `story/` modules for 3D exploration and campaign systems are planned for Phase 5B but not yet implemented.

### 5.2 Battle System Integration

**Key Challenge**: Bridge stateless card engine with stateful Bevy ECS

**Solution**: Bevy resources hold `GameClient`, ECS systems react to events

```rust
// battle/bridge.rs

/// Bevy resource wrapping the game client
#[derive(Resource)]
pub struct BattleSession {
    client: GameClient,
    pending_events: VecDeque<GameEvent>,
    animation_queue: VecDeque<Animation>,
}

/// Bevy state for battle flow
#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum BattleState {
    #[default]
    Loading,
    PlayerTurn,
    WaitingForAI,
    AnimatingAction,
    BattleOver,
}

/// System: Start a new battle
fn start_battle(
    mut commands: Commands,
    mut next_state: ResMut<NextState<BattleState>>,
    story_state: Res<StoryProgress>,
) {
    let deck1 = story_state.get_player_deck();
    let deck2 = story_state.get_enemy_deck();
    
    let mut client = GameClient::new(&CARD_DB);
    client.start_game(deck1, deck2, story_state.battle_seed());
    
    commands.insert_resource(BattleSession {
        client,
        pending_events: VecDeque::new(),
        animation_queue: VecDeque::new(),
    });
    
    next_state.set(BattleState::PlayerTurn);
}

/// System: Process player input
fn handle_player_action(
    mut battle: ResMut<BattleSession>,
    mut next_state: ResMut<NextState<BattleState>>,
    input: Res<BattleInput>,
) {
    if let Some(action) = input.selected_action {
        let events = battle.client.apply_action(action).unwrap();
        battle.pending_events.extend(events);
        next_state.set(BattleState::AnimatingAction);
    }
}

/// System: Execute AI turn
fn ai_turn(
    mut battle: ResMut<BattleSession>,
    mut next_state: ResMut<NextState<BattleState>>,
    mut ai_bot: ResMut<OpponentBot>,
) {
    let action = battle.client.get_ai_action(&mut ai_bot.bot);
    let events = battle.client.apply_action(action).unwrap();
    
    battle.pending_events.extend(events);
    next_state.set(BattleState::AnimatingAction);
}

/// System: Convert events to animations
fn process_events(
    mut battle: ResMut<BattleSession>,
    mut next_state: ResMut<NextState<BattleState>>,
) {
    while let Some(event) = battle.pending_events.pop_front() {
        match event {
            GameEvent::CreatureSpawned { slot, card_id, .. } => {
                battle.animation_queue.push_back(
                    Animation::SpawnCreature { slot, card_id, duration: 0.5 }
                );
            }
            GameEvent::CombatResolved { attacker, defender, result } => {
                battle.animation_queue.push_back(
                    Animation::AttackSequence { attacker, defender, result }
                );
            }
            GameEvent::GameEnded { result } => {
                next_state.set(BattleState::BattleOver);
                return;
            }
            _ => {}
        }
    }
    
    if battle.animation_queue.is_empty() {
        // Determine next state based on current player
        if battle.client.get_state().active_player == PlayerId::PLAYER_ONE {
            next_state.set(BattleState::PlayerTurn);
        } else {
            next_state.set(BattleState::WaitingForAI);
        }
    }
}
```

### 5.3 Story-Battle Integration (Planned for Phase 5B)

> **Status**: This section describes planned functionality for Phase 5B.
> **Current Implementation**: The `essence-wars-3d` client focuses on the battle system and Glassbox AI visualization. Story mode, overworld exploration, dialogue system, and quest progression features are planned for Phase 5B.

**Data Flow:**

```
Story Event → Triggers Battle → Configure Decks → Battle Plays → Update Story State
```

**Planned Implementation:**

```rust
// story/campaign.rs

/// Persistent story progress
#[derive(Resource, Serialize, Deserialize)]
pub struct StoryProgress {
    pub faction: Faction,
    pub chapter: u8,
    pub unlocked_cards: HashSet<CardId>,
    pub completed_battles: HashSet<BattleId>,
    pub player_deck: Vec<CardId>,
}

impl StoryProgress {
    /// Check if player can access a battle
    pub fn can_start_battle(&self, battle_id: BattleId) -> bool;
    
    /// Get enemy deck for a story battle
    pub fn get_enemy_deck(&self, battle_id: BattleId) -> Vec<CardId>;
    
    /// Unlock cards after battle victory
    pub fn unlock_cards_from_battle(&mut self, battle_id: BattleId);
}

// overworld/encounters.rs

/// Trigger system for story battles
fn check_battle_trigger(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    encounter_query: Query<(&Transform, &BattleEncounter)>,
    mut story: ResMut<StoryProgress>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let player_pos = player_query.single().translation;
    
    for (encounter_pos, encounter) in encounter_query.iter() {
        let distance = player_pos.distance(encounter_pos.translation);
        
        if distance < 2.0 && story.can_start_battle(encounter.battle_id) {
            // Transition to battle
            commands.insert_resource(PendingBattle {
                battle_id: encounter.battle_id,
                seed: rand::random(),
            });
            next_state.set(GameState::Battle);
            return;
        }
    }
}
```

---

## 6. Web Client (Phase 5A)

### 6.1 Technology Options

> **Current Situation**: The `cardgame` crate has optional WASM support via `wasm-bindgen`, and Bevy can compile to WASM natively using WebGL/WebGPU. No final decision has been made on the web approach.

**Option A: Bevy WASM (Recommended)**

Compile the `essence-wars-3d` client directly to WASM using Bevy's native WASM support.

| Pros | Cons |
|------|------|
| Single codebase for desktop and web | Larger bundle size (~5-10MB) |
| Full 3D rendering in browser via WebGL/WebGPU | Slower compile times |
| Glassbox AI visualization works identically | Requires WebGPU for best performance |
| Bevy UI and game logic reused completely | Mobile browser support varies |
| Rust type safety throughout | |

**Option B: Lightweight TypeScript UI**

Use the `cardgame` crate's existing `wasm-bindgen` bindings with a custom TypeScript/React frontend.

| Pros | Cons |
|------|------|
| Smaller bundle size (~1-2MB) | Must reimplement entire UI from scratch |
| Simpler deployment (static files) | Duplicate UI logic between clients |
| Faster iteration on web-specific features | WASM FFI overhead for each call |
| Familiar web development stack | No 3D visualization (2D only) |
| Better mobile browser compatibility | Glassbox Mode needs reimplementation |

**Recommended Path: Option A**

Since the Bevy 3D client (`essence-wars-3d`) already exists and Bevy has mature WASM support, Option A is the likely path forward. This approach:

1. Eliminates duplicate UI development effort
2. Ensures feature parity between desktop and web
3. Allows Glassbox AI visualization to work identically in browser
4. Leverages Bevy's growing WebGPU support for modern browsers

Option B remains viable for scenarios requiring minimal bundle size or maximum mobile compatibility, but would require significant UI reimplementation work.

### 6.2 WASM Bridge (Option B)

```rust
// src/wasm/mod.rs (new module)

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmGameClient {
    client: GameClient,
}

#[wasm_bindgen]
impl WasmGameClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Initialize card database (embedded in WASM)
        Self {
            client: GameClient::new(&EMBEDDED_CARD_DB),
        }
    }
    
    #[wasm_bindgen]
    pub fn start_game(&mut self, deck1_json: &str, deck2_json: &str, seed: u64) -> Result<(), JsValue> {
        let deck1: Vec<CardId> = serde_json::from_str(deck1_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let deck2: Vec<CardId> = serde_json::from_str(deck2_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        self.client.start_game(deck1, deck2, seed);
        Ok(())
    }
    
    #[wasm_bindgen]
    pub fn get_state_json(&self) -> String {
        serde_json::to_string(self.client.get_state()).unwrap()
    }
    
    #[wasm_bindgen]
    pub fn apply_action(&mut self, action_json: &str) -> Result<String, JsValue> {
        let action: Action = serde_json::from_str(action_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let events = self.client.apply_action(action)
            .map_err(|e| JsValue::from_str(&e))?;
        
        Ok(serde_json::to_string(&events).unwrap())
    }
}
```

**TypeScript Usage:**

```typescript
import init, { WasmGameClient } from './pkg/essence_wars';

await init();
const client = new WasmGameClient();

client.start_game(
    JSON.stringify(playerDeck),
    JSON.stringify(aiDeck),
    Date.now()
);

// Game loop
const state = JSON.parse(client.get_state_json());
const action = await ui.waitForPlayerAction(state);
const events = JSON.parse(client.apply_action(JSON.stringify(action)));

for (const event of events) {
    await ui.animateEvent(event);
}
```

---

## 7. Data Flow & Serialization

### 7.1 Core Data Formats

**GameState → JSON** (Already implemented in `serde`)
```json
{
  "players": [
    {
      "life": 20,
      "current_essence": 5,
      "creatures": [
        {"card_id": 42, "slot": 0, "power": 3, "toughness": 4}
      ]
    }
  ],
  "active_player": 0,
  "current_turn": 5
}
```

**Action → JSON**
```json
{
  "PlayCard": { "hand_index": 2, "slot": 1 }
}
```

**Replay → MessagePack** (Binary, smaller than JSON)
```rust
// More efficient for large replays
use rmp_serde::{Serializer, Deserializer};

let bytes = rmp_serde::to_vec(&replay)?;
std::fs::write("replay.msgpack", bytes)?;
```

---

## 8. Implementation Roadmap

### Phase 1-2: Core Engine ✅ Complete
- [x] Game engine with 14 keywords
- [x] Bot system (Random, Greedy, MCTS)
- [x] Arena CLI and tuning pipeline

### Phase 3: Client API ✅ Complete
- [x] GameClient wrapper
- [x] Bot introspection system (MctsTreeSnapshot, BotDecision)
- [x] Event system foundation

### Phase 4: Bevy 3D Client ✅ Complete (January 2026)
- [x] Cargo workspace migration
- [x] 3D board and creature rendering
- [x] egui UI panels (HUD, hand, menus)
- [x] Glassbox AI visualization (MCTS panel, action probs, value gauge)

### Phase 5A: Card Expansion 🔄 In Progress
- [ ] Expand to 300 cards
- [ ] Balance validation

### Phase 5B: Story Mode 📋 Planned
- [ ] Overworld exploration
- [ ] Dialogue system
- [ ] Quest progression
- [ ] Story-battle integration

### Phase 6: Network & Distribution 📋 Planned
- [ ] Multiplayer support
- [ ] WASM/web deployment
- [ ] Python bindings for RL

---

## 9. Testing Strategy

### 9.1 Client API Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_game_client_lifecycle() {
        let mut client = GameClient::new(&test_card_db());
        client.start_game(test_deck(), test_deck(), 42);
        
        assert!(!client.is_terminal());
        
        let actions = client.get_legal_actions();
        assert!(!actions.is_empty());
        
        let events = client.apply_action(actions[0]).unwrap();
        assert!(!events.is_empty());
    }
    
    #[test]
    fn test_replay_determinism() {
        let mut client1 = GameClient::new(&test_card_db());
        let mut client2 = GameClient::new(&test_card_db());
        
        client1.start_game(test_deck(), test_deck(), 42);
        client2.start_game(test_deck(), test_deck(), 42);
        
        for _ in 0..100 {
            let actions1 = client1.get_legal_actions();
            let actions2 = client2.get_legal_actions();
            assert_eq!(actions1, actions2);
            
            client1.apply_action(actions1[0]).unwrap();
            client2.apply_action(actions2[0]).unwrap();
            
            assert_eq!(client1.get_state(), client2.get_state());
        }
    }
}
```

### 9.2 Integration Tests

```rust
// tests/bevy_integration_tests.rs
#[test]
fn test_battle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, start_battle);
    
    app.update();
    
    let session = app.world.resource::<BattleSession>();
    assert!(session.client.get_legal_actions().len() > 0);
}
```

---

## 10. Performance Considerations

### 10.1 Clone Overhead

**Current**: `GameEngine::fork()` clones entire state (~2KB)
- Fast for MCTS (1000s of clones per second)
- No issue for JRPG (1 clone per action at most)

### 10.2 WASM Bundle Size

**Estimated Sizes:**
- Core engine: ~500KB (optimized)
- Full bot suite: ~800KB
- Card database: ~100KB (embedded JSON)
- **Total**: ~1.4MB (acceptable for web)

**Optimization:**
```toml
[profile.release]
opt-level = 'z'      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Strip symbols
```

### 10.3 Bevy Performance

**Target**: 60 FPS battle animations
- Card game logic: <1ms per action (headroom: 16ms/frame)
- Bevy overhead: ~2-5ms (state sync, rendering)
- Animation smoothing: Use Bevy's interpolation systems

---

## 11. Open Questions & Future Work

### 11.1 Network Multiplayer (Phase 6?)

**Challenge**: Current engine is single-process only

**Possible Solutions:**
- **Lockstep**: Send actions, both clients simulate (relies on determinism ✓)
- **Server Authority**: Server runs `GameEngine`, sends state updates
- **Rollback Netcode**: Fork engine on prediction mismatch

**Recommendation**: Defer to Phase 6, not essential for JRPG

### 11.2 Mod Support (Phase 6?)

**Current**: Cards are YAML + Rust code (effects hardcoded)

**Future**: Consider scripting layer (Lua/Rhai) for community cards

**Trade-off**: Complexity vs. extensibility

---

## 12. Conclusion

The current Essence Wars architecture is **exceptionally well-suited** for JRPG integration:

✅ **Clean Separation**: Engine has no UI coupling  
✅ **Deterministic**: Perfect for replays and debugging  
✅ **Cloneable**: Enables tree search without side effects  
✅ **Type-Safe**: Rust compiler prevents client errors  
✅ **Headless-Ready**: Can run on servers, WASM, or GPU-less CI  

**Recommended Next Steps:**
1. Implement `GameClient` API (Phase 3) - ~1 week
2. Add analytics hooks to MCTS/Greedy bots (Phase 3) - ~3 days
3. Build Bevy prototype (Phase 4) - ~2 weeks
4. Validate with web WASM build (Phase 4) - ~1 week

The modular approach ensures the card engine remains pristine for ML research while enabling rich game experiences on top. No architectural rewrites needed—only additive layers.

---

**Document Version**: 1.0  
**Author**: AI Planning Agent  
**Review Status**: Draft (Awaiting Human Review)
