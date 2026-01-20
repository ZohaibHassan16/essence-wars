# Track A: Expert Iteration (ExIt)

> **Research Question**: Can we iteratively improve BC by generating better training data using neural-guided MCTS?

**Status**: 🔄 In Progress
**Started**: 2026-01-20
**Track**: A (from research-tracker.md)

---

## Hypothesis

Expert Iteration should improve model performance by:
1. Using current model to guide MCTS (better search than vanilla MCTS)
2. Generating games with improved MCTS policies
3. Training new BC model on improved data
4. Repeating until convergence

This avoids the distribution shift problem of AlphaZero by keeping training purely supervised (BC).

---

## Background

### Expert Iteration Algorithm

```
ExIt Loop:
┌─────────────────────────────────────────────────────────────┐
│  1. Start with policy π₀ (our BC model at 59%)             │
│                                                             │
│  2. Generate "expert" data:                                 │
│     - Play games: MCTS(π₀) vs MCTS(π₀)                     │
│     - Record (state, MCTS_policy, outcome) tuples          │
│                                                             │
│  3. Train new policy π₁ via BC on expert data              │
│                                                             │
│  4. Evaluate π₁ - if improved, set π₀ = π₁ and goto 2     │
└─────────────────────────────────────────────────────────────┘
```

### Why ExIt Might Work

| Problem with AlphaZero | How ExIt Solves It |
|------------------------|-------------------|
| Self-play distribution shift | Training is pure BC (supervised) |
| Degraded model generates bad data | MCTS provides consistent quality |
| Feedback loops | No online learning during play |

### Current Baseline

| Model | Win Rate vs Greedy | Training Data |
|-------|-------------------|---------------|
| BC (original) | 59-64% | Vanilla MCTS vs MCTS |
| BC + MCTS-25 | 65-68% | - |

**Goal**: Train BC-v2 that exceeds 65% raw (without needing MCTS at inference).

---

## Implementation Plan

### Phase 1: Data Generation

Generate games using neural-guided MCTS:

```python
# ExIt data generation
for game_idx in range(num_games):
    game = PyGame()
    game.reset()

    game_data = []
    while not game.is_done():
        # Get MCTS policy using neural network as prior
        action, policy = neural_mcts.get_action_with_game(game)

        # Record training sample
        game_data.append({
            'state': game.observe(),
            'mask': game.action_mask(),
            'policy': policy,  # MCTS visit distribution
            'player': game.current_player(),
        })

        game.step(action)

    # Add outcome to all samples
    outcome = game.get_reward(0)
    for sample in game_data:
        sample['value'] = outcome if sample['player'] == 0 else -outcome
```

### Phase 2: BC Training

Train new BC model on ExIt data (same as original BC training).

### Phase 3: Evaluation

Compare BC-v2 to:
- BC-v1 (original, 59%)
- BC-v1 + MCTS-25 (65%)
- Greedy baseline

---

## Experiments

### Experiment A1: ExIt Iteration 1

**Configuration**:
- Base model: BC (59% vs Greedy)
- MCTS simulations: 50 (for data generation)
- Games to generate: 1,000
- Expected samples: ~90,000 (90 moves/game avg)

**Data Generation**:
| Metric | Value |
|--------|-------|
| Games generated | ? |
| Total samples | ? |
| Generation time | ? |
| Avg game length | ? |

**Training**:
| Metric | Value |
|--------|-------|
| Epochs | 10 |
| Best epoch | ? |
| Final loss | ? |

**Results**:
| Model | Win Rate vs Greedy | Improvement |
|-------|-------------------|-------------|
| BC-v1 (baseline) | 59% | - |
| BC-v2 (ExIt iter 1) | ? | ? |
| BC-v2 + MCTS-25 | ? | ? |

---

### Experiment A2: ExIt Iteration 2 (if A1 successful)

**Configuration**:
- Base model: BC-v2 (from iteration 1)
- Games: 1,000

**Results**:
| Model | Win Rate vs Greedy |
|-------|-------------------|
| BC-v2 | ? |
| BC-v3 (ExIt iter 2) | ? |

---

### Experiment A3: Comparison with Different Base Models

Try ExIt starting from PPO instead of BC:

| Base Model | ExIt Result |
|------------|-------------|
| BC (59%) | ? |
| PPO-Argentum (59%) | ? |

---

## Analysis Questions

1. **Does neural-guided MCTS generate better training data?**
   - Compare BC trained on vanilla MCTS data vs neural MCTS data

2. **How many ExIt iterations until convergence?**
   - Track improvement per iteration

3. **Does starting model quality matter?**
   - Compare BC vs PPO as starting point

4. **What's the ceiling?**
   - How good can we get with this approach?

---

## Key Findings

*To be filled in after experiments*

1. TBD
2. TBD
3. TBD

---

## Code

### Implementation Files

| File | Purpose |
|------|---------|
| `python/scripts/generate_exit_data.py` | ExIt data generation |
| `python/scripts/train_behavioral_cloning.py` | BC training (existing) |

### Usage

```bash
# Generate ExIt data
uv run python python/scripts/generate_exit_data.py \
    --model models/bc_mcts_10k_best.pt \
    --games 1000 \
    --sims 50 \
    --output data/datasets/exit_iter1.jsonl.gz

# Train BC on ExIt data
uv run python python/scripts/train_behavioral_cloning.py \
    --dataset data/datasets/exit_iter1.jsonl.gz \
    --epochs 10 \
    --output models/bc_exit_iter1.pt
```

---

## Timeline

| Date | Milestone |
|------|-----------|
| 2026-01-20 | Track A started |
| TBD | Data generation script complete |
| TBD | ExIt iteration 1 data generated |
| TBD | BC-v2 trained and evaluated |
| TBD | Analysis complete |

---

## References

- [Expert Iteration paper (Anthony et al., 2017)](https://arxiv.org/abs/1705.08439)
- [Thinking Fast and Slow with Deep Learning](https://arxiv.org/abs/1705.08439)
- Track B findings: `papers/track-b-mcts-augmented-inference.md`
