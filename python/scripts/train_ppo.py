#!/usr/bin/env python3
"""
Train PPO agent for Essence Wars.

This script trains a PPO agent using vectorized environments and
evaluates it against the built-in GreedyBot.

Usage:
    python train_ppo.py                           # Default generalist training
    python train_ppo.py --timesteps 300000        # Recommended timesteps
    python train_ppo.py --player-faction argentum # Train Argentum specialist
    python train_ppo.py --observation-mode embedded  # Use card embeddings
    python train_ppo.py --tensorboard             # Enable TensorBoard

Examples:
    # Generalist (flat architecture) - baseline
    python train_ppo.py --timesteps 300000 --observation-mode flat

    # Generalist with learned embeddings
    python train_ppo.py --timesteps 300000 --observation-mode embedded

    # Generalist with pretrained Card2Vec embeddings
    python train_ppo.py --timesteps 300000 --observation-mode embedded_pretrained \\
        --pretrained-embeds models/card2vec_*.pt

    # Faction specialist (Argentum)
    python train_ppo.py --timesteps 300000 --player-faction argentum

    # Faction specialist (Symbiote)
    python train_ppo.py --timesteps 300000 --player-faction symbiote

    # Faction specialist (Obsidion)
    python train_ppo.py --timesteps 300000 --player-faction obsidion
"""

import argparse
import sys
from datetime import datetime
from pathlib import Path

# Add parent to path for local development
sys.path.insert(0, str(Path(__file__).parent.parent))


