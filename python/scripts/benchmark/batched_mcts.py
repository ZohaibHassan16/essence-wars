#!/usr/bin/env python3
"""
Benchmark Batched vs Sequential Neural MCTS.

This script measures the speedup from GPU batching in Neural MCTS.
The key insight: a batch of 16-64 states takes almost the same time
as a single state on GPU, so batching gives near-linear speedup.

Usage:
    uv run python python/scripts/benchmark_batched_mcts.py
    uv run python python/scripts/benchmark_batched_mcts.py --batch-sizes 8 16 32 64
    uv run python python/scripts/benchmark_batched_mcts.py --sims 100 --games 20
"""

import argparse
import sys
import time
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).parent.parent))

from essence_wars._core import PyGame
from essence_wars.agents.neural_mcts import NeuralMctsBot


def load_bc_network_legacy(checkpoint_path: str, device: str = "cuda"):
    """Load BC network handling both legacy and new checkpoint formats."""
    import torch.nn as nn

    class ResidualBlock(nn.Module):
        def __init__(self, hidden_dim: int) -> None:
            super().__init__()
            self.layers = nn.Sequential(
                nn.Linear(hidden_dim, hidden_dim),
                nn.ReLU(),
                nn.Linear(hidden_dim, hidden_dim),
            )

        def forward(self, x):
            return torch.relu(x + self.layers(x))

    class AlphaZeroNetwork(nn.Module):
        """AlphaZero-style network that handles both legacy and new formats."""

        def __init__(self, hidden_dim: int = 256, num_blocks: int = 4, has_input_norm: bool = False):
            super().__init__()
            self.has_input_norm = has_input_norm

            if has_input_norm:
                self.input_norm = nn.LayerNorm(326)

            self.input_proj = nn.Sequential(
                nn.Linear(326, hidden_dim),
                nn.ReLU(),
            )
            self.residual_tower = nn.ModuleList([
                ResidualBlock(hidden_dim) for _ in range(num_blocks)
            ])
            self.policy_head = nn.Sequential(
                nn.Linear(hidden_dim, hidden_dim),
                nn.ReLU(),
                nn.Linear(hidden_dim, 256),
            )
            self.value_head = nn.Sequential(
                nn.Linear(hidden_dim, hidden_dim),
                nn.ReLU(),
                nn.Linear(hidden_dim, 1),
                nn.Tanh(),
            )

        def forward(self, obs, action_mask=None):
            if self.has_input_norm:
                obs = self.input_norm(obs)
            x = self.input_proj(obs)
            for block in self.residual_tower:
                x = block(x)
            logits = self.policy_head(x)
            if action_mask is not None:
                logits = logits.masked_fill(~action_mask, float("-inf"))
            value = self.value_head(x).squeeze(-1)
            return logits, value

    checkpoint = torch.load(checkpoint_path, map_location=device, weights_only=False)
    args = checkpoint.get("args", {})
    if hasattr(args, "__dict__"):
        args = vars(args)

    state_dict = checkpoint["model_state_dict"]

    # Check if new format (has input_norm)
    has_input_norm = "input_norm.weight" in state_dict

    network = AlphaZeroNetwork(
        hidden_dim=args.get("hidden_dim", 256),
        num_blocks=args.get("num_blocks", 4),
        has_input_norm=has_input_norm,
    )
    network.load_state_dict(state_dict)
    network.to(device)
    network.eval()
    return network


def benchmark_sequential(bot: NeuralMctsBot, num_games: int, verbose: bool = True) -> dict:
    """Benchmark sequential (non-batched) MCTS."""
    total_time = 0.0
    total_moves = 0
    wins = 0

    for g in range(num_games):
        game = PyGame()
        game.reset(seed=g * 100)

        while not game.is_done():
            player = game.current_player()
            if player == 0:
                start = time.perf_counter()
                action, _ = bot.get_action_with_game(game)
                total_time += time.perf_counter() - start
                total_moves += 1
            else:
                action = game.greedy_action()
            game.step(action)

        if game.get_reward(0) > 0:
            wins += 1

        if verbose and (g + 1) % 5 == 0:
            print(f"  Sequential: {g + 1}/{num_games} games")

    return {
        "method": "sequential",
        "total_time": total_time,
        "total_moves": total_moves,
        "avg_time_ms": total_time / total_moves * 1000 if total_moves > 0 else 0,
        "games": num_games,
        "wins": wins,
        "win_rate": wins / num_games,
    }


def benchmark_batched(
    bot: NeuralMctsBot, num_games: int, batch_size: int, verbose: bool = True
) -> dict:
    """Benchmark batched MCTS."""
    total_time = 0.0
    total_moves = 0
    wins = 0

    for g in range(num_games):
        game = PyGame()
        game.reset(seed=g * 100)

        while not game.is_done():
            player = game.current_player()
            if player == 0:
                start = time.perf_counter()
                action, _ = bot.get_action_with_game_batched(game, batch_size=batch_size)
                total_time += time.perf_counter() - start
                total_moves += 1
            else:
                action = game.greedy_action()
            game.step(action)

        if game.get_reward(0) > 0:
            wins += 1

        if verbose and (g + 1) % 5 == 0:
            print(f"  Batched-{batch_size}: {g + 1}/{num_games} games")

    return {
        "method": f"batched-{batch_size}",
        "batch_size": batch_size,
        "total_time": total_time,
        "total_moves": total_moves,
        "avg_time_ms": total_time / total_moves * 1000 if total_moves > 0 else 0,
        "games": num_games,
        "wins": wins,
        "win_rate": wins / num_games,
    }


