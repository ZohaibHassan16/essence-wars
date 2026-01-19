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
| PPO (flat) | 300k | 10.5% | 6.5% |
| PPO (embedded) | 300k | **60%** | 36% |
| PPO (pretrained) | 300k | 45.5% | 38.5% |
| PPO (pretrained-frozen) | 300k | 17.5% | 7.5% |
| BC (epoch 10) | 900k samples | **59%** | 49% |
| BC (epoch 50) | 900k samples | 48% | 53% |
| AlphaZero (5 iter) | 9k samples | 0% | 0% |
| AlphaZero fine-tune (LR=1e-3) | BC + 10 iter | 0% | - |
| AlphaZero fine-tune (LR=1e-4) | BC + 10 iter | 0% | - |

### 4.2 PPO Architecture Comparison (Embedding Study)

**Experiment**: Compare four PPO architectures over 300k timesteps each.

| Architecture | Input Dim | Peak Win Rate | Final Win Rate | Stability |
|--------------|-----------|---------------|----------------|-----------|
| flat | 326 | 63% @ 32k | 10.5% | ❌ Collapsed |
| embedded | 1,668 | 65% @ 163k | **60%** | ✅ Stable |
| pretrained | 1,668 | 54% @ 163k | 45.5% | ✅ Stable |
| pretrained-frozen | 1,668 | 26% @ 229k | 17.5% | ❌ Slow |

**Key Finding**: Learned embeddings dramatically outperform flat input representation.

**Policy Collapse Pattern**: All agents except `embedded` showed performance collapse:
- `flat`: 63% → 43% → 7% → 10.5%
- `pretrained`: peaked at 54%, drifted to 45.5%
- `pretrained-frozen`: never exceeded 26%

**Entropy Analysis**: Entropy dropped from ~1.8 to ~1.0-1.1 across all runs, but `embedded` maintained stable performance while `flat` collapsed. The larger input dimension (1,668 vs 326) may provide implicit regularization.

**Bug Fixes Required**: Two critical bugs were discovered in the embedding implementation:
1. **Pretrained weights not loading**: `_init_weights()` checked a non-existent attribute, causing pretrained embeddings to be overwritten with random initialization.
2. **Raw card IDs passed as features**: When `include_embed_section=False`, 170 raw card ID values (1000-4074) were incorrectly passed to the network as unnormalized features, completely breaking learning.

Before fixes: embedded=0%, pretrained=0%. After fixes: embedded=60%, pretrained=45.5%.

### 4.3 Faction Specialist Results

| Specialist | Final Win Rate | Peak Win Rate | Notes |
|------------|----------------|---------------|-------|
| Argentum | 10% | 27% @ 163k | Collapsed |
| Symbiote | 7% | 62% @ 65k | Strong start, collapsed |
| Obsidion | 2% | 20% @ 65k | Collapsed |

**Key Finding**: Faction specialists suffer from severe policy collapse despite early promising performance. The deck cycling (changing player/opponent decks every 25k steps) may destabilize learning.

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

1. **Embeddings are crucial for PPO**: Learned embeddings (60%) dramatically outperform flat input (10.5%)
2. **End-to-end embeddings beat pretrained**: Learning embeddings jointly with policy (60%) > Card2Vec pretrained (45.5%)
3. **Freezing embeddings hurts performance**: Frozen pretrained (17.5%) << fine-tuned pretrained (45.5%)
4. **Policy collapse is a major issue**: Most architectures show strong early performance followed by collapse
5. **BC still competitive**: BC (59%) matches PPO-embedded (60%) with potentially less compute
6. **Early stopping critical**: For both BC and PPO, more training often hurts
7. **Win rate vs Random not correlated with vs Greedy**: Different skills required

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
- [x] Train PPO specialists per faction - **Done: all collapsed (2-10% final)**
- [x] Compare generalist vs specialist performance - **Generalist wins due to specialist collapse**
- [ ] Investigate deck cycling impact on stability
- [ ] Try training specialists with embedded architecture

### 6.4 Policy Collapse Investigation
- [ ] Higher entropy coefficient (0.02-0.05 vs current 0.01)
- [ ] Lower learning rate (1e-4 vs current 3e-4)
- [ ] Entropy bonus scheduling (anneal from high to low)
- [ ] Early stopping based on eval performance (save best checkpoint)
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

| Model | Path | Performance |
|-------|------|-------------|
| PPO-embedded (best) | `experiments/ppo/20260119_144555_embedded/final_model.pt` | **60% vs Greedy** |
| PPO-pretrained | `experiments/ppo/20260119_144650_embedded_pretrained/final_model.pt` | 45.5% vs Greedy |
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
