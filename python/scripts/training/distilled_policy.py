#!/usr/bin/env python3
"""Train a distilled policy network from MCTS data.

This script trains a neural network to predict MCTS policies directly,
allowing fast inference without tree search. The network learns to
imitate the MCTS policy distribution rather than raw actions.

Usage:
    uv run python python/scripts/train_distilled_policy.py \
        --dataset data/datasets/distillation_1k_mcts25.jsonl.gz \
        --epochs 30 \
        --output models/distilled_mcts25.pt
"""

from __future__ import annotations

import argparse
import gzip
import json
import time
from datetime import datetime
from pathlib import Path

import numpy as np
import torch
import torch.nn.functional as F
from torch.optim import AdamW
from torch.optim.lr_scheduler import CosineAnnealingLR
from torch.utils.data import DataLoader, Dataset, random_split

from essence_wars.agents.networks import AlphaZeroNetwork


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Train distilled policy network from MCTS data"
    )
    parser.add_argument(
        "--dataset",
        type=str,
        required=True,
        help="Path to distillation dataset (.jsonl.gz)",
    )
    parser.add_argument(
        "--output",
        type=str,
        default="models/distilled_policy.pt",
        help="Output path for trained model",
    )
    parser.add_argument(
        "--epochs",
        type=int,
        default=30,
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
        help="Fraction of data for validation",
    )
    parser.add_argument(
        "--max-games",
        type=int,
        default=None,
        help="Maximum games to load",
    )
    parser.add_argument(
        "--no-tensorboard",
        action="store_true",
        help="Disable TensorBoard logging",
    )
    return parser.parse_args()


class DistillationDataset(Dataset):
    """Dataset for policy distillation from MCTS data."""

    def __init__(self, path: str | Path, max_games: int | None = None):
        self.samples = []
        self._load_data(Path(path), max_games)

    def _load_data(self, path: Path, max_games: int | None) -> None:
        games_loaded = 0

        with gzip.open(path, "rt", encoding="utf-8") as f:
            for line in f:
                if max_games and games_loaded >= max_games:
                    break

                game = json.loads(line)
                winner = game["winner"]

                for move in game["moves"]:
                    player = move["player"]

                    # Value target: +1 if this player won, -1 if lost, 0 if draw
                    if winner == -1:
                        value_target = 0.0
                    elif winner == player:
                        value_target = 1.0
                    else:
                        value_target = -1.0

                    self.samples.append({
                        "state": np.array(move["state_tensor"], dtype=np.float32),
                        "mask": np.array(move["action_mask"], dtype=np.float32),
                        "policy": np.array(move["mcts_policy"], dtype=np.float32),
                        "action": move["action"],
                        "value": value_target,
                    })

                games_loaded += 1

        print(f"Loaded {games_loaded} games, {len(self.samples)} samples")

    def __len__(self) -> int:
        return len(self.samples)

    def __getitem__(self, idx: int) -> dict[str, torch.Tensor]:
        sample = self.samples[idx]
        return {
            "obs": torch.from_numpy(sample["state"]),
            "mask": torch.from_numpy(sample["mask"]),
            "policy_target": torch.from_numpy(sample["policy"]),
            "action": torch.tensor(sample["action"], dtype=torch.long),
            "value_target": torch.tensor(sample["value"], dtype=torch.float32),
        }


def distillation_loss(
    policy_logits: torch.Tensor,
    value_pred: torch.Tensor,
    policy_target: torch.Tensor,
    value_target: torch.Tensor,
    action_mask: torch.Tensor,
    policy_weight: float = 1.0,
    value_weight: float = 0.5,
) -> tuple[torch.Tensor, dict[str, float]]:
    """Compute distillation loss.

    Uses KL divergence between predicted policy and MCTS policy.
    """
    # Mask and renormalize target policy
    mask_float = action_mask.float()
    masked_target = policy_target * mask_float
    target_sum = masked_target.sum(dim=-1, keepdim=True).clamp(min=1e-8)
    masked_target = masked_target / target_sum

    # Policy loss: KL divergence (as cross-entropy since target is fixed)
    log_probs = F.log_softmax(policy_logits, dim=-1)
    policy_loss = -(masked_target * log_probs * mask_float).sum(dim=-1).mean()

    # Value loss: MSE
    value_loss = F.mse_loss(value_pred.squeeze(-1), value_target)

    # Total loss
    total_loss = policy_weight * policy_loss + value_weight * value_loss

    # Compute metrics
    with torch.no_grad():
        # Policy accuracy: top action matches MCTS top action
        masked_logits = policy_logits.clone()
        masked_logits[action_mask == 0] = float("-inf")
        pred_action = masked_logits.argmax(dim=-1)
        target_action = policy_target.argmax(dim=-1)
        policy_acc = (pred_action == target_action).float().mean().item()

        # Top-3 accuracy
        _, top3_pred = masked_logits.topk(3, dim=-1)
        top3_match = (top3_pred == target_action.unsqueeze(-1)).any(dim=-1)
        top3_acc = top3_match.float().mean().item()

        # Value accuracy
        value_sign_acc = ((value_pred.squeeze(-1) * value_target) > 0).float().mean().item()

    return total_loss, {
        "policy_loss": policy_loss.item(),
        "value_loss": value_loss.item(),
        "total_loss": total_loss.item(),
        "policy_acc": policy_acc,
        "top3_acc": top3_acc,
        "value_sign_acc": value_sign_acc,
    }


