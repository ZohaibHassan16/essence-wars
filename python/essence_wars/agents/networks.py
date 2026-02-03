"""
Neural Network Architectures for Essence Wars RL Agents.

This module provides shared policy-value networks for PPO and AlphaZero.
All networks support action masking for handling illegal actions.
"""

from __future__ import annotations

from typing import ClassVar, cast

import torch
import torch.nn as nn
from torch.distributions import Categorical


class EssenceWarsNetwork(nn.Module):
    """
    Shared policy-value network for PPO.

    Architecture:
    - Shared trunk: 2 hidden layers with ReLU
    - Policy head: 1 hidden layer -> action logits
    - Value head: 1 hidden layer -> scalar value

    The network supports action masking by setting illegal action logits
    to a large negative value before softmax.

    Args:
        obs_dim: Observation dimension (default: 326)
        action_dim: Number of actions (default: 256)
        hidden_dim: Hidden layer size (default: 256)

    Example:
        network = EssenceWarsNetwork()
        obs = torch.randn(32, 326)  # batch of 32
        mask = torch.ones(32, 256)  # all actions legal

        logits, value = network(obs, mask)
        action, log_prob, entropy = network.get_action(obs, mask)
    """

    def __init__(
        self,
        obs_dim: int = 326,
        action_dim: int = 256,
        hidden_dim: int = 256,
    ) -> None:
        super().__init__()

        self.obs_dim = obs_dim
        self.action_dim = action_dim
        self.hidden_dim = hidden_dim

        # Input normalization (prevents feature explosion from unnormalized inputs)
        self.input_norm = nn.LayerNorm(obs_dim)

        # Shared trunk
        self.trunk = nn.Sequential(
            nn.Linear(obs_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
        )

        # Policy head
        self.policy_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, action_dim),
        )

        # Value head
        self.value_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
        )

        # Initialize weights
        self._init_weights()

    def _init_weights(self) -> None:
        """Initialize network weights using orthogonal initialization."""
        for module in self.modules():
            if isinstance(module, nn.Linear):
                nn.init.orthogonal_(module.weight, gain=1.0)
                nn.init.zeros_(module.bias)

        # Smaller initialization for policy output (more uniform initial policy)
        policy_last = cast("nn.Linear", self.policy_head[-1])
        nn.init.orthogonal_(policy_last.weight, gain=0.01)

        # Smaller initialization for value output
        value_last = cast("nn.Linear", self.value_head[-1])
        nn.init.orthogonal_(value_last.weight, gain=1.0)

    def forward(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor | None = None,
    ) -> tuple[torch.Tensor, torch.Tensor]:
        """
        Forward pass through the network.

        Args:
            obs: Observations of shape (batch_size, obs_dim)
            action_mask: Boolean mask of shape (batch_size, action_dim)
                         where True = legal action

        Returns:
            logits: Action logits of shape (batch_size, action_dim)
            value: State values of shape (batch_size,)
        """
        # Apply input normalization first
        obs = self.input_norm(obs)

        features = self.trunk(obs)

        logits = self.policy_head(features)
        value = self.value_head(features).squeeze(-1)

        # Apply action mask
        if action_mask is not None:
            # Set illegal action logits to large negative value
            logits = logits.masked_fill(~action_mask, -1e8)

        return logits, value

    def get_action(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor,
        deterministic: bool = False,
    ) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
        """
        Sample action from the policy.

        Args:
            obs: Observations of shape (batch_size, obs_dim)
            action_mask: Boolean mask of shape (batch_size, action_dim)
            deterministic: If True, return argmax action

        Returns:
            action: Sampled actions of shape (batch_size,)
            log_prob: Log probabilities of shape (batch_size,)
            entropy: Policy entropy of shape (batch_size,)
        """
        logits, _ = self.forward(obs, action_mask)
        dist = Categorical(logits=logits)

        action = logits.argmax(dim=-1) if deterministic else dist.sample()

        log_prob = dist.log_prob(action)
        entropy = dist.entropy()

        return action, log_prob, entropy

    def get_value(self, obs: torch.Tensor) -> torch.Tensor:
        """
        Get value estimate for observations.

        Args:
            obs: Observations of shape (batch_size, obs_dim)

        Returns:
            value: State values of shape (batch_size,)
        """
        features = self.trunk(obs)
        value = self.value_head(features).squeeze(-1)
        return value

    def evaluate_actions(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor,
        actions: torch.Tensor,
    ) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
        """
        Evaluate log probability and entropy of given actions.

        Used during PPO update to compute policy loss.

        Args:
            obs: Observations of shape (batch_size, obs_dim)
            action_mask: Boolean mask of shape (batch_size, action_dim)
            actions: Actions to evaluate of shape (batch_size,)

        Returns:
            log_prob: Log probabilities of shape (batch_size,)
            entropy: Policy entropy of shape (batch_size,)
            value: State values of shape (batch_size,)
        """
        logits, value = self.forward(obs, action_mask)
        dist = Categorical(logits=logits)

        log_prob = dist.log_prob(actions)
        entropy = dist.entropy()

        return log_prob, entropy, value


