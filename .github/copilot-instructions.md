# AI Coding Agent Instructions - Essence Wars

## Project Overview
**Essence Wars** is a deterministic, perfect-information card game engine built in Rust. Think "Chess with Cards" - no hidden information, no RNG during play. The engine prioritizes performance (cloning speed for tree search) and correctness (629+ tests).

**Current Version:** 0.6.0

## Workspace Structure

This project uses a **Cargo workspace** with the core engine crate:

| Crate | Purpose | Path |
|-------|---------|------|
| `cardgame` | Core game engine, AI bots, research tools | `crates/cardgame/` |

The `cardgame` crate is pure engine code designed for AI research.

## Architecture

### Core Rust Engine (`crates/cardgame/src/`)
- **Game Loop**: core/engine module - Effect queue system (FIFO, no recursion), turn structure, `GameEnvironment` trait for AI
- **State**: core/state module - Uses `ArrayVec` for stack allocation (fast MCTS cloning)
- **Actions**: core/actions module - Fixed 256-index action space (0-99: PlayCard, 100-149: Attack, 150-249: UseAbility, 255: EndTurn)
- **Combat**: core/combat module - Lane-based, 14 keywords as u16 bitfield (Guard, Rush, Lethal, Stealth, etc.)
- **AI Interface**: tensor module - 326-float state tensor, `get_legal_action_mask()` for neural networks

### Bot System (`crates/cardgame/src/bots/`)
- **RandomBot**: Baseline uniform random
- **GreedyBot**: Simulate-and-evaluate with 24 tunable weights
- **MctsBot**: UCB1 tree search, uses GreedyBot for rollouts (configurable weights improve search quality)
- **Introspection**: AI transparency system for Glassbox visualization (MctsNodeStats, BotDecision)
- **Agent Types**: Pre-tuned bots with specialist weights (agent-argentum, agent-symbiote, agent-obsidion, agent-generalist)

### Tuning & Arena (`crates/cardgame/src/tuning/`, `src/arena/`)
- **CMA-ES Optimizer**: tuning/cmaes module - Parallel fitness evaluation (14x speedup on 16 cores)
- **Experiment Outputs**: `experiments/{mcts,ppo,alphazero}/YYYY-MM-DD_HHMM_tag/` (gitignored)
- **GameRunner**: arena/runner module - Executes bot matches, optional ActionLogger for replay
- **Modal Cloud**: modal_tune.py - Serverless parallel tuning/validation (4x faster than local)

### Diagnostics (`crates/cardgame/src/diagnostics/`)
- **P1/P2 Asymmetry Analysis**: DiagnosticRunner, GameDiagnostics for balance testing
- **Metrics**: BoardAdvantage, TempoMetrics with statistical validation (Wilson CI, chi-square)
- **Export**: CSV/JSON export for deeper analysis

### Python Tooling (`python/`)
- **Analysis**: `python/cardgame/analysis/` - Parses tuning logs, generates plots (4-panel dashboards)
- **Entry Script**: analyze-tuning.sh script - Wrapper using `uv run` (no venv needed)

## Critical Workflows

### Build & Test
```bash
# Build
cargo build --release                    # Full workspace
cargo build --release -p cardgame        # Engine only

# Run tests
cargo nextest run --status-level=fail    # Preferred: 629+ tests, only shows failures
cargo test                               # Alternative: standard cargo test

# Run linter
./scripts/run-clippy.sh                  # Lint production code (excludes tests)

# Run benchmarks
cargo bench -p cardgame                  # Criterion benchmarks
./scripts/run-benchmarks.sh              # Full benchmark suite with report
```

### Run Bots
```bash
# Arena matches
cargo run --release --bin arena -- --bot1 greedy --bot2 random --games 100 --progress

# With custom decks and weights
cargo run --release --bin arena -- \
  --deck1 symbiote_aggro --deck2 argentum_control \
  --bot1 mcts --weights1 data/weights/tuned_multi_opponent.toml \
  --games 50 --debug

# List available decks
cargo run --release --bin arena -- --list-decks
```

