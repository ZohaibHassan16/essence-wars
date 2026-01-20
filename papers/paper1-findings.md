# Paper 1: Neural Network Training for Essence Wars

**Working Title**: "Learning to Play Essence Wars: A Comparison of Reinforcement Learning Approaches for Deterministic Card Games"

**Status**: Data Collection Phase
**Last Updated**: 2026-01-20

---

## Abstract (Draft)

We present a comparative study of neural network training approaches for Essence Wars, a deterministic perfect-information card game designed for AI research. We evaluate Proximal Policy Optimization (PPO), Behavioral Cloning (BC) from MCTS demonstrations, and AlphaZero-style self-play, along with the impact of pre-trained card embeddings on sample efficiency.

---

## 1. Game Environment

### 1.1 Essence Wars Overview

- **Type**: Deterministic, perfect-information, two-player card game
- **State Space**: 326-dimensional continuous tensor
- **Action Space**: 256 discrete actions (masked by legality)
- **Cards**: 300 unique cards across 4 factions
- **Average Game Length**: ~90 moves (45 per player)
- **Engine Performance**: ~80k random games/sec, ~17k greedy games/sec

### 1.2 Factions

| Faction | Cards | Playstyle |
|---------|-------|-----------|
| Argentum Combine | 75 | Defensive, Guard/Shield |
| Symbiote Circles | 75 | Aggressive, Rush/Lethal |
| Obsidion Syndicate | 75 | Burst damage, Lifesteal/Stealth |
| Free-Walkers (Neutral) | 75 | Utility, Ranged/Charge |

### 1.3 Baseline Agents

| Agent | Description | Self-Play Win Rate |
|-------|-------------|-------------------|
| RandomBot | Uniform random legal actions | 50% (by definition) |
| GreedyBot | Hand-crafted heuristic (24 weights) | 50% vs self |
| MctsBot (100 sims) | UCB1 tree search | ~65% vs Greedy |
| MctsBot (1000 sims) | UCB1 tree search | ~75% vs Greedy |

---

## 2. Datasets

### 2.1 MCTS Self-Play Data

Generated using MctsBot (100 simulations) vs MctsBot (100 simulations):

| Dataset | Games | Samples | Compressed Size |
|---------|-------|---------|-----------------|
| mcts_1k | 1,000 | ~90,000 | 60 MB |
| mcts_10k | 10,000 | ~900,000 | 565 MB |
| mcts_100k | 100,000 | ~9,000,000 | 5.6 GB |

Each sample contains:
- State tensor (326 floats)
- Action mask (256 bools)
- MCTS policy (visit count distribution)
- Game outcome (+1/-1/0)

### 2.2 Card Co-occurrence Data

Extracted from MCTS games for Card2Vec training:
- Co-occurrence pairs from hands and board states
- Window size: 5 cards
- Total pairs: ~50M (sampled to 500k via reservoir sampling)

---

## 3. Methods

### 3.1 Network Architectures

#### Flat Network (Baseline)
- Input: 326-dim state tensor
- Architecture: MLP with residual blocks
- Hidden dim: 256
- Residual blocks: 4
- Output: Policy (256) + Value (1)
- Parameters: ~2.4M

#### Embedded Network
- Input: 326-dim state tensor with card ID positions
- Card embedding: 64-dim per card (5000 card vocabulary)
- Non-card features: 132 positions passed through directly
- Card positions: 194 positions → embeddings → concatenated
- Total input dim: 132 + 194×64 = 12,548
- Parameters: ~2.7M (includes embedding table)

### 3.2 Card2Vec Pre-training

**Objectives**:
1. Co-occurrence prediction (skip-gram style)
2. Attribute prediction (cost, attack, health, keywords, faction)

**Training**:
- Embedding dim: 64
- Epochs: 100
- Batch size: 256
- Learning rate: 1e-3

**Results** (qualitative):
- Cards cluster by faction in embedding space
- Similar cards have high cosine similarity (0.6-0.8)
- Example: "Brass Sentinel" similar to "Iron Colossus", "Siege Cannon" (all Argentum constructs)

### 3.3 PPO Training

**Hyperparameters**:
- Timesteps: ~500k-1M
- Num envs: 64 (vectorized)
- Rollout steps: 128
- Learning rate: 3e-4
- GAE lambda: 0.95
- Clip range: 0.2
- Entropy coefficient: 0.01

**Opponent**: GreedyBot (curriculum could be added)

