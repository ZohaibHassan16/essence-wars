# cardgame - Core Engine CLAUDE.md

<!-- Last verified: 2026-02-05 -->

## Overview

The core Essence Wars game engine. Handles game state, combat resolution, effect processing, and AI bots.

## Key Directories

| Path | Purpose |
|------|---------|
| `src/core/` | Game types, state, actions, combat |
| `src/bots/` | RandomBot, GreedyBot, MctsBot, AlphaBetaBot |
| `src/tensor.rs` | AI state tensor encoding |
| `src/bin/` | CLI tools: arena, swiss, tune, validate, benchmark, diagnose, replay |
| `tests/unit/` | Unit tests (separate from src) |

## Bot System

| Bot | Description |
|-----|-------------|
| `random` | Uniform random selection |
| `greedy` | Heuristic evaluation (28 tunable weights) |
| `mcts` | UCB1 tree search with greedy rollouts |
| `alphabeta` | Alpha-Beta minimax search (depth 6-8 recommended) |

### Bot Trait

```rust
pub trait Bot: Send {
    fn select_action(&mut self, state_tensor: &[f32; 328],
                     legal_mask: &[f32; 256], legal_actions: &[Action]) -> Action;
    fn name(&self) -> &str;
    fn reset(&mut self);
    fn clone_box(&self) -> Box<dyn Bot>;
}
```

### Alpha-Beta Bot

Minimax with alpha-beta pruning. 60-70% win rate vs MCTS-1000 at depth 8.

```bash
cargo run --release --bin arena -- --bot1 alphabeta --bot2 mcts --ab-depth 8 --games 50
```

Performance: Depth 6 ~8s/game, Depth 8 ~50s/game, Depth 10+ too slow.

### MCTS Bot

Monte Carlo Tree Search with UCB1 selection and greedy rollouts.

```rust
MctsConfig::default()           // 1000 sims, sequential
MctsConfig::fast()              // 100 sims, for testing
MctsConfig::strong()            // 5000 sims, serious play
MctsConfig::interactive(sims)   // Parallel trees (MCP/UI)
MctsConfig::batch(sims)         // Sequential, for arena
```

**Optimizations (enabled by default):**
- Early termination: Stop rollouts at ±300 evaluation
- Transposition table: Cache position evaluations
- `tt_min_visits: 3`: Minimum visits before trusting cache

### GreedyBot Weights (28 params)

Life, creature stats, board state, resources, keywords (guard/lethal/lifesteal/rush/ranged/piercing/shield/quick), terminal bonuses. See `src/bots/weights.rs`.

## State Tensor (328 floats)

| Section | Indices | Size | Description |
|---------|---------|------|-------------|
| Global state | 0-5 | 6 | Turn number, current player, game state |
| Player 1 state | 6-80 | 75 | Life, essence, AP, deck/hand info, board |
| Player 2 state | 81-155 | 75 | Same as P1 |
| Card embeddings | 156-325 | 170 | Card IDs from hands/boards (normalized) |
| Commander IDs | 326-327 | 2 | P1 commander (326), P2 commander (327) |

**Player state breakdown (75 floats each):**
- Base stats: 5 (life, essence, AP, deck size, hand size)
- Hand cards: 10 (normalized card IDs)
- Creatures: 50 (5 slots × 10 floats)
- Supports: 10 (2 slots × 5 floats)

**Normalization:** Card IDs divided by 6000.0 (handles cards up to 4074, commanders up to 5011)

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
    fn get_state_tensor(&self) -> [f32; 328];
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
| `generalist` | Cross-deck optimization (all archetypes) |
| `archetype` | Playstyle-specific (aggro, control, tempo, midrange) |
| `specialist` | Specific deck matchup |

```bash
# Tune all archetypes
./scripts/tune-archetypes.sh

# Tune specific ones
./scripts/tune-archetypes.sh aggro tempo

# Preview commands
./scripts/tune-archetypes.sh --dry-run
```

## Balance Tools

### Quick Validation (`validate`)

Fast balance check with Greedy bot. ~1 sec for default 10 games/matchup.

```bash
cargo run --release --bin validate -- --progress
cargo run --release --bin validate -- -n 50 --progress
```

### Thorough Benchmark (`benchmark`)

Alpha-Beta (default) or MCTS. 50 games/matchup default.

```bash
cargo run --release --bin benchmark -- --progress              # Alpha-Beta depth 6
cargo run --release --bin benchmark -- --ab-depth 8            # Deeper search
cargo run --release --bin benchmark -- --bot mcts --mcts-sims 200
```

**Output:** Per-deck win rates with 95% CI, best/worst matchups, visual indicators (▲ >60%, ▼ <40%).

### Swiss Tournaments

Full tournaments with score-based pairing. Updates `data/ratings/deck_elo.json`.

```bash
cargo run --release --bin swiss -- --games 20 --progress
cargo run --release --bin swiss -- --faction argentum --games 30
```

### Game Replay

```bash
# Generate and summarize
cargo run --release --bin replay -- --seed 12345 --deck1 X --deck2 Y

# Interactive step-through
cargo run --release --bin replay -- --seed 12345 --deck1 X --deck2 Y -i

# Export transcript
cargo run --release --bin replay -- --seed 12345 --deck1 X --deck2 Y --export transcript -o game.txt
```

## Performance Benchmarks

| Benchmark | Result | Notes |
|-----------|--------|-------|
| Random game | ~33k/sec | ~30 µs/game |
| Greedy game | ~4.3k/sec | ~230 µs/game |
| MCTS 100 sims | ~22 ms | Per move |
| Engine fork | ~245 ns | State cloning |
| State tensor | ~158 ns | 328-float encoding |
| Legal actions | ~55 ns | Action enumeration |

```bash
cargo bench -p cardgame
```
