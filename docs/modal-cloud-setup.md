# Modal Cloud Training Setup

**Status: Implemented**

Complete guide to running Essence Wars tuning on Modal's serverless compute platform.

## 📋 Prerequisites

- Python 3.11+
- Modal account (sign up at https://modal.com)
- Local workspace with essence-wars repository

## 🚀 Quick Start (5 Minutes)

### 1. Install Modal CLI

```bash
# Install Modal as a global tool with uv (recommended)
uv tool install modal

# Or with pip (if you prefer)
pip install modal
```

### 2. Authenticate

```bash
# Creates API token and saves to ~/.modal.toml
modal token new
```

Follow the browser prompts to authenticate with GitHub or Google.

### 3. Run Training

```bash
# Full pipeline: train all 4 configs + run balance validation
modal run modal_tune.py::main

# Train only (skip validation)
modal run modal_tune.py::main --mode train-only

# Validate only (use existing trained weights) - PARALLEL by default
modal run modal_tune.py::main --mode validate-only

# Validate with custom game counts (round-robin: 40 matchups × 2 directions × games)
modal run modal_tune.py::main --mode validate-only --validation-games 1500  # 120k total
modal run modal_tune.py::main --mode validate-only --validation-games 500   # 40k total (default)
modal run modal_tune.py::main --mode validate-only --validation-games 250   # 20k total - quick

# Use sequential validation (original single-container mode)
modal run modal_tune.py::main --mode validate-only --sequential

# Run single configuration
modal run modal_tune.py::main --single generalist
modal run modal_tune.py::main --single argentum
modal run modal_tune.py::main --single symbiote
modal run modal_tune.py::main --single obsidion
```

### 4. Auto-Deploy Weights

After a successful training run, weights are **automatically deployed** to your local repository:

```
data/weights/
├── generalist.toml              ← Updated automatically
└── specialists/
    ├── argentum.toml            ← Updated automatically
    ├── symbiote.toml            ← Updated automatically
    └── obsidion.toml            ← Updated automatically
```

To skip auto-deploy (e.g., for testing):

```bash
modal run modal_tune.py::main --no-deploy
```

**Note:** Validation-only mode doesn't deploy weights (it only reads existing weights for testing).

**⚡ Round-Robin Game Count Guide:**

With round-robin validation (40 deck matchups × 2 directions), the total game count scales significantly:

| `--validation-games` | Total Games | Use Case | Runtime* |
|---------------------|-------------|----------|----------|
| 100 | 8,000 | Quick smoke test | ~1 min |
| 250 | 20,000 | Fast balance check | ~2 min |
| 500 (default) | 40,000 | Standard validation | ~3 min |
| 1000 | 80,000 | Thorough analysis | ~6 min |
| 1500 | 120,000 | High-confidence metrics | ~10 min |
| 2000 | 160,000 | Research-grade data | ~13 min |

*Runtime with 32-core parallel validation

### 5. Download Experiment Logs (Optional)

For detailed analysis of training runs:

```bash
# List available experiments
modal run modal_tune.py::list_experiments

# Download latest results
modal run modal_tune.py::download_latest

# Extract
tar -xzf experiments_modal_*.tar.gz
```

---

## 🏗️ How It Works

### Architecture

```
Local Machine                 Modal Cloud
┌─────────────┐
│             │              Phase 1: Training (4x Parallel)
│  Workspace  │              ┌──────────────────────────────────┐
│  Snapshot   │─────────────▶│  Instance 1-4: 16 cores each     │
│  (~5-10 MB) │              │  → Generalist, Argentum,         │
└─────────────┘              │    Symbiote, Obsidion            │
                             │  → Build + Train + Deploy        │
                             └──────────────────────────────────┘
                                            │
                                            ▼
                             Phase 2: Validation (3x Parallel)
                             ┌──────────────────────────────────┐
                             │  Instance 5-7: 32 cores each     │
                             │  → Argentum vs Symbiote          │
                             │  → Argentum vs Obsidion          │
                             │  → Symbiote vs Obsidion          │
                             │  → ~3x faster than sequential!   │
                             └──────────────────────────────────┘

                             Modal Persistent Volume (Shared)
                             ┌──────────────────────────────────┐
                             │ experiments/                     │
                             │   2026-01-14_HHMM_gen-v0.4/     │
                             │   2026-01-14_HHMM_arg-v0.4/     │
                             │   ...                            │
                             │ weights/                         │
                             │   generalist.toml                │
                             │   specialists/                   │
                             │ validation_2026-01-14_HHMM.json  │
                             └──────────────────────────────────┘
```

### Execution Flow

1. **Local: Create Snapshot**
   - Tar workspace directory (exclude target/, experiments/, .git/)
   - Upload to Modal (~5-10 MB, <1s)

2. **Cloud: Phase 1 - Spawn 4 Parallel Training Instances**
   - Each gets 16 dedicated CPU cores + 8 GB RAM
   - Extract workspace snapshot
   - Build Rust binary (`cargo build --release`)
   - Run tuning command (`cargo run --release --bin tune -- ...`)

3. **Cloud: Training (10-15 min per job)**
   - 100 generations × 100 games vs Random/Greedy/MCTS
   - CMA-ES optimization with Rayon parallelism (16 cores)
   - Auto-deploy weights to data/weights/

4. **Cloud: Phase 2 - Balance Validation (~3 min default, scales with game count)**
   - **PARALLEL mode (default):** Spawn 3 containers, one per matchup
     - Each matchup runs on dedicated 32-core instance
     - ~3x faster than sequential for large game counts
     - Results merged automatically
   - **Sequential mode (--sequential):** Single 32-core instance
     - Uses trained weights from Modal volume (or local if none exist)
   - **Round-robin testing:** ALL deck combinations (40 total) across 3 faction pairs
   - 500 games per matchup per direction by default = 40,000 TOTAL games
   - Configurable via --validation-games (e.g., 1500 = 120k total games)
   - Output JSON results with balance summary

5. **Cloud: Save Results**
   - Copy experiments/ to Modal persistent volume
   - Copy weights/ to volume
   - Save validation_results.json
   - Commit volume changes

6. **Local: Auto-Deploy Weights**
   - Download trained weights from Modal volume
   - Deploy to `data/weights/generalist.toml`
   - Deploy specialists to `data/weights/specialists/*.toml`
   - Ready for immediate use (no manual copying!)

7. **Local: Auto-Save Validation Results**
   - Validation results automatically saved to `experiments/validation/{run_id}/`
   - Includes `results.json`, `summary.txt`, and `config.toml`
   - No manual download required!

8. **Local: Download Training Logs (Optional)**
   - Retrieve full experiment logs as tarball
   - Extract to local experiments/ directory
   - Analyze with `./scripts/analyze-tuning.sh`

---

## 💰 Cost Breakdown

### Per-Job Costs

**Instance:** 16 vCPU, 8 GB RAM  
**Typical Runtime:** 12 minutes (build: 2 min, train: 10 min)  
**Cost:** ~$0.025/CPU-hour × 16 cores × 0.2 hours = **~$0.08 per job**

### Parallel Run (4 Jobs)

**Wall Time:** ~15 minutes (all 4 run simultaneously)  
**Total CPU Time:** 4 jobs × 12 min = 48 minutes  
**Total Cost:** ~$0.32 per full training run

### Validation Only (500 games/matchup = 40k total)

**Instance:** 32 vCPU (default for validation)  
**Typical Runtime:** ~3 minutes (build + validate 40k games)  
**Cost:** ~$0.025/CPU-hour × 32 cores × 0.05 hours = **~$0.04 per validation**

**Note:** With round-robin, `--validation-games 500` now runs 40,000 total games (40 deck matchups × 2 directions × 500).

### Monthly Estimate

- **10 full runs/month:** ~$3.20
- **20 full runs/month:** ~$6.40
- **50 full runs/month:** ~$16.00

**Modal Credits:** $10/month free tier included with subscription ($20/month Pro plan)

---

## ⚙️ Configuration Options

### Adjust CPU Cores

Edit `modal_tune.py`:

```python
# Faster training (more expensive)
CPU_COUNT = 32  # ~$0.16 per job, ~2x faster

# Cheaper (slower)
CPU_COUNT = 8   # ~$0.04 per job, ~2x slower
```

**Sweet Spot:** 16 cores provides excellent price/performance ratio.

### Adjust Training Parameters

Edit `TRAINING_CONFIGS` in `modal_tune.py`:

```python
{
    "tag": "generalist-v0.4-quick",
    "mode": "generalist",
    "args": [
        "--generations", "50",   # Half the iterations
        "--games", "50",         # Half the games
        "--mcts-sims", "50"
    ],
    "description": "Quick generalist test",
}
```

**Quick Test:** 50 gens, 50 games → ~5 min, ~$0.03 per job

### Persistent Volume

Modal automatically creates a persistent volume named `essence-wars-experiments` that stores all experiment outputs across runs.

**View in dashboard:** https://modal.com/storage

**Clear volume** (to save space):
```bash
modal volume delete essence-wars-experiments
```

---

## 🔧 Advanced Usage

### Deploy as Persistent App

Instead of running locally, deploy to Modal and trigger via webhook:

```bash
# Deploy app
modal deploy modal_tune.py

# Trigger training via HTTP
curl -X POST https://your-app-id.modal.run/train
```

### Parallel Hyperparameter Search

Modify `TRAINING_CONFIGS` to test multiple hyperparameter combinations:

```python
TRAINING_CONFIGS = [
    # Test different MCTS simulations
    {"tag": "gen-mcts25", "mode": "generalist", "args": ["--mcts-sims", "25"]},
    {"tag": "gen-mcts50", "mode": "generalist", "args": ["--mcts-sims", "50"]},
    {"tag": "gen-mcts100", "mode": "generalist", "args": ["--mcts-sims", "100"]},
    
    # Test different generation counts
    {"tag": "gen-50gen", "mode": "generalist", "args": ["--generations", "50"]},
    {"tag": "gen-100gen", "mode": "generalist", "args": ["--generations", "100"]},
    {"tag": "gen-200gen", "mode": "generalist", "args": ["--generations", "200"]},
]
```

Run all 6 configurations in parallel! (~$0.50, ~15 minutes)

### Custom Image with Pre-built Binary

Speed up execution by pre-building the Rust binary in the image:

```python
rust_image = (
    modal.Image.debian_slim()
    .apt_install("curl", "build-essential")
    .run_commands(
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
    )
    .env({"PATH": "/root/.cargo/bin:$PATH"})
    # Pre-clone and build
    .run_commands(
        "git clone https://github.com/christianWissmann85/essence-wars.git",
        "cd essence-wars && cargo build --release --bin tune",
    )
)
```

**Benefit:** Saves 2 minutes build time per job (but image build takes longer)

---

## 🐛 Troubleshooting

### "ModuleNotFoundError: No module named 'modal'"

**Solution:** Install Modal CLI
```bash
uv tool install modal
# or with pip
pip install modal
```

### "Authentication required"

**Solution:** Authenticate with Modal
```bash
modal token new
```

### "Build failed: linker error"

**Problem:** Missing build dependencies in image

**Solution:** Add required packages to `rust_image`:
```python
.apt_install("pkg-config", "libssl-dev")
```

### "Training timeout after 2 hours"

**Problem:** Training taking longer than expected

**Solution:** Increase timeout or reduce workload:
```python
TIMEOUT_SECONDS = 10800  # 3 hours
```

### "Persistent volume full"

**Problem:** Too many experiment results stored

**Solution:** Clear old experiments:
```bash
modal volume delete essence-wars-experiments
# Recreated automatically on next run
```

### Can't download results

**Problem:** No experiments in volume

**Check:**
```bash
modal run modal_tune.py::list_experiments
```

If empty, training didn't save results. Check logs in Modal dashboard.

---

## 📊 Performance Comparison

### Local vs Cloud

| Metric | Local (Ryzen AI 7 350) | Modal (16-core) | Modal (32-core) |
|--------|------------------------|-----------------|-----------------|
| **Single Job** | ~15 minutes | ~12 minutes | ~8 minutes |
| **4 Jobs Sequential** | ~60 minutes | ~48 minutes | ~32 minutes |
| **4 Jobs Parallel** | N/A | ~15 minutes ⚡ | ~10 minutes ⚡ |
| **Cost** | $0 (electricity) | ~$0.32 | ~$0.64 |

**Best Use Case:** Run 4+ jobs in parallel on Modal for **4x speedup** at low cost.

### Modal vs Other Cloud Providers

| Provider | 4 Jobs Parallel | Cost | Setup Time |
|----------|----------------|------|------------|
| **Modal** | ✅ 15 min | **$0.32** | 5 min |
| AWS EC2 (4x c7i.2xlarge) | ✅ 15 min | ~$0.48 | 30 min |
| Hetzner (4x CCX33) | ✅ 15 min | ~€0.20 | 45 min |
| DigitalOcean (4x droplets) | ✅ 15 min | ~$0.96 | 30 min |

**Winner:** Modal for parallel execution (simplest setup, competitive pricing)

---

## 🎯 Best Practices

1. **Test locally first** - Verify tuning works before running on Modal
2. **Use parallel execution** - Run all 4 configs simultaneously for 4x speedup
3. **Monitor dashboard** - Watch real-time logs at https://modal.com/apps
4. **Trust auto-deploy** - Weights are automatically deployed to `data/weights/` after training
5. **Use `--no-deploy` for experiments** - Skip auto-deploy when testing hyperparameters
6. **Version your configs** - Use descriptive tags (e.g., `generalist-v0.4-modal`)
7. **Scale cores wisely** - 16 cores is sweet spot for most workloads

---

## 🔗 Resources

- **Modal Docs:** https://modal.com/docs
- **Modal Pricing:** https://modal.com/pricing
- **Modal Dashboard:** https://modal.com/apps
- **Support:** https://modal.com/slack

---

## 📝 Quick Reference

```bash
# Install
uv tool install modal
modal token new

# Run full pipeline (train + validate + auto-deploy)
modal run modal_tune.py::main                         # All 4 + validation + deploy
modal run modal_tune.py::main --mode train-only       # Training only + deploy
modal run modal_tune.py::main --mode validate-only    # Validation only (reads existing weights)
modal run modal_tune.py::main --single generalist     # Single config + deploy
modal run modal_tune.py::main --no-deploy             # Skip auto-deploy to local repo

# Custom validation parameters (parallel by default - ~3x faster!)
# Round-robin: 40 deck matchups × 2 directions × games = total games
modal run modal_tune.py::main --mode validate-only --validation-games 1500   # 120k total, parallel
modal run modal_tune.py::main --mode validate-only --validation-games 500    # 40k total (default)
modal run modal_tune.py::main --mode validate-only --validation-games 250    # 20k total - quick
modal run modal_tune.py::main --mode validate-only --sequential              # Force sequential (1 container)
modal run modal_tune.py::main --mode validate-only --validation-timeout 7200 # 2hr timeout per matchup
modal run modal_tune.py::main --mode validate-only --validation-timeout 7200 # 2hr timeout per matchup

# Download experiment logs (optional)
modal run modal_tune.py::list_experiments
modal run modal_tune.py::download_latest

# Deploy as app
modal deploy modal_tune.py

# View logs
modal app logs essence-wars-tuning

# Check usage/billing
# https://modal.com/settings/billing

# Local validation (no Modal required - round-robin: 40 matchups × 2 directions)
cargo run --release --bin validate -- --games 100                             # 8k total - quick
cargo run --release --bin validate -- --games 500 --output results.json       # 40k total - full test
cargo run --release --bin validate -- --games 1500                            # 120k total - comprehensive
cargo run --release --bin validate -- --matchup argentum-symbiote --games 100 # Single faction pair
```

---

**🎉 You're ready to train at cloud scale!**

Questions? Check the Modal docs or ask in their Slack community.