### 3.4 Behavioral Cloning

**Training**:
- Dataset: mcts_10k (900k samples)
- Epochs: 10-50
- Batch size: 512
- Learning rate: 1e-3 (cosine annealing)
- Loss: Cross-entropy (policy) + MSE (value)

**Key Finding**: Early stopping crucial - best performance at epoch 10, overfitting after epoch 30.

### 3.5 AlphaZero Self-Play

**Configuration**:
- MCTS simulations: 100 per move
- Games per iteration: 100
- Training steps per iteration: 100
- Replay buffer: 100k samples
- PUCT constant: 1.5
- Dirichlet noise: α=0.3, ε=0.25

**Fine-tuning from BC**:
- Initialize network from BC checkpoint
- Continue with AlphaZero self-play loop
- Expected: Faster convergence, higher final performance

---

## 4. Results

### 4.1 Win Rates vs GreedyBot

| Method | Timesteps/Samples | Win Rate vs Greedy | Notes |
|--------|-------------------|--------------------|--------------------|
| RandomBot | - | ~15% | Baseline |
| GreedyBot | - | 50% | Self-play |
| **PPO-flat (best)** | 300k | **71.0%** | Best checkpoint |
| **PPO-embedded (best)** | 300k | **65.0%** | Best checkpoint |
| **PPO-argentum (best)** | 300k | **72.0%** | Faction specialist |
| **PPO-symbiote (best)** | 300k | **65.0%** | Faction specialist |
| **PPO-obsidion (best)** | 300k | **62.0%** | Faction specialist |
| BC (epoch 10) | 900k samples | 59% | Early stopping |
| AlphaZero (100 iter) | 10k games | 0% | Cold-start problem |
| AlphaZero (BC init) | 10k games | 0% | Catastrophic forgetting |
| AlphaZero (BC warm-start, short) | 10 games | **64%** | BC ratio 83% |
| AlphaZero (BC warm-start, long) | 600 games | 62%→0% | BC ratio dropped <50% |
| AlphaZero (dual buffer, short) | 15 games | **62%** | BC ratio 70%, stable ✅ |
| AlphaZero (dual buffer, long) | 4,000 games | 2% | BC ratio 70%, still collapses ❌ |

### 4.2 PPO Architecture Comparison (Embedding Study)

**Experiment**: Compare four PPO architectures over 300k timesteps with improvements:
- Higher entropy coefficient (0.02 vs 0.01)
- Best checkpoint saving (saves model at peak eval performance)

| Architecture | Input Dim | Best Win Rate | Final Win Rate | Recovered |
|--------------|-----------|---------------|----------------|-----------|
| **flat** | 326 | **71.0%** @ 262k | 63.5% | +7.5% |
| **embedded** | 1,668 | **65.0%** @ 163k | 51.0% | +14% |
| pretrained | 1,668 | 34.0% @ 262k | 26.0% | +8% |
| pretrained-frozen | 1,668 | 57.0% @ 196k | 33.5% | +23.5% |

**Key Finding**: Best checkpoint saving recovers significant performance lost to policy collapse. The flat architecture achieved the highest single evaluation (71%) when peak performance is captured.

**Policy Collapse Still Occurs**: All architectures show performance degradation after peak:
- `flat`: 71% → 65% → 48% (final 63.5%)
- `embedded`: 65% → 59% → 48% (final 51%)
- `pretrained-frozen`: 57% → 49% → 30% (final 33.5%)

**Entropy Coefficient Impact**: Increasing from 0.01 to 0.02 helped maintain exploration longer, but collapse still occurred. The best checkpoint mechanism is essential for capturing peak performance.

**Bug Fixes Applied**: Two critical bugs were fixed in the embedding implementation:
1. **Pretrained weights not loading**: `_init_weights()` checked a non-existent attribute
2. **Raw card IDs passed as features**: 170 unnormalized card IDs dominated the input

Before fixes: embedded=0%, pretrained=0%. After fixes: embedded=65%, pretrained=34%.

### 4.3 Faction Specialist Results (v2 - with embedded architecture)

| Specialist | Best Win Rate | Final Win Rate | Improvement from v1 |
|------------|---------------|----------------|---------------------|
| **Argentum** | **72.0%** @ 131k | 60.5% | +62% (was 10%) |
| **Symbiote** | **65.0%** @ 65k | 45.0% | +58% (was 7%) |
| **Obsidion** | **62.0%** @ 65k | 16.5% | +60% (was 2%) |

