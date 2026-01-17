# Essence Wars

A deterministic, perfect-information card game engine designed for AI research (reinforcement learning, MCTS, neural network training).

## Overview

Essence Wars is a lane-based digital card game with:
- **300 cards** across 3 factions + neutrals (New Horizons Edition)
- **12 pre-built Commander Decks** for balanced matchup testing
- **16 keywords** with rich mechanical interactions
- **Deterministic engine** for reproducible experiments
- **AI interface** with tensor representation (326 floats) and fixed action space (256 actions)

## Quick Start

```bash
# Build
cargo build --release

# Run bot arena matches
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 100 --progress

# Run balance validation (8,000 games, ~2 min)
cargo run --release --bin validate -- --games 100

# Generate interactive dashboard
./scripts/generate-dashboard.sh
# Open docs/dashboard/index.html in browser
```

## Faction Balance

The game features a deliberate **rock-paper-scissors** dynamic:

| Matchup | Favored | Win Rate |
|---------|---------|----------|
| Argentum vs Obsidion | Argentum | ~60% |
| Obsidion vs Symbiote | Obsidion | ~52% |
| Symbiote vs Argentum | Even | ~49% |

Interactive balance visualization: `docs/dashboard/index.html`

### Factions

| Faction | Identity | Playstyle |
|---------|----------|-----------|
| **Argentum Combine** | "The Wall" | Defensive, high-HP constructs, Guard synergy |
| **Symbiote Circles** | "The Swarm" | Aggressive tempo, Rush creatures, death triggers |
| **Obsidion Syndicate** | "The Shadow" | Burst damage, Lifesteal, Stealth assassins |
| **Free-Walkers** | Neutral | Utility cards that splash into any faction |

## Research Features

### AI Interface

```rust
// Get game state as neural network input
let tensor: [f32; 326] = env.get_state_tensor();

// Get legal action mask
let mask: [f32; 256] = env.get_legal_action_mask();

// Apply action from neural network output
env.apply_action_by_index(action_idx);

// Get reward signal
let reward = env.get_reward(player_id);

// Clone state for tree search
let clone = env.fork();
```

### Bot Types

| Bot | Description | Use Case |
|-----|-------------|----------|
| `random` | Uniform random selection | Baseline |
| `greedy` | Heuristic evaluation | Training opponent |
| `mcts` | Monte Carlo Tree Search | Strong benchmark |
| `agent-*` | MCTS with tuned weights | Faction specialists |

### Weight Tuning (CMA-ES)

```bash
# Train generalist weights
cargo run --release --bin tune -- --mode generalist --generations 100

# Train faction specialist
cargo run --release --bin tune -- --mode faction-specialist --faction argentum
```

### Diagnostics

```bash
# P1/P2 asymmetry analysis
cargo run --release --bin diagnose -- 500 --export json

# Text-based validation report
./scripts/analyze-validation.sh --latest
```

## Performance

| Benchmark | Throughput |
|-----------|------------|
| Random games | ~80,000/sec |
| Greedy games | ~17,000/sec |
| State tensor | ~7.2M/sec |
| Engine fork | ~10M/sec |

## Documentation

- `docs/essence-wars-design.md` - Full game rules and mechanics
- `docs/design-engine.md` - Engine architecture and API
- `docs/cards-new-horizons.md` - Complete card database (300 cards)
- `docs/modal-cloud-setup.md` - Cloud training with Modal

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
├── python/                # Analysis and visualization
│   └── cardgame/analysis/ # Dashboards, aggregators
├── scripts/               # Shell wrappers
└── tests/                 # ~576 tests
```

## Requirements

- Rust 1.75+ with Cargo
- Python 3.11+ with uv (for analysis tools)

## License

MIT

## Citation

If you use Essence Wars in your research, please cite:

```bibtex
@software{essence_wars,
  title = {Essence Wars: A Deterministic Card Game Engine for AI Research},
  author = {Wissmann, Christian},
  year = {2026},
  url = {https://github.com/your-repo/essence-wars}
}
```
