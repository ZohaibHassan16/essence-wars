# CLAUDE.md - AI Assistant Context for Essence Wars

## Project Overview

**Essence Wars** is a deterministic, perfect-information card game engine designed for AI research (reinforcement learning, MCTS). Written in Rust with focus on performance and correctness.

**Current Version:** 0.7.0 | **Author:** Christian Wissmann (Chris), Best Friends with Claude

## Quick Commands

```bash
# Build
cargo build --release                    # All crates
cargo build --release -p cardgame        # Core engine only
cargo build --release -p essence-wars-3d # 3D client only

# Test
cargo nextest run --status-level=fail    # ~629 tests (recommended)
cargo test                               # Alternative

# Lint
./scripts/run-clippy.sh                  # Recommended: lib + binaries

# Stress tests by tier
./scripts/run-tests.sh                   # Standard only
./scripts/run-tests.sh quick|medium|long|overnight

# Arena matches
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 100 --progress
cargo run --release --bin arena -- --list-decks

# Weight tuning
cargo run --release --bin tune -- --mode generalist --tag my_run --generations 50
cargo run --release --bin tune -- --mode faction-specialist --faction argentum --tag argentum_v1

# Analysis & validation
./scripts/analyze-tuning.sh --latest
cargo run --release --bin validate -- --games 100
cargo run --release --bin diagnose -- 200

# Benchmarks
cargo bench -p cardgame

# 3D client
cargo run --release -p essence-wars-3d

# Modal cloud (setup: uv tool install modal && modal token new)
modal run modal_tune.py::main
```

## Project Structure

**Cargo workspace** with two crates:

| Crate | Purpose |
|-------|---------|
| `cardgame` | Core game engine, AI bots, tuning infrastructure |
| `essence-wars-3d` | Bevy 3D client with Glassbox AI visualization |

**Key locations:**
- `crates/cardgame/src/` - Core engine source (~25k lines)
- `crates/cardgame/src/core/` - Game types, state, actions, combat
- `crates/cardgame/src/engine/` - GameEngine, effect processing
- `crates/cardgame/src/bots/` - RandomBot, GreedyBot, MctsBot
- `crates/cardgame/src/bin/` - CLIs: arena, tune, validate, diagnose
- `crates/cardgame/tests/unit/` - Unit tests (separate from src)
- `crates/essence-wars-3d/src/` - Bevy 3D client
- `python/essence_wars/` - Python bindings and ML agents
- `python/essence_wars/agents/` - PPO, AlphaZero, Card2Vec, embeddings
- `python/essence_wars/data/` - Dataset loaders (MCTSDataset, ChunkedMCTSDataset)
- `python/scripts/` - Training scripts (train_ppo.py, train_alphazero.py, etc.)
- `data/cards/core_set/` - 300 cards in 4 YAML files (by faction)
- `data/decks/{argentum,symbiote,obsidion}/` - 12 commander decks
- `data/datasets/` - MCTS game datasets (JSONL.gz)
- `data/weights/` - Tuned bot weights (generalist + specialists)
- `models/` - Trained ML models (Card2Vec, BC, PPO checkpoints)
- `experiments/` - Training outputs (GITIGNORED)
- `docs/` - Design docs, guides

## Test Organization

**IMPORTANT**: Unit tests are **separate from source code** (not inline `#[cfg(test)]` blocks).

- **Unit tests**: `crates/cardgame/tests/unit/<module>_tests.rs`
- **Integration tests**: `crates/cardgame/tests/*.rs`
- **Shared utilities**: `crates/cardgame/tests/common/mod.rs`

When adding tests, create in `tests/unit/` and add module to `tests/unit.rs`.

### Test Tiers

| Tier | Duration | When |
|------|----------|------|
| Standard | ~2 min | Every commit |
| tier_quick | ~2 min | PRs |
| tier_medium | ~10 min | Nightly |
| tier_long | ~30 min | Nightly |
| tier_overnight | ~2 hours | Weekly |

## Bot System

| Bot | Description |
|-----|-------------|
| `random` | Uniform random selection |
| `greedy` | Heuristic evaluation (24 tunable weights) |
| `mcts` | UCB1 tree search with greedy rollouts |
| `agent-{faction}` | MCTS with faction-specialist weights |
| `agent-generalist` | MCTS with cross-faction weights |

### Bot Trait

```rust
pub trait Bot: Send {
    fn select_action(&mut self, state_tensor: &[f32; 326],
                     legal_mask: &[f32; 256], legal_actions: &[Action]) -> Action;
    fn name(&self) -> &str;
    fn reset(&mut self);
    fn clone_box(&self) -> Box<dyn Bot>;
}
```

### GreedyBot Weights (24 params)

Life, creature stats, board state, resources, keywords (guard/lethal/lifesteal/rush/ranged/piercing/shield/quick), terminal bonuses. See `crates/cardgame/src/bots/weights.rs`.

### Bot Introspection

