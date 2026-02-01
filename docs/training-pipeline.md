# Training Pipeline & Callbacks

*Complete guide to automated training workflows in Essence Wars*

**Last Updated:** 2026-02-01

---

## Overview

The Essence Wars training pipeline provides a callback system for automating common workflows during ML agent training. Callbacks enable automatic checkpointing, evaluation, report generation, and ELO tracking without manual intervention.

## Quick Start

```bash
# Enable all auto-callbacks
essence-wars train ppo \
  --tag my_experiment \
  --auto-callbacks \
  --update-elo

# Or enable individually
essence-wars train ppo \
  --tag my_experiment \
  --auto-evaluate \
  --auto-report
```

---

## Architecture

### Callback System

```
essence_wars/training/callbacks.py
├── CallbackContext          # Context passed to callbacks
├── TrainingCallback         # Abstract base class
├── CallbackList             # Container for multiple callbacks
├── CheckpointCallback       # Periodic model saving
├── EvaluationCallback       # During-training evaluation
├── AutoEvaluateCallback     # Post-training evaluation
├── AutoReportCallback       # Post-training report generation
└── LoggingCallback          # Metric logging
```

### Workflow Automation

```
Training Flow with Callbacks:
┌──────────────────────────────────────────────────────────────┐
│                      Training Loop                           │
│  ┌────────────────────────────────────────────────────┐     │
│  │  on_train_start()  ← Initialize callbacks         │     │
│  └────────────────────────────────────────────────────┘     │
│                         ↓                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │  Training Steps                                    │     │
│  │  ├─ on_step() ← Periodic checkpointing            │     │
│  │  ├─ on_step() ← Evaluation during training        │     │
│  │  └─ on_step() ← Logging                           │     │
│  └────────────────────────────────────────────────────┘     │
│                         ↓                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │  on_train_complete() ← Final actions               │     │
│  │  ├─ Final evaluation (AutoEvaluateCallback)       │     │
│  │  ├─ Update ELO ratings                            │     │
│  │  └─ Generate HTML report (AutoReportCallback)     │     │
│  └────────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────────┘
```

---

## Built-in Callbacks

### CheckpointCallback

Periodically save model checkpoints during training.

**Features:**
- Configurable save frequency
- Keep only N most recent checkpoints
- Save best model by metric
- Timestamp-based naming

**Usage:**
```python
from essence_wars.training.callbacks import CheckpointCallback

callback = CheckpointCallback(
    save_freq=10000,      # Save every 10k steps
    save_path="checkpoints/",
    keep_last=5,          # Keep 5 most recent
    save_best=True,       # Save best by metric
)
```

**CLI:**
```bash
# Checkpointing is enabled by default
essence-wars train ppo --tag exp1
# Checkpoints saved to: experiments/ppo/{timestamp}_exp1/checkpoints/
```

---

### EvaluationCallback

Run evaluation against baselines during training.

**Features:**
- Configurable evaluation frequency
- Multiple opponents
- Progress tracking
- Metric logging

**Usage:**
```python
from essence_wars.training.callbacks import EvaluationCallback

callback = EvaluationCallback(
    eval_freq=50000,      # Evaluate every 50k steps
    eval_games=50,        # Games per opponent
    opponents=["random", "greedy", "mcts-50"],
)
```

**CLI:**
```bash
# Enable during-training evaluation
essence-wars train ppo \
  --tag exp1 \
  --eval-freq 50000 \
  --eval-games 50
```

---

### AutoEvaluateCallback

Run comprehensive evaluation after training completes.

**Features:**
- Full benchmark against all baselines
- Optional ELO rating update
- Results saved to JSON
- Detailed metrics logging

**Usage:**
```python
from essence_wars.training.callbacks import AutoEvaluateCallback

callback = AutoEvaluateCallback(
    eval_games=200,       # Games per opponent
    opponents=["random", "greedy", "mcts-50", "mcts-100"],
    update_elo=True,      # Update agent ELO ratings
)
```

