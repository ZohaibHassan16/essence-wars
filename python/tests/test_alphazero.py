"""Tests for AlphaZero agent components."""

import numpy as np
import torch


def test_residual_block():
    """Test ResidualBlock forward pass."""
    from essence_wars.agents.networks import ResidualBlock

    block = ResidualBlock(hidden_dim=64)
    x = torch.randn(8, 64)

    out = block(x)

    assert out.shape == (8, 64)
    # Output should differ from input due to residual connection
    assert not torch.allclose(out, x)


def test_alphazero_network_creation():
    """Test AlphaZeroNetwork can be created."""
    from essence_wars.agents.networks import AlphaZeroNetwork

    network = AlphaZeroNetwork()

    assert network is not None
    assert network.obs_dim == 326
    assert network.action_dim == 256
    assert network.hidden_dim == 256
    assert network.num_blocks == 4


def test_alphazero_network_forward():
    """Test AlphaZeroNetwork forward pass."""
    from essence_wars.agents.networks import AlphaZeroNetwork

    network = AlphaZeroNetwork(hidden_dim=128, num_blocks=2)
    network.eval()

    batch_size = 8
    obs = torch.randn(batch_size, 326)
    mask = torch.ones(batch_size, 256, dtype=torch.bool)

    logits, value = network(obs, mask)

    assert logits.shape == (batch_size, 256)
    assert value.shape == (batch_size,)
    # Value should be in [-1, 1] due to tanh
    assert (value >= -1).all() and (value <= 1).all()


def test_alphazero_network_action_masking():
    """Test that action masking works correctly."""
    from essence_wars.agents.networks import AlphaZeroNetwork

    network = AlphaZeroNetwork()
    network.eval()

    obs = torch.randn(1, 326)

    # Mask all but one action
    mask = torch.zeros(1, 256, dtype=torch.bool)
    mask[0, 42] = True

    with torch.no_grad():
        logits, _ = network(obs, mask)
        probs = torch.softmax(logits, dim=-1)

    # Action 42 should have probability ~1.0
    assert probs[0, 42] > 0.99


def test_alphazero_network_get_policy():
    """Test getting policy probabilities."""
    from essence_wars.agents.networks import AlphaZeroNetwork

    network = AlphaZeroNetwork()
    network.eval()

    obs = torch.randn(8, 326)
    mask = torch.ones(8, 256, dtype=torch.bool)

    with torch.no_grad():
        policy = network.get_policy(obs, mask)

    assert policy.shape == (8, 256)
    # Should be valid probability distribution
    assert torch.allclose(policy.sum(dim=-1), torch.ones(8), atol=1e-5)
    assert (policy >= 0).all()


def test_alphazero_network_evaluate():
    """Test evaluate method for MCTS."""
    from essence_wars.agents.networks import AlphaZeroNetwork

    network = AlphaZeroNetwork()
    network.eval()

    obs = torch.randn(4, 326)
    mask = torch.ones(4, 256, dtype=torch.bool)

    with torch.no_grad():
        policy, value = network.evaluate(obs, mask)

    assert policy.shape == (4, 256)
    assert value.shape == (4,)
    assert torch.allclose(policy.sum(dim=-1), torch.ones(4), atol=1e-5)


def test_mcts_node_creation():
    """Test MCTSNode creation."""
    from essence_wars.agents.alphazero import MCTSNode

    node = MCTSNode(prior=0.5)

    assert node.prior == 0.5
    assert node.visit_count == 0
    assert node.value_sum == 0.0
    assert node.value == 0.0
    assert not node.is_expanded


def test_mcts_node_expand():
    """Test MCTSNode expansion."""
    from essence_wars.agents.alphazero import MCTSNode

    node = MCTSNode()
    legal_actions = [0, 5, 10, 20]
    priors = np.zeros(256, dtype=np.float32)
    priors[0] = 0.4
    priors[5] = 0.3
    priors[10] = 0.2
    priors[20] = 0.1

    node.expand(legal_actions, priors)

    assert node.is_expanded
    assert len(node.children) == 4
    assert set(node.children.keys()) == {0, 5, 10, 20}

    # Check priors are normalized
    total_prior = sum(child.prior for child in node.children.values())
    assert abs(total_prior - 1.0) < 1e-5


