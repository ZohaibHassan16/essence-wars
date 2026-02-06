# Essence Wars - Python ML/AI Infrastructure

High-performance reinforcement learning environment and ML agent implementations for the Essence Wars card game.

## Overview

This package provides:

- **Fast Rust-backed game engine** with Python bindings via PyO3
- **Gymnasium & PettingZoo environments** for single and multi-agent RL
- **Production-ready ML agents**: PPO, AlphaZero, Neural MCTS, Card2Vec
- **Comprehensive benchmarking** with Elo ratings and baselines
- **Training infrastructure** with callbacks, experiment tracking, and reporting

## Installation

```bash
# Basic installation (core + gymnasium)
pip install essence-wars

# With training dependencies (PyTorch, TensorBoard, W&B)
pip install essence-wars[train]

# With analysis tools (pandas, plotly, reporting)
pip install essence-wars[analysis]

# Everything
pip install essence-wars[all]

# Development (using uv)
uv sync --all-groups
```

## Quick Start

### Single Game Environment

```python
from essence_wars import PyGame

game = PyGame()
game.reset(seed=42)

while not game.is_terminal():
    obs = game.observe()           # Shape: (328,)
    mask = game.action_mask()      # Shape: (256,), 1.0 = legal

    # Select action (0-255)
    action = select_action(obs, mask)

    reward, done = game.step(action)

print(f"Winner: {game.winner()}")
```

### Vectorized Training Environment

```python
from essence_wars import PyParallelGames

# 64 parallel games for efficient training
games = PyParallelGames(num_envs=64)
games.reset(seeds=list(range(64)))

obs_batch = games.observe_batch()       # Shape: (64, 328)
masks = games.action_mask_batch()       # Shape: (64, 256)
rewards, dones = games.step_batch(actions)
```

### Gymnasium Interface

```python
import gymnasium as gym

# Standard Gym environment
env = gym.make("EssenceWars-v0")
obs, info = env.reset(seed=42)

# With reward shaping
env = gym.make("EssenceWars-v0", reward_shaping=True)

# Vectorized
from essence_wars.env import VectorizedEssenceWars
vec_env = VectorizedEssenceWars(num_envs=64)
```

### PettingZoo Multi-Agent

```python
from essence_wars.parallel_env import EssenceWarsParallelEnv

env = EssenceWarsParallelEnv()
observations, infos = env.reset(seed=42)

# Self-play or multi-agent training
for agent in env.agent_iter():
    obs = observations[agent]
    action = policy(obs)
    observations, rewards, terminations, truncations, infos = env.step(action)
```

## CLI Tools

```bash
# Training
essence-wars train ppo --timesteps 500000
essence-wars train alphazero --iterations 100
essence-wars train behavioral-cloning --data expert.jsonl.gz
essence-wars train card2vec --data games.jsonl.gz --embed-dim 64

# Evaluation
essence-wars benchmark --checkpoint model.pt
essence-wars benchmark --checkpoint model.pt --full-eval
essence-wars evaluate-mcts --checkpoint model.pt --simulations 100

# Data Generation
essence-wars data generate-distillation --games 10000 --output data.jsonl.gz

# Reporting
essence-wars report generate --run-id latest --open
essence-wars report leaderboard
```

## Architecture

```
essence_wars/
├── _core                  # Rust bindings (PyO3)
│   ├── PyGame             # Single game interface
│   ├── PyParallelGames    # Vectorized games
│   ├── STATE_TENSOR_SIZE  # 328 (observation dimension)
│   └── ACTION_SPACE_SIZE  # 256 (discrete action space)
│
├── env.py                 # Gymnasium environments
├── parallel_env.py        # PettingZoo multi-agent
│
├── agents/                # ML Algorithms
│   ├── ppo.py             # Proximal Policy Optimization
│   ├── alphazero.py       # AlphaZero self-play
│   ├── neural_mcts.py     # Neural MCTS inference
│   ├── card2vec.py        # Card embedding pretraining
│   ├── networks.py        # Neural network architectures
│   └── embeddings.py      # Observation transformers
│
├── benchmark/             # Evaluation Framework
│   ├── api.py             # Main benchmark suite
│   ├── agents.py          # Agent protocols
│   └── elo.py             # Elo rating system
│
├── training/              # Training Utilities
│   └── callbacks.py       # Checkpoint, evaluation, reporting
│
├── infra/                 # Infrastructure
│   └── experiment.py      # Experiment tracking
│
├── analysis/              # Analysis & Reporting
│   └── report/            # HTML report generation
│
└── data/                  # Dataset Utilities
    └── dataset.py         # MCTS dataset loading
```

## ML Algorithms

### PPO (Proximal Policy Optimization)

Production-ready PPO with action masking for legal move handling.

```python
from essence_wars.agents.ppo import PPOTrainer, PPOConfig

config = PPOConfig(
    total_timesteps=500_000,
    num_envs=64,
    learning_rate=3e-4,
    gamma=0.99,
    gae_lambda=0.95,
    clip_epsilon=0.2,
    normalize_obs=True,
    # Faction specialist training
    player_faction="symbiote",
    opponent_factions=["argentum", "obsidion"],
)

trainer = PPOTrainer(config)
trainer.train()
```

**Features:**
- Vectorized environments (64+ parallel games)
- GAE advantage estimation
- Observation normalization
- Faction-specific training
- Deck cycling during training
- TensorBoard logging

### AlphaZero

Self-play training with neural MCTS, featuring a dual-buffer approach to prevent catastrophic forgetting.

