# Essence Wars CLI Guide

*Complete reference for the `essence-wars` command-line interface*

**Last Updated:** 2026-02-01

---

## Overview

The `essence-wars` CLI provides a unified interface for training ML agents, running benchmarks, generating reports, and managing datasets. All Python analysis tools are accessible through this single command.

## Installation

```bash
# Install the package in development mode
cd /path/to/ai-cardgame
uv sync --all-groups

# Verify installation
essence-wars --help
```

## Command Structure

```
essence-wars [COMMAND] [SUBCOMMAND] [OPTIONS]
```

**Available Commands:**
- `train` - Train ML agents (PPO, AlphaZero, BC, etc.)
- `benchmark` - Evaluate agents against baselines
- `report` - Generate HTML reports and leaderboards
- `data` - Generate training datasets

---

## Training Commands

### `essence-wars train ppo`

Train a Proximal Policy Optimization (PPO) agent.

**Usage:**
```bash
essence-wars train ppo [OPTIONS]
```

**Options:**
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--tag` | str | (required) | Experiment identifier |
| `--timesteps` | int | 300000 | Total training timesteps |
| `--games` | int | 1000 | Number of self-play games |
| `--lr` | float | 3e-4 | Learning rate |
| `--batch-size` | int | 256 | Mini-batch size |
| `--gamma` | float | 0.99 | Discount factor |
| `--auto-callbacks` | flag | False | Enable all auto-callbacks |
| `--auto-evaluate` | flag | False | Auto-evaluate after training |
| `--auto-report` | flag | False | Auto-generate report |
| `--update-elo` | flag | False | Update ELO ratings after evaluation |

**Examples:**
```bash
# Basic training
essence-wars train ppo --tag my_experiment --timesteps 100000

# Full automated pipeline (train → evaluate → report → ELO)
essence-wars train ppo \
  --tag production_v1 \
  --timesteps 500000 \
  --auto-callbacks \
  --update-elo

# Custom hyperparameters
essence-wars train ppo \
  --tag custom_hp \
  --lr 1e-4 \
  --batch-size 512 \
  --gamma 0.95
```

**Output:**
- Checkpoints: `experiments/ppo/{timestamp}_{tag}/checkpoints/`
- Logs: `experiments/ppo/{timestamp}_{tag}/logs/`
- Report: `experiments/reports/{timestamp}_{tag}/index.html` (if `--auto-report`)

---

### `essence-wars train alphazero`

Train an AlphaZero agent with neural MCTS.

**Usage:**
```bash
essence-wars train alphazero [OPTIONS]
```

**Options:**
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--tag` | str | (required) | Experiment identifier |
| `--iterations` | int | 100 | Self-play iterations |
| `--games-per-iter` | int | 100 | Games per iteration |
| `--mcts-sims` | int | 50 | MCTS simulations per move |
| `--train-epochs` | int | 10 | Training epochs per iteration |
| `--auto-callbacks` | flag | False | Enable all auto-callbacks |

**Example:**
```bash
essence-wars train alphazero \
  --tag az_baseline \
  --iterations 200 \
  --mcts-sims 100 \
  --auto-callbacks
```

---

### `essence-wars train bc`

Train a Behavioral Cloning agent from expert demonstrations.

**Usage:**
```bash
essence-wars train bc [OPTIONS]
```

**Options:**
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--tag` | str | (required) | Experiment identifier |
| `--dataset` | path | (required) | Path to MCTS dataset |
| `--epochs` | int | 20 | Training epochs |
| `--batch-size` | int | 256 | Batch size |
| `--lr` | float | 1e-3 | Learning rate |
| `--auto-callbacks` | flag | False | Enable all auto-callbacks |

**Example:**
```bash
essence-wars train bc \
  --tag bc_expert \
  --dataset data/datasets/mcts_10k.npz \
  --epochs 50 \
  --auto-evaluate
```

---

### Other Training Commands

**Card2Vec Embeddings:**
```bash
essence-wars train card2vec --tag embeddings_v1 --epochs 100
```

**Decision Transformer:**
```bash
essence-wars train decision-transformer \
  --tag dt_baseline \
  --dataset data/datasets/mcts_trajectories.npz \
  --epochs 30
