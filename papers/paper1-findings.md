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

| Method | Timesteps/Samples | Win Rate vs Greedy | Win Rate vs Random |
|--------|-------------------|--------------------|--------------------|
| RandomBot | - | ~15% | 50% |
| PPO (flat) | 295k | **56%** | 41.5% |
| PPO (flat) | 1M | 31.5% | 27% |
| BC (epoch 10) | 900k samples | **59%** | 49% |
| BC (epoch 50) | 900k samples | 48% | 53% |
| AlphaZero (5 iter) | 9k samples | 0% | 0% |
| AlphaZero fine-tune (LR=1e-3) | BC + 10 iter | 0% | - |
| AlphaZero fine-tune (LR=1e-4) | BC + 10 iter | 0% | - |

### 4.2 AlphaZero Fine-tuning Results

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

### 4.3 Key Observations

1. **BC outperforms PPO** at similar compute: 59% vs 56% win rate
2. **Early stopping critical for BC**: Performance drops from 59% → 48% between epoch 10 and 50
3. **PPO can overfit**: 1M timesteps performed worse than 295k timesteps
4. **AlphaZero needs more iterations**: 5 iterations insufficient for learning
5. **Win rate vs Random not correlated with vs Greedy**: Different skills required

### 4.3 Training Efficiency

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

---

## 6. Planned Experiments

### 6.1 Embedding Comparison
- [ ] PPO (flat) vs PPO (embedded) vs PPO (pretrained) sample efficiency
- [ ] Same comparison for AlphaZero

### 6.2 AlphaZero Fine-tuning
- [x] BC → AlphaZero fine-tuning (initialize from 59% BC model) - **Failed: catastrophic forgetting**
- [ ] Test mitigation strategies (KL penalty, mixed replay, lower LR)
- [ ] Compare to training from scratch (longer run)

### 6.3 Faction Specialists
- [ ] Train PPO specialists per faction
- [ ] Compare generalist vs specialist performance

### 6.4 Scaling Laws
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

| Model | Path | Performance |
|-------|------|-------------|
| PPO (best) | `experiments/ppo/20260119_074234/final_model.pt` | 56% vs Greedy |
| BC (best) | `models/bc_mcts_10k_best.pt` | 59% vs Greedy |
| Card2Vec | `models/card2vec_20260119_120507.pt` | Embeddings (64-dim) |

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
