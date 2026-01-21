# Essence Wars

**A Deterministic Card Game Engine for AI Research**

[![PyPI](https://img.shields.io/pypi/v/essence-wars?style=for-the-badge&logo=pypi&logoColor=white)](https://pypi.org/project/essence-wars/)
[![Python](https://img.shields.io/pypi/pyversions/essence-wars?style=for-the-badge&logo=python&logoColor=white)](https://pypi.org/project/essence-wars/)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)

[![Balance Dashboard](https://img.shields.io/badge/📊_Balance-Dashboard-blue?style=for-the-badge)](https://christianWissmann85.github.io/essence-wars/dashboard/index.html)
[![Training Dashboard](https://img.shields.io/badge/📈_Training-Dashboard-purple?style=for-the-badge)](https://christianWissmann85.github.io/essence-wars/dashboard/training.html)
[![Performance](https://img.shields.io/badge/⚡_Performance-Dashboard-orange?style=for-the-badge)](https://christianWissmann85.github.io/essence-wars/dashboard/performance.html)

---

## Live Research Dashboards

> **[View Interactive Dashboards →](https://christianWissmann85.github.io/essence-wars/)**

| Dashboard | Description |
|-----------|-------------|
| [**Balance Dashboard**](https://christianWissmann85.github.io/essence-wars/dashboard/index.html) | Faction matchups, deck rankings, P1/P2 analysis, combat statistics |
| [**Training Dashboard**](https://christianWissmann85.github.io/essence-wars/dashboard/training.html) | MCTS weight tuning, fitness curves, convergence analysis |
| [**Performance Dashboard**](https://christianWissmann85.github.io/essence-wars/dashboard/performance.html) | Engine benchmarks, throughput metrics, latency analysis |

---

## Key Metrics

| Metric | Value |
|--------|-------|
| **Cards** | 300 (New Horizons Edition) |
| **Commander Decks** | 12 pre-built decks |
| **Keywords** | 16 mechanical interactions |
| **Random Game Throughput** | ~80,000 games/sec |
| **Greedy Game Throughput** | ~17,000 games/sec |
| **Vectorized Env SPS** | ~268,000 steps/sec |
| **Batched Neural MCTS** | 10-20x GPU speedup |
| **Best Agent (PPO)** | 72% vs Greedy |

---

## Faction Balance

The game features a deliberate **rock-paper-scissors** dynamic:

```
              Argentum   Obsidion   Symbiote
Argentum         -        59.8%      50.8%    ← Argentum beats Obsidion
Obsidion       40.2%        -        52.2%    ← Obsidion beats Symbiote
Symbiote       49.2%      47.8%        -      ← Balanced matchup
```

| Faction | Identity | Playstyle |
|---------|----------|-----------|
| **Argentum Combine** | "The Wall" | Defensive constructs, Guard synergy, high HP |
| **Symbiote Circles** | "The Swarm" | Aggressive tempo, Rush creatures, death triggers |
| **Obsidion Syndicate** | "The Shadow" | Burst damage, Lifesteal, Stealth assassins |
| **Free-Walkers** | Neutral | Utility cards that splash into any faction |

---

## Game Modes

| Mode | Win Condition | Status |
|------|---------------|--------|
| **Attrition** | 0 life OR turn 30 → higher life | Default, MCTS trained |
| **Essence Duel** | First to 50 VP (face damage) | Experimental |

```bash
# Attrition (default)
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 100

# Essence Duel
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 100 --mode essence-duel
```

See [docs/game-modes.md](https://christianWissmann85.github.io/essence-wars/game-modes.md) for details.

---

## Installation

### Python (Recommended for ML Research)

```bash
# Install from PyPI
pip install essence-wars

# With training dependencies (PyTorch, TensorBoard)
pip install essence-wars[train]

# With all optional dependencies
pip install essence-wars[train,analysis,hub]
```

### From Source (Rust Development)

```bash
# Clone and build
git clone https://github.com/christianWissmann85/essence-wars
cd essence-wars
cargo build --release

# Install Python package in development mode
pip install maturin
maturin develop --release
```

---

## Quick Start

### Python

```python
from essence_wars import PyGame

# Create and reset a game
game = PyGame()
game.reset(seed=42)

# Get observation and legal actions
obs = game.observe()        # numpy array (326,)
mask = game.action_mask()   # numpy array (256,)

# Play a game
while not game.is_done():
    legal_actions = mask.nonzero()[0]
    action = legal_actions[0]  # Or use your policy
    reward, done = game.step(action)
```

### Rust CLI Tools

```bash
# Run bot arena matches
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 100 --progress

# Run balance validation (8,000 games)
cargo run --release --bin validate -- --games 100

# Generate all dashboards
./scripts/generate-all-dashboards.sh
```

---

## AI Research Interface

```rust
use cardgame::engine::GameEnvironment;

// Get game state as neural network input (326 floats)
let tensor: [f32; 326] = env.get_state_tensor();

// Get legal action mask (256 floats, 0.0 or 1.0)
let mask: [f32; 256] = env.get_legal_action_mask();

// Apply action from neural network output
env.apply_action_by_index(action_idx);

// Get reward signal (-1.0, 0.0, or 1.0)
let reward = env.get_reward(player_id);

// Clone state for MCTS tree search
let clone = env.fork();
```

### Bot Types

| Bot | Description | Use Case |
|-----|-------------|----------|
| `random` | Uniform random selection | Baseline |
| `greedy` | Heuristic evaluation | Training opponent |
| `mcts` | Monte Carlo Tree Search | Strong benchmark |
| `agent-*` | MCTS with tuned weights | Faction specialists |

### Python / Gymnasium Interface

```python
from essence_wars import EssenceWarsEnv, VectorizedEssenceWars

# Single environment (Gymnasium v26+ compliant)
env = EssenceWarsEnv(opponent="greedy")
obs, info = env.reset(seed=42)

while True:
    action = policy.select(obs, info["action_mask"])
    obs, reward, terminated, truncated, info = env.step(action)
    if terminated or truncated:
        break

# Vectorized for high-throughput training (~268k SPS)
vec_env = VectorizedEssenceWars(num_envs=64)
obs, masks = vec_env.reset(seed=42)

for _ in range(num_steps):
    actions = policy.batch_select(obs, masks)
    obs, rewards, dones, masks = vec_env.step(actions)
    # Done environments auto-reset
```

---

## Trained Models & Datasets

Pre-trained models and datasets are available on HuggingFace:

| Model | Win Rate vs Greedy | HuggingFace |
|-------|-------------------|-------------|
| PPO-Argentum | **72%** | [Chris-Essence-Wars/ppo-argentum](https://huggingface.co/Chris-Essence-Wars/ppo-argentum) |
| PPO-Flat | 71% | [Chris-Essence-Wars/ppo-flat](https://huggingface.co/Chris-Essence-Wars/ppo-flat) |
| BC-MCTS-10k | 66% | [Chris-Essence-Wars/bc-mcts-10k-best](https://huggingface.co/Chris-Essence-Wars/bc-mcts-10k-best) |
| Distilled-MCTS50 | 71% | [Chris-Essence-Wars/distilled-mcts50-10k](https://huggingface.co/Chris-Essence-Wars/distilled-mcts50-10k) |

| Dataset | Games | Samples | HuggingFace |
|---------|-------|---------|-------------|
| MCTS-100k | 100,000 | ~9M | [Chris-Essence-Wars/mcts-100k-sims100](https://huggingface.co/datasets/Chris-Essence-Wars/mcts-100k-sims100) |
| MCTS-10k | 10,000 | ~900k | [Chris-Essence-Wars/mcts-10k-sims100](https://huggingface.co/datasets/Chris-Essence-Wars/mcts-10k-sims100) |

---

## Neural MCTS (Batched GPU Inference)

Neural MCTS with **batched GPU inference** achieves 10-20x speedup over sequential evaluation:

```python
from essence_wars.agents.neural_mcts import NeuralMctsBot

bot = NeuralMctsBot(network, num_simulations=100, device="cuda")

# Batched inference (10-20x faster)
action, policy = bot.get_action_with_game_batched(game, batch_size=32)
```

```bash
# Generate distillation data with batched MCTS
uv run python python/scripts/generate_distillation_data.py \
    --model models/bc_mcts_values.pt \
    --games 1000 --sims 50 --batch-size 32
```

See [docs/batched-mcts.md](docs/batched-mcts.md) for details.

---

## Weight Tuning (CMA-ES)

```bash
# Train generalist weights
cargo run --release --bin tune -- --mode generalist --generations 100

# Train faction specialist
cargo run --release --bin tune -- --mode faction-specialist --faction argentum

# View training results
./scripts/analyze-mcts.sh
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [**Game Design**](https://christianWissmann85.github.io/essence-wars/essence-wars-design.md) | Full game rules, mechanics, keywords |
| [**Engine Architecture**](https://christianWissmann85.github.io/essence-wars/design-engine.md) | API reference, state representation |
| [**Card Database**](https://christianWissmann85.github.io/essence-wars/cards-new-horizons.md) | All 300 cards, commander abilities |
| [**Cloud Training**](https://christianWissmann85.github.io/essence-wars/modal-cloud-setup.md) | Training with Modal.com |

---

## Project Structure

```
├── src/                    # Rust game engine
│   ├── core/              # Game state, rules, effects
│   ├── engine/            # Game loop, effect resolution
│   ├── bots/              # AI players (Random, Greedy, MCTS)
│   └── bin/               # CLI tools (arena, tune, validate)
├── data/
│   ├── cards/core_set/    # 300 cards in YAML
│   ├── decks/             # 12 Commander Decks in TOML
│   └── weights/           # Tuned bot weights
├── python/                # Python package + analysis
│   └── essence_wars/      # Gymnasium env, analysis tools
├── docs/                  # Documentation + GitHub Pages
│   ├── dashboard/         # Interactive dashboards
│   └── *.md               # Design documents
├── scripts/               # Shell wrappers
└── tests/                 # ~576 tests
```

---

## Requirements

- **Rust** 1.75+ with Cargo
- **Python** 3.11+ with uv (for analysis tools)

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

---

## License

MIT