class ResidualBlock(nn.Module):
    """
    Residual block with two linear layers and skip connection.

    Architecture: x -> Linear -> ReLU -> Linear -> + x -> ReLU
    """

    def __init__(self, hidden_dim: int) -> None:
        super().__init__()
        self.layers = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return torch.relu(x + self.layers(x))


class AlphaZeroNetwork(nn.Module):
    """
    Policy-value network for AlphaZero-style training.

    Key differences from PPO network:
    - Uses residual blocks for deeper feature extraction
    - Value head uses tanh activation (output in [-1, 1])
    - Designed for MCTS-guided training

    Architecture:
    - Input projection: Linear -> ReLU
    - Residual tower: N residual blocks
    - Policy head: Linear -> ReLU -> Linear (action logits)
    - Value head: Linear -> ReLU -> Linear -> Tanh (scalar in [-1, 1])

    Args:
        obs_dim: Observation dimension (default: 326)
        action_dim: Number of actions (default: 256)
        hidden_dim: Hidden layer size (default: 256)
        num_blocks: Number of residual blocks (default: 4)

    Example:
        network = AlphaZeroNetwork()
        obs = torch.randn(32, 326)  # batch of 32
        mask = torch.ones(32, 256, dtype=torch.bool)

        logits, value = network(obs, mask)
        policy_probs = network.get_policy(obs, mask)
    """

    def __init__(
        self,
        obs_dim: int = 326,
        action_dim: int = 256,
        hidden_dim: int = 256,
        num_blocks: int = 4,
    ) -> None:
        super().__init__()

        self.obs_dim = obs_dim
        self.action_dim = action_dim
        self.hidden_dim = hidden_dim
        self.num_blocks = num_blocks

        # Input normalization (prevents feature explosion from unnormalized inputs)
        self.input_norm = nn.LayerNorm(obs_dim)

        # Input projection
        self.input_proj = nn.Sequential(
            nn.Linear(obs_dim, hidden_dim),
            nn.ReLU(),
        )

        # Residual tower
        self.residual_tower = nn.ModuleList([
            ResidualBlock(hidden_dim) for _ in range(num_blocks)
        ])

        # Policy head
        self.policy_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, action_dim),
        )

        # Value head (with tanh for [-1, 1] output)
        self.value_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
            nn.Tanh(),
        )

        # Initialize weights
        self._init_weights()

    def _init_weights(self) -> None:
        """Initialize network weights."""
        for module in self.modules():
            if isinstance(module, nn.Linear):
                nn.init.orthogonal_(module.weight, gain=1.0)
                nn.init.zeros_(module.bias)

        # Smaller initialization for policy output
        policy_last = cast("nn.Linear", self.policy_head[-1])
        nn.init.orthogonal_(policy_last.weight, gain=0.01)

    def forward(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor | None = None,
    ) -> tuple[torch.Tensor, torch.Tensor]:
        """
        Forward pass through the network.

        Args:
            obs: Observations of shape (batch_size, obs_dim)
            action_mask: Boolean mask of shape (batch_size, action_dim)
                         where True = legal action

        Returns:
            logits: Action logits of shape (batch_size, action_dim)
            value: State values of shape (batch_size,) in [-1, 1]
        """
        # Apply input normalization first (prevents feature explosion)
        obs = self.input_norm(obs)

        # Input projection
        x = self.input_proj(obs)

        # Residual tower
        for block in self.residual_tower:
            x = block(x)

        # Heads
        logits = self.policy_head(x)
        value = self.value_head(x).squeeze(-1)

        # Apply action mask
        if action_mask is not None:
            logits = logits.masked_fill(~action_mask, -1e8)

        return logits, value

    def get_policy(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor,
    ) -> torch.Tensor:
        """
        Get policy probabilities for MCTS.

        Args:
            obs: Observations of shape (batch_size, obs_dim)
            action_mask: Boolean mask of shape (batch_size, action_dim)

        Returns:
            policy: Action probabilities of shape (batch_size, action_dim)
        """
        logits, _ = self.forward(obs, action_mask)
        return torch.softmax(logits, dim=-1)

    def get_value(self, obs: torch.Tensor) -> torch.Tensor:
        """
        Get value estimate for observations.

        Args:
            obs: Observations of shape (batch_size, obs_dim)

        Returns:
            value: State values of shape (batch_size,) in [-1, 1]
        """
        # Apply input normalization first
        obs = self.input_norm(obs)
        x = self.input_proj(obs)
        for block in self.residual_tower:
            x = block(x)
        return self.value_head(x).squeeze(-1)

    def evaluate(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor,
    ) -> tuple[torch.Tensor, torch.Tensor]:
        """
        Evaluate position for MCTS.

        Returns both policy probabilities and value estimate.

        Args:
            obs: Observations of shape (batch_size, obs_dim)
            action_mask: Boolean mask of shape (batch_size, action_dim)

        Returns:
            policy: Action probabilities of shape (batch_size, action_dim)
            value: State values of shape (batch_size,) in [-1, 1]
        """
        logits, value = self.forward(obs, action_mask)
        policy = torch.softmax(logits, dim=-1)
        return policy, value


