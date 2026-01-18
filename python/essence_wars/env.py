"""
Gymnasium-compatible environment for Essence Wars.

This module provides a standard Gymnasium interface for training RL agents
on the Essence Wars card game. It wraps the high-performance Rust engine
via PyO3 bindings.

Features:
- Gymnasium v26+ API compliance (5-tuple step returns)
- Action masking via info["action_mask"] for invalid action handling
- Configurable opponent (greedy, random, or external policy)
- Support for both game modes (Attrition, Essence Duel)

Example:
    import gymnasium as gym
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv()
    obs, info = env.reset(seed=42)

    while True:
        # Use action mask for valid action selection
        mask = info["action_mask"]
        action = select_action(obs, mask)  # Your policy

        obs, reward, terminated, truncated, info = env.step(action)
        if terminated or truncated:
            break
"""

from __future__ import annotations

from typing import Any, SupportsFloat

import gymnasium as gym
import numpy as np
from gymnasium import spaces

from essence_wars._core import ACTION_SPACE_SIZE, STATE_TENSOR_SIZE, PyGame, PyParallelGames


class EssenceWarsEnv(gym.Env[np.ndarray, int]):
    """
    Gymnasium environment for the Essence Wars card game.

    This environment wraps the Rust game engine and provides a standard
    RL interface. The agent plays as Player 1 against a configurable opponent.

    Observation Space:
        Box(low=-inf, high=inf, shape=(326,), dtype=float32)
        A 326-dimensional state tensor encoding the full game state.

    Action Space:
        Discrete(256)
        - 0-49: PlayCard (hand_idx * 5 + slot)
        - 50-74: Attack (attacker * 5 + defender)
        - 75-254: UseAbility (reserved for future)
        - 255: EndTurn

    Rewards:
        - +1.0 for winning the game
        - -1.0 for losing the game
        - 0.0 for ongoing game or draw

    Args:
        deck1: Deck name for the agent (Player 1). Default: "artificer_tokens"
        deck2: Deck name for the opponent (Player 2). Default: "broodmother_swarm"
        opponent: Opponent type - "greedy", "random", or None for external control.
                  Default: "greedy"
        game_mode: "attrition" (default) or "essence_duel"
        max_turns: Maximum turns before truncation. Default: 100 (game limit is 30)
        render_mode: Optional render mode. Currently only None is supported.
    """

    metadata = {
        "render_modes": [],
        "render_fps": 1,
    }

    def __init__(
        self,
        deck1: str = "artificer_tokens",
        deck2: str = "broodmother_swarm",
        opponent: str | None = "greedy",
        game_mode: str = "attrition",
        max_turns: int = 100,
        render_mode: str | None = None,
    ) -> None:
        super().__init__()

        self.deck1 = deck1
        self.deck2 = deck2
        self.opponent_type = opponent
        self.game_mode = game_mode
        self.max_turns = max_turns
        self.render_mode = render_mode

        # Create the game instance
        self._game = PyGame(
            deck1=deck1,
            deck2=deck2,
            game_mode=game_mode,
        )

        # Define spaces
        self.observation_space = spaces.Box(
            low=-np.inf,
            high=np.inf,
            shape=(STATE_TENSOR_SIZE,),
            dtype=np.float32,
        )

        self.action_space = spaces.Discrete(ACTION_SPACE_SIZE)

        # Track episode state
        self._step_count = 0
        self._episode_reward = 0.0

    def reset(
        self,
        *,
        seed: int | None = None,
        options: dict[str, Any] | None = None,
    ) -> tuple[np.ndarray, dict[str, Any]]:
        """
        Reset the environment to start a new episode.

        Args:
            seed: Random seed for reproducibility
            options: Additional options (unused)

        Returns:
            observation: Initial state tensor (326,)
            info: Dictionary with action_mask and metadata
        """
        super().reset(seed=seed)

        # Use provided seed or generate one
        if seed is None:
            seed = int(self.np_random.integers(0, 2**32 - 1))

        self._game.reset(seed=seed)
        self._step_count = 0
        self._episode_reward = 0.0

        # If opponent moves first (we're player 1, opponent is player 2)
        # and it's player 2's turn, let opponent play
        self._play_opponent_turn()

        obs = self._get_obs()
        info = self._get_info()

        return obs, info

    def step(
        self,
        action: int,
    ) -> tuple[np.ndarray, SupportsFloat, bool, bool, dict[str, Any]]:
        """
        Execute one step in the environment.

        Args:
            action: Action index (0-255)

        Returns:
            observation: New state tensor (326,)
            reward: Reward for this step (+1 win, -1 loss, 0 otherwise)
            terminated: True if game ended (win/loss/draw)
            truncated: True if max_turns exceeded
            info: Dictionary with action_mask and metadata
        """
        self._step_count += 1

        # Validate action is legal
        mask = self._game.action_mask()
        if mask[action] == 0.0:
            # Invalid action - return large negative reward and end episode
            # This shouldn't happen if agent uses the mask properly
            obs = self._get_obs()
            info = self._get_info()
            info["invalid_action"] = True
            return obs, -10.0, True, False, info

        # Execute the action
        reward, done = self._game.step(action)

        # If game not over and it's opponent's turn, let opponent play
        if not done:
            self._play_opponent_turn()
            # Check if game ended after opponent's turn
            done = self._game.is_done()
            if done:
                reward = self._game.get_reward(0)  # Get reward for player 1

        self._episode_reward += reward

        # Check for truncation (max turns)
        truncated = self._step_count >= self.max_turns and not done

        obs = self._get_obs()
        info = self._get_info()
        info["episode_reward"] = self._episode_reward

        return obs, reward, done, truncated, info

    def _get_obs(self) -> np.ndarray:
        """Get current observation (state tensor)."""
        obs: np.ndarray = self._game.observe()
        return obs

    def _get_info(self) -> dict[str, Any]:
        """Get info dictionary with action mask and metadata."""
        return {
            "action_mask": self._game.action_mask(),
            "current_player": self._game.current_player(),
            "turn_number": self._game.turn_number(),
            "is_done": self._game.is_done(),
            "step_count": self._step_count,
        }

    def _play_opponent_turn(self) -> None:
        """Let the opponent play their turn(s) until it's our turn or game ends."""
        if self.opponent_type is None:
            # External control - don't auto-play opponent
            return

        # Play while it's opponent's turn (player 2 = index 1)
        while self._game.current_player() == 1 and not self._game.is_done():
            if self.opponent_type == "greedy":
                action = self._game.greedy_action()
            elif self.opponent_type == "random":
                action = self._game.random_action()
            else:
                raise ValueError(f"Unknown opponent type: {self.opponent_type}")

            self._game.step(action)

    def render(self) -> None:
        """Render the environment (not implemented)."""
        pass

    def close(self) -> None:
        """Clean up resources."""
        pass

    @property
    def unwrapped(self) -> EssenceWarsEnv:
        """Return the unwrapped environment."""
        return self

    def action_masks(self) -> np.ndarray:
        """
        Get the current action mask.

        This method is provided for compatibility with SB3's MaskablePPO.
        Returns a boolean mask where True = legal action.

        Returns:
            Boolean array of shape (256,)
        """
        mask: np.ndarray = self._game.action_mask()
        result: np.ndarray = mask > 0.0
        return result