def train_epoch(
    model: AlphaZeroNetwork,
    loader: DataLoader,
    optimizer: torch.optim.Optimizer,
    device: torch.device,
) -> dict[str, float]:
    model.train()
    total_metrics: dict[str, float] = {}
    num_batches = 0

    for batch in loader:
        obs = batch["obs"].to(device)
        mask = batch["mask"].to(device).bool()
        policy_target = batch["policy_target"].to(device)
        value_target = batch["value_target"].to(device)

        optimizer.zero_grad()

        policy_logits, value_pred = model(obs, mask)

        loss, metrics = distillation_loss(
            policy_logits, value_pred,
            policy_target, value_target,
            mask.float(),
        )

        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()

        for k, v in metrics.items():
            total_metrics[k] = total_metrics.get(k, 0.0) + v
        num_batches += 1

    for k in total_metrics:
        total_metrics[k] /= num_batches

    return total_metrics


@torch.no_grad()
def validate(
    model: AlphaZeroNetwork,
    loader: DataLoader,
    device: torch.device,
) -> dict[str, float]:
    model.eval()
    total_metrics: dict[str, float] = {}
    num_batches = 0

    for batch in loader:
        obs = batch["obs"].to(device)
        mask = batch["mask"].to(device).bool()
        policy_target = batch["policy_target"].to(device)
        value_target = batch["value_target"].to(device)

        policy_logits, value_pred = model(obs, mask)

        _, metrics = distillation_loss(
            policy_logits, value_pred,
            policy_target, value_target,
            mask.float(),
        )

        for k, v in metrics.items():
            total_metrics[k] = total_metrics.get(k, 0.0) + v
        num_batches += 1

    for k in total_metrics:
        total_metrics[k] /= num_batches

    return total_metrics


def main() -> None:
    args = parse_args()

    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    print(f"Device: {device}")

    # Setup output
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    # Load dataset
    print(f"\nLoading dataset from {args.dataset}...")
    dataset = DistillationDataset(args.dataset, max_games=args.max_games)

    # Split
    val_size = int(len(dataset) * args.val_split)
    train_size = len(dataset) - val_size
    train_dataset, val_dataset = random_split(dataset, [train_size, val_size])

    print(f"  Train samples: {len(train_dataset):,}")
    print(f"  Val samples:   {len(val_dataset):,}")

    # Create loaders
    train_loader = DataLoader(
        train_dataset,
        batch_size=args.batch_size,
        shuffle=True,
        num_workers=4,
        pin_memory=device.type == "cuda",
    )
    val_loader = DataLoader(
        val_dataset,
        batch_size=args.batch_size,
        shuffle=False,
        num_workers=4,
        pin_memory=device.type == "cuda",
    )

    # Create model
    print(f"\n=== Model Configuration ===")
    print(f"  Hidden dim:     {args.hidden_dim}")
    print(f"  Residual blocks: {args.num_blocks}")

    model = AlphaZeroNetwork(
        obs_dim=326,
        action_dim=256,
        hidden_dim=args.hidden_dim,
        num_blocks=args.num_blocks,
    ).to(device)

    num_params = sum(p.numel() for p in model.parameters())
    print(f"  Parameters:     {num_params:,}")

    # Optimizer
    optimizer = AdamW(model.parameters(), lr=args.lr, weight_decay=1e-4)
    scheduler = CosineAnnealingLR(optimizer, T_max=args.epochs)

    # TensorBoard
    writer = None
    if not args.no_tensorboard:
        try:
            from torch.utils.tensorboard import SummaryWriter
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            log_dir = Path(f"experiments/distillation/{timestamp}")
            log_dir.mkdir(parents=True, exist_ok=True)
            writer = SummaryWriter(log_dir)
            print(f"  TensorBoard:    {log_dir}")
        except ImportError:
            pass

    # Training
    print(f"\n=== Training ===")
    print(f"  Epochs:        {args.epochs}")
    print(f"  Batch size:    {args.batch_size}")
    print(f"  Learning rate: {args.lr}")
    print()

    best_val_loss = float("inf")
    start_time = time.time()

    for epoch in range(args.epochs):
        epoch_start = time.time()

        train_metrics = train_epoch(model, train_loader, optimizer, device)
        val_metrics = validate(model, val_loader, device)

        scheduler.step()

        epoch_time = time.time() - epoch_start
        print(
            f"Epoch {epoch+1:3d}/{args.epochs} | "
            f"Train Loss: {train_metrics['total_loss']:.4f} | "
            f"Val Loss: {val_metrics['total_loss']:.4f} | "
            f"Policy Acc: {val_metrics['policy_acc']:.2%} | "
            f"Top3 Acc: {val_metrics['top3_acc']:.2%} | "
            f"LR: {scheduler.get_last_lr()[0]:.2e} | "
            f"Time: {epoch_time:.1f}s"
        )

        if writer:
            for k, v in train_metrics.items():
                writer.add_scalar(f"train/{k}", v, epoch)
            for k, v in val_metrics.items():
                writer.add_scalar(f"val/{k}", v, epoch)
            writer.add_scalar("lr", scheduler.get_last_lr()[0], epoch)

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

    total_time = time.time() - start_time
    print(f"\n=== Training Complete ===")
    print(f"  Total time:     {total_time / 60:.1f} minutes")
    print(f"  Best val loss:  {best_val_loss:.4f}")
    print(f"  Model saved to: {output_path}")

    if writer:
        writer.close()


if __name__ == "__main__":
    main()
