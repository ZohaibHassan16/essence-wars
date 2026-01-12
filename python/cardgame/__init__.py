"""AI Card Game - Python bindings for the Essence Wars engine."""

from __future__ import annotations

__version__ = "0.1.0"

# Infrastructure
from cardgame.infra import Experiment

__all__ = ["Experiment", "__version__"]

# Native bindings will be imported when built with maturin
# from cardgame._cardgame import GameEngine, CardDatabase, Action, etc.
