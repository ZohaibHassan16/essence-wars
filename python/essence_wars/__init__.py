"""
Essence Wars - High-performance card game environment for RL research.

This package provides:
- PyGame: Single game wrapper for Python
- PyParallelGames: Batched environments for vectorized training
- Gymnasium-compatible environments (optional)
- PettingZoo-compatible multi-agent environments (optional)

Submodules:
- essence_wars.infra: Experiment management utilities
- essence_wars.analysis: Visualization and analysis tools (requires pandas, plotly)
- essence_wars.agents: PPO and AlphaZero agent implementations
- essence_wars.hub: Huggingface Hub integration for model/dataset sharing
- essence_wars.data: Dataset loading utilities
- essence_wars.benchmark: Standardized evaluation API

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

    env = EssenceWarsEnv(deck="architect_fortify")
    obs, info = env.reset(seed=42)
    obs, reward, terminated, truncated, info = env.step(action)

With Analysis Tools:
    from essence_wars.infra import Experiment
    from essence_wars.analysis import ExperimentAggregator, MCTSDashboard
"""

__version__ = "0.6.0"

# Import core Rust bindings
try:
    from essence_wars._core import (
        ACTION_SPACE_SIZE,
        STATE_TENSOR_SIZE,
        PyGame,
        PyParallelGames,
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
    result: list[str] = PyGame.list_decks()
    return result

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

# Lazy imports for optional modules to avoid import errors when dependencies missing
def __getattr__(name: str) -> object:
    """Lazy import for optional modules."""
    if name == "EssenceWarsEnv":
        from essence_wars.env import EssenceWarsEnv
        return EssenceWarsEnv
    elif name == "EssenceWarsSelfPlayEnv":
        from essence_wars.env import EssenceWarsSelfPlayEnv
        return EssenceWarsSelfPlayEnv
    elif name == "make_env":
        from essence_wars.env import make_env
        return make_env
    elif name == "VectorizedEssenceWars":
        from essence_wars.env import VectorizedEssenceWars
        return VectorizedEssenceWars
    elif name == "EssenceWarsParallelEnv":
        from essence_wars.parallel_env import EssenceWarsParallelEnv
        return EssenceWarsParallelEnv
    elif name == "parallel_env":
        from essence_wars.parallel_env import parallel_env
        return parallel_env
    # Hub functions (lazy import to avoid huggingface_hub dependency)
    elif name == "load_pretrained":
        from essence_wars.hub import load_pretrained
        return load_pretrained
    elif name == "download_dataset":
        from essence_wars.hub import download_dataset
        return download_dataset
    elif name == "upload_model":
        from essence_wars.hub import upload_model
        return upload_model
    elif name == "upload_dataset":
        from essence_wars.hub import upload_dataset
        return upload_dataset
    raise AttributeError(f"module 'essence_wars' has no attribute {name!r}")
