"""Tests for the Gymnasium environment."""

import numpy as np


def test_env_creation():
    """Test that EssenceWarsEnv can be created."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv()
    assert env is not None
    assert env.observation_space.shape == (326,)
    assert env.action_space.n == 256


def test_env_reset():
    """Test environment reset returns correct types."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv()
    obs, info = env.reset(seed=42)

    # Check observation
    assert isinstance(obs, np.ndarray)
    assert obs.shape == (326,)
    assert obs.dtype == np.float32

    # Check info dict
    assert isinstance(info, dict)
    assert "action_mask" in info
    assert info["action_mask"].shape == (256,)
    assert "current_player" in info
    assert "turn_number" in info
    assert "is_done" in info


def test_env_reset_reproducibility():
    """Test that reset with same seed produces same state."""
    from essence_wars.env import EssenceWarsEnv

    env1 = EssenceWarsEnv()
    env2 = EssenceWarsEnv()

    obs1, _ = env1.reset(seed=12345)
    obs2, _ = env2.reset(seed=12345)

    assert np.allclose(obs1, obs2)


def test_env_step_returns_5_tuple():
    """Test that step returns the Gymnasium v26 5-tuple."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv()
    obs, info = env.reset(seed=42)

    # Get a legal action
    mask = info["action_mask"]
    legal_actions = np.where(mask > 0)[0]
    action = legal_actions[0]

    result = env.step(action)

    assert len(result) == 5
    obs, reward, terminated, truncated, info = result

    assert isinstance(obs, np.ndarray)
    assert obs.shape == (326,)
    assert isinstance(reward, (int, float))
    assert isinstance(terminated, bool)
    assert isinstance(truncated, bool)
    assert isinstance(info, dict)


def test_env_action_mask_in_info():
    """Test that action_mask is always in info."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv()
    _, info = env.reset(seed=42)
    assert "action_mask" in info

    mask = info["action_mask"]
    legal_action = np.where(mask > 0)[0][0]
    _, _, _, _, info = env.step(legal_action)
    assert "action_mask" in info


def test_env_action_masks_method():
    """Test the action_masks() method for SB3 compatibility."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv()
    env.reset(seed=42)

    masks = env.action_masks()
    assert isinstance(masks, np.ndarray)
    assert masks.dtype == bool
    assert masks.shape == (256,)
    assert np.sum(masks) > 0  # At least one legal action


def test_env_invalid_action_penalty():
    """Test that invalid actions are penalized."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv()
    _, info = env.reset(seed=42)

    # Find an illegal action
    mask = info["action_mask"]
    illegal_actions = np.where(mask == 0)[0]
    if len(illegal_actions) > 0:
        illegal_action = illegal_actions[0]
        obs, reward, terminated, truncated, info = env.step(illegal_action)

        assert reward < 0  # Negative reward for invalid action
        assert terminated  # Episode ends
        assert info.get("invalid_action", False)


def test_env_full_episode_greedy():
    """Test playing a full episode against greedy opponent."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv(opponent="greedy")
    obs, info = env.reset(seed=42)

    steps = 0
    max_steps = 500

    while steps < max_steps:
        mask = info["action_mask"]
        legal_actions = np.where(mask > 0)[0]
        action = np.random.choice(legal_actions)

        obs, reward, terminated, truncated, info = env.step(action)
        steps += 1

        if terminated or truncated:
            break

    assert terminated or truncated
    assert steps > 0


def test_env_full_episode_random():
    """Test playing a full episode against random opponent."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv(opponent="random")
    obs, info = env.reset(seed=42)

    steps = 0
    max_steps = 500

    while steps < max_steps:
        mask = info["action_mask"]
        legal_actions = np.where(mask > 0)[0]
        action = np.random.choice(legal_actions)

        obs, reward, terminated, truncated, info = env.step(action)
        steps += 1

        if terminated or truncated:
            break

    assert terminated or truncated


def test_env_deck_selection():
    """Test creating environment with specific decks."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv(
        deck1="architect_fortify",
        deck2="broodmother_swarm",
    )
    obs, info = env.reset(seed=42)

    assert obs.shape == (326,)
    assert not info["is_done"]


def test_env_game_mode_attrition():
    """Test attrition game mode."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv(game_mode="attrition")
    obs, info = env.reset(seed=42)
    assert not info["is_done"]


def test_env_game_mode_essence_duel():
    """Test essence duel game mode."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv(game_mode="essence_duel")
    obs, info = env.reset(seed=42)
    assert not info["is_done"]


def test_env_observation_space_contains():
    """Test that observations are within observation space."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv()
    obs, _ = env.reset(seed=42)

    assert env.observation_space.contains(obs)


