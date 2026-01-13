# CLAUDE.md - AI Assistant Context for Essence Wars

## Project Overview

**Essence Wars** is a deterministic, perfect-information card game engine designed for AI research (reinforcement learning, MCTS). The engine is written in Rust with a focus on performance and correctness.

## Quick Commands

```bash
# Build
cargo build --release

# Run all tests
cargo nextest run --status-level=fail  # ~485 tests (only shows failures)
cargo test                              # Alternative: use standard cargo test

# Run stress tests by tier (use helper script)
./scripts/run-tests.sh                  # Standard tests only
./scripts/run-tests.sh quick            # + quick tier (~2 min)
./scripts/run-tests.sh medium           # + medium tier (~10 min)
./scripts/run-tests.sh long             # + long tier (~30 min)
./scripts/run-tests.sh overnight        # + overnight tier (~1-2 hours)

# Or run tiers directly
cargo nextest run --release -- --ignored tier_quick
cargo nextest run --release -- --ignored tier_medium
cargo nextest run --release -- --ignored tier_long
cargo nextest run --release -- --ignored tier_overnight

# Run arena matches
cargo run --release --bin arena -- --bot1 greedy --bot2 random --games 100
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 10
cargo run --release --bin arena -- --deck1 aggressive_assault --deck2 defensive_control
cargo run --release --bin arena -- --list-decks
cargo run --release --bin arena -- --progress --games 1000  # Show progress bar

# Run weight tuning (NEW: outputs to experiments/ directory)
cargo run --release --bin tune -- --tag baseline --generations 50
cargo run --release --bin tune -- --tag vs_greedy --mode vs-greedy --generations 50
cargo run --release --bin tune -- --tag specialist --mode specialist --deck aggressive_assault --opponent defensive_control

# Analyze tuning results (NEW: unified analysis pipeline)
./scripts/analyze-tuning.sh --latest                           # Analyze latest experiment
./scripts/analyze-tuning.sh experiments/mcts/2026-01-12_1430_baseline  # Analyze specific
./scripts/analyze-tuning.sh --all                              # Analyze all experiments

# Run benchmarks
cargo bench
```

## Project Structure

```
ai-cardgame/
├── src/
│   ├── lib.rs          # Module exports
│   ├── version.rs      # Version info for reproducibility
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
│   │   ├── starter.yaml        # 47 cards (base + Phase 1.5 test)
│   │   └── new-horizons.yaml   # 60 cards (4 faction batches)
│   ├── decks/
│   │   ├── argentum_fortress.toml    # Argentum faction deck
│   │   ├── symbiote_swarm.toml       # Symbiote faction deck
│   │   ├── obsidion_shadow.toml      # Obsidion faction deck
│   │   ├── freewalker_mercenary.toml # Free-Walker utility deck
│   │   ├── aggressive_assault.toml   # Starter aggro deck
│   │   └── defensive_control.toml    # Starter control deck
│   └── weights/
│       ├── specialists/        # Faction-specific weights
│       │   ├── argentum.toml
│       │   ├── symbiote.toml
│       │   └── obsidion.toml
│       └── generalist.toml     # Cross-faction weights
├── experiments/             # Experiment outputs (gitignored)
│   ├── mcts/                # MCTS tuning experiments
│   │   └── YYYY-MM-DD_HHMM_tag/
│   │       ├── train.log    # Full training log
│   │       ├── weights.toml # Best weights found
│   │       ├── stats.csv    # Parsed metrics (generated)
│   │       ├── summary.txt  # Quick stats
│   │       └── plots/       # Visualizations (generated)
│   ├── ppo/                 # Future: PPO training runs
│   └── alphazero/           # Future: AlphaZero runs
├── python/
│   ├── cardgame/
│   │   ├── infra/           # Experiment management
│   │   ├── analysis/        # Visualization & stats tools
│   │   │   ├── parse_log.py
│   │   │   └── visualize.py
│   │   └── scripts/         # Test scripts
│   └── scripts/
│       └── analyze_tuning.py # Main analysis CLI
├── scripts/
│   └── analyze-tuning.sh    # Wrapper script for easy usage
├── tests/
│   ├── common/             # Shared test utilities
│   │   └── mod.rs
│   ├── unit/               # Unit tests for src/core/ modules
│   │   ├── types_tests.rs
│   │   ├── keywords_tests.rs
│   │   ├── config_tests.rs
│   │   ├── effects_tests.rs
│   │   ├── state_tests.rs
│   │   ├── cards_tests.rs
│   │   ├── actions_tests.rs
│   │   ├── legal_tests.rs
│   │   └── combat_tests.rs
│   ├── unit.rs             # Unit test entry point
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

## Test Organization

**IMPORTANT**: This project keeps unit tests **separate from source code**, not inline with `#[cfg(test)] mod tests` blocks as is common in Rust. This reduces token usage when AI assistants read source files.

