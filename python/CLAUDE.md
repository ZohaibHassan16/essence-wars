# python - Python Bindings & ML CLAUDE.md

<!-- Last verified: 2026-02-05 -->

## Overview

Python bindings for Essence Wars engine plus ML agent implementations.

## Installation

```bash
pip install essence-wars           # Core package
pip install essence-wars[analysis] # With analysis tools
uv sync --group analysis           # Development (with uv)
```

## CLI

The `essence-wars` command provides unified access to ML tools:

```bash
essence-wars --help

# Training
essence-wars train ppo --timesteps 500000
essence-wars train alphazero --iterations 100
essence-wars train behavioral-cloning --data data.jsonl.gz
essence-wars train card2vec --data data.jsonl.gz --embed-dim 64

# Benchmarking
essence-wars benchmark --checkpoint model.pt
essence-wars benchmark --checkpoint model.pt --full-eval

# Reports
essence-wars report generate --run-id latest --open
essence-wars report leaderboard

# Data generation
essence-wars data generate-distillation --games 10000 --output data.jsonl.gz
```

## Core API

```python
from essence_wars import PyGame, PyParallelGames

# Single game
game = PyGame()
game.reset(seed=42)
obs = game.observe()         # numpy array (328,)
mask = game.action_mask()    # numpy array (256,)
reward, done = game.step(action)

# Vectorized (for training) - RECOMMENDED
from essence_wars import VectorizedEssenceWars
vec_env = VectorizedEssenceWars(num_envs=64)
obs, masks = vec_env.reset(seed=42)
obs, rewards, dones, masks = vec_env.step(actions)  # Accepts any int array

# Raw parallel API (advanced) - requires uint8 actions
games = PyParallelGames(num_envs=64)
games.reset_all(base_seed=42)
obs = games.observe_batch()           # (64, 328)
actions = np.array([...], dtype=np.uint8)  # MUST be uint8
rewards, dones = games.step_batch(actions)
```

**Note**: Use `VectorizedEssenceWars` for training - it handles type casting and auto-reset.
The raw `PyParallelGames.step_batch()` requires `np.uint8` actions.

## Directory Structure

```
essence_wars/
├── __init__.py        # Core exports: PyGame, PyParallelGames
├── env.py             # Gymnasium environments
├── parallel_env.py    # PettingZoo multi-agent
├── agents/            # PPO, AlphaZero, Card2Vec
├── analysis/report/   # HTML report generator
├── ratings/           # Deck + agent ELO tracking
├── training/          # Training utilities
├── data/              # Dataset loaders
└── hub.py             # HuggingFace Hub integration
scripts/
├── training/          # Training scripts
├── evaluation/        # Benchmarks
├── data/              # Dataset generation
└── reporting/         # HTML reports
tests/
└── test_core.py       # Core binding tests
```

## Testing

```bash
uv run pytest python/tests
uv run mypy python/essence_wars
uv run ruff check python/essence_wars
```

## Key Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `STATE_TENSOR_SIZE` | 328 | Observation shape |
| `ACTION_SPACE_SIZE` | 256 | Action space size |