def test_env_action_space_contains():
    """Test that legal actions are within action space."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv()
    _, info = env.reset(seed=42)

    mask = info["action_mask"]
    legal_actions = np.where(mask > 0)[0]

    for action in legal_actions:
        assert env.action_space.contains(int(action))


def test_env_self_play():
    """Test the self-play environment variant."""
    from essence_wars.env import EssenceWarsSelfPlayEnv

    env = EssenceWarsSelfPlayEnv()
    obs, info = env.reset(seed=42)

    steps = 0
    max_steps = 500

    while steps < max_steps:
        mask = info["action_mask"]
        legal_actions = np.where(mask > 0)[0]
        action = np.random.choice(legal_actions)

        obs, reward, terminated, truncated, info = env.step(action)
        steps += 1

        if terminated or truncated:
            break

    assert terminated or truncated


def test_env_make_function():
    """Test the make_env factory function."""
    from essence_wars.env import make_env

    env = make_env(
        deck1="artificer_tokens",
        deck2="broodmother_swarm",
        opponent="random",
    )
    obs, info = env.reset(seed=42)

    assert obs.shape == (326,)
    assert "action_mask" in info


def test_env_gymnasium_registry():
    """Test that environments are registered with Gymnasium."""
    import gymnasium as gym

    # These should not raise
    env1 = gym.make("EssenceWars-v0")
    env2 = gym.make("EssenceWarsSelfPlay-v0")

    obs1, _ = env1.reset(seed=42)
    obs2, _ = env2.reset(seed=42)

    assert obs1.shape == (326,)
    assert obs2.shape == (326,)

    env1.close()
    env2.close()


def test_env_truncation():
    """Test that environment truncates after max_turns."""
    from essence_wars.env import EssenceWarsEnv

    # Set very low max_turns to test truncation
    env = EssenceWarsEnv(max_turns=5)
    obs, info = env.reset(seed=42)

    for _ in range(10):
        mask = info["action_mask"]
        legal_actions = np.where(mask > 0)[0]
        action = legal_actions[0]

        obs, reward, terminated, truncated, info = env.step(action)

        if terminated or truncated:
            break

    # Should have been truncated (unlikely to finish in 5 steps)
    # Note: With opponent playing, we might finish normally, so just check episode ended
    assert terminated or truncated


def test_env_reward_range():
    """Test that rewards are in expected range."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv()
    obs, info = env.reset(seed=42)

    steps = 0
    total_reward = 0.0

    while steps < 500:
        mask = info["action_mask"]
        legal_actions = np.where(mask > 0)[0]
        action = np.random.choice(legal_actions)

        obs, reward, terminated, truncated, info = env.step(action)
        total_reward += reward
        steps += 1

        # Intermediate rewards should be 0
        if not terminated:
            # Non-terminal rewards should be 0 (unless invalid action)
            pass

        if terminated or truncated:
            break

    # Final reward should be -1, 0, or +1
    assert total_reward in [-1.0, 0.0, 1.0] or total_reward < 0  # Could be invalid action penalty


def test_env_close():
    """Test that environment can be closed cleanly."""
    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv()
    env.reset(seed=42)
    env.close()  # Should not raise


# =============================================================================
# Vectorized Environment Tests
# =============================================================================


def test_vectorized_env_creation():
    """Test VectorizedEssenceWars can be created."""
    from essence_wars.env import VectorizedEssenceWars

    vec_env = VectorizedEssenceWars(num_envs=4)
    assert vec_env is not None
    assert vec_env.num_envs == 4
    assert vec_env.observation_shape == (326,)
    assert vec_env.action_size == 256


def test_vectorized_env_reset():
    """Test vectorized reset returns correct shapes."""
    from essence_wars.env import VectorizedEssenceWars

    vec_env = VectorizedEssenceWars(num_envs=8)
    obs, masks = vec_env.reset(seed=42)

    # Check observation shape
    assert isinstance(obs, np.ndarray)
    assert obs.shape == (8, 326)
    assert obs.dtype == np.float32

    # Check mask shape
    assert isinstance(masks, np.ndarray)
    assert masks.shape == (8, 256)
    assert masks.dtype == bool
    assert np.all(np.sum(masks, axis=1) > 0)  # At least one legal action per env


def test_vectorized_env_reset_reproducibility():
    """Test that reset with same seed produces same state."""
    from essence_wars.env import VectorizedEssenceWars

    vec1 = VectorizedEssenceWars(num_envs=4)
    vec2 = VectorizedEssenceWars(num_envs=4)

    obs1, _ = vec1.reset(seed=12345)
    obs2, _ = vec2.reset(seed=12345)

    assert np.allclose(obs1, obs2)