class EssenceWarsSelfPlayEnv(EssenceWarsEnv):
    """
    Self-play variant of the Essence Wars environment.

    In this variant, both players are controlled externally (no built-in opponent).
    The agent alternates between playing as Player 1 and Player 2.
    Rewards are from the perspective of the current player.

    This is useful for:
    - Self-play training
    - Multi-agent RL
    - Evaluation against external policies

    Note: For proper self-play training, consider using PettingZoo's
    ParallelEnv or AECEnv interfaces instead.
    """

    def __init__(
        self,
        deck1: str = "artificer_tokens",
        deck2: str = "broodmother_swarm",
        game_mode: str = "attrition",
        max_turns: int = 100,
        render_mode: str | None = None,
    ) -> None:
        super().__init__(
            deck1=deck1,
            deck2=deck2,
            opponent=None,  # No built-in opponent
            game_mode=game_mode,
            max_turns=max_turns,
            render_mode=render_mode,
        )

    def step(
        self,
        action: int,
    ) -> tuple[np.ndarray, SupportsFloat, bool, bool, dict[str, Any]]:
        """
        Execute one step. Reward is from current player's perspective.
        """
        self._step_count += 1
        current_player = self._game.current_player()

        # Validate action
        mask = self._game.action_mask()
        if mask[action] == 0.0:
            obs = self._get_obs()
            info = self._get_info()
            info["invalid_action"] = True
            return obs, -10.0, True, False, info

        # Execute action
        reward, done = self._game.step(action)

        # Reward is from Player 1's perspective; flip if we were Player 2
        if current_player == 1:
            reward = -reward

        truncated = self._step_count >= self.max_turns and not done

        obs = self._get_obs()
        info = self._get_info()
        info["current_player_was"] = current_player

        return obs, reward, done, truncated, info