**Key Finding**: Using embedded architecture + best checkpoint saving transforms faction specialists from failures (2-10%) to strong performers (62-72%).

**Argentum achieves highest win rate** (72%) of all PPO models, suggesting defensive playstyle (Guard, Shield) may be easier to learn than aggressive strategies.

**Collapse pattern varies by faction**:
- Argentum: Most stable, maintains 60%+ through most of training
- Symbiote/Obsidion: Peak early (~65k steps) then collapse to 16-45%

### 4.4 AlphaZero From-Scratch Training

**Experiment**: Train AlphaZero from random initialization with 100 iterations.

**Configuration**:
- Iterations: 100
- Games per iteration: 100
- Total self-play games: 10,000
- MCTS simulations: 100 per move
- Training steps per iteration: 100
- Batch size: 256
- Learning rate: 1e-3
- Network: Flat (326 input, 256 hidden, 4 residual blocks)
- Training time: ~14.3 hours (51,450 seconds) on RTX 3090

**Results**:
| Iteration | Win Rate vs Greedy | Loss |
|-----------|-------------------|------|
| 5 | 0% | ~3.5 |
| 50 | 0% | ~2.9 |
| 100 | **0%** | 2.88 |

**Final Evaluation**:
- vs GreedyBot: **0%** win rate
- vs RandomBot: **0%** win rate

**Analysis - The Cold-Start Problem**:

AlphaZero's cold-start problem is severe in Essence Wars:

1. **Bootstrapping failure**: AlphaZero requires MCTS to produce good training signal, but MCTS requires a good value function to guide search. With random initialization, both are terrible.

2. **Sparse rewards**: Games are ~90 moves long with only terminal rewards (+1/-1). Early random networks provide no useful gradient signal for intermediate positions.

3. **Large action space**: With 256 possible actions (many illegal), random exploration rarely finds winning sequences.

4. **Insufficient iterations**: AlphaGo Zero used 4.9M self-play games; we only generated 10k games. The ratio is ~500x less data.

5. **No curriculum**: Starting against random opponents might help the network learn basic heuristics before self-play.

**Comparison to Original AlphaZero**:
| Aspect | AlphaGo Zero | Essence Wars |
|--------|--------------|--------------|
| Self-play games | 4,900,000 | 10,000 |
| Training time | 40 days | 14 hours |
| TPUs | 4 | 1 GPU |
| Compute ratio | ~10,000x more | baseline |

The experiment confirms that AlphaZero-style training from scratch is not viable for this game without significantly more compute or algorithmic improvements.

### 4.5 AlphaZero Fine-tuning from BC

**Experiment**: Initialize AlphaZero with BC checkpoint (59% vs Greedy), continue self-play training.

| Run | Learning Rate | Iter 5 | Iter 10 | Notes |
|-----|---------------|--------|---------|-------|
| 1 | 1e-3 | 0% | 0% | Immediate catastrophic forgetting |
| 2 | 1e-4 | 0% | 0% | Still catastrophic forgetting |

**Key Finding**: Both learning rates resulted in complete loss of BC knowledge. During training, iteration 4 briefly showed ~68% win rate in online evaluation, but all saved checkpoints evaluate to 0%. This suggests:
1. The online evaluation during training may be unreliable
2. MCTS self-play generates out-of-distribution states that confuse the BC-pretrained network
3. Policy gradient updates quickly overwrite the BC initialization

**Potential mitigations** (not yet tested):
- KL divergence penalty to anchor to BC policy
- ~~Mixed replay buffer (BC data + self-play data)~~ **TESTED - works!**
- Even lower learning rate (1e-5) or learning rate warmup
- Freeze early layers during initial fine-tuning

### 4.6 AlphaZero BC Warm-Start (Mixed Replay Buffer)

**Experiment**: Initialize from BC checkpoint + pre-fill replay buffer with BC data before self-play.

#### Quick Test (2 iterations)

**Configuration**:
- BC checkpoint: 59% vs Greedy (epoch 9)
- BC data pre-loaded: 4,489 samples
- Learning rate: 1e-4
- MCTS simulations: 25 per move

**Results**:
| Iteration | Self-Play Games | BC Ratio | Win Rate vs Greedy |
|-----------|----------------|----------|-------------------|
| BC start | 0 | 100% | 59% |
| 1 | 5 | 91% | 40% |
| 2 | 10 | **83%** | **64%** |

