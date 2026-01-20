# Chris Cheat Sheet (CCS)
*Quick reference for Essence Wars scripts & tools*

## Build & Test
```bash
cargo build --release                 # Production build (~14s)
cargo nextest run -p cardgame --status-level=fail  # All tests (workspace)
cargo test -p cardgame                # Alternative test runner
cargo bench -p cardgame               # Criterion benchmarks
./scripts/run-clippy.sh               # Lint (lib + binaries only)
./scripts/run-tests.sh                # Test wrapper (standard only)
./scripts/run-tests.sh quick|medium|long|overnight  # Stress test tiers
./scripts/run-benchmarks.sh           # Full benchmark suite
```

## Arena (Bot Battles)
```bash
# Quick match
cargo run --release -p cardgame --bin arena -- \
  --bot1 greedy --bot2 random --games 100 --progress

# Bot types: random, greedy, mcts, agent-argentum, agent-symbiote, agent-obsidion, agent-generalist

# Agent specialists with decks
cargo run --release -p cardgame --bin arena -- \
  --bot1 agent-argentum --deck1 colossus_wall \
  --bot2 agent-symbiote --deck2 alpha_frenzy \
  --games 50 --progress

# MCTS with custom simulations
cargo run --release -p cardgame --bin arena -- \
  --bot1 mcts --bot2 greedy \
  --mcts-sims 500 --mcts-trees 4 \
  --games 100 --progress

# Debug mode with logging
cargo run --release -p cardgame --bin arena -- \
  --bot1 mcts --bot2 greedy --games 10 \
  --debug --log-file match.log

# Tracing (forces sequential execution)
cargo run --release -p cardgame --bin arena -- \
  --bot1 greedy --bot2 greedy --games 5 \
  --trace-combat --trace-effects  # or --trace-all

# Essence Duel mode (experimental)
cargo run --release -p cardgame --bin arena -- \
  --bot1 mcts --bot2 greedy --mode essence-duel --games 100

# List available decks
cargo run --release -p cardgame --bin arena -- --list-decks
```

## Tuning (CMA-ES Weight Optimization)
```bash
# Generalist (default - vs multiple opponents)
cargo run --release -p cardgame --bin tune -- \
  --mode generalist --tag generalist_v1 --generations 100

# Faction specialist
cargo run --release -p cardgame --bin tune -- \
  --mode faction-specialist --faction argentum \
  --tag argentum_v1 --generations 100

# Specialist for specific matchup
cargo run --release -p cardgame --bin tune -- \
  --mode specialist \
  --deck colossus_wall --opponent alpha_frenzy \
  --tag matchup_test --generations 50

# Resume from existing weights
cargo run --release -p cardgame --bin tune -- \
  --mode generalist \
  --initial-weights data/weights/generalist.toml \
  --target-win-rate 0.95 --tag refinement
```

**Outputs:** `experiments/mcts/{YYYY-MM-DD_HHMM}_{tag}/`
- `train.log`, `weights.toml`, `version.toml`, `summary.txt`, `plots/`

## Validation (Balance Testing)
```bash
# Quick balance check (8k total games: 40 matchups × 2 directions × 100)
cargo run --release -p cardgame --bin validate -- --games 100

# Full validation (40k total games)
cargo run --release -p cardgame --bin validate -- --games 500

# Comprehensive (120k total games)
cargo run --release -p cardgame --bin validate -- --games 1500

# Export to JSON
cargo run --release -p cardgame --bin validate -- --games 500 --output results.json

# Specific matchup only
cargo run --release -p cardgame --bin validate -- \
  --matchup argentum-symbiote --games 200

# Interactive mode with progress
cargo run --release -p cardgame --bin validate -- --games 500 --interactive
```

## Diagnostics (P1/P2 Analysis)
```bash
# Basic analysis (200 games)
cargo run --release -p cardgame --bin diagnose -- 200

# Export to CSV
cargo run --release -p cardgame --bin diagnose -- 500 --export csv --output ./diagnostics

# Export to JSON with turn data
cargo run --release -p cardgame --bin diagnose -- 500 --export json --include-turns --output ./results.json

# Custom deck with progress
cargo run --release -p cardgame --bin diagnose -- 1000 --deck colossus_wall --progress
```

## Dataset Generation (ML Training Data)
```bash
# Self-play dataset (100k games, compressed)
cargo run --release -p cardgame --bin generate-dataset -- \
  --games 100000 --sims 100 --output data/datasets/mcts_100k.jsonl.gz

# Round-robin (all deck matchups)
cargo run --release -p cardgame --bin generate-dataset -- \
  --games 10000 --mode round-robin --sims 150 --output balanced.jsonl.gz

# With custom weights
cargo run --release -p cardgame --bin generate-dataset -- \
  --games 50000 --weights data/weights/generalist.toml --output tuned.jsonl.gz
```

## MCTS Profiling
```bash
cargo run --release -p cardgame --bin profile_mcts
# Shows: fork speed, rollout time, tree search overhead
```

## Modal Cloud Training
```bash
# Full pipeline (train all 4 configs + validate + auto-deploy)
modal run modal_tune.py

# Train only (skip validation)
modal run modal_tune.py --mode train-only

# Validate only (use existing weights)
modal run modal_tune.py --mode validate-only

# Single configuration
modal run modal_tune.py --single generalist
modal run modal_tune.py --single argentum

# Custom validation
modal run modal_tune.py --validation-games 1000 --cores 64

# Skip auto-deploy to local repo
modal run modal_tune.py --no-deploy

# Sequential validation (slower, fewer resources)
modal run modal_tune.py --sequential

# Check status
modal app list
modal app logs essence-wars-tuning
```

