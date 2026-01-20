# Post-AlphaZero Research Tracker

> **Goal**: Explore compute-efficient alternatives to AlphaZero for improving neural agents in Essence Wars.

**Status**: 🆕 Starting Fresh
**Started**: January 2026
**Context**: AlphaZero requires ~1000x more compute than available. Pivoting to practical alternatives.

---

## Executive Summary

After extensive AlphaZero experimentation, we concluded that self-play from scratch is not viable for independent researchers. Our best results so far:

| Method | Win Rate vs Greedy | Compute | Status |
|--------|-------------------|---------|--------|
| PPO-Argentum | **72%** | 30 min | ✅ Best model |
| PPO-Flat | 71% | 30 min | ✅ |
| BC (MCTS-10k) | 59% | 12 min | ✅ |
| AlphaZero (all variants) | 0-2% | 14+ hours | ❌ Failed |

**New Goal**: Push beyond 72% using compute-efficient methods.

---

## Research Tracks

### Track A: Expert Iteration (ExIt)

**Status**: ❌ Not Started
**Priority**: ⭐⭐ High (2nd)
**Estimated Effort**: Medium

#### Concept

Iterative improvement without online self-play:

```
┌─────────────────────────────────────────────────────────┐
│  1. Train BC on MCTS data (current: 59%)                │
│  2. Use BC model as MCTS prior (policy + value)         │
│  3. Generate new games with BC-guided MCTS              │
│  4. Train new BC on improved data                       │
│  5. Repeat until convergence                            │
└─────────────────────────────────────────────────────────┘
```

#### Why It Might Work

- **No online self-play**: Training is purely supervised (BC), avoiding distribution shift
- **MCTS quality improves**: Better BC → better MCTS prior → better games → better BC
- **Proven approach**: Used in Expert Iteration paper, Libratus poker AI

#### Implementation Plan

1. **Modify data generation** to accept neural network for MCTS prior
2. **Generate ExIt dataset**: BC-guided MCTS vs BC-guided MCTS games
3. **Train BC on new dataset**
4. **Evaluate and iterate**

#### Tasks

- [ ] Add neural network prior support to Rust MCTS
- [ ] Create `generate_exit_data.py` script
- [ ] Run ExIt iteration 1: BC(59%) → MCTS-BC games → BC-v2
- [ ] Evaluate BC-v2 vs Greedy
- [ ] Run ExIt iteration 2 if promising
- [ ] Document results

#### Expected Outcome

- **Optimistic**: BC improves from 59% → 65%+ per iteration
- **Pessimistic**: No improvement (MCTS-BC not better than MCTS-MCTS)

#### Notes

```
Key question: Does BC-guided MCTS produce better training signal than vanilla MCTS?
- If yes: ExIt should improve BC iteratively
- If no: We're just regenerating similar data
```

---

### Track B: MCTS-Augmented Inference

**Status**: ✅ Complete
**Priority**: ⭐⭐⭐ Highest (1st)
**Estimated Effort**: Low
**Result**: +6% improvement (PPO: 59% → 65% with 25 sims)

#### Concept

Use trained neural network inside MCTS at play time (not training):

```
┌─────────────────────────────────────────────────────────┐
│  Standard Play:                                         │
│    state → neural_net → action                          │
│                                                         │
│  MCTS-Augmented Play:                                   │
│    state → MCTS(prior=neural_net, value=neural_net)     │
│          → action                                       │
└─────────────────────────────────────────────────────────┘
```

#### Why It Might Work

- **This is how AlphaZero actually plays**: The raw network is never used directly
- **Combines intuition + search**: Neural net provides good starting point, MCTS refines
- **No training required**: Just inference-time enhancement
- **Immediate results**: Can test today with existing models

#### Implementation Plan

1. **Create `NeuralMctsBot`** that wraps any neural network
2. **Use network for**:
   - Policy prior (guide which actions to explore)
   - Value estimation (evaluate leaf nodes)
3. **Benchmark** with different sim counts (25, 50, 100, 200)

#### Tasks

- [x] Implement `NeuralMctsBot` class in Python
- [x] Add policy prior support to MCTS
- [x] Add value function support to MCTS
- [x] Test with PPO-Argentum + MCTS → **59% → 65% (+6%)**
- [x] Test with BC + MCTS → **64% → 68% (+4%)**
- [x] Benchmark: sims vs win rate vs inference time
- [x] Document results → `papers/track-b-mcts-augmented-inference.md`