**Success!** With 83% BC ratio, the model maintains and improves performance.

#### Extended Test (20 iterations)

**Configuration**:
- BC data pre-loaded: 26,596 samples
- Learning rate: 1e-4
- MCTS simulations: 100 per move
- Games per iteration: 30

**Results**:
| Iteration | Replay Buffer | BC Ratio | Win Rate vs Greedy |
|-----------|--------------|----------|-------------------|
| 1 | 29,653 | 90% | (not evaluated) |
| 5 | 41,351 | **64%** | **62%** ✅ |
| 10 | 56,244 | **47%** | 2% ❌ |
| 15 | 71,195 | 37% | 0% ❌ |
| 20 | 85,557 | 31% | 0% ❌ |

#### Key Finding: BC Ratio Stability Window

The BC warm-start has a **stability window** dependent on maintaining high BC ratio:

| BC Ratio | Performance | Status |
|----------|-------------|--------|
| >70% | Maintains/improves | ✅ Stable |
| 50-70% | Degrades slowly | ⚠️ Warning |
| <50% | Collapses to 0% | ❌ Failed |

**Why performance degrades**:
1. As self-play data accumulates, BC ratio drops
2. Self-play data quality degrades as model drifts
3. Feedback loop: worse model → worse self-play data → worse model
4. Eventually collapses to random-like behavior

**Comparison to Previous Attempts**:
| Approach | Learning Rate | BC Ratio | Result |
|----------|--------------|----------|--------|
| BC → AlphaZero (naive) | 1e-3 | 0% | 0% (immediate forgetting) |
| BC → AlphaZero (lower LR) | 1e-4 | 0% | 0% (immediate forgetting) |
| BC warm-start (short) | 1e-4 | **83%** | **64%** ✅ |
| BC warm-start (long) | 1e-4 | 47%→31% | 62%→0% ❌ |

#### Proposed Solutions

1. **Larger BC buffer**: Load 80-100k BC samples to maintain >60% ratio longer
2. **Fixed sampling ratio**: Sample 70% BC, 30% self-play regardless of buffer composition
3. **Periodic BC refresh**: Re-inject BC samples every N iterations
4. ~~**Two-buffer approach**: Separate BC and self-play buffers with fixed sampling ratio~~ **IMPLEMENTED - see Section 4.7**

### 4.7 AlphaZero BC Warm-Start (Dual Buffer)

**Problem Solved**: The single mixed buffer (Section 4.6) fails because the BC ratio inevitably drops below the stability threshold (~50%) as self-play data accumulates.

**Solution**: Implement a `DualReplayBuffer` that maintains **separate BC and self-play buffers** with a **fixed sampling ratio** (default 70% BC, 30% self-play).

**Implementation**:
```python
class DualReplayBuffer:
    def __init__(self, bc_ratio=0.7, ...):
        self.bc_buffer = deque(maxlen=capacity)      # BC samples (fixed)
        self.selfplay_buffer = deque(maxlen=capacity)  # Self-play samples

    def sample(self, batch_size):
        bc_size = int(batch_size * self.bc_ratio)    # Always 70%
        sp_size = batch_size - bc_size               # Always 30%
        # Sample from respective buffers...
```

**Configuration**:
- BC samples: 4,489 (loaded from BC dataset)
- BC ratio: 70% (fixed throughout training)
- Learning rate: 1e-4
- MCTS simulations: 25 per move

**Results**:
| Iteration | BC Buffer | Self-Play Buffer | Sampling Ratio | Win Rate vs Greedy |
|-----------|-----------|------------------|----------------|-------------------|
| BC start | 4,489 | 0 | 100% BC | 59% |
| 1 | 4,489 | 461 | **70% BC** | 75% |
| 2 | 4,489 | 931 | **70% BC** | 75% |
| 3 | 4,489 | 1,388 | **70% BC** | 55% |
| Final | 4,489 | 1,388 | **70% BC** | **62%** ✅ |

**Key Findings**:

1. **Fixed ratio prevents collapse**: Unlike the single buffer (which dropped to 0% at iter 10), the dual buffer maintains 62% at equivalent training progression.

2. **Sampling ratio always 70%**: Regardless of self-play accumulation, batches always contain 70% BC and 30% self-play samples.

3. **Variance is expected**: Win rates fluctuate (75%→55%→62%) but remain stable above the target (60%).

