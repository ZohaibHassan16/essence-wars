"""
Neural MCTS Bot for Essence Wars.

This module provides MCTS-augmented inference for trained neural networks.
Instead of using the raw network output, we run MCTS with the network
as a prior (policy) and evaluator (value), similar to how AlphaZero plays.

Key benefits:
- Combines learned intuition (neural net) with search (MCTS)
- Can boost performance of any trained model
- No additional training required

Usage:
    from essence_wars.agents.neural_mcts import NeuralMctsBot

    # Load any trained network
    network = load_network("models/ppo_argentum.pt")

    # Create MCTS-augmented bot
    bot = NeuralMctsBot(network, num_simulations=100)

    # Use like any other bot
    action = bot.select_action(state_tensor, legal_mask, legal_actions)
"""

from __future__ import annotations

import math
import time
from dataclasses import dataclass
from typing import TYPE_CHECKING, Protocol

import numpy as np
import torch
import torch.nn as nn

from essence_wars._core import PyGame

if TYPE_CHECKING:
    from numpy.typing import NDArray


class NetworkProtocol(Protocol):
    """Protocol for networks that can be used with NeuralMctsBot."""

    def forward(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor | None = None,
    ) -> tuple[torch.Tensor, torch.Tensor]:
        """Forward pass returning (logits, value)."""
        ...

    def to(self, device: torch.device) -> "NetworkProtocol":
        """Move to device."""
        ...

    def eval(self) -> "NetworkProtocol":
        """Set to evaluation mode."""
        ...


@dataclass
class NeuralMctsConfig:
    """Configuration for Neural MCTS."""

    num_simulations: int = 100  # MCTS simulations per move
    c_puct: float = 1.5  # Exploration constant
    temperature: float = 0.0  # Action selection temperature (0 = greedy)
    add_noise: bool = False  # Add Dirichlet noise at root (for training, not inference)
    dirichlet_alpha: float = 0.3
    dirichlet_epsilon: float = 0.25
    device: str = "cuda" if torch.cuda.is_available() else "cpu"
    use_value: bool = True  # Whether to use value function for leaf evaluation
    rollout_policy: str = "random"  # "random", "greedy", or "neural" for rollouts


class MCTSNode:
    """
    Node in the MCTS tree.

    Stores visit counts, value sums, and prior probabilities.
    All values are stored from player 0's perspective for consistency.
    """

    __slots__ = ['prior', 'visit_count', 'value_sum', 'children', 'is_expanded']

    def __init__(self, prior: float = 0.0) -> None:
        self.prior = prior
        self.visit_count = 0
        self.value_sum = 0.0  # Sum of values from player 0's perspective
        self.children: dict[int, MCTSNode] = {}
        self.is_expanded = False

    @property
    def value(self) -> float:
        """Mean value Q(s,a) = W(s,a) / N(s,a) from player 0's perspective."""
        if self.visit_count == 0:
            return 0.0
        return self.value_sum / self.visit_count

    def expand(self, legal_actions: list[int], priors: NDArray[np.float32]) -> None:
        """Expand node with children for each legal action."""
        self.is_expanded = True

        # Normalize priors over legal actions
        legal_priors = priors[legal_actions]
        legal_priors = legal_priors / (legal_priors.sum() + 1e-8)

        for i, action in enumerate(legal_actions):
            self.children[action] = MCTSNode(prior=legal_priors[i])

    def select_child(self, c_puct: float, current_player: int) -> tuple[int, "MCTSNode"]:
        """Select child with highest UCB score.

        Values are stored from player 0's perspective.
        - At player 0's turn: maximize Q (higher Q = better for P0)
        - At player 1's turn: minimize Q (higher Q = better for P0 = worse for P1)
        """
        best_score = -float("inf")
        best_action = -1
        best_child = None

        sqrt_total = math.sqrt(self.visit_count + 1)

        for action, child in self.children.items():
            # Q is from player 0's perspective
            q_value = child.value
            # At player 1's turn, flip sign (P1 wants low Q for P0)
            if current_player == 1:
                q_value = -q_value
            u_value = c_puct * child.prior * sqrt_total / (1 + child.visit_count)
            score = q_value + u_value

            if score > best_score:
                best_score = score
                best_action = action
                best_child = child

        assert best_child is not None
        return best_action, best_child

    def add_exploration_noise(self, alpha: float, epsilon: float) -> None:
        """Add Dirichlet noise to priors for exploration."""
        actions = list(self.children.keys())
        if len(actions) == 0:
            return
        noise = np.random.dirichlet([alpha] * len(actions))
        for i, action in enumerate(actions):
            self.children[action].prior = (1 - epsilon) * self.children[action].prior + epsilon * noise[i]