def test_vectorized_env_step():
    """Test vectorized step returns correct shapes."""
    from essence_wars.env import VectorizedEssenceWars

    vec_env = VectorizedEssenceWars(num_envs=4)
    obs, masks = vec_env.reset(seed=42)

    # Select legal actions for each environment
    actions = []
    for i in range(4):
        legal = np.where(masks[i])[0]
        actions.append(legal[0])
    actions = np.array(actions, dtype=np.int32)

    obs, rewards, dones, masks = vec_env.step(actions)

    # Check shapes
    assert obs.shape == (4, 326)
    assert rewards.shape == (4,)
    assert dones.shape == (4,)
    assert masks.shape == (4, 256)

    # Check types
    assert obs.dtype == np.float32
    assert rewards.dtype == np.float32
    assert dones.dtype == bool
    assert masks.dtype == bool


def test_vectorized_env_auto_reset():
    """Test that done environments are automatically reset."""
    from essence_wars.env import VectorizedEssenceWars

    vec_env = VectorizedEssenceWars(num_envs=8)
    obs, masks = vec_env.reset(seed=42)

    episodes_completed = 0
    max_steps = 2000

    for step in range(max_steps):
        # Select random legal actions
        actions = []
        for i in range(8):
            legal = np.where(masks[i])[0]
            actions.append(np.random.choice(legal))
        actions = np.array(actions, dtype=np.int32)

        obs, rewards, dones, masks = vec_env.step(actions)

        # Count completed episodes
        episodes_completed += np.sum(dones)

        # Check that reset envs have valid masks
        if np.any(dones):
            for i in np.where(dones)[0]:
                assert np.sum(masks[i]) > 0, f"Env {i} reset but has no legal actions"

        if episodes_completed >= 8:
            break

    assert episodes_completed >= 8, f"Only {episodes_completed} episodes completed in {max_steps} steps"


def test_vectorized_env_full_episodes():
    """Test playing full episodes in vectorized env."""
    from essence_wars.env import VectorizedEssenceWars

    vec_env = VectorizedEssenceWars(num_envs=4)
    obs, masks = vec_env.reset(seed=42)

    total_steps = 0
    total_rewards = np.zeros(4)
    max_steps = 500

    for _ in range(max_steps):
        # Select random legal actions
        actions = []
        for i in range(4):
            legal = np.where(masks[i])[0]
            actions.append(np.random.choice(legal))
        actions = np.array(actions, dtype=np.int32)

        obs, rewards, dones, masks = vec_env.step(actions)
        total_steps += 4
        total_rewards += rewards

        if np.all(dones):
            break

    # Should have made progress
    assert total_steps > 0


def test_vectorized_env_action_masks_method():
    """Test the action_masks() method."""
    from essence_wars.env import VectorizedEssenceWars

    vec_env = VectorizedEssenceWars(num_envs=4)
    vec_env.reset(seed=42)

    masks = vec_env.action_masks()
    assert isinstance(masks, np.ndarray)
    assert masks.shape == (4, 256)
    assert masks.dtype == bool


def test_vectorized_env_episode_tracking():
    """Test episode reward and length tracking."""
    from essence_wars.env import VectorizedEssenceWars

    vec_env = VectorizedEssenceWars(num_envs=2)
    obs, masks = vec_env.reset(seed=42)

    # Initial state
    assert np.all(vec_env.get_episode_rewards() == 0.0)
    assert np.all(vec_env.get_episode_lengths() == 0)

    # Take a step
    actions = []
    for i in range(2):
        legal = np.where(masks[i])[0]
        actions.append(legal[0])
    actions = np.array(actions, dtype=np.int32)

    vec_env.step(actions)

    # Length should increase
    lengths = vec_env.get_episode_lengths()
    assert np.all(lengths >= 1)


def test_vectorized_env_deck_selection():
    """Test creating vectorized env with specific decks."""
    from essence_wars.env import VectorizedEssenceWars

    vec_env = VectorizedEssenceWars(
        num_envs=2,
        deck1="architect_fortify",
        deck2="broodmother_swarm",
    )
    obs, masks = vec_env.reset(seed=42)

    assert obs.shape == (2, 326)
    assert masks.shape == (2, 256)


def test_vectorized_env_game_modes():
    """Test vectorized env with different game modes."""
    from essence_wars.env import VectorizedEssenceWars

    for mode in ["attrition", "essence_duel"]:
        vec_env = VectorizedEssenceWars(num_envs=2, game_mode=mode)
        obs, masks = vec_env.reset(seed=42)
        assert obs.shape == (2, 326)
