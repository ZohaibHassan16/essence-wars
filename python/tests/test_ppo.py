"""Tests for PPO agent components."""

import numpy as np
import torch


def test_network_creation():
    """Test EssenceWarsNetwork can be created."""
    from essence_wars.agents.networks import EssenceWarsNetwork

    network = EssenceWarsNetwork()
    assert network is not None
    assert network.obs_dim == 326
    assert network.action_dim == 256
    assert network.hidden_dim == 256


def test_network_forward():
    """Test network forward pass."""
    from essence_wars.agents.networks import EssenceWarsNetwork

    network = EssenceWarsNetwork()
    network.eval()

    batch_size = 8
    obs = torch.randn(batch_size, 326)
    mask = torch.ones(batch_size, 256, dtype=torch.bool)

    logits, value = network(obs, mask)

    assert logits.shape == (batch_size, 256)
    assert value.shape == (batch_size,)


def test_network_action_masking():
    """Test that action masking works correctly."""
    from essence_wars.agents.networks import EssenceWarsNetwork

    network = EssenceWarsNetwork()
    network.eval()

    obs = torch.randn(1, 326)

    # Mask all but one action
    mask = torch.zeros(1, 256, dtype=torch.bool)
    mask[0, 42] = True  # Only action 42 is legal

    with torch.no_grad():
        logits, _ = network(obs, mask)
        probs = torch.softmax(logits, dim=-1)

    # Action 42 should have probability ~1.0
    assert probs[0, 42] > 0.99


def test_network_get_action():
    """Test action sampling."""
    from essence_wars.agents.networks import EssenceWarsNetwork

    network = EssenceWarsNetwork()
    network.eval()

    batch_size = 4
    obs = torch.randn(batch_size, 326)
    mask = torch.ones(batch_size, 256, dtype=torch.bool)

    with torch.no_grad():
        action, log_prob, entropy = network.get_action(obs, mask)

    assert action.shape == (batch_size,)
    assert log_prob.shape == (batch_size,)
    assert entropy.shape == (batch_size,)
    assert (action >= 0).all() and (action < 256).all()


def test_network_get_action_deterministic():
    """Test deterministic action selection."""
    from essence_wars.agents.networks import EssenceWarsNetwork

    network = EssenceWarsNetwork()
    network.eval()

    obs = torch.randn(1, 326)
    mask = torch.ones(1, 256, dtype=torch.bool)

    with torch.no_grad():
        action1, _, _ = network.get_action(obs, mask, deterministic=True)
        action2, _, _ = network.get_action(obs, mask, deterministic=True)

    # Deterministic should give same action
    assert action1.item() == action2.item()


def test_network_get_value():
    """Test value estimation."""
    from essence_wars.agents.networks import EssenceWarsNetwork

    network = EssenceWarsNetwork()
    network.eval()

    obs = torch.randn(8, 326)

    with torch.no_grad():
        value = network.get_value(obs)

    assert value.shape == (8,)


def test_network_evaluate_actions():
    """Test action evaluation for PPO update."""
    from essence_wars.agents.networks import EssenceWarsNetwork

    network = EssenceWarsNetwork()
    network.eval()

    batch_size = 8
    obs = torch.randn(batch_size, 326)
    mask = torch.ones(batch_size, 256, dtype=torch.bool)
    actions = torch.randint(0, 256, (batch_size,))

    log_prob, entropy, value = network.evaluate_actions(obs, mask, actions)

    assert log_prob.shape == (batch_size,)
    assert entropy.shape == (batch_size,)
    assert value.shape == (batch_size,)


def test_ppo_config():
    """Test PPO configuration."""
    from essence_wars.agents.ppo import PPOConfig

    config = PPOConfig(num_envs=32, total_timesteps=100_000)

    assert config.num_envs == 32
    assert config.total_timesteps == 100_000
    assert config.batch_size == 32 * config.num_steps


def test_rollout_buffer():
    """Test rollout buffer operations."""
    from essence_wars.agents.ppo import RolloutBuffer

    buffer = RolloutBuffer(
        num_steps=8,
        num_envs=4,
        obs_dim=326,
        action_dim=256,
        device="cpu",
    )

    # Add some data
    for _ in range(8):
        buffer.add(
            obs=torch.randn(4, 326),
            action=torch.randint(0, 256, (4,)),
            action_mask=torch.ones(4, 256, dtype=torch.bool),
            log_prob=torch.randn(4),
            reward=torch.randn(4),
            done=torch.zeros(4),
            value=torch.randn(4),
        )

    assert buffer.step == 8

    # Compute returns
    buffer.compute_returns_and_advantages(
        last_value=torch.randn(4),
        gamma=0.99,
        gae_lambda=0.95,
    )

    assert buffer.advantages.shape == (8, 4)
    assert buffer.returns.shape == (8, 4)

    # Get batches
    batches = list(buffer.get_batches(minibatch_size=16))
    assert len(batches) == 2  # 32 total / 16 per batch


def test_ppo_trainer_creation():
    """Test PPOTrainer can be created."""
    from essence_wars.agents.ppo import PPOConfig, PPOTrainer

    config = PPOConfig(num_envs=4, total_timesteps=1000)
    trainer = PPOTrainer(config=config)

    assert trainer is not None
    assert trainer.network is not None
    assert trainer.envs is not None


def test_ppo_trainer_collect_rollout():
    """Test collecting a rollout."""
    from essence_wars.agents.ppo import PPOConfig, PPOTrainer

    config = PPOConfig(num_envs=4, num_steps=8, total_timesteps=1000)
    trainer = PPOTrainer(config=config)

    # Initialize environment
    trainer._last_obs, trainer._last_masks = trainer.envs.reset(seed=42)

    rollout_info = trainer.collect_rollout()

    assert "episode_infos" in rollout_info
    assert trainer.buffer.step == config.num_steps  # Buffer filled with num_steps
    assert trainer.global_step == config.num_envs * config.num_steps