**Comparison: Single Buffer vs Dual Buffer**

| Metric | Single Buffer (Sec 4.6) | Dual Buffer (Sec 4.7) |
|--------|-------------------------|----------------------|
| BC ratio at iter 3 | 64% (dropping) | **70% (fixed)** |
| BC ratio at iter 10 | 47% (critical) | **70% (fixed)** |
| Win rate at iter 5 | 62% | N/A |
| Win rate at iter 10 | **2% (collapsed)** | **62% (stable)** ✅ |
| Final win rate | **0%** | **62%** ✅ |

**Short-term Conclusion**: The dual buffer approach solves the immediate catastrophic forgetting problem for short training runs.

#### Extended Test (40+ iterations) - Dual Buffer Failure at Scale

**Configuration**:
- BC samples: 89,020 (loaded from BC dataset)
- BC ratio: 70% (fixed throughout training)
- Learning rate: 1e-4
- MCTS simulations: 100 per move
- Games per iteration: 100

**Results**:
| Iteration | Self-Play Buffer | Sampling Ratio | Win Rate vs Greedy | Loss |
|-----------|------------------|----------------|-------------------|------|
| 5 | 500 | 70% BC | 4% | 2.68 |
| 10 | 1,000 | 70% BC | **20%** ← Peak | 2.55 |
| 15 | 1,500 | 70% BC | 0% | 2.50 |
| 20 | 2,000 | 70% BC | 0% | 2.46 |
| 30 | 3,000 | 70% BC | 0% | 2.39 |
| 40 | 4,000 | 70% BC | 2% | 2.35 |

**Critical Finding: Loss-Performance Paradox**

The loss decreased steadily (2.97 → 2.35) while performance collapsed (20% → 2%). This reveals:

1. **Distribution shift**: The model learns to fit the training distribution (70% BC + 30% self-play), but the self-play data comes from an increasingly degraded model.

2. **Corrupted self-play signal**: Even at 30%, the self-play data is harmful because:
   - Generated by a weak model (starting at 59%, degrading to near-random)
   - MCTS with 100 sims isn't strong enough to compensate
   - Creates feedback loop: bad model → bad data → worse model

3. **BC anchoring insufficient**: 70% BC ratio slows the collapse but doesn't prevent it. The 30% bad signal accumulates over time.

**Comparison: Short vs Long Dual Buffer Training**

| Metric | Short (3 iter) | Long (40 iter) |
|--------|----------------|----------------|
| Self-play games | 15 | 4,000 |
| Peak win rate | 75% | 20% |
| Final win rate | **62%** ✅ | **2%** ❌ |
| Loss trend | Stable | Decreasing (bad sign) |

### 4.8 AlphaZero: Compute Requirements Analysis

**Why AlphaZero Fails at Small Scale**

| Aspect | AlphaGo Zero | Our Setup | Ratio |
|--------|--------------|-----------|-------|
| Self-play games | 4,900,000 | 4,000 | **1,225x less** |
| MCTS sims/move | 1,600 | 100 | **16x less** |
| Training time | 40 days | 8 hours | **120x less** |
| Hardware | 5,000 TPUs + 64 GPUs | 1 GPU | **~5,000x less** |
| Parallel self-play | ~25,000 games/batch | 1 game | **25,000x less** |

**The Bootstrapping Problem**

AlphaZero requires:
1. **High-quality MCTS** (800+ sims) to generate good training signal
2. **Massive volume** to overcome noise in early random play
3. **Rapid iteration** via parallel self-play

With limited compute:
- MCTS quality is low (100 sims ≈ weak amateur)
- Volume is insufficient (4k games vs 5M)
- No parallelism means slow iteration

**Conclusion**: AlphaZero-style self-play from scratch requires ~1,000-10,000x more compute than available to independent researchers. Alternative approaches are needed.

**Usage** (for reference):
```bash
# AlphaZero with dual buffer BC warm-start
uv run python python/scripts/train_alphazero.py \
    --iterations 100 \
    --load models/bc_mcts_10k_best.pt \
    --bc-data data/datasets/mcts_10k_sims100_*.jsonl.gz \
    --bc-ratio 0.7  # Default: 70% BC, 30% self-play
```

### 4.9 Key Observations

