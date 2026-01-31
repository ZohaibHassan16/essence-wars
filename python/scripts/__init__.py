"""
Essence Wars Scripts Package.

Organized into categories:
- training/    - ML model training scripts
- evaluation/  - Model evaluation and benchmarking
- data/        - Dataset generation scripts
- analysis/    - Training analysis and diagnostics
- benchmark/   - Performance profiling scripts
- reporting/   - HTML report generation

Each script can be run directly or imported:
    python -m scripts.training.ppo --help
    python scripts/training/ppo.py --help

For CLI commands, use the unified interface:
    essence-wars --help
"""

__all__ = [
    "training",
    "evaluation",
    "data",
    "analysis",
    "benchmark",
    "reporting",
]
