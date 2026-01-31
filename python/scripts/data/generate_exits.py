#!/usr/bin/env python3
"""
Generate Expert Iteration (ExIt) training data.

This script generates games using neural-guided MCTS, where a trained
neural network provides the policy prior and value estimation for MCTS.
The resulting games should be higher quality than vanilla MCTS games.

Uses batched GPU inference for ~10-20x speedup over sequential evaluation.

Usage:
    # Generate 1000 games using BC model with 50 sims
    uv run python python/scripts/generate_exit_data.py \
        --model models/bc_mcts_10k_best.pt \
        --model-type bc \
        --games 1000 \
        --sims 50 \
        --batch-size 32 \
        --output data/datasets/exit_iter1.jsonl.gz

    # Generate using PPO model
    uv run python python/scripts/generate_exit_data.py \
        --model experiments/ppo/20260119_151841_argentum_embedded/best_model.pt \
        --model-type ppo \
        --games 1000 \
        --sims 50 \
        --output data/datasets/exit_ppo_iter1.jsonl.gz

    # Quick test
    uv run python python/scripts/generate_exit_data.py \
        --model models/bc_mcts_10k_best.pt \
        --model-type bc \
        --games 10 \
        --sims 25 \
        --output data/datasets/exit_test.jsonl.gz
"""

import argparse
import gzip
import json
import random
import sys
import time
from datetime import datetime
from pathlib import Path

import numpy as np

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent))