1. **Best checkpoint saving is essential**: Recovers 7-62% performance lost to policy collapse
2. **PPO achieves 72% win rate**: Argentum specialist is our best model, beating BC (59%)
3. **Faction specialists work**: With proper setup, specialists (62-72%) outperform generalists (65-71%)
4. **Flat vs Embedded comparable**: Both achieve 65-71% with best checkpoint; embeddings not strictly necessary
5. **Pretrained Card2Vec underperforms**: Co-occurrence objective doesn't transfer well to game-winning objective
6. **Policy collapse is ubiquitous**: All architectures show strong early performance followed by collapse
7. **Early stopping / best checkpoint critical**: For both BC and PPO, more training often hurts
8. **AlphaZero from scratch fails**: Cold-start problem + insufficient compute (10k vs 5M games) = 0% win rate
9. **BC → AlphaZero catastrophic forgetting**: Fine-tuning BC model with AlphaZero immediately destroys learned policy
10. **BC warm-start has stability window**: Works when BC ratio >70% (64% win rate), collapses when ratio <50%
11. **Dual buffer delays but doesn't prevent collapse**: Fixed 70% BC ratio maintains 62% for ~3 iterations, but still collapses to 2% by iteration 40
12. **Loss-performance paradox**: In AlphaZero fine-tuning, loss decreases while performance collapses - classic distribution shift
13. **Self-play from weak models is harmful**: Even 30% self-play data corrupts training when generated by degrading model
14. **AlphaZero requires ~1000x more compute**: 5M games vs 4k games, 1600 sims vs 100 sims - not viable for independent researchers
15. **Training is fast**: Full roster (7 agents × 300k steps) completes in ~6 minutes on RTX 3090

### 4.9 Training Efficiency

| Method | Wall Clock Time | Hardware | Throughput |
|--------|-----------------|----------|------------|
| PPO (500k steps) | ~30 min | RTX 3090 | ~268k steps/sec |
| BC (50 epochs) | ~12 min | RTX 3090 | ~75k samples/sec |
| AlphaZero (100 iter) | ~14.3 hours | RTX 3090 | ~12 games/min |

---

## 5. Technical Contributions

### 5.1 Memory-Safe Data Loading

**Problem**: Large datasets (100k games = ~31 GB) cause OOM crashes on typical systems.

**Solutions Implemented**:

1. **Reservoir Sampling** (Card2Vec): Bound pair collection to 500k samples
2. **ChunkedMCTSDataset**: Stream data in fixed-size chunks with in-chunk shuffling
3. **Bounded Replay Buffer**: `deque(maxlen=N)` for AlphaZero

**Memory Estimates**:
- 10k games: ~3.1 GB (safe for 16 GB RAM)
- 100k games: ~31 GB (requires streaming)

### 5.2 Bug Fixes Discovered

1. **AlphaZero training loop**: Training/evaluation code was outside the iteration loop (indentation bug), causing 0% win rate despite 22 hours of training.

2. **Embedding pretrained weights not loading** (`embeddings.py`): The `_init_weights()` method checked `hasattr(self.card_embedding, '_from_pretrained')`, but PyTorch's `nn.Embedding.from_pretrained()` doesn't set this attribute. Result: pretrained embeddings were always overwritten with random initialization. Fix: Added explicit `_using_pretrained_embeds` flag.

3. **Raw card IDs passed as features** (`embeddings.py`): When `include_embed_section=False`, the embed section (positions 156-325) containing raw card IDs (values 1000-4074) was added back to `non_card_positions`. These 170 unnormalized values (up to 4074) dominated the network input (other features normalized to [-1, 1]), completely preventing learning. Fix: Exclude embed section entirely when not embedding it.

**Impact of embedding bugs**: Before fixes, both `embedded` and `pretrained` modes achieved 0% win rate. After fixes: `embedded`=60%, `pretrained`=45.5%.

---

## 6. Planned Experiments

### 6.1 Embedding Comparison
- [x] PPO (flat) vs PPO (embedded) vs PPO (pretrained) sample efficiency - **Done: embedded wins (60% vs 10.5%)**
- [ ] Same comparison for AlphaZero
- [ ] Investigate why pretrained underperforms end-to-end (Card2Vec trained on different objective)

### 6.2 AlphaZero Fine-tuning - CONCLUDED
- [x] BC → AlphaZero fine-tuning (initialize from 59% BC model) - **Failed: catastrophic forgetting**
- [x] Test mitigation strategies (dual buffer) - **Works short-term (62%), fails long-term (2%)**
- [x] Compare to training from scratch - **Both fail: insufficient compute for AlphaZero approach**
- [ ] ~~Try AlphaZero with embedded architecture~~ - **Deprioritized: fundamental compute problem**