`MctsBot` provides introspection for Glassbox visualization:
```rust
let bot = MctsBot::new(1000).with_introspection(IntrospectionConfig::full());
// Access via bot.last_decision() -> Option<BotDecision>
```

## Game Rules

- 5 creature slots, 2 support slots per player
- 3 Action Points per turn
- 30 turn limit with life-based tiebreaker
- 14 keywords: Rush, Ranged, Piercing, Guard, Lifesteal, Lethal, Shield, Quick, Ephemeral, Regenerate, Stealth, Charge, Frenzy, Volatile

## Faction System

| Faction | Keywords | Playstyle |
|---------|----------|-----------|
| **Argentum Combine** | Guard, Piercing, Shield | Defensive, outlast |
| **Symbiote Circles** | Rush, Lethal, Regenerate | Aggressive tempo |
| **Obsidion Syndicate** | Lifesteal, Stealth, Quick | Burst damage |
| **Free-Walkers** (neutral) | Ranged, Charge | Utility splash |

**Deck composition**: 30 cards = 21 faction (70%) + 9 neutral (30%)

## Card System

- **300 cards** total (75 per faction)
- **ID ranges**: Argentum 1000-1074, Symbiote 2000-2074, Obsidion 3000-3074, Neutral 4000-4074
- **Files**: `data/cards/core_set/{argentum,symbiote,obsidion,neutral}.yaml`

### YAML Schema

```yaml
# Creature
- id: 1000
  name: "Brass Sentinel"
  cost: 2
  card_type: creature
  attack: 2
  health: 4
  keywords: [Guard]
  rarity: Common
  tags: [Construct]

# Spell
- id: 1010
  name: "Reinforce"
  cost: 2
  card_type: spell
  targeting: TargetAllyCreature
  effects:
    - type: buff_stats
      attack: 0
      health: 3

# Support
- id: 1013
  name: "Assembly Line"
  cost: 4
  card_type: support
  durability: 3
  triggered_effects:
    - trigger: StartOfTurn
      effects:
        - type: heal
          amount: 2
```

**Advanced features**: filters (`max_health`, `has_keyword`), conditional effects (`target_died`), bounce. See `docs/cards-new-horizons.md`.

## Deck System

12 commander decks (4 per faction). Use `cargo run --release --bin arena -- --list-decks` to see all.

**Format** (`data/decks/{faction}/{deck_id}.toml`):
```toml
id = "architect_fortify"
name = "Architect's Bastion"
commander = 1060
cards = [1060, 1001, 1002, ...]  # 30 total
```

## State Tensor (326 floats)

| Section | Size |
|---------|------|
| Global (turn, phase, etc.) | 10 |
| Player 1/2 creatures | 60 each (5 slots × 12) |
| Player 1/2 supports | 8 each (2 slots × 4) |
| Player 1/2 hands | 60 each (20 cards × 3) |
| Player 1/2 decks | 30 each |

## Action Space (256 indices)

| Range | Action |
|-------|--------|
| 0-99 | PlayCard (hand × slot) |
| 100-149 | Attack (slot × target) |
| 150-249 | UseAbility (slot × ability × target) |
| 255 | EndTurn |

## AI Interface (GameEnvironment trait)

```rust
trait GameEnvironment {
    fn get_state_tensor(&self) -> [f32; 326];
    fn get_legal_action_mask(&self) -> [f32; 256];
    fn apply_action_by_index(&mut self, index: u8);
    fn get_reward(&self, player: PlayerId) -> f32;  // -1.0, 0.0, or 1.0
    fn fork(&self) -> Self;  // Clone for MCTS
}
```

## Weight Tuning

CMA-ES optimizer with parallel evaluation. Outputs to `experiments/mcts/YYYY-MM-DD_HHMM_tag/`.

| Mode | Use Case |
|------|----------|
| `generalist` | Cross-faction optimization |
| `specialist` | Specific deck matchup |
| `faction-specialist` | Faction-wide optimization |

See `docs/tuning-pipeline.md` for full options.

## Performance

| Benchmark | Throughput |
|-----------|------------|
| Random game | ~80k/sec |
| Greedy game | ~17k/sec |
| Engine fork | ~99 ns |

## Bevy 3D Client

```bash
cargo run --release -p essence-wars-3d
```

Press **'G'** to toggle Glassbox AI visualization (MCTS tree, action probabilities, value gauge).

See `crates/essence-wars-3d/README.md` for details.

## Python/ML Infrastructure

Published to PyPI as `essence-wars`. See `python/README.md` for full docs.

```bash
pip install essence-wars
```

```python
from essence_wars import PyGame, PyParallelGames, EssenceWarsEnv
```

- Gymnasium v26+ with action masking
- PettingZoo multi-agent
- PPO and AlphaZero trainers
- ~268k steps/sec vectorized

### Training Scripts