### Tune Weights (Outputs to experiments/)
```bash
# Local tuning - Generalist (most robust, recommended)
cargo run --release --bin tune -- --mode generalist --tag vs_all --generations 100 --games 200

# Specialist for specific matchup
cargo run --release --bin tune -- --mode specialist \
  --deck symbiote_aggro --opponent argentum_control --tag aggro_spec

# Faction specialist (auto-saves to data/weights/specialists/)
cargo run --release --bin tune -- --mode faction-specialist \
  --faction argentum --tag argentum_v1 --generations 100
```

### Validation & Diagnostics
```bash
# Balance validation (round-robin: 66 deck matchups × 2 directions × games)
cargo run --release --bin validate -- --games 100 --output results.json

# P1/P2 asymmetry diagnostics
cargo run --release --bin diagnose -- 500 --export all --include-turns
```

## Project-Specific Conventions

### Test Organization (CRITICAL!)
**Tests live separately from source code** (not inline `#[cfg(test)]` blocks). This keeps source files token-lean for AI context.
- Unit tests: `crates/cardgame/tests/unit/` with `_tests.rs` files (e.g., types_tests.rs for core/types module)
- Integration tests: `crates/cardgame/tests/` with `_tests.rs` files (e.g., engine_tests.rs)
- Shared utilities: `crates/cardgame/tests/common/mod` module

When adding tests for a core module, create the corresponding `_tests.rs` file in `crates/cardgame/tests/unit/` and register it in the `unit.rs` file.

### Data & Experiment Strategy
**NEVER commit to git:**
- `experiments/` - All training/tuning run artifacts
- `data/weights/` - Large binary weights
- `data/datasets/`, `data/replays/` - ML datasets

**Experiment ID Convention:** `{YYYY-MM-DD_HHMM}_{tag}` (e.g., `2026-01-12_1430_baseline`)


### Card Definitions
- **YAML format**: data/cards/core_set - 300 cards organized by faction (argentum.yaml, symbiote.yaml, obsidion.yaml, neutral.yaml - 75 cards each)
- **Deck format**: TOML files in data/decks - Card ID arrays (20-30 cards), organized by faction subdirectories
- **12 Commander Decks**: 4 Argentum, 4 Symbiote, 4 Obsidion (see `cargo run --release --bin arena -- --list-decks`)

### Python Environment
Use `uv` for dependency management (no manual venv):
```bash
uv sync --all-groups           # Sync dependencies
uv run pytest python/tests -v
uv run ruff check python/
```

## Key Design Patterns

### Effect Queue (Not Recursion)
Effects trigger other effects without recursion - new effects go to FIFO queue back. See core/engine module's `EffectQueue` for pattern.

### ArrayVec for Performance
Uses `arrayvec::ArrayVec` for fixed-capacity vectors (creatures, hand, deck) - stack allocation means fast cloning for MCTS. Example: `ArrayVec<Creature, 5>` for creature slots.

### Bot Trait with AI Interface
Bots receive state as tensor + legal mask, return Action. See bots/mod module:
```rust
fn select_action(&mut self, state_tensor: &[f32; 326], legal_mask: &[f32; 256], 
                 legal_actions: &[Action]) -> Action;
```

### Parallel Evaluation (Tuning)
Fitness evaluation uses Rayon for parallel game execution - enabled by default in tune.rs. Each worker gets independent RNG seed.

## Common Pitfalls

1. **Don't inline tests** - Use `crates/cardgame/tests/` directory structure
2. **Don't forget `--release`** - Debug builds are ~10x slower
3. **Check `--list-decks`** - Before using custom deck IDs in arena/tune
4. **Arena needs card DB** - Default path `data/cards`, override with `--cards`
5. **Weights are optional** - GreedyBot/MctsBot use defaults if no `--weights` specified

## Documentation References
- Game rules: design-engine.md in docs - Complete game specification
- Full context: CLAUDE.md in root - Detailed project documentation