class CardEmbeddingNetwork(nn.Module):
    """
    Policy-value network with learned card embeddings.

    This network extracts card IDs from specific tensor positions and looks them up
    in a learned embedding table, allowing the network to learn meaningful card
    representations.

    IMPORTANT: This network expects the Rust tensor to output NORMALIZED card IDs
    (card_id / 5000.0). The embedding lookup uses floor(id * 5000) to recover the
    original card ID for embedding lookup.

    Card ID positions in tensor (per player, 0-indexed within player section):
    - Hand cards: indices 5-14 (10 cards)
    - Support card IDs: indices 67 and 72 (2 supports, offset +2 within each slot)

    Architecture:
    - Card embedding: nn.Embedding(num_cards, embed_dim)
    - Input projection: Concat(numeric_features, embedded_cards) -> Linear
    - Same residual tower and heads as AlphaZeroNetwork

    Args:
        obs_dim: Observation dimension (default: 326)
        action_dim: Number of actions (default: 256)
        hidden_dim: Hidden layer size (default: 256)
        num_blocks: Number of residual blocks (default: 4)
        num_cards: Maximum number of cards (default: 5000)
        embed_dim: Card embedding dimension (default: 32)
    """

    # Indices of card IDs within each player's section (relative to player start)
    HAND_CARD_INDICES: ClassVar[list[int]] = list(range(5, 15))  # 10 hand cards
    SUPPORT_CARD_INDICES: ClassVar[list[int]] = [67, 72]  # 2 support card IDs (slot offset + 2)

    def __init__(
        self,
        obs_dim: int = 326,
        action_dim: int = 256,
        hidden_dim: int = 256,
        num_blocks: int = 4,
        num_cards: int = 5000,
        embed_dim: int = 32,
    ) -> None:
        super().__init__()

        self.obs_dim = obs_dim
        self.action_dim = action_dim
        self.hidden_dim = hidden_dim
        self.num_blocks = num_blocks
        self.num_cards = num_cards
        self.embed_dim = embed_dim

        # Card embedding table
        self.card_embedding = nn.Embedding(num_cards, embed_dim, padding_idx=0)

        # Calculate how many card positions we extract per player
        self.cards_per_player = len(self.HAND_CARD_INDICES) + len(self.SUPPORT_CARD_INDICES)
        total_cards = self.cards_per_player * 2  # 2 players

        # Input normalization for non-card features
        # Non-card features = obs_dim - (card positions extracted)
        # Note: We keep normalized card IDs in tensor and ALSO embed them
        self.input_norm = nn.LayerNorm(obs_dim)

        # Adjusted input size: original obs + card embeddings
        input_size = obs_dim + total_cards * embed_dim

        # Input projection
        self.input_proj = nn.Sequential(
            nn.Linear(input_size, hidden_dim),
            nn.ReLU(),
        )

        # Residual tower
        self.residual_tower = nn.ModuleList([
            ResidualBlock(hidden_dim) for _ in range(num_blocks)
        ])

        # Policy head
        self.policy_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, action_dim),
        )

        # Value head (with tanh for [-1, 1] output)
        self.value_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
            nn.Tanh(),
        )

        # Initialize weights
        self._init_weights()

    def _init_weights(self) -> None:
        """Initialize network weights."""
        for module in self.modules():
            if isinstance(module, nn.Linear):
                nn.init.orthogonal_(module.weight, gain=1.0)
                nn.init.zeros_(module.bias)

        # Smaller initialization for policy output
        policy_last = cast("nn.Linear", self.policy_head[-1])
        nn.init.orthogonal_(policy_last.weight, gain=0.01)

        # Small initialization for card embeddings
        nn.init.normal_(self.card_embedding.weight, mean=0.0, std=0.02)
        self.card_embedding.weight.data[0].zero_()  # Zero embedding for padding

    def _extract_card_ids(self, obs: torch.Tensor) -> torch.Tensor:
        """
        Extract card IDs from observation tensor.

        Args:
            obs: Observations of shape (batch_size, obs_dim) with normalized card IDs

        Returns:
            card_ids: Integer card IDs of shape (batch_size, total_cards)
        """
        obs.shape[0]
        card_ids = []

        # Player 1 starts at index 6
        p1_start = 6
        for idx in self.HAND_CARD_INDICES + self.SUPPORT_CARD_INDICES:
            # Denormalize: normalized_id * 5000 -> original card ID
            normalized_id = obs[:, p1_start + idx]
            card_id = (normalized_id * 5000).long().clamp(0, self.num_cards - 1)
            card_ids.append(card_id)

        # Player 2 starts at index 81 (6 + 75)
        p2_start = 81
        for idx in self.HAND_CARD_INDICES + self.SUPPORT_CARD_INDICES:
            normalized_id = obs[:, p2_start + idx]
            card_id = (normalized_id * 5000).long().clamp(0, self.num_cards - 1)
            card_ids.append(card_id)

        return torch.stack(card_ids, dim=1)  # (batch_size, total_cards)

    def forward(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor | None = None,
    ) -> tuple[torch.Tensor, torch.Tensor]:
        """
        Forward pass through the network.

        Args:
            obs: Observations of shape (batch_size, obs_dim)
            action_mask: Boolean mask of shape (batch_size, action_dim)
                         where True = legal action

        Returns:
            logits: Action logits of shape (batch_size, action_dim)
            value: State values of shape (batch_size,) in [-1, 1]
        """
        # Apply input normalization
        obs_normalized = self.input_norm(obs)

        # Extract and embed card IDs
        card_ids = self._extract_card_ids(obs)  # (batch, num_cards)
        card_embeds = self.card_embedding(card_ids)  # (batch, num_cards, embed_dim)
        card_embeds_flat = card_embeds.view(obs.shape[0], -1)  # (batch, num_cards * embed_dim)

        # Concatenate normalized obs with card embeddings
        combined = torch.cat([obs_normalized, card_embeds_flat], dim=-1)

        # Input projection
        x = self.input_proj(combined)

        # Residual tower
        for block in self.residual_tower:
            x = block(x)

        # Heads
        logits = self.policy_head(x)
        value = self.value_head(x).squeeze(-1)

        # Apply action mask
        if action_mask is not None:
            logits = logits.masked_fill(~action_mask, -1e8)

        return logits, value

    def get_policy(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor,
    ) -> torch.Tensor:
        """Get policy probabilities for MCTS."""
        logits, _ = self.forward(obs, action_mask)
        return torch.softmax(logits, dim=-1)

    def get_value(self, obs: torch.Tensor) -> torch.Tensor:
        """Get value estimate for observations."""
        _logits, value = self.forward(obs, None)
        return value

    def evaluate(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor,
    ) -> tuple[torch.Tensor, torch.Tensor]:
        """Evaluate position for MCTS."""
        logits, value = self.forward(obs, action_mask)
        policy = torch.softmax(logits, dim=-1)
        return policy, value