class VectorizedEssenceWars:
    """
    Vectorized environment for high-throughput training.

    This environment wraps PyParallelGames from the Rust engine to enable
    batched operations across many game instances simultaneously. This is
    essential for achieving high samples-per-second (SPS) in RL training.

    Unlike Gymnasium's VectorEnv, this class provides a simpler interface
    optimized for the Essence Wars game engine.

    Features:
    - Batch reset and step operations
    - Auto-reset for done environments
    - Action masking for all environments
    - ~60,000+ SPS on modern hardware

    Args:
        num_envs: Number of parallel environments
        deck1: Deck name for player 1 in all environments
        deck2: Deck name for player 2 in all environments
        game_mode: "attrition" or "essence_duel"

    Example:
        vec_env = VectorizedEssenceWars(num_envs=64)
        obs, masks = vec_env.reset(seed=42)

        while True:
            # Select actions using masks (shape: [64, 256])
            actions = policy.select_actions(obs, masks)

            obs, rewards, dones, masks = vec_env.step(actions)

            # Done environments are auto-reset, no manual handling needed
            if training_complete:
                break
    """

    def __init__(
        self,
        num_envs: int = 64,
        deck1: str = "artificer_tokens",
        deck2: str = "broodmother_swarm",
        game_mode: str = "attrition",
    ) -> None:
        self.num_envs = num_envs
        self.deck1 = deck1
        self.deck2 = deck2
        self.game_mode = game_mode

        # Create parallel games instance (positional num_envs)
        self._games = PyParallelGames(
            num_envs,  # positional argument
            deck1,
            deck2,
            game_mode,
        )

        # Space definitions (for reference, not enforced)
        self.single_observation_space: spaces.Box = spaces.Box(
            low=-np.inf,
            high=np.inf,
            shape=(STATE_TENSOR_SIZE,),
            dtype=np.float32,
        )
        self.single_action_space: spaces.Space[int] = spaces.Discrete(ACTION_SPACE_SIZE)

        # Track episode statistics
        self._episode_rewards = np.zeros(num_envs, dtype=np.float32)
        self._episode_lengths = np.zeros(num_envs, dtype=np.int32)

    def reset(
        self,
        *,
        seed: int | None = None,
    ) -> tuple[np.ndarray, np.ndarray]:
        """
        Reset all environments.

        Args:
            seed: Base seed for reproducibility. Each environment gets
                  seed + env_index for deterministic but varied games.

        Returns:
            observations: Array of shape (num_envs, 326)
            action_masks: Array of shape (num_envs, 256) with dtype bool
        """
        if seed is None:
            seed = np.random.randint(0, 2**31 - 1)

        # Reset all environments with base seed
        self._games.reset_all(seed)

        # Reset episode tracking
        self._episode_rewards.fill(0.0)
        self._episode_lengths.fill(0)

        observations = self._games.observe_batch()
        action_masks = self._games.action_mask_batch() > 0.0

        return observations, action_masks

    def step(
        self,
        actions: np.ndarray,
    ) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
        """
        Step all environments with the given actions.

        Done environments are automatically reset. The returned observations
        and masks for reset environments are from the new episode.

        Args:
            actions: Array of action indices, shape (num_envs,)

        Returns:
            observations: Array of shape (num_envs, 326)
            rewards: Array of shape (num_envs,) with values in {-1, 0, +1}
            dones: Boolean array of shape (num_envs,)
            action_masks: Array of shape (num_envs, 256) with dtype bool
        """
        actions = np.asarray(actions, dtype=np.uint8)

        # Step all games
        rewards, dones = self._games.step_batch(actions)

        # Convert to numpy arrays with correct dtypes
        rewards = np.asarray(rewards, dtype=np.float32)
        dones = np.asarray(dones, dtype=bool)

        # Update episode tracking
        self._episode_lengths += 1
        self._episode_rewards += rewards

        # Auto-reset only done environments
        done_indices = np.where(dones)[0]
        if len(done_indices) > 0:
            for idx in done_indices:
                new_seed = np.random.randint(0, 2**31 - 1)
                self._games.reset_single(int(idx), new_seed)

            # Reset tracking for done episodes
            self._episode_rewards[done_indices] = 0.0
            self._episode_lengths[done_indices] = 0

        observations = self._games.observe_batch()
        action_masks = self._games.action_mask_batch() > 0.0

        return observations, rewards, dones, action_masks

    def step_async(self, actions: np.ndarray) -> None:
        """
        Begin stepping environments asynchronously.

        For compatibility with async training loops. Currently synchronous.
        """
        self._pending_actions = np.asarray(actions, dtype=np.uint8)

    def step_wait(self) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
        """
        Wait for async step to complete and return results.

        For compatibility with async training loops. Currently synchronous.
        """
        return self.step(self._pending_actions)

    def close(self) -> None:
        """Clean up resources."""
        pass

    @property
    def observation_shape(self) -> tuple[int, ...]:
        """Shape of a single observation."""
        return (STATE_TENSOR_SIZE,)

    @property
    def action_size(self) -> int:
        """Number of possible actions."""
        size: int = ACTION_SPACE_SIZE
        return size

    def action_masks(self) -> np.ndarray:
        """
        Get current action masks for all environments.

        Returns:
            Boolean array of shape (num_envs, 256)
        """
        masks: np.ndarray = self._games.action_mask_batch()
        result: np.ndarray = masks > 0.0
        return result

    def get_episode_rewards(self) -> np.ndarray:
        """Get cumulative rewards for current episodes."""
        return self._episode_rewards.copy()

    def get_episode_lengths(self) -> np.ndarray:
        """Get current episode lengths."""
        return self._episode_lengths.copy()