- **Unit tests**: Go in `tests/unit/<module>_tests.rs` (e.g., `tests/unit/types_tests.rs` for `src/core/types.rs`)
- **Integration tests**: Go directly in `tests/*.rs` (e.g., `tests/engine_tests.rs`)
- **Shared test utilities**: Go in `tests/common/mod.rs`

When adding new tests for core modules, create them in `tests/unit/` and add the module to `tests/unit.rs`.

## Test Tiers & CI/CD

Tests are organized into tiers by runtime:

| Tier | Duration | When Run | Examples |
|------|----------|----------|----------|
| **Standard** | ~2 min | Every commit | Unit tests, integration tests |
| **tier_quick** | ~2 min | Every PR | 20-50 game MCTS tests |
| **tier_medium** | ~10 min | Nightly | 100 game bot validation |
| **tier_long** | ~30 min | Nightly | 500 game stress tests |
| **tier_overnight** | ~1-2 hours | Weekly | 100k game exhaustive tests |

### GitHub Actions Workflows

- **ci.yml** - Runs on every push/PR: formatting, linting, standard tests, quick tier
- **nightly.yml** - Runs at 2 AM UTC daily: quick + medium + long tiers
- **weekly.yml** - Runs Sundays at 3 AM UTC: all tiers including overnight

### Running Tests Locally

```bash
./scripts/run-tests.sh              # Standard tests
./scripts/run-tests.sh quick        # + quick (~2 min)
./scripts/run-tests.sh medium       # + quick + medium (~12 min)
./scripts/run-tests.sh long         # + all above + long (~45 min)
./scripts/run-tests.sh overnight    # Full suite (~2 hours)
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

### GreedyBot Weights (24 parameters)

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
| `faction-specialist` | Train specialist for a faction (requires `--faction`) |
| `agent-generalist` | Train generalist against all faction specialists + mirror |

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

# === Faction-based Agent Training ===

# Train Argentum specialist (saves to data/weights/specialists/argentum.toml)
cargo run --release --bin tune -- \
  --mode faction-specialist --faction argentum \
  --tag argentum_v1 --generations 100

# Train Symbiote specialist
cargo run --release --bin tune -- \
  --mode faction-specialist --faction symbiote \
  --tag symbiote_v1 --generations 100

# Train Obsidion specialist
cargo run --release --bin tune -- \
  --mode faction-specialist --faction obsidion \
  --tag obsidion_v1 --generations 100

# Train generalist (saves to data/weights/generalist.toml)
cargo run --release --bin tune -- \
  --mode agent-generalist \
  --tag generalist_v1 --generations 100

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

# === Agent Types (with auto-loaded specialist weights) ===

# Faction specialist vs specialist match
cargo run --release --bin arena -- \
  --bot1 agent-argentum --deck1 argentum_control \
  --bot2 agent-symbiote --deck2 symbiote_aggro \
  --games 100

# Generalist vs all specialists
cargo run --release --bin arena -- \
  --bot1 agent-generalist --bot2 agent-obsidion \
  --deck2 obsidion_burst --games 100

# Available Agent bot types:
#   agent-argentum    - Argentum Combine specialist
#   agent-symbiote    - Symbiote Circles specialist
#   agent-obsidion    - Obsidion Syndicate specialist
#   agent-generalist  - Works with all factions

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

## Faction System

Essence Wars uses a **faction-based card system** with three true factions plus neutral cards:

### True Factions

| Faction | Identity | Primary Keywords | Playstyle |
|---------|----------|------------------|-----------|
| **Argentum Combine** | "The Wall" | Guard, Piercing, Shield | Defensive, high-HP, outlast |
| **Symbiote Circles** | "The Swarm" | Rush, Lethal, Regenerate | Aggressive tempo, efficient trades |
| **Obsidion Syndicate** | "The Shadow" | Lifesteal, Stealth, Ephemeral, Quick | Burst damage, life manipulation |

### Neutral Cards

| Category | Identity | Primary Keywords | Role |
|----------|----------|------------------|------|
| **Free-Walkers** | "The Toolbox" | Ranged, Charge | Utility splash for any faction |

Free-Walkers are **not a standalone faction**—they are neutral utility cards (like MTG artifacts) that can be splashed into any faction deck.

### Deck Composition

Standard decks follow the **Faction Core + Neutral Splash** model:

```
STANDARD DECK: 20 cards
├── Faction Core: 14 cards (70%)    ← Primary faction identity
└── Neutral Splash: 6 cards (30%)   ← Free-Walker utility
```

## Agent Architecture

All AI agents (MCTS, PPO, AlphaZero) follow this architecture:

### Agent Types

| Agent | Faction | Deck Binding | Training |
|-------|---------|--------------|----------|
| **Agent-Argentum** | Argentum | `argentum_*` only | vs other specialists + mirror |
| **Agent-Symbiote** | Symbiote | `symbiote_*` only | vs other specialists + mirror |
| **Agent-Obsidion** | Obsidion | `obsidion_*` only | vs other specialists + mirror |
| **Agent-Generalist** | Any | Any deck | vs all specialists + mirror (25% each) |

**Key Rule:** Specialists are **bound to their faction's decks**. Generalist can play any deck.

### Weight/Model Organization

```
data/weights/                     # MCTS/Greedy weights
├── specialists/
│   ├── argentum.toml
│   ├── symbiote.toml
│   └── obsidion.toml
└── generalist.toml

