"""
Essence Wars - High-performance card game environment for RL research.

This package provides:
- PyGame: Single game wrapper for Python
- PyParallelGames: Batched environments for vectorized training
- Gymnasium-compatible environments (optional)
- PettingZoo-compatible multi-agent environments (optional)

Quick Start:
    from essence_wars import PyGame

    game = PyGame()
    game.reset(seed=42)

    obs = game.observe()        # numpy array (326,)
    mask = game.action_mask()   # numpy array (256,)

    action = 255  # EndTurn action
    reward, done = game.step(action)

With Gymnasium:
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv(deck="argentum_control")
    obs, info = env.reset(seed=42)
    obs, reward, terminated, truncated, info = env.step(action)
"""

__version__ = "0.6.0"

# Import core Rust bindings
try:
    from essence_wars._core import (
        PyGame,
        PyParallelGames,
        STATE_TENSOR_SIZE,
        ACTION_SPACE_SIZE,
    )
except ImportError as e:
    raise ImportError(
        "Failed to import Rust bindings. "
        "Make sure you installed the package with: pip install essence-wars\n"
        f"Original error: {e}"
    ) from e

# Convenience function to list available decks
def list_decks() -> list[str]:
    """List all available deck names."""
    return PyGame.list_decks()

__all__ = [
    # Version
    "__version__",
    # Core classes
    "PyGame",
    "PyParallelGames",
    # Constants
    "STATE_TENSOR_SIZE",
    "ACTION_SPACE_SIZE",
    # Functions
    "list_decks",
]
