"""
Analysis and visualization tools for Essence Wars experiments.

Modules:
    - aggregator: Collect and aggregate MCTS training runs
    - dashboard: MCTS training dashboard (HTML with Plotly)
    - research_dashboard: Balance validation dashboard (HTML with Plotly)
    - validation_cli: CLI analyzer for validation results
    - visualize: Matplotlib visualizations for training
    - parse_log: Log file parsing utilities
"""

from .aggregator import ExperimentAggregator, ExperimentRun, ExperimentMetadata
from .dashboard import MCTSDashboard

__all__ = [
    "ExperimentAggregator",
    "ExperimentRun",
    "ExperimentMetadata",
    "MCTSDashboard",
]