| Script | Purpose | Memory Safe |
|--------|---------|-------------|
| `train_ppo.py` | PPO reinforcement learning | ✅ (streaming) |
| `train_alphazero.py` | AlphaZero self-play | ✅ (bounded buffer) |
| `train_behavioral_cloning.py` | Imitation learning on MCTS data | ✅ (with `--streaming`) |
| `train_card2vec.py` | Card embedding pre-training | ✅ (reservoir sampling) |

### Memory Safety Guidelines

**CRITICAL**: Large datasets (10k+ games) can cause OOM crashes on WSL2/limited RAM systems.

**Safe patterns used:**
- `deque(maxlen=N)` - Bounded replay buffers (AlphaZero)
- Reservoir sampling with `max_pairs` - Bounded pair collection (Card2Vec)
- `ChunkedMCTSDataset` - Streaming with bounded chunks (BC)
- Pre-allocated tensors - Fixed rollout buffers (PPO)

**Memory estimates for MCTS datasets:**
- 1k games: ~90k samples → ~315 MB
- 10k games: ~900k samples → ~3.1 GB
- 100k games: ~9M samples → ~31 GB ⚠️

### Behavioral Cloning

```bash
# Standard mode (loads all into memory) - use for small datasets
uv run python python/scripts/train_behavioral_cloning.py \
    --dataset data/datasets/mcts_10k_sims100_*.jsonl.gz \
    --epochs 50

# Streaming mode (memory-efficient) - use for large datasets
uv run python python/scripts/train_behavioral_cloning.py \
    --dataset data/datasets/mcts_100k_sims100_*.jsonl.gz \
    --streaming \
    --chunk-size 50000 \
    --epochs 50
```

### Card2Vec Embeddings

Pre-train card embeddings using co-occurrence and attribute prediction:

```bash
uv run python python/scripts/train_card2vec.py \
    --dataset data/datasets/mcts_10k_sims100_*.jsonl.gz \
    --max-pairs 500000 \
    --epochs 100 \
    --output models/card2vec.pt
```

Use embeddings in training:
```bash
# PPO with pre-trained embeddings
uv run python python/scripts/train_ppo.py \
    --observation-mode embedded_pretrained \
    --pretrained-embeds models/card2vec.pt

# AlphaZero with pre-trained embeddings
uv run python python/scripts/train_alphazero.py \
    --observation-mode embedded_pretrained \
    --pretrained-embeds models/card2vec.pt
```

### AlphaZero Training

```bash
# Quick test (5 iterations)
uv run python python/scripts/train_alphazero.py \
    --iterations 5 --games-per-iter 20 --sims 50

# Full training with checkpoints
uv run python python/scripts/train_alphazero.py \
    --iterations 200 \
    --sims 100 \
    --checkpoint-interval 10 \
    --tensorboard

# Fine-tune from behavioral cloning
uv run python python/scripts/train_alphazero.py \
    --iterations 100 \
    --load models/bc_mcts_100k.pt
```

### PPO Training

```bash
# Standard flat observation
uv run python python/scripts/train_ppo.py \
    --timesteps 1000000 \
    --observation-mode flat

# With learned embeddings
uv run python python/scripts/train_ppo.py \
    --timesteps 1000000 \
    --observation-mode embedded \
    --embed-dim 64
```

## Data & Experiments

```
experiments/           # GITIGNORED - all training artifacts
data/cards/           # Card definitions (YAML)
data/decks/           # Deck definitions (TOML)
data/datasets/        # MCTS game datasets (JSONL.gz)
data/weights/         # Tuned bot weights (committed)
models/               # Trained ML models (Card2Vec, BC, etc.)
```

### MCTS Datasets

Generated with `cargo run --release --bin generate_mcts_data`:

| Dataset | Games | Samples | Size | Use Case |
|---------|-------|---------|------|----------|
| `mcts_1k_sims100_*.jsonl.gz` | 1,000 | ~90k | ~60 MB | Quick tests |
| `mcts_10k_sims100_*.jsonl.gz` | 10,000 | ~900k | ~565 MB | Training |
| `mcts_100k_sims100_*.jsonl.gz` | 100,000 | ~9M | ~5.6 GB | Full training (use `--streaming`) |

Also published on HuggingFace: [`ChristianWissworWo/essence-wars-mcts-games`](https://huggingface.co/Chris-Essence-Wars/datasets)

**Experiment IDs**: `YYYY-MM-DD_HHMM_tag` (e.g., `2026-01-14_0755_generalist-v0.4`)

**Rules**: Always create experiment folders, never commit experiments/, use timestamped tags.

## Versioning

Version in **root `Cargo.toml`** `[workspace.package]`. Both crates share it.

| Change | Bump |
|--------|------|
| Game rules/API | MINOR |
| New features | MINOR |
| Bug fixes | PATCH |

**Update checklist**:
1. Root `Cargo.toml`: `version = "X.Y.Z"`
2. `crates/cardgame/src/version.rs`: Update test
3. `CHANGELOG.md`: Add entry
4. Run tests
