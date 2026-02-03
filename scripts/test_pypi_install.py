#!/usr/bin/env python3
"""
Test script to verify the essence-wars PyPI package works correctly.

Usage:
    # First, install from PyPI in a fresh environment:
    pip install essence-wars

    # Then run this script:
    python test_pypi_install.py
"""

import sys


def test_import_and_version():
    """Test basic import and version."""
    print("=" * 60)
    print("Test 1: Import and Version")
    print("=" * 60)

    import essence_wars
    print(f"✓ Imported essence_wars")
    print(f"✓ Version: {essence_wars.__version__}")

    from essence_wars import PyGame, PyParallelGames, STATE_TENSOR_SIZE, ACTION_SPACE_SIZE
    print(f"✓ STATE_TENSOR_SIZE: {STATE_TENSOR_SIZE}")
    print(f"✓ ACTION_SPACE_SIZE: {ACTION_SPACE_SIZE}")

    return True


def test_list_decks():
    """Test listing available decks."""
    print("\n" + "=" * 60)
    print("Test 2: List Decks")
    print("=" * 60)

    from essence_wars import list_decks

    decks = list_decks()
    print(f"✓ Found {len(decks)} decks:")
    for deck in decks[:6]:  # Show first 6
        print(f"  - {deck}")
    if len(decks) > 6:
        print(f"  ... and {len(decks) - 6} more")

    return len(decks) == 12  # Expected: 12 commander decks


def test_single_game():
    """Test running a single game."""
    print("\n" + "=" * 60)
    print("Test 3: Single Game")
    print("=" * 60)

    from essence_wars import PyGame
    import numpy as np

    game = PyGame()
    game.reset(seed=42)
    print("✓ Created and reset game with seed 42")

    # Get observation
    obs = game.observe()
    assert isinstance(obs, np.ndarray), "Observation should be numpy array"
    assert obs.shape == (328,), f"Expected shape (328,), got {obs.shape}"
    print(f"✓ Observation shape: {obs.shape}")

    # Get action mask
    mask = game.action_mask()
    assert isinstance(mask, np.ndarray), "Action mask should be numpy array"
    assert mask.shape == (256,), f"Expected shape (256,), got {mask.shape}"
    legal_count = int(mask.sum())
    print(f"✓ Action mask shape: {mask.shape}, legal actions: {legal_count}")

    # Play a few random moves
    steps = 0
    done = False
    while not done and steps < 100:
        legal_actions = np.where(mask > 0)[0]
        action = np.random.choice(legal_actions)
        reward, done = game.step(int(action))
        mask = game.action_mask()
        steps += 1

    print(f"✓ Played {steps} steps, game over: {done}")

    return True


def test_parallel_games():
    """Test parallel game execution."""
    print("\n" + "=" * 60)
    print("Test 4: Parallel Games")
    print("=" * 60)

    from essence_wars import PyParallelGames
    import numpy as np

    num_envs = 4
    games = PyParallelGames(num_envs)
    games.reset_all(base_seed=123)
    print(f"✓ Created {num_envs} parallel games")

    # Get batch observations
    obs_batch = games.observe_all()
    assert obs_batch.shape == (num_envs, 328), f"Expected ({num_envs}, 328), got {obs_batch.shape}"
    print(f"✓ Batch observation shape: {obs_batch.shape}")

    # Get batch masks
    mask_batch = games.action_masks_all()
    assert mask_batch.shape == (num_envs, 256), f"Expected ({num_envs}, 256), got {mask_batch.shape}"
    print(f"✓ Batch action mask shape: {mask_batch.shape}")

    # Step all with random actions
    actions = []
    for i in range(num_envs):
        legal = np.where(mask_batch[i] > 0)[0]
        actions.append(int(np.random.choice(legal)))

    rewards, dones = games.step_all(actions)
    print(f"✓ Stepped all games, rewards: {rewards}, dones: {dones}")

    return True


def test_gymnasium_env():
    """Test Gymnasium environment wrapper."""
    print("\n" + "=" * 60)
    print("Test 5: Gymnasium Environment")
    print("=" * 60)

    try:
        from essence_wars.env import EssenceWarsEnv
        import numpy as np
    except ImportError as e:
        print(f"⚠ Skipped (gymnasium not installed): {e}")
        return True  # Not a failure, just optional

    env = EssenceWarsEnv(deck="architect_fortify")
    print("✓ Created EssenceWarsEnv")

    obs, info = env.reset(seed=42)
    assert obs.shape == (328,), f"Expected (328,), got {obs.shape}"
    print(f"✓ Reset environment, obs shape: {obs.shape}")

    # Play a few steps
    done = False
    steps = 0
    while not done and steps < 50:
        action = env.action_space.sample()
        # Ensure action is legal
        mask = info.get("action_mask", env.unwrapped.game.action_mask())
        if mask[action] == 0:
            legal = np.where(mask > 0)[0]
            action = int(np.random.choice(legal))

        obs, reward, terminated, truncated, info = env.step(action)
        done = terminated or truncated
        steps += 1

    print(f"✓ Played {steps} steps through Gymnasium API")
    env.close()

    return True


def test_determinism():
    """Test that games are deterministic with same seed."""
    print("\n" + "=" * 60)
    print("Test 6: Determinism")
    print("=" * 60)

    from essence_wars import PyGame
    import numpy as np

    def play_game(seed):
        game = PyGame()
        game.reset(seed=seed)

        actions_taken = []
        done = False
        while not done:
            mask = game.action_mask()
            legal = np.where(mask > 0)[0]
            # Always pick first legal action for determinism
            action = int(legal[0])
            actions_taken.append(action)
            _, done = game.step(action)
            if len(actions_taken) > 200:
                break

        return actions_taken

    # Play same seed twice
    actions1 = play_game(seed=777)
    actions2 = play_game(seed=777)

    assert actions1 == actions2, "Games with same seed should be identical!"
    print(f"✓ Determinism verified: {len(actions1)} actions match perfectly")

    return True


def main():
    """Run all tests."""
    print("\n" + "=" * 60)
    print("  essence-wars PyPI Package Test Suite")
    print("=" * 60)

    tests = [
        ("Import and Version", test_import_and_version),
        ("List Decks", test_list_decks),
        ("Single Game", test_single_game),
        ("Parallel Games", test_parallel_games),
        ("Gymnasium Env", test_gymnasium_env),
        ("Determinism", test_determinism),
    ]

    results = []
    for name, test_fn in tests:
        try:
            passed = test_fn()
            results.append((name, passed, None))
        except Exception as e:
            results.append((name, False, str(e)))
            print(f"✗ FAILED: {e}")

    # Summary
    print("\n" + "=" * 60)
    print("  Test Summary")
    print("=" * 60)

    passed = sum(1 for _, p, _ in results if p)
    total = len(results)

    for name, p, error in results:
        status = "✓ PASS" if p else f"✗ FAIL: {error}"
        print(f"  {name}: {status}")

    print(f"\n  {passed}/{total} tests passed")
    print("=" * 60)

    if passed == total:
        print("\n🎉 All tests passed! The package is working correctly.\n")
        return 0
    else:
        print("\n❌ Some tests failed. Please check the output above.\n")
        return 1


if __name__ == "__main__":
    sys.exit(main())
