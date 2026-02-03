"""Tests for learned card embeddings module."""

import numpy as np
import pytest
import torch


class TestCardIdPositions:
    """Tests for CardIdPositions class."""

    def test_default_positions(self):
        """Test default card ID positions are correctly defined."""
        from essence_wars.agents.embeddings import CardIdPositions, STATE_TENSOR_SIZE

        positions = CardIdPositions.default()

        # Check hand positions
        assert positions.p1_hand == list(range(11, 21))
        assert positions.p2_hand == list(range(86, 96))
        assert len(positions.p1_hand) == 10
        assert len(positions.p2_hand) == 10

        # Check support positions (card_id at offset +2 in 5-float slots)
        assert positions.p1_supports == [73, 78]
        assert positions.p2_supports == [148, 153]

        # Check embed section (includes commander IDs at 326-327)
        assert positions.embed_section == list(range(156, STATE_TENSOR_SIZE))
        assert len(positions.embed_section) == STATE_TENSOR_SIZE - 156  # 172 with commanders

    def test_all_positions(self):
        """Test all_positions returns complete list."""
        from essence_wars.agents.embeddings import CardIdPositions, STATE_TENSOR_SIZE

        positions = CardIdPositions.default()
        all_pos = positions.all_positions()

        # Total: 10 + 10 + 2 + 2 + (STATE_TENSOR_SIZE - 156) card ID positions
        # With STATE_TENSOR_SIZE=328: 10 + 10 + 2 + 2 + 172 = 196
        expected = 10 + 10 + 2 + 2 + (STATE_TENSOR_SIZE - 156)
        assert len(all_pos) == expected

    def test_non_embed_section_positions(self):
        """Test non_embed_section_positions excludes trailing section."""
        from essence_wars.agents.embeddings import CardIdPositions

        positions = CardIdPositions.default()
        non_embed = positions.non_embed_section_positions()

        # Just hands and supports: 10 + 10 + 2 + 2 = 24
        assert len(non_embed) == 24


class TestGetNonCardPositions:
    """Tests for get_non_card_positions function."""

    def test_non_card_positions_count(self):
        """Test correct number of non-card positions."""
        from essence_wars.agents.embeddings import (
            CardIdPositions,
            get_non_card_positions,
            STATE_TENSOR_SIZE,
        )

        positions = CardIdPositions.default()
        non_card = get_non_card_positions(positions)

        # Total - card positions
        all_card = positions.all_positions()
        expected = STATE_TENSOR_SIZE - len(all_card)
        assert len(non_card) == expected
        assert len(non_card) == 132  # Non-card positions remain 132

    def test_no_overlap(self):
        """Test card and non-card positions don't overlap."""
        from essence_wars.agents.embeddings import (
            CardIdPositions,
            get_non_card_positions,
        )

        positions = CardIdPositions.default()
        card_positions = set(positions.all_positions())
        non_card_positions = set(get_non_card_positions(positions))

        assert len(card_positions & non_card_positions) == 0

    def test_complete_coverage(self):
        """Test card + non-card positions cover full tensor."""
        from essence_wars.agents.embeddings import (
            CardIdPositions,
            get_non_card_positions,
            STATE_TENSOR_SIZE,
        )

        positions = CardIdPositions.default()
        card_positions = set(positions.all_positions())
        non_card_positions = set(get_non_card_positions(positions))

        all_positions = card_positions | non_card_positions
        assert all_positions == set(range(STATE_TENSOR_SIZE))


