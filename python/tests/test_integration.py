"""Integration tests for the full Essence Wars training pipeline.

These tests verify that all components work together:
1. Environment creation and basic usage
2. Short training runs complete without errors
3. Model checkpoints can be saved and loaded
4. Trained models can be evaluated
"""

from __future__ import annotations

import tempfile
from pathlib import Path

import numpy as np
import pytest
import torch

from essence_wars import EssenceWarsEnv, PyGame, VectorizedEssenceWars
from essence_wars._core import ACTION_SPACE_SIZE, STATE_TENSOR_SIZE
from essence_wars.benchmark import GreedyAgent, RandomAgent


class TestEnvironmentIntegration:
    """Test that all environment variants work together."""

    def test_single_env_full_episode(self) -> None:
        """Run a complete episode with the single-env API."""
        env = EssenceWarsEnv(
            deck1="artificer_tokens",
            deck2="broodmother_pack",
            opponent="greedy",
        )

        obs, info = env.reset(seed=42)
        assert obs.shape == (STATE_TENSOR_SIZE,)
        assert "action_mask" in info

        total_reward = 0.0
        steps = 0

        while True:
            mask = info["action_mask"]
            valid_actions = np.where(mask > 0)[0]
            action = np.random.choice(valid_actions)

            obs, reward, terminated, truncated, info = env.step(action)
            total_reward += float(reward)
            steps += 1

            if terminated or truncated:
                break

        # Game should complete in reasonable number of steps
        assert steps > 0
        assert steps < 500
        assert total_reward in (-1.0, 0.0, 1.0)  # Win/loss/draw

        env.close()

    def test_vectorized_env_full_episode(self) -> None:
        """Run episodes with the vectorized API."""
        num_envs = 8
        env = VectorizedEssenceWars(num_envs=num_envs)

        obs, masks = env.reset(seed=42)
        assert obs.shape == (num_envs, STATE_TENSOR_SIZE)
        assert masks.shape == (num_envs, ACTION_SPACE_SIZE)

        # Run for a fixed number of steps
        done_count = 0
        for _ in range(100):
            actions = np.array(
                [np.random.choice(np.where(masks[i] > 0)[0]) for i in range(num_envs)],
                dtype=np.uint8,
            )

            obs, rewards, dones, masks = env.step(actions)
            done_count += np.sum(dones)

        # Some games should have completed
        assert done_count > 0
        env.close()

    def test_game_fork_consistency(self) -> None:
        """Test that game forking produces consistent results."""
        game = PyGame()
        game.reset(seed=42)

        # Play a few moves
        for _ in range(5):
            action = game.greedy_action()
            game.step(action)

        # Fork the game
        forked = game.fork()

        # Play same action on both
        action = game.greedy_action()
        r1, d1 = game.step(action)
        r2, d2 = forked.step(action)

        assert r1 == r2
        assert d1 == d2
        np.testing.assert_array_equal(game.observe(), forked.observe())


class TestTrainingIntegration:
    """Test that training pipelines complete without errors."""

    @pytest.fixture
    def simple_network(self) -> torch.nn.Module:
        """Create a simple test network."""

        class SimpleNetwork(torch.nn.Module):
            def __init__(self) -> None:
                super().__init__()
                self.fc = torch.nn.Linear(STATE_TENSOR_SIZE, 128)
                self.policy = torch.nn.Linear(128, ACTION_SPACE_SIZE)
                self.value = torch.nn.Linear(128, 1)

            def forward(
                self, obs: torch.Tensor, mask: torch.Tensor
            ) -> tuple[torch.Tensor, torch.Tensor]:
                x = torch.relu(self.fc(obs))
                logits = self.policy(x)
                logits = logits.masked_fill(~mask.bool(), float("-inf"))
                value = self.value(x)
                return logits, value.squeeze(-1)

        return SimpleNetwork()

    def test_ppo_training_cycle(self, simple_network: torch.nn.Module) -> None:
        """Test a minimal PPO training cycle."""
        env = VectorizedEssenceWars(num_envs=4)
        optimizer = torch.optim.Adam(simple_network.parameters(), lr=1e-3)

        obs, masks = env.reset(seed=42)

        # Collect rollout
        obs_list, action_list, reward_list = [], [], []
        for _ in range(10):
            obs_t = torch.from_numpy(obs).float()
            mask_t = torch.from_numpy(masks).float()

            with torch.no_grad():
                logits, _ = simple_network(obs_t, mask_t)
                probs = torch.softmax(logits, dim=-1)
                actions = torch.multinomial(probs, 1).squeeze(-1)

            actions_np = actions.numpy().astype(np.uint8)
            obs, rewards, dones, masks = env.step(actions_np)

            obs_list.append(obs_t)
            action_list.append(actions)
            reward_list.append(torch.from_numpy(rewards))

        # Stack and compute simple loss
        obs_batch = torch.cat(obs_list)
        actions_batch = torch.cat(action_list)
        rewards_batch = torch.cat(reward_list)

        mask_batch = torch.ones(len(obs_batch), ACTION_SPACE_SIZE)  # Simplified
        logits, values = simple_network(obs_batch, mask_batch)
        log_probs = torch.log_softmax(logits, dim=-1)
        action_log_probs = log_probs.gather(1, actions_batch.unsqueeze(1)).squeeze()

        # Simple policy gradient loss
        loss = -(action_log_probs * rewards_batch).mean()

        optimizer.zero_grad()
        loss.backward()
        optimizer.step()

        # Loss should be finite
        assert torch.isfinite(loss)
        env.close()

    def test_checkpoint_save_load(self, simple_network: torch.nn.Module) -> None:
        """Test that checkpoints can be saved and loaded."""
        with tempfile.TemporaryDirectory() as tmpdir:
            checkpoint_path = Path(tmpdir) / "test_checkpoint.pt"

            # Save
            checkpoint = {
                "network_state_dict": simple_network.state_dict(),
                "config": {
                    "hidden_dim": 128,
                    "obs_dim": STATE_TENSOR_SIZE,
                    "action_dim": ACTION_SPACE_SIZE,
                },
            }
            torch.save(checkpoint, checkpoint_path)
            assert checkpoint_path.exists()

            # Load
            loaded = torch.load(checkpoint_path, weights_only=False)
            assert "network_state_dict" in loaded
            assert "config" in loaded

            # Verify state dict can be loaded
            new_network = type(simple_network)()
            new_network.load_state_dict(loaded["network_state_dict"])

            # Verify outputs match
            test_input = torch.randn(1, STATE_TENSOR_SIZE)
            test_mask = torch.ones(1, ACTION_SPACE_SIZE)

            with torch.no_grad():
                out1, _ = simple_network(test_input, test_mask)
                out2, _ = new_network(test_input, test_mask)

            torch.testing.assert_close(out1, out2)