class ObsNormalizer:
    """Simple observation normalizer that matches PPO's RunningMeanStd."""

    def __init__(self, mean: np.ndarray, var: np.ndarray, epsilon: float = 1e-8):
        self.mean = mean
        self.var = var
        self.epsilon = epsilon

    def normalize(self, x: np.ndarray) -> np.ndarray:
        return (x - self.mean) / np.sqrt(self.var + self.epsilon)


class NeuralMctsBot:
    """
    MCTS-augmented neural network bot.

    Uses a trained neural network as prior and value function inside MCTS.
    This combines the fast pattern recognition of neural networks with
    the deep search capabilities of MCTS.

    Args:
        network: Any network with forward(obs, mask) -> (logits, value)
        num_simulations: Number of MCTS simulations per move
        c_puct: Exploration constant (higher = more exploration)
        temperature: Action selection temperature (0 = greedy, 1 = proportional)
        device: Device to run network on
        obs_normalizer: Optional observation normalizer (for PPO models)

    Example:
        # Load PPO network with normalizer
        network, normalizer = load_ppo_network("models/ppo_argentum.pt")

        # Create MCTS bot
        bot = NeuralMctsBot(network, num_simulations=100, obs_normalizer=normalizer)

        # Play
        action = bot.select_action(state, mask, legal_actions)
    """

    def __init__(
        self,
        network: nn.Module,
        num_simulations: int = 100,
        c_puct: float = 1.5,
        temperature: float = 0.0,
        device: str | None = None,
        obs_normalizer: ObsNormalizer | None = None,
        use_value: bool = True,
        rollout_policy: str = "random",
    ) -> None:
        self.config = NeuralMctsConfig(
            num_simulations=num_simulations,
            c_puct=c_puct,
            temperature=temperature,
            device=device or ("cuda" if torch.cuda.is_available() else "cpu"),
            use_value=use_value,
            rollout_policy=rollout_policy,
        )
        self.device = torch.device(self.config.device)
        self.network = network.to(self.device)
        self.network.eval()
        self.obs_normalizer = obs_normalizer

        # Stats for debugging
        self.last_search_time = 0.0
        self.last_root_value = 0.0

    def name(self) -> str:
        """Return bot name."""
        return f"NeuralMCTS-{self.config.num_simulations}"

    def reset(self) -> None:
        """Reset bot state (no-op for MCTS)."""
        pass

    def select_action(
        self,
        state_tensor: NDArray[np.float32],
        legal_mask: NDArray[np.float32],
        legal_actions: list,
    ) -> int:
        """
        Select action using MCTS with neural network prior.

        Args:
            state_tensor: Game state observation (326,)
            legal_mask: Action mask where 1 = legal (256,)
            legal_actions: List of legal Action objects

        Returns:
            Index of selected action
        """
        # Convert legal_actions to indices if needed
        if legal_actions and hasattr(legal_actions[0], 'to_index'):
            legal_indices = [a.to_index() for a in legal_actions]
        else:
            legal_indices = list(legal_actions)

        # Run MCTS search
        action_probs = self._search(state_tensor, legal_mask, legal_indices)

        # Select action
        if self.config.temperature == 0:
            # Greedy selection
            action = int(np.argmax(action_probs))
        else:
            # Sample with temperature
            probs = action_probs ** (1 / self.config.temperature)
            probs = probs / probs.sum()
            action = int(np.random.choice(256, p=probs))

        return action

    def _search(
        self,
        obs: NDArray[np.float32],
        mask: NDArray[np.float32],
        legal_actions: list[int],
    ) -> NDArray[np.float32]:
        """
        Run MCTS search from current state.

        Args:
            obs: Observation array (326,)
            mask: Action mask (256,)
            legal_actions: List of legal action indices

        Returns:
            Action probabilities based on visit counts (256,)
        """
        start_time = time.time()

        # Create root node
        root = MCTSNode()

        # Evaluate root with network
        policy, value = self._evaluate(obs, mask)
        root.expand(legal_actions, policy)
        self.last_root_value = value

        # Add exploration noise if enabled
        if self.config.add_noise and len(legal_actions) > 1:
            root.add_exploration_noise(
                self.config.dirichlet_alpha,
                self.config.dirichlet_epsilon,
            )

        # We need a game instance to simulate
        # For now, we'll use a simplified approach that doesn't require game forking
        # This requires the state tensor to be enough to reconstruct the game
        #
        # NOTE: For full MCTS, we need access to the game's fork() method
        # This version does a single-level search (policy + value only)
        # For deeper search, use NeuralMctsGameBot which takes PyGame

        # Single-level "search" - just use the policy weighted by value
        # This is not full MCTS but gives us a baseline
        self.last_search_time = time.time() - start_time

        # Return policy from network (this is the "0 simulation" baseline)
        # For actual MCTS we need game state access
        return policy

    def get_action_with_game(
        self,
        game: PyGame,
        add_noise: bool = False,
    ) -> tuple[int, NDArray[np.float32]]:
        """
        Select action using full MCTS with game state.

        This method performs actual MCTS by forking the game state.

        Args:
            game: Current game state
            add_noise: Whether to add exploration noise at root

        Returns:
            (action, action_probs) tuple
        """
        start_time = time.time()

        # Get observation and legal actions
        obs = game.observe()
        mask = game.action_mask()
        legal_actions = np.where(mask > 0)[0].tolist()

        if not legal_actions:
            return 255, np.zeros(256, dtype=np.float32)  # End turn

        # Create root node
        root = MCTSNode()

        # Evaluate root
        policy, value = self._evaluate(obs, mask)
        root.expand(legal_actions, policy)
        self.last_root_value = value

        # Add exploration noise
        if add_noise and len(legal_actions) > 1:
            root.add_exploration_noise(
                self.config.dirichlet_alpha,
                self.config.dirichlet_epsilon,
            )

        # All values stored from player 0's perspective
        root_player = game.current_player()

        # Run simulations
        for _ in range(self.config.num_simulations):
            # Fork game for simulation
            sim_game = game.fork()
            node = root
            search_path = [node]
            current_player = root_player

            # Selection: traverse tree using UCB
            while node.is_expanded and not sim_game.is_done():
                action, node = node.select_child(self.config.c_puct, current_player)
                sim_game.step(action)
                search_path.append(node)
                if not sim_game.is_done():
                    current_player = sim_game.current_player()

            # Get value from the leaf position (always from player 0's perspective)
            if sim_game.is_done():
                # Terminal: use actual result from player 0's perspective
                value = sim_game.get_reward(0)
            else:
                # Non-terminal: expand and evaluate
                sim_obs = sim_game.observe()
                sim_mask = sim_game.action_mask()
                sim_legal = np.where(sim_mask > 0)[0].tolist()

                if sim_legal:
                    policy, nn_value = self._evaluate(sim_obs, sim_mask)
                    leaf_player = sim_game.current_player()
                    node.expand(sim_legal, policy)

                    if self.config.use_value:
                        # Use neural network value estimate
                        value = nn_value
                        # Convert to player 0's perspective
                        if leaf_player != 0:
                            value = -value
                    else:
                        # Do rollout to get actual outcome
                        value = self._rollout(sim_game)
                else:
                    value = 0.0

            # Backup: propagate value up the tree
            # Value is always from player 0's perspective - just accumulate it
            for node in reversed(search_path):
                node.visit_count += 1
                node.value_sum += value

        # Get action probabilities from visit counts
        action_probs = np.zeros(256, dtype=np.float32)
        total_visits = sum(child.visit_count for child in root.children.values())

        if total_visits > 0:
            for action, child in root.children.items():
                action_probs[action] = child.visit_count / total_visits

        self.last_search_time = time.time() - start_time

        # Select action
        if self.config.temperature == 0:
            action = int(np.argmax(action_probs))
        else:
            probs = action_probs ** (1 / self.config.temperature)
            probs = probs / (probs.sum() + 1e-8)
            action = int(np.random.choice(256, p=probs))

        return action, action_probs

    def _evaluate(
        self,
        obs: NDArray[np.float32],
        mask: NDArray[np.float32],
    ) -> tuple[NDArray[np.float32], float]:
        """
        Evaluate position with neural network.

        Args:
            obs: Observation array (326,)
            mask: Action mask (256,)

        Returns:
            (policy, value) tuple where policy is softmax probabilities
        """
        # Apply observation normalization if available
        if self.obs_normalizer is not None:
            obs = self.obs_normalizer.normalize(obs).astype(np.float32)

        obs_t = torch.tensor(obs, dtype=torch.float32, device=self.device).unsqueeze(0)
        mask_t = torch.tensor(mask > 0, dtype=torch.bool, device=self.device).unsqueeze(0)

        with torch.no_grad():
            logits, value = self.network(obs_t, mask_t)
            policy = torch.softmax(logits, dim=-1)

        return policy.squeeze(0).cpu().numpy(), value.squeeze().item()

    def _rollout(self, game: PyGame) -> float:
        """
        Do a rollout from the given game state to terminal.

        Args:
            game: Game state to rollout from (will be forked)

        Returns:
            Reward from player 0's perspective
        """
        rollout_game = game.fork()

        while not rollout_game.is_done():
            if self.config.rollout_policy == "greedy":
                action = rollout_game.greedy_action()
            elif self.config.rollout_policy == "neural":
                # Use neural network policy for rollout
                obs = rollout_game.observe()
                mask = rollout_game.action_mask()
                policy, _ = self._evaluate(obs, mask)
                action = int(np.argmax(policy))
            else:  # "random"
                action = rollout_game.random_action()

            rollout_game.step(action)

        return rollout_game.get_reward(0)


