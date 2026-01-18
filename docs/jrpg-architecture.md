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

### 3.2 Analytics API (Glassbox Mode)

**Purpose**: Expose AI decision-making internals for visualization

**Location**: `src/analytics/mod.rs` (new module)

**Core Types:**

```rust
/// Analytics collector for AI decision-making
pub struct AnalyticsCollector {
    mcts_snapshots: Vec<MctsSnapshot>,
    policy_outputs: Vec<PolicyOutput>,
    value_estimates: Vec<ValueEstimate>,
}

/// Snapshot of MCTS tree at a decision point
pub struct MctsSnapshot {
    pub turn: u16,
    pub root_visits: u32,
    pub children: Vec<MctsChildInfo>,
}

pub struct MctsChildInfo {
    pub action: Action,
    pub visits: u32,
    pub wins: i32,
    pub ucb1_score: f32,
}

/// Neural network policy output
pub struct PolicyOutput {
    pub turn: u16,
    pub action_probs: Vec<(Action, f32)>,  // Top-N actions with probabilities
}

/// Value estimate for current state
pub struct ValueEstimate {
    pub turn: u16,
    pub player: PlayerId,
    pub value: f32,  // -1.0 to +1.0 (expected outcome)
}

impl AnalyticsCollector {
    /// Attach to a bot to track decisions
    pub fn attach(&mut self, bot: &mut dyn Bot);
    
    /// Export collected data as JSON
    pub fn export_json(&self) -> String;
    
    /// Get visualization data for specific turn
    pub fn get_turn_analytics(&self, turn: u16) -> TurnAnalytics;
}
```

**Integration with Bots:**

```rust
// Modify MctsBot to expose tree state
impl MctsBot {
    /// Get current tree state for visualization
    pub fn get_tree_snapshot(&self) -> MctsSnapshot {
        MctsSnapshot {
            root_visits: self.root.borrow().visits,
            children: self.root.borrow().children.iter()
                .map(|child| {
                    let c = child.borrow();
                    MctsChildInfo {
                        action: c.action.unwrap(),
                        visits: c.visits,
                        wins: c.wins,
                        ucb1_score: c.ucb1(self.root.borrow().visits, self.config.exploration),
                    }
                })
                .collect(),
        }
    }
}
```

**Glassbox UI Flow:**

```rust
let mut client = GameClient::new(&CARD_DB);
let mut analytics = AnalyticsCollector::new();
let mut bot1 = MctsBot::new(...);
let mut bot2 = GreedyBot::new(...);

analytics.attach(&mut bot1);
analytics.attach(&mut bot2);

client.start_game(...);

// Step-by-step execution with analytics capture
while !client.is_terminal() {
    let action = bot1.select_action_with_engine(&client.engine);
    
    // Capture MCTS tree before move
    analytics.record_mcts_snapshot(bot1.get_tree_snapshot());
    
    client.apply_action(action)?;
    
    // UI can now visualize the decision tree
}

// Export for researchers
std::fs::write("replay_analytics.json", analytics.export_json())?;
```

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

## 4. 3D JRPG Integration (Phase 5B)

### 4.1 Bevy Project Structure

```
crates/essence-game-3d/
├── Cargo.toml             # Depends on: essence-wars (workspace), bevy
├── src/
│   ├── main.rs            # Entry point
│   ├── lib.rs             # Re-export core modules
│   │
│   ├── battle/            # Card game integration
│   │   ├── mod.rs
│   │   ├── bridge.rs      # GameClient → Bevy ECS bridge
│   │   ├── ui.rs          # Battle UI (hand, board, stats)
│   │   ├── animations.rs  # Card play, attack, death animations
│   │   └── camera.rs      # Battle camera (fixed 2.5D view)
│   │
│   ├── overworld/         # 3D exploration
│   │   ├── mod.rs
│   │   ├── player.rs      # Character controller
│   │   ├── camera.rs      # Third-person follow camera
│   │   ├── npc.rs         # Dialogue system
│   │   └── encounters.rs  # Trigger battles from 3D world
│   │
│   ├── story/             # Campaign system
│   │   ├── mod.rs
│   │   ├── campaign.rs    # Story progress tracking
│   │   ├── dialogue.rs    # Dialogue trees
│   │   ├── quests.rs      # Quest system
│   │   └── progression.rs # Unlock cards via story
│   │
│   └── shared/            # Cross-cutting concerns
│       ├── state.rs       # Bevy States (Menu, Overworld, Battle)
│       ├── assets.rs      # Asset loading
│       └── ui.rs          # Shared UI components
│
└── assets/
    ├── models/            # GLTF/GLB files
    ├── textures/
    ├── audio/
    └── data/
        ├── campaigns/     # Story definitions (RON/YAML)
        └── scenes/        # 3D scene layouts (Blender exports)
```

