# Essence Wars - Core Game Engine

High-performance, deterministic card game engine designed for AI/RL research and competitive play.

## Overview

This crate provides:

- **Complete game engine** with state management, turn resolution, and action processing
- **Deterministic execution** via seeded RNG for reproducible gameplay
- **Perfect information** design suitable for adversarial AI training
- **Multiple AI bots** from random baselines to competitive Alpha-Beta search
- **ML-ready interfaces** with 328-float state tensors and 256 discrete actions
- **CLI tools** for analysis, tuning, tournaments, and replay

## Installation

```bash
# Build the crate
cargo build --release -p cardgame

# Run tests
cargo nextest run -p cardgame --status-level=fail

# Run benchmarks
cargo bench -p cardgame
```

## Quick Start

```rust
use cardgame::{Engine, CardDatabase, DeckRegistry, Action};
use cardgame::bots::{create_bot, BotType, MctsConfig};

// Load game data
let db = CardDatabase::load_from_dir("data/cards/core_set")?;
let decks = DeckRegistry::load_from_dir("data/decks")?;

// Create engine with seed for reproducibility
let mut engine = Engine::new_with_decks(
    &db,
    decks.get("iron_wall")?,
    decks.get("swarm_aggro")?,
    12345,  // seed
)?;

// Create an MCTS bot
let mut bot = create_bot(&BotType::Mcts(MctsConfig::default()), None)?;

// Game loop
while !engine.is_game_over() {
    let state = engine.get_state_tensor();
    let mask = engine.get_legal_action_mask();
    let legal = engine.get_legal_actions();

    let action = bot.select_action(&state, &mask, &legal);
    engine.apply_action(action)?;
}

println!("Winner: {:?}", engine.state().result);
```

## Game Concepts

### Core Mechanics

| Concept | Description |
|---------|-------------|
| **Players** | 2 opponents (P1 plays first, P2 draws 2 bonus cards) |
| **Life** | 30 HP per player (0 HP = defeat) |
| **Essence** | Mana pool (starts at 1, +1/turn, max 10) |
| **Action Points** | 3 per turn for plays, attacks, abilities |
| **Board** | 5 creature slots + 2 support slots per player |
| **Hand** | Max 10 cards (excess burned) |
| **Deck** | 30 cards standard (29 + 1 commander) |
| **Turn Limit** | 30 turns (draw if no winner) |

### Card Types

- **Creatures** - Deploy to board, attack/defend, have stats and keywords
- **Spells** - One-time effects (instant resolution)
- **Supports** - Passive auras in support slots
- **Tokens** - Creatures spawned by effects (no card ID)

### Keywords (16)

| Keyword | Effect |
|---------|--------|
| Rush | Can attack turn played |
| Ranged | Takes no counter-damage |
| Piercing | Excess damage hits face |
| Guard | Must be attacked first |
| Lifesteal | Attacker heals for damage dealt |
| Lethal | Any non-zero damage kills |
| Shield | First damage absorbed |
| Quick | Damage dealt before counter-attack |
| Ephemeral | Dies at end of turn |
| Regenerate | Heals 2 HP at turn start |
| Stealth | Untargetable by attacks |
| Charge | +2 attack when attacking |
| Frenzy | +1 attack per attack this turn |
| Volatile | Deal 2 damage to all enemies on death |
| Fortify | Takes 1 less damage (min 1) |
| Ward | First targeting spell/ability has no effect |

### Factions

| Faction | Identity | Card IDs |
|---------|----------|----------|
| **Argentum Combine** | Defense & durability (Guard, Shield, Fortify) | 1000-1074 |
| **Symbiote Circles** | Aggressive tempo (Rush, Lethal, Volatile) | 2000-2074 |
| **Obsidion Syndicate** | Burst damage (Lifesteal, Stealth, Quick) | 3000-3074 |
| **Neutral** | Flexible utility (Ranged, various) | 4000-4074 |
| **Commanders** | Unique passive abilities (one per deck) | 5000-5011 |

## Bot System