```

**Distilled Policy:**
```bash
essence-wars train distilled-policy \
  --tag distilled_mcts50 \
  --teacher-sims 50 \
  --games 5000
```

---

## Benchmarking Commands

### `essence-wars benchmark`

Evaluate an agent against standard baselines.

**Usage:**
```bash
essence-wars benchmark [OPTIONS]
```

**Options:**
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--checkpoint` | path | (required) | Path to model checkpoint |
| `--opponents` | list | random,greedy | Comma-separated opponent list |
| `--games` | int | 100 | Games per opponent |
| `--update-elo` | flag | False | Update agent ELO ratings |
| `--output` | path | None | Save results to JSON file |

**Available Opponents:**
- `random` - Uniform random baseline
- `greedy` - Greedy heuristic bot
- `mcts-25`, `mcts-50`, `mcts-100` - MCTS with N simulations

**Example:**
```bash
essence-wars benchmark \
  --checkpoint models/bc_mcts_10k_best.pt \
  --opponents random,greedy,mcts-50,mcts-100 \
  --games 200 \
  --update-elo \
  --output benchmark_results.json
```

**Output:**
```
Benchmark Results
─────────────────────────────────────────
Agent: bc_mcts_10k_best
Opponent          Win Rate    Avg Turns
────────────────────────────────────────
random            98.5%       8.2
greedy            67.0%       12.4
mcts-50           52.5%       15.1
mcts-100          43.0%       16.8
────────────────────────────────────────
Overall ELO: 1654 (+28)
```

---

## Reporting Commands

### `essence-wars report generate`

Generate an HTML report for a validation run or experiment.

**Usage:**
```bash
essence-wars report generate [OPTIONS]
```

**Options:**
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--run-id` | str | latest | Run identifier or "latest" |
| `--tabs` | list | all | Comma-separated tab list |
| `--open` | flag | False | Open report in browser |
| `--output` | path | Auto | Custom output directory |

**Available Tabs:**
- `overview` - Summary metrics and health score
- `validation` - Deck performance and matchup heatmaps
- `tuning` - CMA-ES training curves
- `research` - Faction-level analysis
- `elo` - Rating rankings and history
- `benchmark` - Agent evaluation results

**Examples:**
```bash
# Generate full report for latest run
essence-wars report generate --run-id latest --open

# Generate specific tabs only
essence-wars report generate \
  --run-id 2026-01-31_1425_baseline \
  --tabs overview,elo,benchmark \
  --open

# Custom output location
essence-wars report generate \
  --run-id production_v1 \
  --output reports/production/ \
  --open
```

**Output Location:** `experiments/reports/{run_id}/index.html`

---

### `essence-wars report generate-all`

Generate reports for all validation runs.

**Usage:**
```bash
essence-wars report generate-all [OPTIONS]
```

**Options:**
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--experiments-dir` | path | experiments/ | Base experiments directory |
| `--force` | flag | False | Regenerate existing reports |

**Example:**
```bash
essence-wars report generate-all --force
```

---

### `essence-wars report aggregate`

Aggregate metrics from multiple experiments.

**Usage:**
```bash
essence-wars report aggregate [OPTIONS]
```

**Options:**
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--experiments-dir` | path | experiments/ | Base experiments directory |
| `--output` | path | aggregated.json | Output JSON file |

**Example:**
```bash
essence-wars report aggregate \
  --experiments-dir experiments/ppo/ \
  --output ppo_summary.json
```

---

### `essence-wars report leaderboard`

Display or export the unified ELO leaderboard.

**Usage:**
```bash
essence-wars report leaderboard [OPTIONS]
```

**Options:**
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--format` | choice | text | Output format: text, json, html |
| `--category` | choice | all | Filter by: all, deck, agent |
| `--limit` | int | 20 | Number of entries to show |
| `--output` | path | None | Save to file (for json/html) |

**Examples:**
```bash
# Display top 10 in terminal
essence-wars report leaderboard --limit 10

# Export to HTML
essence-wars report leaderboard \
  --format html \
  --output leaderboard.html

# Show only agent ratings
essence-wars report leaderboard --category agent --limit 5
```