def test_mcts_node_select_child():
    """Test UCB selection."""
    from essence_wars.agents.alphazero import MCTSNode

    node = MCTSNode()
    legal_actions = [0, 1]
    priors = np.zeros(256, dtype=np.float32)
    priors[0] = 0.8
    priors[1] = 0.2

    node.expand(legal_actions, priors)

    # Initially, should select action with higher prior
    action, child = node.select_child(c_puct=1.5)
    assert action == 0  # Higher prior

    # After visiting action 0 many times, action 1 becomes more attractive
    node.children[0].visit_count = 100
    node.children[0].value_sum = 50  # Q = 0.5
    node.visit_count = 100

    action, child = node.select_child(c_puct=1.5)
    assert action == 1  # Unexplored action now more attractive


def test_mcts_node_add_noise():
    """Test Dirichlet noise addition."""
    from essence_wars.agents.alphazero import MCTSNode

    node = MCTSNode()
    legal_actions = [0, 1, 2, 3]
    priors = np.array([0.25, 0.25, 0.25, 0.25] + [0.0] * 252, dtype=np.float32)

    node.expand(legal_actions, priors)
    original_priors = [node.children[a].prior for a in legal_actions]

    node.add_exploration_noise(dirichlet_alpha=0.3, epsilon=0.25)
    noisy_priors = [node.children[a].prior for a in legal_actions]

    # Priors should have changed
    assert not all(abs(o - n) < 1e-6 for o, n in zip(original_priors, noisy_priors))


def test_replay_buffer():
    """Test ReplayBuffer operations."""
    from essence_wars.agents.alphazero import ReplayBuffer

    buffer = ReplayBuffer(capacity=100)

    # Add samples
    for i in range(50):
        obs = np.random.randn(326).astype(np.float32)
        mask = np.ones(256, dtype=np.float32)
        policy = np.random.dirichlet(np.ones(256)).astype(np.float32)
        value = np.random.uniform(-1, 1)
        buffer.add(obs, mask, policy, value)

    assert len(buffer) == 50

    # Sample batch
    obs, masks, policies, values = buffer.sample(16)

    assert obs.shape == (16, 326)
    assert masks.shape == (16, 256)
    assert policies.shape == (16, 256)
    assert values.shape == (16,)


def test_replay_buffer_add_game():
    """Test adding a full game to replay buffer."""
    from essence_wars.agents.alphazero import ReplayBuffer

    buffer = ReplayBuffer(capacity=1000)

    # Simulate a 10-move game
    observations = [np.random.randn(326).astype(np.float32) for _ in range(10)]
    masks = [np.ones(256, dtype=np.float32) for _ in range(10)]
    policies = [np.random.dirichlet(np.ones(256)).astype(np.float32) for _ in range(10)]
    outcome = 1.0  # Player 0 won

    buffer.add_game(observations, masks, policies, outcome)

    assert len(buffer) == 10


def test_alphazero_config():
    """Test AlphaZeroConfig creation."""
    from essence_wars.agents.alphazero import AlphaZeroConfig

    config = AlphaZeroConfig(
        num_simulations=50,
        num_iterations=10,
        games_per_iteration=20,
    )

    assert config.num_simulations == 50
    assert config.num_iterations == 10
    assert config.games_per_iteration == 20


def test_neural_mcts_creation():
    """Test NeuralMCTS creation."""
    from essence_wars.agents.alphazero import AlphaZeroConfig, NeuralMCTS
    from essence_wars.agents.networks import AlphaZeroNetwork

    network = AlphaZeroNetwork()
    config = AlphaZeroConfig(num_simulations=10)
    mcts = NeuralMCTS(network, config)

    assert mcts.network is network
    assert mcts.config.num_simulations == 10


def test_neural_mcts_search():
    """Test MCTS search produces valid action probabilities."""
    from essence_wars._core import PyGame
    from essence_wars.agents.alphazero import AlphaZeroConfig, NeuralMCTS
    from essence_wars.agents.networks import AlphaZeroNetwork

    network = AlphaZeroNetwork()
    config = AlphaZeroConfig(num_simulations=10)  # Few sims for speed
    mcts = NeuralMCTS(network, config)

    game = PyGame()
    game.reset(seed=42)

    action_probs = mcts.search(game, add_noise=False)

    assert action_probs.shape == (256,)
    assert abs(action_probs.sum() - 1.0) < 1e-5
    assert (action_probs >= 0).all()

    # Probs should be concentrated on legal actions
    mask = game.action_mask()
    illegal_prob = action_probs[mask == 0].sum()
    assert illegal_prob < 1e-5


