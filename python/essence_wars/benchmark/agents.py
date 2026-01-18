"""Agent interfaces and wrappers for benchmarking.

This module defines the BenchmarkAgent protocol that all agents must
implement, plus wrapper classes for built-in agents (Random, Greedy, MCTS)
and neural network agents.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Protocol, runtime_checkable

import numpy as np
import torch


@runtime_checkable
class BenchmarkAgent(Protocol):
    """Protocol for agents that can be benchmarked.

    All agents must implement:
    - select_action(): Choose action given observation and mask
    - reset(): Reset internal state for new game
    - name: Human-readable name for display

    Example:
        class MyAgent:
            @property
            def name(self) -> str:
                return "MyAgent"

            def select_action(self, obs: np.ndarray, mask: np.ndarray) -> int:
                # Your logic here
                valid_actions = np.where(mask > 0.5)[0]
                return int(np.random.choice(valid_actions))

            def reset(self) -> None:
                pass
    """

    @property
    def name(self) -> str:
        """Human-readable agent name."""
        ...

    def select_action(
        self,
        observation: np.ndarray,
        action_mask: np.ndarray,
    ) -> int:
        """Select an action given observation and legal action mask.

        Args:
            observation: State tensor of shape (326,)
            action_mask: Binary mask of shape (256,), 1.0 = legal

        Returns:
            Action index in [0, 255]
        """
        ...

    def reset(self) -> None:
        """Reset agent state for a new game."""
        ...


class BaseAgent(ABC):
    """Abstract base class for benchmark agents."""

    def __init__(self, agent_name: str):
        self._name = agent_name

    @property
    def name(self) -> str:
        return self._name

    @abstractmethod
    def select_action(
        self,
        observation: np.ndarray,
        action_mask: np.ndarray,
    ) -> int:
        pass

    def reset(self) -> None:
        """Default reset does nothing."""
        pass


class RandomAgent(BaseAgent):
    """Agent that selects random legal actions.

    This is the weakest baseline - any trained agent should beat it
    with >95% win rate.
    """

    def __init__(self, seed: int | None = None):
        super().__init__("Random")
        self.rng = np.random.default_rng(seed)

    def select_action(
        self,
        observation: np.ndarray,
        action_mask: np.ndarray,
    ) -> int:
        valid_actions = np.where(action_mask > 0.5)[0]
        if len(valid_actions) == 0:
            return 255  # EndTurn fallback
        return int(self.rng.choice(valid_actions))

    def reset(self) -> None:
        pass  # RNG continues across games


class GreedyAgent(BaseAgent):
    """Agent that uses the Rust GreedyBot via environment.

    This agent evaluates each action by simulation and picks the
    best according to a heuristic evaluation function.

    Win rates to expect:
    - vs Random: ~100%
    - vs Greedy: ~50% (mirror match)
    """

    def __init__(self) -> None:
        super().__init__("Greedy")
        # We'll use the Rust greedy bot through the game interface
        self._use_rust_bot = True

    def select_action(
        self,
        observation: np.ndarray,
        action_mask: np.ndarray,
    ) -> int:
        # This is a placeholder - actual implementation uses Rust bot
        # through the benchmark runner which has access to the game state
        raise NotImplementedError(
            "GreedyAgent requires game state access. "
            "Use EssenceWarsBenchmark.evaluate() which handles this internally."
        )

    def reset(self) -> None:
        pass


class MCTSAgent(BaseAgent):
    """Agent that uses Monte Carlo Tree Search.

    MCTS with configurable simulation count provides a strong baseline.

    Win rates to expect (with 100 sims):
    - vs Random: ~100%
    - vs Greedy: ~60-70%
    - vs MCTS-50: ~55%
    """

    def __init__(self, simulations: int = 100):
        super().__init__(f"MCTS-{simulations}")
        self.simulations = simulations
        self._use_rust_bot = True

    def select_action(
        self,
        observation: np.ndarray,
        action_mask: np.ndarray,
    ) -> int:
        # Placeholder - uses Rust MCTS through benchmark runner
        raise NotImplementedError(
            "MCTSAgent requires game state access. "
            "Use EssenceWarsBenchmark.evaluate() which handles this internally."
        )

    def reset(self) -> None:
        pass


class NeuralAgent(BaseAgent):
    """Agent that uses a neural network for action selection.

    Wraps any PyTorch model that takes observation and mask tensors
    and outputs action logits or probabilities.
    """

    def __init__(
        self,
        network: torch.nn.Module,
        name: str = "Neural",
        device: str | torch.device = "cpu",
        deterministic: bool = True,
    ):
        """Initialize neural agent.

        Args:
            network: PyTorch model with forward(obs, mask) -> (logits, value)
            name: Display name for the agent
            device: Device to run inference on
            deterministic: If True, pick argmax; if False, sample from policy
        """
        super().__init__(name)
        self.network = network
        self.device = torch.device(device)
        self.deterministic = deterministic
        self.network.to(self.device)
        self.network.eval()

    def select_action(
        self,
        observation: np.ndarray,
        action_mask: np.ndarray,
    ) -> int:
        with torch.no_grad():
            obs = torch.from_numpy(observation).float().unsqueeze(0).to(self.device)
            mask = torch.from_numpy(action_mask).bool().unsqueeze(0).to(self.device)

            # Forward pass
            logits, _ = self.network(obs, mask)

            if self.deterministic:
                # Argmax over legal actions
                action = logits.argmax(dim=-1).item()
            else:
                # Sample from softmax distribution
                probs = torch.softmax(logits, dim=-1)
                action = torch.multinomial(probs, 1).item()

        return int(action)

    def reset(self) -> None:
        pass

    @classmethod
    def from_checkpoint(
        cls,
        checkpoint_path: str,
        name: str | None = None,
        device: str = "cpu",
        deterministic: bool = True,
    ) -> NeuralAgent:
        """Load a neural agent from a checkpoint file.

        Supports both AlphaZero and Behavioral Cloning checkpoints.

        Args:
            checkpoint_path: Path to .pt checkpoint file
            name: Display name (default: derived from filename)
            device: Device for inference
            deterministic: Whether to use argmax or sampling

        Returns:
            NeuralAgent instance
        """
        from pathlib import Path

        from essence_wars.agents.networks import AlphaZeroNetwork

        checkpoint = torch.load(checkpoint_path, map_location=device, weights_only=False)

        # Determine network architecture from checkpoint
        if "model_state_dict" in checkpoint:
            # BC checkpoint
            state_dict = checkpoint["model_state_dict"]
            args = checkpoint.get("args", {})
        elif "network_state_dict" in checkpoint:
            # AlphaZero checkpoint
            state_dict = checkpoint["network_state_dict"]
            args = checkpoint.get("config", {})
        else:
            raise ValueError(f"Unknown checkpoint format: {checkpoint_path}")

        # Create network with matching architecture
        hidden_dim = args.get("hidden_dim", 256)
        num_blocks = args.get("num_blocks", 4)

        network = AlphaZeroNetwork(
            obs_dim=326,
            action_dim=256,
            hidden_dim=hidden_dim,
            num_blocks=num_blocks,
        )
        network.load_state_dict(state_dict)

        # Generate name from filename if not provided
        if name is None:
            name = Path(checkpoint_path).stem

        return cls(network, name=name, device=device, deterministic=deterministic)
