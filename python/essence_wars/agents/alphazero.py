"""
AlphaZero-style training for Essence Wars.

This module provides:
- NeuralMCTS: MCTS guided by neural network policy and value
- AlphaZeroTrainer: Self-play training loop with replay buffer

Key differences from vanilla MCTS:
- Prior probabilities from policy network (not uniform)
- Leaf evaluation from value network (not rollout)
- PUCT selection formula for exploration-exploitation

Based on the AlphaZero paper (Silver et al., 2017).
"""

from __future__ import annotations

import math
import random
import time
from collections import deque
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any

import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim

from essence_wars._core import STATE_TENSOR_SIZE, PyGame
from essence_wars.agents.networks import AlphaZeroNetwork
from essence_wars.env import EssenceWarsEnv

if TYPE_CHECKING:
    from numpy.typing import NDArray

# Type alias for replay buffer samples
ReplaySample = tuple[np.ndarray, np.ndarray, np.ndarray, float]


@dataclass
class AlphaZeroConfig:
    """Configuration for AlphaZero training."""

    # MCTS
    num_simulations: int = 100  # MCTS simulations per move
    c_puct: float = 1.5  # Exploration constant
    dirichlet_alpha: float = 0.3  # Noise for root exploration
    dirichlet_epsilon: float = 0.25  # Noise weight
    mcts_batch_size: int = 32  # Batch size for GPU inference during MCTS
    mcts_virtual_loss: float = 3.0  # Virtual loss for batched MCTS

    # Training
    num_iterations: int = 100  # Training iterations
    games_per_iteration: int = 100  # Self-play games per iteration
    training_steps_per_iteration: int = 100  # Gradient steps per iteration
    batch_size: int = 256
    learning_rate: float = 1e-3
    weight_decay: float = 1e-4

    # Replay buffer
    replay_buffer_size: int = 100_000
    min_replay_size: int = 1000  # Min samples before training starts

    # BC warm-start (dual buffer mode)
    bc_ratio: float = 0.7  # Ratio of BC samples in training batches (0.7 = 70% BC)

    # Network
    hidden_dim: int = 256
    num_blocks: int = 4

    # Temperature for action selection
    temperature_moves: int = 30  # Use temperature=1 for first N moves
    temperature_final: float = 0.1  # Temperature after N moves

    # Evaluation
    eval_interval: int = 5  # Evaluate every N iterations
    eval_games: int = 50  # Games for evaluation

    # Checkpointing
    checkpoint_interval: int = 10  # Save checkpoint every N iterations (0 = only at end)

    # Device
    device: str = "cuda" if torch.cuda.is_available() else "cpu"

    # Observation normalization (reuse from PPO)
    normalize_obs: bool = True


class MCTSNode:
    """
    Node in the MCTS tree.

    Each node stores:
    - Visit count N(s,a) for each action
    - Total value W(s,a) for each action
    - Prior probability P(s,a) from neural network
    - Children nodes
    """

    def __init__(
        self,
        prior: float = 0.0,
        parent: MCTSNode | None = None,
        action: int | None = None,
    ) -> None:
        self.prior = prior
        self.parent = parent
        self.action = action  # Action that led to this node

        self.visit_count = 0
        self.value_sum = 0.0

        # Children: action -> MCTSNode
        self.children: dict[int, MCTSNode] = {}

        # Whether this node has been expanded
        self.is_expanded = False

    @property
    def value(self) -> float:
        """Mean value Q(s,a) = W(s,a) / N(s,a)."""
        if self.visit_count == 0:
            return 0.0
        return self.value_sum / self.visit_count

    def expand(
        self,
        legal_actions: list[int],
        priors: NDArray[np.float32],
    ) -> None:
        """
        Expand this node with children for each legal action.

        Args:
            legal_actions: List of legal action indices
            priors: Prior probabilities for all actions (256,)
        """
        self.is_expanded = True

        # Normalize priors over legal actions only
        legal_priors = priors[legal_actions]
        legal_priors = legal_priors / (legal_priors.sum() + 1e-8)

        for i, action in enumerate(legal_actions):
            self.children[action] = MCTSNode(
                prior=legal_priors[i],
                parent=self,
                action=action,
            )

    def select_child(self, c_puct: float) -> tuple[int, MCTSNode]:
        """
        Select child with highest UCB score.

        UCB(s,a) = Q(s,a) + c_puct * P(s,a) * sqrt(N(s)) / (1 + N(s,a))

        Args:
            c_puct: Exploration constant

        Returns:
            (action, child_node) tuple
        """
        best_score = -float("inf")
        best_action = -1
        best_child = None

        sqrt_total = math.sqrt(self.visit_count + 1)

        for action, child in self.children.items():
            # UCB score
            q_value = child.value
            u_value = c_puct * child.prior * sqrt_total / (1 + child.visit_count)
            score = q_value + u_value

            if score > best_score:
                best_score = score
                best_action = action
                best_child = child

        assert best_child is not None
        return best_action, best_child

    def add_exploration_noise(
        self,
        dirichlet_alpha: float,
        epsilon: float,
    ) -> None:
        """
        Add Dirichlet noise to priors at root for exploration.

        Args:
            dirichlet_alpha: Dirichlet distribution parameter
            epsilon: Weight of noise (1-epsilon is weight of original prior)
        """
        actions = list(self.children.keys())
        noise = np.random.dirichlet([dirichlet_alpha] * len(actions))

        for i, action in enumerate(actions):
            child = self.children[action]
            child.prior = (1 - epsilon) * child.prior + epsilon * noise[i]

    def apply_virtual_loss(self, virtual_loss: float = 3.0) -> None:
        """Apply virtual loss to discourage other threads from selecting same path."""
        self.visit_count += 1
        self.value_sum -= virtual_loss

    def remove_virtual_loss(self, virtual_loss: float = 3.0) -> None:
        """Remove virtual loss after evaluation is complete."""
        self.visit_count -= 1
        self.value_sum += virtual_loss


