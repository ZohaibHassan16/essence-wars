# Decision Transformer for Essence Wars

> **Goal**: Apply Decision Transformer (DT) to learn game-playing policies from offline data, avoiding the instabilities of online RL.

**Status**: In Progress
**Started**: January 2026
**Paper Reference**: [Decision Transformer: Reinforcement Learning via Sequence Modeling](https://arxiv.org/abs/2106.01345) (Chen et al., 2021)

---

## Background

### Why Decision Transformer?

Our previous approaches hit a ~72% ceiling against GreedyBot:

| Approach | Win Rate | Problem |
|----------|----------|---------|
| PPO | 72% | Policy collapse, unstable training |
| AlphaZero | 0-2% | Requires 1000x more compute |
| Behavioral Cloning | 59% | Doesn't use reward signal |
| Policy Distillation | 71.7% | Still just imitation |
| Expert Iteration | 51% | Negative transfer |

**Decision Transformer offers a different paradigm**:
- Treats RL as **sequence modeling** (not value estimation)
- Uses **offline data** (no unstable self-play)
- Conditions on **desired return** (goal-conditioned)
- Standard **supervised learning** (stable, well-understood)

### How Decision Transformer Works

Traditional RL learns: `π(a|s)` or `Q(s,a)` → action

Decision Transformer learns: `π(a|s, R, history)` → action given desired return

```
Input Sequence (GPT-style):
┌─────────┬─────────┬─────────┬─────────┬─────────┬─────────┐
│ R̂₁      │ s₁      │ a₁      │ R̂₂      │ s₂      │ a₂      │ ...
│ return  │ state   │ action  │ return  │ state   │ action  │
│ to-go   │         │         │ to-go   │         │         │
└─────────┴─────────┴─────────┴─────────┴─────────┴─────────┘
                              ↓
                    Causal Transformer
                              ↓
                    Predict next action
```

At inference time:
1. Set desired return R̂ = +1 (want to win)
2. Feed current state
3. Model predicts action that historically led to that return

---

## Implementation Plan

### Phase 1: Data Preparation

Convert MCTS dataset to trajectory format:

```python
# Current format (per-move):
{
    "state_tensor": [326 floats],
    "action": int,
    "action_mask": [256 bools],
    "mcts_policy": [256 floats],
    "value": float  # game outcome
}

# Decision Transformer format (per-game trajectory):
{
    "states": [[326], [326], ...],      # T x 326
    "actions": [a1, a2, ...],            # T
    "returns_to_go": [R, R-r1, R-r2...], # T (cumulative return remaining)
    "timesteps": [0, 1, 2, ...],         # T
    "attention_mask": [1, 1, 1, ...],    # T
}
```

**Return-to-go calculation**:
- Game ends with reward +1 (win) or -1 (loss)
- returns_to_go[t] = final_reward (since only terminal reward)
- All timesteps for winner have R̂ = +1
- All timesteps for loser have R̂ = -1

### Phase 2: Model Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Decision Transformer                    │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Embeddings:                                            │
│  ├── return_embed: Linear(1 → d_model)                 │
│  ├── state_embed:  Linear(326 → d_model)               │
│  ├── action_embed: Embedding(256, d_model)              │
│  └── timestep_embed: Embedding(max_len, d_model)        │
│                                                         │
│  Transformer:                                           │
│  ├── n_layers: 4-6                                      │
│  ├── n_heads: 4-8                                       │
│  ├── d_model: 128-256                                   │
│  └── d_ff: 512-1024                                     │
│                                                         │
│  Output:                                                │
│  └── action_head: Linear(d_model → 256)                │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Parameter count estimate**:
- d_model=128, n_layers=4: ~1.5M parameters
- d_model=256, n_layers=6: ~5M parameters

### Phase 3: Training

```python
# Loss: Cross-entropy on action prediction
loss = CrossEntropy(predicted_actions, target_actions, mask=action_mask)

# Training config
batch_size = 64
context_length = 20  # Last K timesteps
learning_rate = 1e-4
epochs = 50
```

### Phase 4: Inference

```python
def get_action(model, state, action_mask, history, target_return=1.0):
    """
    Condition on winning (target_return=1.0) and predict action.
    """
    # Build input sequence from history
    returns = [target_return] * len(history)
    states = [h.state for h in history] + [state]
    actions = [h.action for h in history]

    # Forward pass
    action_logits = model(returns, states, actions)

    # Mask invalid actions and sample
    action_logits[~action_mask] = -inf
    action = argmax(action_logits[-1])

    return action
```

---

## Experiment Log

| Date | Experiment | Result | Notes |
|------|------------|--------|-------|
| 2026-01-21 | Paper created | - | Starting implementation |
| 2026-01-21 | Model implemented | 914k params | d_model=128, n_layers=4, K=20 |
| 2026-01-21 | Training on 10k MCTS data | 43.5% val acc | 50 epochs, 13min, best at epoch 45 |
| 2026-01-21 | Eval vs Greedy (R=+1) | **42.8%** | Below behavioral cloning (59%) |
| 2026-01-21 | Eval vs Greedy (R=-1) | 31.5% | Conditioning works (worse with lose target) |
| 2026-01-21 | Eval vs Random | 83.5% | Reasonable baseline |

---

## Results

### First Experiment: Baseline Decision Transformer

**Configuration:**
- d_model: 128
- n_layers: 4
- n_heads: 4
- context_length: 20
- Parameters: 914,432

**Training:**
- Dataset: 10k MCTS-50 games (20k trajectories, 836k steps)
- Best validation accuracy: 43.5%
- Best validation loss: 2.45
- Training time: 13.2 minutes

**Evaluation vs Greedy (500 games):**
| Target Return | Win Rate |
|---------------|----------|
| +1.0 (aim to win) | 42.8% |
| -1.0 (aim to lose) | 31.5% |

**Evaluation vs Random (200 games):**
| Target Return | Win Rate |
|---------------|----------|
| +1.0 (aim to win) | 83.5% |

### Comparison to Other Approaches

| Approach | vs Greedy | Notes |
|----------|-----------|-------|
| PPO | 72% | Best raw network |
| Policy Distillation | 71.7% | From MCTS-50 |
| ExIt Iteration 1 | 70.4% | Marginal improvement |
| Behavioral Cloning | 59% | Basic imitation |
| **Decision Transformer** | **42.8%** | Underperforms |
| Random | ~5% | Baseline |

### Analysis: Why Did DT Underperform?

1. **Sparse Rewards Problem**: With only terminal rewards (+1/-1), the return-to-go is constant throughout each trajectory. The model learns "winning player actions" vs "losing player actions" but can't learn intermediate progress signals.

2. **No State-Value Learning**: Unlike distillation which learns policy π(a|s), DT learns π(a|s,R,history). The history conditioning adds complexity without benefit for this game.

3. **Context Window Limitation**: K=20 sees ~1/4 of average game. Winning patterns may require longer-range dependencies.

4. **Action Conditioning Hurts**: DT predicts actions conditioned on past actions. In chess/card games, positions are largely independent - the best move depends on current state, not action history.

5. **Data Efficiency**: The same 10k games that give 71.7% with distillation only give 42.8% with DT.

### Potential Improvements

1. **Use MCTS values as intermediate returns**: Instead of constant +1/-1, use MCTS value estimates at each step
2. **Remove action conditioning**: Predict action from (return, state) only
3. **Longer context**: Try K=50 or full game
4. **Different architecture**: Consider state-only transformer without action/return interleaving

---

## Architecture Decisions

### Context Length

Decision Transformer uses a fixed context window of recent timesteps.

Options for Essence Wars:
- **K=10**: Very recent history only
- **K=20**: ~1/4 of average game (recommended start)
- **K=50**: Half game history
- **Full game**: Memory intensive, may not help

### State Representation

Options:
1. **Raw state tensor** (326 floats) - Current approach
2. **State + action mask** (326 + 256 = 582 floats)
3. **Embedded state** (pass through learned embedding first)

Starting with option 1 for simplicity.

### Return Conditioning

For Essence Wars (sparse terminal reward):
- Winner: returns_to_go = [+1, +1, +1, ...] (constant)
- Loser: returns_to_go = [-1, -1, -1, ...] (constant)

This simplifies to binary conditioning: "am I on track to win?"

Alternative: Use MCTS value estimates as intermediate returns (if available in data).

---

## Expected Outcomes

### Optimistic
- DT learns to condition on winning trajectory patterns
- Achieves 75%+ vs Greedy (beats all previous approaches)
- Stable training without collapse

### Realistic
- DT matches distillation (~72%)
- Provides different failure modes (complementary to other approaches)
- Cleaner training dynamics

### Pessimistic
- DT underperforms BC (~65%)
- Context window too short to capture game strategy
- Return conditioning doesn't help (all moves look similar)

---

## Comparison to Other Approaches

| Aspect | PPO | BC | AlphaZero | Decision Transformer |
|--------|-----|----|-----------|--------------------|
| Learning signal | Reward | Actions | MCTS policy | Return-conditioned actions |
| Online/Offline | Online | Offline | Online | **Offline** |
| Stability | Poor | Good | Poor | **Good** |
| Uses reward? | Yes | No | Yes | **Yes** |
| Temporal reasoning | RNN/None | None | MCTS | **Attention** |
| Compute | Medium | Low | Very High | **Low-Medium** |

---

## References

1. Chen, L., et al. (2021). "Decision Transformer: Reinforcement Learning via Sequence Modeling"
2. Janner, M., et al. (2021). "Offline Reinforcement Learning as One Big Sequence Modeling Problem"
3. Lee, K., et al. (2022). "Multi-Game Decision Transformers"

---

## Files

| File | Purpose |
|------|---------|
| `python/essence_wars/agents/decision_transformer.py` | Model implementation |
| `python/scripts/train_decision_transformer.py` | Training script |
| `python/scripts/prepare_dt_dataset.py` | Data preparation |
| `papers/decision-transformer.md` | This document |