class TestObservationTransformer:
    """Tests for ObservationTransformer class."""

    def test_creation(self):
        """Test transformer can be created."""
        from essence_wars.agents.embeddings import ObservationTransformer

        embed_dim = 64
        card_embedding = torch.nn.Embedding(5000, embed_dim, padding_idx=0)
        transformer = ObservationTransformer(card_embedding)

        assert transformer.embed_dim == embed_dim
        assert transformer.output_dim > 0

    def test_output_dimension_with_embed_section(self):
        """Test output dimension calculation with embed section."""
        from essence_wars.agents.embeddings import ObservationTransformer, STATE_TENSOR_SIZE

        embed_dim = 64
        card_embedding = torch.nn.Embedding(5000, embed_dim)
        transformer = ObservationTransformer(card_embedding, include_embed_section=True)

        # 132 non-card features + card slots * embed_dim
        # Card slots = 10 + 10 + 2 + 2 + (STATE_TENSOR_SIZE - 156)
        num_card_slots = 10 + 10 + 2 + 2 + (STATE_TENSOR_SIZE - 156)
        expected = 132 + num_card_slots * embed_dim
        assert transformer.output_dim == expected

    def test_output_dimension_without_embed_section(self):
        """Test output dimension without trailing embed section."""
        from essence_wars.agents.embeddings import ObservationTransformer, STATE_TENSOR_SIZE

        embed_dim = 32
        card_embedding = torch.nn.Embedding(5000, embed_dim)
        transformer = ObservationTransformer(card_embedding, include_embed_section=False)

        # When include_embed_section=False:
        # - The embed section is still part of the card positions for get_non_card_positions
        # - But we only embed 24 cards (hands + supports)
        # - non_card_positions = 132 (doesn't change based on include_embed_section)
        # - But the output uses a different calculation when include_embed_section=False
        # Actually checking the actual implementation:
        # transformer.non_card_positions uses get_non_card_positions() which returns 132
        # transformer.card_id_positions = non_embed_section_positions() = 24
        # So output_dim = 132 + 24 * embed_dim
        expected = 132 + 24 * embed_dim
        assert transformer.output_dim == expected

    def test_forward_single_obs(self):
        """Test forward pass with single observation."""
        from essence_wars.agents.embeddings import ObservationTransformer

        embed_dim = 64
        card_embedding = torch.nn.Embedding(5000, embed_dim)
        transformer = ObservationTransformer(card_embedding, include_embed_section=False)

        obs = torch.randn(328)
        embedded = transformer(obs)

        assert embedded.dim() == 1
        assert embedded.shape[0] == transformer.output_dim

    def test_forward_batch_obs(self):
        """Test forward pass with batch of observations."""
        from essence_wars.agents.embeddings import ObservationTransformer

        embed_dim = 64
        card_embedding = torch.nn.Embedding(5000, embed_dim)
        transformer = ObservationTransformer(card_embedding, include_embed_section=False)

        batch_size = 32
        obs = torch.randn(batch_size, 328)
        embedded = transformer(obs)

        assert embedded.dim() == 2
        assert embedded.shape == (batch_size, transformer.output_dim)

    def test_card_id_clamping(self):
        """Test card IDs are clamped to valid range."""
        from essence_wars.agents.embeddings import ObservationTransformer, MAX_CARD_ID

        embed_dim = 64
        card_embedding = torch.nn.Embedding(MAX_CARD_ID, embed_dim)
        transformer = ObservationTransformer(card_embedding, include_embed_section=False)

        # Create observation with out-of-range card IDs
        obs = torch.zeros(328)
        obs[11] = 99999.0  # Way too large
        obs[12] = -1.0     # Negative

        # Should not raise
        embedded = transformer(obs)
        assert embedded.shape[0] == transformer.output_dim

    def test_device_handling(self):
        """Test transformer works on different devices."""
        from essence_wars.agents.embeddings import ObservationTransformer

        embed_dim = 64
        card_embedding = torch.nn.Embedding(5000, embed_dim)
        transformer = ObservationTransformer(card_embedding, include_embed_section=False)

        obs = torch.randn(8, 328)
        embedded = transformer(obs)

        # Check buffers are on correct device
        assert transformer.card_id_indices.device == obs.device


