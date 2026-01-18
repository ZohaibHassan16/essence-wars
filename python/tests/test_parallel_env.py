"""Tests for the PettingZoo parallel environment."""

import numpy as np
import pytest


def test_parallel_env_creation():
    """Test that EssenceWarsParallelEnv can be created."""
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env = EssenceWarsParallelEnv()
    assert env is not None
    assert env.possible_agents == ["player_0", "player_1"]
    assert len(env.agents) == 2


def test_parallel_env_reset():
    """Test environment reset returns correct types."""
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env = EssenceWarsParallelEnv()
    observations, infos = env.reset(seed=42)

    # Check observations
    assert isinstance(observations, dict)
    assert "player_0" in observations
    assert "player_1" in observations
    assert observations["player_0"].shape == (326,)
    assert observations["player_1"].shape == (326,)

    # Check infos
    assert isinstance(infos, dict)
    assert "player_0" in infos
    assert "player_1" in infos
    assert "action_mask" in infos["player_0"]
    assert "action_mask" in infos["player_1"]


def test_parallel_env_action_mask():
    """Test that only active player has legal actions."""
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env = EssenceWarsParallelEnv()
    _, infos = env.reset(seed=42)

    # One player should be active, one inactive
    p0_legal = np.sum(infos["player_0"]["action_mask"])
    p1_legal = np.sum(infos["player_1"]["action_mask"])

    # Exactly one player should have legal actions
    assert (p0_legal > 0) != (p1_legal > 0), "Exactly one player should be active"

    # The active player should have at least one legal action
    assert p0_legal > 0 or p1_legal > 0


def test_parallel_env_spaces():
    """Test observation and action spaces."""
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env = EssenceWarsParallelEnv()

    for agent in env.possible_agents:
        obs_space = env.observation_space(agent)
        act_space = env.action_space(agent)

        assert obs_space.shape == (326,)
        assert act_space.n == 256


def test_parallel_env_step():
    """Test stepping the environment."""
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env = EssenceWarsParallelEnv()
    observations, infos = env.reset(seed=42)

    # Find active player and get a legal action
    active_agent = None
    action = None
    for agent in env.agents:
        mask = infos[agent]["action_mask"]
        legal_actions = np.where(mask)[0]
        if len(legal_actions) > 0:
            active_agent = agent
            action = int(legal_actions[0])
            break

    assert active_agent is not None, "Should have an active player"

    # Step with the action
    actions = {active_agent: action}
    observations, rewards, terminations, truncations, infos = env.step(actions)

    # Check return types
    assert isinstance(observations, dict)
    assert isinstance(rewards, dict)
    assert isinstance(terminations, dict)
    assert isinstance(truncations, dict)
    assert isinstance(infos, dict)

    # Check all agents are present
    for agent in env.possible_agents:
        assert agent in observations
        assert agent in rewards
        assert agent in terminations
        assert agent in truncations
        assert agent in infos


def test_parallel_env_rewards_zero_sum():
    """Test that rewards are zero-sum (opposite for each player)."""
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env = EssenceWarsParallelEnv()
    observations, infos = env.reset(seed=42)

    # Play a few steps
    for _ in range(10):
        if not env.agents:
            break

        # Get active player's action
        actions = {}
        for agent in env.agents:
            mask = infos[agent]["action_mask"]
            legal_actions = np.where(mask)[0]
            if len(legal_actions) > 0:
                actions[agent] = int(np.random.choice(legal_actions))

        if not actions:
            break

        observations, rewards, terminations, truncations, infos = env.step(actions)

        # Check zero-sum property
        assert rewards["player_0"] == -rewards["player_1"], \
            f"Rewards should be opposite: {rewards}"


