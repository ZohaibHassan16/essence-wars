# Phase 3.5: Research Platform Infrastructure

**Mission**: Transform Essence Wars from "a game with ML agents" into "the go-to benchmark for card game AI research" by providing datasets, benchmarks, tutorials, and pre-trained models.

**Motivation**: AlphaZero self-play takes 22-30 hours on consumer hardware due to sequential Python game generation. Pre-generated Rust datasets + behavioral cloning achieves similar results in 2-4 hours with 80%+ GPU utilization.

---

## The Platform Vision

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Essence Wars Research Platform                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│   📊 Datasets              🏆 Benchmarks           📓 Tutorials              │
│   ├── mcts_1M_games       ├── vs_random           ├── 01_quickstart         │
│   ├── balanced_matchups   ├── vs_greedy           ├── 02_mcts_tuning        │
│   └── human_play          ├── vs_mcts100          ├── 03_custom_decks       │
│                           ├── elo_rating          └── 04_self_play          │
│                           └── transfer_eval                                  │
│                                                                               │
│   🤗 Model Zoo (Huggingface)                                                 │
│   ├── essence-wars/mcts-baseline                                             │
│   ├── essence-wars/ppo-generalist                                            │
│   └── essence-wars/alphazero-v1                                              │
│                                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Why This Matters

| Problem | Solution | Impact |
|---------|----------|--------|
| Data generation is slow (Python) | Rust MCTS dataset generator | 10-50x faster |
| Researchers don't want to generate data | Pre-built datasets on Huggingface | Instant start |
| No standard evaluation metrics | Benchmark API with Elo | Fair paper comparisons |
| High barrier to entry | Tutorial notebooks | 15-min onboarding |
| Can't reproduce results | Model zoo with configs | Exact reproducibility |

---

## Milestone M5.5A: Dataset Generation Infrastructure

**Goal**: Generate high-quality MCTS vs MCTS datasets in Rust (10-50x faster than Python).

### Binary: `generate-dataset`

```bash
# Generate 100k MCTS vs MCTS games (parallelized)
cargo run --release --bin generate-dataset -- \
  --games 100000 \
  --bot1 mcts --bot2 mcts \
  --sims 100 \
  --output data/datasets/mcts_100k.jsonl \
  --parallel 16

# Generate balanced deck matchups (all 36 combinations)
cargo run --release --bin generate-dataset -- \
  --games 10000 \
  --mode round-robin \
  --sims 100 \
  --output data/datasets/balanced_matchups.jsonl
```

### Output Format (JSONL)

```json
{
  "game_id": "g_001_seed_42",
  "deck1": "argentum_control",
  "deck2": "symbiote_aggro",
  "winner": 0,
  "moves": [
    {
      "turn": 1,
      "player": 0,
      "state_tensor": [0.03, 0.0, ...],  // 326 floats
      "action_mask": [1.0, 0.0, ...],     // 256 floats
      "action": 42,
      "mcts_policy": [0.0, 0.0, ..., 0.85, ...],  // 256 floats (visit counts)
      "mcts_value": 0.23
    },
    ...
  ],
  "metadata": {
    "total_turns": 24,
    "bot1_sims": 100,
    "bot2_sims": 100,
    "seed": 42
  }
}
```

### Dataset Registry

```
data/datasets/
├── mcts_self_play/
│   ├── mcts_100k_sims50.jsonl.gz     # Quick training (100k games, 50 sims)
│   ├── mcts_1M_sims100.jsonl.gz      # Full training (1M games, 100 sims)
│   └── metadata.json                  # Statistics, deck distributions
├── balanced_matchups/
│   ├── round_robin_10k_per_matchup.jsonl.gz
│   └── win_rate_matrix.csv
└── README.md
```

### Deliverables

- [x] `src/bin/generate_dataset.rs` - Rust binary for fast dataset generation
- [x] JSONL output format with state tensors, actions, MCTS policies
- [x] Parallel generation using rayon
- [x] Round-robin mode for balanced deck matchups
- [x] Compression support (.gz)
- [x] Progress bar and ETA

---

## Milestone M5.5B: Behavioral Cloning Training

**Goal**: Train networks on pre-generated MCTS data (2-4 hours vs 22-30 hours for AlphaZero).

### Training Script

```bash
# Train network to imitate MCTS policy + predict value
uv run python python/scripts/train_behavioral_cloning.py \
  --dataset data/datasets/mcts_100k.jsonl.gz \
  --epochs 50 \
  --batch-size 512 \
  --output models/bc_mcts_100k.pt \
  --tensorboard

# Expected: 80%+ GPU utilization, 2-4 hours for 50 epochs on 100k games
```

### PyTorch Dataset

```python
class MCTSDataset(torch.utils.data.Dataset):
    """
    Load pre-generated MCTS games for behavioral cloning.

    Each sample: (state_tensor, action_mask, mcts_policy, game_outcome)
    """

    def __init__(self, path: str, normalize: bool = True):
        self.samples = self._load_jsonl(path)
        self.normalizer = RunningMeanStd((326,)) if normalize else None

    def __getitem__(self, idx):
        sample = self.samples[idx]
        return {
            "obs": sample["state_tensor"],
            "mask": sample["action_mask"],
            "policy_target": sample["mcts_policy"],  # From MCTS visit counts
            "value_target": sample["game_outcome"],   # +1/-1 from winner
        }
```

