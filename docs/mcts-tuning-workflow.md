# Complete Tuning + MCTS Workflow

```bash
  ┌──────────────────────────────────────────────────────────────────────────────┐
  │                    FULL BOT OPTIMIZATION PIPELINE                            │
  └──────────────────────────────────────────────────────────────────────────────┘
```

## Default Weights

Both **GreedyBot** and **MctsBot** automatically load tuned weights from:
```
data/weights/default.toml
```

If the file doesn't exist, they fall back to hardcoded defaults. You can:
- **Override per-run**: Use `--weights` flag (e.g., `--weights1 path/to/custom.toml`)
- **Update defaults**: Replace `default.toml` with your best tuned weights

**Quick workflow to update defaults:**
```bash
# After tuning completes
cp experiments/mcts/2026-01-12_HHMM_tag/weights.toml data/weights/default.toml
git add data/weights/default.toml
git commit -m "Update default weights: [description]"
```

---

## STEP 1: TUNE WEIGHTS

```bash
# Run tuning experiment (outputs to experiments/mcts/)
$ cargo run --release --bin tune -- \
    --tag my_experiment \
    --mode vs-greedy \
    --generations 50
```

* Creates: `experiments/mcts/2026-01-12_HHMM_my_experiment/`
* Output: `weights.toml` with 20 optimized weight parameters
* Also creates: `train.log`, `summary.txt`, and `plots/` (after analysis)

## STEP 1b: ANALYZE RESULTS

```bash
# Analyze and visualize the tuning run
$ ./scripts/analyze-tuning.sh --latest
```

* Generates: `stats.csv`, `plots/overview.png`, `plots/efficiency.png`, etc.
* Creates: `plots/summary_report.txt` with detailed metrics


## STEP 2: USE WITH GREEDY BOT

```bash
# GreedyBot now uses default.toml automatically
$ cargo run --release --bin arena -- \
    --bot1 greedy --bot2 greedy \
    --games 100

# Or override with specific weights for comparison
$ cargo run --release --bin arena -- \
    --bot1 greedy --bot2 greedy \
    --weights1 experiments/mcts/2026-01-12_HHMM_my_experiment/weights.toml \
    --games 100
```

* → GreedyBot evaluates states using default.toml (or hardcoded fallback)
* → Use `--weights` to test different weight configurations


## STEP 3: USE WITH MCTS BOT  ← POWERFUL!

```bash
# MctsBot automatically uses default.toml for rollouts
$ cargo run --release --bin arena -- \
    --bot1 mcts --bot2 mcts \
    --games 20

# Or test with specific weights
$ cargo run --release --bin arena -- \
    --bot1 mcts --bot2 mcts \
    --weights1 experiments/mcts/2026-01-12_HHMM_my_experiment/weights.toml \
    --games 20
```

* → MctsBot uses tuned weights from default.toml for rollout evaluation
* → Better rollouts = more accurate value estimates = better moves
* → Can still override with `--weights` for testing


##  HOW IT WORKS

```bash
  ════════════

  ┌─────────────┐
  │  MctsBot    │
  └──────┬──────┘
         │ for each simulation:
         ▼
  ┌─────────────┐     ┌─────────────┐
  │  Selection  │────▶│  Expansion  │
  │   (UCB1)    │     │             │
  └─────────────┘     └──────┬──────┘
                             │
                             ▼
                     ┌─────────────────┐
                     │    Rollout      │
                     │  (GreedyBot)    │◀── Uses tuned weights!
                     │                 │    Better state evaluation
                     └────────┬────────┘    = smarter play during
                              │             simulation
                              ▼
                     ┌─────────────────┐
                     │ Backpropagation │
                     │  (update tree)  │
                     └─────────────────┘
```

The key insight: MCTS quality depends heavily on its rollout policy. By using tuned GreedyBot weights for rollouts, the simulations more accurately reflect good play, leading to better value estimates for each action.