def test_parallel_env_full_game():
    """Test playing a full game until termination."""
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env = EssenceWarsParallelEnv()
    observations, infos = env.reset(seed=42)

    steps = 0
    max_steps = 500

    while env.agents and steps < max_steps:
        # Get active player's action
        actions = {}
        for agent in env.agents:
            mask = infos[agent]["action_mask"]
            legal_actions = np.where(mask)[0]
            if len(legal_actions) > 0:
                actions[agent] = int(np.random.choice(legal_actions))

        if not actions:
            break

        observations, rewards, terminations, truncations, infos = env.step(actions)
        steps += 1

        if all(terminations.values()) or all(truncations.values()):
            break

    # Game should have ended
    assert steps > 0
    assert len(env.agents) == 0, "Game should have ended"


def test_parallel_env_deck_selection():
    """Test creating environment with specific decks."""
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env = EssenceWarsParallelEnv(
        deck1="architect_fortify",
        deck2="broodmother_swarm",
    )
    observations, infos = env.reset(seed=42)

    assert observations["player_0"].shape == (326,)


def test_parallel_env_game_modes():
    """Test different game modes."""
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    for mode in ["attrition", "essence_duel"]:
        env = EssenceWarsParallelEnv(game_mode=mode)
        observations, infos = env.reset(seed=42)
        assert observations["player_0"].shape == (326,)


def test_parallel_env_reset_reproducibility():
    """Test that reset with same seed produces same state."""
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env1 = EssenceWarsParallelEnv()
    env2 = EssenceWarsParallelEnv()

    obs1, _ = env1.reset(seed=12345)
    obs2, _ = env2.reset(seed=12345)

    assert np.allclose(obs1["player_0"], obs2["player_0"])
    assert np.allclose(obs1["player_1"], obs2["player_1"])


def test_parallel_env_state():
    """Test the global state method."""
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env = EssenceWarsParallelEnv()
    env.reset(seed=42)

    state = env.state()
    assert isinstance(state, np.ndarray)
    assert state.shape == (326,)


def test_parallel_env_factory_functions():
    """Test the factory functions."""
    from essence_wars.parallel_env import env, parallel_env, raw_env

    env1 = parallel_env()
    env2 = raw_env()
    env3 = env()

    assert env1 is not None
    assert env2 is not None
    assert env3 is not None

    # All should reset properly
    for e in [env1, env2, env3]:
        obs, info = e.reset(seed=42)
        assert "player_0" in obs


def test_parallel_env_two_random_agents():
    """Test example: two random agents playing against each other."""
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env = EssenceWarsParallelEnv()

    num_games = 5
    results = {"player_0": 0, "player_1": 0, "draw": 0}

    for game_idx in range(num_games):
        observations, infos = env.reset(seed=game_idx)

        while env.agents:
            # Both agents select random actions (but only active one matters)
            actions = {}
            for agent in env.agents:
                mask = infos[agent]["action_mask"]
                legal_actions = np.where(mask)[0]
                if len(legal_actions) > 0:
                    actions[agent] = int(np.random.choice(legal_actions))

            if not actions:
                break

            observations, rewards, terminations, truncations, infos = env.step(actions)

        # Determine winner
        if rewards["player_0"] > 0:
            results["player_0"] += 1
        elif rewards["player_1"] > 0:
            results["player_1"] += 1
        else:
            results["draw"] += 1

    # Games should complete
    assert sum(results.values()) == num_games
    print(f"Results after {num_games} games: {results}")


def test_parallel_env_close():
    """Test that environment can be closed cleanly."""
    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env = EssenceWarsParallelEnv()
    env.reset(seed=42)
    env.close()  # Should not raise


@pytest.mark.skipif(
    not pytest.importorskip("pettingzoo", reason="PettingZoo not installed"),
    reason="PettingZoo not available"
)
def test_parallel_env_api_compliance():
    """Test PettingZoo API compliance using official test utilities."""
    from pettingzoo.test import parallel_api_test

    from essence_wars.parallel_env import EssenceWarsParallelEnv

    env = EssenceWarsParallelEnv()

    # This will raise if API is not compliant
    try:
        parallel_api_test(env, num_cycles=100)
    except Exception as e:
        pytest.fail(f"PettingZoo API test failed: {e}")
