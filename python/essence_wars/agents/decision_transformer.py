"""Decision Transformer for Essence Wars.

Implements the Decision Transformer architecture from:
"Decision Transformer: Reinforcement Learning via Sequence Modeling"
(Chen et al., 2021)

The key idea is to treat RL as sequence modeling:
- Input: (return-to-go, state, action) sequences
- Output: Predict next action conditioned on desired return
- Training: Standard supervised learning (cross-entropy)
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Optional

import torch
import torch.nn as nn
import torch.nn.functional as F


@dataclass
class DecisionTransformerConfig:
    """Configuration for Decision Transformer."""

    # State/action dimensions
    state_dim: int = 326
    action_dim: int = 256

    # Transformer architecture
    d_model: int = 128  # Embedding dimension
    n_heads: int = 4  # Number of attention heads
    n_layers: int = 4  # Number of transformer layers
    d_ff: int = 512  # Feedforward dimension
    dropout: float = 0.1

    # Sequence parameters
    max_length: int = 100  # Maximum game length
    context_length: int = 20  # Context window for attention

    # Training
    max_timestep: int = 100  # Maximum timestep embedding


class CausalSelfAttention(nn.Module):
    """Causal (masked) self-attention layer."""

    def __init__(self, config: DecisionTransformerConfig):
        super().__init__()
        assert config.d_model % config.n_heads == 0

        self.n_heads = config.n_heads
        self.d_model = config.d_model
        self.head_dim = config.d_model // config.n_heads

        # Key, query, value projections
        self.qkv = nn.Linear(config.d_model, 3 * config.d_model)
        self.proj = nn.Linear(config.d_model, config.d_model)

        self.attn_dropout = nn.Dropout(config.dropout)
        self.proj_dropout = nn.Dropout(config.dropout)

        # Causal mask - each position can only attend to previous positions
        # For DT, we have 3 tokens per timestep: (return, state, action)
        # We allow: return -> nothing, state -> return, action -> return + state
        max_seq_len = config.context_length * 3
        self.register_buffer(
            "causal_mask",
            torch.tril(torch.ones(max_seq_len, max_seq_len)).view(
                1, 1, max_seq_len, max_seq_len
            ),
        )

    def forward(
        self, x: torch.Tensor, attention_mask: Optional[torch.Tensor] = None
    ) -> torch.Tensor:
        B, T, C = x.shape

        # Compute Q, K, V
        qkv = self.qkv(x)
        q, k, v = qkv.chunk(3, dim=-1)

        # Reshape for multi-head attention
        q = q.view(B, T, self.n_heads, self.head_dim).transpose(1, 2)
        k = k.view(B, T, self.n_heads, self.head_dim).transpose(1, 2)
        v = v.view(B, T, self.n_heads, self.head_dim).transpose(1, 2)

        # Attention scores
        scale = 1.0 / math.sqrt(self.head_dim)
        attn = (q @ k.transpose(-2, -1)) * scale

        # Apply causal mask
        attn = attn.masked_fill(self.causal_mask[:, :, :T, :T] == 0, float("-inf"))

        # Apply padding mask if provided
        if attention_mask is not None:
            # attention_mask: (B, T) -> (B, 1, 1, T)
            attn = attn.masked_fill(
                attention_mask.unsqueeze(1).unsqueeze(2) == 0, float("-inf")
            )

        attn = F.softmax(attn, dim=-1)
        attn = self.attn_dropout(attn)

        # Apply attention to values
        out = attn @ v
        out = out.transpose(1, 2).contiguous().view(B, T, C)
        out = self.proj_dropout(self.proj(out))

        return out


class TransformerBlock(nn.Module):
    """Transformer block with pre-norm architecture."""

    def __init__(self, config: DecisionTransformerConfig):
        super().__init__()
        self.ln1 = nn.LayerNorm(config.d_model)
        self.attn = CausalSelfAttention(config)
        self.ln2 = nn.LayerNorm(config.d_model)
        self.mlp = nn.Sequential(
            nn.Linear(config.d_model, config.d_ff),
            nn.GELU(),
            nn.Dropout(config.dropout),
            nn.Linear(config.d_ff, config.d_model),
            nn.Dropout(config.dropout),
        )

    def forward(
        self, x: torch.Tensor, attention_mask: Optional[torch.Tensor] = None
    ) -> torch.Tensor:
        x = x + self.attn(self.ln1(x), attention_mask)
        x = x + self.mlp(self.ln2(x))
        return x


class DecisionTransformer(nn.Module):
    """Decision Transformer for game playing.

    Architecture:
    - Embeds (return-to-go, state, action) triplets
    - Processes with causal transformer
    - Predicts action at each timestep

    At inference:
    - Condition on desired return (e.g., +1 for winning)
    - Predict action that historically led to that return
    """

    def __init__(self, config: DecisionTransformerConfig):
        super().__init__()
        self.config = config

        # Embeddings for each modality
        self.return_embed = nn.Sequential(
            nn.Linear(1, config.d_model),
            nn.Tanh(),
        )
        self.state_embed = nn.Sequential(
            nn.Linear(config.state_dim, config.d_model),
            nn.Tanh(),
        )
        self.action_embed = nn.Embedding(config.action_dim, config.d_model)

        # Timestep embedding (learnable positional encoding)
        self.timestep_embed = nn.Embedding(config.max_timestep, config.d_model)

        # Learned position embedding for (return, state, action) within timestep
        self.position_embed = nn.Embedding(3, config.d_model)

        # Dropout after embedding
        self.embed_dropout = nn.Dropout(config.dropout)

        # Transformer blocks
        self.blocks = nn.ModuleList(
            [TransformerBlock(config) for _ in range(config.n_layers)]
        )

        # Final layer norm
        self.ln_f = nn.LayerNorm(config.d_model)

        # Action prediction head
        self.action_head = nn.Linear(config.d_model, config.action_dim)

        # Initialize weights
        self.apply(self._init_weights)

        # Count parameters
        n_params = sum(p.numel() for p in self.parameters())
        print(f"DecisionTransformer: {n_params:,} parameters")

    def _init_weights(self, module: nn.Module) -> None:
        if isinstance(module, nn.Linear):
            nn.init.normal_(module.weight, mean=0.0, std=0.02)
            if module.bias is not None:
                nn.init.zeros_(module.bias)
        elif isinstance(module, nn.Embedding):
            nn.init.normal_(module.weight, mean=0.0, std=0.02)
        elif isinstance(module, nn.LayerNorm):
            nn.init.ones_(module.weight)
            nn.init.zeros_(module.bias)

    def forward(
        self,
        returns_to_go: torch.Tensor,  # (B, T)
        states: torch.Tensor,  # (B, T, state_dim)
        actions: torch.Tensor,  # (B, T)
        timesteps: torch.Tensor,  # (B, T)
        attention_mask: Optional[torch.Tensor] = None,  # (B, T)
    ) -> torch.Tensor:
        """Forward pass.

        Args:
            returns_to_go: Target returns at each timestep (B, T)
            states: Game states (B, T, state_dim)
            actions: Actions taken (B, T)
            timesteps: Timestep indices (B, T)
            attention_mask: Mask for padding (B, T)

        Returns:
            action_logits: Predicted action logits (B, T, action_dim)
        """
        B, T = states.shape[:2]

        # Embed each modality
        return_embeddings = self.return_embed(returns_to_go.unsqueeze(-1))  # (B, T, d)
        state_embeddings = self.state_embed(states)  # (B, T, d)
        action_embeddings = self.action_embed(actions)  # (B, T, d)

        # Add timestep embeddings
        time_embeddings = self.timestep_embed(timesteps)  # (B, T, d)
        return_embeddings = return_embeddings + time_embeddings
        state_embeddings = state_embeddings + time_embeddings
        action_embeddings = action_embeddings + time_embeddings

        # Add position embeddings (0=return, 1=state, 2=action)
        pos_ids = torch.arange(3, device=states.device)
        pos_embeddings = self.position_embed(pos_ids)  # (3, d)

        return_embeddings = return_embeddings + pos_embeddings[0]
        state_embeddings = state_embeddings + pos_embeddings[1]
        action_embeddings = action_embeddings + pos_embeddings[2]

        # Interleave: (return_1, state_1, action_1, return_2, state_2, action_2, ...)
        # Shape: (B, T, 3, d) -> (B, T*3, d)
        stacked = torch.stack(
            [return_embeddings, state_embeddings, action_embeddings], dim=2
        )
        h = stacked.view(B, T * 3, -1)

        # Apply dropout
        h = self.embed_dropout(h)

        # Expand attention mask to cover all 3 tokens per timestep
        if attention_mask is not None:
            # (B, T) -> (B, T*3)
            attention_mask = (
                attention_mask.unsqueeze(-1).repeat(1, 1, 3).view(B, T * 3)
            )

        # Transformer blocks
        for block in self.blocks:
            h = block(h, attention_mask)

        # Final layer norm
        h = self.ln_f(h)

        # Reshape back: (B, T*3, d) -> (B, T, 3, d)
        h = h.view(B, T, 3, -1)

        # Predict actions from state embeddings (index 1)
        # The action at position t is predicted from the state at position t
        state_h = h[:, :, 1, :]  # (B, T, d)
        action_logits = self.action_head(state_h)  # (B, T, action_dim)

        return action_logits

    def get_action(
        self,
        states: torch.Tensor,  # (T, state_dim) or (1, T, state_dim)
        actions: torch.Tensor,  # (T,) or (1, T)
        returns_to_go: torch.Tensor,  # (T,) or (1, T)
        timesteps: torch.Tensor,  # (T,) or (1, T)
        action_mask: Optional[torch.Tensor] = None,  # (action_dim,)
    ) -> int:
        """Get action for inference.

        Uses the last K timesteps as context.
        """
        # Ensure batch dimension
        if states.dim() == 2:
            states = states.unsqueeze(0)
            actions = actions.unsqueeze(0)
            returns_to_go = returns_to_go.unsqueeze(0)
            timesteps = timesteps.unsqueeze(0)

        # Truncate to context length
        K = self.config.context_length
        states = states[:, -K:]
        actions = actions[:, -K:]
        returns_to_go = returns_to_go[:, -K:]
        timesteps = timesteps[:, -K:]

        # Forward pass
        action_logits = self.forward(returns_to_go, states, actions, timesteps)

        # Get logits for last timestep
        logits = action_logits[0, -1]  # (action_dim,)

        # Apply action mask
        if action_mask is not None:
            logits[~action_mask] = float("-inf")

        # Greedy selection
        action = logits.argmax().item()

        return action


class DecisionTransformerAgent:
    """Agent wrapper for Decision Transformer."""

    def __init__(
        self,
        model: DecisionTransformer,
        device: str = "cpu",
        target_return: float = 1.0,
    ):
        self.model = model
        self.device = device
        self.target_return = target_return
        self.model.to(device)
        self.model.eval()

        # History for current game
        self.reset()

    def reset(self) -> None:
        """Reset history for new game."""
        self.states: list[torch.Tensor] = []
        self.actions: list[int] = []
        self.timestep = 0

    def get_action(
        self,
        state: torch.Tensor | list[float],
        action_mask: torch.Tensor | list[bool],
    ) -> int:
        """Get action for current state."""
        # Convert to tensors
        if not isinstance(state, torch.Tensor):
            state = torch.tensor(state, dtype=torch.float32)
        if not isinstance(action_mask, torch.Tensor):
            action_mask = torch.tensor(action_mask, dtype=torch.bool)

        state = state.to(self.device)
        action_mask = action_mask.to(self.device)

        # Add current state to history
        self.states.append(state)

        # Build sequences
        T = len(self.states)
        states = torch.stack(self.states).unsqueeze(0).to(self.device)  # (1, T, 326)

        # For actions, we need T-1 past actions + dummy for current
        if self.actions:
            actions = torch.tensor(
                self.actions + [0], dtype=torch.long, device=self.device
            ).unsqueeze(0)
        else:
            actions = torch.zeros(1, 1, dtype=torch.long, device=self.device)

        # Returns-to-go: all set to target return
        returns_to_go = torch.full(
            (1, T), self.target_return, dtype=torch.float32, device=self.device
        )

        # Timesteps
        timesteps = torch.arange(T, dtype=torch.long, device=self.device).unsqueeze(0)

        # Get action
        with torch.no_grad():
            action = self.model.get_action(
                states, actions, returns_to_go, timesteps, action_mask
            )

        # Record action for next step
        self.actions.append(action)
        self.timestep += 1

        return action

    def name(self) -> str:
        return "DecisionTransformer"