def parse_args():
    parser = argparse.ArgumentParser(
        description="Train PPO agent for Essence Wars",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    # Training
    parser.add_argument(
        "--timesteps",
        type=int,
        default=500_000,
        help="Total training timesteps (default: 500000)",
    )
    parser.add_argument(
        "--num-envs",
        type=int,
        default=64,
        help="Number of parallel environments (default: 64)",
    )
    parser.add_argument(
        "--num-steps",
        type=int,
        default=128,
        help="Steps per rollout per env (default: 128)",
    )

    # Hyperparameters
    parser.add_argument(
        "--lr",
        type=float,
        default=3e-4,
        help="Learning rate (default: 3e-4)",
    )
    parser.add_argument(
        "--gamma",
        type=float,
        default=0.99,
        help="Discount factor (default: 0.99)",
    )
    parser.add_argument(
        "--ent-coef",
        type=float,
        default=0.02,
        help="Entropy coefficient (default: 0.02, higher reduces policy collapse)",
    )
    parser.add_argument(
        "--early-stopping-patience",
        type=int,
        default=None,
        help="Stop if no improvement for N evals (default: None = disabled)",
    )
    parser.add_argument(
        "--hidden-dim",
        type=int,
        default=256,
        help="Hidden layer dimension (default: 256)",
    )

    # Observation mode (card embeddings)
    parser.add_argument(
        "--observation-mode",
        type=str,
        default="flat",
        choices=["flat", "embedded", "embedded_pretrained"],
        help="Observation mode: flat (default), embedded, or embedded_pretrained",
    )
    parser.add_argument(
        "--embed-dim",
        type=int,
        default=64,
        help="Card embedding dimension (default: 64)",
    )
    parser.add_argument(
        "--pretrained-embeds",
        type=str,
        default=None,
        help="Path to pre-trained card embeddings (for embedded_pretrained mode)",
    )
    parser.add_argument(
        "--freeze-embeds",
        action="store_true",
        help="Freeze card embeddings during training",
    )

    # Faction/Deck selection (for specialist training)
    parser.add_argument(
        "--player-faction",
        type=str,
        default=None,
        choices=["argentum", "obsidion", "symbiote"],
        help="Train as faction specialist (cycles through faction's decks)",
    )
    parser.add_argument(
        "--player-deck",
        type=str,
        default=None,
        help="Train with specific deck (overrides --player-faction)",
    )
    parser.add_argument(
        "--opponent-faction",
        type=str,
        default=None,
        choices=["argentum", "obsidion", "symbiote"],
        help="Opponent uses decks from this faction only",
    )
    parser.add_argument(
        "--deck-cycle-interval",
        type=int,
        default=25_000,
        help="Steps between deck changes for faction training (default: 25000)",
    )

    # Evaluation
    parser.add_argument(
        "--eval-interval",
        type=int,
        default=25_000,
        help="Evaluation interval in timesteps (default: 25000)",
    )
    parser.add_argument(
        "--eval-episodes",
        type=int,
        default=100,
        help="Number of evaluation episodes (default: 100)",
    )

    # Logging
    parser.add_argument(
        "--tensorboard",
        action="store_true",
        help="Enable TensorBoard logging",
    )
    parser.add_argument(
        "--log-interval",
        type=int,
        default=5000,
        help="Log interval in timesteps (default: 5000)",
    )

    # Checkpointing
    parser.add_argument(
        "--save-path",
        type=str,
        default=None,
        help="Directory to save checkpoints (default: experiments/ppo/)",
    )
    parser.add_argument(
        "--save-interval",
        type=int,
        default=100_000,
        help="Checkpoint save interval (default: 100000)",
    )
    parser.add_argument(
        "--load",
        type=str,
        default=None,
        help="Load checkpoint from path",
    )

    # Other
    parser.add_argument(
        "--seed",
        type=int,
        default=42,
        help="Random seed (default: 42)",
    )
    parser.add_argument(
        "--device",
        type=str,
        default="auto",
        help="Device: 'cpu', 'cuda', or 'auto' (default: auto)",
    )

    return parser.parse_args()


def main():
    args = parse_args()

    # Set device
    if args.device == "auto":
        import torch
        device = "cuda" if torch.cuda.is_available() else "cpu"
    else:
        device = args.device

    print("=" * 60)
    print("Essence Wars PPO Training")
    print("=" * 60)
    print(f"  Timesteps:     {args.timesteps:,}")
    print(f"  Environments:  {args.num_envs}")
    print(f"  Steps/rollout: {args.num_steps}")
    print(f"  Batch size:    {args.num_envs * args.num_steps:,}")
    print(f"  Learning rate: {args.lr}")
    print(f"  Device:        {device}")
    print(f"  Obs mode:      {args.observation_mode}")
    if args.observation_mode != "flat":
        print(f"  Embed dim:     {args.embed_dim}")
        if args.pretrained_embeds:
            print(f"  Pretrained:    {args.pretrained_embeds}")
        print(f"  Freeze embeds: {args.freeze_embeds}")
    if args.player_faction:
        print(f"  Player faction: {args.player_faction} (specialist)")
    elif args.player_deck:
        print(f"  Player deck:   {args.player_deck}")
    else:
        print(f"  Player:        generalist (all decks)")
    print("=" * 60)

    # Setup save path with faction/mode info
    if args.save_path is None:
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        # Include mode info in path for easy identification
        mode_suffix = ""
        if args.player_faction:
            mode_suffix = f"_{args.player_faction}"
        mode_suffix += f"_{args.observation_mode}"
        save_path = Path(f"experiments/ppo/{timestamp}{mode_suffix}")
    else:
        save_path = Path(args.save_path)

    save_path.mkdir(parents=True, exist_ok=True)
    print(f"Saving to: {save_path}")

    # Setup TensorBoard
    writer = None
    if args.tensorboard:
        try:
            from torch.utils.tensorboard import SummaryWriter
            writer = SummaryWriter(log_dir=str(save_path / "tensorboard"))
            print(f"TensorBoard logging enabled: {save_path / 'tensorboard'}")
        except ImportError:
            print("Warning: TensorBoard not available, skipping logging")

    # Create config
    from essence_wars.agents.ppo import PPOConfig, PPOTrainer

    config = PPOConfig(
        num_envs=args.num_envs,
        total_timesteps=args.timesteps,
        learning_rate=args.lr,
        num_steps=args.num_steps,
        gamma=args.gamma,
        ent_coef=args.ent_coef,
        hidden_dim=args.hidden_dim,
        observation_mode=args.observation_mode,
        embed_dim=args.embed_dim,
        pretrained_embeds_path=args.pretrained_embeds,
        freeze_embeds=args.freeze_embeds,
        player_faction=args.player_faction,
        player_deck=args.player_deck,
        opponent_faction=args.opponent_faction,
        deck_cycle_interval=args.deck_cycle_interval,
        eval_interval=args.eval_interval,
        eval_episodes=args.eval_episodes,
        log_interval=args.log_interval,
        save_interval=args.save_interval,
        device=device,
        save_best=True,
        early_stopping_patience=args.early_stopping_patience,
    )

    # Create trainer
    trainer = PPOTrainer(config=config, writer=writer)

    # Load checkpoint if specified
    if args.load is not None:
        trainer.load(args.load)

    # Callback for saving checkpoints
    def save_callback(step: int, info: dict) -> bool:
        if step > 0 and step % args.save_interval == 0:
            checkpoint_path = save_path / f"checkpoint_{step}.pt"
            trainer.save(str(checkpoint_path))
        return False  # Don't stop training

    # Train
    try:
        results = trainer.train(callback=save_callback, save_path=str(save_path))
    except KeyboardInterrupt:
        print("\nTraining interrupted by user")
        results = {
            "total_timesteps": trainer.global_step,
            "best_win_rate": trainer.best_win_rate,
            "interrupted": True,
        }

    # Save final model
    final_path = save_path / "final_model.pt"
    trainer.save(str(final_path))

    # Final evaluation
    print("\n" + "=" * 60)
    print("Final Evaluation")
    print("=" * 60)

    win_rate_greedy = trainer.evaluate_vs_greedy(200)
    win_rate_random = trainer.evaluate_vs_random(200)

    print(f"  vs GreedyBot: {win_rate_greedy:.1%} win rate")
    print(f"  vs RandomBot: {win_rate_random:.1%} win rate")

    # Save summary
    best_win_rate = results.get("best_win_rate", trainer.best_win_rate)
    summary_path = save_path / "summary.txt"
    with open(summary_path, "w") as f:
        f.write("Essence Wars PPO Training Summary\n")
        f.write("=" * 40 + "\n\n")
        f.write(f"Timesteps: {trainer.global_step:,}\n")
        f.write(f"Final win rate vs Greedy: {win_rate_greedy:.1%}\n")
        f.write(f"Best win rate vs Greedy: {best_win_rate:.1%}\n")
        f.write(f"Win rate vs Random: {win_rate_random:.1%}\n")
        f.write("\nConfig:\n")
        for key, value in vars(config).items():
            if not key.startswith("_"):
                f.write(f"  {key}: {value}\n")

    print(f"\nSummary saved to: {summary_path}")
    print(f"Model saved to: {final_path}")

    # Note best model if it was saved
    best_model_path = save_path / "best_model.pt"
    if best_model_path.exists():
        print(f"Best model saved to: {best_model_path} ({best_win_rate:.1%} win rate)")

    # Success check
    if win_rate_greedy >= 0.6:
        print(f"\n[SUCCESS] Achieved {win_rate_greedy:.1%} win rate vs Greedy (target: 60%)")
    else:
        print(f"\n[PROGRESS] Current: {win_rate_greedy:.1%} vs Greedy (target: 60%)")
        print("  Consider: more timesteps, tuning hyperparameters, or longer training")

    if writer is not None:
        writer.close()


if __name__ == "__main__":
    main()
