"""
Neural Network Architectures for Essence Wars RL Agents.

This module provides shared policy-value networks for PPO and AlphaZero.
All networks support action masking for handling illegal actions.
"""

from __future__ import annotations

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
        nn.init.orthogonal_(self.policy_head[-1].weight, gain=0.01)

        # Smaller initialization for value output
        nn.init.orthogonal_(self.value_head[-1].weight, gain=1.0)

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

        if deterministic:
            action = logits.argmax(dim=-1)
        else:
            action = dist.sample()

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