class TestEmbeddedPPONetwork:
    """Tests for EmbeddedPPONetwork class."""

    def test_creation_end_to_end(self):
        """Test network creation with end-to-end embeddings."""
        from essence_wars.agents.embeddings import EmbeddedPPONetwork

        network = EmbeddedPPONetwork(embed_dim=64, hidden_dim=256)

        assert network.embed_dim == 64
        assert network.hidden_dim == 256
        assert network.card_embedding is not None

    def test_creation_pretrained(self):
        """Test network creation with pre-trained embeddings."""
        from essence_wars.agents.embeddings import EmbeddedPPONetwork

        # Create fake pre-trained embeddings
        pretrained = torch.randn(5000, 32)

        network = EmbeddedPPONetwork(pretrained_embeds=pretrained)

        assert network.embed_dim == 32  # Inherited from pretrained
        assert network.card_embedding.weight.shape == (5000, 32)

    def test_forward(self):
        """Test forward pass."""
        from essence_wars.agents.embeddings import EmbeddedPPONetwork

        network = EmbeddedPPONetwork(embed_dim=64)
        network.eval()

        batch_size = 8
        obs = torch.randn(batch_size, 328)
        mask = torch.ones(batch_size, 256, dtype=torch.bool)

        logits, value = network(obs, mask)

        assert logits.shape == (batch_size, 256)
        assert value.shape == (batch_size,)

    def test_action_masking(self):
        """Test action masking works correctly."""
        from essence_wars.agents.embeddings import EmbeddedPPONetwork

        network = EmbeddedPPONetwork(embed_dim=32)
        network.eval()

        obs = torch.randn(1, 328)
        mask = torch.zeros(1, 256, dtype=torch.bool)
        mask[0, 42] = True  # Only action 42 is legal

        with torch.no_grad():
            logits, _ = network(obs, mask)
            probs = torch.softmax(logits, dim=-1)

        assert probs[0, 42] > 0.99

    def test_get_action(self):
        """Test action sampling."""
        from essence_wars.agents.embeddings import EmbeddedPPONetwork

        network = EmbeddedPPONetwork(embed_dim=64)
        network.eval()

        batch_size = 4
        obs = torch.randn(batch_size, 328)
        mask = torch.ones(batch_size, 256, dtype=torch.bool)

        with torch.no_grad():
            action, log_prob, entropy = network.get_action(obs, mask)

        assert action.shape == (batch_size,)
        assert log_prob.shape == (batch_size,)
        assert entropy.shape == (batch_size,)
        assert (action >= 0).all() and (action < 256).all()

    def test_get_action_deterministic(self):
        """Test deterministic action selection."""
        from essence_wars.agents.embeddings import EmbeddedPPONetwork

        network = EmbeddedPPONetwork(embed_dim=64)
        network.eval()

        obs = torch.randn(1, 328)
        mask = torch.ones(1, 256, dtype=torch.bool)

        with torch.no_grad():
            action1, _, _ = network.get_action(obs, mask, deterministic=True)
            action2, _, _ = network.get_action(obs, mask, deterministic=True)

        assert action1.item() == action2.item()

    def test_get_value(self):
        """Test value estimation."""
        from essence_wars.agents.embeddings import EmbeddedPPONetwork

        network = EmbeddedPPONetwork(embed_dim=64)
        network.eval()

        obs = torch.randn(8, 328)

        with torch.no_grad():
            value = network.get_value(obs)

        assert value.shape == (8,)

    def test_evaluate_actions(self):
        """Test action evaluation for PPO update."""
        from essence_wars.agents.embeddings import EmbeddedPPONetwork

        network = EmbeddedPPONetwork(embed_dim=64)
        network.eval()

        batch_size = 8
        obs = torch.randn(batch_size, 328)
        mask = torch.ones(batch_size, 256, dtype=torch.bool)
        actions = torch.randint(0, 256, (batch_size,))

        log_prob, entropy, value = network.evaluate_actions(obs, mask, actions)

        assert log_prob.shape == (batch_size,)
        assert entropy.shape == (batch_size,)
        assert value.shape == (batch_size,)


class TestEmbeddedAlphaZeroNetwork:
    """Tests for EmbeddedAlphaZeroNetwork class."""

    def test_creation(self):
        """Test network creation."""
        from essence_wars.agents.embeddings import EmbeddedAlphaZeroNetwork

        network = EmbeddedAlphaZeroNetwork(embed_dim=64, num_blocks=4)

        assert network.embed_dim == 64
        assert network.num_blocks == 4
        assert len(network.residual_tower) == 4

    def test_forward(self):
        """Test forward pass."""
        from essence_wars.agents.embeddings import EmbeddedAlphaZeroNetwork

        network = EmbeddedAlphaZeroNetwork(embed_dim=64, num_blocks=2)
        network.eval()

        batch_size = 8
        obs = torch.randn(batch_size, 328)
        mask = torch.ones(batch_size, 256, dtype=torch.bool)

        logits, value = network(obs, mask)

        assert logits.shape == (batch_size, 256)
        assert value.shape == (batch_size,)

    def test_value_range(self):
        """Test value output is in [-1, 1] range."""
        from essence_wars.agents.embeddings import EmbeddedAlphaZeroNetwork

        network = EmbeddedAlphaZeroNetwork(embed_dim=64)
        network.eval()

        obs = torch.randn(100, 328)

        with torch.no_grad():
            _, value = network(obs)

        assert (value >= -1.0).all()
        assert (value <= 1.0).all()

    def test_get_policy(self):
        """Test policy probability output."""
        from essence_wars.agents.embeddings import EmbeddedAlphaZeroNetwork

        network = EmbeddedAlphaZeroNetwork(embed_dim=64)
        network.eval()

        obs = torch.randn(8, 328)
        mask = torch.ones(8, 256, dtype=torch.bool)

        with torch.no_grad():
            policy = network.get_policy(obs, mask)

        assert policy.shape == (8, 256)
        # Policy should sum to 1
        assert torch.allclose(policy.sum(dim=-1), torch.ones(8), atol=1e-5)

    def test_evaluate(self):
        """Test combined policy and value evaluation for MCTS."""
        from essence_wars.agents.embeddings import EmbeddedAlphaZeroNetwork

        network = EmbeddedAlphaZeroNetwork(embed_dim=64)
        network.eval()

        obs = torch.randn(8, 328)
        mask = torch.ones(8, 256, dtype=torch.bool)

        with torch.no_grad():
            policy, value = network.evaluate(obs, mask)

        assert policy.shape == (8, 256)
        assert value.shape == (8,)
        assert torch.allclose(policy.sum(dim=-1), torch.ones(8), atol=1e-5)


