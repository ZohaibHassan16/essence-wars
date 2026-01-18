"""
PettingZoo Multi-Agent Environment for Essence Wars.

This module provides PettingZoo-compatible environments for multi-agent
reinforcement learning research. Both ParallelEnv and AECEnv interfaces
are supported.

Features:
- Two-player turn-based gameplay
- Action masking for illegal action handling
- Observations from each player's perspective
- Compatible with MAPPO, QMIX, and other MARL algorithms

Example:
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env = EssenceWarsParallelEnv()
    observations, infos = env.reset(seed=42)

    while env.agents:
        actions = {agent: policy(observations[agent]) for agent in env.agents}
        observations, rewards, terminations, truncations, infos = env.step(actions)
"""

from __future__ import annotations

from typing import Any

import numpy as np
from gymnasium import spaces

try:
    from pettingzoo import ParallelEnv as ParallelEnvBase
    from pettingzoo.utils import wrappers
    PETTINGZOO_AVAILABLE = True
except ImportError:
    PETTINGZOO_AVAILABLE = False
    # Create a dummy base class for when PettingZoo isn't installed
    ParallelEnvBase = object  # type: ignore[assignment]

from essence_wars._core import ACTION_SPACE_SIZE, STATE_TENSOR_SIZE, PyGame


class EssenceWarsParallelEnv(ParallelEnvBase):  # type: ignore[misc]
    """
    PettingZoo ParallelEnv for two-player Essence Wars.

    In this environment, both agents receive observations at every step,
    but only the active player's action is executed. The inactive player's
    action is ignored (their action_mask is all zeros).

    This design allows training algorithms that expect simultaneous actions
    from all agents (like independent PPO) while maintaining the turn-based
    nature of the game.

    Observation Space (per agent):
        Box(low=-inf, high=inf, shape=(326,), dtype=float32)

    Action Space (per agent):
        Discrete(256)

    Action Masking:
        Available via `infos[agent]["action_mask"]` - a boolean array where
        True indicates a legal action. Only the active player has legal actions.

    Rewards:
        +1.0 for winning, -1.0 for losing, 0.0 otherwise
        Rewards are from each player's own perspective.

    Args:
        deck1: Deck name for player_0
        deck2: Deck name for player_1
        game_mode: "attrition" or "essence_duel"
        max_turns: Maximum turns before truncation
        render_mode: Optional render mode (currently unused)
    """

    metadata = {
        "render_modes": ["human", "ansi"],
        "name": "essence_wars_v1",
        "is_parallelizable": True,
    }

    def __init__(
        self,
        deck1: str = "artificer_tokens",
        deck2: str = "broodmother_swarm",
        game_mode: str = "attrition",
        max_turns: int = 100,
        render_mode: str | None = None,
    ) -> None:
        if not PETTINGZOO_AVAILABLE:
            raise ImportError(
                "PettingZoo is required for multi-agent environments. "
                "Install with: pip install pettingzoo"
            )

        super().__init__()

        self.deck1 = deck1
        self.deck2 = deck2
        self.game_mode = game_mode
        self.max_turns = max_turns
        self.render_mode = render_mode

        # Agent configuration
        self.possible_agents = ["player_0", "player_1"]
        self.agents = self.possible_agents[:]

        # Create game instance
        self._game = PyGame(
            deck1=deck1,
            deck2=deck2,
            game_mode=game_mode,
        )

        # Define spaces for each agent
        self._observation_spaces = {
            agent: spaces.Box(
                low=-np.inf,
                high=np.inf,
                shape=(STATE_TENSOR_SIZE,),
                dtype=np.float32,
            )
            for agent in self.possible_agents
        }

        self._action_spaces: dict[str, spaces.Space[int]] = {
            agent: spaces.Discrete(ACTION_SPACE_SIZE)
            for agent in self.possible_agents
        }

        # Episode tracking
        self._step_count = 0
        self._cumulative_rewards = dict.fromkeys(self.possible_agents, 0.0)

    def observation_space(self, agent: str) -> spaces.Box:
        """Return the observation space for an agent."""
        return self._observation_spaces[agent]

    def action_space(self, agent: str) -> spaces.Space[int]:
        """Return the action space for an agent."""
        return self._action_spaces[agent]

    def reset(
        self,
        seed: int | None = None,
        options: dict[str, Any] | None = None,
    ) -> tuple[dict[str, np.ndarray], dict[str, dict[str, Any]]]:
        """
        Reset the environment to start a new episode.

        Args:
            seed: Random seed for reproducibility
            options: Additional options (unused)

        Returns:
            observations: Dict mapping agent names to observations
            infos: Dict mapping agent names to info dicts with action_mask
        """
        if seed is None:
            seed = np.random.randint(0, 2**31 - 1)

        self._game.reset(seed=seed)
        self.agents = self.possible_agents[:]
        self._step_count = 0
        self._cumulative_rewards = dict.fromkeys(self.possible_agents, 0.0)

        observations = self._get_observations()
        infos = self._get_infos()

        return observations, infos

    def step(
        self,
        actions: dict[str, int],
    ) -> tuple[
        dict[str, np.ndarray],
        dict[str, float],
        dict[str, bool],
        dict[str, bool],
        dict[str, dict[str, Any]],
    ]:
        """
        Execute one step in the environment.

        Only the active player's action is executed. The inactive player's
        action is ignored.

        Args:
            actions: Dict mapping agent names to action indices

        Returns:
            observations: Dict mapping agent names to observations
            rewards: Dict mapping agent names to rewards
            terminations: Dict mapping agent names to termination flags
            truncations: Dict mapping agent names to truncation flags
            infos: Dict mapping agent names to info dicts
        """
        self._step_count += 1

        # Determine active player
        active_player_idx = self._game.current_player()
        active_agent = f"player_{active_player_idx}"

        # Get the active player's action
        if active_agent in actions:
            action = actions[active_agent]
        else:
            # Fallback: if active agent didn't provide action, use first legal
            mask = self._game.action_mask()
            legal_actions = np.where(mask > 0)[0]
            action = int(legal_actions[0]) if len(legal_actions) > 0 else 255

        # Validate action
        mask = self._game.action_mask()
        if mask[action] == 0.0:
            # Invalid action - this shouldn't happen with proper masking
            # Penalize and end game
            rewards = {
                active_agent: -1.0,
                self._other_agent(active_agent): 1.0,
            }
            terminations = dict.fromkeys(self.possible_agents, True)
            truncations = dict.fromkeys(self.possible_agents, False)
            self.agents = []

            observations = self._get_observations()
            infos = self._get_infos()
            for agent in self.possible_agents:
                infos[agent]["invalid_action"] = (agent == active_agent)

            return observations, rewards, terminations, truncations, infos

        # Execute the action
        reward_p0, done = self._game.step(action)

        # Rewards from each player's perspective
        rewards = {
            "player_0": reward_p0,
            "player_1": -reward_p0,
        }

        # Update cumulative rewards
        for agent, reward in rewards.items():
            self._cumulative_rewards[agent] += reward

        # Check for truncation (max turns)
        truncated = self._step_count >= self.max_turns and not done

        terminations = dict.fromkeys(self.possible_agents, done)
        truncations = dict.fromkeys(self.possible_agents, truncated)

        if done or truncated:
            self.agents = []

        observations = self._get_observations()
        infos = self._get_infos()

        return observations, rewards, terminations, truncations, infos

    def _get_observations(self) -> dict[str, np.ndarray]:
        """Get observations for all agents."""
        # Both players see the same game state tensor
        # (The tensor is from the game's perspective, not player-relative)
        obs = self._game.observe()
        return {agent: obs.copy() for agent in self.possible_agents}

    def _get_infos(self) -> dict[str, dict[str, Any]]:
        """Get info dicts for all agents with action masks."""
        current_player = self._game.current_player()
        game_mask = self._game.action_mask()

        infos = {}
        for i, agent in enumerate(self.possible_agents):
            if i == current_player and not self._game.is_done():
                # Active player gets the real action mask
                action_mask = game_mask > 0.0
            else:
                # Inactive player has no legal actions
                action_mask = np.zeros(ACTION_SPACE_SIZE, dtype=bool)

            infos[agent] = {
                "action_mask": action_mask,
                "is_active": i == current_player,
                "turn_number": self._game.turn_number(),
            }

        return infos

    def _other_agent(self, agent: str) -> str:
        """Get the other agent's name."""
        return "player_1" if agent == "player_0" else "player_0"

    def render(self) -> None:
        """Render the environment (not implemented)."""
        pass

    def close(self) -> None:
        """Clean up resources."""
        pass

    @property
    def unwrapped(self) -> EssenceWarsParallelEnv:
        """Return the unwrapped environment."""
        return self

    def state(self) -> np.ndarray:
        """
        Return the global state of the environment.

        For Essence Wars, the state is the same as the observation since
        it's a perfect information game.
        """
        state: np.ndarray = self._game.observe()
        return state


def parallel_env(**kwargs: Any) -> EssenceWarsParallelEnv:
    """
    Create a PettingZoo parallel environment.

    This is the standard factory function expected by PettingZoo.
    """
    return EssenceWarsParallelEnv(**kwargs)


def raw_env(**kwargs: Any) -> EssenceWarsParallelEnv:
    """
    Create a raw (unwrapped) environment.

    This is the standard factory function expected by PettingZoo.
    """
    return EssenceWarsParallelEnv(**kwargs)


def env(**kwargs: Any) -> EssenceWarsParallelEnv:
    """
    Create an environment with standard wrappers.

    This is the recommended way to create environments for training.
    """
    environment = EssenceWarsParallelEnv(**kwargs)
    # Note: Could add wrappers here if needed
    # environment = wrappers.OrderEnforcingWrapper(environment)
    return environment
