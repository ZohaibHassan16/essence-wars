#!/usr/bin/env python3
"""Train a Decision Transformer for Essence Wars.

This script trains a Decision Transformer on MCTS game data,
treating RL as sequence modeling.

Usage:
    uv run python python/scripts/train_decision_transformer.py \
        --dataset data/datasets/distillation_10k_mcts50.jsonl.gz \
        --epochs 50 \
        --output models/decision_transformer.pt
"""

from __future__ import annotations

import argparse
import gzip
import json
import time
from datetime import datetime
from pathlib import Path
from typing import Any

import numpy as np
import torch
import torch.nn.functional as F
from torch.optim import AdamW
from torch.optim.lr_scheduler import CosineAnnealingLR
from torch.utils.data import DataLoader, Dataset, random_split

from essence_wars.agents.decision_transformer import (
    DecisionTransformer,
    DecisionTransformerConfig,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Train Decision Transformer from MCTS data"
    )
    parser.add_argument(
        "--dataset",
        type=str,
        required=True,
        help="Path to MCTS dataset (.jsonl.gz)",
    )
    parser.add_argument(
        "--output",
        type=str,
        default="models/decision_transformer.pt",
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
        default=64,
        help="Training batch size",
    )
    parser.add_argument(
        "--lr",
        type=float,
        default=1e-4,
        help="Learning rate",
    )
    parser.add_argument(
        "--context-length",
        type=int,
        default=20,
        help="Context window length",
    )
    parser.add_argument(
        "--d-model",
        type=int,
        default=128,
        help="Transformer embedding dimension",
    )
    parser.add_argument(
        "--n-layers",
        type=int,
        default=4,
        help="Number of transformer layers",
    )
    parser.add_argument(
        "--n-heads",
        type=int,
        default=4,
        help="Number of attention heads",
    )
    parser.add_argument(
        "--val-split",
        type=float,
        default=0.1,
        help="Fraction of games for validation",
    )
    parser.add_argument(
        "--max-games",
        type=int,
        default=None,
        help="Maximum games to load",
    )
    return parser.parse_args()


class TrajectoryDataset(Dataset):
    """Dataset of game trajectories for Decision Transformer."""

    def __init__(
        self,
        path: str | Path,
        context_length: int = 20,
        max_games: int | None = None,
    ):
        self.context_length = context_length
        self.trajectories: list[dict[str, Any]] = []
        self._load_data(Path(path), max_games)

    def _load_data(self, path: Path, max_games: int | None) -> None:
        games_loaded = 0
        total_steps = 0

        with gzip.open(path, "rt", encoding="utf-8") as f:
            for line in f:
                if max_games and games_loaded >= max_games:
                    break

                game = json.loads(line)
                winner = game["winner"]

                # Skip draws
                if winner == -1:
                    continue

                # Process each player's trajectory separately
                for player in [0, 1]:
                    # Get moves for this player
                    player_moves = [m for m in game["moves"] if m["player"] == player]

                    if len(player_moves) < 2:
                        continue

                    # Determine return for this player
                    if winner == player:
                        returns_to_go = 1.0  # Winner
                    else:
                        returns_to_go = -1.0  # Loser

                    # Build trajectory
                    states = []
                    actions = []
                    action_masks = []

                    for move in player_moves:
                        states.append(np.array(move["state_tensor"], dtype=np.float32))
                        actions.append(move["action"])
                        action_masks.append(
                            np.array(move["action_mask"], dtype=np.float32)
                        )

                    self.trajectories.append(
                        {
                            "states": np.stack(states),
                            "actions": np.array(actions, dtype=np.int64),
                            "action_masks": np.stack(action_masks),
                            "returns_to_go": returns_to_go,
                            "length": len(states),
                        }
                    )
                    total_steps += len(states)

                games_loaded += 1

        print(f"Loaded {games_loaded} games, {len(self.trajectories)} trajectories")
        print(f"Total steps: {total_steps:,}")

    def __len__(self) -> int:
        return len(self.trajectories)

    def __getitem__(self, idx: int) -> dict[str, torch.Tensor]:
        traj = self.trajectories[idx]
        K = self.context_length
        T = traj["length"]

        # Random starting point (or from beginning if trajectory is short)
        if T <= K:
            start = 0
            end = T
        else:
            start = np.random.randint(0, T - K + 1)
            end = start + K

        # Slice trajectory
        states = traj["states"][start:end]
        actions = traj["actions"][start:end]
        action_masks = traj["action_masks"][start:end]

        # Pad if necessary
        actual_len = end - start
        if actual_len < K:
            pad_len = K - actual_len
            states = np.pad(states, ((0, pad_len), (0, 0)), mode="constant")
            actions = np.pad(actions, (0, pad_len), mode="constant")
            action_masks = np.pad(action_masks, ((0, pad_len), (0, 0)), mode="constant")
            attention_mask = np.array([1] * actual_len + [0] * pad_len, dtype=np.float32)
        else:
            attention_mask = np.ones(K, dtype=np.float32)

        # Returns-to-go (constant for sparse reward)
        returns_to_go = np.full(K, traj["returns_to_go"], dtype=np.float32)

        # Timesteps
        timesteps = np.arange(start, start + K, dtype=np.int64)

        return {
            "states": torch.from_numpy(states),
            "actions": torch.from_numpy(actions),
            "action_masks": torch.from_numpy(action_masks),
            "returns_to_go": torch.from_numpy(returns_to_go),
            "timesteps": torch.from_numpy(timesteps),
            "attention_mask": torch.from_numpy(attention_mask),
        }


