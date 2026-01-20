#!/usr/bin/env python3
"""
Train AlphaZero agent for Essence Wars.

AlphaZero uses self-play with MCTS to generate training data,
then trains a neural network to predict MCTS policy and game outcomes.

Usage:
    uv run python python/scripts/train_alphazero.py --iterations 100

    # Quick test
    uv run python python/scripts/train_alphazero.py --iterations 5 --games-per-iter 10 --sims 25

    # Full training (TensorBoard enabled by default)
    uv run python python/scripts/train_alphazero.py --iterations 200

    # BC warm-start with dual buffer (prevents catastrophic forgetting)
    uv run python python/scripts/train_alphazero.py \
        --iterations 100 \
        --load models/bc_mcts_10k_best.pt \
        --bc-data data/datasets/mcts_10k_sims100_*.jsonl.gz \
        --bc-max-samples 100000 \
        --bc-ratio 0.7 \
        --lr 1e-4

    The dual buffer approach:
    - Maintains a FIXED BC ratio (e.g., 70%) throughout training
    - Separate buffers for BC and self-play data (never diluted)
    - Prevents the "delayed catastrophic forgetting" that occurs when
      BC samples get diluted below ~50% in a single mixed buffer
"""

import argparse
import sys
from datetime import datetime
from pathlib import Path

# Add parent to path for local development
sys.path.insert(0, str(Path(__file__).parent.parent))


