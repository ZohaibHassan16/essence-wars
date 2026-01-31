# Bot Weights

This directory contains weight configurations for GreedyBot, MctsBot, and AlphaBetaBot evaluation functions.

## Files

- **`generalist.toml`** - Auto-deployed generalist weights for Greedy/MCTS (all decks)
- **`specialists/`** - Auto-deployed faction specialist weights for Greedy/MCTS
  - `argentum.toml` - Optimized for Argentum faction
  - `symbiote.toml` - Optimized for Symbiote faction
  - `obsidion.toml` - Optimized for Obsidion faction
- **`alphabeta/`** - Auto-deployed weights for Alpha-Beta bot
  - `generalist.toml` - Cross-faction weights (auto-deployed)
  - `specialists/` - Faction-specific Alpha-Beta weights (auto-deployed)
- **`default.toml`** - Legacy fallback (deprecated, use generalist.toml)

## Auto-Deploy System

Weights are **automatically deployed** after tuning completes:

```bash
# Train generalist → auto-deploys to generalist.toml
cargo run --release --bin tune -- --mode generalist --tag gen-v0.4

# Train specialist → auto-deploys to specialists/argentum.toml
cargo run --release --bin tune -- --mode faction-specialist --faction argentum --tag arg-v0.4

# Train Alpha-Beta weights → auto-deploys to alphabeta/generalist.toml
cargo run --release --bin tune -- --mode alphabeta --ab-depth 6 --generations 30 --tag ab-v1

# Train Alpha-Beta specialist → auto-deploys to alphabeta/specialists/argentum.toml
cargo run --release --bin tune -- --mode alphabeta-specialist --faction argentum --ab-depth 8 --tag ab-arg-v1
```

**All modes now auto-deploy!** No manual copying needed.

## Usage

### Automatic Loading

**GreedyBot and MctsBot** automatically load weights from:
1. **Generalist weights**: `data/weights/generalist.toml` (most common)
2. **Specialist weights**: `data/weights/specialists/{faction}.toml` (when using faction decks)
3. **Fallback**: Hardcoded defaults in `src/bots/weights.rs`

**AlphaBetaBot** automatically loads weights from:
1. **Alpha-Beta specific**: `data/weights/alphabeta/generalist.toml` (preferred)
2. **Shared generalist**: `data/weights/generalist.toml` (fallback)
3. **Hardcoded defaults**: `src/bots/weights.rs`

```bash
# Uses generalist.toml automatically
cargo run --release --bin arena -- --bot1 greedy --bot2 random --games 100
cargo run --release --bin arena -- --bot1 mcts --bot2 random --games 20

# Uses alphabeta/generalist.toml automatically (NEW!)
cargo run --release --bin arena -- --bot1 alphabeta --bot2 mcts --ab-depth 8 --games 50
```

### Override with Custom Weights

Use the `--weights` flag to test different configurations:

```bash
# Test specific weights
cargo run --release --bin arena -- \
  --bot1 greedy --bot2 greedy \
  --weights1 experiments/mcts/2026-01-14_0719_generalist-v0.4/weights.toml \
  --games 100

# Compare generalist vs specialist
cargo run --release --bin arena -- \
  --bot1 greedy --weights1 data/weights/generalist.toml \
  --bot2 greedy --weights2 data/weights/specialists/argentum.toml \
  --deck1 architect_fortify --deck2 broodmother_pack \
  --games 500
```

## Version Control

**DO commit** these files when you've validated improvements:

```bash
# After tuning and testing
git add data/weights/generalist.toml
git add data/weights/specialists/*.toml
git commit -m "Update tuned weights v0.4: [describe improvements]"
```

Include benchmarks in commit message:
- Win rate vs baseline
- Matchup-specific improvements
- Training configuration used

## Fallback Behavior

If weight files are missing or corrupted:
1. Bots fall back to hardcoded defaults in [src/bots/weights.rs](../../src/bots/weights.rs)
2. Warning logged (if verbose mode enabled)
3. Continues execution normally

This ensures bots always work, even in fresh clones.

## See Also

- [tuning-pipeline.md](../../docs/tuning-pipeline.md) - Complete tuning guide
- [src/bots/weights.rs](../../src/bots/weights.rs) - Weight data structures
- [src/tuning/](../../src/tuning/) - Tuning implementation

## Weight Structure

All weight files follow this TOML format:

```toml
name = "config_name"
version = 1

[default.greedy]
own_life = 2.0
enemy_life_damage = 2.0
# ... 24 weight parameters total

[deck_specific]
# Optional deck-specific overrides
```

See [tuning-pipeline-reference.md](../../docs/tuning-pipeline-reference.md) for details on the tuning workflow.
