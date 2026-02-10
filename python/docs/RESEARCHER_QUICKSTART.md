# Essence Wars: Quickstart Guide for ML/AI Researchers

Welcome! This guide helps you get started with the Essence Wars RL environment in minutes.

## What is Essence Wars?

Essence Wars is a deterministic, perfect-information card game designed as an RL research testbed. It features:
- **Fast simulation**: 60K+ steps/second with vectorized environments
- **Rich action space**: 256 discrete actions (play cards, attack, use abilities)
- **Complex state**: 328-dimensional observation tensor
- **Built-in baselines**: Random, Greedy, MCTS, Alpha-Beta bots
- **Standard APIs**: Gymnasium v26 and PettingZoo compatible

## Installation

```bash
# Basic installation
pip install essence-wars

# With PyTorch for training
pip install essence-wars[train]

# With analysis/visualization tools
pip install essence-wars[analysis]

# Everything
pip install essence-wars[all]

# Development (from source)
git clone https://github.com/yourrepo/essence-wars
cd essence-wars
uv sync --all-groups  # or pip install -e .[all]
```

## Quick Verification

```python
# Verify installation works
from essence_wars import PyGame

game = PyGame()
game.reset(seed=42)
print(f"State shape: {game.observe().shape}")  # (328,)
print(f"Action space: {game.action_mask().shape}")  # (256,)
print("Installation verified!")
```

## Using the Gymnasium Environment

```python
import gymnasium as gym
import numpy as np
from essence_wars import EssenceWarsEnv

# Create environment
env = EssenceWarsEnv(
    deck1='artificer_tokens',  # Player deck
    deck2='broodmother_pack',  # Opponent deck
    opponent='greedy',         # Built-in opponent
)

# Standard Gymnasium loop
obs, info = env.reset(seed=42)
total_reward = 0

while True:
    # Get valid actions from action mask
    mask = info['action_mask']
    valid_actions = np.where(mask > 0)[0]
    action = np.random.choice(valid_actions)

    obs, reward, terminated, truncated, info = env.step(action)
    total_reward += reward

    if terminated or truncated:
        break

print(f"Episode reward: {total_reward}")
env.close()
```

## Available Decks

```python
from essence_wars._core import PyGame

decks = PyGame.list_decks()
print(decks)
# ['alpha_frenzy', 'sanctum_healer', 'shadow_weaver',
#  'grove_regenerate', 'broodmother_pack', ...]
```

## Vectorized Training

For high-throughput training:

```python
from essence_wars import VectorizedEssenceWars
import numpy as np

# Create 64 parallel environments
vec_env = VectorizedEssenceWars(num_envs=64)
obs, masks = vec_env.reset(seed=42)

# Batch operations
for step in range(1000):
    # Select actions (e.g., random valid actions)
    actions = np.array([
        np.random.choice(np.where(masks[i] > 0)[0])
        for i in range(64)
    ])

    obs, rewards, dones, masks = vec_env.step(actions)
    # Environments auto-reset when done

vec_env.close()
```

## Multi-Agent (PettingZoo)

```python
from essence_wars.parallel_env import parallel_env
import numpy as np

env = parallel_env()
observations, infos = env.reset(seed=42)

while env.agents:
    actions = {}
    for agent in env.agents:
        mask = infos[agent]['action_mask']
        valid = np.where(mask > 0)[0]
        actions[agent] = np.random.choice(valid)

    observations, rewards, terminations, truncations, infos = env.step(actions)

env.close()
```

## Training a PPO Agent

```bash
# Quick training run (for testing)
uv run python scripts/training/ppo.py \
    --timesteps 10000 \
    --num-envs 16 \
    --no-tensorboard

# Full training run
uv run python scripts/training/ppo.py \
    --timesteps 500000 \
    --num-envs 64

# Faction specialist
uv run python scripts/training/ppo.py \
    --timesteps 300000 \
    --player-faction argentum
```

## Evaluating Your Agent

```python
from essence_wars.benchmark import EssenceWarsBenchmark, NeuralAgent

# Load your trained model
agent = NeuralAgent.from_checkpoint('experiments/ppo/latest/best_model.pt')

# Run benchmark
benchmark = EssenceWarsBenchmark(games_per_opponent=100)
results = benchmark.evaluate(agent)

print(results.summary())
# === Benchmark Results ===
# Elo Rating: 1450
# vs Random:   95.0%
# vs Greedy:   65.0%
# vs MCTS-50:  55.0%
# vs MCTS-100: 45.0%
```

## Using Built-in Bots

```python
from essence_wars._core import PyGame

game = PyGame()
game.reset(seed=42)

while not game.is_done():
    # Choose a bot
    action = game.greedy_action()      # Fast heuristic
    # action = game.mcts_action(100)   # MCTS with 100 simulations
    # action = game.alphabeta_action(6) # Alpha-Beta depth 6
    # action = game.random_action()    # Random valid action

    reward, done = game.step(action)
```

## Key Spaces

### Observation Space (328 floats)
| Index Range | Description |
|-------------|-------------|
| 0-15 | Global state (turn, phase, essence, AP) |
| 16-75 | Player 1 creatures (5 slots x 12 features) |
| 76-85 | Player 1 supports (2 slots x 5 features) |
| 86-155 | Player 1 hand (7 cards x 10 features) |
| 156-225 | Player 2 creatures + supports |
| 226-232 | Player 2 hand count (hidden info) |
| 233-327 | Commander abilities |

### Action Space (256 discrete)
| Index Range | Action Type |
|-------------|-------------|
| 0-34 | Play card from hand to slot |
| 35-59 | Attack with creature |
| 60-84 | Use creature ability |
| 255 | End turn |

## Experiment Tracking

```python
from essence_wars.infra.experiment import Experiment

exp = Experiment('training', tag='my_experiment')
exp.save_config({'lr': 0.001, 'epochs': 100})

for epoch in range(100):
    loss = train_one_epoch()
    exp.log_metric('loss', loss, step=epoch)

exp.save_metrics()  # Persists to experiments/training/TIMESTAMP/
```

## Next Steps

1. **Explore the codebase**: See `python/essence_wars/` for all components
2. **Read the design doc**: `docs/essence-wars-design.md` explains game rules
3. **Try AlphaZero**: `scripts/training/alphazero.py` for self-play training
4. **Run benchmarks**: Compare your agent against baselines

## Getting Help

- Open an issue on GitHub
- Check the test suite for usage examples: `python/tests/`
- Read the module docstrings

Happy researching!