#### Expected Outcome

- **Optimistic**: PPO-Argentum + MCTS(100) achieves 80%+ vs Greedy
- **Pessimistic**: Minimal improvement over raw network (MCTS overhead not worth it)

#### Key Metrics

| Model | Raw Win Rate | +MCTS(50) | +MCTS(100) | +MCTS(200) |
|-------|--------------|-----------|------------|------------|
| PPO-Argentum | 72% | ? | ? | ? |
| PPO-Flat | 71% | ? | ? | ? |
| BC | 59% | ? | ? | ? |

#### Notes

```
This is the lowest-hanging fruit. If MCTS-augmented inference works,
we get an immediate boost with zero additional training.

The question is whether our networks provide useful priors, or if
vanilla MCTS is already optimal at 100 sims.
```

---

### Track C: Offline RL (DQN, CQL, IQL)

**Status**: ❌ Not Started
**Priority**: ⭐ Medium (4th)
**Estimated Effort**: High

#### Concept

Learn from fixed MCTS dataset using offline RL algorithms:

```
┌─────────────────────────────────────────────────────────┐
│  Dataset: 10k MCTS games (state, action, reward)        │
│                                                         │
│  DQN:  Learn Q(s,a) from transitions                    │
│  CQL:  Q-learning + conservative penalty for OOD        │
│  IQL:  Implicit Q-learning (no policy in Q update)      │
└─────────────────────────────────────────────────────────┘
```

#### Why It Might Work

- **Designed for fixed data**: These algorithms handle distribution shift
- **No self-play**: Learn purely from expert demonstrations
- **Rich signal**: Q-learning uses per-step rewards (if available) or bootstrapped values

#### Challenges

- **Sparse rewards**: Only win/lose at game end
- **Long horizons**: ~90 steps per game
- **Implementation complexity**: CQL/IQL are non-trivial

#### Implementation Plan

