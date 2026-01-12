# Data & Experiment Strategy

> **Goal:** Organize training data, experiments, and analysis into a scalable "treasure trove" for future research.

## 1. Directory Structure

We strictly separate **Code**, **Experiments**, and **Raw Data**.

```text
/home/chris/ai-cardgame/
├── experiments/                 # ALL Run artifacts (Config, Logs, Checkpoints)
│   ├── mcts/                    # MCTS Parameter Tuning runs
│   │   └── YYYY-MM-DD_HHMM_tag/ # Single Experiment ID
│   │       ├── config.yaml      # EXACT config used (Reproducibility)
│   │       ├── stats.csv        # Metrics (Fitness, WinRate, Loss)
│   │       ├── weights.toml     # Best found weights
│   │       └── plots/           # Generated graphs
│   ├── ppo/                     # PPO Training runs
│   │   └── ...
│   └── alphazero/               # AlphaZero Training runs
│       └── ...
├── data/                        # Large/Binary Data (GitIgnored)
│   ├── replays/                 # Raw game replays (for offline RL)
│   └── datasets/                # Pre-tokenized/Tensor datasets
├── python/
│   ├── cardgame/
│   │   ├── infra/               # Experiment Management & Logging
│   │   ├── analysis/            # Visualization & Stats tools
│   │   ├── env/                 # Gym/PettingZoo Envs
│   │   └── algo/                # Algorithm implementations
│   └── scripts/                 # CLI entry points (calling cardgame package)
└── docs/                        # Documentation
```

## 2. Experiment ID Convention

Every run gets a unique ID: `{timestamp}_{tag}`
*   **Timestamp:** `YYYY-MM-DD_HH-MM-SS` (UTC or Local)
*   **Tag:** Short descriptive string (e.g., `baseline`, `lr_sweep_01`, `new_net_arch`)

**Example:** `2026-01-12_14-30-00_mcts_baseline`

## 3. The `Experiment` Class

All Python training scripts must use the `Experiment` class to initialize paths.

```python
from cardgame.infra import Experiment

exp = Experiment(name="mcts_tuning", tag="aggressive_test")

# Automatically creates: experiments/mcts/2026-01-12_..._aggressive_test/
# Saves config.yaml immediately.

exp.log_metric("win_rate", 0.55, step=1)
exp.save_checkpoint(model, "gen_10.pt")
```

## 4. Git Hygiene

*   **`experiments/`**: **IGNORED** by default. We do NOT commit heavy checkpoints or logs.
*   **`data/`**: **IGNORED**.
*   **`docs/experiments/`**: (Optional) Curated reports/summaries of successful experiments can be committed here.

## 5. Workflow

1.  **Launch:** Run training script (`python -m cardgame.scripts.train_ppo --tag my_run`).
2.  **Monitor:** Logs go to `experiments/ppo/<id>/train.log`.
3.  **Analyze:** Run analysis script pointing to the experiment folder.
    *   `python -m cardgame.analysis.plot_training experiments/ppo/<id>`
4.  **Archive:** If a run is important, back it up to cloud storage or keep locally. Delete failed runs to save space.