**CLI:**
```bash
# Enable post-training evaluation
essence-wars train ppo \
  --tag exp1 \
  --auto-evaluate \
  --update-elo
```

**Output:**
```
experiments/ppo/{timestamp}_exp1/
├── checkpoints/
├── logs/
└── final_evaluation.json  ← Evaluation results
```

---

### AutoReportCallback

Generate HTML report after training completes.

**Features:**
- Full report with all tabs
- Auto-open in browser (optional)
- Includes final evaluation results
- Linked to ELO leaderboard

**Usage:**
```python
from essence_wars.training.callbacks import AutoReportCallback

callback = AutoReportCallback(
    tabs=["overview", "elo", "benchmark"],  # Customize tabs
    open_browser=True,    # Open report automatically
)
```

**CLI:**
```bash
# Enable post-training report
essence-wars train ppo \
  --tag exp1 \
  --auto-report
```

**Output:**
```
experiments/reports/{timestamp}_exp1/
├── index.html          ← Open this in browser
├── assets/
└── data/
```

---

### LoggingCallback

Log training metrics to console or file.

**Features:**
- Configurable log frequency
- Multiple log destinations
- Structured logging
- Integration with TensorBoard

**Usage:**
```python
from essence_wars.training.callbacks import LoggingCallback

callback = LoggingCallback(
    log_freq=1000,        # Log every 1k steps
    log_file="training.log",
    tensorboard=True,     # Enable TensorBoard logging
)
```

---

## Callback Composition

### Multiple Callbacks

Combine callbacks for full automation:

```python
from essence_wars.training import make_callback

callbacks = make_callback(
    save_path="experiments/my_run",
    checkpoint_freq=10000,
    eval_freq=50000,
    auto_evaluate=True,
    auto_report=True,
    update_elo=True,
)

# Use in training
trainer.train(callbacks=callbacks)
```

### Custom Callback Order

Callbacks execute in the order they're added:

```python
from essence_wars.training.callbacks import CallbackList

callbacks = CallbackList([
    LoggingCallback(),         # First: Log metrics
    CheckpointCallback(),      # Second: Save checkpoint
    EvaluationCallback(),      # Third: Evaluate
])
```

---

## CLI Usage

### Auto-Callbacks Flag

Enable all recommended callbacks with one flag:

```bash
essence-wars train ppo \
  --tag production_v1 \
  --auto-callbacks      # Enables checkpoint, eval, report, ELO
```

**Equivalent to:**
```bash
essence-wars train ppo \
  --tag production_v1 \
  --checkpoint-freq 10000 \
  --eval-freq 50000 \
  --auto-evaluate \
  --auto-report \
  --update-elo
```

### Individual Flags

Enable callbacks selectively:

```bash
# Just checkpointing
essence-wars train ppo --tag exp1 --checkpoint-freq 5000

# Evaluation without report
essence-wars train ppo --tag exp1 --auto-evaluate

# Report without ELO update
essence-wars train ppo --tag exp1 --auto-report
```

---

## Custom Callbacks

### Creating Custom Callbacks

Extend `TrainingCallback` for custom behavior:

```python
from essence_wars.training.callbacks import TrainingCallback, CallbackContext

class CustomCallback(TrainingCallback):
    """Custom callback for specialized logging."""

    def on_train_start(self, context: CallbackContext):
        """Called when training starts."""
        print(f"Starting training: {context.experiment_dir}")

    def on_step(self, context: CallbackContext, step: int, metrics: dict):
        """Called after each training step."""
        if step % 1000 == 0:
            print(f"Step {step}: Loss = {metrics.get('loss', 0):.4f}")

    def on_train_complete(self, context: CallbackContext):
        """Called when training completes."""
        print(f"Training complete: {context.experiment_dir}")
        # Custom post-processing
        self.analyze_results(context)

    def analyze_results(self, context: CallbackContext):
        """Custom result analysis."""
        # Your custom logic here
        pass
```

### Using Custom Callbacks