class RunningMeanStd:
    """Running mean and standard deviation for observation normalization."""

    def __init__(self, shape: tuple[int, ...], epsilon: float = 1e-8):
        self.mean = np.zeros(shape, dtype=np.float64)
        self.var = np.ones(shape, dtype=np.float64)
        self.count = epsilon
        self.epsilon = epsilon

    def update(self, batch: np.ndarray) -> None:
        """Update running statistics with a batch of observations."""
        batch = np.asarray(batch)
        if batch.ndim == 1:
            batch = batch.reshape(1, -1)
        batch_mean = batch.mean(axis=0)
        batch_var = batch.var(axis=0)
        batch_count = batch.shape[0]
        self._update_from_moments(batch_mean, batch_var, batch_count)

    def _update_from_moments(self, batch_mean, batch_var, batch_count):
        delta = batch_mean - self.mean
        tot_count = self.count + batch_count
        new_mean = self.mean + delta * batch_count / tot_count
        m_a = self.var * self.count
        m_b = batch_var * batch_count
        m_2 = m_a + m_b + np.square(delta) * self.count * batch_count / tot_count
        new_var = m_2 / tot_count
        self.mean = new_mean
        self.var = new_var
        self.count = tot_count

    def normalize(self, x: np.ndarray) -> np.ndarray:
        return (x - self.mean) / np.sqrt(self.var + self.epsilon)