**Output (text format):**
```
═══════════════════════════════════════════════════════════
                  ESSENCE WARS LEADERBOARD
═══════════════════════════════════════════════════════════
Rank  Name                                   Rating  Category
───────────────────────────────────────────────────────────
  1   Obsidion: Void Queen (Ruin)             1687   DECK
  2   bc_mcts_10k_best                        1654   AGENT
  3   Argentum: Celestial Vanguard            1642   DECK
  4   alphazero_iter200                       1628   AGENT
  5   Symbiote: Broodmother's Swarm           1615   DECK
...
```

---

## Data Generation Commands

### `essence-wars data generate`

Generate training datasets for ML agents.

**Usage:**
```bash
essence-wars data generate [OPTIONS]
```

**Options:**
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--type` | choice | (required) | Dataset type: distillation, exits |
| `--games` | int | 1000 | Number of games to generate |
| `--output` | path | Auto | Output file path |
| `--mcts-sims` | int | 50 | MCTS simulations (distillation only) |

**Examples:**
```bash
# Generate distillation dataset from MCTS play
essence-wars data generate \
  --type distillation \
  --games 5000 \
  --mcts-sims 100 \
  --output data/datasets/mcts_expert_5k.npz

# Generate exit position dataset
essence-wars data generate \
  --type exits \
  --games 2000 \
  --output data/datasets/exits_2k.npz
```

---

## Configuration Files

### Callback Configuration

Callbacks can be configured via TOML file:

**`configs/callbacks.toml`:**
```toml
[checkpoints]
enabled = true
save_freq = 10000
keep_last = 5

[evaluation]
enabled = true
eval_freq = 50000
eval_games = 100
opponents = ["random", "greedy", "mcts-50"]

[auto_evaluate]
enabled = true
update_elo = true
games = 200

[auto_report]
enabled = true
open_browser = false
```

**Usage:**
```bash
essence-wars train ppo --tag exp1 --config configs/callbacks.toml
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ESSENCE_WARS_DATA_DIR` | `data/` | Base data directory |
| `ESSENCE_WARS_EXPERIMENTS_DIR` | `experiments/` | Experiments output directory |
| `ESSENCE_WARS_MODELS_DIR` | `models/` | Model checkpoints directory |
| `ESSENCE_WARS_LOG_LEVEL` | `INFO` | Logging level |

---

## Tips & Best Practices

### 1. Use Tags Consistently
```bash
# Good: Descriptive, dated tags
essence-wars train ppo --tag 2026-02-01_baseline_v1

# Bad: Generic tags
essence-wars train ppo --tag test
```

### 2. Enable Auto-Callbacks for Long Runs
```bash
# Automatically evaluate and report when training completes
essence-wars train ppo \
  --tag overnight_run \
  --timesteps 2000000 \
  --auto-callbacks
```

### 3. Track Progress with Reports
```bash
# Check latest training status
essence-wars report generate --run-id latest --open
```

### 4. Update ELO for Comparisons
```bash
# Always update ELO after benchmarking
essence-wars benchmark \
  --checkpoint new_model.pt \
  --update-elo
```

### 5. Use Specific Run IDs for Production
```bash
# Don't rely on "latest" for important reports
essence-wars report generate --run-id 2026-02-01_1430_prod_v1
```

---

## Troubleshooting

### Command Not Found

```bash
# Ensure package is installed
uv sync --all-groups

# Or reinstall
pip install -e .
```

### Import Errors

```bash
# Check Python environment
which python
python --version

# Verify essence_wars is importable
python -c "import essence_wars; print(essence_wars.__version__)"
```

### Missing Rust Binaries

The CLI requires Rust binaries for game simulation:

```bash
# Build Rust components
cargo build --release

# Verify binaries exist
ls -la target/release/
```

---

## See Also

- [Ratings System](ratings-system.md) - ELO tracking and leaderboards
- [Reporting](reporting.md) - HTML report generation details
- [Training Pipeline](training-pipeline.md) - Callback system documentation
- [Bots & Tuning Pipeline](bots-tuning-pipeline.md) - Rust-side bot configuration

---

*For issues or feature requests, see the project's GitHub repository.*