class TestCreateNetwork:
    """Tests for create_network factory function."""

    def test_create_flat_ppo(self):
        """Test creating flat PPO network."""
        from essence_wars.agents.embeddings import create_network

        network = create_network(observation_mode="flat", network_type="ppo")

        assert network.__class__.__name__ == "EssenceWarsNetwork"

    def test_create_flat_alphazero(self):
        """Test creating flat AlphaZero network."""
        from essence_wars.agents.embeddings import create_network

        network = create_network(observation_mode="flat", network_type="alphazero")

        assert network.__class__.__name__ == "AlphaZeroNetwork"

    def test_create_embedded_ppo(self):
        """Test creating embedded PPO network."""
        from essence_wars.agents.embeddings import create_network

        network = create_network(
            observation_mode="embedded",
            network_type="ppo",
            embed_dim=32,
        )

        assert network.__class__.__name__ == "EmbeddedPPONetwork"

    def test_create_embedded_alphazero(self):
        """Test creating embedded AlphaZero network."""
        from essence_wars.agents.embeddings import create_network

        network = create_network(
            observation_mode="embedded",
            network_type="alphazero",
            embed_dim=32,
            num_blocks=2,
        )

        assert network.__class__.__name__ == "EmbeddedAlphaZeroNetwork"


class TestGetEmbeddingInfo:
    """Tests for get_embedding_info function."""

    def test_embedding_info_structure(self):
        """Test embedding info returns expected structure."""
        from essence_wars.agents.embeddings import get_embedding_info

        info = get_embedding_info()

        assert "state_tensor_size" in info
        assert "num_card_positions" in info
        assert "num_non_card_positions" in info
        assert "card_positions" in info
        assert "max_card_id" in info

    def test_embedding_info_values(self):
        """Test embedding info has correct values."""
        from essence_wars.agents.embeddings import get_embedding_info, STATE_TENSOR_SIZE

        info = get_embedding_info()

        assert info["state_tensor_size"] == STATE_TENSOR_SIZE  # 328
        # Card positions = 10 + 10 + 2 + 2 + (STATE_TENSOR_SIZE - 156) = 196 with STATE_TENSOR_SIZE=328
        assert info["num_card_positions"] == 10 + 10 + 2 + 2 + (STATE_TENSOR_SIZE - 156)
        assert info["num_non_card_positions"] == 132
        assert info["max_card_id"] == 5000


class TestIntegrationWithRealObservations:
    """Integration tests using real game observations."""

    @pytest.fixture
    def real_observation(self):
        """Get a real observation from the game engine."""
        from essence_wars import PyGame

        game = PyGame()
        game.reset(seed=42)
        return torch.tensor(game.observe(), dtype=torch.float32)

    def test_transformer_with_real_obs(self, real_observation):
        """Test transformer works with real game observation."""
        from essence_wars.agents.embeddings import ObservationTransformer

        embed_dim = 64
        card_embedding = torch.nn.Embedding(5000, embed_dim)
        transformer = ObservationTransformer(card_embedding, include_embed_section=False)

        embedded = transformer(real_observation)

        assert embedded.dim() == 1
        assert embedded.shape[0] == transformer.output_dim
        assert not torch.isnan(embedded).any()
        assert not torch.isinf(embedded).any()

    def test_network_with_real_obs(self, real_observation):
        """Test embedded network works with real game observation."""
        from essence_wars.agents.embeddings import EmbeddedPPONetwork

        network = EmbeddedPPONetwork(embed_dim=64)
        network.eval()

        obs = real_observation.unsqueeze(0)
        mask = torch.ones(1, 256, dtype=torch.bool)

        with torch.no_grad():
            logits, value = network(obs, mask)

        assert logits.shape == (1, 256)
        assert value.shape == (1,)
        assert not torch.isnan(logits).any()
        assert not torch.isnan(value).any()

    def test_network_produces_valid_actions(self, real_observation):
        """Test network produces valid action probabilities."""
        from essence_wars.agents.embeddings import EmbeddedPPONetwork
        from essence_wars import PyGame

        game = PyGame()
        game.reset(seed=42)

        network = EmbeddedPPONetwork(embed_dim=64)
        network.eval()

        obs = torch.tensor(game.observe(), dtype=torch.float32).unsqueeze(0)
        mask = torch.tensor(game.action_mask(), dtype=torch.bool).unsqueeze(0)

        with torch.no_grad():
            action, log_prob, _ = network.get_action(obs, mask)

        # Action should be legal
        assert mask[0, action.item()], f"Selected illegal action {action.item()}"