**Conclusion**: AlphaZero requires ~1000x more compute than available. Pivoting to alternative approaches (see Section 6.6).

### 6.3 Faction Specialists
- [x] Train PPO specialists per faction - **v1: collapsed (2-10%), v2: 62-72% with embedded+best checkpoint**
- [x] Compare generalist vs specialist performance - **Argentum specialist (72%) beats all generalists (65-71%)**
- [ ] Investigate deck cycling impact on stability
- [x] Try training specialists with embedded architecture - **Done: solves collapse (62-72%)**

### 6.4 Policy Collapse Investigation
- [x] Higher entropy coefficient (0.02 vs 0.01) - **Implemented, helps but collapse still occurs**
- [ ] Lower learning rate (1e-4 vs current 3e-4)
- [ ] Entropy bonus scheduling (anneal from high to low)
- [x] Early stopping based on eval performance (save best checkpoint) - **Implemented: recovers 7-62% perf**
- [ ] Separate policy and value networks

### 6.5 Scaling Laws
- [ ] Performance vs compute budget
- [ ] Performance vs dataset size (BC)
- [ ] Performance vs MCTS simulations (AlphaZero)

### 6.6 Research Tracks Forward (Post-AlphaZero)

Given AlphaZero's compute requirements, we propose four alternative research tracks:

#### Track A: Expert Iteration (ExIt)
**Concept**: Iterative improvement via BC → MCTS → new data → BC cycle

**Approach**:
1. Train BC model on MCTS data (current: 59%)
2. Use BC model to guide MCTS (as value/policy prior)
3. Generate new games with BC-guided MCTS
4. Train new BC model on improved data
5. Repeat

**Why it might work**:
- No online self-play during training (avoids distribution shift)
- MCTS quality improves as BC improves
- Used successfully in poker AI research

**Compute**: Moderate (data generation is expensive, but training is cheap)

#### Track B: MCTS-Augmented Inference
**Concept**: Use trained BC/PPO model as value function inside MCTS at play time

**Approach**:
1. Keep our best model (PPO-Argentum at 72%)
2. At inference time, run MCTS using model for:
   - Policy prior (guide search)
   - Value estimation (evaluate leaf nodes)
3. Play the MCTS-selected action

**Why it might work**:
- Combines learned intuition (neural net) with search (MCTS)
- No additional training required
- Similar to how AlphaZero actually plays (not just the raw network)

**Compute**: Minimal training, moderate inference

#### Track C: Offline RL (DQN, CQL, IQL)
**Concept**: Learn from fixed dataset without environment interaction

**Approach**:
1. Use existing MCTS dataset (10k-100k games)
2. Apply offline RL algorithms:
   - **DQN**: Q-learning on transitions
   - **CQL**: Conservative Q-Learning (penalizes OOD actions)
   - **IQL**: Implicit Q-Learning (avoids policy evaluation)

**Why it might work**:
- Algorithms designed for learning from fixed data
- No distribution shift from self-play
- Can leverage our large MCTS dataset

**Compute**: Similar to BC (dataset-based training)

#### Track D: PPO Improvements
**Concept**: Push PPO further with advanced techniques

**Approaches**:
1. **Curriculum Learning**: Start vs RandomBot, progress to GreedyBot, then self-play
2. **Population-Based Training (PBT)**: Evolve hyperparameters during training
3. **League Play**: Train against diverse opponents (pool of past checkpoints)
4. **Reward Shaping**: Intermediate rewards for good plays (not just win/lose)

**Why it might work**:
- PPO already achieves 72% (our best result)
- Room for improvement with better training setup
- Well-understood algorithm with many enhancement options

**Compute**: Moderate (more training runs, but each is fast)

#### Track Priority Recommendation

| Track | Potential | Effort | Priority |
|-------|-----------|--------|----------|
| **B: MCTS-Augmented Inference** | High | Low | ⭐ **1st** |
| **A: Expert Iteration** | High | Medium | ⭐ **2nd** |
| **D: PPO Improvements** | Medium | Medium | 3rd |
| **C: Offline RL** | Medium | High | 4th |

**Rationale**: Track B requires no new training and could immediately boost our 72% model. Track A addresses the core problem (improving BC beyond 59%) with proven methodology.

---

## 7. Code & Reproducibility

### 7.1 Repository Structure