1. **Start with DQN** as baseline
2. **Convert dataset** to (s, a, r, s', done) transitions
3. **Implement conservative Q-learning** if DQN struggles
4. **Compare to BC** (which is simpler)

#### Tasks

- [ ] Implement basic DQN trainer
- [ ] Convert MCTS dataset to transition format
- [ ] Train DQN on MCTS-10k
- [ ] Evaluate vs Greedy
- [ ] If DQN fails, implement CQL
- [ ] Document results

#### Expected Outcome

- **Optimistic**: Offline RL matches or beats BC (59%)
- **Pessimistic**: Sparse rewards + long horizons make Q-learning impractical

#### Notes

```
This track has the highest implementation effort and uncertain payoff.
Only pursue if Tracks A and B don't pan out.
```

---

### Track D: PPO Improvements

**Status**: ❌ Not Started
**Priority**: ⭐⭐ Medium (3rd)
**Estimated Effort**: Medium

#### Concept

Push PPO beyond 72% with advanced techniques:

```
┌─────────────────────────────────────────────────────────┐
│  D1. Curriculum Learning                                │
│      RandomBot → GreedyBot → Self-Play                  │
│                                                         │
│  D2. League Play                                        │
│      Train against pool of past checkpoints             │
│                                                         │
│  D3. Population-Based Training (PBT)                    │
│      Evolve hyperparameters during training             │
│                                                         │
│  D4. Reward Shaping                                     │
│      Add intermediate rewards (life diff, board ctrl)   │
└─────────────────────────────────────────────────────────┘
```

#### Sub-tracks

##### D1. Curriculum Learning

**Idea**: Start with easy opponents, gradually increase difficulty

```python
# Curriculum schedule
opponent_schedule = [
    (0, 100k, RandomBot),      # Easy start
    (100k, 200k, GreedyBot),   # Medium
    (200k, 300k, SelfPlay),    # Hard
]
```

**Tasks**:
- [ ] Implement curriculum scheduler in PPO trainer
- [ ] Test curriculum: Random → Greedy
- [ ] Test curriculum: Random → Greedy → Self-play
- [ ] Compare to fixed-opponent training

##### D2. League Play

**Idea**: Train against diverse opponents (pool of past checkpoints)

```python
# League of opponents
league = [
    GreedyBot,
    PPOCheckpoint(step=50k),
    PPOCheckpoint(step=100k),
    PPOCheckpoint(step=150k),
    CurrentPolicy,  # Self-play
]
```

**Tasks**:
- [ ] Implement league opponent sampling
- [ ] Save checkpoints at regular intervals
- [ ] Train with league (50% Greedy, 50% league)
- [ ] Compare to single-opponent training

##### D3. Population-Based Training (PBT)

**Idea**: Run multiple agents, copy hyperparams from best performers

**Tasks**:
- [ ] Implement basic PBT with 4-8 agents
- [ ] Hyperparams to evolve: LR, entropy coef, clip range
- [ ] Run PBT for 300k steps
- [ ] Compare best PBT agent to fixed-hyperparam agent

##### D4. Reward Shaping

**Idea**: Add dense intermediate rewards

```python
# Shaped reward
reward = (
    win_reward +                    # +1 / -1
    0.01 * life_differential +      # Encourage life lead
    0.01 * board_control +          # Encourage board presence
    0.001 * cards_played            # Encourage action
)
```

**Tasks**:
- [ ] Implement shaped rewards in environment
- [ ] Train PPO with shaped rewards
- [ ] Compare to sparse rewards
- [ ] Check if shaped rewards hurt final performance

#### Expected Outcome

- **Optimistic**: One of these pushes PPO to 80%+
- **Pessimistic**: Marginal gains (73-75%)

---

## Experiment Log

| Date | Track | Experiment | Result | Notes |
|------|-------|------------|--------|-------|
| 2026-01-20 | - | Research tracker created | - | Fresh start post-AlphaZero |
| 2026-01-20 | B | PPO-Argentum + MCTS-25 | 59% → 65% | +6% improvement |
| 2026-01-20 | B | PPO-Argentum + MCTS-50 | 59% → 61% | +2% improvement |
| 2026-01-20 | B | PPO-Argentum + MCTS-100 | 59% → 63% | +4% improvement |
| 2026-01-20 | B | BC + MCTS-100 | 64% → 68% | +4% improvement |
| 2026-01-20 | B | Track B complete | ✅ | 25 sims is sweet spot |

---

## Current Focus

**Next Action**: Track A (Expert Iteration) or Track D (PPO Improvements)

**Rationale**: Track B complete with modest gains (+6%). Expert Iteration could potentially break through the ~65% ceiling by generating better training data.

---

## Success Metrics

| Metric | Current Best | Target | Stretch |
|--------|--------------|--------|---------|
| Win Rate vs Greedy (raw) | 59% (PPO-Argentum) | 75% | 85% |
| Win Rate vs Greedy (MCTS-25) | **65%** (PPO-Argentum) | 80% | 85% |
| Win Rate vs MCTS-100 | ~50%? | 60% | 70% |

*Note: Original 72% measurement may have been seed-specific; 59-65% more representative*

---

## Resources

### Existing Assets

- **Models**: PPO-Argentum (72%), PPO-Flat (71%), BC (59%)
- **Datasets**: MCTS-10k (900k samples), MCTS-100k (9M samples)
- **Infrastructure**: TensorBoard, benchmark suite, HuggingFace integration

### Key Files

| File | Purpose |
|------|---------|
| `python/essence_wars/agents/ppo.py` | PPO implementation |
| `python/essence_wars/agents/alphazero.py` | MCTS + neural network |
| `python/scripts/train_ppo.py` | PPO training script |
| `python/scripts/train_behavioral_cloning.py` | BC training script |
| `crates/cardgame/src/bots/mcts.rs` | Rust MCTS implementation |

### References

- [Expert Iteration paper](https://arxiv.org/abs/1705.08439)
- [Conservative Q-Learning (CQL)](https://arxiv.org/abs/2006.04779)
- [Implicit Q-Learning (IQL)](https://arxiv.org/abs/2110.06169)
- [Population Based Training](https://arxiv.org/abs/1711.09846)

---

## Notes & Ideas

```
Random thoughts and ideas to explore:

1. What if we use MCTS-augmented inference to generate ExIt data?
   - BC + MCTS(100) generates games
   - Train BC on those games
   - This combines Track A and B

2. Could we use the PPO value head inside MCTS?
   - PPO learns V(s), not Q(s,a)
   - But MCTS needs leaf evaluation, which V(s) provides

3. Ensemble of models?
   - PPO-Argentum + PPO-Flat + BC voting
   - Might be more robust than single model

4. Temperature scheduling for inference?
   - Start with high temperature (exploration)
   - Anneal to low temperature (exploitation)
   - Might help early game vs late game
```