### Loss Function

```python
def bc_loss(policy_logits, value_pred, policy_target, value_target):
    # Policy: cross-entropy with MCTS policy
    policy_loss = F.cross_entropy(policy_logits, policy_target)

    # Value: MSE with game outcome
    value_loss = F.mse_loss(value_pred, value_target)

    return policy_loss + value_loss
```

### Deliverables

- [x] `python/essence_wars/data/dataset.py` - MCTSDataset class
- [x] `python/scripts/train_behavioral_cloning.py` - Training script
- [x] Support for .jsonl and .jsonl.gz formats
- [x] Data augmentation (player perspective flipping via `player_perspective` param)
- [ ] Integration with existing observation normalizer (deferred - BC script has own normalization)
- [x] TensorBoard logging
- [x] `--load` flag added to AlphaZero for fine-tuning from BC models

---

## Milestone M5.5C: Benchmark API

**Goal**: Standardized evaluation API for fair comparisons.

### Usage

```python
from essence_wars.benchmark import EssenceWarsBenchmark

benchmark = EssenceWarsBenchmark()

# Standard evaluation suite
results = benchmark.evaluate(my_agent)
print(results)
# {
#   "win_rate_vs_random": 0.98,
#   "win_rate_vs_greedy": 0.67,
#   "win_rate_vs_mcts50": 0.52,
#   "win_rate_vs_mcts100": 0.43,
#   "elo_rating": 1650,
#   "games_played": 1000,
#   "avg_game_length": 22.3,
#   "avg_decision_time_ms": 15.2
# }

# Transfer learning evaluation
transfer = benchmark.evaluate_transfer(
    my_agent,
    train_mode="essence_duel",
    test_mode="attrition"
)

# Cross-faction generalization
generalization = benchmark.evaluate_generalization(
    my_agent,
    train_factions=["argentum", "symbiote"],
    test_factions=["obsidion"]
)

# Deck-specific performance
deck_results = benchmark.evaluate_per_deck(my_agent)
# Returns win rates for each of 6 decks
```

### Agent Interface

```python
class BenchmarkAgent(Protocol):
    """Interface for agents to be benchmarked."""

    def select_action(
        self,
        observation: np.ndarray,  # (326,)
        action_mask: np.ndarray,  # (256,)
    ) -> int:
        """Return action index [0, 255]."""
        ...

    def reset(self) -> None:
        """Reset agent state for new game."""
        ...
```

### Elo Rating System

```python
class EloTracker:
    """Track Elo ratings across agents."""

    def __init__(self, k_factor: float = 32, initial_rating: float = 1500):
        self.ratings = {}

    def update(self, agent1: str, agent2: str, winner: int):
        """Update ratings after a match."""
        ...

    def get_rating(self, agent: str) -> float:
        """Get current Elo rating."""
        ...
```

### Deliverables

- [x] `python/essence_wars/benchmark/api.py` - EssenceWarsBenchmark class
- [x] `python/essence_wars/benchmark/elo.py` - Elo rating tracker
- [x] `python/essence_wars/benchmark/metrics.py` - Standard metrics
- [x] Baseline evaluations (random, greedy, mcts50, mcts100)
- [ ] Transfer and generalization evaluation modes (deferred to Phase 4)
- [x] JSON export for results

---

## Milestone M5.5D: Tutorial Notebooks ✅ [COMPLETE]

**Goal**: Lower barrier to entry with comprehensive Jupyter tutorials.

### Notebook Suite

```
notebooks/
├── 01_quickstart.ipynb          # Install → Train PPO → Evaluate (15 min)
├── 02_environment_basics.ipynb  # Understanding observations, actions, rewards
├── 03_mcts_tuning.ipynb         # Tune MCTS weights with CMA-ES
├── 04_custom_decks.ipynb        # Create and test your own deck
├── 05_behavioral_cloning.ipynb  # Train on MCTS dataset
├── 06_ppo_training.ipynb        # Full PPO training walkthrough
├── 07_alphazero_intro.ipynb     # AlphaZero concepts and mini-training
├── 08_benchmark_evaluation.ipynb # Use benchmark API
└── 09_research_ideas.ipynb      # Open questions and project ideas
```

### 01_quickstart.ipynb Outline

```markdown
# Essence Wars Quickstart (15 minutes)

## 1. Installation (2 min)
pip install essence-wars[train]

## 2. Play a Game (3 min)
- Create environment
- Take random actions
- Observe rewards

## 3. Train PPO Agent (5 min)
- 50k timesteps (~2 min training)
- Watch TensorBoard

## 4. Evaluate Against Greedy (2 min)
- Run benchmark
- See win rate

## 5. Next Steps (3 min)
- Links to other tutorials
- Research ideas
```

### Deliverables

