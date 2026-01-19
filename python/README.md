# Essence Wars - Python Quickstart

**Get from `pip install` to training in 5 minutes.**

[![PyPI](https://img.shields.io/pypi/v/essence-wars)](https://pypi.org/project/essence-wars/)
[![Leaderboard](https://img.shields.io/badge/🏆-Leaderboard-blue)](https://huggingface.co/spaces/Chris-Essence-Wars/essence-wars-leaderboard)

---

## 1. Install (30 seconds)

```bash
pip install essence-wars[train]
```

This installs:
- Core game engine (Rust bindings)
- PyTorch for neural networks
- Gymnasium environment
- Training utilities

---

## 2. Play Your First Game (30 seconds)

```python
from essence_wars import PyGame

game = PyGame()
game.reset(seed=42)

while not game.is_done():
    obs = game.observe()       # (326,) state tensor
    mask = game.action_mask()  # (256,) legal action mask

    # Random action from legal moves
    action = mask.nonzero()[0][0]
    reward, done = game.step(action)

print(f"Winner: Player {1 if game.get_reward(0) > 0 else 2}")
```

---

## 3. Gymnasium Environment (1 minute)

```python
from essence_wars import EssenceWarsEnv

# Play against the built-in Greedy bot
env = EssenceWarsEnv(opponent="greedy")
obs, info = env.reset(seed=42)

total_reward = 0
while True:
    # Your policy here (random for demo)
    mask = info["action_mask"]
    action = mask.nonzero()[0][0]

    obs, reward, terminated, truncated, info = env.step(action)
    total_reward += reward

    if terminated or truncated:
        break

print(f"Result: {'Win' if total_reward > 0 else 'Loss'}")
```

**Opponents:** `"random"`, `"greedy"`, `"mcts"` (50 sims), `"mcts100"` (100 sims)

---

## 4. Load a Pretrained Agent (1 minute)

```python
from essence_wars.benchmark import NeuralAgent, EssenceWarsBenchmark

# Load from HuggingFace (best agent: 72% vs Greedy)
agent = NeuralAgent.from_checkpoint(
    "https://huggingface.co/Chris-Essence-Wars/ppo-argentum/resolve/main/model.pt"
)

# Quick benchmark
benchmark = EssenceWarsBenchmark(verbose=True)
results = benchmark.quick_evaluate(agent, games=40)

print(f"Win rate vs Greedy: {results['vs_greedy']:.1%}")
print(f"Elo rating: {results['elo']:.0f}")
```

**Available models:**
| Model | Win Rate | HuggingFace |
|-------|----------|-------------|
| PPO-Argentum | 72% | [ppo-argentum](https://huggingface.co/Chris-Essence-Wars/ppo-argentum) |
| PPO-Flat | 71% | [ppo-flat](https://huggingface.co/Chris-Essence-Wars/ppo-flat) |
| PPO-Embedded | 65% | [ppo-embedded](https://huggingface.co/Chris-Essence-Wars/ppo-embedded) |

---

## 5. Train Your Own Agent (2 minutes)

### Option A: Command Line

```bash
# Train PPO for 300k steps (~10 min on GPU)
python -m essence_wars.scripts.train_ppo --timesteps 300000

# Train faction specialist
python -m essence_wars.scripts.train_ppo \
    --timesteps 300000 \
    --player-faction argentum \
    --observation-mode embedded
```

### Option B: Python API

```python
from essence_wars.agents.ppo import PPOTrainer, PPOConfig

config = PPOConfig(
    total_timesteps=300_000,
    num_envs=64,
    hidden_dim=256,
    learning_rate=3e-4,
    ent_coef=0.02,  # Higher entropy prevents collapse
)

trainer = PPOTrainer(config)
trainer.train()

# Evaluate
from essence_wars.benchmark import NeuralAgent, EssenceWarsBenchmark

agent = NeuralAgent(trainer.network, name="my-ppo")
benchmark = EssenceWarsBenchmark()
results = benchmark.quick_evaluate(agent)
print(f"Your agent: {results['vs_greedy']:.1%} vs Greedy")
```

---

## 6. Submit to Leaderboard (Optional)

```bash
# Submit your trained model
python -m essence_wars.scripts.submit_agent \
    --checkpoint experiments/ppo/*/best_model.pt \
    --name "My Agent" \
    --repo-id yourusername/essence-wars-agent
```

See the [submission guide](https://github.com/christianWissmann85/essence-wars/blob/master/docs/SUBMIT_AGENT.md) for details.

---

## Key Concepts

### State Space (326 floats)

| Section | Size | Description |
|---------|------|-------------|
| Global | 10 | Turn, phase, active player |
| Creatures | 120 | 5 slots × 2 players × 12 features |
| Supports | 16 | 2 slots × 2 players × 4 features |
| Hands | 120 | 20 cards × 2 players × 3 features |
| Decks | 60 | 30 cards × 2 players |

### Action Space (256 discrete)

| Range | Action |
|-------|--------|
| 0-99 | PlayCard (hand_idx × slot) |
| 100-149 | Attack (attacker × target) |
| 150-249 | UseAbility |
| 255 | EndTurn |

### Baselines

| Bot | Elo | Description |
|-----|-----|-------------|
| RandomBot | 1000 | Uniform random |
| GreedyBot | 1300 | Heuristic evaluation |
| MCTS-50 | 1450 | 50 simulations |
| MCTS-100 | 1500 | 100 simulations |

---

## High-Throughput Training

```python
from essence_wars import VectorizedEssenceWars

# 64 parallel environments
vec_env = VectorizedEssenceWars(num_envs=64, opponent="greedy")
obs, masks = vec_env.reset(seed=42)

# ~268,000 steps/second on modern hardware
for _ in range(1000):
    actions = your_policy(obs, masks)
    obs, rewards, dones, masks = vec_env.step(actions)
```

---

## Resources

| Resource | Link |
|----------|------|
| Leaderboard | [HuggingFace Space](https://huggingface.co/spaces/Chris-Essence-Wars/essence-wars-leaderboard) |
| Datasets | [mcts-10k-sims100](https://huggingface.co/datasets/Chris-Essence-Wars/mcts-10k-sims100) |
| Notebooks | [tutorials/](https://github.com/christianWissmann85/essence-wars/tree/master/notebooks) |
| Full Docs | [docs/](https://github.com/christianWissmann85/essence-wars/tree/master/docs) |
| Submit Agent | [SUBMIT_AGENT.md](https://github.com/christianWissmann85/essence-wars/blob/master/docs/SUBMIT_AGENT.md) |

---

## Citation

```bibtex
@software{essence_wars,
  title = {Essence Wars: A Deterministic Card Game Engine for AI Research},
  author = {Wissmann, Christian},
  year = {2026},
  url = {https://github.com/christianWissmann85/essence-wars}
}
```