def main():
    parser = argparse.ArgumentParser(
        description="Benchmark Batched vs Sequential Neural MCTS",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--model",
        type=str,
        default="models/bc_mcts_values.pt",
        help="Path to model checkpoint",
    )
    parser.add_argument(
        "--sims",
        type=int,
        default=100,
        help="Number of MCTS simulations per move",
    )
    parser.add_argument(
        "--games",
        type=int,
        default=10,
        help="Number of games to benchmark",
    )
    parser.add_argument(
        "--batch-sizes",
        type=int,
        nargs="+",
        default=[8, 16, 32],
        help="Batch sizes to test",
    )
    parser.add_argument(
        "--device",
        type=str,
        default="cuda",
        help="Device for inference",
    )

    args = parser.parse_args()

    device = args.device
    if device == "cuda" and not torch.cuda.is_available():
        device = "cpu"
        print("CUDA not available, using CPU")

    print("=" * 70)
    print("Batched vs Sequential Neural MCTS Benchmark")
    print("=" * 70)
    print(f"  Model:       {args.model}")
    print(f"  Simulations: {args.sims}")
    print(f"  Games:       {args.games}")
    print(f"  Batch sizes: {args.batch_sizes}")
    print(f"  Device:      {device}")
    print("=" * 70)

    # Load model
    print("\nLoading model...")
    if not Path(args.model).exists():
        print(f"ERROR: Model not found: {args.model}")
        return 1

    network = load_bc_network_legacy(args.model, device)
    print("  Model loaded successfully")

    # Create bot
    bot = NeuralMctsBot(
        network=network,
        num_simulations=args.sims,
        device=device,
        use_value=True,
    )

    # Warm up GPU
    print("\nWarming up GPU...")
    game = PyGame()
    game.reset(seed=0)
    for _ in range(3):
        bot.get_action_with_game(game)
    print("  GPU warmed up")

    results = []

    # Benchmark sequential
    print(f"\n{'=' * 50}")
    print("Benchmarking SEQUENTIAL (baseline)...")
    print("=" * 50)
    seq_result = benchmark_sequential(bot, args.games)
    results.append(seq_result)
    print(f"  Avg time: {seq_result['avg_time_ms']:.1f}ms/move")
    print(f"  Win rate: {seq_result['win_rate']*100:.1f}%")

    # Benchmark batched versions
    for batch_size in args.batch_sizes:
        print(f"\n{'=' * 50}")
        print(f"Benchmarking BATCHED (batch_size={batch_size})...")
        print("=" * 50)
        batch_result = benchmark_batched(bot, args.games, batch_size)
        results.append(batch_result)

        speedup = seq_result["avg_time_ms"] / batch_result["avg_time_ms"] if batch_result["avg_time_ms"] > 0 else 0
        print(f"  Avg time: {batch_result['avg_time_ms']:.1f}ms/move")
        print(f"  Win rate: {batch_result['win_rate']*100:.1f}%")
        print(f"  SPEEDUP:  {speedup:.2f}x")

    # Summary table
    print("\n" + "=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print(f"{'Method':<20} | {'Time (ms)':<12} | {'Speedup':<10} | {'Win Rate':<10}")
    print("-" * 70)

    baseline_time = seq_result["avg_time_ms"]
    for r in results:
        speedup = baseline_time / r["avg_time_ms"] if r["avg_time_ms"] > 0 else 0
        speedup_str = f"{speedup:.2f}x" if r["method"] != "sequential" else "baseline"
        print(f"{r['method']:<20} | {r['avg_time_ms']:<12.1f} | {speedup_str:<10} | {r['win_rate']*100:<10.1f}%")

    print("=" * 70)

    # Key insight
    best_batch = max(
        [r for r in results if r["method"] != "sequential"],
        key=lambda r: baseline_time / r["avg_time_ms"] if r["avg_time_ms"] > 0 else 0,
    )
    best_speedup = baseline_time / best_batch["avg_time_ms"] if best_batch["avg_time_ms"] > 0 else 0

    print(f"\nKEY RESULT: Best speedup is {best_speedup:.2f}x with batch_size={best_batch['batch_size']}")

    if best_speedup > 1.5:
        print("  GPU batching is effective! Your RTX 5070 Ti is being utilized.")
    elif best_speedup > 1.1:
        print("  Moderate speedup. Consider larger batch sizes or more simulations.")
    else:
        print("  Limited speedup. The bottleneck may be elsewhere (CPU tree traversal?).")

    return 0


if __name__ == "__main__":
    sys.exit(main())
