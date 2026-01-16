# AI Coding Agent Instructions - Essence Wars

## Project Overview
**Essence Wars** is a deterministic, perfect-information card game engine built in Rust for AI research (MCTS, RL). Think "Chess with Cards" - no hidden information, no RNG during play. The engine prioritizes performance (cloning speed for tree search) and correctness (500+ tests).

## Architecture

### Core Rust Engine (`src/`)
- **Game Loop**: core/engine module - Effect queue system (FIFO, no recursion), turn structure, `GameEnvironment` trait for AI
- **State**: core/state module - Uses `ArrayVec` for stack allocation (fast MCTS cloning)
- **Actions**: core/actions module - Fixed 256-index action space (0-99: PlayCard, 100-149: Attack, 150-249: UseAbility, 255: EndTurn)
- **Combat**: core/combat module - Lane-based, 8 keywords as u8 bitfield (Guard, Rush, Lethal, etc.)
- **AI Interface**: tensor module - 326-float state tensor, `get_legal_action_mask()` for neural networks

### Bot System (`src/bots/`)
- **RandomBot**: Baseline uniform random
- **GreedyBot**: Simulate-and-evaluate with 24 tunable weights
- **MctsBot**: UCB1 tree search, uses GreedyBot for rollouts (configurable weights improve search quality)

### Tuning & Arena (`src/tuning/`, `src/arena/`)
- **CMA-ES Optimizer**: tuning/cmaes module - Parallel fitness evaluation (14x speedup on 16 cores)
- **Experiment Outputs**: `experiments/{mcts,ppo,alphazero}/YYYY-MM-DD_HHMM_tag/` (gitignored)
- **GameRunner**: arena/runner module - Executes bot matches, optional ActionLogger for replay
- **Modal Cloud**: modal_tune.py - Serverless parallel tuning/validation (4x faster than local)

### Python Tooling (`python/`)
- **Analysis**: `python/cardgame/analysis/` - Parses tuning logs, generates plots
- **Entry Script**: analyze-tuning.sh script - Wrapper using `uv run` (no venv needed)

## Critical Workflows

### Build & Test
```bash
cargo build --release          # 14s typical
cargo test                     # 500+ tests, ~10s
cargo bench                    # Criterion benchmarks
./scripts/run-clippy.sh        # Lint production code (excludes tests)
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
```

### Tune Weights (Outputs to experiments/)
```bash
# Local tuning (vs random baseline)
cargo run --release --bin tune -- --tag baseline --generations 50

# Multi-opponent (most robust, recommended)
cargo run --release --bin tune -- --tag vs_all --mode multi-opponent --generations 100 --games 200

# Specialist for specific matchup
cargo run --release --bin tune -- --tag aggro_spec \
  --mode specialist --deck symbiote_aggro --opponent argentum_control

# Cloud tuning via Modal (4x faster, parallel execution)
modal run modal_tune.py::main --mode train-only
modal run modal_tune.py::main --single argentum  # Single specialist
```

### Analyze Results
```bash
./scripts/analyze-tuning.sh --latest                    # Latest experiment
./scripts/analyze-tuning.sh experiments/mcts/2026-01-12_1430_baseline
```

### Validation
```bash
# Local validation (quick check)
cargo run --release --bin validate -- --games 100 --output results.json

# Cloud validation (high confidence, 1500 games per matchup)
modal run modal_tune.py::main --mode validate-only --validation-games 1500
```

## Project-Specific Conventions

### Test Organization (CRITICAL!)
**Tests live separately from source code** (not inline `#[cfg(test)]` blocks). This keeps source files token-lean for AI context.
- Unit tests: tests/unit directory with _tests.rs files (e.g., types_tests.rs for core/types module)
- Integration tests: tests directory with _tests.rs files (e.g., engine_tests.rs)
- Shared utilities: tests/common/mod module

When adding tests for a core module, create the corresponding _tests.rs file in tests/unit and register it in the unit.rs file.

### Data & Experiment Strategy
**NEVER commit to git:**
- `experiments/` - All training/tuning run artifacts
- `data/weights/` - Large binary weights
- `data/datasets/`, `data/replays/` - ML datasets

**Experiment ID Convention:** `{YYYY-MM-DD_HHMM}_{tag}` (e.g., `2026-01-12_1430_baseline`)

Every training run must:
1. Create timestamped folder in `experiments/{mcts,ppo,alphazero}/`
2. Save `config.yaml` at start
3. Log metrics to `stats.csv`
4. Generate plots in `plots/` subdirectory

Use `docs/experiments/` for curated reports worth preserving in git.

### Card Definitions
- **YAML format**: data/cards/core_set - 107+ cards with effects/abilities (expanding to 300 for New Horizons Edition)
- **Deck format**: TOML files in data/decks - Card ID arrays (20-30 cards), organized by faction (argentum, obsidion, symbiote)

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

1. **Don't inline tests** - Use `tests/` directory structure
2. **Don't forget `--release`** - Debug builds are ~10x slower
3. **Check `--list-decks`** - Before using custom deck IDs in arena/tune
4. **Arena needs card DB** - Default path `data/cards`, override with `--cards`
5. **Weights are optional** - GreedyBot/MctsBot use defaults if no `--weights` specified

## Documentation References
- Game rules: design-engine.md in docs - Complete game specification
- Full context: CLAUDE.md in root - Detailed project documentation (500 lines)
- Roadmap: roadmap.txt in root - Future work (Python bindings, PPO/AlphaZero, web client)