class NeuralMCTS:
    """
    MCTS guided by neural network policy and value.

    Key features:
    - Uses neural network for prior probabilities and leaf evaluation
    - PUCT formula for exploration-exploitation balance
    - Dirichlet noise at root for exploration

    Args:
        network: AlphaZeroNetwork for policy and value
        config: AlphaZeroConfig with MCTS parameters
        obs_normalizer: Optional observation normalizer

    Example:
        network = AlphaZeroNetwork()
        mcts = NeuralMCTS(network, config)

        game = PyGame()
        game.reset(seed=42)

        action_probs = mcts.search(game)
        action = np.random.choice(256, p=action_probs)
    """

    def __init__(
        self,
        network: AlphaZeroNetwork,
        config: AlphaZeroConfig,
        obs_normalizer: RunningMeanStd | None = None,
    ) -> None:
        self.config = config
        self.device = torch.device(config.device)
        self.network = network.to(self.device)
        self.obs_normalizer = obs_normalizer

    def search(
        self,
        game: PyGame,
        add_noise: bool = True,
    ) -> NDArray[np.float32]:
        """
        Run MCTS search from current game state.

        Args:
            game: Current game state (will be forked, not modified)
            add_noise: Whether to add Dirichlet noise at root

        Returns:
            Action probabilities based on visit counts (256,)
        """
        # Create root node
        root = MCTSNode()

        # Get initial evaluation
        obs = game.observe()
        mask = game.action_mask()
        legal_actions = np.where(mask > 0)[0].tolist()

        # Evaluate root position
        policy, _ = self._evaluate(obs, mask)
        root.expand(legal_actions, policy)

        # Add exploration noise at root
        if add_noise and len(legal_actions) > 1:
            root.add_exploration_noise(
                self.config.dirichlet_alpha,
                self.config.dirichlet_epsilon,
            )

        # Run simulations
        for _ in range(self.config.num_simulations):
            # Fork game for simulation
            sim_game = game.fork()
            node = root
            search_path = [node]

            # Selection: traverse tree using UCB
            while node.is_expanded and not sim_game.is_done():
                action, node = node.select_child(self.config.c_puct)
                sim_game.step(action)
                search_path.append(node)

            # Get value
            if sim_game.is_done():
                # Terminal node: use actual game result
                # Reward is from player 0's perspective
                value = sim_game.get_reward(0)
            else:
                # Non-terminal: expand and evaluate with network
                obs = sim_game.observe()
                mask = sim_game.action_mask()
                legal_actions = np.where(mask > 0)[0].tolist()

                if legal_actions:
                    policy, value = self._evaluate(obs, mask)
                    node.expand(legal_actions, policy)
                else:
                    # No legal actions (shouldn't happen in normal game)
                    value = 0.0

            # Backup: propagate value up the tree
            # Note: value alternates sign as we go up (opponent's perspective)
            sim_game.current_player()
            for node in reversed(search_path):
                # Flip value for opponent's nodes
                node.visit_count += 1
                node.value_sum += value
                value = -value  # Flip for parent (opponent's perspective)

        # Return visit count distribution as action probabilities
        return self._get_action_probs(root)

    def _evaluate(
        self,
        obs: NDArray[np.float32],
        mask: NDArray[np.float32],
    ) -> tuple[NDArray[np.float32], float]:
        """
        Evaluate position with neural network.

        Args:
            obs: Observation array (STATE_TENSOR_SIZE,)
            mask: Action mask (256,)

        Returns:
            (policy, value) tuple
        """
        # Normalize observation if enabled
        if self.obs_normalizer is not None:
            obs = self.obs_normalizer.normalize(obs).astype(np.float32)

        obs_t = torch.tensor(obs, dtype=torch.float32, device=self.device).unsqueeze(0)
        mask_t = torch.tensor(mask > 0, dtype=torch.bool, device=self.device).unsqueeze(0)

        self.network.eval()
        with torch.no_grad():
            policy, value = self.network.evaluate(obs_t, mask_t)

        return policy.squeeze(0).cpu().numpy(), value.item()

    def _get_action_probs(self, root: MCTSNode) -> NDArray[np.float32]:
        """
        Get action probabilities from root visit counts.

        Args:
            root: Root node of MCTS tree

        Returns:
            Action probabilities (256,)
        """
        probs = np.zeros(256, dtype=np.float32)
        total_visits = sum(child.visit_count for child in root.children.values())

        if total_visits > 0:
            for action, child in root.children.items():
                probs[action] = child.visit_count / total_visits

        return probs

    def _evaluate_batch(
        self,
        obs_batch: NDArray[np.float32],
        mask_batch: NDArray[np.float32],
    ) -> tuple[NDArray[np.float32], NDArray[np.float32]]:
        """
        Evaluate a batch of positions with neural network.

        Args:
            obs_batch: Observations array (batch_size, STATE_TENSOR_SIZE)
            mask_batch: Action masks (batch_size, 256)

        Returns:
            (policies, values) tuple - policies (batch_size, 256), values (batch_size,)
        """
        # Normalize observations if enabled
        if self.obs_normalizer is not None:
            obs_batch = np.stack([
                self.obs_normalizer.normalize(obs).astype(np.float32)
                for obs in obs_batch
            ])

        obs_t = torch.tensor(obs_batch, dtype=torch.float32, device=self.device)
        mask_t = torch.tensor(mask_batch > 0, dtype=torch.bool, device=self.device)

        self.network.eval()
        with torch.no_grad():
            policies, values = self.network.evaluate(obs_t, mask_t)

        return policies.cpu().numpy(), values.cpu().numpy().flatten()

    def search_batched(
        self,
        game: PyGame,
        batch_size: int = 32,
        add_noise: bool = True,
        virtual_loss: float = 3.0,
    ) -> NDArray[np.float32]:
        """
        Run batched MCTS search for GPU efficiency.

        Collects multiple leaf nodes before doing a single batched GPU inference,
        using virtual loss to encourage exploration of different paths.

        Args:
            game: Current game state (will be forked, not modified)
            batch_size: Number of leaves to collect before GPU inference
            add_noise: Whether to add Dirichlet noise at root
            virtual_loss: Virtual loss to apply during selection

        Returns:
            Action probabilities based on visit counts (256,)
        """
        # Create root node
        root = MCTSNode()

        # Get initial evaluation
        obs = game.observe()
        mask = game.action_mask()
        legal_actions = np.where(mask > 0)[0].tolist()

        # Evaluate root position
        policy, _ = self._evaluate(obs, mask)
        root.expand(legal_actions, policy)

        # Add exploration noise at root
        if add_noise and len(legal_actions) > 1:
            root.add_exploration_noise(
                self.config.dirichlet_alpha,
                self.config.dirichlet_epsilon,
            )

        # Run simulations in batches
        remaining_sims = self.config.num_simulations
        while remaining_sims > 0:
            # Collect leaves for this batch
            current_batch = min(batch_size, remaining_sims)
            leaves_to_evaluate = []

            for _ in range(current_batch):
                # Fork game for simulation
                sim_game = game.fork()
                node = root
                search_path = [node]

                # Selection with virtual loss
                while node.is_expanded and not sim_game.is_done():
                    action, node = node.select_child(self.config.c_puct)
                    node.apply_virtual_loss(virtual_loss)
                    sim_game.step(action)
                    search_path.append(node)

                # Check if terminal
                if sim_game.is_done():
                    # Terminal: use actual game result, remove virtual loss and backup
                    value = sim_game.get_reward(0)
                    for path_node in reversed(search_path[1:]):
                        path_node.remove_virtual_loss(virtual_loss)
                        path_node.visit_count += 1
                        path_node.value_sum += value
                        value = -value
                    root.visit_count += 1
                else:
                    # Non-terminal: collect for batch evaluation
                    leaf_obs = sim_game.observe()
                    leaf_mask = sim_game.action_mask()
                    leaf_legal = np.where(leaf_mask > 0)[0].tolist()

                    if leaf_legal:
                        leaves_to_evaluate.append({
                            "obs": leaf_obs,
                            "mask": leaf_mask,
                            "legal_actions": leaf_legal,
                            "node": node,
                            "search_path": search_path,
                        })
                    else:
                        # No legal actions: backup with 0 value
                        value = 0.0
                        for path_node in reversed(search_path[1:]):
                            path_node.remove_virtual_loss(virtual_loss)
                            path_node.visit_count += 1
                            path_node.value_sum += value
                            value = -value
                        root.visit_count += 1

            # Batch evaluate all collected leaves
            if leaves_to_evaluate:
                obs_batch = np.stack([leaf["obs"] for leaf in leaves_to_evaluate])
                mask_batch = np.stack([leaf["mask"] for leaf in leaves_to_evaluate])

                policies, values = self._evaluate_batch(obs_batch, mask_batch)

                # Expand and backup each leaf
                for i, leaf in enumerate(leaves_to_evaluate):
                    node = leaf["node"]
                    search_path = leaf["search_path"]

                    # Expand leaf
                    node.expand(leaf["legal_actions"], policies[i])

                    # Remove virtual loss and backup
                    value = values[i]
                    for path_node in reversed(search_path[1:]):
                        path_node.remove_virtual_loss(virtual_loss)
                        path_node.visit_count += 1
                        path_node.value_sum += value
                        value = -value
                    root.visit_count += 1

            remaining_sims -= current_batch

        return self._get_action_probs(root)


