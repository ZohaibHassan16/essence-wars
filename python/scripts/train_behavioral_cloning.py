#!/usr/bin/env python3
"""Behavioral Cloning training script for Essence Wars.

Train a neural network to imitate MCTS policy and predict game outcomes
using pre-generated MCTS vs MCTS game data.

This is much faster than AlphaZero self-play training because:
1. Data generation is parallelized in Rust (10-50x faster)
2. GPU utilization is high (80%+) - just batch training, no sequential game gen
3. Can be repeated quickly for architecture experiments

Usage:
    uv run python python/scripts/train_behavioral_cloning.py \\
        --dataset data/datasets/mcts_100k.jsonl \\
        --epochs 50 \\
        --output models/bc_mcts_100k.pt \\
        --tensorboard
"""

from __future__ import annotations

import argparse
import time
from datetime import datetime
from pathlib import Path

import torch
import torch.nn.functional as F
from torch.optim import AdamW
from torch.optim.lr_scheduler import CosineAnnealingLR
from torch.utils.data import DataLoader, random_split
from torch.utils.tensorboard import SummaryWriter

from essence_wars.agents.networks import AlphaZeroNetwork
from essence_wars.data import MCTSDataset, get_dataset_stats


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Train neural network via behavioral cloning on MCTS data"
    )
    parser.add_argument(
        "--dataset",
        type=str,
        required=True,
        help="Path to MCTS dataset (.jsonl or .jsonl.gz)",
    )
    parser.add_argument(
        "--output",
        type=str,
        default="models/bc_model.pt",
        help="Output path for trained model",
    )
    parser.add_argument(
        "--epochs",
        type=int,
        default=50,
        help="Number of training epochs",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=256,
        help="Training batch size",
    )
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
        help="Weight decay (L2 regularization)",
    )
    parser.add_argument(
        "--hidden-dim",
        type=int,
        default=256,
        help="Hidden dimension for network",
    )
    parser.add_argument(
        "--num-blocks",
        type=int,
        default=4,
        help="Number of residual blocks",
    )
    parser.add_argument(
        "--val-split",
        type=float,
        default=0.1,
        help="Fraction of data to use for validation",
    )
    parser.add_argument(
        "--policy-weight",
        type=float,
        default=1.0,
        help="Weight for policy loss",
    )
    parser.add_argument(
        "--value-weight",
        type=float,
        default=1.0,
        help="Weight for value loss",
    )
    parser.add_argument(
        "--max-games",
        type=int,
        default=None,
        help="Maximum games to load (for quick testing)",
    )
    parser.add_argument(
        "--tensorboard",
        action="store_true",
        help="Enable TensorBoard logging",
    )
    parser.add_argument(
        "--log-dir",
        type=str,
        default=None,
        help="TensorBoard log directory (auto-generated if not specified)",
    )
    parser.add_argument(
        "--save-every",
        type=int,
        default=10,
        help="Save checkpoint every N epochs",
    )
    parser.add_argument(
        "--resume",
        type=str,
        default=None,
        help="Resume training from checkpoint",
    )
    parser.add_argument(
        "--num-workers",
        type=int,
        default=4,
        help="Number of DataLoader workers",
    )
    return parser.parse_args()


def behavioral_cloning_loss(
    policy_logits: torch.Tensor,
    value_pred: torch.Tensor,
    policy_target: torch.Tensor,
    value_target: torch.Tensor,
    action_mask: torch.Tensor,
    policy_weight: float = 1.0,
    value_weight: float = 1.0,
) -> tuple[torch.Tensor, dict[str, float]]:
    """Compute behavioral cloning loss.

    Args:
        policy_logits: Raw logits from network (B, 256) - already masked
        value_pred: Value prediction (B,)
        policy_target: MCTS policy distribution (B, 256)
        value_target: Game outcome (B,)
        action_mask: Legal action mask (B, 256) boolean
        policy_weight: Weight for policy loss
        value_weight: Weight for value loss

    Returns:
        Tuple of (total_loss, metrics_dict)
    """
    # Policy loss: cross-entropy with MCTS policy as soft target
    # Only compute on legal actions to avoid -inf * 0 = nan
    # Use the raw logits since they're already masked in the forward pass

    # Approach: use KL divergence on legal actions only
    # First mask the target policy to only legal actions and renormalize
    masked_target = policy_target.clone()
    mask_float = action_mask.float()
    masked_target = masked_target * mask_float
    target_sum = masked_target.sum(dim=-1, keepdim=True).clamp(min=1e-8)
    masked_target = masked_target / target_sum  # Renormalize

    # Compute log softmax over logits (which are already masked with -1e8)
    log_probs = F.log_softmax(policy_logits, dim=-1)

    # KL divergence: sum of target * log(target/pred) = sum(target*log(target)) - sum(target*log(pred))
    # Since we only care about gradient wrt pred, use cross-entropy form:
    # -sum(target * log_pred) on legal actions only
    policy_loss = -(masked_target * log_probs * mask_float).sum(dim=-1).mean()

    # Value loss: MSE with game outcome
    value_loss = F.mse_loss(value_pred.squeeze(-1), value_target)

    # Total loss
    total_loss = policy_weight * policy_loss + value_weight * value_loss

    # Compute accuracy (for monitoring)
    with torch.no_grad():
        # Policy accuracy: does the top predicted action match MCTS top action?
        # Mask out illegal actions for comparison
        masked_logits = policy_logits.clone()
        masked_logits[~action_mask] = float("-inf")
        pred_action = masked_logits.argmax(dim=-1)
        target_action = policy_target.argmax(dim=-1)
        policy_acc = (pred_action == target_action).float().mean().item()

        # Value accuracy: is prediction in the right direction?
        value_sign_acc = ((value_pred.squeeze(-1) * value_target) > 0).float().mean().item()

    metrics = {
        "policy_loss": policy_loss.item(),
        "value_loss": value_loss.item(),
        "total_loss": total_loss.item(),
        "policy_acc": policy_acc,
        "value_sign_acc": value_sign_acc,
    }

    return total_loss, metrics


