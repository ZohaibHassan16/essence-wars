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
- Base model: BC (61% vs Greedy, 10k games training data)
- MCTS simulations: 50 (for data generation)
- Games to generate: 1,000
- Expected samples: ~90,000 (90 moves/game avg)

**Data Generation**:
| Metric | Value |
|--------|-------|
| Games generated | 1,000 |
| Total samples | 100,863 |
| Generation time | 45.9 minutes |
| Avg game length | 100.9 moves |
| P0 win rate | 52.8% (balanced) |

**Training**:
| Metric | Value |
|--------|-------|
| Epochs | 20 |
| Best epoch | 13 |
| Final val loss | 3.7859 |
| Policy accuracy | 50.8% |
| Training time | 0.7 minutes |

**Results**:
| Model | Win Rate vs Greedy | Improvement |
|-------|-------------------|-------------|
| BC-v1 (baseline) | 61% | - |
| BC-v1 + MCTS-25 | 64% | +3% |
| BC-v2 (ExIt iter 1) | **51%** | **-10%** ❌ |
| BC-v2 + MCTS-25 | **36%** | **-25%** ❌❌ |
| BC-v2 + MCTS-50 | **32%** | **-29%** ❌❌❌ |

**Critical Finding**: BC-v2's value function is broken! MCTS *hurts* the model because bad value estimates degrade search quality.

**Analysis - Why Did ExIt Fail?**

The ExIt iteration produced a *worse* model than the baseline. Potential causes:

1. **Insufficient data**: 100k samples vs original 900k (9x less data)
2. **Lower MCTS quality**: 50 sims vs original 100 sims for vanilla MCTS
3. **Neural prior bias**: BC-guided MCTS may reinforce model's blind spots
4. **Value function weakness**: BC may not have a good value estimate for leaf evaluation

**Hypothesis**: The BC model we used as a guide has only ~60% accuracy. Its policy prior may bias MCTS away from good moves, and its value estimates for leaf nodes may be inaccurate. This creates a negative feedback loop where the search quality is degraded by the very model we're trying to improve.

**Next Steps**:
- [ ] Try ExIt with more games (5k-10k) to match original data size
- [ ] Try ExIt with 100 sims instead of 50
- [ ] Try ExIt starting from PPO-Argentum instead of BC
- [ ] Compare vanilla MCTS data generation vs neural-guided

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

### Finding 1: ExIt Iteration 1 Failed (-10% regression)

Training on BC-guided MCTS data produced a worse model (51% vs 61% baseline).

### Finding 2: Value Function Corruption is Critical

The ExIt-trained model's value function is broken:
- Raw BC-v2: 52% win rate
- BC-v2 + MCTS-25: 36% win rate (**MCTS hurts!**)
- BC-v2 + MCTS-50: 32% win rate

This is the opposite of what happens with BC-v1, where MCTS helps (+3%).

### Finding 3: Data Size Likely Insufficient

The ExIt dataset had 100k samples vs the original 900k (9x less). The model may have underfit, leading to poor generalization.

### Finding 4: Neural Prior May Degrade Search

When BC-v1 guides MCTS, its ~60% accuracy means the search is biased. Combined with potentially inaccurate value estimates, the generated training data may be lower quality than vanilla MCTS data.

### Recommendation

Before pursuing ExIt further:
1. **FIX THE VALUE FUNCTION** - Root cause identified (see Finding 5)
2. Generate more data (5k-10k games) to match original dataset size
3. Consider using vanilla MCTS for data generation instead of neural-guided

### Finding 6: Policy-Guided MCTS Works When Value Is Disabled (BREAKTHROUGH)

After fixing the card ID normalization issue, we discovered that the value function itself is the problem:

| Configuration | Win Rate vs Greedy |
|--------------|-------------------|
| Raw BC network | 55% |
| BC + MCTS-50 (neural value) | 4% |
| BC + MCTS-50 (random rollouts) | **86%** |

**Key Insight**: The neural policy is excellent for guiding MCTS exploration, but the value function gives misleading signals. When we replace neural value evaluation with random rollouts to terminal states, MCTS works brilliantly (+31% improvement).

**Why Value Function Fails**:
- Training on game outcomes (±1) creates binary predictions
- Model gives extreme values (±0.99) that saturate softmax
- Values are inconsistent between "your turn with actions" vs "opponent's turn"
- MCTS needs calibrated probabilistic estimates, not binary classifications

**Solution Options**:
1. Use MCTS with random rollouts (current: 86%)
2. Use MCTS with greedy rollouts (likely even better)
3. Train value function differently (TD learning, MCTS value targets)
4. Distill MCTS policy back into network

---

### Finding 5: ROOT CAUSE - Unnormalized Card IDs (CRITICAL)

**Investigation revealed the fundamental problem:**

The Rust `tensor.rs` outputs **raw card IDs** (1000-4000) without normalization:
```rust
tensor[*idx] = card.card_id.0 as f32;  // Raw value, not normalized!
```

This causes **feature explosion** through the network:
- Input tensor norm: 12,195
- After 4 residual blocks: 35,091
- Pre-tanh value: 10,789 → tanh → 1.0

**The value function isn't "learning badly" - it's saturated by extreme inputs!**

| Stage | Norm/Value | Problem |
|-------|------------|---------|
| Input | 12,195 | Card IDs are 2000-4000 |
| After ResBlocks | 35,091 | Features explode |
| Value head output | 10,789 | Extreme value |
| After tanh | 1.0 | Always saturates |

**Why policy still works**: Softmax normalizes extreme logits into valid probabilities.

**Fix options**:
1. **Rust fix**: `card_id / 5000.0` in tensor.rs (cleanest)
2. **Python fix**: Input normalization layer
3. **Embeddings**: Use `nn.Embedding` for card IDs (best long-term)

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