def test_alphazero_trainer_creation():
    """Test AlphaZeroTrainer can be created."""
    from essence_wars.agents.alphazero import AlphaZeroConfig, AlphaZeroTrainer

    config = AlphaZeroConfig(num_simulations=10)
    trainer = AlphaZeroTrainer(config=config)

    assert trainer is not None
    assert trainer.network is not None
    assert trainer.mcts is not None
    assert trainer.replay_buffer is not None


def test_alphazero_self_play_game():
    """Test self-play game generation."""
    from essence_wars.agents.alphazero import AlphaZeroConfig, AlphaZeroTrainer

    config = AlphaZeroConfig(num_simulations=5)  # Very few for speed
    trainer = AlphaZeroTrainer(config=config)

    observations, masks, policies, outcome = trainer.self_play_game(seed=42)

    assert len(observations) > 0
    assert len(observations) == len(masks) == len(policies)
    assert outcome in [-1.0, 1.0]

    # Check shapes
    assert observations[0].shape == (326,)
    assert masks[0].shape == (256,)
    assert policies[0].shape == (256,)


def test_alphazero_train_step():
    """Test a single training step."""
    from essence_wars.agents.alphazero import AlphaZeroConfig, AlphaZeroTrainer

    config = AlphaZeroConfig(
        num_simulations=5,
        batch_size=16,
        min_replay_size=10,
    )
    trainer = AlphaZeroTrainer(config=config)

    # Generate some data
    for _ in range(3):
        obs, masks, policies, outcome = trainer.self_play_game()
        trainer.replay_buffer.add_game(obs, masks, policies, outcome)

    # Train step
    loss_info = trainer.train_step()

    assert "policy_loss" in loss_info
    assert "value_loss" in loss_info
    assert "total_loss" in loss_info


def test_alphazero_evaluate_vs_greedy():
    """Test evaluation against greedy bot."""
    from essence_wars.agents.alphazero import AlphaZeroConfig, AlphaZeroTrainer

    config = AlphaZeroConfig(num_simulations=5)
    trainer = AlphaZeroTrainer(config=config)

    # Evaluate untrained network
    win_rate = trainer.evaluate_vs_greedy(num_games=5)

    assert 0.0 <= win_rate <= 1.0


def test_alphazero_evaluate_vs_random():
    """Test evaluation against random bot."""
    from essence_wars.agents.alphazero import AlphaZeroConfig, AlphaZeroTrainer

    config = AlphaZeroConfig(num_simulations=5)
    trainer = AlphaZeroTrainer(config=config)

    win_rate = trainer.evaluate_vs_random(num_games=5)

    assert 0.0 <= win_rate <= 1.0


def test_alphazero_save_load(tmp_path):
    """Test saving and loading checkpoints."""
    from essence_wars.agents.alphazero import AlphaZeroConfig, AlphaZeroTrainer

    config = AlphaZeroConfig(num_simulations=5)
    trainer1 = AlphaZeroTrainer(config=config)

    # Generate some data and train briefly
    obs, masks, policies, outcome = trainer1.self_play_game(seed=42)
    trainer1.replay_buffer.add_game(obs, masks, policies, outcome)
    if len(trainer1.replay_buffer) >= config.batch_size:
        trainer1.train_step()

    # Save
    save_path = str(tmp_path / "test_alphazero.pt")
    trainer1.save(save_path)

    # Load into new trainer
    trainer2 = AlphaZeroTrainer(config=config)
    trainer2.load(save_path)

    # Check weights match
    for p1, p2 in zip(trainer1.network.parameters(), trainer2.network.parameters()):
        assert torch.allclose(p1, p2)


def test_alphazero_short_training():
    """Test a very short training run."""
    from essence_wars.agents.alphazero import AlphaZeroConfig, AlphaZeroTrainer

    config = AlphaZeroConfig(
        num_iterations=2,
        games_per_iteration=3,
        training_steps_per_iteration=5,
        num_simulations=5,
        batch_size=8,
        min_replay_size=5,
        eval_interval=100,  # Don't eval during test
    )
    trainer = AlphaZeroTrainer(config=config)

    results = trainer.train()

    assert results["iterations"] == 2
    assert results["total_games"] == 6
    assert "final_win_rate" in results
