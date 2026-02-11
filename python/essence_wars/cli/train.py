"""Training commands for Essence Wars."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import click


def create_train_group() -> click.Group:
    """Create the train command group."""
    import click

    from .utils import run_training_script

    @click.group()
    def train() -> None:
        """Train ML agents for Essence Wars.

        \b
        Available agents:
          ppo                 - Proximal Policy Optimization
          alphazero           - AlphaZero (MCTS + neural network)
          behavioral-cloning  - Supervised learning from MCTS data
          card2vec            - Card embedding model
          decision-transformer - Sequence modeling approach
        """

    @train.command("ppo")
    @click.option("--timesteps", "-t", default=500_000, help="Total training timesteps")
    @click.option("--num-envs", default=64, help="Number of parallel environments")
    @click.option("--lr", default=3e-4, help="Learning rate")
    @click.option("--gamma", default=0.99, help="Discount factor")
    @click.option("--ent-coef", default=0.02, help="Entropy coefficient")
    @click.option("--hidden-dim", default=256, help="Hidden layer dimension")
    @click.option(
        "--observation-mode",
        type=click.Choice(["flat", "embedded", "embedded_pretrained"]),
        default="flat",
        help="Observation mode",
    )
    @click.option(
        "--player-faction",
        type=click.Choice(["argentum", "symbiote", "obsidion"]),
        default=None,
        help="Train as faction specialist",
    )
    @click.option("--player-deck", default=None, help="Train with specific deck")
    @click.option("--reward-shaping", is_flag=True, help="Enable dense reward shaping")
    @click.option("--eval-interval", default=25_000, help="Evaluation interval")
    @click.option("--save-path", type=click.Path(), default=None, help="Save directory")
    @click.option(
        "--load", type=click.Path(exists=True), default=None, help="Load checkpoint"
    )
    @click.option("--seed", default=42, help="Random seed")
    @click.option("--device", default="auto", help="Device (cpu, cuda, auto)")
    @click.option("--no-tensorboard", is_flag=True, help="Disable TensorBoard logging")
    def train_ppo(**kwargs: Any) -> None:
        """Train a PPO (Proximal Policy Optimization) agent.

        \b
        Examples:
          essence-wars train ppo --timesteps 300000
          essence-wars train ppo --player-faction argentum --timesteps 500000
          essence-wars train ppo --observation-mode embedded --embed-dim 64
        """
        run_training_script("train_ppo", kwargs)

    @train.command("alphazero")
    @click.option(
        "--iterations", "-i", default=100, help="Number of training iterations"
    )
    @click.option("--games-per-iter", default=100, help="Self-play games per iteration")
    @click.option("--training-steps", default=100, help="Training steps per iteration")
    @click.option("--batch-size", default=256, help="Training batch size")
    @click.option("--sims", default=100, help="MCTS simulations per move")
    @click.option("--c-puct", default=1.5, help="MCTS exploration constant")
    @click.option("--hidden-dim", default=256, help="Network hidden dimension")
    @click.option("--lr", default=1e-3, help="Learning rate")
    @click.option("--save-path", type=click.Path(), default=None, help="Save directory")
    @click.option(
        "--load", type=click.Path(exists=True), default=None, help="Load checkpoint"
    )
    @click.option(
        "--bc-data",
        type=click.Path(exists=True),
        default=None,
        help="BC data for warm-start (prevents catastrophic forgetting)",
    )
    @click.option("--bc-ratio", default=0.7, help="Ratio of BC data in training batches")
    @click.option("--seed", default=42, help="Random seed")
    @click.option("--device", default="auto", help="Device (cpu, cuda, auto)")
    def train_alphazero(**kwargs: Any) -> None:
        """Train an AlphaZero agent using MCTS self-play.

        AlphaZero combines Monte Carlo Tree Search with a neural network
        that predicts both policy (move probabilities) and value (win probability).

        \b
        Examples:
          essence-wars train alphazero --iterations 100
          essence-wars train alphazero --sims 200 --games-per-iter 200
          essence-wars train alphazero --load model.pt --bc-data data.jsonl.gz
        """
        run_training_script("train_alphazero", kwargs)

    @train.command("behavioral-cloning")
    @click.option(
        "--data",
        "-d",
        type=click.Path(exists=True),
        required=True,
        help="Training data file (JSONL or JSONL.gz)",
    )
    @click.option("--epochs", "-e", default=10, help="Number of training epochs")
    @click.option("--batch-size", default=256, help="Training batch size")
    @click.option("--lr", default=1e-3, help="Learning rate")
    @click.option("--hidden-dim", default=256, help="Network hidden dimension")
    @click.option("--save-path", type=click.Path(), default=None, help="Save directory")
    @click.option("--seed", default=42, help="Random seed")
    @click.option("--device", default="auto", help="Device (cpu, cuda, auto)")
    def train_bc(**kwargs: Any) -> None:
        """Train a Behavioral Cloning agent from MCTS demonstration data.

        Supervised learning approach that imitates MCTS policy decisions.
        Fast to train and provides a good baseline.

        \b
        Examples:
          essence-wars train behavioral-cloning --data mcts_data.jsonl.gz
          essence-wars train behavioral-cloning --data mcts_data.jsonl.gz --epochs 20
        """
        run_training_script("train_behavioral_cloning", kwargs)

    @train.command("card2vec")
    @click.option(
        "--data",
        "-d",
        type=click.Path(exists=True),
        required=True,
        help="Training data file",
    )
    @click.option("--embed-dim", default=64, help="Embedding dimension")
    @click.option("--epochs", "-e", default=50, help="Number of training epochs")
    @click.option("--batch-size", default=512, help="Training batch size")
    @click.option("--lr", default=1e-3, help="Learning rate")
    @click.option("--save-path", type=click.Path(), default=None, help="Save directory")
    @click.option("--seed", default=42, help="Random seed")
    @click.option("--device", default="auto", help="Device (cpu, cuda, auto)")
    def train_card2vec(**kwargs: Any) -> None:
        """Train card embeddings using Card2Vec approach.

        Learns dense vector representations for cards based on co-occurrence
        patterns in gameplay. Can be used with PPO's embedded observation mode.

        \b
        Examples:
          essence-wars train card2vec --data mcts_data.jsonl.gz --embed-dim 64
        """
        run_training_script("train_card2vec", kwargs)

    @train.command("decision-transformer")
    @click.option(
        "--data",
        "-d",
        type=click.Path(exists=True),
        required=True,
        help="Training data file",
    )
    @click.option("--epochs", "-e", default=10, help="Number of training epochs")
    @click.option("--batch-size", default=64, help="Training batch size")
    @click.option("--context-length", default=20, help="Sequence context length")
    @click.option("--hidden-dim", default=128, help="Transformer hidden dimension")
    @click.option("--num-layers", default=4, help="Number of transformer layers")
    @click.option("--num-heads", default=4, help="Number of attention heads")
    @click.option("--lr", default=1e-4, help="Learning rate")
    @click.option("--save-path", type=click.Path(), default=None, help="Save directory")
    @click.option("--seed", default=42, help="Random seed")
    @click.option("--device", default="auto", help="Device (cpu, cuda, auto)")
    def train_dt(**kwargs: Any) -> None:
        """Train a Decision Transformer agent.

        Sequence modeling approach that learns to predict actions conditioned
        on desired returns (reward-to-go).

        \b
        Examples:
          essence-wars train decision-transformer --data mcts_data.jsonl.gz
          essence-wars train decision-transformer --context-length 30 --num-layers 6
        """
        run_training_script("train_decision_transformer", kwargs)

    return train
