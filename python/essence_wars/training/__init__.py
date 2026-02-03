"""
Training utilities for Essence Wars agents.

This module provides:
- Callback system for training lifecycle hooks
- Auto-evaluation after training
- Auto-report generation
- Checkpoint management

Example:
    >>> from essence_wars.training import (
    ...     CallbackList, CallbackContext,
    ...     CheckpointCallback, AutoEvaluateCallback,
    ...     make_callback,
    ... )
    >>>
    >>> # Quick setup with convenience function
    >>> callbacks = make_callback(
    ...     save_path="experiments/run1",
    ...     auto_evaluate=True,
    ...     auto_report=True,
    ... )
    >>>
    >>> # Or manual setup for more control
    >>> callbacks = CallbackList([
    ...     CheckpointCallback(save_path, save_freq=5000),
    ...     AutoEvaluateCallback(eval_games=200),
    ... ])
    >>>
    >>> # Use with trainer
    >>> context = CallbackContext(trainer=trainer, experiment_dir=save_path)
    >>> callbacks.on_train_start(context)
    >>> results = trainer.train(callback=callbacks.as_functional())
    >>> callbacks.on_train_complete(results)
"""

from .callbacks import (
    AutoEvaluateCallback,
    AutoReportCallback,
    # Core types
    CallbackContext,
    CallbackList,
    # Built-in callbacks
    CheckpointCallback,
    EvaluationCallback,
    LoggingCallback,
    TrainingCallback,
    # Convenience function
    make_callback,
)

__all__ = [
    # Core types
    "CallbackContext",
    "TrainingCallback",
    "CallbackList",
    # Built-in callbacks
    "CheckpointCallback",
    "EvaluationCallback",
    "AutoEvaluateCallback",
    "AutoReportCallback",
    "LoggingCallback",
    # Convenience function
    "make_callback",
]