def make_env(
    deck1: str = "artificer_tokens",
    deck2: str = "broodmother_swarm",
    opponent: str | None = "greedy",
    game_mode: str = "attrition",
    seed: int | None = None,
) -> EssenceWarsEnv:
    """
    Factory function to create an Essence Wars environment.

    This is a convenience function for creating environments with common settings.

    Args:
        deck1: Deck for player 1
        deck2: Deck for player 2 (opponent)
        opponent: "greedy", "random", or None
        game_mode: "attrition" or "essence_duel"
        seed: Random seed (applied on first reset)

    Returns:
        Configured EssenceWarsEnv instance
    """
    env = EssenceWarsEnv(
        deck1=deck1,
        deck2=deck2,
        opponent=opponent,
        game_mode=game_mode,
    )
    return env


# Register with Gymnasium
try:
    gym.register(
        id="EssenceWars-v0",
        entry_point="essence_wars.env:EssenceWarsEnv",
        kwargs={
            "deck1": "artificer_tokens",
            "deck2": "broodmother_swarm",
            "opponent": "greedy",
            "game_mode": "attrition",
        },
    )

    gym.register(
        id="EssenceWarsSelfPlay-v0",
        entry_point="essence_wars.env:EssenceWarsSelfPlayEnv",
        kwargs={
            "deck1": "artificer_tokens",
            "deck2": "broodmother_swarm",
            "game_mode": "attrition",
        },
    )
except gym.error.Error:
    # Already registered (happens on reimport)
    pass