**Cloud outputs:** Auto-synced to `data/weights/` after training

## Analysis Tools

### MCTS Training Analysis
```bash
./scripts/analyze-mcts.sh                    # All experiments
./scripts/analyze-mcts.sh --tag generalist   # Filter by tag
./scripts/analyze-mcts.sh --min-gens 50      # Minimum generations
./scripts/analyze-mcts.sh --list-experiments # List available
```

### Validation Analysis
```bash
./scripts/analyze-validation.sh              # Latest results
./scripts/analyze-validation.sh path/to/results.json
```

### Dashboard Generation
```bash
./scripts/generate-dashboard.sh              # Balance dashboard
./scripts/generate-all-dashboards.sh         # All 3 dashboards
```

## Python Tools (uv)

### Training
```bash
# PPO agent
uv run python python/scripts/train_ppo.py --timesteps 500000 --tensorboard

# AlphaZero
uv run python python/scripts/train_alphazero.py --iterations 100

# Behavioral cloning
uv run python python/scripts/train_behavioral_cloning.py \
  --dataset data/datasets/mcts_100k.jsonl.gz --epochs 50
```

### Analysis & Diagnostics
```bash
uv run python python/scripts/mcts_analysis.py --tag generalist
uv run python python/scripts/diagnose_ppo.py   # PPO infrastructure check
uv run python python/scripts/benchmark_env.py  # Environment throughput

# Watch TensorBoard with Trainin Metrics
uv run tensorboard --logdir experiments/alphazero/20260120_063304/tensorboard
```

### Hub (Huggingface)
```bash
uv run python python/scripts/upload_model.py \
  --checkpoint model.pt --repo user/essence-wars-model --type ppo

uv run python python/scripts/upload_dataset.py \
  --dataset data.jsonl.gz --repo user/essence-wars-data
```

### Quality
```bash
uv sync --all-groups              # Install dependencies
uv run pytest python/tests -v     # Run tests
uv run ruff check python/ --fix   # Lint + autofix
uv run mypy python/               # Type check
```

## Key Directories

```
ai-cardgame/
├── crates/
│   ├── cardgame/               # Core engine (Rust)
│   └── essence-wars-3d/        # Bevy 3D client
├── data/
│   ├── cards/core_set/         # 140 cards (4 YAML files by faction)
│   ├── decks/                  # 12 decks (4 per faction)
│   │   ├── argentum/           # colossus_wall, vex_piercing, ...
│   │   ├── symbiote/           # alpha_frenzy, broodmother_swarm, ...
│   │   └── obsidion/           # kael_assassin, shadow_weaver, ...
│   ├── weights/
│   │   ├── generalist.toml     # Cross-faction weights
│   │   ├── specialists/        # Faction-specific weights
│   │   └── neural/             # PPO models (.pt files)
│   └── datasets/               # MCTS training data (.jsonl.gz)
├── experiments/                # Training outputs (gitignored)
│   ├── mcts/                   # Weight tuning runs
│   ├── validation/             # Balance validation
│   ├── ppo/                    # PPO training
│   └── alphazero/              # AlphaZero training
├── python/
│   ├── essence_wars/           # Python package
│   └── scripts/                # Training & analysis scripts
└── scripts/                    # Shell helpers
```

## Common Flags

### Bots & Agents
- `random`, `greedy`, `mcts` - Basic bot types
- `agent-generalist` - MCTS with generalist weights
- `agent-argentum`, `agent-symbiote`, `agent-obsidion` - Faction specialists

### MCTS Configuration
- `--mcts-sims N` - Simulations per move (default: 500)
- `--mcts-trees N` - Parallel trees (root parallelization)
- `--mcts-rollouts N` - Parallel rollouts per leaf

### General
- `--games N` - Number of games
- `--seed N` - Random seed (reproducibility)
- `--progress` - Show progress bar
- `--debug` - Verbose logging
- `--output PATH` - Export results

### Tuning
- `--mode {generalist|specialist|faction-specialist}`
- `--faction {argentum|symbiote|obsidion}` - For faction-specialist
- `--generations N` - CMA-ES iterations (default: 100)
- `--population N` - Population size (default: auto)
- `--tag NAME` - Experiment identifier

## Tips

1. **Always use `--release`** - Debug builds are ~10x slower
2. **Check decks first** - `arena --list-decks` shows available IDs
3. **Workspace builds** - Use `-p cardgame` for most commands
4. **Parallel speedup** - Tune/validate use all cores by default
5. **Modal for big runs** - 16-32 cores, faster than local
6. **Auto-deploy** - Modal training auto-copies weights to `data/weights/`

## Troubleshooting

```bash
# Workspace issues
cargo build --workspace                # Build all crates
cargo clean && cargo build --release   # Clear artifacts

# Card database not found
export CARDS_DIR=data/cards/core_set

# Python env issues
uv sync --all-groups                   # Reset dependencies

# Test failures
cargo nextest run -p cardgame --no-fail-fast  # See all failures

# Modal setup
uv tool install modal && modal token new      # One-time setup
```

---
*Last updated: 2026-01-19*
