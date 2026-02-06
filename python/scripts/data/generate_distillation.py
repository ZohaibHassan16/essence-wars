#!/usr/bin/env python3
"""Generate distillation data from Neural MCTS.

This script plays games using Neural MCTS (with greedy rollouts) and records
the MCTS policies as training targets. The resulting data can be used to
train a network that plays like MCTS without needing search at inference.

Uses batched GPU inference for ~10-20x speedup over sequential evaluation.

Usage:
    python scripts/data/generate_distillation.py \
        --model models/bc_mcts_values.pt \
        --games 1000 \
        --sims 25 \
        --batch-size 32 \
        --output data/datasets/distillation_1k.jsonl.gz
"""

from __future__ import annotations

import argparse
import gzip
import json
import sys
import time
from pathlib import Path

import torch

# Add parent to path for local development
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from essence_wars import PyGame
from essence_wars.agents.networks import AlphaZeroNetwork
from essence_wars.agents.neural_mcts import NeuralMctsBot


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate distillation data from Neural MCTS"
    )
    parser.add_argument(
        "--model",
        type=str,
        required=True,
        help="Path to trained model for MCTS prior",
    )
    parser.add_argument(
        "--games",
        type=int,
        default=1000,
        help="Number of games to generate",
    )
    parser.add_argument(
        "--sims",
        type=int,
        default=25,
        help="MCTS simulations per move",
    )
    parser.add_argument(
        "--output",
        type=str,
        default="data/datasets/distillation.jsonl.gz",
        help="Output path for dataset",
    )
    parser.add_argument(
        "--rollout-policy",
        type=str,
        default="greedy",
        choices=["random", "greedy"],
        help="Rollout policy for MCTS",
    )
    parser.add_argument(
        "--hidden-dim",
        type=int,
        default=256,
        help="Hidden dimension of the model",
    )
    parser.add_argument(
        "--num-blocks",
        type=int,
        default=4,
        help="Number of residual blocks in model",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=32,
        help="Batch size for GPU inference (higher = faster, uses more memory)",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    # Setup
    device = "cpu"  # Use CPU for data generation (more stable)
    print(f"Device: {device}")

    # Load model
    print(f"Loading model from {args.model}...")
    model = AlphaZeroNetwork(
        obs_dim=326,
        action_dim=256,
        hidden_dim=args.hidden_dim,
        num_blocks=args.num_blocks,
    )
    checkpoint = torch.load(args.model, map_location=device)
    model.load_state_dict(checkpoint["model_state_dict"])
    model.eval()

    # Create output directory
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    # Stats
    total_moves = 0
    p0_wins = 0
    p1_wins = 0
    draws = 0

    print(f"\nGenerating {args.games} games with MCTS-{args.sims} ({args.rollout_policy} rollouts)...")
    print("=" * 60)

    start_time = time.time()

    with gzip.open(output_path, "wt", encoding="utf-8") as f:
        for game_idx in range(args.games):
            game = PyGame()
            game.reset(game_idx)  # Use game index as seed for reproducibility

            # Create MCTS bots for both players
            bot_p0 = NeuralMctsBot(
                model,
                num_simulations=args.sims,
                device=device,
                use_value=False,
                rollout_policy=args.rollout_policy,
            )
            bot_p1 = NeuralMctsBot(
                model,
                num_simulations=args.sims,
                device=device,
                use_value=False,
                rollout_policy=args.rollout_policy,
            )

            game_moves = []

            while not game.is_done():
                player = game.current_player()
                bot = bot_p0 if player == 0 else bot_p1

                # Get state before move
                state = game.observe()
                mask = game.action_mask()

                # Get MCTS action and policy (batched for GPU efficiency)
                action, mcts_policy = bot.get_action_with_game_batched(
                    game, batch_size=args.batch_size
                )

                # Record move
                game_moves.append({
                    "turn": int(game.turn_number()),
                    "player": int(player),
                    "state_tensor": [float(x) for x in state],
                    "action_mask": [int(x) for x in mask],
                    "action": int(action),
                    "mcts_policy": [float(x) for x in mcts_policy],
                })

                game.step(action)
                total_moves += 1

            # Get game result
            reward_p0 = game.get_reward(0)
            if reward_p0 > 0:
                winner = 0
                p0_wins += 1
            elif reward_p0 < 0:
                winner = 1
                p1_wins += 1
            else:
                winner = -1
                draws += 1

            # Write game to file
            game_record = {
                "game_id": game_idx,
                "winner": winner,
                "num_moves": len(game_moves),
                "moves": game_moves,
            }
            f.write(json.dumps(game_record) + "\n")

            # Progress update
            if (game_idx + 1) % 100 == 0:
                elapsed = time.time() - start_time
                games_per_sec = (game_idx + 1) / elapsed
                eta = (args.games - game_idx - 1) / games_per_sec
                print(
                    f"  Game {game_idx + 1}/{args.games} | "
                    f"Moves: {total_moves:,} | "
                    f"P0: {p0_wins} P1: {p1_wins} D: {draws} | "
                    f"ETA: {eta/60:.1f}min"
                )

    elapsed = time.time() - start_time

    print("\n" + "=" * 60)
    print("Generation complete!")
    print(f"  Games:      {args.games:,}")
    print(f"  Total moves: {total_moves:,}")
    print(f"  Avg moves/game: {total_moves / args.games:.1f}")
    print(f"  P0 wins:    {p0_wins} ({p0_wins/args.games*100:.1f}%)")
    print(f"  P1 wins:    {p1_wins} ({p1_wins/args.games*100:.1f}%)")
    print(f"  Draws:      {draws} ({draws/args.games*100:.1f}%)")
    print(f"  Time:       {elapsed/60:.1f} minutes")
    print(f"  Output:     {output_path}")


if __name__ == "__main__":
    main()
