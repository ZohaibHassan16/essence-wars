# CLAUDE.md - AI Assistant Context for Essence Wars

## Project Overview

**Essence Wars** is a deterministic, perfect-information card game engine designed for AI research (reinforcement learning, MCTS). The engine is written in Rust with a focus on performance and correctness.

## Quick Commands

```bash
# Build
cargo build --release

# Run all tests
cargo test  # 224 tests

# Run arena matches
cargo run --release --bin arena -- --bot1 greedy --bot2 random --games 100
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 10
cargo run --release --bin arena -- --deck1 aggressive_assault --deck2 defensive_control
cargo run --release --bin arena -- --list-decks
cargo run --release --bin arena -- --progress --games 1000  # Show progress bar

# Run weight tuning
cargo run --release --bin tune -- --mode vs-random --generations 50
cargo run --release --bin tune -- --mode vs-greedy --generations 50
cargo run --release --bin tune -- --mode specialist --deck aggressive_assault --opponent defensive_control
cargo run --release --bin tune -- --output tuned_weights.toml

# Run benchmarks
cargo bench
```

## Project Structure

```
ai-cardgame/
├── src/
│   ├── lib.rs          # Module exports
│   ├── types.rs        # Core types: CardId, PlayerId, Slot, etc.
│   ├── keywords.rs     # 8 keywords as u8 bitfield
│   ├── effects.rs      # Trigger, Effect, EffectTarget, TargetingRule
│   ├── state.rs        # GameState, PlayerState, Creature, Support
│   ├── cards.rs        # CardDefinition, CardDatabase, YAML loading
│   ├── actions.rs      # Action enum with index mapping (256 actions)
│   ├── legal.rs        # Legal action generation
│   ├── engine.rs       # GameEngine, effect queue, turn structure
│   ├── combat.rs       # Combat resolution with keyword interactions
│   ├── tensor.rs       # State to tensor conversion (326 floats)
│   ├── config.rs       # GameConfig with all parameters
│   ├── decks.rs        # DeckDefinition, DeckRegistry, TOML loading
│   ├── bots/
│   │   ├── mod.rs      # Bot trait definition
│   │   ├── random.rs   # RandomBot - uniform random selection
│   │   ├── greedy.rs   # GreedyBot - heuristic evaluation, action simulation
│   │   ├── mcts.rs     # MctsBot - Monte Carlo Tree Search with UCB1
│   │   └── weights.rs  # Configurable weights for GreedyBot (20 params)
│   ├── arena/
│   │   ├── mod.rs      # Arena module exports
│   │   ├── runner.rs   # GameRunner for executing matches
│   │   ├── logger.rs   # ActionLogger for debug tracing
│   │   └── stats.rs    # MatchStats for win rate tracking
│   ├── tuning/
│   │   ├── mod.rs      # Tuning module exports
│   │   ├── cmaes.rs    # CMA-ES optimizer implementation
│   │   └── evaluator.rs # Fitness evaluation via game matches
│   └── bin/
│       ├── arena.rs    # CLI for running bot matches
│       └── tune.rs     # CLI for weight optimization
├── benches/
│   └── game_benchmarks.rs  # Criterion benchmarks
├── data/
│   ├── cards/sets/
│   │   └── starter.yaml  # 43 card definitions
│   └── decks/
│       ├── aggressive_assault.toml  # Aggro deck
│       └── defensive_control.toml   # Control deck
├── tests/
│   ├── engine_tests.rs
│   ├── effect_queue_tests.rs
│   ├── card_playing_tests.rs
│   ├── game_interface_tests.rs
│   ├── integration_tests.rs
│   └── simulation_tests.rs
└── docs/
    ├── essence-wars-design.md
    ├── design-engine.md
    └── implementation-plan.md
```

## Bot System

### Bot Types

| Bot | Description | Strength |
|-----|-------------|----------|
| RandomBot | Uniform random action selection | Baseline |
| GreedyBot | Simulates each action, picks best by heuristic | Beats Random 100% |
| MctsBot | UCB1 tree search with GreedyBot rollouts | Beats Greedy 60-100% |

### Bot Trait

```rust
pub trait Bot: Send {
    fn name(&self) -> &str;
    fn select_action(
        &mut self,
        state_tensor: &[f32; 326],
        legal_mask: &[f32; 256],
        legal_actions: &[Action],
    ) -> Action;
    fn reset(&mut self);
    fn clone_box(&self) -> Box<dyn Bot>;
}
```

### GreedyBot Weights (20 parameters)

GreedyBot uses configurable weights for state evaluation:

