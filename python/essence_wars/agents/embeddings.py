"""
Learned Card Embeddings for Essence Wars.

This module provides infrastructure for replacing raw card IDs in the
observation tensor with learned embedding vectors. Two approaches are supported:

1. End-to-End: Learn embeddings jointly with the policy network
2. Pre-trained: Load pre-trained card embeddings (Card2Vec) and fine-tune

The key insight is that raw card IDs (e.g., 1000, 1001) are numerically close
but may represent very different cards. Embeddings allow the network to learn
semantic relationships between cards.

Usage:
    # End-to-end embeddings
    network = EmbeddedPPONetwork(embed_dim=64)

    # Pre-trained embeddings
    pretrained = torch.load("card2vec.pt")
    network = EmbeddedPPONetwork(pretrained_embeds=pretrained)

See docs/embedding-design.md for full design documentation.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

import torch
import torch.nn as nn
from torch.distributions import Categorical

# =============================================================================
# Tensor Structure Constants
# =============================================================================

# State tensor size (from Rust config)
STATE_TENSOR_SIZE = 326

# Global state section
GLOBAL_START = 0
GLOBAL_SIZE = 6

# Player state structure (75 floats each)
PLAYER_STATE_SIZE = 75

# Player 1 section
P1_START = 6
P1_BASE_STATS_SIZE = 5
P1_HAND_START = 11  # P1_START + P1_BASE_STATS_SIZE
P1_HAND_SIZE = 10
P1_CREATURES_START = 21  # P1_HAND_START + P1_HAND_SIZE
P1_CREATURES_SIZE = 50  # 5 slots × 10 floats
P1_SUPPORTS_START = 71  # P1_CREATURES_START + P1_CREATURES_SIZE
P1_SUPPORTS_SIZE = 10  # 2 slots × 5 floats

# Player 2 section
P2_START = 81  # P1_START + PLAYER_STATE_SIZE
P2_HAND_START = 86  # P2_START + P1_BASE_STATS_SIZE
P2_CREATURES_START = 96
P2_SUPPORTS_START = 146

# Card embedding section (trailing)
EMBED_SECTION_START = 156
EMBED_SECTION_SIZE = 170  # 326 - 156

# Slot sizes
CREATURE_SLOT_SIZE = 10
SUPPORT_SLOT_SIZE = 5

# Card ID positions within slots
# Support slots: [occupied, durability, card_id, reserved, reserved]
SUPPORT_CARD_ID_OFFSET = 2

# Maximum card ID (card IDs range from 1000-4074, but we use 5000 for buffer)
MAX_CARD_ID = 5000


# =============================================================================
# Card ID Position Mapping
# =============================================================================

@dataclass
class CardIdPositions:
    """Positions of card IDs in the 326-float state tensor."""

    # Hand card IDs (10 per player)
    p1_hand: list[int]
    p2_hand: list[int]

    # Support card IDs (2 per player, within 5-float slots)
    p1_supports: list[int]
    p2_supports: list[int]

    # Card embedding section (variable length)
    embed_section: list[int]

    @classmethod
    def default(cls) -> CardIdPositions:
        """Create default card ID positions based on tensor structure."""
        return cls(
            p1_hand=list(range(P1_HAND_START, P1_HAND_START + P1_HAND_SIZE)),
            p2_hand=list(range(P2_HAND_START, P2_HAND_START + P1_HAND_SIZE)),
            p1_supports=[
                P1_SUPPORTS_START + SUPPORT_CARD_ID_OFFSET,  # Support slot 0
                P1_SUPPORTS_START + SUPPORT_SLOT_SIZE + SUPPORT_CARD_ID_OFFSET,  # Support slot 1
            ],
            p2_supports=[
                P2_SUPPORTS_START + SUPPORT_CARD_ID_OFFSET,
                P2_SUPPORTS_START + SUPPORT_SLOT_SIZE + SUPPORT_CARD_ID_OFFSET,
            ],
            embed_section=list(range(EMBED_SECTION_START, STATE_TENSOR_SIZE)),
        )

    def all_positions(self) -> list[int]:
        """Get all card ID positions as a flat list."""
        return (
            self.p1_hand +
            self.p2_hand +
            self.p1_supports +
            self.p2_supports +
            self.embed_section
        )

    def non_embed_section_positions(self) -> list[int]:
        """Get card ID positions excluding the trailing embed section."""
        return (
            self.p1_hand +
            self.p2_hand +
            self.p1_supports +
            self.p2_supports
        )


def get_non_card_positions(card_positions: CardIdPositions) -> list[int]:
    """Get all tensor positions that are NOT card IDs."""
    all_card_positions = set(card_positions.all_positions())
    return [i for i in range(STATE_TENSOR_SIZE) if i not in all_card_positions]


# =============================================================================
# Observation Transformer
# =============================================================================

class ObservationTransformer(nn.Module):
    """
    Transform flat observations by replacing card IDs with learned embeddings.

    This module extracts card IDs from specific positions in the state tensor,
    looks up their embeddings, and concatenates them with the non-card features.

    Args:
        card_embedding: The embedding layer to use for card IDs
        include_embed_section: Whether to embed the trailing card ID section

    Example:
        card_embed = nn.Embedding(5000, 64)
        transformer = ObservationTransformer(card_embed)

        obs = torch.randn(32, 326)  # Batch of flat observations
        embedded = transformer(obs)  # Shape: (32, new_dim)
    """

    def __init__(
        self,
        card_embedding: nn.Embedding,
        include_embed_section: bool = True,
    ):
        super().__init__()

        self.card_embedding = card_embedding
        self.include_embed_section = include_embed_section
        self.embed_dim = card_embedding.embedding_dim

        # Get card ID positions
        self.positions = CardIdPositions.default()

        if include_embed_section:
            self.card_id_positions = self.positions.all_positions()
        else:
            self.card_id_positions = self.positions.non_embed_section_positions()

        # Get non-card positions (excludes ALL card ID positions including embed section)
        self.non_card_positions = get_non_card_positions(self.positions)
        # NOTE: When include_embed_section=False, we do NOT add embed section back
        # as raw features. Those positions contain card IDs (1000-4074) which would
        # dominate the input and break learning. Instead, we only embed the 24 card
        # IDs from hands and supports, and exclude the embed section entirely.

        # Pre-compute dimensions
        self.num_card_slots = len(self.card_id_positions)
        self.num_non_card_features = len(self.non_card_positions)
        self.output_dim = self.num_non_card_features + self.num_card_slots * self.embed_dim

        # Register positions as buffers for device handling
        self.register_buffer(
            'card_id_indices',
            torch.tensor(self.card_id_positions, dtype=torch.long)
        )
        self.register_buffer(
            'non_card_indices',
            torch.tensor(self.non_card_positions, dtype=torch.long)
        )

    def forward(self, obs: torch.Tensor) -> torch.Tensor:
        """
        Transform flat observation to embedded observation.

        Args:
            obs: Flat observation tensor of shape (batch, 326) or (326,)

        Returns:
            Embedded observation of shape (batch, output_dim) or (output_dim,)
        """
        squeeze = obs.dim() == 1
        if squeeze:
            obs = obs.unsqueeze(0)

        batch_size = obs.shape[0]

        # Extract non-card features
        non_card_features = obs[:, self.non_card_indices]  # (batch, num_non_card)

        # Extract card IDs and convert to long indices
        card_ids = obs[:, self.card_id_indices].long()  # (batch, num_cards)

        # Clamp card IDs to valid range (0 = padding, 1000-4074 = valid cards)
        card_ids = card_ids.clamp(0, MAX_CARD_ID - 1)

        # Look up embeddings
        card_embeds = self.card_embedding(card_ids)  # (batch, num_cards, embed_dim)

        # Flatten card embeddings
        card_embeds_flat = card_embeds.view(batch_size, -1)  # (batch, num_cards * embed_dim)

        # Concatenate
        embedded_obs = torch.cat([non_card_features, card_embeds_flat], dim=1)

        if squeeze:
            embedded_obs = embedded_obs.squeeze(0)

        return embedded_obs


# =============================================================================
# Embedded Network Architectures
# =============================================================================

class ResidualBlock(nn.Module):
    """Residual block with two linear layers and skip connection."""

    def __init__(self, hidden_dim: int):
        super().__init__()
        self.layers = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return torch.relu(x + self.layers(x))


class EmbeddedPPONetwork(nn.Module):
    """
    PPO network with learned card embeddings.

    This network replaces raw card IDs in the observation with learned
    embedding vectors before processing through the policy-value network.

    Args:
        embed_dim: Dimension of card embeddings (default: 64)
        hidden_dim: Hidden layer dimension (default: 256)
        num_cards: Number of cards in embedding table (default: 5000)
        include_embed_section: Whether to embed the trailing card section
        pretrained_embeds: Optional pre-trained embedding weights
        freeze_embeds: Whether to freeze embeddings during training

    Example:
        # End-to-end learning
        network = EmbeddedPPONetwork(embed_dim=64)

        # Pre-trained embeddings
        pretrained = torch.load("card2vec_embeds.pt")
        network = EmbeddedPPONetwork(pretrained_embeds=pretrained)
    """

    def __init__(
        self,
        embed_dim: int = 64,
        hidden_dim: int = 256,
        num_cards: int = MAX_CARD_ID,
        include_embed_section: bool = False,  # Start with simpler version
        pretrained_embeds: torch.Tensor | None = None,
        freeze_embeds: bool = False,
    ):
        super().__init__()

        self.embed_dim = embed_dim
        self.hidden_dim = hidden_dim
        self.num_cards = num_cards
        self.include_embed_section = include_embed_section
        self._using_pretrained_embeds = pretrained_embeds is not None

        # Create or load card embedding layer
        if pretrained_embeds is not None:
            self.card_embedding = nn.Embedding.from_pretrained(
                pretrained_embeds,
                freeze=freeze_embeds,
                padding_idx=0,
            )
            self.embed_dim = pretrained_embeds.shape[1]
        else:
            self.card_embedding = nn.Embedding(
                num_cards,
                embed_dim,
                padding_idx=0,
            )

        # Create observation transformer
        self.obs_transformer = ObservationTransformer(
            self.card_embedding,
            include_embed_section=include_embed_section,
        )

        # Calculate input dimension after transformation
        input_dim = self.obs_transformer.output_dim

        # Shared trunk
        self.trunk = nn.Sequential(
            nn.Linear(input_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
        )

        # Policy head
        self.policy_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 256),  # ACTION_SPACE_SIZE
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

        # Smaller initialization for policy output
        nn.init.orthogonal_(self.policy_head[-1].weight, gain=0.01)

        # Initialize embeddings with small values (only if not using pretrained)
        if not self._using_pretrained_embeds:
            nn.init.normal_(self.card_embedding.weight, mean=0.0, std=0.1)
            # Zero out padding embedding
            self.card_embedding.weight.data[0].zero_()

    def forward(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor | None = None,
    ) -> tuple[torch.Tensor, torch.Tensor]:
        """
        Forward pass through the network.

        Args:
            obs: Flat observations of shape (batch, 326) or (326,)
            action_mask: Boolean mask of shape (batch, 256) where True = legal

        Returns:
            logits: Action logits of shape (batch, 256)
            value: State values of shape (batch,)
        """
        # Transform observation (replace card IDs with embeddings)
        embedded_obs = self.obs_transformer(obs)

        # Process through trunk
        features = self.trunk(embedded_obs)

        # Get outputs
        logits = self.policy_head(features)
        value = self.value_head(features).squeeze(-1)

        # Apply action mask
        if action_mask is not None:
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
            obs: Observations of shape (batch, 326)
            action_mask: Boolean mask of shape (batch, 256)
            deterministic: If True, return argmax action

        Returns:
            action: Sampled actions of shape (batch,)
            log_prob: Log probabilities of shape (batch,)
            entropy: Policy entropy of shape (batch,)
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
        """Get value estimate for observations."""
        embedded_obs = self.obs_transformer(obs)
        features = self.trunk(embedded_obs)
        return self.value_head(features).squeeze(-1)

    def evaluate_actions(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor,
        actions: torch.Tensor,
    ) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
        """
        Evaluate log probability and entropy of given actions.

        Used during PPO update to compute policy loss.
        """
        logits, value = self.forward(obs, action_mask)
        dist = Categorical(logits=logits)

        log_prob = dist.log_prob(actions)
        entropy = dist.entropy()

        return log_prob, entropy, value


class EmbeddedAlphaZeroNetwork(nn.Module):
    """
    AlphaZero-style network with learned card embeddings.

    Uses residual blocks for deeper feature extraction and tanh activation
    on the value head for output in [-1, 1].

    Args:
        embed_dim: Dimension of card embeddings (default: 64)
        hidden_dim: Hidden layer dimension (default: 256)
        num_blocks: Number of residual blocks (default: 4)
        num_cards: Number of cards in embedding table (default: 5000)
        include_embed_section: Whether to embed the trailing card section
        pretrained_embeds: Optional pre-trained embedding weights
        freeze_embeds: Whether to freeze embeddings during training
    """

    def __init__(
        self,
        embed_dim: int = 64,
        hidden_dim: int = 256,
        num_blocks: int = 4,
        num_cards: int = MAX_CARD_ID,
        include_embed_section: bool = False,
        pretrained_embeds: torch.Tensor | None = None,
        freeze_embeds: bool = False,
    ):
        super().__init__()

        self.embed_dim = embed_dim
        self.hidden_dim = hidden_dim
        self.num_blocks = num_blocks
        self._using_pretrained_embeds = pretrained_embeds is not None

        # Create or load card embedding layer
        if pretrained_embeds is not None:
            self.card_embedding = nn.Embedding.from_pretrained(
                pretrained_embeds,
                freeze=freeze_embeds,
                padding_idx=0,
            )
            self.embed_dim = pretrained_embeds.shape[1]
        else:
            self.card_embedding = nn.Embedding(
                num_cards,
                embed_dim,
                padding_idx=0,
            )

        # Create observation transformer
        self.obs_transformer = ObservationTransformer(
            self.card_embedding,
            include_embed_section=include_embed_section,
        )

        # Calculate input dimension
        input_dim = self.obs_transformer.output_dim

        # Input projection
        self.input_proj = nn.Sequential(
            nn.Linear(input_dim, hidden_dim),
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
            nn.Linear(hidden_dim, 256),
        )

        # Value head with tanh for [-1, 1] output
        self.value_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
            nn.Tanh(),
        )

        self._init_weights()

    def _init_weights(self) -> None:
        """Initialize network weights."""
        for module in self.modules():
            if isinstance(module, nn.Linear):
                nn.init.orthogonal_(module.weight, gain=1.0)
                nn.init.zeros_(module.bias)

        nn.init.orthogonal_(self.policy_head[-1].weight, gain=0.01)

        # Initialize embeddings with small values (only if not using pretrained)
        if not self._using_pretrained_embeds:
            nn.init.normal_(self.card_embedding.weight, mean=0.0, std=0.1)
            self.card_embedding.weight.data[0].zero_()

    def forward(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor | None = None,
    ) -> tuple[torch.Tensor, torch.Tensor]:
        """
        Forward pass through the network.

        Returns:
            logits: Action logits of shape (batch, 256)
            value: State values of shape (batch,) in [-1, 1]
        """
        # Transform observation
        embedded_obs = self.obs_transformer(obs)

        # Input projection
        x = self.input_proj(embedded_obs)

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
        embedded_obs = self.obs_transformer(obs)
        x = self.input_proj(embedded_obs)
        for block in self.residual_tower:
            x = block(x)
        return self.value_head(x).squeeze(-1)

    def evaluate(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor,
    ) -> tuple[torch.Tensor, torch.Tensor]:
        """Evaluate position for MCTS (returns policy probs and value)."""
        logits, value = self.forward(obs, action_mask)
        policy = torch.softmax(logits, dim=-1)
        return policy, value


# =============================================================================
# Factory Functions
# =============================================================================

ObservationMode = Literal["flat", "embedded", "embedded_pretrained"]


def create_network(
    observation_mode: ObservationMode = "flat",
    network_type: Literal["ppo", "alphazero"] = "ppo",
    embed_dim: int = 64,
    hidden_dim: int = 256,
    num_blocks: int = 4,
    pretrained_path: str | None = None,
    freeze_embeds: bool = False,
    include_embed_section: bool = False,
) -> nn.Module:
    """
    Factory function to create networks with different observation modes.

    Args:
        observation_mode: "flat" for original, "embedded" for end-to-end,
                         "embedded_pretrained" for pre-trained embeddings
        network_type: "ppo" or "alphazero"
        embed_dim: Embedding dimension (ignored if pretrained)
        hidden_dim: Hidden layer dimension
        num_blocks: Number of residual blocks (alphazero only)
        pretrained_path: Path to pre-trained embeddings (for embedded_pretrained)
        freeze_embeds: Whether to freeze embeddings
        include_embed_section: Whether to embed the trailing card section

    Returns:
        Configured network module
    """
    # Load pre-trained embeddings if specified
    pretrained_embeds = None
    if observation_mode == "embedded_pretrained" and pretrained_path:
        pretrained_embeds = torch.load(pretrained_path)
        if isinstance(pretrained_embeds, dict):
            pretrained_embeds = pretrained_embeds.get("weight", pretrained_embeds.get("embeddings"))

    if observation_mode == "flat":
        # Use original networks
        if network_type == "ppo":
            from essence_wars.agents.networks import EssenceWarsNetwork
            return EssenceWarsNetwork(hidden_dim=hidden_dim)
        else:
            from essence_wars.agents.networks import AlphaZeroNetwork
            return AlphaZeroNetwork(hidden_dim=hidden_dim, num_blocks=num_blocks)

    # Embedded modes
    if network_type == "ppo":
        return EmbeddedPPONetwork(
            embed_dim=embed_dim,
            hidden_dim=hidden_dim,
            pretrained_embeds=pretrained_embeds,
            freeze_embeds=freeze_embeds,
            include_embed_section=include_embed_section,
        )
    else:
        return EmbeddedAlphaZeroNetwork(
            embed_dim=embed_dim,
            hidden_dim=hidden_dim,
            num_blocks=num_blocks,
            pretrained_embeds=pretrained_embeds,
            freeze_embeds=freeze_embeds,
            include_embed_section=include_embed_section,
        )


def get_embedding_info() -> dict:
    """Get information about the embedding configuration."""
    positions = CardIdPositions.default()
    non_card = get_non_card_positions(positions)

    return {
        "state_tensor_size": STATE_TENSOR_SIZE,
        "num_card_positions": len(positions.all_positions()),
        "num_non_card_positions": len(non_card),
        "card_positions": {
            "p1_hand": positions.p1_hand,
            "p2_hand": positions.p2_hand,
            "p1_supports": positions.p1_supports,
            "p2_supports": positions.p2_supports,
            "embed_section_size": len(positions.embed_section),
        },
        "max_card_id": MAX_CARD_ID,
    }
