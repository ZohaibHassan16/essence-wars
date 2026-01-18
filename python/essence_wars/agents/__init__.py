"""
Reinforcement Learning Agents for Essence Wars.

This module provides neural network architectures and training algorithms
for learning to play Essence Wars.

Available Agents:
- PPO: Proximal Policy Optimization with action masking
- AlphaZero: MCTS + neural network (future)

Example:
    from essence_wars.agents import EssenceWarsNetwork, PPOTrainer

    # Create and train a PPO agent
    trainer = PPOTrainer(num_envs=64)
    trainer.train(total_timesteps=1_000_000)

    # Evaluate against GreedyBot
    win_rate = trainer.evaluate_vs_greedy(num_games=100)
    print(f"Win rate: {win_rate:.1%}")
"""

from essence_wars.agents.networks import EssenceWarsNetwork

__all__ = [
    "EssenceWarsNetwork",
]

# Lazy imports for optional components
def __getattr__(name: str):
    if name == "PPOTrainer":
        from essence_wars.agents.ppo import PPOTrainer
        return PPOTrainer
    raise AttributeError(f"module 'essence_wars.agents' has no attribute {name!r}")