def train_epoch(
    model: DecisionTransformer,
    loader: DataLoader,
    optimizer: torch.optim.Optimizer,
    device: torch.device,
) -> dict[str, float]:
    model.train()
    total_loss = 0.0
    total_correct = 0
    total_count = 0
    num_batches = 0

    for batch in loader:
        states = batch["states"].to(device)
        actions = batch["actions"].to(device)
        action_masks = batch["action_masks"].to(device).bool()
        returns_to_go = batch["returns_to_go"].to(device)
        timesteps = batch["timesteps"].to(device)
        attention_mask = batch["attention_mask"].to(device)

        optimizer.zero_grad()

        # Forward pass
        action_logits = model(returns_to_go, states, actions, timesteps, attention_mask)

        # Mask invalid actions and compute loss
        B, T, A = action_logits.shape

        # Flatten for loss computation
        logits_flat = action_logits.view(B * T, A)
        actions_flat = actions.view(B * T)
        mask_flat = action_masks.view(B * T, A)
        attn_flat = attention_mask.view(B * T)

        # Apply action mask to logits
        logits_flat = logits_flat.masked_fill(~mask_flat, float("-inf"))

        # Compute cross-entropy loss only on valid timesteps
        loss = F.cross_entropy(logits_flat, actions_flat, reduction="none")
        loss = (loss * attn_flat).sum() / attn_flat.sum()

        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()

        # Compute accuracy
        with torch.no_grad():
            pred_actions = logits_flat.argmax(dim=-1)
            correct = ((pred_actions == actions_flat) * attn_flat).sum().item()
            count = attn_flat.sum().item()

        total_loss += loss.item()
        total_correct += correct
        total_count += count
        num_batches += 1

    return {
        "loss": total_loss / num_batches,
        "accuracy": total_correct / total_count if total_count > 0 else 0,
    }