def main():
    parser = argparse.ArgumentParser(
        description="Train AlphaZero agent for Essence Wars",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )

    # Training parameters
    parser.add_argument(
        "--iterations",
        type=int,
        default=100,
        help="Number of training iterations",
    )
    parser.add_argument(
        "--games-per-iter",
        type=int,
        default=100,
        help="Self-play games per iteration",
    )
    parser.add_argument(
        "--training-steps",
        type=int,
        default=100,
        help="Training steps per iteration",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=256,
        help="Training batch size",
    )

    # MCTS parameters
    parser.add_argument(
        "--sims",
        type=int,
        default=100,
        help="MCTS simulations per move",
    )
    parser.add_argument(
        "--c-puct",
        type=float,
        default=1.5,
        help="MCTS exploration constant",
    )

    # Network parameters
    parser.add_argument(
        "--hidden-dim",
        type=int,
        default=256,
        help="Network hidden dimension",
    )
    parser.add_argument(
        "--num-blocks",
        type=int,
        default=4,
        help="Number of residual blocks",
    )

    # Observation mode (embeddings)
    parser.add_argument(
        "--observation-mode",
        type=str,
        default="flat",
        choices=["flat", "embedded", "embedded_pretrained"],
        help="Observation mode: flat (326 floats), embedded (learn embeddings), "
             "embedded_pretrained (use pre-trained Card2Vec)",
    )
    parser.add_argument(
        "--embed-dim",
        type=int,
        default=64,
        help="Card embedding dimension (for embedded modes)",
    )
    parser.add_argument(
        "--pretrained-embeds",
        type=str,
        default=None,
        help="Path to pre-trained Card2Vec embeddings (for embedded_pretrained mode)",
    )

    # Learning parameters
    parser.add_argument(
        "--lr",
        type=float,
        default=1e-3,
        help="Learning rate",
    )
    parser.add_argument(
        "--weight-decay",
        type=float,
        default=1e-4,
        help="Weight decay",
    )

    # Evaluation
    parser.add_argument(
        "--eval-interval",
        type=int,
        default=5,
        help="Evaluate every N iterations",
    )
    parser.add_argument(
        "--eval-games",
        type=int,
        default=50,
        help="Games for evaluation",
    )

    # Replay buffer
    parser.add_argument(
        "--buffer-size",
        type=int,
        default=100_000,
        help="Replay buffer capacity",
    )
    parser.add_argument(
        "--min-buffer-size",
        type=int,
        default=1000,
        help="Minimum buffer size before training",
    )

    # Logging
    parser.add_argument(
        "--no-tensorboard",
        action="store_true",
        help="Disable TensorBoard logging (enabled by default)",
    )
    parser.add_argument(
        "--save-dir",
        type=str,
        default=None,
        help="Directory to save checkpoints (default: experiments/alphazero/YYYYMMDD_HHMMSS)",
    )
    parser.add_argument(
        "--checkpoint-interval",
        type=int,
        default=10,
        help="Save checkpoint every N iterations (0 = only at end)",
    )

    # Fine-tuning
    parser.add_argument(
        "--load",
        type=str,
        default=None,
        help="Load pre-trained model checkpoint (e.g., from behavioral cloning)",
    )

    # BC warm-start (dual buffer mode)
    parser.add_argument(
        "--bc-data",
        type=str,
        default=None,
        help="Path to BC dataset (JSONL/JSONL.gz) for dual buffer warm-start. "
             "This maintains a fixed BC ratio throughout training, preventing "
             "catastrophic forgetting when fine-tuning from BC.",
    )
    parser.add_argument(
        "--bc-max-samples",
        type=int,
        default=100_000,
        help="Maximum BC samples to load into dual buffer",
    )
    parser.add_argument(
        "--bc-ratio",
        type=float,
        default=0.7,
        help="Ratio of BC samples in training batches (0.7 = 70%% BC, 30%% self-play). "
             "Higher values anchor more strongly to BC policy.",
    )

    args = parser.parse_args()

    # Import here to avoid slow startup for --help
    from essence_wars.agents.alphazero import AlphaZeroConfig, AlphaZeroTrainer

    # Create config
    config = AlphaZeroConfig(
        num_iterations=args.iterations,
        games_per_iteration=args.games_per_iter,
        training_steps_per_iteration=args.training_steps,
        batch_size=args.batch_size,
        num_simulations=args.sims,
        c_puct=args.c_puct,
        hidden_dim=args.hidden_dim,
        num_blocks=args.num_blocks,
        learning_rate=args.lr,
        weight_decay=args.weight_decay,
        eval_interval=args.eval_interval,
        eval_games=args.eval_games,
        replay_buffer_size=args.buffer_size,
        min_replay_size=args.min_buffer_size,
        checkpoint_interval=args.checkpoint_interval,
    )

    # Setup save directory
    if args.save_dir is None:
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        save_dir = Path("experiments/alphazero") / timestamp
    else:
        save_dir = Path(args.save_dir)

    save_dir.mkdir(parents=True, exist_ok=True)

    # Setup TensorBoard (enabled by default)
    writer = None
    if not args.no_tensorboard:
        try:
            from torch.utils.tensorboard import SummaryWriter
            writer = SummaryWriter(log_dir=str(save_dir / "tensorboard"))
            print(f"TensorBoard logging to: {save_dir / 'tensorboard'}")
        except ImportError:
            print("Warning: tensorboard not installed, skipping logging")

    # Print configuration
    print("=" * 60)
    print("Essence Wars AlphaZero Training")
    print("=" * 60)
    print(f"  Iterations:      {config.num_iterations:,}")
    print(f"  Games/iteration: {config.games_per_iteration}")
    print(f"  MCTS sims/move:  {config.num_simulations}")
    print(f"  Batch size:      {config.batch_size}")
    print(f"  Learning rate:   {config.learning_rate}")
    print(f"  Hidden dim:      {config.hidden_dim}")
    print(f"  Residual blocks: {config.num_blocks}")
    print(f"  Obs mode:        {args.observation_mode}")
    if args.observation_mode != "flat":
        print(f"  Embed dim:       {args.embed_dim}")
    if args.pretrained_embeds:
        print(f"  Pretrained:      {args.pretrained_embeds}")
    print(f"  Device:          {config.device}")
    print("=" * 60)
    print(f"Saving to: {save_dir}")

    # Create network based on observation mode
    import torch
    network = None
    if args.observation_mode == "flat":
        # Use default AlphaZeroNetwork (created by trainer)
        pass
    elif args.observation_mode in ("embedded", "embedded_pretrained"):
        from essence_wars.agents.embeddings import EmbeddedAlphaZeroNetwork

        pretrained_embeds = None
        if args.observation_mode == "embedded_pretrained":
            if not args.pretrained_embeds:
                raise ValueError("--pretrained-embeds required for embedded_pretrained mode")
            print(f"\nLoading Card2Vec embeddings from: {args.pretrained_embeds}")
            checkpoint = torch.load(args.pretrained_embeds, map_location=config.device)
            if isinstance(checkpoint, dict) and "embeddings" in checkpoint:
                pretrained_embeds = checkpoint["embeddings"]
            else:
                pretrained_embeds = checkpoint
            print(f"  Embedding shape: {pretrained_embeds.shape}")

        network = EmbeddedAlphaZeroNetwork(
            embed_dim=args.embed_dim,
            hidden_dim=args.hidden_dim,
            num_blocks=args.num_blocks,
            pretrained_embeds=pretrained_embeds,
        )

    # Create trainer
    trainer = AlphaZeroTrainer(config=config, network=network, writer=writer)

    # Load pre-trained model if specified
    if args.load:
        print(f"\nLoading pre-trained model from: {args.load}")
        checkpoint = torch.load(args.load, map_location=config.device, weights_only=False)

        # Handle both AlphaZero checkpoints and BC checkpoints
        if "network_state_dict" in checkpoint:
            # AlphaZero checkpoint
            trainer.load(args.load)
            print(f"  Loaded AlphaZero checkpoint (iteration {trainer.iteration})")
        elif "model_state_dict" in checkpoint:
            # Behavioral cloning checkpoint
            trainer.network.load_state_dict(checkpoint["model_state_dict"])
            print(f"  Loaded BC checkpoint (epoch {checkpoint.get('epoch', 'unknown')})")
            print("  Note: Starting fresh training from iteration 0")
        else:
            raise ValueError(f"Unknown checkpoint format in {args.load}")

    # Initialize dual buffer with BC data (prevents catastrophic forgetting)
    if args.bc_data:
        print(f"\nInitializing dual buffer with BC data from: {args.bc_data}")
        from essence_wars.data import MCTSDataset

        # Calculate max_games from max_samples (avg ~90 moves per game)
        max_games = args.bc_max_samples // 90 + 1

        bc_dataset = MCTSDataset(args.bc_data, max_games=max_games)

        # Collect BC samples (limit to max_samples)
        bc_samples = []
        for sample in bc_dataset.samples:
            if len(bc_samples) >= args.bc_max_samples:
                break
            bc_samples.append((
                sample.state_tensor,
                sample.action_mask,
                sample.mcts_policy,
                sample.value_target,
            ))

        # Initialize dual buffer with BC data
        trainer.init_dual_buffer(bc_samples, bc_ratio=args.bc_ratio)
        print(f"  Note: Training batches will always sample {args.bc_ratio:.0%} BC, {1-args.bc_ratio:.0%} self-play")

    # Train
    try:
        results = trainer.train(save_dir=str(save_dir))
    except KeyboardInterrupt:
        print("\nTraining interrupted by user")
        results = {
            "iterations": trainer.iteration,
            "total_games": trainer.total_games,
            "final_win_rate": trainer.evaluate_vs_greedy(50),
        }

    # Save final model
    model_path = save_dir / "final_model.pt"
    trainer.save(str(model_path))

    # Save summary
    summary_path = save_dir / "summary.txt"
    with open(summary_path, "w") as f:
        f.write("AlphaZero Training Summary\n")
        f.write("=" * 40 + "\n")
        f.write(f"Iterations: {results.get('iterations', 'N/A')}\n")
        f.write(f"Total games: {results.get('total_games', 'N/A')}\n")
        f.write(f"Final loss: {results.get('final_loss', 'N/A'):.4f}\n")
        f.write(f"Final win rate vs Greedy: {results.get('final_win_rate', 0):.1%}\n")

    if writer is not None:
        writer.close()

    # Print final results
    print("\n" + "=" * 60)
    print("Final Evaluation")
    print("=" * 60)
    print(f"  vs GreedyBot: {results.get('final_win_rate', 0):.1%} win rate")

    final_vs_random = trainer.evaluate_vs_random(100)
    print(f"  vs RandomBot: {final_vs_random:.1%} win rate")

    print(f"\nSummary saved to: {summary_path}")
    print(f"Model saved to: {model_path}")

    # Progress indicator
    win_rate = results.get("final_win_rate", 0)
    target = 0.60
    if win_rate >= target:
        print(f"\n[SUCCESS] Target reached: {win_rate:.1%} >= {target:.0%}")
    else:
        print(f"\n[PROGRESS] Current: {win_rate:.1%} vs Greedy (target: {target:.0%})")
        print("  Consider: more iterations, more simulations, or longer training")


if __name__ == "__main__":
    main()