### 4.2 Battle System Integration

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

### 4.3 Story-Battle Integration

**Data Flow:**

```
Story Event → Triggers Battle → Configure Decks → Battle Plays → Update Story State
```

**Implementation:**

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

## 5. Web Client (Phase 5A)

### 5.1 Technology Options

**Option A: Bevy WASM** (Unified codebase)
- **Pros**: Same code as 3D game, Bevy UI, Rust type safety
- **Cons**: Large bundle size (~5MB), slower compile times

**Option B: Web-Native (TypeScript + Canvas)** (Separate UI)
- **Pros**: Smaller bundle, faster iteration, familiar web stack
- **Cons**: Duplicate UI logic, WASM FFI overhead

**Key Decision**: **Option B** for Phase 5A (faster iteration on UI/UX)

### 5.2 WASM Bridge

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

## 6. Data Flow & Serialization

### 6.1 Core Data Formats

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

## 7. Implementation Roadmap

### Phase 3: ML Infrastructure
- [ ] Add `Serialize` + `Deserialize` to all public types (already done)
- [ ] Create `client_api` module with `GameClient`
- [ ] Create `analytics` module with `AnalyticsCollector`
- [ ] Add MCTS/Greedy bot introspection methods
- [ ] Write `docs/api-reference.md`

### Phase 4: Research Platform
- [ ] Build minimal Bevy prototype (single battle scene)
- [ ] Test `GameClient` → Bevy ECS bridge
- [ ] Implement basic 3D→Battle transition
- [ ] Validate WASM build with `wasm-pack`
- [ ] Create `replay` module with serialization

### Phase 5A: Web Playable
- [ ] Implement full `GameClient` with event system
- [ ] Build TypeScript web UI (2D card game)
- [ ] Integrate WASM bridge
- [ ] Create Glassbox Mode UI (MCTS tree visualization)
- [ ] Deploy to Huggingface Spaces

### Phase 5B: JRPG Campaign
- [ ] Expand Bevy prototype to full game structure
- [ ] Implement story/campaign system
- [ ] Build 3D overworld + battle integration
- [ ] Create three faction campaigns
- [ ] Polish animations and UI

---

## 8. Testing Strategy

### 8.1 Client API Tests

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

### 8.2 Integration Tests

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

## 9. Performance Considerations

### 9.1 Clone Overhead

**Current**: `GameEngine::fork()` clones entire state (~2KB)
- Fast for MCTS (1000s of clones per second)
- No issue for JRPG (1 clone per action at most)

### 9.2 WASM Bundle Size

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

### 9.3 Bevy Performance

**Target**: 60 FPS battle animations
- Card game logic: <1ms per action (headroom: 16ms/frame)
- Bevy overhead: ~2-5ms (state sync, rendering)
- Animation smoothing: Use Bevy's interpolation systems

---

## 10. Open Questions & Future Work

### 10.1 Network Multiplayer (Phase 6?)

**Challenge**: Current engine is single-process only

**Possible Solutions:**
- **Lockstep**: Send actions, both clients simulate (relies on determinism ✓)
- **Server Authority**: Server runs `GameEngine`, sends state updates
- **Rollback Netcode**: Fork engine on prediction mismatch

**Recommendation**: Defer to Phase 6, not essential for JRPG

### 10.2 Mod Support (Phase 6?)

**Current**: Cards are YAML + Rust code (effects hardcoded)

**Future**: Consider scripting layer (Lua/Rhai) for community cards

**Trade-off**: Complexity vs. extensibility

---

## 11. Conclusion

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