@dataclass
class ReplayBuffer:
    """
    Replay buffer for AlphaZero training.

    Stores (observation, action_mask, policy_target, value_target) tuples
    from self-play games.
    """

    capacity: int = 100_000
    buffer: deque[ReplaySample] = field(default_factory=deque)

    def __post_init__(self):
        self.buffer = deque(maxlen=self.capacity)

    def add(
        self,
        obs: NDArray[np.float32],
        mask: NDArray[np.float32],
        policy: NDArray[np.float32],
        value: float,
    ) -> None:
        """Add a training sample to the buffer."""
        self.buffer.append((obs.copy(), mask.copy(), policy.copy(), value))

    def add_game(
        self,
        observations: list[NDArray[np.float32]],
        masks: list[NDArray[np.float32]],
        policies: list[NDArray[np.float32]],
        outcome: float,
    ) -> None:
        """
        Add all positions from a game to the buffer.

        Args:
            observations: List of observations from game
            masks: List of action masks from game
            policies: List of MCTS policy targets from game
            outcome: Game outcome from player 0's perspective (+1/-1)
        """
        # Assign values: each position gets the outcome from its player's perspective
        # Positions alternate between players, so values alternate
        for i, (obs, mask, policy) in enumerate(zip(observations, masks, policies, strict=False)):
            # Even indices are player 0, odd are player 1
            value = outcome if i % 2 == 0 else -outcome
            self.add(obs, mask, policy, value)

    def sample(self, batch_size: int) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
        """
        Sample a random batch from the buffer.

        Returns:
            (observations, masks, policies, values) tensors
        """
        indices = np.random.choice(len(self.buffer), size=batch_size, replace=False)
        batch = [self.buffer[i] for i in indices]

        obs = np.stack([b[0] for b in batch])
        masks = np.stack([b[1] for b in batch])
        policies = np.stack([b[2] for b in batch])
        values = np.array([b[3] for b in batch], dtype=np.float32)

        return obs, masks, policies, values

    def __len__(self) -> int:
        return len(self.buffer)


