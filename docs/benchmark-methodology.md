# Benchmark Methodology

This document describes how agents are evaluated in the Essence Wars benchmark suite.

---

## Overview

The Essence Wars benchmark provides a standardized evaluation protocol for comparing AI agents. All agents on the [leaderboard](https://huggingface.co/spaces/Chris-Essence-Wars/essence-wars-leaderboard) are evaluated using this methodology.

**Key Principles:**
- **Reproducibility**: Deterministic game engine, fixed seeds, documented protocol
- **Fairness**: Balanced deck selection, alternating starting positions
- **Statistical rigor**: 400+ games for official rankings, confidence intervals reported

---

## Baselines

All agents are evaluated against four standard baselines:

| Baseline | Elo | Description | Strength |
|----------|-----|-------------|----------|
| **RandomBot** | 1000 | Uniform random action selection | Weakest baseline |
| **GreedyBot** | 1300 | Heuristic evaluation with 24 tuned weights | Medium |
| **MCTS-50** | 1450 | Monte Carlo Tree Search, 50 simulations/move | Strong |
| **MCTS-100** | 1500 | Monte Carlo Tree Search, 100 simulations/move | Strongest |

### Baseline Descriptions

**RandomBot**: Selects uniformly at random from legal actions. Any trained agent should achieve >95% win rate against Random.

**GreedyBot**: Evaluates each legal action by simulating it one step forward and computing a heuristic score. Uses 24 tuned weights covering:
- Life differential
- Board state (creature count, total stats)
- Card advantage (hand + deck sizes)
- Keyword bonuses (Guard, Lethal, etc.)
- Terminal state bonuses

**MCTS-50/100**: Monte Carlo Tree Search with UCB1 selection and GreedyBot rollouts. The number indicates simulations per move. Uses tuned exploration constant (c=1.4).

---

## Evaluation Protocol

### Quick Evaluation (40 games)

Used for rapid iteration during training and local submissions:

```python
from essence_wars.benchmark import EssenceWarsBenchmark, NeuralAgent

agent = NeuralAgent.from_checkpoint("model.pt")
benchmark = EssenceWarsBenchmark()
results = benchmark.quick_evaluate(agent, games=40)
# 20 games vs Random + 20 games vs Greedy
```

**Output:**
- Win rate vs Random
- Win rate vs Greedy
- Estimated Elo rating

**Confidence interval:** ±63 Elo (based on 40 games)

### Full Evaluation (400 games)

Used for official leaderboard rankings:

```python
results = benchmark.evaluate(agent, games_per_opponent=100)
# 100 games vs each of: Random, Greedy, MCTS-50, MCTS-100
```

**Output:**
- Win rates vs all 4 baselines
- Elo rating with confidence interval
- Average game length
- Average decision time

**Confidence interval:** ±20 Elo (based on 400 games)

---

## Game Setup

### Deck Selection

Each game uses randomly selected decks from the 12 available commander decks:

| Faction | Decks |
|---------|-------|
| Argentum | architect_fortify, sentinel_guard, ironclad_assault, clockwork_combo |
| Symbiote | hivemind_swarm, spore_spread, regen_tank, death_trigger |
| Obsidion | shadow_strike, lifedrain_control, stealth_burst, quick_combo |

Both players' decks are selected independently and uniformly at random, ensuring coverage of all matchup types.

### Starting Positions

Players alternate who goes first:
- Games 1, 3, 5, ... : Evaluated agent goes first
- Games 2, 4, 6, ... : Baseline goes first

This balances any first-player advantage.

### Seeds

Games use deterministic seeds based on game index:
- Quick eval: Seeds 1000 + game_index
- Full eval: Seeds 1000 + game_index

This ensures reproducibility while providing variety across games.

---

## Elo Rating System

### Formula

We use the standard Elo rating system with K-factor 32:

```
Expected_A = 1 / (1 + 10^((Rating_B - Rating_A) / 400))
New_Rating_A = Rating_A + K × (Actual - Expected_A)
```

Where:
- `K = 32` (standard for games)
- `Actual = 1` for win, `0.5` for draw, `0` for loss

### Initialization

Baseline Elo ratings are set empirically based on cross-play:

| Baseline | Initial Elo |
|----------|-------------|
| RandomBot | 1000 |
| GreedyBot | 1300 |
| MCTS-50 | 1450 |
| MCTS-100 | 1500 |

New agents start at 1500 Elo and are adjusted based on results against baselines.

### Confidence Intervals

Confidence intervals are computed as:

```
CI = 400 / sqrt(games_played)
```

For 400 games: CI ≈ ±20 Elo

---

## Win Rate to Elo Conversion

For quick Elo estimation from win rate vs GreedyBot (Elo 1300):

```python
import math

def win_rate_to_elo(win_rate_vs_greedy):
    if win_rate_vs_greedy <= 0:
        return 1000
    if win_rate_vs_greedy >= 1:
        return 1600
    elo_diff = -400 * math.log10((1 / win_rate_vs_greedy) - 1)
    return 1300 + elo_diff
```

**Reference table:**

| Win Rate vs Greedy | Estimated Elo |
|-------------------|---------------|
| 10% | 1059 |
| 25% | 1180 |
| 50% | 1300 |
| 75% | 1420 |
| 90% | 1541 |
| 95% | 1618 |

---

## Agent Interface

All evaluated agents must implement the `BenchmarkAgent` protocol:

```python
from essence_wars.benchmark import BenchmarkAgent
import numpy as np

class MyAgent(BenchmarkAgent):
    @property
    def name(self) -> str:
        return "MyAgent"

    def select_action(
        self,
        observation: np.ndarray,  # Shape: (326,)
        action_mask: np.ndarray,  # Shape: (256,), 1.0 = legal
    ) -> int:
        # Your logic here
        valid_actions = np.where(action_mask > 0.5)[0]
        return int(valid_actions[0])  # Example: first legal action

    def reset(self) -> None:
        # Called at start of each game
        pass
```

### Input Specifications

**Observation** (326 floats):
- Normalized to [0, 1] range where applicable
- See [State Tensor documentation](./design-engine.md) for full breakdown

**Action Mask** (256 floats):
- `1.0` = legal action
- `0.0` = illegal action
- Agent must only select actions where mask > 0.5

### Output Specification

**Action** (int):
- Must be in range [0, 255]
- Must correspond to a legal action (mask[action] > 0.5)
- Invalid actions result in automatic loss

---

## Running Evaluations

### Command Line

```bash
# Quick evaluation
python python/scripts/run_benchmark.py \
    --checkpoint model.pt \
    --output results.json

# Full evaluation
python python/scripts/run_benchmark.py \
    --checkpoint model.pt \
    --output results.json \
    --full-eval
```

### Python API

```python
from essence_wars.benchmark import EssenceWarsBenchmark, NeuralAgent

# Load agent
agent = NeuralAgent.from_checkpoint("model.pt")

# Create benchmark
benchmark = EssenceWarsBenchmark(
    games_per_opponent=100,
    verbose=True,
)

# Run evaluation
results = benchmark.evaluate(agent)

# Access results
print(f"Elo: {results.elo_rating}")
print(f"vs Greedy: {results.win_rate_vs_greedy:.1%}")
print(f"vs MCTS-100: {results.win_rate_vs_mcts100:.1%}")

# Save to JSON
results.save("benchmark_results.json")
```

---

## Leaderboard Submission

### Requirements

1. **Checkpoint format**: Must be loadable by `NeuralAgent.from_checkpoint()`
2. **Minimum performance**: No minimum, but <10% vs Greedy triggers a warning
3. **Reproducibility**: Checkpoint must produce consistent results

### Submission Methods

1. **Local**: `python python/scripts/submit_agent.py --checkpoint model.pt`
2. **HuggingFace**: Upload to HF Hub, then submit
3. **GitHub Issue**: Open issue with `evaluate-agent` label for automated evaluation

See [SUBMIT_AGENT.md](./SUBMIT_AGENT.md) for detailed instructions.

---

## Reproducibility

### Determinism Guarantees

The Essence Wars engine is fully deterministic:
- Same seed → same game state sequence
- No floating-point non-determinism
- No random number generator leakage between games

### Reproducing Results

```python
# Reproduce a specific game
game = PyGame(deck1="architect_fortify", deck2="hivemind_swarm")
game.reset(seed=1042)

# Game will play out identically every time with same actions
```

### Version Compatibility

Benchmark results include the engine version. Results from different versions may not be directly comparable if game mechanics changed.

---

## Known Limitations

1. **Deck diversity**: 12 decks may not cover all strategic space
2. **Opponent variety**: Training against only GreedyBot may lead to overfitting
3. **Game length**: 30-turn limit may affect strategies differently
4. **First-player advantage**: ~52% for first player with GreedyBot mirror

---

## References

- [Leaderboard](https://huggingface.co/spaces/Chris-Essence-Wars/essence-wars-leaderboard)
- [Submission Guide](./SUBMIT_AGENT.md)
- [Engine Design](./design-engine.md)
- [Game Rules](./essence-wars-design.md)