def test_ppo_trainer_update():
    """Test PPO update step."""
    from essence_wars.agents.ppo import PPOConfig, PPOTrainer

    config = PPOConfig(num_envs=4, num_steps=8, num_epochs=1, num_minibatches=2)
    trainer = PPOTrainer(config=config)

    # Collect rollout first
    trainer._last_obs, trainer._last_masks = trainer.envs.reset(seed=42)
    trainer.collect_rollout()

    # Perform update
    update_info = trainer.update()

    assert "policy_loss" in update_info
    assert "value_loss" in update_info
    assert "entropy" in update_info


def test_ppo_trainer_short_training():
    """Test a very short training run."""
    from essence_wars.agents.ppo import PPOConfig, PPOTrainer

    config = PPOConfig(
        num_envs=4,
        num_steps=16,
        total_timesteps=256,  # Very short: 4 updates
        log_interval=10000,  # Don't log during test
        eval_interval=10000,  # Don't eval during test
    )
    trainer = PPOTrainer(config=config)

    results = trainer.train()

    assert results["total_timesteps"] >= 256
    assert "mean_reward" in results


def test_ppo_evaluate_vs_greedy():
    """Test evaluation against greedy bot."""
    from essence_wars.agents.ppo import PPOConfig, PPOTrainer

    config = PPOConfig(num_envs=4)
    trainer = PPOTrainer(config=config)

    # Evaluate untrained network (should be ~random)
    win_rate = trainer.evaluate_vs_greedy(num_games=10)

    assert 0.0 <= win_rate <= 1.0


def test_ppo_evaluate_vs_random():
    """Test evaluation against random bot."""
    from essence_wars.agents.ppo import PPOConfig, PPOTrainer

    config = PPOConfig(num_envs=4)
    trainer = PPOTrainer(config=config)

    win_rate = trainer.evaluate_vs_random(num_games=10)

    assert 0.0 <= win_rate <= 1.0


def test_ppo_save_load(tmp_path):
    """Test saving and loading checkpoints."""
    from essence_wars.agents.ppo import PPOConfig, PPOTrainer

    config = PPOConfig(num_envs=4)
    trainer1 = PPOTrainer(config=config)

    # Train briefly
    trainer1._last_obs, trainer1._last_masks = trainer1.envs.reset(seed=42)
    trainer1.collect_rollout()
    trainer1.update()

    # Save
    save_path = str(tmp_path / "test_checkpoint.pt")
    trainer1.save(save_path)

    # Load into new trainer
    trainer2 = PPOTrainer(config=config)
    trainer2.load(save_path)

    # Check weights match
    for p1, p2 in zip(trainer1.network.parameters(), trainer2.network.parameters()):
        assert torch.allclose(p1, p2)


def test_observation_normalization():
    """Test that observation normalization works correctly."""
    from essence_wars.agents.ppo import PPOConfig, PPOTrainer, RunningMeanStd

    # Test RunningMeanStd directly
    rms = RunningMeanStd((10,))
    batch = np.random.randn(100, 10) * 10 + 50  # Mean 50, std 10
    rms.update(batch)

    # After update, normalization should work
    normalized = rms.normalize(batch)
    assert abs(normalized.mean()) < 1.0
    assert abs(normalized.std() - 1.0) < 0.5

    # Test in PPOTrainer
    config = PPOConfig(num_envs=4, num_steps=8, normalize_obs=True)
    trainer = PPOTrainer(config=config)

    assert trainer.obs_normalizer is not None

    # Collect rollout
    trainer._last_obs, trainer._last_masks = trainer.envs.reset(seed=42)
    trainer.collect_rollout()

    # Check normalized observations are in reasonable range
    obs_min = trainer.buffer.obs.min().item()
    obs_max = trainer.buffer.obs.max().item()
    assert abs(obs_min) < 20, f"Normalized obs min {obs_min} too extreme"
    assert abs(obs_max) < 20, f"Normalized obs max {obs_max} too extreme"


def test_observation_normalization_disabled():
    """Test that observation normalization can be disabled."""
    from essence_wars.agents.ppo import PPOConfig, PPOTrainer

    config = PPOConfig(num_envs=4, num_steps=8, normalize_obs=False)
    trainer = PPOTrainer(config=config)

    assert trainer.obs_normalizer is None


def test_normalizer_state_save_load(tmp_path):
    """Test that normalizer state is saved and loaded correctly."""
    from essence_wars.agents.ppo import PPOConfig, PPOTrainer

    config = PPOConfig(num_envs=4, num_steps=8, normalize_obs=True)
    trainer1 = PPOTrainer(config=config)

    # Collect some data to update normalizer
    trainer1._last_obs, trainer1._last_masks = trainer1.envs.reset(seed=42)
    trainer1.collect_rollout()

    # Save normalizer state
    original_mean = trainer1.obs_normalizer.mean.copy()
    original_var = trainer1.obs_normalizer.var.copy()
    original_count = trainer1.obs_normalizer.count

    # Save and load
    save_path = str(tmp_path / "test_normalizer.pt")
    trainer1.save(save_path)

    trainer2 = PPOTrainer(config=config)
    trainer2.load(save_path)

    # Check normalizer state matches
    assert np.allclose(trainer2.obs_normalizer.mean, original_mean)
    assert np.allclose(trainer2.obs_normalizer.var, original_var)
    assert trainer2.obs_normalizer.count == original_count