@dataclass
class DualReplayBuffer:
    """
    Dual replay buffer for BC warm-start training.

    Maintains separate BC and self-play buffers with fixed sampling ratio.
    This prevents BC ratio from dropping as self-play data accumulates,
    solving the "delayed catastrophic forgetting" problem.

    The key insight is that mixing BC and self-play data in a single buffer
    causes the BC ratio to drop over time, eventually leading to collapse.
    By keeping them separate and sampling with a fixed ratio, we maintain
    stable training indefinitely.

    Args:
        bc_ratio: Ratio of BC samples in training batches (0.7 = 70% BC)
        bc_capacity: Maximum BC samples to store
        selfplay_capacity: Maximum self-play samples to store

    Example:
        buffer = DualReplayBuffer(bc_ratio=0.7)

        # Load BC data during initialization
        for sample in bc_dataset:
            buffer.add_bc(sample.obs, sample.mask, sample.policy, sample.value)

        # Add self-play during training
        buffer.add_selfplay_game(observations, masks, policies, outcome)

        # Sample with guaranteed 70% BC ratio
        batch = buffer.sample(256)
    """

    bc_ratio: float = 0.7
    bc_capacity: int = 100_000
    selfplay_capacity: int = 100_000

    def __post_init__(self) -> None:
        self.bc_buffer: deque[ReplaySample] = deque(maxlen=self.bc_capacity)
        self.selfplay_buffer: deque[ReplaySample] = deque(maxlen=self.selfplay_capacity)

    def add_bc(
        self,
        obs: NDArray[np.float32],
        mask: NDArray[np.float32],
        policy: NDArray[np.float32],
        value: float,
    ) -> None:
        """Add sample to BC buffer (typically during initialization)."""
        self.bc_buffer.append((obs.copy(), mask.copy(), policy.copy(), value))

    def add_selfplay(
        self,
        obs: NDArray[np.float32],
        mask: NDArray[np.float32],
        policy: NDArray[np.float32],
        value: float,
    ) -> None:
        """Add sample to self-play buffer (during training)."""
        self.selfplay_buffer.append((obs.copy(), mask.copy(), policy.copy(), value))

    def add_selfplay_game(
        self,
        observations: list[NDArray[np.float32]],
        masks: list[NDArray[np.float32]],
        policies: list[NDArray[np.float32]],
        outcome: float,
    ) -> None:
        """
        Add all positions from a self-play game to the buffer.

        Args:
            observations: List of observations from game
            masks: List of action masks from game
            policies: List of MCTS policy targets from game
            outcome: Game outcome from player 0's perspective (+1/-1)
        """
        for i, (obs, mask, policy) in enumerate(zip(observations, masks, policies, strict=False)):
            value = outcome if i % 2 == 0 else -outcome
            self.add_selfplay(obs, mask, policy, value)

    def sample(self, batch_size: int) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
        """
        Sample batch with fixed BC ratio.

        If self-play buffer doesn't have enough samples, fills remainder
        from BC buffer to ensure full batch size.

        Returns:
            (observations, masks, policies, values) numpy arrays
        """
        bc_size = int(batch_size * self.bc_ratio)
        sp_size = batch_size - bc_size

        # Sample from BC buffer
        bc_indices = np.random.choice(len(self.bc_buffer), size=bc_size, replace=True)
        bc_samples = [self.bc_buffer[i] for i in bc_indices]

        # Sample from self-play buffer (or fallback to BC if not enough)
        if len(self.selfplay_buffer) >= sp_size:
            sp_indices = np.random.choice(
                len(self.selfplay_buffer), size=sp_size, replace=True
            )
            sp_samples = [self.selfplay_buffer[i] for i in sp_indices]
        elif len(self.selfplay_buffer) > 0:
            # Use all self-play samples + fill remainder from BC
            sp_samples = list(self.selfplay_buffer)
            extra_needed = sp_size - len(sp_samples)
            extra_bc = np.random.choice(
                len(self.bc_buffer), size=extra_needed, replace=True
            )
            sp_samples.extend([self.bc_buffer[i] for i in extra_bc])
        else:
            # No self-play yet, use all BC
            extra_bc = np.random.choice(len(self.bc_buffer), size=sp_size, replace=True)
            sp_samples = [self.bc_buffer[i] for i in extra_bc]

        # Combine and shuffle
        all_samples = bc_samples + sp_samples
        random.shuffle(all_samples)

        # Convert to arrays
        obs = np.stack([s[0] for s in all_samples])
        masks = np.stack([s[1] for s in all_samples])
        policies = np.stack([s[2] for s in all_samples])
        values = np.array([s[3] for s in all_samples], dtype=np.float32)

        return obs, masks, policies, values

    def __len__(self) -> int:
        """Total samples across both buffers."""
        return len(self.bc_buffer) + len(self.selfplay_buffer)

    @property
    def bc_count(self) -> int:
        """Number of BC samples."""
        return len(self.bc_buffer)

    @property
    def selfplay_count(self) -> int:
        """Number of self-play samples."""
        return len(self.selfplay_buffer)

    def stats(self) -> dict[str, Any]:
        """Get buffer statistics."""
        total = len(self)
        return {
            "bc_samples": self.bc_count,
            "selfplay_samples": self.selfplay_count,
            "total_samples": total,
            "effective_bc_ratio": self.bc_ratio,  # Always the configured ratio
            "actual_bc_ratio": self.bc_count / total if total > 0 else 0,
        }