def evaluate_neural_mcts(
    network: nn.Module,
    num_simulations: int = 100,
    num_games: int = 100,
    opponent: str = "greedy",
    device: str | None = None,
    verbose: bool = True,
    obs_normalizer: ObsNormalizer | None = None,
) -> dict:
    """
    Evaluate a neural network with MCTS augmentation.

    Args:
        network: Trained neural network
        num_simulations: MCTS simulations per move
        num_games: Number of evaluation games
        opponent: "greedy" or "random"
        device: Device to use
        verbose: Print progress
        obs_normalizer: Optional observation normalizer

    Returns:
        Dict with win_rate, avg_search_time, etc.
    """
    from essence_wars._core import PyGame

    bot = NeuralMctsBot(
        network=network,
        num_simulations=num_simulations,
        device=device,
        obs_normalizer=obs_normalizer,
    )

    wins = 0
    total_search_time = 0.0
    total_moves = 0

    for game_idx in range(num_games):
        game = PyGame()
        game.reset(seed=game_idx)

        while not game.is_done():
            current_player = game.current_player()

            if current_player == 0:
                # Neural MCTS bot
                action, _ = bot.get_action_with_game(game)
                total_search_time += bot.last_search_time
                total_moves += 1
            else:
                # Opponent
                mask = game.action_mask()
                legal_actions = np.where(mask > 0)[0]

                if opponent == "greedy":
                    # Use game's built-in greedy evaluation
                    action = game.greedy_action()
                else:
                    # Random
                    action = int(np.random.choice(legal_actions))

            game.step(action)

        # Check result
        reward = game.get_reward(0)
        if reward > 0:
            wins += 1

        if verbose and (game_idx + 1) % 10 == 0:
            print(f"  Game {game_idx + 1}/{num_games}: {wins}/{game_idx + 1} wins ({wins/(game_idx+1)*100:.1f}%)")

    win_rate = wins / num_games
    avg_search_time = total_search_time / total_moves if total_moves > 0 else 0

    return {
        "win_rate": win_rate,
        "wins": wins,
        "games": num_games,
        "avg_search_time_ms": avg_search_time * 1000,
        "total_moves": total_moves,
        "simulations": num_simulations,
    }


