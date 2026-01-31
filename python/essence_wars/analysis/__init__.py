"""
Analysis and visualization tools for Essence Wars experiments.

Modules:
    - aggregator: Collect and aggregate MCTS training runs
    - validation_cli: CLI analyzer for validation results
    - visualize: Matplotlib visualizations for training
    - parse_log: Log file parsing utilities
    - performance_dashboard: Criterion benchmark dashboard
    - report/: Unified HTML report generator (preferred)
"""

from .aggregator import ExperimentAggregator, ExperimentMetadata, ExperimentRun

__all__ = [
    "ExperimentAggregator",
    "ExperimentMetadata",
    "ExperimentRun",
]