def train_epoch(
    model: AlphaZeroNetwork,
    loader: DataLoader,
    optimizer: torch.optim.Optimizer,
    device: torch.device,
    policy_weight: float,
    value_weight: float,
) -> dict[str, float]:
    """Train for one epoch."""
    model.train()
    total_metrics: dict[str, float] = {}
    num_batches = 0

    for batch in loader:
        obs = batch["obs"].to(device)
        mask = batch["mask"].to(device).bool()  # Convert to boolean for masking
        policy_target = batch["policy_target"].to(device)
        value_target = batch["value_target"].to(device)

        optimizer.zero_grad()

        # Forward pass
        policy_logits, value_pred = model(obs, mask)

        # Compute loss
        loss, metrics = behavioral_cloning_loss(
            policy_logits, value_pred,
            policy_target, value_target,
            mask, policy_weight, value_weight,
        )

        # Backward pass
        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()

        # Accumulate metrics
        for k, v in metrics.items():
            total_metrics[k] = total_metrics.get(k, 0.0) + v
        num_batches += 1

    # Average metrics
    for k in total_metrics:
        total_metrics[k] /= num_batches

    return total_metrics


@torch.no_grad()
def validate(
    model: AlphaZeroNetwork,
    loader: DataLoader,
    device: torch.device,
    policy_weight: float,
    value_weight: float,
) -> dict[str, float]:
    """Validate model on held-out data."""
    model.eval()
    total_metrics: dict[str, float] = {}
    num_batches = 0

    for batch in loader:
        obs = batch["obs"].to(device)
        mask = batch["mask"].to(device).bool()  # Convert to boolean for masking
        policy_target = batch["policy_target"].to(device)
        value_target = batch["value_target"].to(device)

        # Forward pass
        policy_logits, value_pred = model(obs, mask)

        # Compute loss
        _, metrics = behavioral_cloning_loss(
            policy_logits, value_pred,
            policy_target, value_target,
            mask, policy_weight, value_weight,
        )

        # Accumulate metrics
        for k, v in metrics.items():
            total_metrics[k] = total_metrics.get(k, 0.0) + v
        num_batches += 1

    # Average metrics
    for k in total_metrics:
        total_metrics[k] /= num_batches

    return total_metrics