models/                           # Neural networks (future)
├── ppo/
│   ├── specialists/{argentum,symbiote,obsidion}/
│   └── generalist/
└── alphazero/
    ├── specialists/{argentum,symbiote,obsidion}/
    └── generalist/
```

### Balance Testing

| Test Type | Purpose | Configuration |
|-----------|---------|---------------|
| **Deck Balance** | Are factions balanced? | Generalist vs Generalist, all matchups |
| **Specialist Quality** | Do specialists outperform? | Specialist vs Generalist, same deck |
| **Full Tournament** | Meta health | All meaningful permutations (27 configs) |

**Balance Targets:** 45-55% win rate between factions, no dominant strategy.

## Key Design Decisions

### Game Rules
- 5 creature slots, 2 support slots per player
- 3 Action Points per turn
- 30 turn limit with life-based tiebreaker
- 12 keywords: Rush, Ranged, Piercing, Guard, Lifesteal, Lethal, Shield, Quick, Ephemeral, Regenerate, Stealth, Charge

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
- All core game rules and 12 keywords
- Essence/mana system (grows +1/turn, caps at 10)
- AI interface (tensor, action mask, rewards)
- 107-card pool (47 starter + 60 New Horizons expansion)
- Faction system (3 factions + neutrals)
- Bot system (RandomBot, GreedyBot, MctsBot)
- Arena for running matches with progress indicator
- Deck system with TOML definitions (6 decks)
- Weight tuning pipeline with CMA-ES optimizer
- Version tracking for ML reproducibility
- Criterion benchmarks for performance testing
- CI/CD with GitHub Actions (nightly + weekly)
- ~485 tests passing

**In Progress:**
- Faction-specific weight tuning (specialists)
- Balance iteration on New Horizons cards

**Future work:**
- Python bindings (PyO3) for ML training
- PPO and AlphaZero agents
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

## Versioning

Version is in `Cargo.toml` line 3. Uses Semantic Versioning: `MAJOR.MINOR.PATCH`

### When to Bump Version

| Change Type | Version Bump | Examples |
|-------------|--------------|----------|
| Game rules/mechanics change | MINOR | Essence system fix, combat changes |
| API breaking changes | MINOR (pre-1.0) | Tensor layout change, action space change |
| New features | MINOR | New bot type, new card effects |
| Bug fixes (no behavior change) | PATCH | Fix crash, fix test |
| Refactoring/cleanup | PATCH | Code cleanup, perf optimization |

### Version Update Checklist

When making changes that warrant a version bump:

1. **Update `Cargo.toml`**: Change `version = "X.Y.Z"`
2. **Update `src/version.rs`**: Update the test assertion for VERSION
3. **Update `CHANGELOG.md`**: Add entry under new version header
4. **Run tests**: `cargo nextest run --status-level=fail`

### Reproducibility

Experiments save `version.toml` with engine version + git hash. This ensures ML experiments are reproducible:

```rust
use cardgame::version::{self, VersionInfo};

// Log version at experiment start
println!("Engine: {}", version::version_string());
// Output: "cardgame v0.2.0 (d03a75c1)"

// Save for reproducibility
let info = VersionInfo::current();
```
