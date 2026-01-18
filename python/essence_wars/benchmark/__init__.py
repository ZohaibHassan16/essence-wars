"""Benchmark API for Essence Wars agent evaluation.

This module provides standardized evaluation tools for comparing
agent performance across multiple metrics and baselines.

Example:
    from essence_wars.benchmark import EssenceWarsBenchmark

    benchmark = EssenceWarsBenchmark()
    results = benchmark.evaluate(my_agent)
    print(f"Win rate vs Greedy: {results['win_rate_vs_greedy']:.1%}")
"""

from .agents import BenchmarkAgent, GreedyAgent, MCTSAgent, NeuralAgent, RandomAgent
from .api import EssenceWarsBenchmark
from .elo import EloTracker
from .metrics import BenchmarkResults

__all__ = [
    "BenchmarkAgent",
    "BenchmarkResults",
    "EloTracker",
    "EssenceWarsBenchmark",
    "GreedyAgent",
    "MCTSAgent",
    "NeuralAgent",
    "RandomAgent",
]