- [x] `notebooks/01_quickstart.ipynb` - Quickstart with game basics, visualization, built-in agents
- [x] `notebooks/02_environment.ipynb` - Environment API deep dive (PyGame, PyParallelGames)
- [x] `notebooks/03_dataset_exploration.ipynb` - Dataset loading and analysis
- [x] `notebooks/04_behavioral_cloning.ipynb` - Train neural network on MCTS data
- [x] `notebooks/05_alphazero_training.ipynb` - Self-play training with MCTS
- [x] All notebooks tested and runnable (Colab-compatible)

---

## Milestone M5.5E: Huggingface Integration ✅ [COMPLETE]

**Goal**: Host datasets and models on Huggingface for easy access.

### Dataset Hosting

```python
from essence_wars.hub import download_dataset
from essence_wars.data import MCTSDataset

# Download pre-generated MCTS dataset from Huggingface
path = download_dataset("mcts-self-play")  # Short name
# or: path = download_dataset("essence-wars/mcts-100k")  # Full repo ID

# Load for training
dataset = MCTSDataset(path)
print(f"Loaded {len(dataset)} samples")

# Iterate
for sample in dataset:
    obs = sample["obs"]  # (326,) tensor
    policy = sample["policy_target"]  # (256,) tensor
    value = sample["value_target"]  # scalar
```

### Model Hosting

```python
from essence_wars.hub import load_pretrained

# Load pre-trained models from Huggingface
agent = load_pretrained("ppo-generalist")  # Short name
agent = load_pretrained("essence-wars/bc-mcts-100k")  # Full repo ID
agent = load_pretrained("username/my-model")  # User model

# Use in benchmark
from essence_wars.benchmark import EssenceWarsBenchmark
benchmark = EssenceWarsBenchmark()
results = benchmark.evaluate(agent)
```

### Repository Structure

```
huggingface.co/essence-wars/
├── datasets/
│   ├── mcts-self-play           # 1M MCTS vs MCTS games
│   └── balanced-matchups        # Round-robin deck matchups
└── models/
    ├── mcts-baseline            # Tuned MCTS weights (TOML)
    ├── ppo-generalist           # PPO trained on all decks
    ├── bc-mcts-100k             # Behavioral cloning on 100k games
    └── alphazero-v1             # AlphaZero self-play
```

### Deliverables

- [ ] Huggingface account setup (essence-wars organization) - *pending account creation*
- [x] `python/essence_wars/hub.py` - Huggingface integration module
- [x] `python/scripts/upload_dataset.py` - Upload script for datasets
- [x] `python/scripts/upload_model.py` - Upload script for models
- [x] `load_pretrained()` function with lazy import
- [x] `download_dataset()` function
- [x] Auto-generated model cards with training details

---

## Implementation Order

### Phase 1: Dataset Infrastructure (Priority - unlocks everything else)
1. **M5.5A**: `generate-dataset` binary in Rust
2. Generate initial 100k game dataset locally
3. **M5.5B**: Behavioral cloning training script

### Phase 2: Evaluation Infrastructure
4. **M5.5C**: Benchmark API
5. Establish baseline metrics

### Phase 3: Accessibility
6. **M5.5D**: Tutorial notebooks (start with quickstart)
7. **M5.5E**: Huggingface integration

---

## Success Criteria

| Milestone | Metric | Target |
|-----------|--------|--------|
| M5.5A | Dataset generation speed | 100k games in <2 hours |
| M5.5B | BC training vs AlphaZero | Same win rate, 5x faster |
| M5.5C | Benchmark coverage | 6 standard metrics |
| M5.5D | Quickstart time | <15 minutes |
| M5.5E | Model loading | `load_pretrained()` works |

---

## Timeline Estimate

| Task | Effort | Dependencies |
|------|--------|--------------|
| M5.5A: Dataset generator | 4-6 hours | None |
| M5.5B: Behavioral cloning | 2-3 hours | M5.5A |
| M5.5C: Benchmark API | 3-4 hours | None |
| M5.5D: Notebooks (quickstart) | 2-3 hours | M5.5B, M5.5C |
| M5.5E: Huggingface | 2-3 hours | M5.5A, M5.5B |

**Total: ~15-20 hours of development**

---

## Research Opportunities Enabled

With this infrastructure, researchers can:

1. **Behavioral Cloning Studies**: How well can networks imitate MCTS?
2. **Transfer Learning**: Train on Essence Duel, test on Attrition
3. **Sample Efficiency**: How many games needed to match MCTS?
4. **Architecture Search**: Compare network architectures fairly
5. **Curriculum Learning**: Start with weak opponents, increase difficulty
6. **Multi-Agent**: PettingZoo + self-play research
7. **Interpretability**: What does the value network learn?

---

## References

- [Huggingface Datasets](https://huggingface.co/docs/datasets)
- [Huggingface Model Hub](https://huggingface.co/docs/hub)
- [OpenAI Spinning Up](https://spinningup.openai.com/) - Tutorial structure inspiration
- [CleanRL](https://github.com/vwxyzjn/cleanrl) - Implementation reference