class TestBenchmarkIntegration:
    """Test the benchmark evaluation pipeline."""

    def test_greedy_vs_random_evaluation(self) -> None:
        """Test evaluation of agents against each other."""
        wins = 0
        num_games = 10

        for seed in range(num_games):
            game = PyGame()
            game.reset(seed=seed)

            while not game.is_done():
                if game.current_player() == 0:
                    action = game.greedy_action()
                else:
                    action = game.random_action()
                game.step(action)

            # get_reward(0) returns reward for player 0: +1 win, -1 loss, 0 draw
            reward = game.get_reward(0)
            if reward > 0:
                wins += 1

        # Greedy should win majority against random
        assert wins >= 5, f"Greedy won {wins}/10 vs random"

    def test_benchmark_agents_have_name(self) -> None:
        """Test that benchmark agents have a name property."""
        agents = [RandomAgent(), GreedyAgent()]

        for agent in agents:
            assert hasattr(agent, "name")
            assert isinstance(agent.name, str)
            assert len(agent.name) > 0


class TestErrorHandling:
    """Test that helpful error messages are provided."""

    def test_invalid_deck_error_message(self) -> None:
        """Test that invalid deck names give helpful errors."""
        with pytest.raises(ValueError) as exc_info:
            EssenceWarsEnv(deck1="invalid_deck", deck2="broodmother_pack")

        error_msg = str(exc_info.value)
        assert "Unknown deck" in error_msg
        assert "Available decks" in error_msg
        assert "artificer_tokens" in error_msg  # Should suggest valid decks

    def test_invalid_opponent_error_message(self) -> None:
        """Test that invalid opponent types give helpful errors."""
        with pytest.raises(ValueError) as exc_info:
            EssenceWarsEnv(opponent="mcts")  # Not a valid opponent type

        error_msg = str(exc_info.value)
        assert "Unknown opponent type" in error_msg
        assert "greedy" in error_msg
        assert "random" in error_msg

    def test_invalid_game_mode_error_message(self) -> None:
        """Test that invalid game modes give helpful errors."""
        with pytest.raises(ValueError) as exc_info:
            EssenceWarsEnv(game_mode="blitz")

        error_msg = str(exc_info.value)
        assert "Unknown game mode" in error_msg
        assert "attrition" in error_msg


class TestCrossComponentConsistency:
    """Test consistency between different components."""

    def test_observation_shape_consistency(self) -> None:
        """Verify observation shape is consistent across APIs."""
        # Single game
        game = PyGame()
        game.reset(seed=42)
        obs1 = game.observe()

        # Gymnasium env
        env = EssenceWarsEnv()
        obs2, _ = env.reset(seed=42)

        # Vectorized
        vec_env = VectorizedEssenceWars(num_envs=1)
        obs3, _ = vec_env.reset(seed=42)

        assert obs1.shape == (STATE_TENSOR_SIZE,)
        assert obs2.shape == (STATE_TENSOR_SIZE,)
        assert obs3.shape == (1, STATE_TENSOR_SIZE)

        env.close()
        vec_env.close()

    def test_action_mask_shape_consistency(self) -> None:
        """Verify action mask shape is consistent across APIs."""
        game = PyGame()
        game.reset(seed=42)
        mask1 = game.action_mask()

        env = EssenceWarsEnv()
        _, info = env.reset(seed=42)
        mask2 = info["action_mask"]

        vec_env = VectorizedEssenceWars(num_envs=1)
        _, masks = vec_env.reset(seed=42)

        assert mask1.shape == (ACTION_SPACE_SIZE,)
        assert mask2.shape == (ACTION_SPACE_SIZE,)
        assert masks.shape == (1, ACTION_SPACE_SIZE)

        env.close()
        vec_env.close()

    def test_deck_listing_consistency(self) -> None:
        """Verify deck listings are consistent."""
        decks = PyGame.list_decks()

        # All decks should work with all environment types
        for deck in decks[:3]:  # Test first 3 for speed
            game = PyGame(deck1=deck, deck2=deck)
            game.reset(seed=42)
            assert not game.is_done() or game.winner() >= -1

            env = EssenceWarsEnv(deck1=deck, deck2=deck)
            obs, _ = env.reset(seed=42)
            assert obs.shape == (STATE_TENSOR_SIZE,)
            env.close()
