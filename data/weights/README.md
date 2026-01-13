# Bot Weights

This directory contains weight configurations for GreedyBot and MctsBot evaluation functions.

## Files

- **`default.toml`** - Current "blessed" default weights, automatically loaded by bots
- **`tuned_multi_opponent.toml`** - Tuned weights optimized vs multiple opponents
- **`tuned_multi_opponent_long.toml`** - Extended tuning run (source of current defaults)

## Usage

### Automatic Loading

Both GreedyBot and MctsBot automatically load `default.toml` when constructed:

```bash
# Uses default.toml automatically
cargo run --release --bin arena -- --bot1 greedy --bot2 random --games 100
cargo run --release --bin arena -- --bot1 mcts --bot2 random --games 20
```

### Override with Custom Weights

Use the `--weights` flag to test different configurations:

```bash
# Test specific weights
cargo run --release --bin arena -- \
  --bot1 greedy --bot2 greedy \
  --weights1 experiments/mcts/2026-01-12_HHMM_tag/weights.toml \
  --games 100
```

### Updating Defaults

After tuning and benchmarking new weights:

```bash
# Copy tuned weights as new defaults
cp experiments/mcts/YYYY-MM-DD_HHMM_tag/weights.toml data/weights/default.toml

# Commit to version control
git add data/weights/default.toml
git commit -m "Update default weights: improved vs multi-opponent"
```

## Fallback Behavior

If `default.toml` is missing or corrupted, bots fall back to hardcoded default weights defined in [src/bots/weights.rs](../../src/bots/weights.rs).

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