class AlphaZeroTrainer:
    """
    AlphaZero training loop.

    Training process:
    1. Self-play: Generate games using current network + MCTS
    2. Store (state, mcts_policy, outcome) in replay buffer
    3. Train network on batches from replay buffer
    4. Repeat

    Args:
        config: AlphaZeroConfig with training parameters
        network: Optional pre-built network
        writer: Optional TensorBoard SummaryWriter

    Example:
        trainer = AlphaZeroTrainer(AlphaZeroConfig())
        trainer.train()
        trainer.save("checkpoints/alphazero_model.pt")
    """

    def __init__(
        self,
        config: AlphaZeroConfig | None = None,
        network: AlphaZeroNetwork | None = None,
        writer=None,
    ) -> None:
        self.config = config or AlphaZeroConfig()
        self.device = torch.device(self.config.device)

        # Create network
        if network is not None:
            self.network = network.to(self.device)
        else:
            self.network = AlphaZeroNetwork(
                hidden_dim=self.config.hidden_dim,
                num_blocks=self.config.num_blocks,
            ).to(self.device)

        # Optimizer
        self.optimizer = optim.Adam(
            self.network.parameters(),
            lr=self.config.learning_rate,
            weight_decay=self.config.weight_decay,
        )

        # Replay buffer (single buffer for standard training)
        self.replay_buffer = ReplayBuffer(capacity=self.config.replay_buffer_size)

        # Dual replay buffer (for BC warm-start, initialized separately)
        self.dual_buffer: DualReplayBuffer | None = None

        # Observation normalizer
        self.obs_normalizer = RunningMeanStd((STATE_TENSOR_SIZE,)) if self.config.normalize_obs else None

        # MCTS
        self.mcts = NeuralMCTS(
            network=self.network,
            config=self.config,
            obs_normalizer=self.obs_normalizer,
        )

        # TensorBoard writer
        self.writer = writer

        # Training state
        self.iteration = 0
        self.total_games = 0
        self.start_time: float | None = None
        self.save_dir: str | None = None  # Will be set by train()

    def init_dual_buffer(
        self,
        bc_samples: list[ReplaySample],
        bc_ratio: float | None = None,
    ) -> None:
        """
        Initialize dual replay buffer with BC data for warm-start training.

        This enables the two-buffer approach that maintains a fixed BC ratio
        throughout training, preventing the "delayed catastrophic forgetting"
        that occurs when BC samples get diluted by self-play data.

        Args:
            bc_samples: List of (obs, mask, policy, value) tuples from BC dataset
            bc_ratio: Override config.bc_ratio if specified

        Example:
            # Load BC data
            bc_dataset = MCTSDataset("data/mcts_10k.jsonl.gz")
            bc_samples = [(s.state_tensor, s.action_mask, s.mcts_policy, s.value_target)
                          for s in bc_dataset.samples]

            # Initialize dual buffer
            trainer.init_dual_buffer(bc_samples, bc_ratio=0.7)
        """
        ratio = bc_ratio if bc_ratio is not None else self.config.bc_ratio

        self.dual_buffer = DualReplayBuffer(
            bc_ratio=ratio,
            bc_capacity=self.config.replay_buffer_size,
            selfplay_capacity=self.config.replay_buffer_size,
        )

        # Load BC samples
        for obs, mask, policy, value in bc_samples:
            self.dual_buffer.add_bc(obs, mask, policy, value)
            # Also update observation normalizer
            if self.obs_normalizer is not None:
                self.obs_normalizer.update(obs)

        print(f"  Initialized dual buffer with {self.dual_buffer.bc_count:,} BC samples")
        print(f"  BC ratio: {ratio:.0%} (fixed throughout training)")

    @property
    def using_dual_buffer(self) -> bool:
        """Check if dual buffer mode is active."""
        return self.dual_buffer is not None

    def self_play_game(self, seed: int | None = None) -> tuple[list[NDArray[np.float32]], list[NDArray[np.float32]], list[NDArray[np.float32]], float]:
        """
        Play one self-play game using MCTS.

        Args:
            seed: Optional random seed for game

        Returns:
            (observations, masks, policies, outcome) tuple
        """
        game = PyGame()
        game.reset(seed if seed is not None else np.random.randint(0, 2**31))

        observations = []
        masks = []
        policies = []
        move_count = 0

        while not game.is_done():
            obs = game.observe()
            mask = game.action_mask()

            # Update normalizer with raw observation
            if self.obs_normalizer is not None:
                self.obs_normalizer.update(obs)

            # MCTS search (batched for GPU efficiency)
            action_probs = self.mcts.search_batched(
                game,
                batch_size=self.config.mcts_batch_size,
                add_noise=True,
                virtual_loss=self.config.mcts_virtual_loss,
            )

            # Store training data
            observations.append(obs)
            masks.append(mask)
            policies.append(action_probs)

            # Select action with temperature
            if move_count < self.config.temperature_moves:
                # Sample proportionally to visit counts
                action = np.random.choice(256, p=action_probs)
            else:
                # Low temperature: almost deterministic
                temp_probs = action_probs ** (1.0 / self.config.temperature_final)
                temp_probs = temp_probs / temp_probs.sum()
                action = np.random.choice(256, p=temp_probs)

            game.step(action)
            move_count += 1

        # Get game outcome from player 0's perspective
        outcome = game.get_reward(0)  # +1 if player 0 won, -1 if lost

        return observations, masks, policies, outcome

    def generate_self_play_games(self, num_games: int) -> None:
        """
        Generate multiple self-play games and add to replay buffer.

        Args:
            num_games: Number of games to generate
        """
        for i in range(num_games):
            observations, masks, policies, outcome = self.self_play_game(
                seed=self.total_games + i
            )
            # Add to appropriate buffer
            if self.dual_buffer is not None:
                self.dual_buffer.add_selfplay_game(observations, masks, policies, outcome)
            else:
                self.replay_buffer.add_game(observations, masks, policies, outcome)
            self.total_games += 1

    def train_step(self) -> dict[str, float]:
        """
        Perform one training step on a batch from replay buffer.

        Returns:
            Dictionary with loss values
        """
        self.network.train()

        # Sample batch from appropriate buffer
        if self.dual_buffer is not None:
            obs, masks, policy_targets, value_targets = self.dual_buffer.sample(
                self.config.batch_size
            )
        else:
            obs, masks, policy_targets, value_targets = self.replay_buffer.sample(
                self.config.batch_size
            )

        # Normalize observations
        if self.obs_normalizer is not None:
            obs = self.obs_normalizer.normalize(obs).astype(np.float32)

        # Convert to tensors
        obs_t = torch.tensor(obs, dtype=torch.float32, device=self.device)
        masks_t = torch.tensor(masks > 0, dtype=torch.bool, device=self.device)
        policy_targets_t = torch.tensor(policy_targets, dtype=torch.float32, device=self.device)
        value_targets_t = torch.tensor(value_targets, dtype=torch.float32, device=self.device)

        # Forward pass
        policy_logits, value_pred = self.network(obs_t, masks_t)

        # Policy loss: cross-entropy with MCTS policy targets
        policy_log_probs = torch.log_softmax(policy_logits, dim=-1)
        policy_loss = -(policy_targets_t * policy_log_probs).sum(dim=-1).mean()

        # Value loss: MSE with game outcomes
        value_loss = nn.functional.mse_loss(value_pred, value_targets_t)

        # Total loss
        total_loss = policy_loss + value_loss

        # Optimize
        self.optimizer.zero_grad()
        total_loss.backward()
        # Gradient clipping
        nn.utils.clip_grad_norm_(self.network.parameters(), max_norm=1.0)
        self.optimizer.step()

        return {
            "policy_loss": policy_loss.item(),
            "value_loss": value_loss.item(),
            "total_loss": total_loss.item(),
        }

    def train(self, num_iterations: int | None = None, save_dir: str | None = None) -> dict[str, Any]:
        """
        Main AlphaZero training loop.

        Args:
            num_iterations: Override config num_iterations
            save_dir: Directory to save checkpoints (for periodic saves)

        Returns:
            Training statistics
        """
        num_iterations = num_iterations or self.config.num_iterations
        self.save_dir = save_dir
        self.start_time = time.time()

        print(f"Starting AlphaZero training for {num_iterations} iterations...")
        print(f"  Games per iteration: {self.config.games_per_iteration}")
        print(f"  Simulations per move: {self.config.num_simulations}")
        print(f"  Device: {self.device}")
        if self.dual_buffer is not None:
            print(f"  Mode: Dual buffer (BC ratio: {self.dual_buffer.bc_ratio:.0%})")
            print(f"  BC samples: {self.dual_buffer.bc_count:,}")
        if self.config.checkpoint_interval > 0:
            print(f"  Checkpoint interval: every {self.config.checkpoint_interval} iterations")

        all_losses = []

        try:
            for iteration in range(1, num_iterations + 1):
                self.iteration = iteration
                iter_start = time.time()

                # Generate self-play games
                print(f"\nIteration {iteration}/{num_iterations}")
                print(f"  Generating {self.config.games_per_iteration} self-play games...")
                self.generate_self_play_games(self.config.games_per_iteration)

                # Print buffer stats
                if self.dual_buffer is not None:
                    stats = self.dual_buffer.stats()
                    print(f"  Buffer: {stats['bc_samples']:,} BC + {stats['selfplay_samples']:,} self-play (sampling {stats['effective_bc_ratio']:.0%} BC)")
                else:
                    print(f"  Replay buffer size: {len(self.replay_buffer)}")

                # Determine if we have enough samples to train
                if self.dual_buffer is not None:
                    # In dual buffer mode, we can train as soon as we have BC data
                    can_train = self.dual_buffer.bc_count >= self.config.min_replay_size
                else:
                    can_train = len(self.replay_buffer) >= self.config.min_replay_size

                # Training
                if can_train:
                    print(f"  Training for {self.config.training_steps_per_iteration} steps...")
                    iter_losses = []
                    for _ in range(self.config.training_steps_per_iteration):
                        loss_info = self.train_step()
                        iter_losses.append(loss_info["total_loss"])
                        all_losses.append(loss_info["total_loss"])

                    mean_loss = np.mean(iter_losses)
                    print(f"  Mean loss: {mean_loss:.4f}")

                    if self.writer is not None:
                        self.writer.add_scalar("loss/total", mean_loss, iteration)
                        self.writer.add_scalar("loss/policy", loss_info["policy_loss"], iteration)
                        self.writer.add_scalar("loss/value", loss_info["value_loss"], iteration)

                # Evaluation
                if iteration % self.config.eval_interval == 0:
                    win_rate_greedy = self.evaluate_vs_greedy(self.config.eval_games)
                    win_rate_random = self.evaluate_vs_random(self.config.eval_games)
                    print(f"  Eval vs Greedy: {win_rate_greedy:.1%}")
                    print(f"  Eval vs Random: {win_rate_random:.1%}")

                    if self.writer is not None:
                        self.writer.add_scalar("eval/win_rate_vs_greedy", win_rate_greedy, iteration)
                        self.writer.add_scalar("eval/win_rate_vs_random", win_rate_random, iteration)

                # Periodic checkpoint saving
                if (
                    self.config.checkpoint_interval > 0
                    and iteration % self.config.checkpoint_interval == 0
                    and self.save_dir is not None
                ):
                    from pathlib import Path
                    checkpoint_path = Path(self.save_dir) / f"checkpoint_iter_{iteration}.pt"
                    self.save(str(checkpoint_path))
                    print(f"  [Checkpoint saved: {checkpoint_path.name}]")

                iter_time = time.time() - iter_start
                print(f"  Iteration time: {iter_time:.1f}s")

        except KeyboardInterrupt:
            print("\n\n[INTERRUPTED] Training stopped by user")
            if self.save_dir is not None:
                from pathlib import Path
                interrupt_checkpoint = Path(self.save_dir) / f"checkpoint_interrupted_iter_{self.iteration}.pt"
                self.save(str(interrupt_checkpoint))
                print(f"[CHECKPOINT SAVED] Progress saved to: {interrupt_checkpoint.name}")
            print(f"Completed {self.iteration}/{num_iterations} iterations")
            # Re-raise to allow outer handler
            raise

        assert self.start_time is not None  # Set at beginning of train()
        total_time = time.time() - self.start_time
        print(f"\nTraining complete in {total_time:.1f}s")

        # Final evaluation
        final_win_rate = self.evaluate_vs_greedy(100)
        print(f"Final win rate vs Greedy: {final_win_rate:.1%}")

        return {
            "iterations": num_iterations,
            "total_games": self.total_games,
            "final_loss": np.mean(all_losses[-100:]) if all_losses else 0,
            "final_win_rate": final_win_rate,
        }

    def evaluate_vs_greedy(self, num_games: int = 100) -> float:
        """
        Evaluate network against GreedyBot.

        Uses MCTS for action selection (same as training).

        Args:
            num_games: Number of evaluation games

        Returns:
            Win rate (0.0 to 1.0)
        """
        self.network.eval()
        wins = 0

        for game_idx in range(num_games):
            env = EssenceWarsEnv(opponent="greedy")
            obs, info = env.reset(seed=game_idx + 50000)
            done = False

            while not done:
                # Create PyGame for MCTS (a bit awkward but maintains interface)
                # For evaluation, use deterministic action selection
                if self.obs_normalizer is not None:
                    obs_norm = self.obs_normalizer.normalize(obs).astype(np.float32)
                else:
                    obs_norm = obs

                obs_t = torch.tensor(obs_norm, dtype=torch.float32, device=self.device).unsqueeze(0)
                mask_t = torch.tensor(info["action_mask"] > 0, dtype=torch.bool, device=self.device).unsqueeze(0)

                with torch.no_grad():
                    policy, _ = self.network.evaluate(obs_t, mask_t)

                # Select best action
                action = int(policy.argmax(dim=-1).item())

                obs, reward, terminated, truncated, info = env.step(action)
                done = terminated or truncated

                if done and float(reward) > 0:
                    wins += 1

        return wins / num_games

    def evaluate_vs_random(self, num_games: int = 100) -> float:
        """Evaluate network against RandomBot."""
        self.network.eval()
        wins = 0

        for game_idx in range(num_games):
            env = EssenceWarsEnv(opponent="random")
            obs, info = env.reset(seed=game_idx + 60000)
            done = False

            while not done:
                if self.obs_normalizer is not None:
                    obs_norm = self.obs_normalizer.normalize(obs).astype(np.float32)
                else:
                    obs_norm = obs

                obs_t = torch.tensor(obs_norm, dtype=torch.float32, device=self.device).unsqueeze(0)
                mask_t = torch.tensor(info["action_mask"] > 0, dtype=torch.bool, device=self.device).unsqueeze(0)

                with torch.no_grad():
                    policy, _ = self.network.evaluate(obs_t, mask_t)

                action = int(policy.argmax(dim=-1).item())

                obs, reward, terminated, truncated, info = env.step(action)
                done = terminated or truncated

                if done and float(reward) > 0:
                    wins += 1

        return wins / num_games

    def evaluate_with_mcts(self, num_games: int = 50) -> float:
        """
        Evaluate using full MCTS search (slower but stronger).

        Args:
            num_games: Number of evaluation games

        Returns:
            Win rate (0.0 to 1.0)
        """
        wins = 0

        for game_idx in range(num_games):
            game = PyGame()
            game.reset(seed=game_idx + 70000)

            # Create a greedy opponent by forking
            while not game.is_done():
                if game.current_player() == 0:
                    # Our turn: use MCTS
                    action_probs = self.mcts.search(game, add_noise=False)
                    action = action_probs.argmax()
                else:
                    # Opponent's turn: use greedy action
                    action = game.greedy_action()

                game.step(action)

            if game.get_reward(0) > 0:
                wins += 1

        return wins / num_games

    def save(self, path: str) -> None:
        """Save model checkpoint."""
        checkpoint = {
            "network_state_dict": self.network.state_dict(),
            "optimizer_state_dict": self.optimizer.state_dict(),
            "iteration": self.iteration,
            "total_games": self.total_games,
            "config": self.config,
        }
        # Save normalizer state if enabled
        if self.obs_normalizer is not None:
            checkpoint["obs_normalizer"] = {
                "mean": self.obs_normalizer.mean,
                "var": self.obs_normalizer.var,
                "count": self.obs_normalizer.count,
            }
        torch.save(checkpoint, path)
        print(f"Saved checkpoint to {path}")

    def load(self, path: str) -> None:
        """Load model checkpoint."""
        checkpoint = torch.load(path, map_location=self.device, weights_only=False)
        self.network.load_state_dict(checkpoint["network_state_dict"])
        self.optimizer.load_state_dict(checkpoint["optimizer_state_dict"])
        self.iteration = checkpoint.get("iteration", 0)
        self.total_games = checkpoint.get("total_games", 0)
        # Restore normalizer state if present
        if "obs_normalizer" in checkpoint and self.obs_normalizer is not None:
            self.obs_normalizer.mean = checkpoint["obs_normalizer"]["mean"]
            self.obs_normalizer.var = checkpoint["obs_normalizer"]["var"]
            self.obs_normalizer.count = checkpoint["obs_normalizer"]["count"]
        print(f"Loaded checkpoint from {path}")

    def get_network(self) -> AlphaZeroNetwork:
        """Get the trained network."""
        return self.network