```python
from essence_wars.agents.alphazero import AlphaZeroTrainer, AlphaZeroConfig

config = AlphaZeroConfig(
    num_iterations=100,
    games_per_iteration=100,
    mcts_simulations=50,
    learning_rate=1e-3,
    batch_size=256,
    # Dual buffer for BC warm-start
    bc_data_path="expert_data.jsonl.gz",
    bc_ratio=0.3,  # 30% BC data in each batch
)

trainer = AlphaZeroTrainer(config)
trainer.train()
```

**Features:**
- Batched MCTS with virtual loss for GPU efficiency
- Dual replay buffer (self-play + behavioral cloning)
- Temperature-based exploration (hot early, cold late)
- Dirichlet noise at root for exploration

### Neural MCTS

Convert any trained network into a stronger player via MCTS search.

```python
from essence_wars.agents.neural_mcts import NeuralMctsBot, load_ppo_network

# Load trained network
network = load_ppo_network("checkpoints/ppo_best.pt")

# Create MCTS-augmented bot
bot = NeuralMctsBot(
    network=network,
    num_simulations=100,
    c_puct=1.5,
    temperature=0.1,
)

# Get action with search
action = bot.get_action_with_game(game)
```

**Features:**
- Batched leaf evaluation (16-64 positions per GPU call)
- Virtual loss for parallel tree search
- Configurable rollout policies

### Card2Vec

Pre-train card embeddings using co-occurrence and attribute prediction.

```python
from essence_wars.agents.card2vec import train_card2vec, Card2VecConfig

config = Card2VecConfig(
    embed_dim=64,
    learning_rate=1e-3,
    batch_size=512,
    # Multi-task learning
    cooccurrence_weight=1.0,
    attribute_weight=0.5,
)

embeddings = train_card2vec(
    data_path="games.jsonl.gz",
    config=config,
)

# Use in PPO/AlphaZero
ppo_config = PPOConfig(
    embedding_mode="pretrained",
    pretrained_embeddings=embeddings,
)
```

## Observation Space

The observation tensor has 328 dimensions encoding:

| Section | Indices | Description |
|---------|---------|-------------|
| Global | 0-15 | Turn, phase, essence, action points |
| Player creatures | 16-75 | 5 slots × 12 features (stats, keywords) |
| Player supports | 76-85 | 2 slots × 5 features |
| Player hand | 86-155 | 7 cards × 10 features |
| Opponent creatures | 156-215 | 5 slots × 12 features |
| Opponent supports | 216-225 | 2 slots × 5 features |
| Opponent hand | 226-232 | Card count (hidden info) |
| Commander abilities | 233-327 | Passive/triggered effects |

## Action Space

256 discrete actions:

| Range | Action Type |
|-------|-------------|
| 0-34 | Play card from hand to slot |
| 35-59 | Attack with creature |
| 60-84 | Use creature ability |
| 85-254 | Reserved |
| 255 | End turn |

## Benchmarking

```python
from essence_wars.benchmark import EssenceWarsBenchmark

benchmark = EssenceWarsBenchmark()

# Evaluate against baselines
results = benchmark.evaluate(
    agent=my_agent,
    num_games=1000,
    opponents=["random", "greedy", "mcts_50", "mcts_100"],
)

print(f"Win rate vs Greedy: {results.vs_greedy.win_rate:.1%}")
print(f"Elo rating: {results.elo:.0f}")

# Export results
results.to_json("benchmark_results.json")
```

**Standard Baselines:**
- `random`: Uniform random legal actions
- `greedy`: Single-step heuristic evaluation
- `mcts_50`: 50 MCTS simulations
- `mcts_100`: 100 MCTS simulations
- `alphabeta_6`: Alpha-beta search depth 6

## Training Callbacks

```python
from essence_wars.training.callbacks import (
    CheckpointCallback,
    EvaluationCallback,
    AutoReportCallback,
)

callbacks = [
    CheckpointCallback(save_freq=10000, save_best=True),
    EvaluationCallback(eval_freq=5000, num_games=100),
    AutoReportCallback(generate_on_complete=True),
]

trainer = PPOTrainer(config, callbacks=callbacks)
trainer.train()
```

## Experiment Tracking

```python
from essence_wars.infra import Experiment

exp = Experiment(
    name="ppo_symbiote_v2",
    config={"lr": 3e-4, "timesteps": 500000},
)

# Automatic directory structure
# experiments/ppo_symbiote_v2_20240215_143022/
#   ├── config.json
#   ├── checkpoints/
#   ├── logs/
#   └── metrics.csv

exp.log_metrics({"loss": 0.5, "reward": 1.2})
exp.save_artifact("final_stats", {"elo": 1850})
```

## Key Constants

```python
from essence_wars import STATE_TENSOR_SIZE, ACTION_SPACE_SIZE

STATE_TENSOR_SIZE  # 328 - observation dimension
ACTION_SPACE_SIZE  # 256 - discrete action space
```

## Performance Tips

1. **Use vectorized environments** for training (64+ parallel games)
2. **Enable observation normalization** for stable training
3. **Use batched MCTS** for Neural MCTS inference
4. **Pre-train Card2Vec embeddings** for faster convergence
5. **Use dual buffer** in AlphaZero to prevent forgetting

## Testing

```bash
# Run all tests
uv run pytest python/tests

# Type checking
uv run mypy python/essence_wars

# Linting
uv run ruff check python/essence_wars
```

## Citation

If you use this codebase in your research, please cite:

```bibtex
@software{essence_wars,
  title = {Essence Wars: A High-Performance Card Game Environment for RL Research},
  author = {Wissmann, Christian},
  year = {2025},
  url = {https://github.com/christianWissmann85/essence-wars}
}
```

## License

MIT License - see [LICENSE](../LICENSE) for details.