def main() -> None:
    args = parse_args()

    # Setup device
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    print(f"Using device: {device}")

    # Setup output directory
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    # Setup TensorBoard
    writer = None
    if args.tensorboard:
        if args.log_dir:
            log_dir = Path(args.log_dir)
        else:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            log_dir = Path(f"experiments/behavioral_cloning/{timestamp}")
        log_dir.mkdir(parents=True, exist_ok=True)
        writer = SummaryWriter(log_dir)
        print(f"TensorBoard logging to: {log_dir}")

    # Print dataset stats
    print("\n=== Dataset Statistics ===")
    stats = get_dataset_stats(args.dataset, max_games=args.max_games)
    print(f"  Total games:    {stats['total_games']:,}")
    print(f"  Total samples:  {stats['total_moves']:,}")
    print(f"  Avg moves/game: {stats['avg_moves_per_game']:.1f}")
    print(f"  P1 win rate:    {stats['win_rate_p1']:.1%}")
    print(f"  Unique decks:   {len(stats['unique_decks'])}")

    # Load dataset
    print("\nLoading dataset...")
    dataset = MCTSDataset(args.dataset, max_games=args.max_games, normalize=False)

    # Split into train/val
    val_size = int(len(dataset) * args.val_split)
    train_size = len(dataset) - val_size
    train_dataset, val_dataset = random_split(dataset, [train_size, val_size])

    print(f"  Train samples: {len(train_dataset):,}")
    print(f"  Val samples:   {len(val_dataset):,}")

    # Create data loaders
    train_loader = DataLoader(
        train_dataset,
        batch_size=args.batch_size,
        shuffle=True,
        num_workers=args.num_workers,
        pin_memory=device.type == "cuda",
    )
    val_loader = DataLoader(
        val_dataset,
        batch_size=args.batch_size,
        shuffle=False,
        num_workers=args.num_workers,
        pin_memory=device.type == "cuda",
    )

    # Create model
    print("\n=== Model Configuration ===")
    print(f"  Hidden dim:     {args.hidden_dim}")
    print(f"  Residual blocks: {args.num_blocks}")

    model = AlphaZeroNetwork(
        obs_dim=326,
        action_dim=256,
        hidden_dim=args.hidden_dim,
        num_blocks=args.num_blocks,
    ).to(device)

    # Count parameters
    num_params = sum(p.numel() for p in model.parameters())
    print(f"  Parameters:     {num_params:,}")

    # Create optimizer and scheduler
    optimizer = AdamW(
        model.parameters(),
        lr=args.lr,
        weight_decay=args.weight_decay,
    )
    scheduler = CosineAnnealingLR(optimizer, T_max=args.epochs)

    # Resume from checkpoint if specified
    start_epoch = 0
    best_val_loss = float("inf")
    if args.resume:
        print(f"\nResuming from: {args.resume}")
        checkpoint = torch.load(args.resume, map_location=device)
        model.load_state_dict(checkpoint["model_state_dict"])
        optimizer.load_state_dict(checkpoint["optimizer_state_dict"])
        start_epoch = checkpoint["epoch"] + 1
        best_val_loss = checkpoint.get("best_val_loss", float("inf"))

    # Training loop
    print("\n=== Training ===")
    print(f"  Epochs:        {args.epochs}")
    print(f"  Batch size:    {args.batch_size}")
    print(f"  Learning rate: {args.lr}")
    print(f"  Policy weight: {args.policy_weight}")
    print(f"  Value weight:  {args.value_weight}")
    print()

    start_time = time.time()

    for epoch in range(start_epoch, args.epochs):
        epoch_start = time.time()

        # Train
        train_metrics = train_epoch(
            model, train_loader, optimizer, device,
            args.policy_weight, args.value_weight,
        )

        # Validate
        val_metrics = validate(
            model, val_loader, device,
            args.policy_weight, args.value_weight,
        )

        # Update scheduler
        scheduler.step()

        # Print progress
        epoch_time = time.time() - epoch_start
        print(
            f"Epoch {epoch+1:3d}/{args.epochs} | "
            f"Train Loss: {train_metrics['total_loss']:.4f} | "
            f"Val Loss: {val_metrics['total_loss']:.4f} | "
            f"Policy Acc: {val_metrics['policy_acc']:.2%} | "
            f"Value Acc: {val_metrics['value_sign_acc']:.2%} | "
            f"LR: {scheduler.get_last_lr()[0]:.2e} | "
            f"Time: {epoch_time:.1f}s"
        )

        # Log to TensorBoard
        if writer:
            for k, v in train_metrics.items():
                writer.add_scalar(f"train/{k}", v, epoch)
            for k, v in val_metrics.items():
                writer.add_scalar(f"val/{k}", v, epoch)
            writer.add_scalar("lr", scheduler.get_last_lr()[0], epoch)

        # Save checkpoint
        if (epoch + 1) % args.save_every == 0:
            checkpoint_path = output_path.with_suffix(f".epoch{epoch+1}.pt")
            torch.save({
                "epoch": epoch,
                "model_state_dict": model.state_dict(),
                "optimizer_state_dict": optimizer.state_dict(),
                "train_metrics": train_metrics,
                "val_metrics": val_metrics,
                "best_val_loss": best_val_loss,
                "args": vars(args),
            }, checkpoint_path)
            print(f"  Saved checkpoint: {checkpoint_path}")

        # Save best model
        if val_metrics["total_loss"] < best_val_loss:
            best_val_loss = val_metrics["total_loss"]
            torch.save({
                "epoch": epoch,
                "model_state_dict": model.state_dict(),
                "optimizer_state_dict": optimizer.state_dict(),
                "train_metrics": train_metrics,
                "val_metrics": val_metrics,
                "best_val_loss": best_val_loss,
                "args": vars(args),
            }, output_path)
            print(f"  New best model saved: {output_path}")

    # Training complete
    total_time = time.time() - start_time
    print("\n=== Training Complete ===")
    print(f"  Total time:     {total_time / 60:.1f} minutes")
    print(f"  Best val loss:  {best_val_loss:.4f}")
    print(f"  Model saved to: {output_path}")

    if writer:
        writer.close()


if __name__ == "__main__":
    main()
