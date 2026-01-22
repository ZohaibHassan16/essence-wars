# Essence Wars

A deterministic, perfect-information card game engine designed for AI research (reinforcement learning, MCTS).

## Quick Start

```bash
# Install with uv
uv sync --all-groups

# Run tests
cargo nextest run

# Run arena matches
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 100 --progress
```

## Documentation

See `docs/` for detailed documentation.
