# Paper 1: Neural Network Training for Essence Wars

**Working Title**: "Learning to Play Essence Wars: A Comparison of Reinforcement Learning Approaches for Deterministic Card Games"

**Status**: Data Collection Phase
**Last Updated**: 2026-01-19

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
| AlphaZero (5 iter) | 9k samples | 0% | Insufficient iterations |

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

### 4.4 AlphaZero Fine-tuning Results

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
- Mixed replay buffer (BC data + self-play data)
- Even lower learning rate (1e-5) or learning rate warmup
- Freeze early layers during initial fine-tuning

### 4.5 Key Observations

1. **Best checkpoint saving is essential**: Recovers 7-62% performance lost to policy collapse
2. **PPO achieves 72% win rate**: Argentum specialist is our best model, beating BC (59%)
3. **Faction specialists work**: With proper setup, specialists (62-72%) outperform generalists (65-71%)
4. **Flat vs Embedded comparable**: Both achieve 65-71% with best checkpoint; embeddings not strictly necessary
5. **Pretrained Card2Vec underperforms**: Co-occurrence objective doesn't transfer well to game-winning objective
6. **Policy collapse is ubiquitous**: All architectures show strong early performance followed by collapse
7. **Early stopping / best checkpoint critical**: For both BC and PPO, more training often hurts
8. **Training is fast**: Full roster (7 agents × 300k steps) completes in ~6 minutes on RTX 3090

### 4.6 Training Efficiency

| Method | Wall Clock Time | Hardware | Throughput |
|--------|-----------------|----------|------------|
| PPO (500k steps) | ~30 min | RTX 3090 | ~268k steps/sec |
| BC (50 epochs) | ~12 min | RTX 3090 | ~75k samples/sec |
| AlphaZero (5 iter) | ~4.5 min | RTX 3090 | ~20 games/min |

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

### 6.2 AlphaZero Fine-tuning
- [x] BC → AlphaZero fine-tuning (initialize from 59% BC model) - **Failed: catastrophic forgetting**
- [ ] Test mitigation strategies (KL penalty, mixed replay, lower LR)
- [ ] Compare to training from scratch (longer run)
- [ ] Try AlphaZero with embedded architecture

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