```python
from essence_wars.agents.ppo import PPOTrainer

# Initialize trainer
trainer = PPOTrainer(config=config)

# Add custom callback
callbacks = [
    CustomCallback(),
    AutoReportCallback(),
]

# Train with callbacks
trainer.train(callbacks=callbacks)
```

---

## Configuration

### TOML Configuration

Configure callbacks via TOML file:

**`configs/callbacks.toml`:**
```toml
[checkpoints]
enabled = true
save_freq = 10000
keep_last = 5
save_best = true

[evaluation]
enabled = true
eval_freq = 50000
eval_games = 100
opponents = ["random", "greedy", "mcts-50", "mcts-100"]

[auto_evaluate]
enabled = true
eval_games = 200
update_elo = true

[auto_report]
enabled = true
tabs = ["overview", "elo", "benchmark"]
open_browser = false

[logging]
enabled = true
log_freq = 1000
log_file = "training.log"
tensorboard = true
```

**Usage:**
```bash
essence-wars train ppo \
  --tag exp1 \
  --config configs/callbacks.toml
```

### Programmatic Configuration

```python
from essence_wars.training import make_callback

callbacks = make_callback(
    save_path="experiments/exp1",
    checkpoint_freq=10000,
    keep_last=5,
    eval_freq=50000,
    eval_games=100,
    auto_evaluate=True,
    eval_games_final=200,
    auto_report=True,
    update_elo=True,
)
```

---

## Integration with Training Scripts

### PPO Training

**File:** `python/scripts/training/ppo.py`

```python
from essence_wars.agents.ppo import train_ppo
from essence_wars.training import make_callback

# Setup callbacks
callbacks = make_callback(
    save_path=f"experiments/ppo/{timestamp}_{tag}",
    auto_evaluate=args.auto_evaluate,
    auto_report=args.auto_report,
    update_elo=args.update_elo,
)

# Train with callbacks
train_ppo(
    tag=args.tag,
    timesteps=args.timesteps,
    callbacks=callbacks,
)
```

**CLI:**
```bash
python scripts/training/ppo.py \
  --tag my_experiment \
  --timesteps 300000 \
  --auto-callbacks
```

---

### AlphaZero Training

**File:** `python/scripts/training/alphazero.py`

```python
from essence_wars.agents.alphazero import train_alphazero
from essence_wars.training import make_callback

# Setup callbacks
callbacks = make_callback(
    save_path=f"experiments/alphazero/{timestamp}_{tag}",
    checkpoint_freq=1,  # Save after each iteration
    auto_evaluate=args.auto_evaluate,
    auto_report=args.auto_report,
)

# Train with callbacks
train_alphazero(
    tag=args.tag,
    iterations=args.iterations,
    callbacks=callbacks,
)
```

---

### Behavioral Cloning Training

**File:** `python/scripts/training/behavioral_cloning.py`

```python
from essence_wars.agents.behavioral_cloning import train_bc
from essence_wars.training import make_callback

# Setup callbacks
callbacks = make_callback(
    save_path=f"experiments/bc/{timestamp}_{tag}",
    checkpoint_freq=5,  # Save every 5 epochs
    auto_evaluate=args.auto_evaluate,
    auto_report=args.auto_report,
    update_elo=args.update_elo,
)

# Train with callbacks
train_bc(
    tag=args.tag,
    dataset=args.dataset,
    epochs=args.epochs,
    callbacks=callbacks,
)
```

---

## Callback Context

### CallbackContext Class

```python
from dataclasses import dataclass
from pathlib import Path

@dataclass
class CallbackContext:
    """Context information passed to callbacks."""

    trainer: Any              # Training object
    experiment_dir: Path      # Experiment output directory
    config: dict             # Training configuration
    device: str              # Device (cuda/cpu)

    # Optional fields
    model: Optional[Any] = None
    optimizer: Optional[Any] = None
    tensorboard_writer: Optional[Any] = None
```

**Usage in Callbacks:**
```python
class MyCallback(TrainingCallback):
    def on_step(self, context: CallbackContext, step: int, metrics: dict):
        # Access model
        model = context.model

        # Access experiment directory
        save_path = context.experiment_dir / "my_output.json"

        # Access config
        lr = context.config.get("learning_rate", 3e-4)
```

