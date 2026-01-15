# Chris Cheat Sheet (CCS) 😅
*Quick reference for Essence Wars scripts & tools*

## 🏃 Quick Commands

### Build & Test
```bash
cargo build --release              # Production build (14s)
cargo test                         # All 243 tests (~3s)
cargo nextest run                  # Parallel test runner
cargo bench                        # Criterion benchmarks
./scripts/run-clippy.sh            # Lint (excludes tests)
./scripts/run-tests.sh             # Test wrapper
./scripts/run-benchmarks.sh        # Benchmark wrapper
```

### Arena (Bot Battles)
```bash
# Quick match
cargo run --release --bin arena -- --bot1 greedy --bot2 random --games 100 --progress

# Custom decks + weights
cargo run --release --bin arena -- \
  --deck1 symbiote_aggro --deck2 argentum_control \
  --bot1 mcts --weights1 data/weights/tuned_multi_opponent.toml \
  --games 50 --debug

# See available decks
cargo run --release --bin arena -- --list-decks
```

### Tuning (Optimize Bot Weights)
```bash
# Generalist (vs multiple opponents)
cargo run --release --bin tune -- --tag generalist --mode multi-opponent --generations 100

# Specialist (for specific matchup)
cargo run --release --bin tune -- --tag symbiote_spec \
  --mode specialist --deck symbiote_aggro --opponent argentum_control --generations 50

# Quick test run
cargo run --release --bin tune -- --tag test --generations 5 --games 50
```

**Outputs:** `experiments/mcts/{YYYY-MM-DD_HHMM}_{tag}/`
- `config.yaml` - Run configuration
- `stats.csv` - Per-generation metrics
- `best_weights.toml` - Final weights
- `plots/` - Fitness curves & stats

### Validation (Balance Testing)
```bash
# Full validation (all factions, both player orders)
cargo run --release --bin validate -- --games 500 --output results.json

# Interactive mode (progress spinner)
cargo run --release --bin validate -- --games 100 --interactive

# Quick check
cargo run --release --bin validate -- --games 50
```

### Diagnostics (P1/P2 Analysis)
```bash
# Basic diagnostic run
cargo run --release --bin diagnose 500

# Export to CSV
cargo run --release --bin diagnose 500 --export csv --output ./diagnostics

# Export to JSON with turn data
cargo run --release --bin diagnose 500 --export json --include-turns --output ./results.json

# Custom deck
cargo run --release --bin diagnose 1000 --deck argentum_control --progress
```

### MCTS Profiling
```bash
# Profile MCTS performance
cargo run --release --bin profile_mcts

# Results show:
# - engine.fork() speed
# - Tree search overhead
# - Simulation rates
```

## ☁️ Modal Cloud (Remote Training)
```bash
# Full pipeline (train + validate)
modal run modal_tune.py::main

# Train only
modal run modal_tune.py::main --mode train-only

# Validate with existing weights (round-robin: 40 matchups × 2 directions × games)
modal run modal_tune.py::main --mode validate-only --validation-games 1500 --validation-timeout 7200  # 120k total games

# Single training config
modal run modal_tune.py::main --single generalist

# Deploy as persistent app
modal deploy modal_tune.py::main

# Check status
modal app list
modal app logs essence-wars-tuning
```

**Cloud outputs:** `data/remote_outputs/{job_id}/` (auto-synced)

## 📊 Analysis Tools

### MCTS Analysis
```bash
# Analyze latest run
./scripts/analyze-mcts.sh --latest

# Specific experiment
./scripts/analyze-mcts.sh experiments/mcts/2026-01-14_1205_generalist-v0.4

# Compare multiple runs
./scripts/analyze-mcts.sh \
  experiments/mcts/2026-01-14_1205_generalist-v0.4 \
  experiments/mcts/2026-01-14_1106_symbiote-specialist-v0.4
```

**Generates:**
- `fitness_evolution.png` - Fitness over generations
- `weight_distributions.png` - Weight value ranges
- `convergence_stats.txt` - Summary metrics
- Interactive HTML dashboard with Plotly charts

### Performance Stats
```bash
# Extract benchmark results
./scripts/perf-stats.sh

# Requires:
# - benchmark_results.txt (from cargo bench)
# - profiling_results.txt (from profile_mcts)
```

## 🐍 Python Tools

### Direct Python Usage
```bash
# MCTS analysis (via uv)
uv run python python/scripts/mcts_analysis.py --experiment experiments/mcts/{run_dir}/ --output ./plots

# Run tests
uv run pytest python/tests -v

# Lint
uv run ruff check python/
```

## 📁 Key Directories

```
data/
  cards/core_set/         # Card definitions (YAML)
  decks/                  # Deck lists (TOML)
  weights/                # Bot weights (TOML)
    default.toml          # Baseline weights
    generalist.toml       # Tuned multi-opponent
    specialists/          # Faction-specific weights

experiments/              # All training outputs (gitignored)
  mcts/{YYYY-MM-DD_HHMM}_{tag}/
  ppo/
  alphazero/
  validation/

docs/experiments/         # Curated reports (git-tracked)
```

## 🔧 Common Flags

### Arena/Tune/Validate
- `--games N` - Games to run
- `--seed N` - Random seed (reproducibility)
- `--progress` - Show progress bar
- `--debug` - Verbose logging
- `--output PATH` - Export results

### MCTS Config
- `--mcts-sims N` - Simulations per move (default: 100)
- `--weights PATH` - Custom weight file
- `--deck ID` - Specific deck

### Tuning
- `--generations N` - CMA-ES iterations (default: 50)
- `--population N` - Population size (default: 20)
- `--mode {multi-opponent|specialist}` - Training strategy
- `--tag NAME` - Experiment identifier

## 💡 Tips

1. **Always use `--release`** - Debug builds are ~10x slower
2. **Check decks first** - `arena --list-decks` shows available IDs
3. **Experiments auto-organized** - Timestamped folders in `experiments/`
4. **Parallel speedup** - Tune/validate use all cores by default
5. **Modal for big runs** - 16-32 cores, faster than local
6. **Latest weights** - `data/weights/generalist.toml` is current best

## 🚨 Quick Troubleshooting

```bash
# Card database not found
export CARDS_DIR=data/cards/core_set  # Or pass --cards flag

# Python env issues
uv sync --all-groups                  # Reset dependencies

# Stale build artifacts
cargo clean && cargo build --release

# Test failures
cargo nextest run --no-fail-fast      # See all failures
```

---
*Last updated: 2026-01-15*
