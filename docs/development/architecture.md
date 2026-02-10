# Essence Wars - System Architecture

> High-level design overview for contributors

## Overview

**Essence Wars** is a deterministic, perfect-information strategy card game engine optimized for AI research and training. The project is organized as a Rust monorepo with Python ML bindings.

**Key Characteristics:**
- Perfect information (no hidden cards)
- Deterministic execution (seeded RNG for reproducible games)
- High performance (~33k games/sec with random bots)
- ML-ready with standardized 328-float state tensors and 256 discrete actions

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              USER INTERFACES                                 │
├─────────────────┬─────────────────────┬─────────────────────────────────────┤
│  Tauri Desktop  │    Claude Code      │           Python ML                 │
│  (Svelte 5 UI)  │    (MCP Server)     │         (PyO3 Bindings)             │
└────────┬────────┴──────────┬──────────┴──────────────────┬──────────────────┘
         │                   │                             │
         │ Tauri Commands    │ JSON-RPC                    │ FFI
         │                   │                             │
┌────────▼───────────────────▼─────────────────────────────▼──────────────────┐
│                          CLIENT API LAYER                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  GameClient                                                          │    │
│  │  - Event emission (CreatureSpawned, Damaged, Killed, etc.)          │    │
│  │  - State diffing for efficient UI updates                           │    │
│  │  - Action history for replay/undo                                   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
┌─────────────────────────────────────▼───────────────────────────────────────┐
│                           CORE GAME ENGINE                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  GameEngine (Facade)                                                 │    │
│  │  - apply_action(): Main entry point for all game mutations          │    │
│  │  - get_legal_actions(): Generate valid moves                        │    │
│  │  - check_victory(): Determine game end                              │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                     │                                        │
│  ┌──────────────┐  ┌────────────────▼───────────┐  ┌────────────────────┐   │
│  │  GameState   │  │      Effect Queue          │  │   Combat System    │   │
│  │  - Players   │  │  - Non-recursive processing│  │  - Damage calc     │   │
│  │  - Board     │  │  - Trigger evaluation      │  │  - Keyword effects │   │
│  │  - Resources │  │  - Deterministic ordering  │  │  - Death handling  │   │
│  └──────────────┘  └────────────────────────────┘  └────────────────────┘   │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
┌─────────────────────────────────────▼───────────────────────────────────────┐
│                          FOUNDATION LAYER                                    │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────────────────┐ │
│  │   Types    │  │  Keywords  │  │   Cards    │  │        Config          │ │
│  │ CardId     │  │ Rush/Guard │  │ CardDatabase│  │ Board size, limits    │ │
│  │ PlayerId   │  │ Lethal     │  │ Commanders │  │ Starting resources    │ │
│  │ Slot       │  │ Lifesteal  │  │ YAML defs  │  │ Turn limits           │ │
│  └────────────┘  └────────────┘  └────────────┘  └────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
┌─────────────────────────────────────▼───────────────────────────────────────┐
│                              DATA LAYER                                      │
│  ┌────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐   │
│  │  data/cards/*.yaml │  │  data/decks/*.toml  │  │  data/weights/*.json│   │
│  │  Card definitions  │  │  Deck compositions  │  │  Bot heuristics     │   │
│  └────────────────────┘  └─────────────────────┘  └─────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Monorepo Structure

```
essence-wars/
├── crates/
│   ├── cardgame/              # Core engine + AI bots (main crate)
│   ├── essence-wars-mcp/      # MCP server for Claude Code
│   └── essence-wars-ui/       # Tauri 2 desktop app
├── python/
│   └── essence_wars/          # Python bindings + ML agents
├── data/
│   ├── cards/                 # Card definitions (YAML)
│   ├── commanders/            # Commander cards (YAML)
│   ├── decks/                 # Deck definitions (TOML)
│   ├── weights/               # Tuned bot weights (JSON)
│   └── ratings/               # ELO tracking
├── docs/                      # Design docs and guides
└── experiments/               # ML training results
```

---

## Component Details

### 1. Core Game Engine (`crates/cardgame/`)

The heart of the system. Provides game logic, state management, and AI bots.

#### Module Structure

| Module | Purpose |
|--------|---------|
| `core/types.rs` | Primitive types: `CardId`, `PlayerId`, `Slot`, `CreatureInstanceId` |
| `core/state.rs` | `GameState`, `PlayerState`, `Creature`, `Support` |
| `core/actions.rs` | Action enum: PlayCard, Attack, UseAbility, EndTurn |
| `core/legal.rs` | Legal move generation and validation |
| `core/engine/` | Action execution, effect processing, victory checking |
| `core/combat/` | Combat resolution, keyword interactions |
| `bots/` | AI implementations: Random, Greedy, MCTS, AlphaBeta |
| `tensor.rs` | ML tensor encoding (328 floats) |
| `client_api/` | High-level interface with event emission |

#### Key Design: Effect Queue

The engine uses a **queue-based effect system** instead of recursion:

```
Action Executed → Immediate Effects → Check Triggers → Triggered Effects → ...
                                           ↑                    │
                                           └────────────────────┘
                                              (repeat until queue empty)
```

Benefits:
- No stack overflow risk
- Deterministic effect ordering
- Easy to debug and test

#### Bot System

```rust
pub trait Bot: Send {
    fn select_action(&mut self,
        state_tensor: &[f32; 328],
        legal_mask: &[f32; 256],
        legal_actions: &[Action]
    ) -> Action;
}
```

| Bot | Algorithm | Speed | Strength |
|-----|-----------|-------|----------|
| RandomBot | Uniform random | 33k games/sec | Baseline |
| GreedyBot | Heuristic (28 weights) | 4.3k games/sec | ~35% vs MCTS |
| MctsBot | UCB1 tree search | 22ms/move | ~60% vs MCTS |
| AlphaBetaBot | Minimax with pruning | ~8s/game @ depth 6 | ~60% |

---

### 2. MCP Server (`crates/essence-wars-mcp/`)

Enables Claude Code to play and analyze games via the Model Context Protocol.

```
Claude Code ←──JSON-RPC (stdio)──→ MCP Server ←──HTTP POST──→ Tauri UI (optional)
```

#### MCP Tools

| Tool | Purpose |
|------|---------|
| `start_game` | Begin new game vs AI |
| `show_state` | ASCII board rendering |
| `legal_actions` | List available moves |
| `play_action` | Execute move by index |
| `ai_hint` | Get AI analysis with ranked moves |
| `explain_*` | Rules, keywords, card info |
| `sync_ui_state` | Push state to Tauri display |

---

### 3. Desktop UI (`crates/essence-wars-ui/`)

Cross-platform desktop application built with Tauri 2 and Svelte 5.

```
┌─────────────────────────────────────┐
│  Svelte 5 Frontend                  │
│  - Runes ($state, $derived, $effect)│
│  - Tailwind CSS v4                  │
│  - 97 components                    │
└────────────────┬────────────────────┘
                 │ Tauri Commands
┌────────────────▼────────────────────┐
│  Rust Backend                       │
│  - GameManager (multi-session)      │
│  - ReplayManager                    │
│  - HTTP sync server                 │
└────────────────┬────────────────────┘
                 │
┌────────────────▼────────────────────┐
│  cardgame crate                     │
└─────────────────────────────────────┘
```

#### Key Features
- Play against AI opponents
- AI vs AI spectator mode with analysis dashboard
- Replay save/load
- Deck builder
- MCP game state visualization

---

### 4. Python Bindings (`python/essence_wars/`)

PyO3-based bindings for ML training and research.

```python
from essence_wars import PyGame, PyParallelGames

# Single game
game = PyGame(deck1="architect_fortify", deck2="broodmother_pack")
game.reset(seed=42)
obs = game.observe()      # numpy (328,)
mask = game.action_mask() # numpy (256,)
reward, done = game.step(action)

# Vectorized for batch training
games = PyParallelGames(num_envs=64)
obs_batch = games.observe_batch()  # (64, 328)
```

#### ML Infrastructure
- **PPO** - Proximal Policy Optimization
- **AlphaZero** - Self-play with MCTS
- **Behavioral Cloning** - Distillation from MCTS
- **Card2Vec** - Card embedding training
- **Gymnasium/PettingZoo** integration

---

## Data Flow

### Turn Resolution

```
1. get_legal_actions()
   └─ Enumerate valid PlayCard, Attack, UseAbility, EndTurn

2. select_action() [Bot or Human]
   └─ Via tensor input or direct UI selection

3. apply_action()
   ├─ Execute action (modify GameState)
   ├─ Process effect queue
   │   ├─ Resolve immediate effects
   │   ├─ Evaluate triggers
   │   └─ Add triggered effects (loop until empty)
   ├─ Check victory conditions
   └─ Emit GameEvents for UI

4. End turn
   ├─ Remove temporary effects
   ├─ Restore resources
   └─ Switch active player
```

### ML Tensor Format

**State tensor (328 floats):**

| Section | Indices | Content |
|---------|---------|---------|
| Global | 0-5 | Turn, active player, game state |
| Player 1 | 6-80 | Life, essence, AP, deck/hand/board |
| Player 2 | 81-155 | Same as Player 1 |
| Cards | 156-325 | Normalized card IDs |
| Commanders | 326-327 | Commander IDs |

**Action space (256 discrete):**

| Range | Action Type |
|-------|-------------|
| 0-99 | PlayCard (10 hand × 10 board) |
| 100-149 | Attack (5 creatures × 10 targets) |
| 150-249 | UseAbility (5 × variants × targets) |
| 255 | EndTurn |

---

## Key Design Decisions

### 1. Determinism
- Seeded LCG for shuffling
- No floating-point non-determinism
- Enables reproducible training and debugging

### 2. Performance via Stack Allocation
- `ArrayVec` for creatures, supports, hands, decks
- Fast cloning (~245ns) essential for MCTS
- Trade-off: Fixed maximum sizes

### 3. Trait-Based Abstraction
- `Bot` trait decouples algorithms from engine
- `GameEnvironment` trait for RL generality
- Easy to add new bots or evaluation methods

### 4. Embedded Data for Deployment
- Python wheels include compiled card/deck data
- No filesystem access required in cloud

### 5. Test Organization
- Unit tests in `tests/unit/` (not inline `#[cfg(test)]`)
- Integration tests at `tests/` root
- Shared utilities in `tests/common/`

---

## CLI Tools

| Binary | Purpose |
|--------|---------|
| `arena` | Run bot matches with statistics |
| `validate` | Quick balance check (~1 sec) |
| `benchmark` | Thorough analysis (AlphaBeta/MCTS) |
| `swiss` | Tournament mode with ELO |
| `tune` | CMA-ES weight optimization |
| `replay` | Game replay and debugging |
| `diagnose` | P1/P2 asymmetry analysis |
| `generate_dataset` | Export ML training data |

---

## Development Quick Reference

```bash
# Build everything
cargo build --release

# Run tests
cargo nextest run --status-level=fail  # Rust
uv run pytest python/tests             # Python

# Lint
cargo lint                             # Clippy
uv run mypy python/essence_wars        # Python types
uv run ruff check python/essence_wars  # Python lint

# Run binaries
cargo run --release --bin arena -- --help
cargo run --release --bin validate --
```

---

## Cross-Component Communication

```
┌─────────────┐
│  Tauri UI   │◄───────HTTP POST /sync_state────────┐
└──────┬──────┘                                      │
       │                                             │
       │ Tauri Commands                              │
       ▼                                             │
┌──────────────┐       ┌──────────────┐       ┌─────┴──────┐
│  GameManager │       │   PyGame     │       │ MCP Server │
└──────┬───────┘       └──────┬───────┘       └──────┬─────┘
       │                      │                      │
       │                      │ FFI                  │ Direct calls
       │                      │                      │
       └──────────────────────┼──────────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │    GameEngine     │
                    │  (cardgame crate) │
                    └───────────────────┘
```

All interfaces ultimately use the same `GameEngine` from `cardgame`, ensuring consistent game behavior across CLI, UI, MCP, and Python.

---

## Further Reading

- `docs/essence-wars-design.md` - Complete game rules and card catalog
- `docs/ml-infrastructure/` - ML training pipeline details
- `crates/*/CLAUDE.md` - AI-focused context for each component
- `python/README.md` - Python ML infrastructure guide