---

## Best Practices

### 1. Use Auto-Callbacks for Long Runs

For experiments that take hours/days, enable full automation:

```bash
essence-wars train ppo \
  --tag overnight_run \
  --timesteps 2000000 \
  --auto-callbacks \
  --update-elo
```

### 2. Disable Browser Opening for Remote Training

When training on remote servers, disable auto-open:

```bash
essence-wars train ppo \
  --tag remote_exp \
  --auto-report  # Report generates, but doesn't open browser
```

### 3. Adjust Frequencies for Debugging

Use frequent callbacks during debugging:

```bash
essence-wars train ppo \
  --tag debug_run \
  --timesteps 10000 \
  --checkpoint-freq 1000 \
  --eval-freq 2000
```

### 4. Keep Checkpoints Manageable

Limit checkpoint retention to save disk space:

```python
callback = CheckpointCallback(
    save_freq=10000,
    keep_last=3,  # Only keep 3 most recent
)
```

### 5. Use Descriptive Tags

Tag experiments clearly for report organization:

```bash
# Good
essence-wars train ppo --tag 2026-02-01_lr1e4_batch512

# Bad
essence-wars train ppo --tag test
```

---

## Troubleshooting

### Callbacks Not Executing

**Problem:** Callbacks don't seem to run during training

**Solution:** Ensure callbacks are passed to trainer:
```python
trainer.train(callbacks=callbacks)  # Don't forget this!
```

### ELO Update Fails

**Problem:** `--update-elo` doesn't update ratings

**Solution:** Ensure auto-evaluate is also enabled:
```bash
essence-wars train ppo \
  --tag exp1 \
  --auto-evaluate \  # Required for ELO update
  --update-elo
```

### Report Generation Fails

**Problem:** Auto-report callback errors

**Solution:** Check that evaluation results exist:
```bash
ls experiments/ppo/{timestamp}_{tag}/final_evaluation.json
```

### Disk Space Issues

**Problem:** Checkpoints fill disk

**Solution:** Use `keep_last` to limit retention:
```bash
essence-wars train ppo \
  --tag exp1 \
  --checkpoint-freq 10000 \
  --keep-last 3
```

---

## API Reference

### TrainingCallback

```python
from essence_wars.training.callbacks import TrainingCallback

class TrainingCallback:
    """Abstract base class for training callbacks."""

    def on_train_start(self, context: CallbackContext):
        """Called when training starts."""
        pass

    def on_step(self, context: CallbackContext, step: int, metrics: dict):
        """Called after each training step."""
        pass

    def on_train_complete(self, context: CallbackContext):
        """Called when training completes."""
        pass
```

### make_callback

```python
from essence_wars.training import make_callback

def make_callback(
    save_path: Path,
    checkpoint_freq: int = 10000,
    keep_last: int = 5,
    eval_freq: int = None,
    eval_games: int = 50,
    auto_evaluate: bool = False,
    eval_games_final: int = 200,
    auto_report: bool = False,
    update_elo: bool = False,
    tensorboard: bool = True,
) -> CallbackList:
    """Create configured callback list.

    Args:
        save_path: Base directory for outputs
        checkpoint_freq: Checkpoint save frequency
        keep_last: Number of checkpoints to keep
        eval_freq: Evaluation frequency during training
        eval_games: Games per evaluation
        auto_evaluate: Enable post-training evaluation
        eval_games_final: Games for final evaluation
        auto_report: Enable post-training report
        update_elo: Update ELO ratings
        tensorboard: Enable TensorBoard logging

    Returns:
        Configured callback list
    """
    ...
```

---

## See Also

- [CLI Guide](cli-guide.md) - Command-line interface documentation
- [Ratings System](ratings-system.md) - ELO tracking details
- [Reporting](reporting.md) - HTML report generation
- [Bots & Tuning Pipeline](bots-tuning-pipeline.md) - Rust-side configuration

---

*For issues or feature requests, see the project's GitHub repository.*
