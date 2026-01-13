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

## ⚡ PERFORMANCE TIP: MCTS Simulations

**CRITICAL:** When using modes that involve MCTS opponents (`multi-opponent`, `generalist`, `faction-specialist`), the `--mcts-sims` parameter **massively affects training time**.

### The Math

MCTS simulations scale linearly in time, but opponent quality improves with **diminishing returns**:

```
Simulations    Opponent Strength    Time/Gen    100 Gens Total
-----------    -----------------    --------    --------------
25             ~70% optimal         3s          5 minutes
50  ⭐         ~85% optimal         6s          10 minutes  ← RECOMMENDED
100            ~92% optimal         15s         25 minutes
200            ~95% optimal         63s         105 minutes
500            ~97% optimal         180s        300 minutes
```

**Real benchmark** (faction-specialist, 100 gens, 50 games):
- `--mcts-sims 200`: **4087.6s** (68 min) 🐌
- `--mcts-sims 50`: **239.7s** (4 min) 🚀
- **17x speedup!**

### Why 50 Sims is Optimal

1. **More generations > stronger opponent** - Fast iteration lets CMA-ES explore better
2. **50-sim MCTS is already strong** - Much better than GreedyBot baseline
3. **Weights converge to high WR anyway** - Example: 89.6% → 97.9% WR vs 50-sim opponent

### Recommended Practice

✅ **DO:** Use `--mcts-sims 50` for tuning (fast iteration)  
✅ **DO:** Validate with `--mcts-sims 200` in arena after tuning (rigorous test)  
❌ **DON'T:** Use 200+ sims during tuning (wastes 10x time for ~5% quality gain)

---

## STEP 1: TUNE WEIGHTS

```bash
# Run tuning experiment (outputs to experiments/mcts/)
# IMPORTANT: Use --mcts-sims 50 for fast iteration!
$ cargo run --release --bin tune -- \
    --tag my_experiment \
    --mode multi-opponent \
    --generations 50 \
    --mcts-sims 50

# Or for truly universal weights (slower but most robust):
$ cargo run --release --bin tune -- \
    --tag generalist_v1 \
    --mode generalist \
    --generations 100 \
    --games 100 \
    --mcts-sims 50
```

**Mode options:**
- `vs-random`: Fast baseline (vs RandomBot)
- `vs-greedy`: Medium baseline (vs default GreedyBot) 
- `multi-opponent`: Robust (vs Random 10%, Greedy 40%, MCTS 50%) **← RECOMMENDED**
- `generalist`: Ultra-robust (ALL deck matchups vs Random/Greedy/MCTS) **← Most powerful, slowest**
- `specialist`: Optimize for specific deck matchup (requires `--deck` and `--opponent`)

* Creates: `experiments/mcts/2026-01-12_HHMM_my_experiment/`
* Output: `weights.toml` with 24 optimized weight parameters
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