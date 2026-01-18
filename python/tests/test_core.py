"""Tests for the Essence Wars Rust bindings."""

import numpy as np


def test_import():
    """Test that the package imports correctly."""
    from essence_wars import ACTION_SPACE_SIZE, STATE_TENSOR_SIZE

    assert STATE_TENSOR_SIZE == 326
    assert ACTION_SPACE_SIZE == 256


def test_pygame_creation():
    """Test PyGame can be created with default parameters."""
    from essence_wars import PyGame

    game = PyGame()
    assert game is not None


def test_pygame_reset():
    """Test PyGame reset produces valid tensors."""
    from essence_wars import ACTION_SPACE_SIZE, STATE_TENSOR_SIZE, PyGame

    game = PyGame()
    game.reset(seed=42)

    obs = game.observe()
    assert obs.shape == (STATE_TENSOR_SIZE,)
    assert obs.dtype == np.float32

    mask = game.action_mask()
    assert mask.shape == (ACTION_SPACE_SIZE,)
    assert mask.dtype == np.float32
    assert np.sum(mask) > 0  # At least one legal action


def test_pygame_step():
    """Test PyGame step executes actions correctly."""
    from essence_wars import PyGame

    game = PyGame()
    game.reset(seed=42)

    # Get a legal action
    mask = game.action_mask()
    legal_actions = np.where(mask > 0)[0]
    assert len(legal_actions) > 0

    action = legal_actions[0]
    reward, done = game.step(action)

    assert isinstance(reward, float)
    assert isinstance(done, bool)


def test_pygame_full_game():
    """Test playing a full game with random actions."""
    from essence_wars import PyGame

    game = PyGame()
    game.reset(seed=42)

    steps = 0
    max_steps = 1000

    while not game.is_done() and steps < max_steps:
        mask = game.action_mask()
        legal_actions = np.where(mask > 0)[0]
        action = np.random.choice(legal_actions)
        reward, done = game.step(action)
        steps += 1

    assert game.is_done() or steps == max_steps
    assert steps > 0


def test_pygame_greedy_action():
    """Test GreedyBot action selection."""
    from essence_wars import PyGame

    game = PyGame()
    game.reset(seed=42)

    action = game.greedy_action()
    assert 0 <= action < 256

    mask = game.action_mask()
    assert mask[action] > 0  # Action should be legal


def test_pygame_random_action():
    """Test RandomBot action selection."""
    from essence_wars import PyGame

    game = PyGame()
    game.reset(seed=42)

    action = game.random_action()
    assert 0 <= action < 256

    mask = game.action_mask()
    assert mask[action] > 0  # Action should be legal


def test_pygame_fork():
    """Test game state cloning for MCTS."""
    from essence_wars import PyGame

    game = PyGame()
    game.reset(seed=42)

    # Take a few actions
    for _ in range(5):
        if game.is_done():
            break
        mask = game.action_mask()
        legal_actions = np.where(mask > 0)[0]
        game.step(legal_actions[0])

    # Fork the game
    forked = game.fork()

    # Original and fork should have same state
    assert np.allclose(game.observe(), forked.observe())
    assert np.allclose(game.action_mask(), forked.action_mask())

    # Take different actions
    if not game.is_done():
        mask = game.action_mask()
        legal_actions = np.where(mask > 0)[0]
        if len(legal_actions) > 1:
            game.step(legal_actions[0])
            forked.step(legal_actions[1])

            # Now they should differ
            assert not np.allclose(game.observe(), forked.observe())


def test_pygame_deck_selection():
    """Test creating game with specific decks."""
    from essence_wars import PyGame

    game = PyGame(deck1="architect_fortify", deck2="broodmother_swarm")
    game.reset(seed=42)

    assert not game.is_done()


def test_pygame_list_decks():
    """Test listing available decks."""
    from essence_wars import PyGame

    decks = PyGame.list_decks()
    assert isinstance(decks, list)
    assert len(decks) > 0
    assert "architect_fortify" in decks  # Updated to use actual deck name


def test_parallel_games_creation():
    """Test PyParallelGames can be created."""
    from essence_wars import PyParallelGames

    games = PyParallelGames(num_envs=4)
    assert games.num_envs == 4


def test_parallel_games_reset():
    """Test PyParallelGames reset."""
    from essence_wars import ACTION_SPACE_SIZE, STATE_TENSOR_SIZE, PyParallelGames

    num_envs = 4
    games = PyParallelGames(num_envs=num_envs)
    games.reset(seeds=[1, 2, 3, 4])

    obs = games.observe_batch()
    assert obs.shape == (num_envs, STATE_TENSOR_SIZE)
    assert obs.dtype == np.float32

    mask = games.action_mask_batch()
    assert mask.shape == (num_envs, ACTION_SPACE_SIZE)
    assert mask.dtype == np.float32


def test_parallel_games_step():
    """Test PyParallelGames step_batch."""
    from essence_wars import PyParallelGames

    num_envs = 4
    games = PyParallelGames(num_envs=num_envs)
    games.reset(seeds=[1, 2, 3, 4])

    # Get legal actions for each env
    mask = games.action_mask_batch()
    actions = np.array([np.where(mask[i] > 0)[0][0] for i in range(num_envs)], dtype=np.uint8)

    rewards, dones = games.step_batch(actions)

    assert rewards.shape == (num_envs,)
    assert dones.shape == (num_envs,)
    assert rewards.dtype == np.float32
    assert dones.dtype == bool


def test_game_mode_attrition():
    """Test attrition game mode (default)."""
    from essence_wars import PyGame

    game = PyGame(game_mode="attrition")
    game.reset(seed=42)
    assert not game.is_done()


def test_game_mode_essence_duel():
    """Test essence duel game mode."""
    from essence_wars import PyGame

    game = PyGame(game_mode="essence_duel")
    game.reset(seed=42)
    assert not game.is_done()
