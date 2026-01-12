# Complete Tuning + MCTS Workflow

```bash
  ┌──────────────────────────────────────────────────────────────────────────────┐
  │                    FULL BOT OPTIMIZATION PIPELINE                            │
  └──────────────────────────────────────────────────────────────────────────────┘
```
## STEP 1: TUNE WEIGHTS

```bash
# ════════════════════
$ cargo run --release --bin tune -- \
    --mode vs-greedy \
    --generations 50 \
    --output tuned.toml
```

* Output: tuned.toml with 20 optimized weight parameters


## STEP 2: USE WITH GREEDY BOT

```bash
# ═══════════════════════════
$ cargo run --release --bin arena -- \
    --bot1 greedy --bot2 greedy \
    --weights1 tuned.toml \
    --games 100
```

* → GreedyBot evaluates states using tuned weights
* → Wins ~90% against default GreedyBot


## STEP 3: USE WITH MCTS BOT  ← NEW!

```bash
# ═════════════════════════════════
$ cargo run --release --bin arena -- \
    --bot1 mcts --bot2 mcts \
    --weights1 tuned.toml \
    --games 20
```

* → MctsBot uses tuned GreedyBot for rollout evaluation
* → Better rollouts = more accurate value estimates = better moves
* → Wins ~70% against default MctsBot


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