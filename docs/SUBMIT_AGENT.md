# How to Submit Your Agent to the Leaderboard

This guide explains how to train an agent for Essence Wars and submit it to the official leaderboard.

## Quick Start

```bash
# 1. Train your agent
uv run python python/scripts/train_ppo.py --timesteps 300000

# 2. Submit to leaderboard
uv run python python/scripts/submit_agent.py \
    --checkpoint experiments/ppo/*/best_model.pt \
    --name "My PPO Agent" \
    --repo-id yourusername/essence-wars-agent
```

## Prerequisites

1. **Install essence-wars**:
   ```bash
   pip install essence-wars
   # or for development
   git clone https://github.com/christianWissmann85/essence-wars
   cd essence-wars
   uv sync --all-extras
   ```

2. **HuggingFace account** (optional, for model hosting):
   ```bash
   pip install huggingface_hub
   huggingface-cli login
   ```

## Training Your Agent

### Option 1: PPO (Recommended for beginners)

```bash
# Generalist agent (plays all factions)
uv run python python/scripts/train_ppo.py \
    --timesteps 300000 \
    --ent-coef 0.02

# Faction specialist
uv run python python/scripts/train_ppo.py \
    --timesteps 300000 \
    --player-faction argentum \
    --observation-mode embedded
```

**Key hyperparameters**:
- `--ent-coef 0.02`: Higher entropy reduces policy collapse
- `--observation-mode embedded`: Use learned card embeddings
- `--eval-interval 25000`: Evaluate every 25k steps

### Option 2: Behavioral Cloning

```bash
# Download MCTS dataset
# Available: mcts-1k-sims100, mcts-10k-sims100, mcts-100k-sims100

uv run python python/scripts/train_behavioral_cloning.py \
    --dataset data/datasets/mcts_10k_*.jsonl.gz \
    --epochs 10
```

### Option 3: AlphaZero

```bash
uv run python python/scripts/train_alphazero.py \
    --iterations 100 \
    --games-per-iteration 100
```

### Option 4: Custom Architecture

Implement the `BenchmarkAgent` protocol:

```python
from essence_wars.benchmark.agents import BenchmarkAgent
import numpy as np

class MyAgent(BenchmarkAgent):
    @property
    def name(self) -> str:
        return "MyCustomAgent"

    def select_action(
        self,
        observation: np.ndarray,  # Shape: (326,)
        action_mask: np.ndarray,  # Shape: (256,)
    ) -> int:
        # Your logic here
        valid_actions = np.where(action_mask > 0)[0]
        return int(np.random.choice(valid_actions))

    def reset(self) -> None:
        pass  # Reset any episode state
```

## Submission Methods

### Method 1: Local Submission (Quick)

For testing or local development:

```bash
uv run python python/scripts/submit_agent.py \
    --checkpoint models/my_agent.pt \
    --name "My Agent" \
    --author "myusername" \
    --dry-run  # Test without modifying leaderboard
```

Remove `--dry-run` to actually submit.

### Method 2: HuggingFace Submission (Recommended)

Upload your model to HuggingFace for permanence and reproducibility:

```bash
uv run python python/scripts/submit_agent.py \
    --checkpoint models/my_agent.pt \
    --name "My PPO Agent" \
    --repo-id yourusername/essence-wars-ppo \
    --description "PPO with custom reward shaping" \
    --tags generalist ppo experimental
```

This will:
1. Validate your checkpoint
2. Run quick evaluation (40 games)
3. Upload to HuggingFace Hub
4. Add entry to leaderboard

### Method 3: GitHub Issue (Full Evaluation)

For official ranking with full evaluation (400 games):

1. Upload your model to HuggingFace
2. Open an issue in the repository with:
   ```
   Title: [Submission] My Agent Name

   Model URL: https://huggingface.co/yourusername/model-name
   Name: My Agent Name
   Description: Brief description of your approach
   ```
3. Add the label `evaluate-agent`
4. GitHub Actions will automatically evaluate and update the leaderboard

## Evaluation Methodology

### Baselines

| Baseline | Elo | Description |
|----------|-----|-------------|
| RandomBot | 1000 | Uniform random action selection |
| GreedyBot | 1300 | Heuristic evaluation with tuned weights |
| MCTS-50 | 1450 | Monte Carlo Tree Search, 50 sims |
| MCTS-100 | 1500 | Monte Carlo Tree Search, 100 sims |

### Quick Evaluation (40 games)

- 20 games vs GreedyBot
- 20 games vs RandomBot
- Estimated Elo from win rate vs Greedy

### Full Evaluation (400 games)

- 100 games vs each baseline
- More accurate Elo calculation
- Required for top-10 ranking

### Elo Calculation

We use standard Elo with K-factor 32:
- Win rate vs Greedy → Elo differential
- Baseline Greedy Elo = 1300
- 95% confidence interval: ±400/√(games)

## Checkpoint Format

Your checkpoint must be loadable by `NeuralAgent.from_checkpoint()`:

```python
# Supported formats:
# 1. PPO checkpoint (from train_ppo.py)
{
    "model_state_dict": {...},
    "config": {"hidden_dim": 256, ...}
}

# 2. AlphaZero checkpoint (from train_alphazero.py)
{
    "model_state_dict": {...},
    "config": {"num_blocks": 4, ...}
}

# 3. BC checkpoint (from train_behavioral_cloning.py)
{
    "model_state_dict": {...},
    "config": {...}
}
```

## Tips for High Performance

1. **Use best checkpoint saving**: Policy collapse is common; save checkpoints at peak performance
   ```bash
   # Enabled by default in train_ppo.py
   # Look for best_model.pt in your experiment folder
   ```

2. **Higher entropy coefficient**: Prevents premature convergence
   ```bash
   --ent-coef 0.02  # Default, increase to 0.05 if collapsing
   ```

3. **Embedded architecture for specialists**: Faction specialists train better with embeddings
   ```bash
   --observation-mode embedded --player-faction argentum
   ```

4. **More training steps**: 300k is minimum, 1M+ for best results
   ```bash
   --timesteps 1000000
   ```

## Leaderboard Rules

1. **One entry per model**: Update existing entries rather than creating duplicates
2. **Reproducibility**: Models must be downloadable and evaluatable
3. **No cheating**: Agents must not exploit evaluation bugs
4. **Fair play**: No hardcoded opponent-specific strategies

## Troubleshooting

### "Checkpoint not found"
```bash
# Check your path
ls -la models/*.pt
```

### "Failed to load checkpoint"
```bash
# Verify checkpoint format
python -c "import torch; print(torch.load('model.pt').keys())"
```

### "Win rate is 0%"
Your agent may not be training correctly. Check:
- Observation normalization
- Action masking (invalid actions should be masked)
- Reward signal

### "HuggingFace upload failed"
```bash
# Re-authenticate
huggingface-cli login
# Check token permissions (need write access)
```

## Questions?

- Open an issue on [GitHub](https://github.com/christianWissmann85/essence-wars/issues)
- Check existing agents for reference implementations

---

Happy training! 🎮