def load_ppo_network(
    checkpoint_path: str,
    device: str = "auto",
) -> tuple[nn.Module, ObsNormalizer | None]:
    """
    Load a PPO network from checkpoint.

    Args:
        checkpoint_path: Path to checkpoint file
        device: Device to load to ("auto", "cuda", "cpu")

    Returns:
        Tuple of (network, obs_normalizer) where obs_normalizer may be None
    """
    from essence_wars.agents.networks import EssenceWarsNetwork
    from essence_wars.agents.embeddings import EmbeddedPPONetwork

    if device == "auto":
        device = "cuda" if torch.cuda.is_available() else "cpu"

    checkpoint = torch.load(checkpoint_path, map_location=device, weights_only=False)

    # Determine network type from state dict
    state_dict = checkpoint.get("network_state_dict", checkpoint.get("model_state_dict", checkpoint))

    # Check if it's an embedded network
    if "obs_transformer.card_embedding.weight" in state_dict:
        # Embedded network
        network = EmbeddedPPONetwork()
    else:
        # Flat network
        network = EssenceWarsNetwork()

    network.load_state_dict(state_dict)
    network.to(device)
    network.eval()

    # Load observation normalizer if present
    obs_normalizer = None
    if "obs_normalizer" in checkpoint:
        norm_data = checkpoint["obs_normalizer"]
        obs_normalizer = ObsNormalizer(
            mean=norm_data["mean"],
            var=norm_data["var"],
            epsilon=norm_data.get("epsilon", 1e-8),
        )

    return network, obs_normalizer


def load_bc_network(checkpoint_path: str, device: str = "auto") -> nn.Module:
    """
    Load a BC (behavioral cloning) network from checkpoint.

    Args:
        checkpoint_path: Path to checkpoint file
        device: Device to load to

    Returns:
        Loaded network (AlphaZeroNetwork)
    """
    from essence_wars.agents.networks import AlphaZeroNetwork

    if device == "auto":
        device = "cuda" if torch.cuda.is_available() else "cpu"

    checkpoint = torch.load(checkpoint_path, map_location=device, weights_only=False)

    # BC uses AlphaZeroNetwork
    network = AlphaZeroNetwork()
    network.load_state_dict(checkpoint["model_state_dict"])
    network.to(device)
    network.eval()

    return network