### Available Bots

| Bot | Strategy | Speed | Use Case |
|-----|----------|-------|----------|
| `RandomBot` | Uniform random selection | ~33k games/sec | Baseline, stress testing |
| `GreedyBot` | Heuristic evaluation (28 weights) | ~4.3k games/sec | Fast validation, tuning |
| `MctsBot` | UCB1 tree search + rollouts | ~22ms/move | Training data, interactive play |
| `AlphaBetaBot` | Minimax with pruning | Depth 6: ~8s/game | Strong opponent, balance testing |

### Bot Trait

```rust
pub trait Bot: Send {
    fn select_action(
        &mut self,
        state_tensor: &[f32; 328],
        legal_mask: &[f32; 256],
        legal_actions: &[Action],
    ) -> Action;

    fn requires_engine(&self) -> bool { false }
    fn reset(&mut self);
    fn clone_box(&self) -> Box<dyn Bot>;
}
```

### Creating Bots

```rust
use cardgame::bots::{create_bot, BotType, MctsConfig, AlphaBetaConfig};

// Random bot
let random = create_bot(&BotType::Random, None)?;

// Greedy with custom weights
let greedy = create_bot(&BotType::Greedy, Some("data/weights/aggressive.toml"))?;

// MCTS configurations
let mcts_fast = create_bot(&BotType::Mcts(MctsConfig::fast()), None)?;        // 100 sims
let mcts_default = create_bot(&BotType::Mcts(MctsConfig::default()), None)?;  // 1000 sims
let mcts_strong = create_bot(&BotType::Mcts(MctsConfig::strong()), None)?;    // 5000 sims

// Alpha-Beta with depth
let ab = create_bot(&BotType::AlphaBeta(AlphaBetaConfig { depth: 8 }), None)?;
```

### GreedyBot Weights (28 Parameters)

The heuristic evaluator uses tunable weights for:

- **Life**: own_life, enemy_life_damage
- **Creatures**: attack, health, count, board advantage
- **Resources**: cards in hand, action points
- **Keywords**: guard, lethal, lifesteal, rush, ranged, piercing, shield, quick, ephemeral, regenerate, stealth, charge, frenzy, volatile, fortify, ward
- **Terminal**: win bonus (~+999), lose penalty (~-1000)

Load custom weights: `data/weights/` or `data/weights/archetypes/`

## ML Interface

### State Tensor (328 floats)

```rust
pub trait GameEnvironment {
    fn get_state_tensor(&self) -> [f32; 328];
    fn get_legal_action_mask(&self) -> [f32; 256];
    fn apply_action_by_index(&mut self, index: u8);
    fn get_reward(&self, player: PlayerId) -> f32;  // -1.0, 0.0, 1.0
    fn fork(&self) -> Self;  // Clone for MCTS
}
```

| Section | Indices | Size | Content |
|---------|---------|------|---------|
| Global | 0-5 | 6 | Turn, current player, game state |
| Player 1 | 6-80 | 75 | Life, essence, AP, hand, board |
| Player 2 | 81-155 | 75 | Same as P1 |
| Card IDs | 156-325 | 170 | Normalized card IDs |
| Commanders | 326-327 | 2 | Commander IDs (÷6000) |

### Action Space (256 indices)

| Range | Action |
|-------|--------|
| 0-49 | PlayCard (hand × slot) |
| 50-74 | Attack (attacker × defender) |
| 75-253 | UseAbility (slot × ability × target) |
| 254 | CommanderInsight (draw for 4 essence) |
| 255 | EndTurn |

## CLI Tools

### Arena - Head-to-Head Matches

```bash
cargo run --release --bin arena -- \
    --bot1 mcts --bot2 greedy \
    --games 100 --seed 42
```

### Validate - Quick Balance Check

```bash
cargo run --release --bin validate -- --progress
cargo run --release --bin validate -- -n 50 --progress
```

### Benchmark - Thorough Analysis

