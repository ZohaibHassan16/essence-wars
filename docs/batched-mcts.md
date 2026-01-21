# Batched Neural MCTS

**10-20x GPU speedup for Neural MCTS through batched inference**

## Overview

Standard Neural MCTS evaluates leaf nodes one at a time, leaving GPU capacity underutilized. Batched MCTS collects multiple leaf nodes and evaluates them in a single forward pass, achieving near-linear speedup with batch size.

## The Problem

In standard Neural MCTS, each simulation does:
1. **Selection**: Traverse tree using UCB
2. **Evaluation**: Single GPU inference (~1ms)
3. **Expansion**: Create child nodes
4. **Backpropagation**: Update values

With 100 simulations, that's 100 sequential GPU calls. GPUs are optimized for parallel workloads - a batch of 32 states takes almost the same time as a single state.

## The Solution: Virtual Loss + Batched Inference

### Virtual Loss

When collecting multiple leaves simultaneously, we need to prevent all paths from converging to the same node. **Virtual loss** temporarily penalizes nodes being explored:

```python
def apply_virtual_loss(self, virtual_loss: float = 3.0):
    self.visit_count += 1
    self.value_sum -= virtual_loss  # Makes node look worse

def remove_virtual_loss(self, virtual_loss: float = 3.0):
    self.visit_count -= 1
    self.value_sum += virtual_loss  # Restore true value
```

This encourages exploration of different paths during batch collection.

### Batched Search Algorithm

```
while simulations_remaining > 0:
    batch = []

    # Phase 1: Collect leaves with virtual loss
    for i in range(min(batch_size, simulations_remaining)):
        path = select_with_virtual_loss(root)
        if path.leaf is terminal:
            backup_immediately(path)
        else:
            batch.append(path)

    # Phase 2: Batched GPU inference
    if batch:
        policies, values = network.forward(batch.observations)  # Single GPU call!

    # Phase 3: Expand and backup
    for path, policy, value in zip(batch, policies, values):
        path.leaf.expand(policy)
        remove_virtual_loss(path)
        backup(path, value)

    simulations_remaining -= batch_size
```

## Usage

### NeuralMctsBot (for inference/data generation)

```python
from essence_wars.agents.neural_mcts import NeuralMctsBot

bot = NeuralMctsBot(
    network=your_network,
    num_simulations=100,
    device="cuda",
)

# Sequential (baseline) - ~100 GPU calls
action, policy = bot.get_action_with_game(game)

# Batched (10-20x faster) - ~4 GPU calls with batch_size=32
action, policy = bot.get_action_with_game_batched(
    game,
    batch_size=32,      # Leaves per batch
    virtual_loss=3.0,   # Exploration encouragement
)
```

### AlphaZero Training (automatic batching)

```python
from essence_wars.agents.alphazero import AlphaZeroConfig, AlphaZeroTrainer

config = AlphaZeroConfig(
    num_simulations=100,
    mcts_batch_size=32,      # Batched search
    mcts_virtual_loss=3.0,   # Virtual loss value
)

trainer = AlphaZeroTrainer(config)
trainer.train()  # Uses batched search automatically
```

### Command Line

```bash
# Distillation data generation
uv run python python/scripts/generate_distillation_data.py \
    --model models/bc_mcts_values.pt \
    --games 1000 \
    --sims 50 \
    --batch-size 32

# Expert Iteration data
uv run python python/scripts/generate_exit_data.py \
    --model models/bc_mcts_10k_best.pt \
    --games 1000 \
    --sims 50 \
    --batch-size 32

# AlphaZero training
uv run python python/scripts/train_alphazero.py \
    --iterations 100 \
    --mcts-batch-size 32
```

## Performance

Benchmarks on RTX 5070 Ti with 100 simulations per move:

| Method | Batch Size | Avg Time/Move | Speedup |
|--------|------------|---------------|---------|
| Sequential | 1 | ~85ms | 1.0x |
| Batched | 8 | ~12ms | 7.1x |
| Batched | 16 | ~6.5ms | 13.1x |
| Batched | 32 | ~4.5ms | 18.9x |
| Batched | 64 | ~4.2ms | 20.2x |

**Key insight**: Speedup plateaus around batch_size=32-64 as GPU becomes saturated.

## Configuration Guidelines

### Batch Size Selection

| Scenario | Recommended | Rationale |
|----------|-------------|-----------|
| Quick testing | 8-16 | Lower latency |
| Data generation | 32 | Good speedup, stable |
| Full training | 32-64 | Maximum throughput |
| Low VRAM (<8GB) | 16 | Memory constraints |

### Virtual Loss Tuning

| Value | Effect |
|-------|--------|
| 1.0 | Mild exploration, paths may converge |
| 3.0 | Balanced (default) |
| 5.0+ | Strong exploration, may reduce quality |

For most use cases, the default `virtual_loss=3.0` works well.

## Implementation Details

### Files

- `python/essence_wars/agents/neural_mcts.py` - `NeuralMctsBot.get_action_with_game_batched()`
- `python/essence_wars/agents/alphazero.py` - `NeuralMCTS.search_batched()`

### Key Classes

**MCTSNode** (both files):
```python
def apply_virtual_loss(self, virtual_loss: float)
def remove_virtual_loss(self, virtual_loss: float)
```

**NeuralMctsBot**:
```python
def get_action_with_game_batched(
    self,
    game: PyGame,
    batch_size: int = 32,
    virtual_loss: float = 3.0,
) -> tuple[int, NDArray[np.float32]]
```

**NeuralMCTS** (AlphaZero):
```python
def search_batched(
    self,
    game: PyGame,
    batch_size: int = 32,
    add_noise: bool = True,
    virtual_loss: float = 3.0,
) -> NDArray[np.float32]
```

## Troubleshooting

### Low Speedup (<5x)

- **CPU bottleneck**: Tree traversal may dominate. Increase simulations.
- **Small batch size**: Try batch_size=32 or higher.
- **GPU not utilized**: Check `device="cuda"` is set.

### Quality Degradation

- **Too much virtual loss**: Reduce to 1.0-2.0.
- **Batch size too large**: With very large batches, early selections become stale. Keep batch_size <= simulations/2.

### Out of Memory

- Reduce batch_size to 16 or lower.
- Use mixed precision (if supported).

## References

- **AlphaGo Zero**: Silver et al., 2017 - Original neural MCTS
- **Virtual Loss**: Standard technique in parallel MCTS (Chaslot et al., 2008)
- **Batched MCTS**: Common in production systems (Leela Chess Zero, KataGo)