| Category | Parameters |
|----------|------------|
| Life | own_life, enemy_life_damage |
| Creatures | own_creature_attack, own_creature_health, enemy_creature_attack, enemy_creature_health |
| Board | creature_count, board_advantage |
| Resources | cards_in_hand, action_points |
| Keywords | guard, lethal, lifesteal, rush, ranged, piercing, shield, quick |
| Terminal | win_bonus, lose_penalty |

Weights can be saved/loaded as TOML for tuning:

```toml
name = "tuned_vs_greedy"
version = 1

[default.greedy]
own_life = 1.5
enemy_life_damage = 2.0
# ... etc
```

### MCTS Configuration

```rust
MctsConfig {
    simulations: 500,      // Iterations per move
    exploration: 1.414,    // UCB1 exploration constant (sqrt(2))
    max_rollout_depth: 100,
}

// MCTS with custom rollout weights
let mcts = MctsBot::with_config_and_weights(&card_db, config, &weights, seed);
```

MCTS uses GreedyBot for rollout evaluation. Custom weights improve rollout quality, leading to better move selection.

## Weight Tuning Pipeline

### CMA-ES Optimizer

The tuning system uses CMA-ES (Covariance Matrix Adaptation Evolution Strategy) to optimize GreedyBot weights.

```rust
CmaEsConfig {
    population_size: None,  // Default: 4 + floor(3 * ln(dim))
    initial_sigma: 0.5,     // Initial step size
    max_generations: 100,   // Max iterations
    target_fitness: None,   // Early stop threshold
    seed: 42,
}
```

### Tuning Modes

| Mode | Description |
|------|-------------|
| `vs-random` | Optimize to beat RandomBot (easy baseline) |
| `vs-greedy` | Optimize to beat default GreedyBot |
| `multi-opponent` | Optimize vs Random (10%), Greedy (40%), MCTS (50%) |
| `generalist` | Optimize across all deck matchups |
| `specialist` | Optimize for specific deck vs opponent |

### Parallel Evaluation

Tuning now uses parallel game evaluation across all CPU cores (enabled by default).
- 14x speedup on 16-core machines
- Use `--parallel false` to disable

### Tune CLI

```bash
# Basic tuning against random
cargo run --release --bin tune -- --generations 50 --games 50

# Tune against greedy baseline
cargo run --release --bin tune -- --mode vs-greedy --generations 100

# Multi-opponent tuning (most robust)
cargo run --release --bin tune -- --mode multi-opponent --generations 100 --games 200

# Specialist tuning for a specific matchup
cargo run --release --bin tune -- \
  --mode specialist \
  --deck aggressive_assault \
  --opponent defensive_control \
  --generations 50 \
  --output aggro_specialist.toml

# Generalist tuning across all decks
cargo run --release --bin tune -- --mode generalist --generations 100

# Full options
cargo run --release --bin tune -- \
  --mode vs-greedy \
  --generations 100 \
  --population 20 \
  --games 100 \
  --sigma 0.3 \
  --target-win-rate 0.95 \
  --seed 12345 \
  --initial-weights existing.toml \
  --output optimized.toml \
  --verbose
```

## Deck System

### TOML Format

```toml
id = "aggressive_assault"
name = "Aggressive Assault"
description = "Fast aggro deck"
tags = ["aggro", "creature-focused"]

cards = [
    1, 1,   # Eager Recruit x2
    3, 3,   # Nimble Scout x2
    # ... 18 total cards
]
```

### Arena CLI

```bash
# List available decks
cargo run --bin arena -- --list-decks

# Run match with specific decks
cargo run --bin arena -- --deck1 aggressive_assault --deck2 defensive_control

# Use tuned weights for greedy or MCTS bots
cargo run --bin arena -- \
  --bot1 greedy --bot2 greedy \
  --weights1 tuned_weights.toml \
  --games 100

# MCTS with tuned rollout weights vs default MCTS
cargo run --bin arena -- \
  --bot1 mcts --bot2 mcts \
  --weights1 tuned_weights.toml \
  --games 20

# Full options
cargo run --bin arena -- \
  --bot1 mcts --bot2 greedy \
  --deck1 aggressive_assault --deck2 defensive_control \
  --weights1 tuned.toml --weights2 default.toml \
  --games 100 --seed 12345 \
  --debug  # or --verbose for state snapshots \
  --progress  # show progress bar
```

## Performance

### Benchmarks

```bash
cargo bench  # Run all benchmarks
```

| Benchmark | Time | Throughput |
|-----------|------|------------|
| Random game | ~12 µs | ~80,000 games/sec |
| Greedy game | ~58 µs | ~17,000 games/sec |
| State tensor | ~138 ns | - |
| Legal actions | ~29 ns | - |
| Engine fork | ~99 ns | - |

### Key Optimizations

- **ArrayVec** for fixed-size collections - stack allocation, fast cloning for MCTS
- **u8 bitfield** for keywords - compact representation
- **Indexed action space** (256 actions) - fixed-size for neural networks
- **Engine fork()** - efficient state cloning for tree search