```bash
# Alpha-Beta (default)
cargo run --release --bin benchmark -- --ab-depth 6 --progress

# MCTS alternative
cargo run --release --bin benchmark -- --bot mcts --mcts-sims 200
```

### Swiss - Tournament Mode

```bash
cargo run --release --bin swiss -- --games 20 --progress
cargo run --release --bin swiss -- --faction argentum --games 30
```

### Tune - Weight Optimization

```bash
# Tune all archetypes
./scripts/tune-archetypes.sh

# Specific archetypes
./scripts/tune-archetypes.sh aggro tempo

# Preview commands
./scripts/tune-archetypes.sh --dry-run
```

### Replay - Game Analysis

```bash
# Generate and summarize
cargo run --release --bin replay -- --seed 12345 --deck1 iron_wall --deck2 swarm_aggro

# Interactive step-through
cargo run --release --bin replay -- --seed 12345 --deck1 iron_wall --deck2 swarm_aggro -i

# Export transcript
cargo run --release --bin replay -- --seed 12345 --deck1 iron_wall --deck2 swarm_aggro --export transcript -o game.txt
```

## Data Formats

### Card Definitions (YAML)

Location: `data/cards/core_set/{faction}/creatures.yaml`

```yaml
- id: 1000
  name: Brass Sentinel
  cost: 2
  card_type: creature
  attack: 2
  health: 5
  keywords: [Guard]
  rarity: Common
  abilities:
    - trigger: OnPlay
      targeting: TargetAllyCreature
      effects:
        - type: buff_stats
          attack: 1
```

### Deck Definitions (TOML)

Location: `data/decks/{faction}/{deck_name}.toml`

```toml
id = "iron_wall"
name = "Iron Wall"
description = "Impenetrable fortress defense"
playstyle = "Control"
commander = 5003

cards = [
    1000, 1000, 1000,  # Brass Sentinel x3
    1001, 1001, 1002,  # ...
    # 30 cards total (excluding commander)
]
```

### Weight Files (TOML)

Location: `data/weights/{name}.toml`

```toml
[default.greedy]
own_life = 2.74
enemy_life_damage = 1.81
own_creature_attack = 1.64
keyword_guard = 2.63
keyword_lethal = 3.11
# ... 28 total weights
```

## Performance

| Operation | Throughput |
|-----------|-----------|
| Random game | ~33k/sec |
| Greedy game | ~4.3k/sec |
| MCTS (100 sims) | ~22ms/move |
| Alpha-Beta (depth 6) | ~8s/game |
| Engine fork | ~245 ns |
| State tensor | ~158 ns |
| Legal actions | ~55 ns |

Run benchmarks: `cargo bench -p cardgame`

## Architecture

```
src/
├── core/           # Game engine core
│   ├── types.rs    # CardId, PlayerId, Slot, etc.
│   ├── keywords.rs # 16 keywords (u16 bitfield)
│   ├── state.rs    # GameState, PlayerState
│   ├── actions.rs  # Action types
│   ├── combat.rs   # Combat resolution
│   ├── engine/     # GameEngine + handlers
│   └── legal.rs    # Legal action generation
├── bots/           # AI implementations
│   ├── random.rs
│   ├── greedy.rs
│   ├── mcts.rs
│   ├── alphabeta.rs
│   └── weights.rs
├── tensor.rs       # ML state encoding
├── decks.rs        # Deck registry
├── replay/         # Game recording
├── arena/          # Match execution
├── validation/     # Balance tools
└── bin/            # CLI tools
```

## Testing

Tests are **separate from source** (not inline `#[cfg(test)]`):

- **Unit tests**: `tests/unit/<module>_tests.rs`
- **Integration tests**: `tests/*.rs`
- **Shared utilities**: `tests/common/mod.rs`

```bash
cargo nextest run -p cardgame --status-level=fail
```

## Features

```toml
[features]
default = ["parallel", "cli"]
parallel = ["dep:rayon"]           # Parallel game execution
cli = [...]                        # CLI tools
python = ["dep:pyo3", "dep:numpy"] # Python bindings
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
