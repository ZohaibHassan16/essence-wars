"""Benchmark API for Essence Wars agent evaluation.

This module provides standardized evaluation tools for comparing
agent performance across multiple metrics and baselines.

Example:
    from essence_wars.benchmark import EssenceWarsBenchmark

    benchmark = EssenceWarsBenchmark()
    results = benchmark.evaluate(my_agent)
    print(f"Win rate vs Greedy: {results['win_rate_vs_greedy']:.1%}")
"""

from .agents import BenchmarkAgent, RandomAgent, GreedyAgent, MCTSAgent, NeuralAgent
from .elo import EloTracker
from .api import EssenceWarsBenchmark
from .metrics import BenchmarkResults

__all__ = [
    "BenchmarkAgent",
    "RandomAgent",
    "GreedyAgent",
    "MCTSAgent",
    "NeuralAgent",
    "EloTracker",
    "EssenceWarsBenchmark",
    "BenchmarkResults",
]