def generate_exit_games(
    model_path: str,
    model_type: str,
    num_games: int,
    num_sims: int,
    output_path: str,
    device: str = "auto",
    seed: int = 42,
    batch_size: int = 32,
) -> dict:
    """
    Generate ExIt training data using neural-guided MCTS.

    Args:
        model_path: Path to model checkpoint
        model_type: "bc" or "ppo"
        num_games: Number of games to generate
        num_sims: MCTS simulations per move
        output_path: Output file path (.jsonl.gz)
        device: Device to use
        seed: Random seed
        batch_size: Batch size for GPU inference (higher = faster)

    Returns:
        Statistics dict
    """
    from essence_wars._core import PyGame
    from essence_wars.agents.neural_mcts import (
        NeuralMctsBot,
        load_bc_network,
        load_ppo_network,
    )

    # Set seeds
    random.seed(seed)
    np.random.seed(seed)

    # Load model
    print(f"Loading model from {model_path}...")
    if model_type == "ppo":
        network, obs_normalizer = load_ppo_network(model_path, device=device)
    else:
        network = load_bc_network(model_path, device=device)
        obs_normalizer = None
    print("  Model loaded")

    # Create bot
    bot = NeuralMctsBot(
        network=network,
        num_simulations=num_sims,
        c_puct=1.5,
        temperature=0.0,
        obs_normalizer=obs_normalizer,
    )

    # Get deck list (for logging)
    game = PyGame()
    deck_names = game.list_decks()
    print(f"  Available decks: {len(deck_names)}")

    # Statistics
    total_moves = 0
    total_time = 0.0
    wins_p0 = 0
    wins_p1 = 0
    draws = 0

    # Open output file
    output_path = Path(output_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    print(f"\nGenerating {num_games} games with {num_sims} sims/move...")
    print(f"Output: {output_path}")

    start_time = time.time()

    with gzip.open(output_path, "wt") as f:
        for game_idx in range(num_games):
            game_start = time.time()

            # Create game with seed (deck selection is deterministic based on seed)
            game = PyGame()
            game_seed = seed + game_idx
            game.reset(seed=game_seed)

            # Play game and record moves
            moves = []
            turn = 0

            while not game.is_done():
                player = game.current_player()
                obs = game.observe()
                mask = game.action_mask()

                # Get action and policy from neural MCTS (batched for GPU efficiency)
                action, policy = bot.get_action_with_game_batched(
                    game, batch_size=batch_size
                )

                # Record move
                moves.append({
                    "turn": turn,
                    "player": player,
                    "state_tensor": obs.tolist(),
                    "action_mask": mask.tolist(),
                    "action": int(action),
                    "mcts_policy": policy.tolist(),
                    "mcts_value": float(bot.last_root_value),
                })

                game.step(action)
                turn += 1

            # Get winner
            reward_p0 = game.get_reward(0)
            if reward_p0 > 0:
                winner = 0
                wins_p0 += 1
            elif reward_p0 < 0:
                winner = 1
                wins_p1 += 1
            else:
                winner = -1  # Draw
                draws += 1

            # Create game record
            game_record = {
                "game_id": f"exit_{game_idx}",
                "game_seed": game_seed,
                "winner": winner,
                "num_turns": turn,
                "moves": moves,
                "metadata": {
                    "model": model_path,
                    "model_type": model_type,
                    "sims": num_sims,
                    "seed": seed,
                    "game_seed": game_seed,
                },
            }

            # Write to file
            f.write(json.dumps(game_record) + "\n")

            total_moves += len(moves)
            game_time = time.time() - game_start
            total_time += game_time

            # Progress
            if (game_idx + 1) % 10 == 0 or game_idx == 0:
                elapsed = time.time() - start_time
                games_per_sec = (game_idx + 1) / elapsed
                eta = (num_games - game_idx - 1) / games_per_sec if games_per_sec > 0 else 0
                print(
                    f"  Game {game_idx + 1}/{num_games} "
                    f"({len(moves)} moves, {game_time:.1f}s) "
                    f"[{games_per_sec:.2f} games/s, ETA: {eta/60:.1f}m]"
                )

    total_elapsed = time.time() - start_time

    # Statistics
    stats = {
        "games": num_games,
        "total_moves": total_moves,
        "avg_moves_per_game": total_moves / num_games,
        "wins_p0": wins_p0,
        "wins_p1": wins_p1,
        "draws": draws,
        "win_rate_p0": wins_p0 / num_games,
        "total_time_seconds": total_elapsed,
        "games_per_second": num_games / total_elapsed,
        "avg_time_per_game": total_elapsed / num_games,
        "sims_per_move": num_sims,
        "output_path": str(output_path),
    }

    return stats


def main():
    parser = argparse.ArgumentParser(
        description="Generate Expert Iteration training data",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )

    parser.add_argument(
        "--model",
        type=str,
        required=True,
        help="Path to model checkpoint",
    )
    parser.add_argument(
        "--model-type",
        type=str,
        default="bc",
        choices=["bc", "ppo"],
        help="Type of model",
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
        default=50,
        help="MCTS simulations per move",
    )
    parser.add_argument(
        "--output",
        type=str,
        default=None,
        help="Output file path (default: auto-generated)",
    )
    parser.add_argument(
        "--device",
        type=str,
        default="auto",
        help="Device (auto, cuda, cpu)",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=42,
        help="Random seed",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=32,
        help="Batch size for GPU inference (higher = faster, uses more memory)",
    )

    args = parser.parse_args()

    # Auto-generate output path
    if args.output is None:
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        args.output = f"data/datasets/exit_{args.model_type}_{args.games}g_sims{args.sims}_{timestamp}.jsonl.gz"

    print("=" * 60)
    print("Expert Iteration Data Generation")
    print("=" * 60)
    print(f"  Model:     {args.model}")
    print(f"  Type:      {args.model_type}")
    print(f"  Games:     {args.games}")
    print(f"  Sims:      {args.sims}")
    print(f"  Output:    {args.output}")
    print(f"  Seed:      {args.seed}")
    print("=" * 60)

    # Generate data
    stats = generate_exit_games(
        model_path=args.model,
        model_type=args.model_type,
        num_games=args.games,
        num_sims=args.sims,
        output_path=args.output,
        device=args.device,
        seed=args.seed,
        batch_size=args.batch_size,
    )

    # Print summary
    print("\n" + "=" * 60)
    print("Generation Complete")
    print("=" * 60)
    print(f"  Games:           {stats['games']}")
    print(f"  Total moves:     {stats['total_moves']:,}")
    print(f"  Avg moves/game:  {stats['avg_moves_per_game']:.1f}")
    print(f"  P0 wins:         {stats['wins_p0']} ({stats['win_rate_p0']*100:.1f}%)")
    print(f"  P1 wins:         {stats['wins_p1']}")
    print(f"  Draws:           {stats['draws']}")
    print(f"  Total time:      {stats['total_time_seconds']/60:.1f} minutes")
    print(f"  Speed:           {stats['games_per_second']:.2f} games/s")
    print(f"  Output:          {stats['output_path']}")
    print("=" * 60)


if __name__ == "__main__":
    main()