@torch.no_grad()
def validate(
    model: DecisionTransformer,
    loader: DataLoader,
    device: torch.device,
) -> dict[str, float]:
    model.eval()
    total_loss = 0.0
    total_correct = 0
    total_count = 0
    num_batches = 0

    for batch in loader:
        states = batch["states"].to(device)
        actions = batch["actions"].to(device)
        action_masks = batch["action_masks"].to(device).bool()
        returns_to_go = batch["returns_to_go"].to(device)
        timesteps = batch["timesteps"].to(device)
        attention_mask = batch["attention_mask"].to(device)

        # Forward pass
        action_logits = model(returns_to_go, states, actions, timesteps, attention_mask)

        # Compute loss
        B, T, A = action_logits.shape
        logits_flat = action_logits.view(B * T, A)
        actions_flat = actions.view(B * T)
        mask_flat = action_masks.view(B * T, A)
        attn_flat = attention_mask.view(B * T)

        logits_flat = logits_flat.masked_fill(~mask_flat, float("-inf"))
        loss = F.cross_entropy(logits_flat, actions_flat, reduction="none")
        loss = (loss * attn_flat).sum() / attn_flat.sum()

        # Accuracy
        pred_actions = logits_flat.argmax(dim=-1)
        correct = ((pred_actions == actions_flat) * attn_flat).sum().item()
        count = attn_flat.sum().item()

        total_loss += loss.item()
        total_correct += correct
        total_count += count
        num_batches += 1

    return {
        "loss": total_loss / num_batches,
        "accuracy": total_correct / total_count if total_count > 0 else 0,
    }


def main() -> None:
    args = parse_args()

    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    print(f"Device: {device}")

    # Setup output
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    # Load dataset
    print(f"\nLoading dataset from {args.dataset}...")
    dataset = TrajectoryDataset(
        args.dataset,
        context_length=args.context_length,
        max_games=args.max_games,
    )

    # Split
    val_size = int(len(dataset) * args.val_split)
    train_size = len(dataset) - val_size
    train_dataset, val_dataset = random_split(dataset, [train_size, val_size])

    print(f"  Train trajectories: {len(train_dataset):,}")
    print(f"  Val trajectories:   {len(val_dataset):,}")

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
    config = DecisionTransformerConfig(
        d_model=args.d_model,
        n_layers=args.n_layers,
        n_heads=args.n_heads,
        context_length=args.context_length,
    )

    print(f"\n=== Model Configuration ===")
    print(f"  d_model:        {config.d_model}")
    print(f"  n_layers:       {config.n_layers}")
    print(f"  n_heads:        {config.n_heads}")
    print(f"  context_length: {config.context_length}")

    model = DecisionTransformer(config).to(device)

    # Optimizer
    optimizer = AdamW(model.parameters(), lr=args.lr, weight_decay=1e-4)
    scheduler = CosineAnnealingLR(optimizer, T_max=args.epochs)

    # TensorBoard
    writer = None
    try:
        from torch.utils.tensorboard import SummaryWriter

        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        log_dir = Path(f"experiments/decision_transformer/{timestamp}")
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
            f"Train Loss: {train_metrics['loss']:.4f} | "
            f"Val Loss: {val_metrics['loss']:.4f} | "
            f"Train Acc: {train_metrics['accuracy']:.2%} | "
            f"Val Acc: {val_metrics['accuracy']:.2%} | "
            f"LR: {scheduler.get_last_lr()[0]:.2e} | "
            f"Time: {epoch_time:.1f}s"
        )

        if writer:
            writer.add_scalar("train/loss", train_metrics["loss"], epoch)
            writer.add_scalar("train/accuracy", train_metrics["accuracy"], epoch)
            writer.add_scalar("val/loss", val_metrics["loss"], epoch)
            writer.add_scalar("val/accuracy", val_metrics["accuracy"], epoch)
            writer.add_scalar("lr", scheduler.get_last_lr()[0], epoch)

        # Save best model
        if val_metrics["loss"] < best_val_loss:
            best_val_loss = val_metrics["loss"]
            torch.save(
                {
                    "epoch": epoch,
                    "model_state_dict": model.state_dict(),
                    "optimizer_state_dict": optimizer.state_dict(),
                    "config": config,
                    "train_metrics": train_metrics,
                    "val_metrics": val_metrics,
                    "best_val_loss": best_val_loss,
                    "args": vars(args),
                },
                output_path,
            )
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