```
essence-wars/
├── crates/cardgame/          # Rust game engine
├── python/
│   ├── essence_wars/         # Python bindings
│   │   ├── agents/           # PPO, AlphaZero, Card2Vec
│   │   └── data/             # Dataset loaders
│   └── scripts/              # Training scripts
├── data/
│   ├── datasets/             # MCTS game data
│   └── cards/                # Card definitions
└── models/                   # Trained checkpoints
```

### 7.2 Key Commands

```bash
# PPO Training
uv run python python/scripts/train_ppo.py --timesteps 500000

# Behavioral Cloning
uv run python python/scripts/train_behavioral_cloning.py \
    --dataset data/datasets/mcts_10k_*.jsonl.gz --epochs 10

# Card2Vec Pre-training
uv run python python/scripts/train_card2vec.py \
    --dataset data/datasets/mcts_10k_*.jsonl.gz --epochs 100

# AlphaZero (with BC initialization)
uv run python python/scripts/train_alphazero.py \
    --iterations 100 --load models/bc_mcts_10k_best.pt
```

### 7.3 Datasets Available

- HuggingFace: `ChristianWissworWo/essence-wars-mcts-games`
- Local: `data/datasets/mcts_{1k,10k,100k}_sims100_*.jsonl.gz`

---

## 8. Next Steps

1. **Run embedding comparison experiment** (flat vs embedded vs pretrained)
2. ~~AlphaZero fine-tuning from BC~~ - Done, catastrophic forgetting observed
3. **Cloud GPU training** for longer AlphaZero runs (Modal/HuggingFace)
   - Train AlphaZero from scratch for 100+ iterations
   - Test mitigation strategies for BC fine-tuning
4. **Write introduction and related work sections**
5. **Create figures**: learning curves, embedding visualizations, architecture diagrams

---

## Appendix A: Model Checkpoints

### PPO v2 Roster (Final)

| Model | Path | Best Win Rate |
|-------|------|---------------|
| **PPO-Argentum** | `experiments/ppo/20260119_151841_argentum_embedded/best_model.pt` | **72.0%** |
| **PPO-Flat** | `experiments/ppo/20260119_151612_flat/best_model.pt` | **71.0%** |
| **PPO-Embedded** | `experiments/ppo/20260119_151512_embedded/best_model.pt` | 65.0% |
| **PPO-Symbiote** | `experiments/ppo/20260119_151945_symbiote_embedded/best_model.pt` | 65.0% |
| **PPO-Obsidion** | `experiments/ppo/20260119_152041_obsidion_embedded/best_model.pt` | 62.0% |

### Other Models

| Model | Path | Performance |
|-------|------|-------------|
| BC (best) | `models/bc_mcts_10k_best.pt` | 59% vs Greedy |
| Card2Vec | `models/card2vec_20260119_120507.pt` | Embeddings (64-dim) |

### HuggingFace

| Model | URL |
|-------|-----|
| PPO-Argentum | https://huggingface.co/Chris-Essence-Wars/ppo-argentum |
| PPO-Flat | https://huggingface.co/Chris-Essence-Wars/ppo-flat |
| PPO-Embedded | https://huggingface.co/Chris-Essence-Wars/ppo-embedded |
| PPO-Symbiote | https://huggingface.co/Chris-Essence-Wars/ppo-symbiote |
| PPO-Obsidion | https://huggingface.co/Chris-Essence-Wars/ppo-obsidion |

---

## Appendix B: Hardware

- **GPU**: NVIDIA RTX 3090 (24 GB VRAM)
- **CPU**: [TBD]
- **RAM**: 16 GB (WSL2 environment)
- **OS**: Windows 11 + WSL2 (Ubuntu)

---

## Notes & Ideas

- BC seems very efficient for this game (59% in 12 min) but fine-tuning fails catastrophically
- The "overfitting" in BC might be due to distributional shift (MCTS vs neural net play styles)
- AlphaZero fine-tuning from BC causes catastrophic forgetting even at LR=1e-4
  - The BC policy may be fragile to the self-play distribution shift
  - Consider freezing feature extractor layers during initial fine-tuning
  - KL penalty to original BC policy might help preserve knowledge
- AlphaZero might need curriculum: start with fewer sims, increase over time
- Consider adding MCTS at evaluation time for BC/PPO models (like MuZero)
- **Key insight**: BC provides best results with minimal compute, but doesn't improve with more training