## Key Design Decisions

### Game Rules
- 5 creature slots, 2 support slots per player
- 3 Action Points per turn
- 30 turn limit with life-based tiebreaker
- 8 keywords: Rush, Ranged, Piercing, Guard, Lifesteal, Lethal, Shield, Quick

### AI Interface (GameEnvironment trait)
- `get_state_tensor()` - 326 floats representing full game state
- `get_legal_action_mask()` - 256 floats (0.0 or 1.0)
- `apply_action_by_index(u8)` - apply action from neural network output
- `get_reward(PlayerId)` - -1.0, 0.0, or 1.0
- `fork()` - clone game state for MCTS tree search

## Card YAML Schema

```yaml
# Creature
- id: 9
  name: "Blood Cultist"
  cost: 2
  card_type: creature
  attack: 3
  health: 2
  keywords: [Rush]  # optional
  abilities:        # optional
    - trigger: OnPlay
      effects:
        - type: damage
          amount: 2

# Spell
- id: 34
  name: "Lightning Bolt"
  cost: 3
  card_type: spell
  targeting: TargetAny
  effects:
    - type: damage
      amount: 4

# Support
- id: 40
  name: "War Drums"
  cost: 3
  card_type: support
  durability: 3
  passive_effects:
    - modifier:
        attack_bonus: 1
```

## State Tensor Layout (326 floats)

| Section | Size | Description |
|---------|------|-------------|
| Global | 10 | turn, phase, active_player, game_over, winner |
| Player 1 creatures | 60 | 5 slots × 12 floats |
| Player 2 creatures | 60 | 5 slots × 12 floats |
| Player 1 supports | 8 | 2 slots × 4 floats |
| Player 2 supports | 8 | 2 slots × 4 floats |
| Player 1 hand | 60 | 20 cards × 3 floats |
| Player 2 hand | 60 | 20 cards × 3 floats |
| Player 1 deck | 30 | 30 card IDs |
| Player 2 deck | 30 | 30 card IDs |

## Action Space (256 indices)

| Range | Action Type |
|-------|-------------|
| 0-99 | PlayCard (hand 0-9 × slot 0-9) |
| 100-149 | Attack (slot 0-4 × target 0-9) |
| 150-249 | UseAbility (slot 0-4 × ability 0-1 × target 0-9) |
| 255 | EndTurn |

## Implementation Status

**Complete:**
- All core game rules and 8 keywords
- AI interface (tensor, action mask, rewards)
- 43-card starter set
- Bot system (RandomBot, GreedyBot, MctsBot)
- Arena for running matches with progress indicator
- Deck system with TOML definitions
- Weight tuning pipeline with CMA-ES optimizer
- Criterion benchmarks for performance testing
- 224 tests passing

**Future work:**
- Python bindings (PyO3) for ML training
- Additional card sets
- Web-based game viewer

## Python Tooling

### Setup with uv
```bash
# Sync all dependencies
uv sync --all-groups

# Run tests
uv run pytest python/tests -v

# Linting & type checking
uv run ruff check python/
uv run mypy python/
```

### Python Package Structure
```
python/
├── cardgame/
│   ├── __init__.py      # Package root
│   ├── infra/           # Experiment management
│   ├── analysis/        # Visualization tools
│   ├── env/             # Gym environments (future)
│   └── algo/            # RL algorithms (future)
└── tests/
    └── test_*.py
```

## Data & Experiment Strategy

> **Important**: Follow these guidelines for all training runs and experiments.

### Directory Structure (Strictly Enforced)
```
ai-cardgame/
├── experiments/           # ALL run artifacts (GITIGNORED)
│   ├── tuning/           # Weight tuning runs
│   ├── training/         # RL training runs
│   └── eval/             # Benchmarking runs
├── data/                  # Large binary data (GITIGNORED)
│   ├── weights/          # Tuned weight files
│   ├── replays/          # Game replays for offline RL
│   └── datasets/         # Pre-processed datasets
└── docs/experiments/      # Curated experiment reports (committed)
```

### Experiment ID Convention
Every run gets: `{YYYY-MM-DD_HHMM}_{tag}`

Example: `2026-01-12_1430_mcts_baseline`

### Experiment Workflow
1. **Launch**: Script creates timestamped folder, saves config.yaml
2. **Run**: Logs metrics to stats.csv, saves checkpoints
3. **Analyze**: Parse logs, generate plots
4. **Archive**: Keep important runs, delete failures

### Rules for Claude
- **ALWAYS** create experiment folders for training/tuning runs
- **ALWAYS** save config.yaml at start of experiment
- **NEVER** commit experiments/ or data/ to git
- **USE** `docs/experiments/` for curated reports worth preserving
