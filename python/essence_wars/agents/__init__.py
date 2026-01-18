"""
Reinforcement Learning Agents for Essence Wars.

This module provides neural network architectures and training algorithms
for learning to play Essence Wars.

Available Agents:
- PPO: Proximal Policy Optimization with action masking
- AlphaZero: MCTS + neural network self-play

Example:
    from essence_wars.agents import EssenceWarsNetwork, PPOTrainer

    # Create and train a PPO agent
    trainer = PPOTrainer(num_envs=64)
    trainer.train(total_timesteps=1_000_000)

    # Evaluate against GreedyBot
    win_rate = trainer.evaluate_vs_greedy(num_games=100)
    print(f"Win rate: {win_rate:.1%}")

    # Or use AlphaZero
    from essence_wars.agents import AlphaZeroNetwork, AlphaZeroTrainer

    trainer = AlphaZeroTrainer()
    trainer.train(num_iterations=100)
"""

from essence_wars.agents.networks import AlphaZeroNetwork, EssenceWarsNetwork, ResidualBlock

__all__ = [
    "AlphaZeroNetwork",
    "EssenceWarsNetwork",
    "ResidualBlock",
]

# Lazy imports for optional components (trainers have heavier dependencies)
def __getattr__(name: str):
    if name == "PPOTrainer":
        from essence_wars.agents.ppo import PPOTrainer
        return PPOTrainer
    if name == "AlphaZeroTrainer":
        from essence_wars.agents.alphazero import AlphaZeroTrainer
        return AlphaZeroTrainer
    if name == "NeuralMCTS":
        from essence_wars.agents.alphazero import NeuralMCTS
        return NeuralMCTS
    raise AttributeError(f"module 'essence_wars.agents' has no attribute {name!r}")
